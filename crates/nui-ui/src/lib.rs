//! The immediate-mode interface: the application describes the interface again on every frame, and this
//! crate lays it out (`nui-layout`), answers the pointer, and returns a draw list for any backend.
//!
//! One frame, in this order:
//! 1. `Ui::event` for every event received since the last frame (pointer state is only stored);
//! 2. `Ui::begin_frame(size)`;
//! 3. the application's calls: `node`, `label`, `button`, `toggle`, `fold`, `tab`, `detail_tab`, `number`,
//!    `text_field`...
//!    Each widget answers with what the
//!    pointer did to it, judged against where the widget stood in the LAST frame (what the person saw and
//!    pointed at), so the first frame of a widget has no interaction;
//! 4. `Ui::end_frame()`: layout, painting into the returned `DrawList`, then the pointer's one-frame edges
//!    are cleared.
//!
//! This crate names no operating system: the draw list is drawn by whichever backend the application uses.

mod edit;
mod field;
mod fold;
mod id;
mod input;
mod number;
mod tab;
mod text;
mod theme;
mod ui;
mod widgets;

pub use edit::MemoryClipboard;
pub use field::{FieldLook, FieldOpts, FieldResp};
pub use number::NumberSpec;
pub use fold::FoldOpts;
pub use id::Id;
pub use input::Input;
pub use tab::{TabLook, TabOpts, choice_style, details_style, strip_style};
pub use text::{ADVANCE_EM, ASCENT_EM, DESCENT_EM, TextLook, Tone, ellipsize, text_width};
pub use theme::{Theme, mix};
pub use ui::{Resp, Ui};
pub use widgets::{ButtonLook, ButtonOpts};
