//! The window: a Win32 class and window, its messages turned into `nui_base::Event`s, and the loop that
//! draws frames only when something asks for one.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::Instant;

use nui_base::{App, Button, Cursor, Edge, Event, Hit, Ime, Key, Mods, Rect, Vec2, WindowRequest, observe};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
use windows::Win32::Graphics::Gdi::{InvalidateRect, ScreenToClient, ValidateRect};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{MARGINS, WM_MOUSELEAVE};
use windows::Win32::UI::HiDpi::{
    AdjustWindowRectExForDpi, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow, GetSystemMetricsForDpi, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Input::Ime::ISC_SHOWUICOMPOSITIONWINDOW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetDoubleClickTime, GetKeyState, ReleaseCapture, SetCapture, TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT, TRACKMOUSEEVENT_FLAGS, TrackMouseEvent,
    VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_LEFT,
    VK_MENU, VK_RETURN, VK_RIGHT, VK_SHIFT, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{Error, PCWSTR, Result, w};

use crate::Renderer;

/// Size of the first window, in device-independent pixels.
const START_W: f32 = 1280.0;
const START_H: f32 = 800.0;
/// Longest step handed to `App::frame`: a stall (a drag of the window, a debugger) must not make a glide jump.
const MAX_STEP: f32 = 0.1;
/// The step of a frame that follows a rest or an event-driven frame, when there is no moving frame to measure.
const NOMINAL_STEP: f32 = 1.0 / 60.0;
/// The one timer: the application's `wake_in`, or a tick while the window is hidden.
const WAKE_TIMER: usize = 1;
/// Posted by the system when the window is covered or shows again (`RegisterOcclusionStatusWindow`).
const WM_OCCLUSION: u32 = WM_APP + 1;
/// The shortest timer: below a frame a wake would only spin.
const MIN_WAKE_MS: u32 = 16;

/// What the window is asked to be: `own` draws its own title bar and borders (the application answers
/// `App::hit`), `min` is the smallest size in device-independent pixels, 0 for the system's.
#[derive(Copy, Clone, Debug)]
pub struct Frame {
    pub own: bool,
    pub min: (f32, f32),
}

/// What the loop does while no event is waiting.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Wait {
    /// Frame after frame: something moves and the window is seen.
    Spin,
    /// Sleep until this many milliseconds pass, or an event.
    Timer(u32),
    Sleep,
}

/// A moving window nobody sees never spins: what moves is not drawn.
fn next_wait(moving: bool, hidden: bool, wake: Option<f32>) -> Wait {
    if moving && !hidden {
        return Wait::Spin;
    }
    match wake {
        Some(s) => Wait::Timer(((s.max(0.0) * 1000.0).ceil() as u32).max(MIN_WAKE_MS)),
        None => Wait::Sleep,
    }
}

/// The hit-test code the system wants for a `Hit`.
fn hit_code(hit: Hit) -> u32 {
    match hit {
        Hit::Client => HTCLIENT,
        Hit::Caption => HTCAPTION,
        Hit::Resize(Edge::N) => HTTOP,
        Hit::Resize(Edge::S) => HTBOTTOM,
        Hit::Resize(Edge::E) => HTRIGHT,
        Hit::Resize(Edge::W) => HTLEFT,
        Hit::Resize(Edge::NW) => HTTOPLEFT,
        Hit::Resize(Edge::NE) => HTTOPRIGHT,
        Hit::Resize(Edge::SW) => HTBOTTOMLEFT,
        Hit::Resize(Edge::SE) => HTBOTTOMRIGHT,
    }
}

/// The system cursor for a shape. Windows has no grab hand: the four arrows are its "can be dragged".
fn idc(cursor: Cursor) -> PCWSTR {
    match cursor {
        Cursor::Arrow => IDC_ARROW,
        Cursor::Hand => IDC_HAND,
        Cursor::IBeam => IDC_IBEAM,
        Cursor::Crosshair => IDC_CROSS,
        Cursor::Grab => IDC_SIZEALL,
        Cursor::ResizeNS => IDC_SIZENS,
        Cursor::ResizeEW => IDC_SIZEWE,
        Cursor::ResizeNWSE => IDC_SIZENWSE,
        Cursor::ResizeNESW => IDC_SIZENESW,
    }
}

/// The window, for the clipboard: `OpenClipboard` ties the clipboard to a window when it is given one.
/// One window per process is all `run` makes.
static WINDOW: AtomicIsize = AtomicIsize::new(0);

pub(crate) fn clipboard_owner() -> Option<HWND> {
    match WINDOW.load(Ordering::Relaxed) {
        0 => None,
        h => Some(HWND(h as *mut std::ffi::c_void)),
    }
}

