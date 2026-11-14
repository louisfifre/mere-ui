//! The window: a Win32 class and window, its messages turned into `nui_base::Event`s, and the loop that
//! draws frames only when something asks for one.

use std::collections::VecDeque;
use std::time::Instant;

use nui_base::{App, Button, Event, Vec2, observe};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{InvalidateRect, ScreenToClient, ValidateRect};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{
    AdjustWindowRectExForDpi, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
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
    }
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
        flush(state);
        settle(state);
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

/// Opens a window for `app` and runs until it closes.
pub fn run(title: &str, app: &mut dyn App) -> Result<()> {
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
        }));
        guard.state = state;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);

        // Size the client area to the start size on the screen the window landed on. The renderer does not
        // exist yet, so the WM_SIZE this sends is ignored.
        let dpi = GetDpiForWindow(hwnd);
        let scale = dpi as f32 / 96.0;
        let mut rect = RECT { left: 0, top: 0, right: (START_W * scale) as i32, bottom: (START_H * scale) as i32 };
        AdjustWindowRectExForDpi(&mut rect, WS_OVERLAPPEDWINDOW, false, WINDOW_EX_STYLE(0), dpi)?;
        SetWindowPos(hwnd, None, 0, 0, rect.right - rect.left, rect.bottom - rect.top, SWP_NOMOVE | SWP_NOZORDER)?;

        let (width, height, scale) = client_size(hwnd);
        let mut renderer = Renderer::for_window(hwnd, width, height, scale)?;
        call_app(state, |app| app.start(&mut renderer));
        (*state).renderer = Some(renderer);
        (*state).size_dirty = false;
        send(state, Event::Resized { width_px: width, height_px: height, scale });
        (*state).redraw = true;
        let _ = ShowWindow(hwnd, SW_SHOW);

        message_loop(state)
    }
}

/// Sleeps in `GetMessageW` while nothing is wanted, which costs no processor; spins through
/// `PeekMessageW` only while a frame is wanted, the frame's own `Present` being what paces it.
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
/// The client area in physical pixels, and the physical pixels per device-independent pixel.
fn client_size(hwnd: HWND) -> (u32, u32, f32) {
    let mut rect = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut rect);
        (rect.right.max(0) as u32, rect.bottom.max(0) as u32, GetDpiForWindow(hwnd) as f32 / 96.0)
    }
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
            WM_SIZE => {
                if wparam.0 as u32 != SIZE_MINIMIZED {
                    resized(state);
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
                send(state, Event::PointerMoved { pos });
                LRESULT(0)
            }
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
                let pos = point(lparam, scale());
                let button = button(msg);
                (*state).last_pos = pos;
                (*state).held[slot(button)] = true;
                // Captured, so a drag keeps reporting after the pointer leaves the window.
                SetCapture(hwnd);
                send(state, Event::PointerDown { pos, button, clicks: 1 });
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
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
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
