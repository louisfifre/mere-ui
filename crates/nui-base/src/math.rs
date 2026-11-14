//! The little linear algebra a 3D view needs. Matrices are column-major, the order HLSL and GLSL read a
//! constant buffer in by default, so they go to the GPU as they are.

use std::ops::{Add, Mul, Sub};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.x && p.y >= self.y && p.x < self.x + self.w && p.y < self.y + self.h
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }
    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(self.y * o.z - self.z * o.y, self.z * o.x - self.x * o.z, self.x * o.y - self.y * o.x)
    }
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn normalized(self) -> Vec3 {
        self * (1.0 / self.length())
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, k: f32) -> Vec3 {
        Vec3::new(self.x * k, self.y * k, self.z * k)
    }
}

/// A 4x4 matrix, `m[column * 4 + row]`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat4 {
    pub m: [f32; 16],
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 { m: [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.] };

    fn at(&self, row: usize, col: usize) -> f32 {
        self.m[col * 4 + row]
    }

    /// A perspective with reversed depth and no far plane: depth 1 at `near`, falling towards 0 at
    /// infinity. Floating-point numbers are densest near 0, and reversing puts that density far away, where
    /// a plain depth buffer runs out of precision; it is what keeps a sea one block under a shore from
    /// flickering from 10 blocks to 200 000, which the page needed a logarithmic depth buffer for.
    /// Right-handed: the camera looks down -z. `aspect` is width over height.
    pub fn perspective_reversed(fov_y: f32, aspect: f32, near: f32) -> Mat4 {
        let f = 1.0 / (fov_y / 2.0).tan();
        Mat4 { m: [f / aspect, 0., 0., 0., 0., f, 0., 0., 0., 0., 0., -1., 0., 0., near, 0.] }
    }

    /// The view from `eye` towards `target`, `up` giving which way is up on screen.
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        let f = (target - eye).normalized();
        let s = f.cross(up).normalized();
        let u = s.cross(f);
        Mat4 {
            m: [
                s.x, u.x, -f.x, 0., //
                s.y, u.y, -f.y, 0., //
                s.z, u.z, -f.z, 0., //
                -s.dot(eye), -u.dot(eye), f.dot(eye), 1.,
            ],
        }
    }

    /// The point `p` (w = 1) through this matrix, as clip coordinates x, y, z, w.
    pub fn transform(&self, p: Vec3) -> [f32; 4] {
        let mut out = [0.0; 4];
        for (row, o) in out.iter_mut().enumerate() {
            *o = self.at(row, 0) * p.x + self.at(row, 1) * p.y + self.at(row, 2) * p.z + self.at(row, 3);
        }
        out
    }
}

impl Mul for Mat4 {
    type Output = Mat4;
    fn mul(self, o: Mat4) -> Mat4 {
        let mut m = [0.0; 16];
        for col in 0..4 {
            for row in 0..4 {
                m[col * 4 + row] = (0..4).map(|k| self.at(row, k) * o.at(k, col)).sum();
            }
        }
        Mat4 { m }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn depth(m: &Mat4, z_view: f32) -> f32 {
        let c = m.transform(Vec3::new(0.0, 0.0, z_view));
        c[2] / c[3]
    }

    #[test]
    fn reversed_depth_is_one_at_near_and_falls_towards_zero() {
        let near = 10.0;
        let p = Mat4::perspective_reversed(50f32.to_radians(), 1.5, near);
        assert!((depth(&p, -near) - 1.0).abs() < 1e-6);
        let (a, b) = (depth(&p, -1000.0), depth(&p, -200_000.0));
        assert!(a > b && b > 0.0);
    }

    #[test]
    fn look_at_puts_the_target_straight_ahead() {
        let v = Mat4::look_at(Vec3::new(0.0, 100.0, 100.0), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let c = v.transform(Vec3::new(0.0, 0.0, 0.0));
        assert!(c[0].abs() < 1e-4 && c[1].abs() < 1e-4);
        assert!((c[2] + 100.0 * 2f32.sqrt()).abs() < 1e-3);
    }

    #[test]
    fn identity_changes_nothing() {
        let v = Mat4::look_at(Vec3::new(3.0, 2.0, 1.0), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(v * Mat4::IDENTITY, v);
        assert_eq!(Mat4::IDENTITY * v, v);
    }
}