/// What the window procedure needs, reached through the window's user-data slot. The procedure is called
/// by the system, so it cannot be given arguments: this is how it finds the application.
///
/// The system calls back into the procedure from inside our own calls (`SetWindowPos`, `DefWindowProcW`,
/// `Present` can all send `WM_SIZE` and friends synchronously). So nothing here is ever borrowed across such
/// a call: the procedure keeps a `*mut State`, and every function below takes a short-lived `&mut` to the
/// one field it needs, ending before the next call out.
struct State {
    /// The caller's application, lifetime erased: `run` outlives every message it can receive.
    app: *mut dyn App,
    hwnd: HWND,
    /// Taken out while a frame or a resize runs on it, so a re-entrant message cannot alias it.
    renderer: Option<Renderer>,
    last_frame: Instant,
    /// A frame was asked for by an event.
    redraw: bool,
    /// The last frame said something still moves.
    moving: bool,
    /// The application is running (`start`, `event`, `frame`): events that arrive meanwhile wait in `queued`
    /// instead of reaching it a second time.
    in_app: bool,
    queued: VecDeque<Event>,
    /// A size message came while the renderer was out; it is applied once the renderer is back.
    size_dirty: bool,
    /// The last attempt to rebuild a lost renderer failed; the next paint tries again.
    rebuild: bool,
    /// Which buttons went down in this window and have not come up, by `slot`.
    held: [bool; 3],
    last_pos: Vec2,
    /// The last button press: button, message time (ms), physical position and the count it reached. The
    /// class has no CS_DBLCLKS, so the system never merges presses and the count is ours to keep.
    last_down: Option<(Button, u32, i32, i32, u8)>,
    /// The first half of a character outside the Basic Multilingual Plane, which WM_CHAR delivers as two
    /// messages.
    high: Option<u16>,
    /// The light/dark setting last read or sent, so a change message that changes nothing sends nothing.
    dark: bool,
    frame: Frame,
    /// `start` has run: events may reach the application.
    started: bool,
    maximized: bool,
    /// A tick said a frame is wanted while the window was hidden; it is drawn when the window shows.
    due: bool,
    /// The pointer is over the client area, so a cursor change can be applied without a move.
    inside: bool,
    /// A leave notification is asked for, for the client area and for the frame.
    tracking: bool,
    tracking_nc: bool,
    /// The shape last set, so the next frame sets one only when it differs.
    cursor: Cursor,
    /// The input method is associated with the window: a text field has the keyboard.
    ime_on: bool,
    /// The spot last given to the input method, in dip.
    ime_spot: Option<Rect>,
    /// A composition is open (between the start and end messages).
    composing: bool,
}

impl State {
    fn busy(&self) -> bool {
        self.redraw || self.moving
    }
}

fn slot(button: Button) -> usize {
    match button {
        Button::Left => 0,
        Button::Right => 1,
        Button::Middle => 2,
    }
}

const BUTTONS: [Button; 3] = [Button::Left, Button::Right, Button::Middle];

/// Runs `f` on the application with `in_app` raised. No reference to `State` lives across `f`.
unsafe fn call_app<R>(state: *mut State, f: impl FnOnce(&mut dyn App) -> R) -> R {
    unsafe {
        (*state).in_app = true;
        // SAFETY: see `State::app`. `in_app` keeps a re-entrant message from making a second `&mut`.
        let out = f(&mut *(*state).app);
        (*state).in_app = false;
        out
    }
}

/// Hands every waiting event to the application, one at a time.
unsafe fn flush(state: *mut State) {
    unsafe {
        while !(*state).in_app {
            let Some(event) = (*state).queued.pop_front() else { break };
            if call_app(state, |app| app.event(event)) {
                (*state).redraw = true;
            }
        }
        pump(state);
    }
}

/// Does what the application asked of its window, until it asks no more. Each can send messages back into
/// the procedure at once (a minimise sends `WM_SIZE`), which is safe: nothing is borrowed here.
unsafe fn pump(state: *mut State) {
    unsafe {
        while !(*state).in_app {
            let Some(request) = call_app(state, |app| app.take_request()) else { break };
            let hwnd = (*state).hwnd;
            match request {
                WindowRequest::Minimize => {
                    let _ = ShowWindow(hwnd, SW_MINIMIZE);
                }
                WindowRequest::ToggleMaximize => {
                    let _ = ShowWindow(hwnd, if IsZoomed(hwnd).as_bool() { SW_RESTORE } else { SW_MAXIMIZE });
                }
                // The ordinary close path: `CloseRequested` is sent, then the window is destroyed.
                WindowRequest::Close => {
                    let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
                }
                WindowRequest::Title(title) => {
                    let wide: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
                    let _ = SetWindowTextW(hwnd, PCWSTR(wide.as_ptr()));
                }
            }
        }
    }
}

/// Not drawn at all: minimised, or covered (the last present said so).
unsafe fn hidden(state: *mut State) -> bool {
    unsafe { IsIconic((*state).hwnd).as_bool() || (*state).renderer.as_ref().is_some_and(Renderer::occluded) }
}

/// Sets the timer the application's `wake_in` asks for, or removes it. A moving, visible window needs none:
/// it draws frame after frame.
unsafe fn rearm(state: *mut State) {
    unsafe {
        if (*state).in_app {
            return;
        }
        let hwnd = (*state).hwnd;
        let wake = call_app(state, |app| app.wake_in());
        match next_wait((*state).moving, hidden(state), wake) {
            Wait::Timer(ms) => {
                SetTimer(Some(hwnd), WAKE_TIMER, ms, None);
            }
            Wait::Spin | Wait::Sleep => {
                let _ = KillTimer(Some(hwnd), WAKE_TIMER);
            }
        }
    }
}

