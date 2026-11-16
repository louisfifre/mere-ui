//! Step 8's checks on the gallery: the page's grid rows and absolute boxes at the numbers worked out from the
//! CSS, nothing overlapping, and the picture `target/tmp/gallery.png`. Every expected number is derived by
//! hand from the page's rule (the line is cited in `src/gallery.rs`), not copied from the output.

use mere_base::Rect;
use mere_layout::{NodeId, Overflow, Position, Tree};
use shell_boxes::draw_boxes;
use shell_boxes::gallery::{GALLERY, build_gallery};

mod common;

fn near(a: Rect, x: f32, y: f32, w: f32, h: f32) {
    let ok = |p: f32, q: f32| (p - q).abs() < 0.01;
    assert!(ok(a.x, x) && ok(a.y, y) && ok(a.w, w) && ok(a.h, h), "got {a:?}, wanted ({x}, {y}, {w}, {h})");
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w - 0.01 && b.x < a.x + a.w - 0.01 && a.y < b.y + b.h - 0.01 && b.y < a.y + a.h - 0.01
}

/// In-flow siblings never overlap. Absolute boxes are skipped, and so are the children of a box that clips or
/// scrolls: they are allowed to reach outside it.
fn check_siblings(t: &Tree, n: NodeId) {
    if t.style(n).overflow != Overflow::Visible {
        return;
    }
    let kids: Vec<NodeId> = t.children(n).iter().copied().filter(|&k| t.style(k).position != Position::Absolute).collect();
    for (i, &a) in kids.iter().enumerate() {
        for &b in &kids[i + 1..] {
            assert!(!overlap(t.abs_rect(a), t.abs_rect(b)), "{} overlaps {}", t.name(a), t.name(b));
        }
    }
    for &k in t.children(n) {
        check_siblings(t, k);
    }
}

#[test]
fn settings_rows_split_label_output_and_field() {
    // `.setting`: columns minmax(0,1fr) auto 8ch (57.6), gap 10, in 320; padding 2 0, row 24 high (the field).
    // Row 2's output "1.00" is 28.8 wide: the label gets 320 - 20 - 28.8 - 57.6 = 213.6, the output starts at
    // 213.6 + 10 = 223.6, the field at 223.6 + 28.8 + 10 = 262.4. The 18 px label is centred in the 24 px row:
    // 2 (padding) + 3 = 5.
    let g = build_gallery();
    let t = &g.tree;
    let row = &g.settings[1];
    near(t.rect(row.label), 0.0, 5.0, 213.6, 18.0);
    near(t.rect(row.output), 223.6, 5.0, 28.8, 18.0);
    near(t.rect(row.field), 262.4, 2.0, 57.6, 24.0);
    // Row 3's hint (11 px, padding 2 0 4: 16.5 + 6 = 22.5) spans all columns, under the 24 px row: y = 2 + 24.
    near(t.rect(g.settings[2].hint.unwrap()), 0.0, 26.0, 320.0, 22.5);
}

#[test]
fn figures_align_on_the_baseline_and_the_rule_spans_all() {
    // Auto columns: values at 14 px are at most "38 %" = 4 x 8.4 = 33.6 wide, before values at 12 px at most
    // "41 %" = 28.8. The name column takes 330 - 33.6 - 28.8 - 2 x 14 = 239.6. Value x = 239.6 + 14 = 253.6,
    // before x = 253.6 + 33.6 + 14 = 301.2. In the first row the 14 px text has baseline 15.75 and the 12 px
    // one 13.5: the small one sits 15.75 - 13.5 = 2.25 lower. The rule: three rows of 21 and three gaps of 4
    // put its row at 63 + 12 = 75; its margin-top 4 gives y = 79; it spans all 330.
    let g = build_gallery();
    let t = &g.tree;
    let [name, value, before] = g.figure_row;
    near(t.rect(name), 0.0, 0.0, 239.6, 21.0);
    near(t.rect(value), 253.6, 0.0, 33.6, 21.0);
    near(t.rect(before), 301.2, 2.25, 28.8, 18.0);
    near(t.rect(g.rule), 0.0, 79.0, 330.0, 1.0);
}

