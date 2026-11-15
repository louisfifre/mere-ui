//! The window's housekeeping as the application sees it: a lost device, a change of scale, a run under way that must
//! not keep the window drawing, and the sheet of the launcher's icons at each scale. Pictures go to target\tmp.

mod common;

use std::time::{Duration, Instant};

use common::*;
use nui_base::{App, DrawList, Event, Gfx, Rect, Vec2};
use nui_ui::Theme;
use nui_win32::Renderer;
use preview::{PreviewApp, PreviewState};

/// An application started on `r` at a size in DIP, as the window would start it.
fn app_on(world: &preview::world::World, r: &mut Renderer, w: u32, h: u32, scale: f32) -> PreviewApp {
    let mut app = PreviewApp::new(Theme::dark(), true, Vec2::new(w as f32, h as f32), PreviewState::for_world(Some(world)), Some(world.clone()));
    app.start(r);
    app.event(Event::Resized { width_px: (w as f32 * scale) as u32, height_px: (h as f32 * scale) as u32, scale });
    app
}

/// Two frames, because the first one can ask for a second (the scrollbar's gutter comes a frame late).
fn settle(app: &mut PreviewApp, r: &mut Renderer) -> Vec<u8> {
    app.frame(r, 0.0);
    app.frame(r, 0.0);
    r.read_rgba().unwrap()
}

/// P1, device loss: a lost renderer is drawn to without harm, and an application restarted on a new one draws the
/// same picture as one that never saw a loss.
#[test]
fn p1_a_lost_device_is_rebuilt_to_the_same_picture() {
    let Some(world) = world() else { return };
    let mut a = Renderer::offscreen(W, H, 1.0).unwrap();
    let mut app = app_on(&world, &mut a, W, H, 1.0);
    settle(&mut app, &mut a);

    a.mark_lost();
    assert!(a.lost());
    app.frame(&mut a, 0.0); // no panic: a lost renderer ignores what it is given

    let mut b = Renderer::offscreen(W, H, 1.0).unwrap();
    app.start(&mut b);
    app.event(Event::Resized { width_px: W, height_px: H, scale: 1.0 });
    let rebuilt = settle(&mut app, &mut b);

    let mut c = Renderer::offscreen(W, H, 1.0).unwrap();
    let mut fresh = app_on(&world, &mut c, W, H, 1.0);
    let expected = settle(&mut fresh, &mut c);
    assert!(rebuilt == expected, "the rebuilt picture differs from a fresh one");
}

/// P2, DPI: the window moving to a screen of another scale resizes its renderer and sends Resized; the picture is
/// then the one a fresh application makes at that scale.
#[test]
fn p2_a_change_of_scale_gives_the_picture_of_that_scale() {
    let Some(world) = world() else { return };
    let mut r = Renderer::offscreen(W, H, 1.0).unwrap();
    let mut app = app_on(&world, &mut r, W, H, 1.0);
    settle(&mut app, &mut r);

    let (pw, ph) = (2100, 1410);
    r.resize(pw, ph, 1.5).unwrap();
    app.event(Event::Resized { width_px: pw, height_px: ph, scale: 1.5 });
    let moved = settle(&mut app, &mut r);

    let mut r2 = Renderer::offscreen(pw, ph, 1.5).unwrap();
    let mut fresh = app_on(&world, &mut r2, W, H, 1.5);
    let expected = settle(&mut fresh, &mut r2);
    assert!(moved == expected, "the picture after a change of scale differs from a fresh one at 1.5");
}

