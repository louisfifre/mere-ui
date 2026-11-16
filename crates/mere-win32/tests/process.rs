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