#[test]
fn ore_rows_and_their_head_use_their_own_character_width() {
    // Row: 12, fr, 43.2, 36, 36, 14 with gap 10 in 320: fr = 128.8; inputs at 160.8, 214, 260. Row padding 2
    // and the 12 px swatch centred in a 22 px row: 2 + 5 = 7.
    // Head (11 px, ch 6.6): 12, fr, 39.6, 33, 33, 14: fr = 320 - 131.6 - 50 = 138.4, so "tries" starts at
    // 12 + 10 + 138.4 + 10 = 170.4: about 10 px right of the rows' inputs (160.8), as the page's CSS reads.
    let g = build_gallery();
    let t = &g.tree;
    let row = &g.ore_rows[0];
    near(t.rect(row.swatch), 0.0, 7.0, 12.0, 12.0);
    near(t.rect(row.inputs[0]), 160.8, 2.0, 43.2, 22.0);
    near(t.rect(row.inputs[1]), 214.0, 2.0, 36.0, 22.0);
    near(t.rect(row.inputs[2]), 260.0, 2.0, 36.0, 22.0);
    near(t.rect(g.ore_tries), 170.4, t.rect(g.ore_tries).y, 39.6, 16.5);
}

#[test]
fn biome_cells_keep_their_minimum_widths() {
    // Columns 12, fr, auto, auto, gap 8 in 330. The width cell is at least 8ch = 57.6, the share cell 6ch =
    // 43.2, and every text is narrower, so the autos are 57.6 and 43.2. fr = 330 - 12 - 57.6 - 43.2 - 24 =
    // 193.2: width x = 12 + 8 + 193.2 + 8 = 221.2, share x = 221.2 + 57.6 + 8 = 286.8.
    let g = build_gallery();
    let row = &g.biome_rows[0];
    near(g.tree.rect(row.width), 221.2, g.tree.rect(row.width).y, 57.6, 18.0);
    near(g.tree.rect(row.share), 286.8, g.tree.rect(row.share).y, 43.2, 18.0);
}

#[test]
fn the_run_block_has_three_choices_and_a_generate_row() {
    // Three minmax(0,1fr) in 320 with padding-left 1 and gap 4: (319 - 8) / 3 = 103.667 each, a stretched
    // button takes 1 px more through its margin-left -1: 104.667 at x 0, 107.667, 215.333.
    // `.go`: 320 - gap 6 - the 34 px square = 280 for #generate; the 12 px icon is centred in the square.
    let g = build_gallery();
    let t = &g.tree;
    for (i, x) in [0.0, 107.667, 215.333].into_iter().enumerate() {
        let r = t.rect(g.choices[i]);
        near(r, x, r.y, 104.667, r.h);
    }
    let gen_ = t.rect(g.generate);
    near(gen_, 0.0, gen_.y, 280.0, 34.0);
    near(t.rect(g.icon), 11.0, 11.0, 12.0, 12.0);
    near(t.rect(g.stop_run), 286.0, gen_.y, 34.0, 34.0);
}

#[test]
fn the_stage_stacks_its_frame_and_absolute_boxes() {
    // #stage 330 wide, place-items start center: #frame 196 (= min(330, 260 - 64)) at x (330 - 196) / 2 = 67.
    // #flat inset 0; #cutLine left/right -9, top 98, height 0: 196 + 18 = 214 wide from -9; #note inset 0 with
    // its text centred (see tests/position.rs: 44, 62, 108 x 72); #under at top 196 + 10, 196 wide, as tall
    // as its content: a 20 px row + 6 + 23 = 49.
    let g = build_gallery();
    let t = &g.tree;
    near(t.rect(g.frame), 67.0, 0.0, 196.0, 196.0);
    near(t.rect(g.flat), 0.0, 0.0, 196.0, 196.0);
    near(t.rect(g.cut_line), -9.0, 98.0, 214.0, 0.0);
    near(t.rect(g.note_text), 44.0, 62.0, 108.0, 72.0);
    near(t.rect(g.under), 0.0, 206.0, 196.0, 49.0);
    // the stage's content reaches the bottom of #under: 206 + 49 = 255, inside its 260.
    assert!((t.content_size(g.frame).y - 255.0).abs() < 0.01);
}

