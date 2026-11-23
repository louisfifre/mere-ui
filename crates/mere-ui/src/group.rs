//! The group: rows stacked, with no frame and no fill, so that it can be placed anywhere (in a menu's list, in a
//! panel, alone). What parts two groups is decided here once, from what the group follows, never by its caller:
//!
//! - a group that follows another group in the same container is parted from it: by a separator (a thin rule, inset
//!   as the rows' texts are) when it has no title, by `gap-wide` when it has one, its title's own rule parting it;
//! - the first group of a container has nothing above it but, when titled, the short space over its title.
//!
//! So a menu of three groups gets its two separators and a panel of titled groups its spacing without either
//! counting them. An empty group is the caller's to leave out: a group cannot know it is empty before it is built.
//!
//! A titled group (`titled_group`, a panel's group of settings): the title in the label's look, in capitals, with an
//! optional small control at its end (an arrow that puts the group back), a separator under it, then the rows.

use mere_layout::{Align, Edges, Margin, NodeId, Side, Style};
use mere_base::Vec2;

use crate::id::Id;
use crate::rule::Rule;
use crate::text::TextLook;
use crate::tokens::{GAP, GAP_WIDE, INSET, INSET_TIGHT};
use crate::ui::{Paint, Ui};

/// The parts of a titled group.
#[derive(Copy, Clone, Debug)]
pub struct GroupResp {
    pub group: Id,
    /// The title and the rule under it.
    pub head: Id,
}

fn stack() -> Style {
    Style { column: true, shrink: 0.0, ..Style::default() }
}

impl Ui {
    /// True when the last thing the open container holds is a group: the next group follows it.
    fn follows_group(&self) -> bool {
        let parent = self.stack.last().expect("the root is always open").0;
        matches!((self.last_group, self.tree.children(parent).last()), (Some(g), Some(&last)) if g == last)
    }

    fn group_node(&mut self, key: &str, style: Style, body: impl FnOnce(&mut Ui)) -> (Id, NodeId) {
        let id = self.key(key);
        let node = self.open(key, id, style, Paint::None);
        body(self);
        self.close();
        self.last_group = Some(node);
        (id, node)
    }

    /// A group of rows with no title, parted from a group it follows by a separator.
    pub fn group(&mut self, key: &str, body: impl FnOnce(&mut Ui)) -> Id {
        if self.follows_group() {
            self.rule(Rule::Separator, Edges::vh(INSET_TIGHT, INSET));
        }
        self.group_node(key, stack(), body).0
    }

    /// A group with a title: `title` (the application's word, written in capitals here) and `end`, what sits at the
    /// end of the title's line (nothing, or a small control such as `small_icon`), then a separator, then the rows
    /// `body` builds. Parted from a group it follows by `gap-wide`.
    pub fn titled_group(&mut self, key: &str, title: &str, end: impl FnOnce(&mut Ui), body: impl FnOnce(&mut Ui)) -> GroupResp {
        let above = if self.follows_group() { GAP_WIDE } else { INSET_TIGHT };
        let mut head = Id::ROOT;
        let style = Style { pad: Edges::new(above, 0.0, 0.0, 0.0), ..stack() };
        let (group, _) = self.group_node(key, style, |ui| {
            head = ui.node("head", Style { column: true, ..Style::default() }, |ui| {
                // Centred: a small control is as tall as the title's line, so it adds no height.
                let line = Style { align_items: Align::Center, gap: Vec2::new(GAP, 0.0), pad: Edges::new(0.0, 0.0, INSET_TIGHT, 0.0), ..Style::default() };
                ui.node("title", line, |ui| {
                    ui.text(&title.to_uppercase(), TextLook::label());
                    ui.node("end", Style { margin: Margin { left: Side::Auto, ..Margin::default() }, ..Style::default() }, end);
                });
                ui.rule(Rule::Separator, Edges::default());
            });
            ui.node("rows", Style { column: true, pad: Edges::new(INSET_TIGHT, 0.0, 0.0, 0.0), ..Style::default() }, body);
        });
        GroupResp { group, head }
    }
}
