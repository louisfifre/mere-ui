//! The preview as an application: the shell built every frame, the map's picture painted and uploaded whenever
//! what it shows changes, and the 3D scene drawn into the stage's square under the interface.

use std::collections::BTreeSet;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Child, ExitStatus, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use nui_base::{App, DrawList, Event, Gfx, Vec2, observe};
use nui_ui::{Id, Theme, Ui};

use crate::cut::{self, CUT_IMAGE, NAV_H, NAV_IMAGE, CutLook};
use crate::figures::Figures;
use crate::history;
use crate::json::Json;
use crate::map::MAP_IMAGE;
use crate::paint::{View, paint};
use crate::run::{self, Ask, DETAIL_BLOCKS, Follow, Pace, Runs, Steps};
use crate::scene;
use crate::section::Shown;
use crate::shell::{PreviewState, ShellIds, Want, build};
use crate::text;
use crate::world::World;

/// A generator process being followed.
struct Running {
    child: Child,
    started: Instant,
    /// When the progress file was last read: it is read once a second, the first time one second after the start
    /// (line 2649).
    looked: Instant,
}

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
    /// How and where runs happen; None for a window that only looks (Generate then says it failed).
    runs: Option<Runs>,
    running: Option<Running>,
    /// What this machine measured: the file's content, which the app owns (the state holds a copy for the shell).
    pace: Pace,
    /// The sheet last uploaded for the cut, as pixels: what an export keeps as section.png.
    last_sheet: Option<(u32, u32, Vec<u8>)>,
    /// A run asked for while another was under way: it starts when that one has ended (line 2696).
    queued: bool,
    /// The window follows the system's light or dark setting (Event::Scheme); a fixed theme ignores it.
    follows_scheme: bool,
}

type CutKey = (usize, usize, usize, usize, usize, bool, BTreeSet<String>, u32);

impl PreviewApp {
    pub fn new(theme: Theme, dark: bool, size: Vec2, mut state: PreviewState, world: Option<World>) -> PreviewApp {
        // What the map was made from is a fact of the world, whatever the form says.
        state.made = world.as_ref().and_then(run::ask_of);
        // The look a caller gave (--cut-look) is for the cut it also named: it is not fresh at the first sync.
        let last_at = Some(state.cut_at);
        let mut app = PreviewApp {
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
            runs: None,
            running: None,
            pace: Pace::default(),
            last_sheet: None,
            queued: false,
            follows_scheme: false,
        };
        // Line 2815: the map that opens the page is kept like any other (in memory until there are runs).
        app.remember();
        app
    }

    /// Lets the window run a generator: the learned pace is read from the runs' folder, and so are the history
    /// and the figures of the run before, which the page keeps in its localStorage.
    pub fn with_runs(mut self, runs: Runs) -> PreviewApp {
        self.pace = Pace::load(&runs.pace());
        self.state.pace = self.pace.clone();
        self.state.history = history::load(&runs.history());
        self.state.before = std::fs::read_to_string(runs.before()).ok().and_then(|t| Json::parse(&t).ok()).and_then(|j| Figures::from_json(&j));
        self.runs = Some(runs);
        self.remember();
        self
    }

    /// `remember` (1546-1560): the map on screen is added to the history unless it is the one kept last, and the
    /// file is written when there are runs to keep it in. There is no quota natively, so nothing is evicted.
    fn remember(&mut self) {
        let (Some(world), Some(made)) = (&self.world, &self.state.made) else { return };
        let at = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0.0, |d| d.as_millis() as f64);
        let entry = history::entry(made, &world.measured.figures, at);
        if !history::remember(&mut self.state.history, entry) {
            return;
        }
        if let Some(runs) = &self.runs {
            if let Err(e) = history::save(&runs.history(), &self.state.history) {
                observe!(Warn, "the history could not be kept in {}: {e}", runs.history().display());
            }
        }
    }

    /// Makes the app switch theme when the system's setting changes, as the page does with prefers-color-scheme.
    pub fn follow_scheme(mut self, yes: bool) -> PreviewApp {
        self.follows_scheme = yes;
        self
    }

    /// Switches to the dark or the light theme. The painted key holds dark, so the map and the drape repaint by
    /// themselves; the cut and the strip do not depend on the theme.
    pub fn set_dark(&mut self, dark: bool) {
        self.theme = if dark { Theme::dark() } else { Theme::light() };
        self.ui.set_theme(self.theme);
        self.dark = dark;
    }

    /// Whether a generator is under way.
    pub fn running(&self) -> bool {
        self.running.is_some()
    }

    /// The generator's process id while it is under way.
    pub fn run_pid(&self) -> Option<u32> {
        self.running.as_ref().map(|r| r.child.id())
    }

    /// Makes the run `seconds` old, as the page's `already` does for a run it found under way: the clock, the
    /// button's fill and the hairline then read from that age. A test uses it to see a run at 40 %.
    pub fn set_run_age(&mut self, seconds: u64) {
        if let Some(r) = self.running.as_mut() {
            r.started = Instant::now().checked_sub(Duration::from_secs(seconds)).unwrap_or_else(Instant::now);
        }
    }

    /// The sheet last uploaded for the cut: its width, its height and its RGBA rows.
    pub fn last_sheet(&self) -> Option<&(u32, u32, Vec<u8>)> {
        self.last_sheet.as_ref()
    }

    pub fn world(&self) -> Option<&World> {
        self.world.as_ref()
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
            self.last_sheet = Some((sight.w() as u32, sight.h() as u32, rgba));
            observe!(Debug, "cut picture {}x{} for blocks {}..{} in {} ms", sight.w(), sight.h(), sight.x0, sight.x1, started.elapsed().as_millis());
            self.cut_painted = Some(key);
        }
    }
}