/// Sets the system cursor again when the application's shape changed while the pointer stayed still.
unsafe fn sync_cursor(state: *mut State) {
    unsafe {
        if !(*state).inside || (*state).in_app {
            return;
        }
        let wanted = call_app(state, |app| app.cursor());
        if wanted != (*state).cursor {
            (*state).cursor = wanted;
            if let Ok(c) = LoadCursorW(None, idc(wanted)) {
                SetCursor(Some(c));
            }
        }
    }
}

/// Gives the input method the caret's spot as of the last frame: on while a text field has the keyboard, off
/// otherwise (a number field neither). The spot is placed again whenever it moves, which is on every typed char,
/// so only the switch is logged.
unsafe fn sync_ime(state: *mut State) {
    unsafe {
        if (*state).in_app {
            return;
        }
        let spot = call_app(state, |app| app.ime_spot());
        let hwnd = (*state).hwnd;
        match (spot, (*state).ime_on) {
            (None, false) => return,
            (None, true) => {
                // A composition is cancelled before the input method is taken away, or it would be left open
                // with no window to finish in.
                if (*state).composing {
                    crate::ime::cancel(hwnd);
                }
                crate::ime::set_enabled(hwnd, false);
                (*state).ime_on = false;
                observe!(Debug, "ime off");
            }
            (Some(_), false) => {
                crate::ime::set_enabled(hwnd, true);
                (*state).ime_on = true;
                observe!(Debug, "ime on");
            }
            (Some(s), true) => {
                if (*state).ime_spot == Some(s) {
                    return;
                }
            }
        }
        (*state).ime_spot = spot;
        if let Some(s) = spot {
            let px = spot_px(s, GetDpiForWindow(hwnd) as f32 / 96.0);
            crate::ime::place(hwnd, px);
        }
    }
}

/// A spot in dip as a rectangle of client pixels: at least one pixel wide.
fn spot_px(s: Rect, scale: f32) -> RECT {
    let left = (s.x * scale).round() as i32;
    RECT { left, top: (s.y * scale).round() as i32, right: left + ((s.w * scale).round() as i32).max(1), bottom: ((s.y + s.h) * scale).round() as i32 }
}

unsafe fn send(state: *mut State, event: Event) {
    unsafe {
        (*state).queued.push_back(event);
        flush(state);
    }
}

unsafe fn draw(state: *mut State) {
    unsafe {
        if (*state).in_app {
            return;
        }
        if (*state).rebuild {
            recover(state);
        }
        // A covered window may show again with no message yet: ask the system before giving up on a frame.
        if let Some(r) = (*state).renderer.as_mut() {
            if r.occluded() && !IsIconic((*state).hwnd).as_bool() {
                r.still_occluded();
            }
        }
        if hidden(state) {
            // Nothing to see, so nothing to draw; the wake timer keeps ticking the application, and showing
            // the window again redraws.
            (*state).redraw = false;
            (*state).moving = false;
            rearm(state);
            return;
        }
        let Some(mut renderer) = (*state).renderer.take() else {
            // Nothing to draw on: staying "busy" would spin the loop.
            (*state).redraw = false;
            (*state).moving = false;
            return;
        };
        let now = Instant::now();
        // After a rest there is no previous frame to measure from: the time since `last_frame` is the rest
        // itself, which would make a glide pop forward. Only a frame that follows a moving one is measured.
        let seconds = if (*state).moving { now.duration_since((*state).last_frame).as_secs_f32().min(MAX_STEP) } else { NOMINAL_STEP };
        (*state).last_frame = now;
        (*state).redraw = false;
        let moving = call_app(state, |app| app.frame(&mut renderer, seconds));
        (*state).moving = moving;
        (*state).renderer = Some(renderer);
        (*state).due = false;
        flush(state);
        settle(state);
        sync_cursor(state);
        sync_ime(state);
        rearm(state);
    }
}

/// What a frame or a resize may have left to do: a size message that came while the renderer was out, or a
/// device that went away.
unsafe fn settle(state: *mut State) {
    unsafe {
        if (*state).size_dirty && (*state).renderer.is_some() {
            (*state).size_dirty = false;
            resized(state);
        }
        recover(state);
    }
}

/// Rebuilds the renderer when the device is lost: meshes and images lived on it, so the application's
/// `start` runs again on the new one.
unsafe fn recover(state: *mut State) {
    unsafe {
        let lost = (*state).renderer.as_ref().is_some_and(Renderer::lost);
        if !lost && !(*state).rebuild {
            return;
        }
        let hwnd = (*state).hwnd;
        let (width, height, scale) = client_size(hwnd);
        // The old swap chain still owns the window: it must be released before a new one can be made.
        (*state).renderer = None;
        match Renderer::for_window(hwnd, width, height, scale) {
            Ok(mut renderer) => {
                observe!(Warn, "graphics device lost: renderer rebuilt at {width}x{height}, application restarted");
                renderer.watch_occlusion(hwnd, WM_OCCLUSION);
                call_app(state, |app| app.start(&mut renderer));
                (*state).rebuild = false;
                (*state).renderer = Some(renderer);
                send(state, Event::Resized { width_px: width, height_px: height, scale });
                (*state).redraw = true;
                if (*state).size_dirty {
                    (*state).size_dirty = false;
                    resized(state);
                }
            }
            Err(e) => {
                observe!(Error, "graphics device lost and the renderer could not be rebuilt: {e}");
                (*state).rebuild = true;
                (*state).redraw = false;
                (*state).moving = false;
            }
        }
    }
}

