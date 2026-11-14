//! A camera that turns around a point, as the page's OrbitControls did: drag to turn, wheel to come closer,
//! and a glide to rest after the hand lets go. The glide is why a frame can ask for the next one.

use crate::math::{Mat4, Vec3};
use std::f32::consts::FRAC_PI_2;

pub struct Orbit {
    pub target: Vec3,
    /// Where the camera is heading; the current values move towards these each frame.
    goal: Pose,
    now: Pose,
    pub fov_y: f32,
    pub near: f32,
    /// How far the wheel can take the camera out; without it the area dwindles to a speck.
    pub max_distance: f32,
    /// How close it can come: the applications set it to what their data still shows as more than a blur.
    pub min_distance: f32,
}

#[derive(Copy, Clone, PartialEq)]
struct Pose {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

// Never quite level, so the ground is never seen exactly edge-on, and never past the zenith.
const PITCH_MIN: f32 = 0.02;
const PITCH_MAX: f32 = FRAC_PI_2 - 0.02;
// OrbitControls (r150+, rotateSpeed 1): a drag of the window's whole height is one full turn.
const FULL_TURN: f32 = std::f32::consts::TAU;
// OrbitControls (zoomSpeed 1): each wheel notch scales the distance by this.
const ZOOM_NOTCH: f32 = 0.95;
// How fast the glide closes the gap, per second: the gap is divided by e every 1 / RATE seconds.
const RATE: f32 = 12.0;

impl Orbit {
    /// Looking at `target` from `eye`.
    pub fn new(target: Vec3, eye: Vec3) -> Orbit {
        let d = eye - target;
        let distance = d.length();
        let pose = Pose {
            yaw: d.x.atan2(d.z),
            pitch: (d.y / distance).asin().clamp(PITCH_MIN, PITCH_MAX),
            distance,
        };
        Orbit { target, goal: pose, now: pose, fov_y: 50f32.to_radians(), near: 10.0, max_distance: distance * 2.0, min_distance: 40.0 }
    }

    /// dx, dy in logical pixels; height is the window's logical height, which sets the angle of a pixel
    /// for both axes, as OrbitControls does with clientHeight.
    pub fn drag(&mut self, dx: f32, dy: f32, height: f32) {
        let per_pixel = FULL_TURN / height.max(1.0);
        self.goal.yaw -= dx * per_pixel;
        self.goal.pitch = (self.goal.pitch + dy * per_pixel).clamp(PITCH_MIN, PITCH_MAX);
    }

    /// One wheel notch is `delta` 1; positive comes closer.
    pub fn zoom(&mut self, delta: f32) {
        self.goal.distance = (self.goal.distance * ZOOM_NOTCH.powf(delta)).clamp(self.min_distance, self.max_distance);
    }

    /// Moves towards the goal; true while still moving.
    pub fn step(&mut self, seconds: f32) -> bool {
        let k = 1.0 - (-RATE * seconds.min(0.1)).exp();
        let mix = |a: f32, b: f32| a + (b - a) * k;
        self.now = Pose {
            yaw: mix(self.now.yaw, self.goal.yaw),
            pitch: mix(self.now.pitch, self.goal.pitch),
            distance: mix(self.now.distance, self.goal.distance),
        };
        let close = (self.now.yaw - self.goal.yaw).abs() < 1e-4
            && (self.now.pitch - self.goal.pitch).abs() < 1e-4
            && (self.now.distance - self.goal.distance).abs() < self.goal.distance * 1e-4;
        if close {
            self.now = self.goal;
        }
        !close
    }

    /// Lifts the camera, current and goal, so it stays above a surface: loor(x, z) is the lowest height
    /// the eye may have there, or 32::NEG_INFINITY where there is none. It first tilts the camera up
    /// round the target, and when even straight above is too low, backs it away.
    pub fn keep_above(&mut self, floor: &dyn Fn(f32, f32) -> f32) {
        let target = self.target;
        let lift = |p: &mut Pose| {
            let at = |p: &Pose| target + Vec3::new(p.pitch.cos() * p.yaw.sin(), p.pitch.sin(), p.pitch.cos() * p.yaw.cos()) * p.distance;
            let eye = at(p);
            let need = floor(eye.x, eye.z) - target.y;
            if need > eye.y - target.y {
                p.pitch = (need / p.distance).clamp(-1.0, 1.0).asin().clamp(p.pitch, PITCH_MAX);
                // Straight above is the highest it can tilt: past that, the distance has to grow.
                p.distance = p.distance.max(need / p.pitch.sin());
            }
        };
        lift(&mut self.goal);
        lift(&mut self.now);
    }

    pub fn eye(&self) -> Vec3 {
        let Pose { yaw, pitch, distance } = self.now;
        self.target + Vec3::new(pitch.cos() * yaw.sin(), pitch.sin(), pitch.cos() * yaw.cos()) * distance
    }

    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_reversed(self.fov_y, aspect, self.near)
            * Mat4::look_at(self.eye(), self.target, Vec3::new(0.0, 1.0, 0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_where_it_was_placed() {
        let eye = Vec3::new(0.0, 1100.0, 1000.0);
        let o = Orbit::new(Vec3::default(), eye);
        assert!((o.eye() - eye).length() < 1e-2);
    }

    #[test]
    fn a_drag_of_the_window_height_is_one_turn_and_a_notch_is_five_percent() {
        let mut o = Orbit::new(Vec3::default(), Vec3::new(0.0, 500.0, 500.0));
        let before = o.eye();
        o.drag(800.0, 0.0, 800.0);
        while o.step(1.0 / 60.0) {}
        assert!((o.eye() - before).length() < 1.0, "a full height of drag comes back round");
        let d = (o.eye() - o.target).length();
        o.min_distance = 1.0;
        o.zoom(2.0);
        while o.step(1.0 / 60.0) {}
        assert!(((o.eye() - o.target).length() / d - 0.95 * 0.95).abs() < 1e-3);
    }

    #[test]
    fn glides_to_rest_and_then_stops_asking_for_frames() {
        let mut o = Orbit::new(Vec3::default(), Vec3::new(0.0, 500.0, 500.0));
        o.drag(100.0, 0.0, 800.0);
        let mut frames = 0;
        while o.step(1.0 / 60.0) {
            frames += 1;
            assert!(frames < 600, "never came to rest");
        }
        assert!(frames > 5);
    }
}