/// P3, no busy loop: while a run is under way the application asks for no frame by itself, says when to look again
/// (within a second), and `tick` follows the run without any renderer.
#[test]
fn p3_a_run_under_way_does_not_keep_the_window_drawing() {
    let Some(world) = world() else { return };
    let root = root("runs-p3");
    let mut state = PreviewState::for_world(Some(&world));
    state.seed = "p3-seed".to_string();
    let mut rig = Rig::new(&world, state, Some(runs(&root, &[("NUI_STUB", "hold")])));
    rig.park();
    let generate = rig.app.ids().generate;
    rig.click_on(generate);
    assert!(rig.app.running(), "the click started the generator");
    rig.park();

    assert!(!rig.app.frame(&mut rig.r, 0.0), "a frame while the run is under way asks for no other");
    let wait = rig.app.wake_in().expect("the run is looked at again");
    assert!(wait > 0.0 && wait <= 1.0, "the next look is within a second: {wait}");

    // No renderer from here on: only `tick`.
    let started = Instant::now();
    while rig.app.state().follow.as_ref().and_then(|f| f.part).is_none() {
        rig.app.tick();
        assert!(started.elapsed() < Duration::from_secs(30), "tick never read progress.json");
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(rig.app.running(), "still under way: tick reads, it does not end the run");
    let stop = rig.app.ids().stop;
    rig.click_on(stop);
    rig.until(15, "the run to end", |r| !r.app.running());
}

/// The sheet of the 16 icons (the pack's cube is the sixteenth) at 16 DIP, in --muted on the first row and --text on
/// the second, at the three scales a screen has. The icons are fully coloured device pixels where they are crisp.
#[test]
fn icon_sheets() {
    let t = Theme::dark();
    let i = preview::bar::icons();
    let mut crisp: Vec<&nui_ui::Icon> = vec![&i.home, &i.preview, &i.minimize, &i.maximize, &i.restore, &i.close];
    crisp.extend(i.views.iter());
    crisp.extend([&i.play, &i.stop, &i.settings]);
    assert_eq!(crisp.len() + 1, 16);

    for (scale, name) in [(1.0f32, "icons_x1.png"), (1.5, "icons_x15.png"), (2.0, "icons_x2.png")] {
        let (w, h) = (16.0 * 24.0 + 24.0, 2.0 * 24.0 + 24.0);
        let (pw, ph) = ((w * scale).round() as u32, (h * scale).round() as u32);
        let mut r = Renderer::offscreen(pw, ph, scale).unwrap();
        let mut list = DrawList::default();
        list.fill(Rect::new(0.0, 0.0, w, h), t.ground);
        for (row, color) in [t.muted, t.text].into_iter().enumerate() {
            let y = 12.0 + row as f32 * 24.0;
            for (n, icon) in crisp.iter().enumerate() {
                list.shape(Rect::new(12.0 + n as f32 * 24.0, y, 16.0, 16.0), icon.grid, icon.figures.clone(), color, true);
            }
            for (icon, face) in &i.pack {
                list.shape(Rect::new(12.0 + 15.0 * 24.0, y, 16.0, 16.0), icon.grid, icon.figures.clone(), *face, false);
            }
        }
        r.begin(t.ground);
        r.draw_list(&list);
        r.end();
        let rgba = r.read_rgba().unwrap();
        let path = tmp().join(name);
        std::fs::write(&path, nui_png::encode(pw, ph, &rgba)).unwrap();
        println!("wrote {}", path.display());

        // Each crisp icon box holds the ground and the icon's colour, nothing between.
        let (ground, muted) = (rgb(t.ground), rgb(t.muted));
        let first = Rect::new(12.0, 12.0, 16.0, 16.0);
        let mut seen = std::collections::BTreeSet::new();
        for yy in (first.y * scale) as u32..((first.y + first.h) * scale) as u32 {
            for xx in (first.x * scale) as u32..((first.x + first.w) * scale) as u32 {
                let k = (yy as usize * pw as usize + xx as usize) * 4;
                seen.insert([rgba[k], rgba[k + 1], rgba[k + 2]]);
            }
        }
        assert!(seen.iter().all(|p| *p == ground || *p == muted), "scale {scale}: a blended pixel in a crisp icon: {seen:?}");
    }
}
