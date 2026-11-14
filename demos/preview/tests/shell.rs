//! The shell's layout without a GPU. The column positions are the page's wide layout (preview.html lines 75-81)
//! at every width: the panels are fixed since step 19. The sizes inside are derived in each test's comment.

use nui_base::{Rect, Vec2};
use nui_ui::{GUTTER, Id, Theme, Ui};
use preview::settings::grouped;
use preview::world::World;
use preview::{BAR_H, Columns, PAGE, PreviewState, ShellIds, picture, picture_with};

const H: f32 = 940.0;

fn shell(w: f32, h: f32) -> (Ui, ShellIds, Columns) {
    let (_, ui, ids) = picture(Theme::dark(), Vec2::new(w, h));
    (ui, ids, PAGE)
}

/// The shell with the real preview on the map, or None (with the reason printed) without the data.
fn shell_real(w: f32, h: f32) -> Option<(Ui, ShellIds, World)> {
    let dir = std::path::Path::new(r"C:\Games\preview\out");
    if !dir.join("preview.json").exists() {
        println!("skipped: no C:\\Games\\preview\\out");
        return None;
    }
    let world = World::load(dir).expect("the data loads");
    let (_, ui, ids) = picture_with(Theme::dark(), Vec2::new(w, h), Some(&world), PreviewState::for_world(Some(&world)));
    Some((ui, ids, world))
}

fn r(ui: &Ui, id: Id) -> Rect {
    ui.rect(id).expect("the node was built")
}

fn near(a: f32, b: f32, what: &str) {
    assert!((a - b).abs() < 0.01, "{what}: {a} != {b}");
}

fn near_r(ui: &Ui, id: Id, x: f32, w: f32, what: &str) {
    let q = r(ui, id);
    near(q.x, x, &format!("{what} x"));
    near(q.w, w, &format!("{what} w"));
}

#[test]
fn l1_columns() {
    // The panels never move: left at 28, right against the right edge (W - 28 - 330), the middle what is left.
    for (w, left, middle, right) in [
        (1400.0, (28.0, 320.0), (376.0, 638.0), (1042.0, 330.0)),
        (1200.0, (28.0, 320.0), (376.0, 438.0), (842.0, 330.0)),
        (1000.0, (28.0, 320.0), (376.0, 238.0), (642.0, 330.0)),
    ] {
        let (ui, ids, c) = shell(w, H);
        let y = BAR_H + c.pad.top;
        let h = H - BAR_H - c.pad.top - c.pad.bottom;
        for (id, (x, cw), name) in [(ids.left, left, "left"), (ids.middle, middle, "middle"), (ids.right, right, "right")] {
            near_r(&ui, id, x, cw, &format!("{name} at {w}"));
            near(r(&ui, id).y, y, &format!("{name} y at {w}"));
            near(r(&ui, id).h, h, &format!("{name} h at {w}"));
        }
    }
}

#[test]
fn l3_sections_in_order() {
    let (ui, ids, _) = shell(1400.0, H);
    let (left, right) = (r(&ui, ids.left), r(&ui, ids.right));
    assert!(r(&ui, ids.world_head).y < r(&ui, ids.settings_head).y);
    assert!(r(&ui, ids.settings_head).y < r(&ui, ids.run).y);
    near(r(&ui, ids.run).y + r(&ui, ids.run).h, left.y + left.h, "run bottom");
    assert!(r(&ui, ids.figures_head).y < r(&ui, ids.history_head).y);
    assert!(r(&ui, ids.history_head).y < r(&ui, ids.list_block).y);
    near(r(&ui, ids.list_block).y + r(&ui, ids.list_block).h, right.y + right.h, "list bottom");
    // Under each fold's 1 px rule.
    near(r(&ui, ids.world_head).y, left.y + 1.0, "world head");
    near(r(&ui, ids.figures_head).y, right.y + 1.0, "figures head");
}

#[test]
fn l4_frame_and_under() {
    for (w, side) in [(1400.0, 638.0), (1200.0, 438.0), (1000.0, 238.0)] {
        let (ui, ids, _) = shell(w, H);
        let (m, f) = (r(&ui, ids.middle), r(&ui, ids.frame));
        near(f.w, side, "side");
        near(f.h, side, "side h");
        near(f.x, m.x + (m.w - side) / 2.0, "frame x");
        near(f.y, m.y, "frame y");
        near(r(&ui, ids.under).y, f.y + side + 10.0, "under y");
        let e = r(&ui, ids.export);
        near(e.x + e.w, m.x + m.w, "export right edge");
    }
    // A short window: the map gives way to the strip, side = 600 - 40 - 40 - 64.
    let (ui, ids, _) = shell(1400.0, 600.0);
    let f = r(&ui, ids.frame);
    near(f.w, 456.0, "short side");
    near(f.x, 376.0 + 91.0, "short frame x");
    near(r(&ui, ids.under).y, f.y + 456.0 + 10.0, "short under y");
    near(r(&ui, ids.export).x + r(&ui, ids.export).w, f.x + f.w, "export right edge on the frame");
}

