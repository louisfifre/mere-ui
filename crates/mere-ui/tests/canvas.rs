//! The canvas: commands in a node's own coordinates, the press edge, and the wheel taken once.

use mere_base::{Button, Cmd, Color, DrawList, Event, Key, Rect, Vec2};
use mere_layout::{Dim, Edges, Style};
use mere_ui::{Resp, Theme, Ui};

const SIZE: Vec2 = Vec2::new(400.0, 300.0);

/// A 100 x 80 canvas inside a padded box, so that its place is not the window's origin.
fn frame(ui: &mut Ui, draw: impl FnOnce(&mut Ui, &Resp) -> Vec<Cmd>) -> (Resp, DrawList) {
    ui.begin_frame(SIZE);
    let mut resp = None;
    ui.node("outer", Style { pad: Edges::new(30.0, 0.0, 0.0, 40.0), ..Style::default() }, |ui| {
        let style = Style { w: Dim::Px(100.0), h: Dim::Px(80.0), shrink: 0.0, ..Style::default() };
        resp = Some(ui.canvas("map", style, draw));
    });
    let list = ui.end_frame().clone();
    (resp.unwrap(), list)
}

fn none(_: &mut Ui, _: &Resp) -> Vec<Cmd> {
    Vec::new()
}

fn centre(ui: &Ui, r: &Resp) -> Vec2 {
    let r = ui.rect(r.id).expect("drawn once");
    Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0)
}

#[test]
fn local_commands_land_at_the_nodes_origin() {
    let mut ui = Ui::new(Theme::dark());
    let red = Color::hex(0xff0000);
    let fill = |_: &mut Ui, _: &Resp| vec![Cmd::Fill { rect: Rect::new(0.0, 0.0, 5.0, 5.0), color: red }];
    let (_, list) = frame(&mut ui, fill);
    let at = list
        .cmds
        .iter()
        .find_map(|c| match c {
            Cmd::Fill { rect, color } if *color == red => Some(*rect),
            _ => None,
        })
        .expect("the canvas's fill is in the list");
    // pad is (top 30, right 0, bottom 0, left 40) in the order Edges::new takes.
    assert_eq!((at.x, at.y, at.w, at.h), (40.0, 30.0, 5.0, 5.0));
    // And the node itself is where the commands think it is.
    let (resp, _) = frame(&mut ui, none);
    assert_eq!(ui.rect(resp.id), Some(Rect::new(40.0, 30.0, 100.0, 80.0)));
}

#[test]
fn pressed_is_the_edge_held_is_the_level_clicked_the_release() {
    let mut ui = Ui::new(Theme::dark());
    let (r, _) = frame(&mut ui, none);
    let p = centre(&ui, &r);
    ui.event(&Event::PointerMoved { pos: p });
    let (r, _) = frame(&mut ui, none);
    assert!(!r.pressed && !r.held && !r.clicked);

    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    let (r, _) = frame(&mut ui, none);
    assert!(r.pressed && r.held && !r.clicked, "the frame of the press");

    let (r, _) = frame(&mut ui, none);
    assert!(!r.pressed && r.held && !r.clicked, "held on the next frame, no longer the edge");

    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
    let (r, _) = frame(&mut ui, none);
    assert!(!r.pressed && !r.held && r.clicked, "released inside is a click");
}

#[test]
fn the_wheel_is_taken_once_and_only_over_the_canvas() {
    let mut ui = Ui::new(Theme::dark());
    let (r, _) = frame(&mut ui, none);
    let inside = centre(&ui, &r);

    // Over the canvas: the notches and the place, once.
    ui.event(&Event::Wheel { pos: inside, delta: 1.0 });
    let mut got = Vec::new();
    frame(&mut ui, |ui, resp| {
        got.push(ui.take_wheel(resp.id));
        got.push(ui.take_wheel(resp.id));
        Vec::new()
    });
    assert_eq!(got, vec![Some((1.0, inside)), None]);

    // Outside it: nothing, and the wheel stays for whoever is under the pointer.
    ui.event(&Event::Wheel { pos: Vec2::new(300.0, 250.0), delta: 1.0 });
    let mut got = None;
    frame(&mut ui, |ui, resp| {
        got = Some(ui.take_wheel(resp.id));
        Vec::new()
    });
    assert_eq!(got, Some(None));
}

#[test]
fn the_pointer_is_known_after_it_moved() {
    let mut ui = Ui::new(Theme::dark());
    assert_eq!(ui.pointer(), None);
    ui.event(&Event::PointerMoved { pos: Vec2::new(3.0, 4.0) });
    assert_eq!(ui.pointer(), Some(Vec2::new(3.0, 4.0)));
}

/// A focusable canvas beside a text field, so that Tab has somewhere to go.
fn focus_frame(ui: &mut Ui, disabled: bool, seen: &mut Vec<Key>, focused: &mut bool) -> Resp {
    ui.begin_frame(SIZE);
    let mut resp = None;
    ui.node("outer", Style { pad: Edges::new(30.0, 0.0, 0.0, 40.0), ..Style::default() }, |ui| {
        let style = Style { w: Dim::Px(100.0), h: Dim::Px(80.0), shrink: 0.0, ..Style::default() };
        resp = Some(ui.canvas_focus("bar", style, disabled, |ui, r, keys| {
            *focused = keys.focused;
            seen.extend(keys.keys.iter().map(|k| k.0));
            let mut cmds = Vec::new();
            if keys.focused {
                cmds.push(ui.focus_ring(r.rect.unwrap_or_default()));
            }
            cmds
        }));
    });
    ui.end_frame();
    resp.unwrap()
}

#[test]
fn a_focusable_canvas_takes_the_keyboard_on_a_press_and_its_keys_only_then() {
    let mut ui = Ui::new(Theme::dark());
    let (mut seen, mut focused) = (Vec::new(), false);
    let r = focus_frame(&mut ui, false, &mut seen, &mut focused);
    // Keys before the focus belong to no one.
    ui.event(&Event::Key { key: Key::Right, mods: Default::default() });
    focus_frame(&mut ui, false, &mut seen, &mut focused);
    assert!(!focused && seen.is_empty());
    let p = centre(&ui, &r);
    ui.event(&Event::PointerMoved { pos: p });
    focus_frame(&mut ui, false, &mut seen, &mut focused);
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    focus_frame(&mut ui, false, &mut seen, &mut focused);
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
    focus_frame(&mut ui, false, &mut seen, &mut focused);
    assert!(focused, "a press focuses it");
    for key in [Key::Left, Key::Home, Key::End, Key::Right] {
        ui.event(&Event::Key { key, mods: Default::default() });
    }
    focus_frame(&mut ui, false, &mut seen, &mut focused);
    assert_eq!(seen, vec![Key::Left, Key::Home, Key::End, Key::Right]);
}

#[test]
fn a_disabled_focusable_canvas_takes_no_focus() {
    let mut ui = Ui::new(Theme::dark());
    let (mut seen, mut focused) = (Vec::new(), false);
    let r = focus_frame(&mut ui, true, &mut seen, &mut focused);
    let p = centre(&ui, &r);
    ui.event(&Event::PointerMoved { pos: p });
    focus_frame(&mut ui, true, &mut seen, &mut focused);
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    focus_frame(&mut ui, true, &mut seen, &mut focused);
    ui.event(&Event::Key { key: Key::Right, mods: Default::default() });
    focus_frame(&mut ui, true, &mut seen, &mut focused);
    assert!(!focused && seen.is_empty());
}