/// Sends `PointerUp` for every button still held and lets go of the capture. Used when the capture is taken
/// away (Alt+Tab during a drag) so the application never keeps a drag that no up will end.
unsafe fn release_all(state: *mut State) {
    unsafe {
        let held = std::mem::replace(&mut (*state).held, [false; 3]);
        if !held.contains(&true) {
            return;
        }
        // Held is already empty, so the WM_CAPTURECHANGED this triggers finds nothing to do.
        let _ = ReleaseCapture();
        let pos = (*state).last_pos;
        for (button, was_held) in BUTTONS.into_iter().zip(held) {
            if was_held {
                send(state, Event::PointerUp { pos, button });
            }
        }
    }
}

/// Owns everything `run` creates, so every exit path, `?` included, gives it back.
struct Guard {
    instance: HINSTANCE,
    registered: bool,
    hwnd: Option<HWND>,
    state: *mut State,
}

impl Drop for Guard {
    fn drop(&mut self) {
        unsafe {
            if let Some(hwnd) = self.hwnd {
                // First, so no message reaches the State while it is being freed.
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            // The renderer goes before the window and the class: it holds the window's swap chain.
            if !self.state.is_null() {
                drop(Box::from_raw(self.state));
            }
            if let Some(hwnd) = self.hwnd {
                if IsWindow(Some(hwnd)).as_bool() {
                    let _ = DestroyWindow(hwnd);
                }
            }
            if self.registered {
                let _ = UnregisterClassW(w!("nui_window"), Some(self.instance));
            }
        }
    }
}

/// Opens a window with the system's own frame for `app` and runs until it closes.
pub fn run(title: &str, app: &mut dyn App) -> Result<()> {
    run_with(title, app, Frame { own: false, min: (0.0, 0.0) })
}

/// Opens a window for `app` and runs until it closes. With `frame.own` the window keeps the system's
/// overlapped style (snapping, the system menu, Win+arrows and the animations come with it) but has no
/// title bar or border of its own: the application draws them and answers `App::hit`.
pub fn run_with(title: &str, app: &mut dyn App, frame: Frame) -> Result<()> {
    unsafe {
        // Per monitor, version 2: the window gets the real pixel size of its screen and is told when it
        // moves to another one, instead of being stretched blurry by the system. Fails harmlessly when a
        // manifest or an earlier call already chose.
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            lpszClassName: w!("nui_window"),
            ..Default::default()
        };
        if RegisterClassExW(&class) == 0 {
            return Err(Error::from_thread());
        }
        let mut guard = Guard { instance, registered: true, hwnd: None, state: std::ptr::null_mut() };

        let wide: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("nui_window"),
            PCWSTR(wide.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(instance),
            None,
        )?;
        guard.hwnd = Some(hwnd);
        WINDOW.store(hwnd.0 as isize, Ordering::Relaxed);

        // The erasure is sound because the guard, which frees the state, drops before this function returns
        // and so before `app` can be used again by the caller.
        let app: *mut dyn App = std::mem::transmute::<&mut dyn App, &'static mut dyn App>(app);
        // Reached from here and from the window procedure alike, so it is a raw pointer: two `&mut` to it
        // alive at once would be undefined behaviour even on one thread.
        let state = Box::into_raw(Box::new(State {
            app,
            hwnd,
            renderer: None,
            last_frame: Instant::now(),
            redraw: false,
            moving: false,
            in_app: false,
            queued: VecDeque::new(),
            size_dirty: false,
            rebuild: false,
            held: [false; 3],
            last_pos: Vec2::new(0.0, 0.0),
            last_down: None,
            high: None,
            dark: crate::scheme::system_dark(),
            frame,
            started: false,
            maximized: false,
            due: false,
            inside: false,
            tracking: false,
            tracking_nc: false,
            cursor: Cursor::Arrow,
            ime_on: true,
            ime_spot: None,
            composing: false,
        }));
        guard.state = state;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
        // No field means no input method: it is switched on only while a text field has the keyboard.
        crate::ime::set_enabled(hwnd, false);
        (*state).ime_on = false;

        // Size the client area to the start size on the screen the window landed on. The renderer does not
        // exist yet, so the WM_SIZE this sends is ignored.
        let dpi = GetDpiForWindow(hwnd);
        let scale = dpi as f32 / 96.0;
        let mut rect = RECT { left: 0, top: 0, right: (START_W * scale) as i32, bottom: (START_H * scale) as i32 };
        if frame.own {
            // Without a frame of the system's the window rectangle is the client area.
            // WM_NCCALCSIZE must be asked again for the change of frame to count.
            SetWindowPos(hwnd, None, 0, 0, 0, 0, SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER)?;
            // One pixel of extended frame keeps the system's shadow around a window with no border.
            let margins = MARGINS { cxLeftWidth: 1, cxRightWidth: 1, cyTopHeight: 1, cyBottomHeight: 1 };
            if let Err(e) = DwmExtendFrameIntoClientArea(hwnd, &margins) {
                observe!(Warn, "frame shadow not kept: {e}");
            }
        } else {
            AdjustWindowRectExForDpi(&mut rect, WS_OVERLAPPEDWINDOW, false, WINDOW_EX_STYLE(0), dpi)?;
        }
        SetWindowPos(hwnd, None, 0, 0, rect.right - rect.left, rect.bottom - rect.top, SWP_NOMOVE | SWP_NOZORDER)?;

        let (width, height, scale) = client_size(hwnd);
        let mut renderer = Renderer::for_window(hwnd, width, height, scale)?;
        renderer.watch_occlusion(hwnd, WM_OCCLUSION);
        call_app(state, |app| app.start(&mut renderer));
        (*state).renderer = Some(renderer);
        (*state).size_dirty = false;
        (*state).started = true;
        send(state, Event::Resized { width_px: width, height_px: height, scale });
        send(state, Event::Window { maximized: (*state).maximized });
        (*state).redraw = true;
        let _ = ShowWindow(hwnd, SW_SHOW);

        message_loop(state)
    }
}

