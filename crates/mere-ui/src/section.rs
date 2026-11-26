//! The section: a fold that does not fold. A section whose head carries actions (a list's "New profile", the mods'
//! "Update all" and "Add a mod") keeps its body open, and puts its actions at the right end of the head, where a
//! fold has its sign: one place for what the head can do, so a head never holds both a fold and actions. A section
//! with no action is a fold (`fold.rs`). Its rule, head and body are the fold's own (`fold::container_style` and the
//! two beside it), so that the two sit on one grid whatever a page mixes.

use mere_layout::{Align, Edges, Margin, Side, Style};
use mere_base::Vec2;

use crate::fold::{body_style, container_style, head_style};
use crate::id::Id;
use crate::rule::Rule;
use crate::text::TextLook;
use crate::tokens::GAP_WIDE;
use crate::ui::Ui;

/// The actions of a head are `gap-wide` apart. The boards draw 16 px, a value the spacing scale has not: the nearest
/// step is taken, to be settled with the design system.
pub const ACTIONS_GAP: f32 = GAP_WIDE;

impl Ui {
    /// A titled section that stays open: the rule that opens it, the head (`title`, which the widget writes in
    /// capitals, and at the head's end what `actions` builds, usually underlined actions) and the body. Centred on
    /// the line, so that an action taller than the title (an underlined word) does not push the title off the line.
    /// Returns the section's id.
    pub fn section(&mut self, key: &str, title: &str, actions: impl FnOnce(&mut Ui), body: impl FnOnce(&mut Ui)) -> Id {
        let t = *self.theme();
        self.node(key, container_style(), |ui| {
            ui.rule(Rule::Section, Edges::default());
            ui.node("head", head_style(Align::Center), |ui| {
                ui.add_text(&title.to_uppercase(), TextLook::heading(), t.text, false, Style::default());
                let end = Style { align_items: Align::Center, gap: Vec2::new(ACTIONS_GAP, 0.0), shrink: 0.0, margin: Margin { left: Side::Auto, ..Margin::default() }, ..Style::default() };
                ui.node("actions", end, actions);
            });
            ui.node("body", body_style(), body);
        })
    }
}

