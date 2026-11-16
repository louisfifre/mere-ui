//! The system clipboard round trip. Ignored by default because it overwrites the user's real clipboard
//! (the saved text is put back afterwards): run it by hand with `-- --ignored`.

use mere_base::Clipboard;
use mere_win32::SystemClipboard;

#[test]
#[ignore = "touches the real clipboard"]
fn text_round_trip() {
    let mut c = SystemClipboard;
    let saved = c.get();
    c.set("mere clipboard round trip");
    let got = c.get();
    if let Some(text) = &saved {
        c.set(text);
    }
    assert_eq!(got.as_deref(), Some("mere clipboard round trip"));
}
