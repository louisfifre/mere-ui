//! W1: the setting read from the registry is what `reg query` prints for the same value (read only).

#[test]
fn system_dark_matches_the_registry_as_reg_query_prints_it() {
    let out = std::process::Command::new("reg")
        .args(["query", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", "/v", "AppsUseLightTheme"])
        .output()
        .expect("reg.exe runs");
    // A non-zero exit means the value is absent: light, as Chrome reads it.
    let dark = out.status.success() && String::from_utf8_lossy(&out.stdout).contains("0x0");
    println!("AppsUseLightTheme says dark = {dark}");
    assert_eq!(nui_win32::system_dark(), dark);
}
#[test]
fn reduced_motion_reads_without_panicking() {
    // Either answer is right on any machine; the call must return.
    let _ = nui_win32::reduced_motion();
}
