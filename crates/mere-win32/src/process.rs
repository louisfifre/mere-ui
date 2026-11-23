//! `mere_base::Launcher` for Windows. Processes with the standard library only: the flags and `taskkill.exe` do what
//! a binding to the job-object API would, with no new system feature to link. The folder dialog is the shell's own
//! (`IFileOpenDialog` with `FOS_PICKFOLDERS`), the one every Windows program shows.

use std::io;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use mere_base::Launcher;
use windows::Win32::Foundation::{ERROR_CANCELLED, RPC_E_CHANGED_MODE};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize};
use windows::Win32::UI::Shell::{FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, FileOpenDialog, IFileOpenDialog, SIGDN_FILESYSPATH};
use windows::core::{HRESULT, HSTRING};

use crate::window::clipboard_owner;

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

    /// Modal over the window `run` made (its handle is kept for the clipboard): the window's loop waits while the
    /// dialog runs its own.
    fn choose_folder(&self, title: &str) -> io::Result<Option<PathBuf>> {
        let err = |e: windows::core::Error| io::Error::other(e.to_string());
        // The dialog is a COM object of the calling thread's apartment. A thread already in COM in another mode keeps
        // it; only an initialisation made here is undone here.
        let init = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        let ours = init.is_ok();
        if !ours && init != RPC_E_CHANGED_MODE {
            return Err(err(init.into()));
        }
        let chosen = (|| -> windows::core::Result<Option<String>> {
            let dialog: IFileOpenDialog = unsafe { CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)? };
            unsafe {
                let options = dialog.GetOptions()?;
                dialog.SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM)?;
                dialog.SetTitle(&HSTRING::from(title))?;
            }
            if let Err(e) = unsafe { dialog.Show(clipboard_owner()) } {
                return if e.code() == HRESULT::from_win32(ERROR_CANCELLED.0) { Ok(None) } else { Err(e) };
            }
            let item = unsafe { dialog.GetResult()? };
            let name = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH)? };
            let path = unsafe { name.to_string() };
            unsafe { CoTaskMemFree(Some(name.0 as *const _)) };
            Ok(path.ok())
        })();
        if ours {
            unsafe { CoUninitialize() };
        }
        chosen.map(|p| p.map(PathBuf::from)).map_err(err)
    }
}
