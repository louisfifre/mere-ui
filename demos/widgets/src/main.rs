//! widgets [--controls | --fields | --lists] [--light] [--shot out.png]
//!
//! Without --shot, opens a window with the live widgets (--controls: the folds, tabs and number fields;
//! --fields: the text and number fields, typed into for real; --lists: the biome list with its scrollbar,
//! the wheel and the thumb working, and the trend rows). With it, draws one frame offscreen and writes
//! the PNG: the same frame the window would show.

use nui_base::{App, DrawList, Event, Gfx, Vec2, log};
use nui_ui::{Theme, Ui};
use widgets::{CONTROLS, ControlsState, FIELDS, FieldsState, LISTS, ListsState, SHEET, controls_live, fields_live, live, lists_live, lists_marks, text};

struct Widgets {
    ui: Ui,
    theme: Theme,
    size: Vec2,
    ons: [bool; 2],
    controls: Option<ControlsState>,
    fields: Option<FieldsState>,
    lists: Option<ListsState>,
}

impl App for Widgets {
    fn start(&mut self, _: &mut dyn Gfx) {}

    fn event(&mut self, e: Event) -> bool {
        if let Event::Resized { width_px, height_px, scale } = e {
            self.size = Vec2::new(width_px as f32 / scale, height_px as f32 / scale);
            return true;
        }
        self.ui.event(&e)
    }

    fn frame(&mut self, gfx: &mut dyn Gfx, _: f32) -> bool {
        gfx.begin(self.theme.ground);
        // The scrollbar's gutter is reserved a frame after the list overflows: the lists page builds again
        // before drawing, so that a picture (which is one frame) shows the settled page.
        let mut list = DrawList::default();
        for _ in 0..3 {
            self.ui.begin_frame(self.size);
            match (&mut self.controls, &mut self.fields, &mut self.lists) {
                (Some(s), _, _) => controls_live(&mut self.ui, s),
                (_, Some(s), _) => fields_live(&mut self.ui, s),
                (_, _, Some(s)) => lists_live(&mut self.ui, s),
                _ => live(&mut self.ui, &mut self.ons),
            }
            list = self.ui.end_frame().clone();
            if self.lists.is_none() || !self.ui.want_redraw() {
                break;
            }
        }
        gfx.draw_list(&list);
        if let Some(s) = &self.lists {
            gfx.draw_list(&lists_marks(&self.ui, s, self.theme));
        }
        gfx.end();
        // A Tab moves the focus after the picture is made: one more frame shows it.
        self.ui.want_redraw()
    }
}

fn usage() -> ! {
    eprintln!("usage: widgets [--controls | --fields | --lists] [--light] [--shot out.png]");
    std::process::exit(2);
}

fn main() {
    log::add_sink(log::stderr_sink(log::Level::Info));
    let (mut light, mut shot, mut controls, mut fields, mut lists) = (false, None, false, false, false);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--light" => light = true,
            "--controls" => controls = true,
            "--fields" => fields = true,
            "--lists" => lists = true,
            "--shot" => shot = Some(args.next().unwrap_or_else(|| usage())),
            _ => usage(),
        }
    }
    if [controls, fields, lists].iter().filter(|b| **b).count() > 1 {
        usage();
    }
    let theme = if light { Theme::light() } else { Theme::dark() };
    let (w, h) = if controls {
        CONTROLS
    } else if fields {
        FIELDS
    } else if lists {
        LISTS
    } else {
        SHEET
    };
    let mut ui = Ui::new(theme);
    ui.set_clipboard(Box::new(nui_win32::SystemClipboard));
    let mut app = Widgets {
        ui,
        theme,
        size: Vec2::new(w as f32, h as f32),
        ons: [false, true],
        controls: controls.then(ControlsState::default),
        fields: fields.then(FieldsState::default),
        lists: lists.then(ListsState::default),
    };
    match shot {
        Some(path) => {
            let rgba = nui_win32::shot(&mut app, w, h, 1.0).unwrap_or_else(|e| {
                eprintln!("widgets: {e}");
                std::process::exit(1);
            });
            if let Err(e) = std::fs::write(&path, nui_png::encode(w, h, &rgba)) {
                eprintln!("widgets: {path}: {e}");
                std::process::exit(1);
            }
            nui_base::observe!(Info, "wrote {path}");
        }
        None => {
            if let Err(e) = nui_win32::run(text::WINDOW_TITLE, &mut app) {
                eprintln!("widgets: {e}");
                std::process::exit(1);
            }
        }
    }
}
