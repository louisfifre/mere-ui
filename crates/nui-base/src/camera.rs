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
// Radians a pixel of drag turns.
const TURN: f32 = 0.006;
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
        Orbit { target, goal: pose, now: pose, fov_y: 50f32.to_radians(), near: 10.0 }
    }

    pub fn drag(&mut self, dx: f32, dy: f32) {
        self.goal.yaw -= dx * TURN;
        self.goal.pitch = (self.goal.pitch + dy * TURN).clamp(PITCH_MIN, PITCH_MAX);
    }

    /// One wheel notch is `delta` 1; positive comes closer.
    pub fn zoom(&mut self, delta: f32) {
        self.goal.distance = (self.goal.distance * 0.9f32.powf(delta)).max(self.near * 4.0);
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
    fn glides_to_rest_and_then_stops_asking_for_frames() {
        let mut o = Orbit::new(Vec3::default(), Vec3::new(0.0, 500.0, 500.0));
        o.drag(100.0, 0.0);
        let mut frames = 0;
        while o.step(1.0 / 60.0) {
            frames += 1;
            assert!(frames < 600, "never came to rest");
        }
        assert!(frames > 5);
    }
}