impl PreviewApp {
    /// The form as the next run would take it.
    fn asked(&self) -> Ask {
        Ask { seed: self.state.seed.trim().to_string(), samples: self.state.samples, detail: DETAIL_BLOCKS[self.state.detail], values: self.state.values }
    }

    /// Looks at the generator: whether it has ended, how old it is, and (once a second) what its progress file says.
    /// Before the frame is built, so the frame shows what was found.
    fn poll(&mut self, gfx: &mut dyn Gfx) {
        let Some(run) = self.running.as_mut() else { return };
        match run.child.try_wait() {
            Ok(Some(status)) => return self.finish(Some(status), gfx),
            Ok(None) => {}
            Err(e) => {
                observe!(Warn, "the generator could not be looked at: {e}");
                return self.finish(None, gfx);
            }
        }
        let (Some(follow), Some(runs)) = (self.state.follow.as_mut(), self.runs.as_ref()) else { return };
        follow.seconds = run.started.elapsed().as_secs();
        if run.looked.elapsed() >= Duration::from_secs(1) {
            let before = follow.pass_key.clone();
            follow.look(run::read_progress(&runs.out()).as_ref());
            run.looked = Instant::now();
            if follow.pass_key != before {
                observe!(Debug, "run pass '{}' at {} s", follow.pass_key, follow.seconds);
            }
        }
    }

    /// The generator ended (`follow`'s end, lines 2664-2696): a stopped run leaves the map it found, a failed one
    /// says so, a finished one is read and shown, and the pace learns from it.
    fn finish(&mut self, status: Option<ExitStatus>, gfx: &mut dyn Gfx) {
        let (Some(run), Some(mut follow)) = (self.running.take(), self.state.follow.take()) else { return };
        let Some(runs) = self.runs.as_ref() else { return };
        let (out, pace_file, before_file, log_file) = (runs.out(), runs.pace(), runs.before(), runs.log());
        let seconds = run.started.elapsed().as_secs();
        follow.seconds = seconds;
        let mut finished = false;
        if follow.stopping {
            self.state.trouble.clear();
            self.state.said = text::STOPPED.to_string();
        } else if status.is_some_and(|s| s.success()) && out.join("preview.json").exists() {
            match World::load(&out) {
                Ok(world) => {
                    // The camera starts again when the area's width changed (line 1873).
                    if self.world.as_ref().map(World::span) != Some(world.span()) {
                        self.state.orbit = None;
                    }
                    scene::upload(gfx, &world);
                    // The figures of the map being replaced are the run before (lines 2656-2658).
                    if let Some(old) = self.world.as_ref().map(|w| w.measured.figures) {
                        self.state.before = Some(old);
                        if let Err(e) = std::fs::write(&before_file, old.to_json()) {
                            observe!(Warn, "the figures of the run before could not be kept: {e}");
                        }
                    }
                    self.world = Some(world);
                    self.painted = None;
                    self.nav_painted = None;
                    self.cut_painted = None;
                    self.shown = None;
                    self.last_at = None;
                    self.last_sheet = None;
                    // `loadWorld` puts the run's settings back in the form.
                    self.state.made = self.world.as_ref().and_then(run::ask_of);
                    if let Some(made) = self.state.made.clone() {
                        run::adopt(&mut self.state, &made);
                    }
                    let last = if follow.stage == text::STEP_START { text::step_last(follow.ask.detail) } else { follow.stage.clone() };
                    follow.done.push((last, seconds));
                    self.state.trouble.clear();
                    self.state.said = text::done(&run::duration(seconds));
                    self.pace.learn(follow.ask.samples, follow.ask.detail, seconds as f64, run::work_seconds(&out));
                    if let Err(e) = self.pace.save(&pace_file) {
                        observe!(Warn, "the pace could not be kept: {e}");
                    }
                    self.state.pace = self.pace.clone();
                    // Line 2661: the new map is kept in the history.
                    self.remember();
                    finished = true;
                }
                Err(e) => {
                    observe!(Warn, "the run's result could not be read: {e}");
                    self.state.trouble = text::FAILED.to_string();
                }
            }
        } else {
            observe!(Warn, "the run failed: {} (see {})", status.map_or("no status".to_string(), |s| s.to_string()), log_file.display());
            self.state.trouble = text::FAILED.to_string();
            self.state.said.clear();
        }
        // The steps stay, to see where the time went; the one that did not end is left out.
        self.state.steps = Steps::Run { done: follow.done.clone(), now: String::new(), share: if finished { 1.0 } else { follow.share() } };
        observe!(Info, "run ended after {seconds} s: {}", if finished { "finished" } else if follow.stopping { "stopped" } else { "failed" });
        // A run asked for while this one was under way starts now that it has stopped (line 2696).
        if std::mem::take(&mut self.queued) {
            self.start();
        }
    }

