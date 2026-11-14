//! The preview as an application: the shell built every frame, the map's picture painted and uploaded whenever
//! what it shows changes, and the 3D scene drawn into the stage's square under the interface.

use std::collections::BTreeSet;
use std::time::Instant;

use nui_base::{App, DrawList, Event, Gfx, Vec2, observe};
use nui_ui::{Id, Theme, Ui};

use crate::cut::{self, CUT_IMAGE, NAV_H, NAV_IMAGE, CutLook};
use crate::map::MAP_IMAGE;
use crate::paint::{View, paint};
use crate::scene;
use crate::section::Shown;
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
    /// The interface list of the last frame, which a test draws alone to see what the scene covered.
    last: DrawList,
    /// The cut being looked at, with its ground and the ores played on it: the heavy data, kept out of the state
    /// (which is cloned and compared at every build). None outside the section view.
    shown: Option<Shown>,
    /// The cut last shown, so that coming back to the same one keeps the look (the page's sameLine).
    last_at: Option<usize>,
    /// What the strip's picture was made for: the cut and its width. None until a first upload and after start.
    nav_painted: Option<(usize, u32)>,
    /// What the cut picture was made for: the cut, the blocks in sight, whether ores are drawn wide, the choice, and
    /// the layer's version.
    cut_painted: Option<CutKey>,
}

type CutKey = (usize, usize, usize, usize, usize, bool, BTreeSet<String>, u32);

impl PreviewApp {
    pub fn new(theme: Theme, dark: bool, size: Vec2, state: PreviewState, world: Option<World>) -> PreviewApp {
        // The look a caller gave (--cut-look) is for the cut it also named: it is not fresh at the first sync.
        let last_at = Some(state.cut_at);
        PreviewApp {
            ui: Ui::new(theme),
            theme,
            dark,
            size,
            state,
            world,
            ids: ShellIds::default(),
            painted: None,
            last: DrawList::default(),
            shown: None,
            last_at,
            nav_painted: None,
            cut_painted: None,
        }
    }

    pub fn ui(&self) -> &Ui {
        &self.ui
    }

    /// Where the parts stood in the last build.
    pub fn ids(&self) -> &ShellIds {
        &self.ids
    }

    /// The interface list of the last frame (what is drawn over the 3D scene).
    pub fn list(&self) -> &DrawList {
        &self.last
    }

    /// The view the uploaded picture was painted for.
    pub fn painted_view(&self) -> Option<View> {
        self.painted.as_ref().map(|p| p.0)
    }

    /// Whether the cut's picture was made with ores two blocks wide (a scale under 0.75); None before a first picture.
    pub fn cut_reach(&self) -> Option<bool> {
        self.cut_painted.as_ref().map(|k| k.5)
    }

    /// The cut being shown, when the section view has one.
    pub fn shown(&self) -> Option<&Shown> {
        self.shown.as_ref()
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
    /// world, the section, or the structures' view of a preview without structures).
    fn wanted(&self) -> Option<(View, bool, BTreeSet<String>)> {
        let world = self.world.as_ref()?;
        let view = View::from_index(self.state.view)?;
        // The 3D view drapes the same picture over the land.
        if view == View::Structures && world.structures.is_none() {
            return None;
        }
        // The land under the marks does not change when the marks do, so the structures' key has no choice.
        let chosen = if view == View::Structures { BTreeSet::new() } else { self.state.chosen[view.index()].clone() };
        Some((view, self.dark, chosen))
    }
}

impl PreviewApp {
    /// Keeps shown for the cut the state names (the page's showCut): made when the section view is first built
    /// and when another cut is asked for, dropped outside the section view. A cut that is not the one last looked
    /// at starts with a fresh look; coming back to the same one keeps it.
    fn sync_cut(&mut self) {
        let Some(section) = self.world.as_ref().filter(|w| self.state.view == 5 && !w.coarse()).and_then(|w| w.section.as_ref()) else {
            self.shown = None;
            return;
        };
        let at = self.state.cut_at.min(section.cuts.len() - 1);
        self.state.cut_at = at;
        if self.shown.as_ref().is_some_and(|s| s.at == at) {
            return;
        }
        if self.last_at != Some(at) {
            self.state.cut_look = CutLook::FRESH;
        }
        let started = Instant::now();
        self.shown = self.world.as_ref().and_then(|w| Shown::new(w, at));
        self.last_at = Some(at);
        observe!(Info, "cut {} of {}: ground and ore figures played in {} ms", at + 1, section.cuts.len(), started.elapsed().as_millis());
    }

