//! The title line of a page that is not the first: the back action (a bare icon action), the page's title in the
//! heading's look and, at the end, what the page offers at its top (a "Play"). The title keeps its case, where a
//! section's is in capitals: a section is a part of a page, the title is the page's name. It is as tall as a large
//! control, so that the line keeps its height with or without an action at its end, and `gap-wide` over what follows.

use mere_base::Vec2;
use mere_layout::{Align, Dim, Margin, Side, Style};

use crate::chrome::{IconLook, IconOpts};
use crate::icon::Icon;
use crate::id::Id;
use crate::text::{TextLook, Tone};
use crate::tokens::{CONTROL_LARGE, GAP, GAP_WIDE};
use crate::ui::{Resp, Ui};

/// What a page's title line answers.
#[derive(Copy, Clone, Debug)]
pub struct PageHeadResp {
    pub head: Id,
    /// The back action: `clicked` asks to leave the page.
    pub back: Resp,
}

impl Ui {
    /// The title line: `back` is the app's arrow, `title` its page's name (cut with an ellipsis at its end when the
    /// room runs out) and `end` what sits at the right end. The caller gives the back action its tooltip.
    pub fn page_head(&mut self, key: &str, back: &Icon, title: &str, end: impl FnOnce(&mut Ui)) -> PageHeadResp {
        let mut resp = None;
        let style = Style { align_items: Align::Center, gap: Vec2::new(GAP, 0.0), min_h: Dim::Px(CONTROL_LARGE), margin: Margin { bottom: Side::Px(GAP_WIDE), ..Margin::default() }, ..Style::default() };
        let head = self.node(key, style, |ui| {
            resp = Some(ui.icon_action("back", back, IconLook::Bare, IconOpts::default()));
            let grow = Style { grow: 1.0, min_w: Dim::Px(0.0), ..Style::default() };
            ui.node("title", grow, |ui| ui.ellipsis(title, TextLook { tone: Tone::Text, ..TextLook::heading() }));
            end(ui);
        });
        PageHeadResp { head, back: resp.expect("the line was built") }
    }
}