/// Sleeps in `GetMessageW` while nothing is wanted, which costs no processor (a wake timer ends the sleep
/// when the application asked for one); spins through `PeekMessageW` only while a frame is wanted, the
/// frame's own `Present` being what paces it.
unsafe fn message_loop(state: *mut State) -> Result<()> {
    let mut msg = MSG::default();
    loop {
        if unsafe { (*state).busy() } {
            while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.as_bool() {
                if msg.message == WM_QUIT {
                    return Ok(());
                }
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        } else {
            match unsafe { GetMessageW(&mut msg, None, 0, 0) }.0 {
                0 => return Ok(()),
                -1 => return Err(Error::from_thread()),
                _ => unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                },
            }
        }
        if unsafe { (*state).busy() } {
            unsafe { draw(state) };
        }
    }
}

/// The smallest window the frame allows, in physical pixels of the window rectangle.
fn min_track(hwnd: HWND, frame: Frame) -> Option<(i32, i32)> {
    if frame.min == (0.0, 0.0) {
        return None;
    }
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    let s = dpi as f32 / 96.0;
    let mut rect = RECT { left: 0, top: 0, right: (frame.min.0 * s).ceil() as i32, bottom: (frame.min.1 * s).ceil() as i32 };
    if !frame.own {
        // The minimum is of the client area; the system's frame comes on top.
        let _ = unsafe { AdjustWindowRectExForDpi(&mut rect, WS_OVERLAPPEDWINDOW, false, WINDOW_EX_STYLE(0), dpi) };
    }
    Some((rect.right - rect.left, rect.bottom - rect.top))
}
/// The client area in physical pixels, and the physical pixels per device-independent pixel.
fn client_size(hwnd: HWND) -> (u32, u32, f32) {
    let mut rect = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut rect);
        (rect.right.max(0) as u32, rect.bottom.max(0) as u32, GetDpiForWindow(hwnd) as f32 / 96.0)
    }
}

/// Asks the system for a leave message; false when it refuses (then the next move asks again).
fn track(hwnd: HWND, flags: TRACKMOUSEEVENT_FLAGS) -> bool {
    let mut request = TRACKMOUSEEVENT { cbSize: size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: flags, hwndTrack: hwnd, dwHoverTime: 0 };
    unsafe { TrackMouseEvent(&mut request) }.is_ok()
}

/// The two signed 16-bit halves of an `LPARAM`.
fn halves(lparam: LPARAM) -> (i32, i32) {
    ((lparam.0 & 0xffff) as u16 as i16 as i32, ((lparam.0 >> 16) & 0xffff) as u16 as i16 as i32)
}

