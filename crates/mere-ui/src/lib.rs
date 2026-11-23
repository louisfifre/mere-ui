//! The immediate-mode interface: the application describes the interface again on every frame, and this
//! crate lays it out (`mere-layout`), answers the pointer, and returns a draw list for any backend.
//!
//! One frame, in this order:
//! 1. `Ui::event` for every event received since the last frame (pointer state is only stored);
//! 2. `Ui::begin_frame(size)`;
//! 3. the application's calls: `node`, `label`, `button`, `toggle`, `fold`, `tab`, `detail_tab`, `number`,
//!    `text_field`, `ellipsis`, `scroll`, `list_row`, `list_head`, `trend_row`...
//!    Each widget answers with what the
//!    pointer did to it, judged against where the widget stood in the LAST frame (what the person saw and
//!    pointed at), so the first frame of a widget has no interaction;
//! 4. `Ui::end_frame()`: layout, painting into the returned `DrawList`, then the pointer's one-frame edges
//!    are cleared.
//!
//! This crate names no operating system: the draw list is drawn by whichever backend the application uses.

mod anim;
mod chrome;
mod compose;
mod edit;
mod field;
mod fold;
mod help;
mod icon;
mod id;
mod input;
mod number;
mod list;
mod menu;
mod scroll;
mod tab;
mod text;
mod theme;
pub mod tokens;
mod ui;
mod widgets;

pub use anim::{SLIDE_SECONDS, Slide, ease};
pub use chrome::{CORNER, GRIP, IconLook, IconOpts, TOOL_GAP, TOOL_PAD_L, TOOL_PAD_R, chrome_hit, grip, tool_button_width};
pub use edit::MemoryClipboard;
pub use icon::Icon;
pub use field::{FieldLook, FieldOpts, FieldResp};
pub use menu::{MENU_ROW, MenuResp, MenuRow, SelectOpts, SelectResp};
pub use list::{ListRow, PLOT_H, PartOpts, ROW_GAP, RowOpts, SHARE_MIN, SWATCH_W, WIDTH_MIN};
pub use number::{NumberOpts, NumberSpec};
pub use scroll::{ARROW, GUTTER, ScrollOpts, ScrollResp, THUMB_MIN, THUMB_W, WHEEL_STEP, arrows_shown, scroll_style};
pub use fold::FoldOpts;
pub use help::{HELP_MARK_W, HelpResp, HelpText, SMALL_ICON, help_anchor, help_label};
pub use id::Id;
pub use input::Input;
pub use tab::{TabLook, TabOpts, choice_style, details_style, strip_style};
pub use text::{ADVANCE_EM, ASCENT_EM, DESCENT_EM, TextLook, Tone, cells, cells_of, ellipsize, ellipsize_start, text_width, wrap_lines, wrap_lines_after};
pub use theme::{Theme, mix};
pub use ui::{Resp, TIP_DELAY, Ui};
pub use widgets::{ButtonLook, ButtonOpts, SWITCH_H, SWITCH_W, ToggleOpts};