    /// Acts on a press the shell recorded.
    fn act(&mut self, want: Want) {
        match want {
            Want::Generate => {
                if let Some(follow) = &self.state.follow {
                    // The run under way is the one on screen: the button only says so (line 2721). A setting moved:
                    // the new run waits for this one to stop, then starts by itself.
                    if !follow.ask.same_as(&self.state.seed, self.state.samples, DETAIL_BLOCKS[self.state.detail], &self.state.values) {
                        self.queued = true;
                        self.stop();
                    }
                } else {
                    self.start();
                }
            }
            Want::Stop => self.stop(),
            Want::Export => self.export(),
            Want::OpenFolder => {
                if let (Steps::Saved { folder }, Some(runs)) = (&self.state.steps, &self.runs) {
                    if let Err(e) = runs.launcher.open_folder(&PathBuf::from(folder)) {
                        observe!(Warn, "the folder could not be opened: {e}");
                        self.state.trouble = text::OPEN_FAILED.to_string();
                    }
                }
            }
        }
    }

    /// `generate()` (lines 2699-2710): an empty seed is one drawn here, a seed the server would refuse is refused here,
    /// then the generator starts with its output in the runs' folder.
    fn start(&mut self) {
        let Some(generator) = self.runs.as_ref().and_then(|r| r.generator.clone()) else {
            observe!(Warn, "no --generator given: a run cannot start");
            self.state.trouble = text::FAILED.to_string();
            return;
        };
        let runs = self.runs.as_ref().expect("a generator comes from the runs");
        if self.state.seed.trim().is_empty() {
            self.state.seed = run::random_seed();
        }
        let ask = self.asked();
        self.state.seed = ask.seed.clone();
        if !run::seed_ok(&ask.seed) {
            self.state.trouble = text::REFUSED.to_string();
            self.state.said.clear();
            return;
        }
        let (out, server) = (runs.out(), runs.server());
        // As the server does before it takes a run (lines ~300-335): the sketch and the progress of the run before.
        for name in ["coarse.json", "progress.json", "partial.json"] {
            let _ = std::fs::remove_file(out.join(name));
        }
        let started = (|| -> std::io::Result<Child> {
            std::fs::create_dir_all(&out)?;
            std::fs::create_dir_all(&server)?;
            let log = File::create(runs.log())?;
            let mut cmd = run::command(&runs.shell, &generator, runs.pack.as_deref(), &server, &out, &ask);
            cmd.stdin(Stdio::null()).stdout(log.try_clone()?).stderr(log).envs(runs.env.iter().map(|(k, v)| (k, v)));
            runs.launcher.spawn(&mut cmd)
        })();
        match started {
            Ok(child) => {
                observe!(Info, "run started: pid {}, seed {}, samples {}, detail {}, out {}", child.id(), ask.seed, ask.samples, ask.detail, out.display());
                let expected = self.pace.about(ask.samples, ask.detail);
                self.state.trouble.clear();
                self.state.said.clear();
                self.state.follow = Some(Follow::new(ask, expected));
                self.state.steps = Steps::Run { done: Vec::new(), now: text::STEP_START.to_string(), share: 0.0 };
                let now = Instant::now();
                self.running = Some(Running { child, started: now, looked: now });
            }
            Err(e) => {
                observe!(Warn, "the generator could not be started: {e}");
                self.state.trouble = text::FAILED.to_string();
            }
        }
    }

