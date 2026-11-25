//! The ListLine card: the states of a line, one under the other in a 480 px column so that the slots can be seen
//! to line up: up to date, to update, an update under way, added, a long name, with a description and a note, and
//! with the pointer on the name, on the action and on the icon.

use mere_layout::{Dim, Style};
use mere_ui::{Icon, LineAction, LineEnd, ListLine, Theme, Ui};

use super::{Block, Card, Cell, State, render};

const TRASH: &str = "M9 2h6v2h5v2H4V4h5zM5 8h2v12h10V8h2v14H5zm4 2h2v8H9zm4 0h2v8h-2z";
const LINK: &str = "M13 3h8v8h-2V7h-2v2h-2v2h-2v2h-2v-2h2V9h2V7h2V5h-4zM3 5h8v2H5v12h12v-6h2v8H3z";
const CHECK: &str = "M4 12h2v2H4zM6 14h2v2H6zM8 16h2v2H8zM10 14h2v2h-2zM12 12h2v2h-2zM14 10h2v2h-2zM16 8h2v2h-2zM18 6h2v2h-2z";
const COL: f32 = 480.0;

/// Which part of the line the pointer rests on.
#[derive(Copy, Clone)]
enum On {
    Line,
    Name,
    Action,
    End,
}

fn icon(path: &str) -> Icon {
    Icon::parse(path, 24.0).expect("the card's icon parses")
}

/// What a line is made of: the trash, the link and the check, which it takes the ones it shows.
type Make = for<'a> fn(&'a Icon, &'a Icon, &'a Icon) -> ListLine<'a>;

fn cell(on: On, state: State, make: Make) -> Cell {
    Cell::new(state, move |ui: &mut Ui, _| {
        let (trash, link, check) = (icon(TRASH), icon(LINK), icon(CHECK));
        let mut id = None;
        ui.node("wrap", Style { w: Dim::Px(COL), column: true, ..Style::default() }, |ui| {
            let r = ui.list_line("l", make(&trash, &link, &check));
            id = Some(match on {
                On::Line => r.line,
                On::Name => r.name.id,
                On::Action => r.action.id,
                On::End => r.end.id,
            });
        });
        id.unwrap()
    })
}

fn up_to_date<'a>(trash: &'a Icon, _: &'a Icon, _: &'a Icon) -> ListLine<'a> {
    ListLine { name: "Tectonic", version: "3.1.0", end: Some(LineEnd { icon: trash, disabled: false }), ..ListLine::default() }
}

fn to_update<'a>(trash: &'a Icon, _: &'a Icon, _: &'a Icon) -> ListLine<'a> {
    ListLine {
        name: "Lithium",
        version: "0.16.0",
        new_version: "\u{2192} 0.16.2",
        action: LineAction::Word("Update"),
        end: Some(LineEnd { icon: trash, disabled: false }),
        ..ListLine::default()
    }
}

fn linked<'a>(trash: &'a Icon, _: &'a Icon, _: &'a Icon) -> ListLine<'a> {
    ListLine { name: "Lithium", version: "0.16.0", link: true, end: Some(LineEnd { icon: trash, disabled: false }), ..ListLine::default() }
}

fn updating<'a>(trash: &'a Icon, _: &'a Icon, _: &'a Icon) -> ListLine<'a> {
    ListLine { action: LineAction::Word("Updating\u{2026}"), action_disabled: true, end: Some(LineEnd { icon: trash, disabled: true }), ..to_update(trash, trash, trash) }
}

fn long_name<'a>(trash: &'a Icon, _: &'a Icon, _: &'a Icon) -> ListLine<'a> {
    ListLine {
        name: "A mod whose name is much too long for the room it has on its line",
        version: "1.0.0-beta.12",
        new_version: "\u{2192} 1.0.0",
        ..to_update(trash, trash, trash)
    }
}

fn added<'a>(_: &'a Icon, link: &'a Icon, check: &'a Icon) -> ListLine<'a> {
    ListLine {
        name: "Iris Shaders",
        version: "1.9.6",
        detail: "Shaders, with Sodium.",
        action: LineAction::Done(check, "Added"),
        end: Some(LineEnd { icon: link, disabled: false }),
        ..ListLine::default()
    }
}

fn noted<'a>(_: &'a Icon, link: &'a Icon, check: &'a Icon) -> ListLine<'a> {
    ListLine {
        name: "Mod Menu",
        version: "15.0.0",
        detail: "A list of the mods in the game.",
        note: "Fabric API came with it: Mod Menu needs it.",
        action: LineAction::Done(check, "Added"),
        end: Some(LineEnd { icon: link, disabled: false }),
        ..ListLine::default()
    }
}

fn to_add<'a>(_: &'a Icon, link: &'a Icon, _: &'a Icon) -> ListLine<'a> {
    ListLine {
        name: "JourneyMap",
        version: "6.0.0",
        detail: "A minimap and a full map of what you explored.",
        action: LineAction::Word("Add"),
        end: Some(LineEnd { icon: link, disabled: false }),
        ..ListLine::default()
    }
}

pub fn blocks() -> Vec<Block> {
    vec![
        Block::matrix(vec![
            vec![cell(On::Line, State::Rest, up_to_date)],
            vec![cell(On::Line, State::Rest, to_update)],
            vec![cell(On::Name, State::Hover, linked)],
            vec![cell(On::Action, State::Hover, to_update)],
            vec![cell(On::Line, State::Rest, updating)],
            vec![cell(On::Line, State::Rest, long_name)],
            vec![cell(On::End, State::Hover, added)],
            vec![cell(On::Line, State::Rest, noted)],
            vec![cell(On::Line, State::Rest, to_add)],
        ])
        .col_width(COL),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
