//! The relief as an application: the textured land, the sheet of water, the orbit camera and a small
//! overlay of facts about the area.

use crate::world::World;
use crate::{mesh, paint, text};
use nui_base::{App, Button, Color, DrawList, Event, Gfx, ImageId, Material, MeshId, Orbit, Rect, Vec2, Vec3};

const LAND: MeshId = MeshId(1);
const WATER: MeshId = MeshId(2);
const MAP: ImageId = ImageId(1);

// The page's colours: its black ground (the dark theme's --ground), its water, and the interface's hairline,
// text and muted text.
const GROUND: Color = Color::hex(0x000000);
const WATER_COLOUR: Color = Color::hex(0x2f5c9e).with_alpha(0.25);
const LINE: Color = Color::hex(0x2a2a2a);
const INK: Color = Color::hex(0xececec);
const MUTED: Color = Color::hex(0x8a8a8a);
// Behind the overlay's text, so it stays legible over bright snow.
const PLATE: Color = Color::hex(0x0b0b0b).with_alpha(0.78);

pub struct Relief {
    world: World,
    orbit: Orbit,
    /// Where the pointer was at the last move while a button holds the drag.
    drag_from: Option<Vec2>,
}

impl Relief {
    pub fn new(world: World) -> Relief {
        // The page starts far and high enough to hold the whole square, its near edge included.
        let w = world.width();
        let orbit = Orbit::new(Vec3::default(), Vec3::new(0.0, 1.1 * w, w));
        Relief { world, orbit, drag_from: None }
    }

    fn overlay(&self) -> DrawList {
        let mut list = DrawList::default();
        let plate = Rect::new(16.0, 16.0, 290.0, 122.0);
        list.fill(plate, PLATE);
        list.frame(plate, 1.0, LINE);
        list.text(Vec2::new(28.0, 26.0), text::TITLE, 16.0, INK);
        list.text(Vec2::new(28.0, 50.0), text::seed(&self.world), 12.0, INK);
        list.text(Vec2::new(28.0, 66.0), text::grid(&self.world), 12.0, MUTED);
        list.text(Vec2::new(28.0, 82.0), text::heights(&self.world), 12.0, MUTED);
        list.text(Vec2::new(28.0, 98.0), text::biomes(&self.world), 12.0, MUTED);
        list.text(Vec2::new(28.0, 114.0), text::HINT, 12.0, MUTED);
        list
    }
}

impl App for Relief {
    fn start(&mut self, gfx: &mut dyn Gfx) {
        let n = self.world.samples as u32;
        gfx.upload_image(MAP, n, n, &paint::paint(&self.world));
        let land = mesh::land(&self.world);
        gfx.upload_mesh(LAND, &land.vertices, &land.indices);
        let water = mesh::water(&self.world);
        gfx.upload_mesh(WATER, &water.vertices, &water.indices);
    }

    fn event(&mut self, event: Event) -> bool {
        match event {
            Event::Resized { .. } => true,
            Event::PointerDown { pos, button: Button::Left } => {
                self.drag_from = Some(pos);
                false
            }
            Event::PointerUp { button: Button::Left, .. } => {
                self.drag_from = None;
                false
            }
            Event::PointerMoved { pos } => match self.drag_from {
                Some(from) => {
                    self.orbit.drag(pos.x - from.x, pos.y - from.y);
                    self.drag_from = Some(pos);
                    true
                }
                None => false,
            },
            Event::Wheel { delta, .. } => {
                self.orbit.zoom(delta);
                true
            }
            _ => false,
        }
    }

    fn frame(&mut self, gfx: &mut dyn Gfx, seconds: f32) -> bool {
        let (wp, hp) = gfx.size_px();
        let s = gfx.scale();
        let (w, h) = (wp as f32 / s, hp as f32 / s);
        let moving = self.orbit.step(seconds);
        gfx.begin(GROUND);
        if w > 0.0 && h > 0.0 {
            let view_proj = self.orbit.view_proj(w / h);
            let all = Rect::new(0.0, 0.0, w, h);
            gfx.draw_mesh(all, LAND, &view_proj, Material::Image { image: MAP, nearest: true });
            // After the land: the sheet is tested against it but leaves no depth of its own.
            gfx.draw_mesh(all, WATER, &view_proj, Material::Flat { color: WATER_COLOUR });
        }
        gfx.draw_list(&self.overlay());
        gfx.end();
        moving
    }
}
