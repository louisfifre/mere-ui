//! `nui_base::Launcher` for Windows, with the standard library only: the flags and `taskkill.exe` do what a
//! binding to the job-object API would, with no new system feature to link.

use std::io;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use nui_base::Launcher;

/// `CREATE_NO_WINDOW`: a console program gets no console window.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct SystemLauncher;

impl Launcher for SystemLauncher {
    fn spawn(&self, cmd: &mut Command) -> io::Result<Child> {
        cmd.creation_flags(CREATE_NO_WINDOW).spawn()
    }

    /// `taskkill /T` walks the tree of the child's descendants; /F because a generator does not answer a polite
    /// request while it waits for its game server.
    fn stop(&self, child: &mut Child) -> io::Result<()> {
        let status = Command::new("taskkill.exe")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status()?;
        if status.success() || child.try_wait()?.is_some() {
            return Ok(());
        }
        Err(io::Error::other(format!("taskkill ended with {status}")))
    }

    /// Explorer's exit code means nothing (it answers 1 when it succeeds), so it is not waited for.
    fn open_folder(&self, path: &Path) -> io::Result<()> {
        Command::new("explorer.exe").arg(path).spawn().map(|_| ())
    }
}