    /// Uploads the strip and the part of the cut in sight when what they show has changed: the strip once per cut
    /// and width, the sheet whenever the sight, the choice or the drawn layer changes. Before the list is drawn.
    fn upload_cut(&mut self, gfx: &mut dyn Gfx) {
        let (Some(world), Some(shown)) = (self.world.as_ref(), self.shown.as_mut()) else { return };
        let Some(section) = world.section.as_ref() else { return };
        let Some(nav) = self.ui.rect(self.ids.nav).filter(|_| self.ids.nav != Id::ROOT) else { return };
        let width = nav.w.round() as u32;
        if self.nav_painted != Some((shown.at, width)) {
            let (rgba, _) = cut::nav_image(section, &shown.ground, width as usize);
            gfx.upload_image(NAV_IMAGE, width, NAV_H as u32, &rgba);
            self.nav_painted = Some((shown.at, width));
        }
        let look = self.state.cut_look;
        // The box the shell settled the look against; the laid-out rect of the cut view is the same, which the tests check.
        let (bw, bh) = self.ids.cut_box;
        let Some(sight) = cut::sight(look, bw, bh, section.length, shown.ground.rows) else { return };
        shown.draw_chunks(world, (sight.x0 >> 4) as i64 - 1, (sight.x1 >> 4) as i64 + 1);
        let key: CutKey = (shown.at, sight.x0, sight.x1, sight.r0, sight.r1, look.k < 0.75, self.state.chosen_ores.clone(), shown.play.version);
        if self.cut_painted.as_ref() != Some(&key) {
            let started = Instant::now();
            let cells = section.cuts[shown.at].cells.as_ref().expect("a shown cut has its cells");
            let rgba = cut::sheet(section, cells, &shown.ground, &shown.play.layer, sight, look.k, &self.state.chosen_ores);
            gfx.upload_image(CUT_IMAGE, sight.w() as u32, sight.h() as u32, &rgba);
            observe!(Debug, "cut picture {}x{} for blocks {}..{} in {} ms", sight.w(), sight.h(), sight.x0, sight.x1, started.elapsed().as_millis());
            self.cut_painted = Some(key);
        }
    }
}

impl App for PreviewApp {
    fn start(&mut self, gfx: &mut dyn Gfx) {
        self.painted = None;
        // A rebuilt device has lost the cut's pictures as well.
        self.nav_painted = None;
        self.cut_painted = None;
        // Also after a device rebuild, which has lost the meshes.
        if let Some(world) = &self.world {
            scene::upload(gfx, world);
        }
    }

    fn event(&mut self, e: Event) -> bool {
        if let Event::Resized { width_px, height_px, scale } = e {
            self.size = Vec2::new(width_px as f32 / scale, height_px as f32 / scale);
            return true;
        }
        self.ui.event(&e)
    }

    fn frame(&mut self, gfx: &mut dyn Gfx, seconds: f32) -> bool {
        // Before building, so the frame shows the settled pose.
        let moving = match (&mut self.state.orbit, &self.world) {
            (Some(orbit), Some(world)) => scene::settle(orbit, world, seconds),
            _ => false,
        };
        gfx.begin(self.theme.ground);
        // The scrollbar's gutter comes a frame late, and a click changes the state after its widget was
        // built: build again until neither is pending, at most three times, so a picture shows the settled page.
        let mut list = DrawList::default();
        for _ in 0..3 {
            let before = self.state.clone();
            self.sync_cut();
            self.ui.begin_frame(self.size);
            self.ids = build(&mut self.ui, &mut self.state, self.size, self.world.as_ref(), self.shown.as_ref());
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
        self.upload_cut(gfx);
        // The meshes come before the list: the interface is drawn over the scene, and the viewport clips it to the
        // square.
        if let (Some(rect), Some(orbit), Some((view, ..))) = (self.ui.rect(self.ids.relief).filter(|_| self.ids.relief != Id::ROOT), &self.state.orbit, &self.painted) {
            scene::draw(gfx, rect, orbit, MAP_IMAGE, scene::water_alpha(*view));
        }
        gfx.draw_list(&list);
        self.last = list;
        gfx.end();
        self.ui.want_redraw() || moving
    }
}
