//! X1: stopping a child takes the processes it started, not only itself.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use mere_base::Launcher;
use mere_win32::SystemLauncher;

fn listed(pid: u32) -> bool {
    let out = Command::new("tasklist.exe").args(["/FI", &format!("PID eq {pid}"), "/NH"]).output().expect("tasklist runs");
    String::from_utf8_lossy(&out.stdout).contains(&pid.to_string())
}

#[test]
fn x1_stop_ends_the_child_and_its_child() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp/process_test");
    std::fs::create_dir_all(&dir).unwrap();
    let pidfile = dir.canonicalize().unwrap().join("child.pid");
    let _ = std::fs::remove_file(&pidfile);

    let script = format!(
        "$p = Start-Process powershell.exe -ArgumentList '-NoProfile','-Command','Start-Sleep -Seconds 120' -NoNewWindow -PassThru; \
         [IO.File]::WriteAllText('{}', \"$($p.Id)\"); $p.WaitForExit()",
        pidfile.display()
    );
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    let mut child = SystemLauncher.spawn(&mut cmd).expect("powershell starts");

    let started = Instant::now();
    let grandchild: u32 = loop {
        if let Some(pid) = std::fs::read_to_string(&pidfile).ok().and_then(|t| t.trim().parse().ok()) {
            break pid;
        }
        assert!(started.elapsed() < Duration::from_secs(20), "the grandchild's pid was not written in 20 s");
        std::thread::sleep(Duration::from_millis(100));
    };
    assert!(listed(grandchild), "the grandchild runs before the stop");

    SystemLauncher.stop(&mut child).expect("the stop is accepted");
    let started = Instant::now();
    while child.try_wait().unwrap().is_none() {
        assert!(started.elapsed() < Duration::from_secs(10), "the child is still there 10 s after the stop");
        std::thread::sleep(Duration::from_millis(100));
    }
    // taskkill has ended the tree before it answered, but give the system a moment to drop the entry.
    let started = Instant::now();
    while listed(grandchild) && started.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(!listed(grandchild), "the grandchild {grandchild} outlived the stop");
    let _ = std::fs::remove_file(&pidfile);
}

/// X2: the folder dialog opens with its title and a closed dialog is no folder. Another thread finds the dialog by its
/// title and closes it, as the person's Cancel would: it shows on the screen for a moment.
#[test]
fn x2_the_folder_dialog_shows_its_title_and_cancel_chooses_nothing() {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, PostMessageW, WM_CLOSE};
    use windows::core::HSTRING;

    let title = format!("mere-win32 test {}", std::process::id());
    let looked = title.clone();
    let closer = std::thread::spawn(move || {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(20) {
            if let Ok(hwnd) = unsafe { FindWindowW(None, &HSTRING::from(looked.as_str())) } {
                if !hwnd.is_invalid() {
                    unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) }.expect("the close is posted");
                    return true;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        false
    });
    let chosen = SystemLauncher.choose_folder(&title).expect("the dialog opens");
    assert!(closer.join().unwrap(), "a window titled {title:?} showed");
    assert_eq!(chosen, None, "closed without a choice: no folder");
}