#[test]
fn the_scroll_box_is_cut_at_90_and_knows_its_content() {
    // 8 rows of 18 and padding-bottom 28: 172 of content in a 90 px box.
    let g = build_gallery();
    assert!((g.tree.rect(g.scroll).h - 90.0).abs() < 0.01);
    assert!((g.tree.content_size(g.scroll).y - 172.0).abs() < 0.01);
    assert_eq!(g.scroll_rows.len(), 8);
}

#[test]
fn everything_fits_the_sheet_and_nothing_overlaps() {
    let g = build_gallery();
    for &c in &g.cells {
        let r = g.tree.abs_rect(c);
        assert!(r.y + r.h <= GALLERY.1 - 20.0 + 0.01, "{} ends at {}", g.tree.name(c), r.y + r.h);
    }
    check_siblings(&g.tree, g.root);
}

#[test]
fn gallery_picture() {
    let g = build_gallery();
    println!("{}", g.tree.dump(g.root));
    let list = draw_boxes(&g.tree, g.root, GALLERY.0, GALLERY.1);
    common::write_picture(&list, GALLERY.0 as u32, GALLERY.1 as u32, "gallery.png");
}

/// Every label drawn lies inside its own box and inside the picture, and the content boxes keep their text.
#[test]
fn labels_stay_inside_their_box_and_the_picture() {
    let g = build_gallery();
    let labels = shell_boxes::label_rects(&g.tree, g.root, GALLERY.0, GALLERY.1);
    assert!(labels.len() > 20, "only {} labels", labels.len());
    for l in &labels {
        let b = g.tree.abs_rect(l.node);
        let inside = l.rect.x >= b.x - 0.01 && l.rect.y >= b.y - 0.01 && l.rect.x + l.rect.w <= b.x + b.w + 0.01 && l.rect.y + l.rect.h <= b.y + b.h + 0.01;
        assert!(inside, "label {:?} leaves its box {b:?}", l.text);
        assert!(l.rect.x >= 0.0 && l.rect.y >= 0.0 && l.rect.x + l.rect.w <= GALLERY.0 && l.rect.y + l.rect.h <= GALLERY.1, "label {:?} leaves the picture", l.text);
    }
    for (i, a) in labels.iter().enumerate() {
        for b in &labels[i + 1..] {
            assert!(!overlap(a.rect, b.rect), "labels {:?} and {:?} overlap", a.text, b.text);
        }
    }
    // A label never sits on the box of a descendant of its own node.
    fn descendants(t: &mere_layout::Tree, n: mere_layout::NodeId, out: &mut Vec<mere_layout::NodeId>) {
        for &c in t.children(n) {
            out.push(c);
            descendants(t, c, out);
        }
    }
    for l in &labels {
        let mut ds = Vec::new();
        descendants(&g.tree, l.node, &mut ds);
        for d in ds {
            let r = g.tree.abs_rect(d);
            if r.w <= 0.0 && r.h <= 0.0 {
                continue;
            }
            let r = mere_base::Rect::new(r.x, r.y, r.w.max(0.01), r.h.max(0.01));
            assert!(!overlap(l.rect, r), "label {:?} sits on its descendant {}", l.text, g.tree.name(d));
        }
    }
    let leaves = |name: &str| labels.iter().any(|l| g.tree.name(l.node) == name || l.text.starts_with(name));
    for name in ["64 blocks", "1 min 30 s", "Coal", "Plains", "Forest", "1.40"] {
        assert!(leaves(name), "no label for the leaf {name}");
    }
}
