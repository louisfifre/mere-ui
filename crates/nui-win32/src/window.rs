//! The window: a Win32 class and window, its messages turned into `nui_base::Event`s, and the loop that
//! draws frames only when something asks for one.

use std::time::Instant;

use nui_base::{App, Button, Event, Vec2, observe};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
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

/// What the window procedure needs, reached through the window's user-data slot. The procedure is called
/// by the system, so it cannot be given arguments: this is how it finds the application.
struct State {
    /// The caller's application, lifetime erased: `run` outlives every message it can receive.
    app: *mut dyn App,
    renderer: Option<Renderer>,
    last_frame: Instant,
    /// A frame was asked for by an event.
    redraw: bool,
    /// The last frame said something still moves.
    moving: bool,
}

impl State {
    fn busy(&self) -> bool {
        self.redraw || self.moving
    }

    fn send(&mut self, event: Event) {
        // SAFETY: see `app`. Messages arrive one at a time on this thread, so no other borrow is live.
        if unsafe { (*self.app).event(event) } {
            self.redraw = true;
        }
    }

    fn draw(&mut self) {
        let Some(renderer) = self.renderer.as_mut() else { return };
        let now = Instant::now();
        let seconds = now.duration_since(self.last_frame).as_secs_f32().min(MAX_STEP);
        self.last_frame = now;
        self.redraw = false;
        // SAFETY: as in `send`.
        self.moving = unsafe { (*self.app).frame(renderer, seconds) };
    }
}

/// Opens a window for `app` and runs until it closes.
pub fn run(title: &str, app: &mut dyn App) -> Result<()> {
    unsafe {
        // Per monitor, version 2: the window gets the real pixel size of its screen and is told when it
        // moves to another one, instead of being stretched blurry by the system. Fails harmlessly when a
        // manifest or an earlier call already chose.
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let instance = GetModuleHandleW(None)?.into();
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

        // The erasure is sound because `state` lives in this function, which returns only after the window
        // is destroyed: no message reaches the procedure with a dangling pointer.
        let app: *mut dyn App = std::mem::transmute::<&mut dyn App, &'static mut dyn App>(app);
        // Reached from here and from the window procedure alike, so it is a raw pointer: two `&mut` to it
        // alive at once would be undefined behaviour even on one thread.
        let state = Box::into_raw(Box::new(State { app, renderer: None, last_frame: Instant::now(), redraw: false, moving: false }));
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
        (*(*state).app).start(&mut renderer);
        (*state).renderer = Some(renderer);
        (*state).send(Event::Resized { width_px: width, height_px: height, scale });
        (*state).redraw = true;
        let _ = ShowWindow(hwnd, SW_SHOW);

        let result = message_loop(state);

        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        // The renderer goes before the class: it holds the window's swap chain.
        drop(Box::from_raw(state));
        let _ = UnregisterClassW(w!("nui_window"), Some(instance));
        result
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
            unsafe { (*state).draw() };
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
    // SAFETY: set by `run` for the window's life and cleared before the State is dropped.
    let state = unsafe { &mut *state };
    let scale = || unsafe { GetDpiForWindow(hwnd) } as f32 / 96.0;

    match msg {
        WM_SIZE => {
            if wparam.0 as u32 != SIZE_MINIMIZED {
                resized(hwnd, state);
            }
            LRESULT(0)
        }
        WM_DPICHANGED => {
            // The system proposes a rectangle that keeps the window the same size in device-independent
            // pixels on the new screen; taking it is what stops it jumping in size.
            let suggested = unsafe { *(lparam.0 as *const RECT) };
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    suggested.left,
                    suggested.top,
                    suggested.right - suggested.left,
                    suggested.bottom - suggested.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            // A change of dpi can leave the pixel size alone, so WM_SIZE may not follow.
            resized(hwnd, state);
            LRESULT(0)
        }
        WM_PAINT => {
            state.draw();
            unsafe {
                let _ = ValidateRect(Some(hwnd), None);
            }
            LRESULT(0)
        }
        // The frame covers every pixel: erasing first would only flicker.
        WM_ERASEBKGND => LRESULT(1),
        WM_MOUSEMOVE => {
            state.send(Event::PointerMoved { pos: point(lparam, scale()) });
            LRESULT(0)
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
            // Captured, so a drag keeps reporting after the pointer leaves the window.
            unsafe {
                SetCapture(hwnd);
            }
            state.send(Event::PointerDown { pos: point(lparam, scale()), button: button(msg) });
            LRESULT(0)
        }
        WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => {
            unsafe {
                let _ = ReleaseCapture();
            }
            state.send(Event::PointerUp { pos: point(lparam, scale()), button: button(msg) });
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            // Unlike the other mouse messages, the wheel reports the position on the screen.
            let (x, y) = halves(lparam);
            let mut p = POINT { x, y };
            unsafe {
                let _ = ScreenToClient(hwnd, &mut p);
            }
            let s = scale();
            // 120 is one notch; away from the user is positive.
            let delta = ((wparam.0 >> 16) & 0xffff) as u16 as i16 as f32 / 120.0;
            state.send(Event::Wheel { pos: Vec2::new(p.x as f32 / s, p.y as f32 / s), delta });
            LRESULT(0)
        }
        WM_CLOSE => {
            state.send(Event::CloseRequested);
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
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
fn resized(hwnd: HWND, state: &mut State) {
    let (width, height, scale) = client_size(hwnd);
    let Some(renderer) = state.renderer.as_mut() else { return };
    if let Err(e) = renderer.resize(width, height, scale) {
        observe!(Error, "resize to {width}x{height} failed: {e}");
        return;
    }
    state.send(Event::Resized { width_px: width, height_px: height, scale });
    // A new size shows nothing until redrawn; WM_PAINT draws it even inside the system's resize loop.
    state.redraw = true;
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}
