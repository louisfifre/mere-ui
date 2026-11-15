//! What the run and export tests share: the real data and PowerShell they need (a test says "skipped" without
//! them), a root folder of their own under the repository's target\tmp, and the application driven offscreen by
//! events and frames as the window would.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use nui_base::{App, Button, Color, Event, Rect, Vec2};
use nui_ui::{Id, Theme};
use nui_win32::{Renderer, SystemLauncher};
use preview::run::Runs;
use preview::world::World;
use preview::{PreviewApp, PreviewState};

pub const W: u32 = 1400;
pub const H: u32 = 940;
const DATA: &str = r"C:\Games\preview\out";

/// The world of the real out folder, or None (with the reason printed) when the data or PowerShell is missing.
pub fn world() -> Option<World> {
    if !Path::new(DATA).join("preview.json").exists() {
        println!("skipped: no C:\\Games\\preview\\out");
        return None;
    }
    let ps = Command::new("powershell.exe").args(["-NoProfile", "-NonInteractive", "-Command", "exit 0"]).stdin(Stdio::null()).output();
    if ps.is_err() {
        println!("skipped: no powershell.exe");
        return None;
    }
    Some(World::load(Path::new(DATA)).expect("the data loads"))
}

/// A plain (not `\\?\`) absolute path.
pub fn plain(p: &Path) -> PathBuf {
    let c = p.canonicalize().expect("the path exists");
    let s = c.to_string_lossy();
    PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(&s).to_string())
}

/// `<workspace>/target/tmp`, made.
pub fn tmp() -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp");
    std::fs::create_dir_all(&dir).unwrap();
    plain(&dir)
}

/// A root of the test's own under target\tmp, emptied first.
pub fn root(name: &str) -> PathBuf {
    let dir = tmp().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn stub() -> PathBuf {
    plain(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tools/stub_run.ps1"))
}

/// The runs a test gives the application: the stub generator, with its mode in `env`.
pub fn runs(root: &Path, env: &[(&str, &str)]) -> Runs {
    Runs {
        shell: PathBuf::from("powershell.exe"),
        generator: Some(stub()),
        pack: None,
        root: root.to_path_buf(),
        env: env.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        launcher: Box::new(SystemLauncher),
        server_dir: None,
        out_dir: None,
        exports_dir: None,
    }
}

pub fn rgb(c: Color) -> [u8; 3] {
    [c.r, c.g, c.b].map(|v| (v * 255.0).round() as u8)
}

pub fn near(a: [u8; 3], b: [u8; 3], tolerance: i32) -> bool {
    a.iter().zip(b).all(|(x, y)| (*x as i32 - y as i32).abs() <= tolerance)
}

pub fn centre(r: Rect) -> Vec2 {
    Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0)
}

/// Whether a process is still listed by the system.
pub fn alive(pid: u32) -> bool {
    let out = Command::new("tasklist.exe").args(["/FI", &format!("PID eq {pid}"), "/NH"]).output().expect("tasklist runs");
    String::from_utf8_lossy(&out.stdout).contains(&pid.to_string())
}

/// Waits (up to `secs`) for the process to be gone.
pub fn gone_within(pid: u32, secs: u64) -> bool {
    let started = Instant::now();
    while alive(pid) {
        if started.elapsed() > Duration::from_secs(secs) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    true
}

/// The application and its offscreen renderer, the page's dark theme.
pub struct Rig {
    pub app: PreviewApp,
    pub r: Renderer,
    pub theme: Theme,
}

impl Rig {
    /// The app with the world loaded as the command line loads it (the form holds the run's settings).
    pub fn new(world: &World, state: PreviewState, runs: Option<Runs>) -> Rig {
        let theme = Theme::dark();
        let mut r = Renderer::offscreen(W, H, 1.0).expect("offscreen renderer");
        let mut app = PreviewApp::new(theme, true, Vec2::new(W as f32, H as f32), state, Some(world.clone()));
        if let Some(runs) = runs {
            app = app.with_runs(runs);
        }
        app.start(&mut r);
        app.event(Event::Resized { width_px: W, height_px: H, scale: 1.0 });
        app.frame(&mut r, 0.0);
        Rig { app, r, theme }
    }

    pub fn frame(&mut self) {
        self.app.frame(&mut self.r, 0.0);
    }

    /// Frames every 100 ms until `done`, or panics with `what` after `secs`.
    pub fn until(&mut self, secs: u64, what: &str, mut done: impl FnMut(&mut Rig) -> bool) {
        let started = Instant::now();
        loop {
            self.frame();
            if done(self) {
                return;
            }
            assert!(started.elapsed() < Duration::from_secs(secs), "timed out after {secs} s waiting for: {what}");
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn rgba(&mut self) -> Vec<u8> {
        self.r.read_rgba().unwrap()
    }

    /// Draws the frame, keeps it as a PNG under target\tmp and returns the pixels.
    pub fn shot(&mut self, name: &str) -> Vec<u8> {
        self.frame();
        let rgba = self.rgba();
        let path = tmp().join(name);
        std::fs::write(&path, nui_png::encode(W, H, &rgba)).unwrap();
        println!("wrote {}", path.display());
        rgba
    }

    pub fn rect(&self, id: Id) -> Rect {
        assert_ne!(id, Id::ROOT, "the widget is not built");
        self.app.ui().rect(id).expect("the widget was built")
    }

    pub fn px(&self, rgba: &[u8], x: f32, y: f32) -> [u8; 3] {
        let i = (y as usize * W as usize + x as usize) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    }

    /// A click as the other tests make it: the pointer arrives, goes down, a frame, goes up, a frame.
    pub fn click(&mut self, pos: Vec2) {
        self.app.event(Event::PointerMoved { pos });
        self.app.event(Event::PointerDown { pos, button: Button::Left, clicks: 1 });
        self.frame();
        self.app.event(Event::PointerUp { pos, button: Button::Left });
        self.frame();
    }

    pub fn click_on(&mut self, id: Id) {
        let at = centre(self.rect(id));
        self.click(at);
    }

    /// Moves the pointer away from everything, so that nothing is hovered.
    pub fn park(&mut self) {
        self.app.event(Event::PointerMoved { pos: Vec2::new(2.0, 2.0) });
        self.frame();
    }
}
