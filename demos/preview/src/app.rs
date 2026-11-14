//! The preview as an application: the shell built every frame, and the map's picture painted and uploaded
//! whenever what it shows changes.

use std::collections::BTreeSet;
use std::time::Instant;

use nui_base::{App, DrawList, Event, Gfx, Vec2, observe};
use nui_ui::{Theme, Ui};

use crate::map::MAP_IMAGE;
use crate::paint::{View, paint};
use crate::shell::{PreviewState, ShellIds, build};
use crate::world::World;

pub struct PreviewApp {
    ui: Ui,
    theme: Theme,
    dark: bool,
    size: Vec2,
    state: PreviewState,
    world: Option<World>,
    ids: ShellIds,
    /// What the uploaded picture was painted for: the view, the theme and the choice. None until a first
    /// upload, and again after `start`, because a rebuilt device has lost its images.
    painted: Option<(View, bool, BTreeSet<String>)>,
}

impl PreviewApp {
    pub fn new(theme: Theme, dark: bool, size: Vec2, state: PreviewState, world: Option<World>) -> PreviewApp {
        PreviewApp { ui: Ui::new(theme), theme, dark, size, state, world, ids: ShellIds::default(), painted: None }
    }

    pub fn ui(&self) -> &Ui {
        &self.ui
    }

    /// Where the parts stood in the last build.
    pub fn ids(&self) -> &ShellIds {
        &self.ids
    }

    pub fn state(&self) -> &PreviewState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut PreviewState {
        &mut self.state
    }

    pub fn ui_mut(&mut self) -> &mut Ui {
        &mut self.ui
    }

    /// What the map's picture should be painted for now, or None when the frame shows no map picture (no
    /// world, the 3D view, the section, or the structures' view of a preview without structures).
    fn wanted(&self) -> Option<(View, bool, BTreeSet<String>)> {
        let world = self.world.as_ref()?;
        let view = View::from_index(self.state.view)?;
        if self.state.in3d || (view == View::Structures && world.structures.is_none()) {
            return None;
        }
        // The land under the marks does not change when the marks do, so the structures' key has no choice.
        let chosen = if view == View::Structures { BTreeSet::new() } else { self.state.chosen[view.index()].clone() };
        Some((view, self.dark, chosen))
    }
}

impl App for PreviewApp {
    fn start(&mut self, _: &mut dyn Gfx) {
        self.painted = None;
    }

    fn event(&mut self, e: Event) -> bool {
        if let Event::Resized { width_px, height_px, scale } = e {
            self.size = Vec2::new(width_px as f32 / scale, height_px as f32 / scale);
            return true;
        }
        self.ui.event(&e)
    }

    fn frame(&mut self, gfx: &mut dyn Gfx, _: f32) -> bool {
        gfx.begin(self.theme.ground);
        // The scrollbar's gutter comes a frame late, and a click changes the state after its widget was
        // built: build again until neither is pending, at most three times, so a picture shows the settled page.
        let mut list = DrawList::default();
        for _ in 0..3 {
            let before = self.state.clone();
            self.ui.begin_frame(self.size);
            self.ids = build(&mut self.ui, &mut self.state, self.size, self.world.as_ref());
            list = self.ui.end_frame().clone();
            if !self.ui.want_redraw() && self.state == before {
                break;
            }
        }
        // Before the list is drawn, so that the first frame already shows the picture.
        if let (Some(world), Some(key)) = (&self.world, self.wanted()) {
            if self.painted.as_ref() != Some(&key) {
                let started = Instant::now();
                let rgba = paint(world, key.0, key.1, &key.2);
                let n = world.samples as u32;
                gfx.upload_image(MAP_IMAGE, n, n, &rgba);
                observe!(
                    Info,
                    "painted {} {} choice {:?} {n}x{n} in {} ms",
                    key.0.key(),
                    if key.1 { "dark" } else { "light" },
                    key.2,
                    started.elapsed().as_millis()
                );
                self.painted = Some(key);
            }
        }
        gfx.draw_list(&list);
        gfx.end();
        self.ui.want_redraw()
    }
}
