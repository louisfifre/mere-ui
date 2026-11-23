//! What a run needed of the kit (build plan step 17): the running look of the generate button and the stop
//! square. Read at the level of the draw list, as the other widget tests do. Numbers come from the page's CSS
//! (preview.html lines 210-222).

#[allow(unused_imports)]
use mere_ui::tokens::*;
use mere_base::{Button, Cmd, Color, DrawList, Event, Rect, Vec2};
use mere_layout::{Dim, Style};
use mere_ui::{ActionLook, ActionOpts, Resp, Theme, Ui};

const SIZE: Vec2 = Vec2::new(600.0, 400.0);

fn near(a: f32, b: f32) {
    assert!((a - b).abs() < 0.01, "{a} != {b}");
}

fn column(w: f32) -> Style {
    Style { column: true, w: Dim::Px(w), ..Style::default() }
}

fn fills(list: &DrawList, color: Color) -> Vec<Rect> {
    list.cmds.iter().filter_map(|k| if let Cmd::Fill { rect, color: c } = k { (*c == color).then_some(*rect) } else { None }).collect()
}

fn frames(list: &DrawList) -> Vec<(Rect, Color)> {
    list.cmds.iter().filter_map(|k| if let Cmd::Frame { rect, color, .. } = k { Some((*rect, *color)) } else { None }).collect()
}

fn words(list: &DrawList) -> Vec<(String, Color)> {
    list.cmds.iter().filter_map(|k| if let Cmd::Text { text, color, .. } = k { Some((text.clone(), *color)) } else { None }).collect()
}

fn running(ui: &mut Ui, share: f32, disabled: bool) -> (DrawList, Rect) {
    let mut id = None;
    ui.begin_frame(SIZE);
    ui.node("col", column(300.0), |ui| {
        let opts = ActionOpts { disabled, grow: 1.0 };
        id = Some(ui.action_with("generate", "Generating\u{2026}", ActionLook::Running { share }, opts).id);
    });
    let list = ui.end_frame().clone();
    (list, ui.rect(id.unwrap()).unwrap())
}

#[test]
fn n1_running_fills_from_the_left_with_the_share() {
    let t = Theme::dark();
    let mut ui = Ui::new(t);
    let (list, r) = running(&mut ui, 0.4, false);
    near(r.w, 300.0);
    near(r.h, CONTROL_LARGE);
    assert_eq!(frames(&list), vec![(r, t.text)]);
    let bars = fills(&list, t.line);
    assert_eq!(bars.len(), 1);
    near(bars[0].x, r.x + 1.0);
    near(bars[0].y, r.y + 1.0);
    near(bars[0].w, 0.4 * 298.0);
    near(bars[0].h, CONTROL_LARGE - 2.0 * FRAME);
    // No fill of the text colour: the button is not the primary one.
    assert!(fills(&list, t.text).is_empty());
    // A share past 1 stops at the padding box's width.
    let (list, _) = running(&mut ui, 1.5, false);
    near(fills(&list, t.line)[0].w, 298.0);
    let (list, _) = running(&mut ui, -1.0, false);
    near(fills(&list, t.line)[0].w, 0.0);
}

#[test]
fn n2_running_and_disabled_keeps_the_word_and_greys_the_frame() {
    let t = Theme::dark();
    let mut ui = Ui::new(t);
    let (list, r) = running(&mut ui, 0.4, true);
    assert_eq!(frames(&list), vec![(r, t.edge)]);
    assert_eq!(words(&list), vec![("Generating\u{2026}".to_string(), t.text)]);
    // The bar is still there: the run goes on while the stop is being taken.
    assert_eq!(fills(&list, t.line).len(), 1);
}

fn stop(ui: &mut Ui, disabled: bool) -> (DrawList, Resp) {
    let mut resp = None;
    ui.begin_frame(SIZE);
    ui.node("row", Style { w: Dim::Px(300.0), ..Style::default() }, |ui| {
        resp = Some(ui.stop_action("stop", disabled));
    });
    (ui.end_frame().clone(), resp.unwrap())
}

#[test]
fn n3_stop_square_looks_and_clicks() {
    let t = Theme::dark();
    let mut ui = Ui::new(t);
    let (list, resp) = stop(&mut ui, false);
    let r = ui.rect(resp.id).unwrap();
    near(r.w, CONTROL_LARGE);
    near(r.h, CONTROL_LARGE);
    assert_eq!(frames(&list), vec![(r, t.edge)]);
    let icon = fills(&list, t.muted);
    assert_eq!(icon.len(), 1);
    assert_eq!((icon[0].x - r.x, icon[0].y - r.y, icon[0].w, icon[0].h), (14.0, 14.0, 8.0, 8.0));

    // Hovered: both the text colour.
    let p = Vec2::new(r.x + 18.0, r.y + 18.0);
    ui.event(&Event::PointerMoved { pos: p });
    let (list, _) = stop(&mut ui, false);
    assert_eq!(frames(&list), vec![(r, t.text)]);
    assert_eq!(fills(&list, t.text).len(), 1);

    // Disabled: both faded to 0.4 over the ground, and a click does nothing.
    let (list, _) = stop(&mut ui, true);
    assert_eq!(frames(&list), vec![(r, t.faded(t.edge, FADE_DISABLED))]);
    assert_eq!(fills(&list, t.faded(t.muted, FADE_DISABLED)).len(), 1);
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    stop(&mut ui, true);
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
    let (_, resp) = stop(&mut ui, true);
    assert!(!resp.clicked && !resp.hovered);

    // Enabled again, the same press is a click.
    let mut ui = Ui::new(t);
    stop(&mut ui, false);
    ui.event(&Event::PointerMoved { pos: p });
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    stop(&mut ui, false);
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
    assert!(stop(&mut ui, false).1.clicked);
}