/// A pointer position in device-independent pixels.
fn point(lparam: LPARAM, scale: f32) -> Vec2 {
    let (x, y) = halves(lparam);
    Vec2::new(x as f32 / scale, y as f32 / scale)
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut State;
    if state.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
    // `state` stays a raw pointer: set by `run` for the window's life and cleared before it is freed. No
    // reference to it is held across a call that can re-enter this procedure.
    let scale = || unsafe { GetDpiForWindow(hwnd) } as f32 / 96.0;

    unsafe {
        match msg {
            WM_NCCALCSIZE if (*state).frame.own => {
                // No border and no title bar: the client area is the whole window. A maximised window
                // overhangs the screen by its frame, so it is pulled back in.
                if wparam.0 != 0 && IsZoomed(hwnd).as_bool() {
                    let params = &mut *(lparam.0 as *mut NCCALCSIZE_PARAMS);
                    let dpi = GetDpiForWindow(hwnd);
                    let pad = GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
                    let (dx, dy) = (GetSystemMetricsForDpi(SM_CXFRAME, dpi) + pad, GetSystemMetricsForDpi(SM_CYFRAME, dpi) + pad);
                    let r = &mut params.rgrc[0];
                    r.left += dx;
                    r.right -= dx;
                    r.top += dy;
                    r.bottom -= dy;
                }
                LRESULT(0)
            }
            WM_NCHITTEST if (*state).frame.own => {
                if (*state).in_app {
                    return LRESULT(HTCLIENT as isize);
                }
                let (x, y) = halves(lparam);
                let mut p = POINT { x, y };
                let _ = ScreenToClient(hwnd, &mut p);
                let s = scale();
                let hit = call_app(state, |app| app.hit(Vec2::new(p.x as f32 / s, p.y as f32 / s)));
                LRESULT(hit_code(hit) as isize)
            }
            WM_SETCURSOR => {
                if (lparam.0 & 0xffff) as u32 == HTCLIENT {
                    let shape = if (*state).in_app { (*state).cursor } else { call_app(state, |app| app.cursor()) };
                    (*state).cursor = shape;
                    if let Ok(c) = LoadCursorW(None, idc(shape)) {
                        SetCursor(Some(c));
                    }
                    return LRESULT(1);
                }
                // The sizing arrows of the edges, and the arrow elsewhere.
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_GETMINMAXINFO => {
                if let Some((w, h)) = min_track(hwnd, (*state).frame) {
                    let info = &mut *(lparam.0 as *mut MINMAXINFO);
                    info.ptMinTrackSize.x = w;
                    info.ptMinTrackSize.y = h;
                    return LRESULT(0);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_SIZE => {
                let kind = wparam.0 as u32;
                if kind != SIZE_MINIMIZED {
                    if kind == SIZE_MAXIMIZED || kind == SIZE_RESTORED {
                        let now = kind == SIZE_MAXIMIZED;
                        if now != (*state).maximized {
                            (*state).maximized = now;
                            if (*state).started {
                                send(state, Event::Window { maximized: now });
                            }
                        }
                    }
                    // Shown again after a minimise: what the application missed is drawn now.
                    (*state).redraw = true;
                    resized(state);
                }
                LRESULT(0)
            }
            WM_TIMER if wparam.0 == WAKE_TIMER => {
                let _ = KillTimer(Some(hwnd), WAKE_TIMER);
                if hidden(state) {
                    // Nothing is drawn, but a run goes on and its end is told in the title.
                    if call_app(state, |app| app.tick()) {
                        (*state).due = true;
                    }
                    pump(state);
                    rearm(state);
                } else {
                    (*state).redraw = true;
                }
                LRESULT(0)
            }
            WM_OCCLUSION => {
                let seen = (*state).renderer.as_mut().is_some_and(|r| !r.still_occluded());
                if seen && !IsIconic(hwnd).as_bool() {
                    (*state).redraw = true;
                }
                LRESULT(0)
            }
            WM_DPICHANGED => {
                // The system proposes a rectangle that keeps the window the same size in device-independent
                // pixels on the new screen; taking it is what stops it jumping in size.
                let suggested = *(lparam.0 as *const RECT);
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    suggested.left,
                    suggested.top,
                    suggested.right - suggested.left,
                    suggested.bottom - suggested.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
                // A change of dpi can leave the pixel size alone, so WM_SIZE may not follow.
                resized(state);
                LRESULT(0)
            }
            WM_PAINT => {
                draw(state);
                let _ = ValidateRect(Some(hwnd), None);
                LRESULT(0)
            }
            // The frame covers every pixel: erasing first would only flicker.
            WM_ERASEBKGND => LRESULT(1),
            WM_MOUSEMOVE => {
                let pos = point(lparam, scale());
                (*state).last_pos = pos;
                (*state).inside = true;
                if !(*state).tracking {
                    (*state).tracking = track(hwnd, TME_LEAVE);
                }
                send(state, Event::PointerMoved { pos });
                LRESULT(0)
            }
            // Over the frame (the drag space of the bar): the application is told, so hover clears there.
            WM_NCMOUSEMOVE => {
                let (x, y) = halves(lparam);
                let mut p = POINT { x, y };
                let _ = ScreenToClient(hwnd, &mut p);
                let s = scale();
                let pos = Vec2::new(p.x as f32 / s, p.y as f32 / s);
                (*state).last_pos = pos;
                (*state).inside = false;
                if !(*state).tracking_nc {
                    (*state).tracking_nc = track(hwnd, TME_LEAVE | TME_NONCLIENT);
                }
                send(state, Event::PointerMoved { pos });
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_MOUSELEAVE => {
                (*state).tracking = false;
                (*state).inside = false;
                send(state, Event::PointerLeft);
                LRESULT(0)
            }
            WM_NCMOUSELEAVE => {
                (*state).tracking_nc = false;
                send(state, Event::PointerLeft);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
                let pos = point(lparam, scale());
                let button = button(msg);
                (*state).last_pos = pos;
                (*state).held[slot(button)] = true;
                // Captured, so a drag keeps reporting after the pointer leaves the window.
                SetCapture(hwnd);
                let (x, y) = halves(lparam);
                let clicks = count_click(state, button, GetMessageTime() as u32, x, y);
                send(state, Event::PointerDown { pos, button, clicks });
                LRESULT(0)
            }
            WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => {
                let pos = point(lparam, scale());
                let button = button(msg);
                (*state).last_pos = pos;
                (*state).held[slot(button)] = false;
                // Only the last button up lets go: releasing right while left is held must not end the drag.
                if !(*state).held.contains(&true) {
                    let _ = ReleaseCapture();
                }
                send(state, Event::PointerUp { pos, button });
                LRESULT(0)
            }
            // The capture went to another window, or the system cancelled the mode (Alt+Tab, a dialog): no
            // up will come for the buttons still held.
            WM_CAPTURECHANGED | WM_CANCELMODE => {
                release_all(state);
                if msg == WM_CAPTURECHANGED { LRESULT(0) } else { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
            WM_MOUSEWHEEL => {
                // Unlike the other mouse messages, the wheel reports the position on the screen.
                let (x, y) = halves(lparam);
                let mut p = POINT { x, y };
                let _ = ScreenToClient(hwnd, &mut p);
                let s = scale();
                // 120 is one notch; away from the user is positive.
                let delta = ((wparam.0 >> 16) & 0xffff) as u16 as i16 as f32 / 120.0;
                send(state, Event::Wheel { pos: Vec2::new(p.x as f32 / s, p.y as f32 / s), delta });
                LRESULT(0)
            }
            WM_CLOSE => {
                send(state, Event::CloseRequested);
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_KEYDOWN => {
                let down = |k: VIRTUAL_KEY| GetKeyState(k.0 as i32) < 0;
                let (ctrl, shift, alt) = (down(VK_CONTROL), down(VK_SHIFT), down(VK_MENU));
                match key_of(wparam.0 as u16, ctrl && !alt) {
                    Some(key) => {
                        send(state, Event::Key { key, mods: Mods { ctrl, shift } });
                        LRESULT(0)
                    }
                    None => DefWindowProcW(hwnd, msg, wparam, lparam),
                }
            }
            // We draw the composition ourselves; the system still draws the candidate list.
            WM_IME_SETCONTEXT => {
                let lparam = if wparam.0 != 0 { LPARAM(lparam.0 & !(ISC_SHOWUICOMPOSITIONWINDOW as isize)) } else { lparam };
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_IME_STARTCOMPOSITION => {
                (*state).composing = true;
                // Some input methods reset the place at the start of a composition.
                if let Some(s) = (*state).ime_spot {
                    crate::ime::place(hwnd, spot_px(s, scale()));
                }
                send(state, Event::Ime(Ime::Start));
                // Not DefWindowProcW: it would open the system's own composition window.
                LRESULT(0)
            }
            WM_IME_COMPOSITION => {
                for ime in crate::ime::on_composition(hwnd, lparam.0) {
                    send(state, Event::Ime(ime));
                }
                // Never DefWindowProcW: it turns the result string into WM_IME_CHAR and then WM_CHAR, which
                // would type the committed text a second time.
                LRESULT(0)
            }
            WM_IME_ENDCOMPOSITION => {
                (*state).composing = false;
                send(state, Event::Ime(Ime::End));
                LRESULT(0)
            }
            WM_CHAR => {
                if let Some(ch) = char_of(state, wparam.0 as u16) {
                    send(state, Event::Char { ch });
                }
                LRESULT(0)
            }
            WM_SETTINGCHANGE => {
                // lParam names what changed, as a wide string, or is null when the sender gave none.
                if lparam.0 != 0 {
                    let mut len = 0;
                    let name = lparam.0 as *const u16;
                    while *name.add(len) != 0 {
                        len += 1;
                    }
                    if crate::scheme::is_scheme_change(std::slice::from_raw_parts(name, len)) {
                        let dark = crate::scheme::system_dark();
                        if dark != (*state).dark {
                            (*state).dark = dark;
                            send(state, Event::Scheme { dark });
                        }
                    }
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_DESTROY => {
                WINDOW.store(0, Ordering::Relaxed);
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// The count a press reaches: one more than the last press of the same button when it came within the
/// system's double-click time and distance, else 1. Only the left button counts up.
unsafe fn count_click(state: *mut State, button: Button, time: u32, x: i32, y: i32) -> u8 {
    unsafe {
        let count = match (*state).last_down {
            Some((b, t, lx, ly, n))
                if button == Button::Left
                    && b == button
                    && time.wrapping_sub(t) <= GetDoubleClickTime()
                    && (x - lx).abs() <= GetSystemMetrics(SM_CXDOUBLECLK) / 2
                    && (y - ly).abs() <= GetSystemMetrics(SM_CYDOUBLECLK) / 2 =>
            {
                (n + 1).min(3)
            }
            _ => 1,
        };
        (*state).last_down = Some((button, time, x, y, count));
        count
    }
}

/// The key a virtual-key code stands for, if the interface acts on it. The letters count only as Ctrl
/// shortcuts: plain typing reaches the application as `WM_CHAR`.
fn key_of(vk: u16, shortcut: bool) -> Option<Key> {
    Some(match VIRTUAL_KEY(vk) {
        VK_LEFT => Key::Left,
        VK_RIGHT => Key::Right,
        VK_UP => Key::Up,
        VK_DOWN => Key::Down,
        VK_HOME => Key::Home,
        VK_END => Key::End,
        VK_BACK => Key::Backspace,
        VK_DELETE => Key::Delete,
        VK_RETURN => Key::Enter,
        VK_ESCAPE => Key::Escape,
        VK_TAB => Key::Tab,
        // VK_PROCESSKEY (0xE5): the input method owns this key, whatever physical key it was.
        VIRTUAL_KEY(0xE5) => return None,
        // Letters have no VK_ names: they are their capital's ASCII code.
        VIRTUAL_KEY(0x41) if shortcut => Key::A,
        VIRTUAL_KEY(0x43) if shortcut => Key::C,
        VIRTUAL_KEY(0x56) if shortcut => Key::V,
        VIRTUAL_KEY(0x58) if shortcut => Key::X,
        VIRTUAL_KEY(0x5A) if shortcut => Key::Z,
        _ => return None,
    })
}

/// The character a `WM_CHAR` unit completes: a high surrogate waits for its low half, and control
/// characters (Ctrl+letter, Backspace, Tab, Enter, Ctrl+Backspace's 0x7F) are dropped because the keys
/// that made them travel as `Event::Key`.
unsafe fn char_of(state: *mut State, unit: u16) -> Option<char> {
    unsafe {
        if (0xD800..=0xDBFF).contains(&unit) {
            (*state).high = Some(unit);
            return None;
        }
        let high = (*state).high.take();
        let ch = match high {
            Some(h) if (0xDC00..=0xDFFF).contains(&unit) => char::decode_utf16([h, unit]).next()?.ok()?,
            _ => char::from_u32(unit as u32)?,
        };
        (ch >= ' ' && ch != '\x7f').then_some(ch)
    }
}

fn button(msg: u32) -> Button {
    match msg {
        WM_LBUTTONDOWN | WM_LBUTTONUP => Button::Left,
        WM_RBUTTONDOWN | WM_RBUTTONUP => Button::Right,
        _ => Button::Middle,
    }
}

/// Brings the swap chain and the application to the window's current size and dpi.
unsafe fn resized(state: *mut State) {
    unsafe {
        let hwnd = (*state).hwnd;
        // Out while a frame or another resize runs on it: remember, and `settle` applies it afterwards.
        let Some(mut renderer) = (*state).renderer.take() else {
            (*state).size_dirty = true;
            return;
        };
        let (width, height, scale) = client_size(hwnd);
        let result = renderer.resize(width, height, scale);
        (*state).renderer = Some(renderer);
        if let Err(e) = result {
            observe!(Error, "resize to {width}x{height} failed: {e}");
            recover(state);
            return;
        }
        send(state, Event::Resized { width_px: width, height_px: height, scale });
        // A new size shows nothing until redrawn; WM_PAINT draws it even inside the system's resize loop.
        (*state).redraw = true;
        let _ = InvalidateRect(Some(hwnd), None, false);
        if (*state).size_dirty && (*state).renderer.is_some() {
            (*state).size_dirty = false;
            resized(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_input_methods_process_key_is_not_a_key_of_ours() {
        assert_eq!(key_of(0xE5, false), None);
        assert_eq!(key_of(0xE5, true), None);
    }

    #[test]
    fn hit_codes_are_the_systems() {
        let cases = [
            (Hit::Client, HTCLIENT),
            (Hit::Caption, HTCAPTION),
            (Hit::Resize(Edge::N), HTTOP),
            (Hit::Resize(Edge::S), HTBOTTOM),
            (Hit::Resize(Edge::E), HTRIGHT),
            (Hit::Resize(Edge::W), HTLEFT),
            (Hit::Resize(Edge::NW), HTTOPLEFT),
            (Hit::Resize(Edge::NE), HTTOPRIGHT),
            (Hit::Resize(Edge::SW), HTBOTTOMLEFT),
            (Hit::Resize(Edge::SE), HTBOTTOMRIGHT),
        ];
        for (hit, code) in cases {
            assert_eq!(hit_code(hit), code, "{hit:?}");
        }
        // The numbers the system documents, so a wrong constant cannot hide behind its own name.
        assert_eq!([HTCLIENT, HTCAPTION, HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT, HTTOPRIGHT, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT], [1, 2, 10, 11, 12, 13, 14, 15, 16, 17]);
    }

    #[test]
    fn every_cursor_has_its_system_shape() {
        let cases = [
            (Cursor::Arrow, IDC_ARROW),
            (Cursor::Hand, IDC_HAND),
            (Cursor::IBeam, IDC_IBEAM),
            (Cursor::Crosshair, IDC_CROSS),
            (Cursor::Grab, IDC_SIZEALL),
            (Cursor::ResizeNS, IDC_SIZENS),
            (Cursor::ResizeEW, IDC_SIZEWE),
            (Cursor::ResizeNWSE, IDC_SIZENWSE),
            (Cursor::ResizeNESW, IDC_SIZENESW),
        ];
        for (cursor, want) in cases {
            assert_eq!(idc(cursor), want, "{cursor:?}");
            // The shape exists on this system.
            assert!(unsafe { LoadCursorW(None, idc(cursor)) }.is_ok(), "{cursor:?}");
        }
    }

    #[test]
    fn the_loop_never_spins_on_a_window_nobody_sees() {
        assert_eq!(next_wait(true, false, None), Wait::Spin);
        assert_eq!(next_wait(true, false, Some(0.5)), Wait::Spin);
        assert_eq!(next_wait(true, true, None), Wait::Sleep);
        assert_eq!(next_wait(true, true, Some(1.0)), Wait::Timer(1000));
        assert_eq!(next_wait(false, false, Some(0.1)), Wait::Timer(100));
        assert_eq!(next_wait(false, false, Some(0.0)), Wait::Timer(MIN_WAKE_MS));
        assert_eq!(next_wait(false, true, Some(0.001)), Wait::Timer(MIN_WAKE_MS));
        assert_eq!(next_wait(false, false, None), Wait::Sleep);
        assert_eq!(next_wait(false, true, None), Wait::Sleep);
    }
}