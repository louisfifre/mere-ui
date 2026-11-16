//! The Cover card: the repository's 960 x 288 cover (components/Cover/preview.html). It is a picture, not a
//! component, so it has no states and the card is the picture alone plus the card padding.

use mere_base::{Cmd, Rect, Weight};
use mere_layout::{Dim, Position, Style};
use mere_ui::{TextLook, Theme, Tone, Ui};

use super::{Block, Card, Cell, State, render};

const W: f32 = 960.0;
const H: f32 = 288.0;
/// The plus grid's pitch: four strip gaps.
const PITCH: f32 = 48.0;

fn at(x: f32, y: f32) -> Style {
    Style {
        position: Position::Absolute,
        inset: mere_layout::Insets { left: Dim::Px(x), top: Dim::Px(y), right: Dim::Auto, bottom: Dim::Auto },
        ..Style::default()
    }
}

fn art(t: Theme) -> Vec<Cmd> {
    let fill = |x: f32, y: f32, w: f32, h: f32, color| Cmd::Fill { rect: Rect::new(x, y, w, h), color };
    let mut cmds = Vec::new();
    // A plus grid in the edge colour; the blocks cover the part under them.
    let mut x = 504.0;
    while x <= 936.0 {
        let mut y = 24.0;
        while y <= 264.0 {
            cmds.push(fill(x - 3.0, y, 7.0, 1.0, t.edge));
            cmds.push(fill(x, y - 3.0, 1.0, 7.0, t.edge));
            y += PITCH;
        }
        x += PITCH;
    }
    cmds.push(fill(600.0, 24.0, 168.0, 240.0, t.text));
    cmds.push(fill(767.0, 24.0, 140.0, 120.0, t.edge));
    cmds.push(fill(767.0, 143.0, 60.0, 61.0, t.warn));
    cmds.push(fill(826.0, 143.0, 81.0, 61.0, t.muted));
    // One hairline seam where neighbours meet.
    cmds.push(fill(767.0, 24.0, 1.0, 180.0, t.ground));
    cmds.push(fill(767.0, 143.0, 140.0, 1.0, t.ground));
    cmds.push(fill(826.0, 143.0, 1.0, 61.0, t.ground));
    cmds
}

pub fn blocks(theme: Theme) -> Vec<Block> {
    vec![Block::single(Cell::new(State::Rest, move |ui: &mut Ui, _| {
        let name = TextLook { size: 76.0, line: 72.2, weight: Weight::SemiBold, tone: Tone::Text, tracking: 0.0 };
        let tag = TextLook { size: 13.0, line: 18.0, weight: Weight::Regular, tone: Tone::Muted, tracking: 0.0 };
        ui.node("cover", Style { w: Dim::Px(W), h: Dim::Px(H), ..Style::default() }, |ui| {
            ui.canvas("art", Style { w: Dim::Px(W), h: Dim::Px(H), ..at(0.0, 0.0) }, |_, _| art(theme));
            // The name's two lines end 58 px above the bottom; the tag's line, 28 px.
            ui.node("name", Style { column: true, ..at(40.0, H - 58.0 - 2.0 * name.line) }, |ui| {
                ui.text("rust-", name);
                ui.text("native-ui", name);
            });
            ui.node("tag", at(40.0, H - 28.0 - tag.line), |ui| {
                ui.text("A native interface toolkit in Rust, drawn by hand.", tag);
            });
        })
    }))]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks(theme))
}
