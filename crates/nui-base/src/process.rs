//! Starting and ending child processes, as the application needs them. The trait names no system: a backend
//! implements it (`nui-win32` for Windows), so that the application runs a generator without knowing how a
//! console window is kept away or how a process tree is ended.

use std::io;
use std::path::Path;
use std::process::{Child, Command};

pub trait Launcher {
    /// Starts the command without a console window of its own.
    fn spawn(&self, cmd: &mut Command) -> io::Result<Child>;
    /// Ends the child and every process it started (a generator leaves a game server behind otherwise). Does
    /// not wait for the child.
    fn stop(&self, child: &mut Child) -> io::Result<()>;
    /// Shows a folder in the system's file browser.
    fn open_folder(&self, path: &Path) -> io::Result<()>;
}