#[test]
fn l5_settings_groups_and_rows() {
    let (ui, ids, _) = shell(1400.0, H);
    let heads: Vec<Rect> = ids.group_heads.iter().map(|&i| r(&ui, i)).collect();
    assert_eq!(heads.len(), 5);
    for (g, q) in heads.iter().enumerate() {
        // 4 (14 between groups) + 16.5 + 4 + the 1 px hairline.
        near(q.h, if g == 0 { 25.5 } else { 35.5 }, "head height");
        if g > 0 {
            assert!(q.y > heads[g - 1].y);
        }
    }
    assert_eq!(ids.rows.len(), 43);
    let groups = grouped();
    for (g, members) in groups.iter().enumerate() {
        for (k, &i) in members.iter().enumerate() {
            // 2 + 24 + 2: the field is 18 + 2 x 3 + 2 x 1 high.
            near(r(&ui, ids.rows[i]).h, 28.0, "row height");
            if k > 0 {
                near(r(&ui, ids.rows[i]).y - r(&ui, ids.rows[members[k - 1]]).y, 28.0, "row pitch");
            }
        }
        let first = r(&ui, ids.rows[members[0]]);
        near(first.y, heads[g].y + heads[g].h + 4.0, "first row under its head");
    }
}

#[test]
fn l6_gutter_is_reserved() {
    let (ui, ids, c) = shell(1400.0, H);
    let left = r(&ui, ids.left);
    let first = r(&ui, ids.rows[0]);
    near(first.x + first.w, left.x + left.w - GUTTER, "row right edge");
    assert_eq!(first.x + first.w, 337.0);
    let _ = c;
}

#[test]
fn l7_world_controls() {
    let (ui, ids, _) = shell(1400.0, H);
    let left = r(&ui, ids.left);
    near(r(&ui, ids.select).h, 23.0, "select h");
    near(r(&ui, ids.select).w, left.w - 108.0 - 10.0, "select w");
    near(r(&ui, ids.seed).h, 26.0, "seed h");
}

#[test]
fn l8_rows() {
    // The relief view's list: one row of 24 px (3 + 18 + 3) for each band the area has, lowest first.
    let Some((ui, ids, world)) = shell_real(1400.0, H) else { return };
    let list = world.measured.lists[2].as_ref().expect("the height list");
    let present = list.cells.iter().filter(|&&c| c > 0.0).count();
    assert_eq!(ids.list_rows.len(), present);
    assert!(present >= 3, "a real area has several bands");
    for (i, &b) in ids.list_rows.iter().enumerate() {
        near(r(&ui, b).h, 24.0, "row h");
        if i > 0 {
            near(r(&ui, b).y - r(&ui, ids.list_rows[i - 1]).y, 24.0, "row pitch");
        }
    }
    let scroll = r(&ui, ids.list_scroll);
    let last = r(&ui, *ids.list_rows.last().unwrap());
    // At most 8 x 24 = 192 of rows and the 28 px of padding for the fade fit without overflow.
    assert!(last.y + last.h <= scroll.y + scroll.h, "the rows overflow their box");
    let block = r(&ui, ids.list_block);
    assert!(scroll.y + scroll.h <= block.y + block.h + 0.01);
}

#[test]
fn l9_goal_values() {
    // The goal figures are 14 px at every width, on a line of 1.5.
    for (w, size) in [(1400.0, 21.0), (1200.0, 21.0), (1000.0, 21.0)] {
        let Some((ui, ids, _)) = shell_real(w, H) else { return };
        assert_eq!(ids.goal_values.len(), 3);
        for &g in &ids.goal_values {
            near(r(&ui, g).h, size, &format!("goal at {w}"));
        }
    }
}
#[test]
fn l10_run_block() {
    let (ui, ids, _) = shell(1400.0, H);
    let left = r(&ui, ids.left);
    let d = r(&ui, ids.details);
    near(d.h, 26.0, "details h");
    let g = r(&ui, ids.generate);
    near(g.h, 34.0, "generate h");
    near(g.w, left.w, "generate w");
}
