//! Starting and ending child processes, as the application needs them, and the two other things it asks of the
//! system's shell (show a folder, ask for one). The trait names no system: a backend implements it (`mere-win32` for
//! Windows), so that the application runs a generator without knowing how a console window is kept away or how a
//! process tree is ended.

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

pub trait Launcher {
    /// Starts the command without a console window of its own.
    fn spawn(&self, cmd: &mut Command) -> io::Result<Child>;
    /// Ends the child and every process it started (a generator leaves a game server behind otherwise). Does
    /// not wait for the child.
    fn stop(&self, child: &mut Child) -> io::Result<()>;
    /// Shows a folder in the system's file browser.
    fn open_folder(&self, path: &Path) -> io::Result<()>;
    /// Asks the person for a folder in the system's own dialog, titled `title`, and waits for the answer: None when
    /// they cancel. A launcher with no dialog (a test's) chooses nothing.
    fn choose_folder(&self, title: &str) -> io::Result<Option<PathBuf>> {
        let _ = title;
        Ok(None)
    }
}