    /// The square beside the button (lines 2712-2715): the run ends a moment later and the wait for it says so.
    fn stop(&mut self) {
        let (Some(run), Some(runs), Some(follow)) = (self.running.as_mut(), self.runs.as_ref(), self.state.follow.as_mut()) else { return };
        follow.stopping = true;
        if let Err(e) = runs.launcher.stop(&mut run.child) {
            observe!(Warn, "the run could not be stopped: {e}");
        }
    }

    /// Export (lines 2736-2796), the 2D pictures only: the whole map of each view whatever is chosen, then the cut as
    /// it was last looked at. Synchronous, so the page's "Pictures: view" steps are not shown one by one.
    fn export(&mut self) {
        if self.state.follow.is_some() {
            return;
        }
        let (Some(world), Some(runs)) = (self.world.as_ref(), self.runs.as_ref()) else {
            if self.world.is_some() {
                observe!(Warn, "no --runs: an export has nowhere to go");
                self.state.trouble = text::EXPORT_FAILED.to_string();
            }
            return;
        };
        self.state.trouble.clear();
        self.state.said.clear();
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let folder = runs.exports().join(run::folder_name(self.state.made.as_ref().map(|m| m.seed.as_str()), secs));
        // A preview stopped at a coarser detail has no flat land to give; structures need the marks' rasteriser.
        let kinds: Vec<View> = [View::Biomes, View::Flat, View::Height, View::Temperature].into_iter().filter(|k| !(*k == View::Flat && world.coarse())).collect();
        let n = world.samples as u32;
        let mut steps = Vec::new();
        let result = (|| -> std::io::Result<()> {
            std::fs::create_dir_all(&folder)?;
            for kind in &kinds {
                let rgba = paint(world, *kind, self.dark, &BTreeSet::new());
                std::fs::write(folder.join(format!("{}.png", kind.key())), nui_png::encode(n, n, &rgba))?;
                steps.push((text::export_step(kind.key()), 0));
            }
            if let Some((w, h, rgba)) = &self.last_sheet {
                std::fs::write(folder.join("section.png"), nui_png::encode(*w, *h, rgba))?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                observe!(Info, "exported {} pictures to {}", kinds.len() + usize::from(self.last_sheet.is_some()), folder.display());
                self.state.said = text::EXPORTED.to_string();
                self.state.steps = Steps::Saved { folder: folder.display().to_string() };
            }
            Err(e) => {
                observe!(Warn, "the export failed: {e}");
                self.state.trouble = text::EXPORT_FAILED.to_string();
                let share = steps.len() as f32 / (kinds.len() + 1) as f32;
                self.state.steps = Steps::Run { done: steps, now: String::new(), share };
            }
        }
    }
}

impl App for PreviewApp {    fn start(&mut self, gfx: &mut dyn Gfx) {
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
        if let Event::Scheme { dark } = e {
            observe!(Info, "the system is now {}{}", if dark { "dark" } else { "light" }, if self.follows_scheme { "" } else { " (not followed)" });
            if self.follows_scheme && dark != self.dark {
                self.set_dark(dark);
                return true;
            }
            return false;
        }
        self.ui.event(&e)
    }

    fn frame(&mut self, gfx: &mut dyn Gfx, seconds: f32) -> bool {
        self.poll(gfx);
        // Before building, so the frame shows the settled pose.
        let moving = match (&mut self.state.orbit, &self.world) {
            (Some(orbit), Some(world)) => scene::settle(orbit, world, seconds),
            _ => false,
        };
        gfx.begin(self.theme.ground);
        // The scrollbar's gutter comes a frame late, and a click changes the state after its widget was
        // built: build again until neither is pending, at most three times, so a picture shows the settled page.
        let mut list = DrawList::default();
        // A press the shell recorded is acted on here, after its build, and the page is built again to show it.
        for _ in 0..4 {
            let before = self.state.clone();
            self.sync_cut();
            self.ui.begin_frame(self.size);
            self.ids = build(&mut self.ui, &mut self.state, self.size, self.world.as_ref(), self.shown.as_ref());
            list = self.ui.end_frame().clone();
            if let Some(want) = self.state.want.take() {
                self.act(want);
            }
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
        self.ui.want_redraw() || moving || self.running.is_some()
    }
}

impl Drop for PreviewApp {
    /// Quitting stops the run under way (serve.ps1's `finally`): the generator would leave a game server behind.
    fn drop(&mut self) {
        if let (Some(mut run), Some(runs)) = (self.running.take(), self.runs.as_ref()) {
            if let Err(e) = runs.launcher.stop(&mut run.child) {
                observe!(Warn, "the run could not be stopped on quitting: {e}");
            }
            let _ = run.child.wait();
        }
    }
}
