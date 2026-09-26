//! 2D vector math. Used for free-body diagrams and rendering, not for the
//! integration itself (every scenario here is one degree of freedom).

use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Point or vector in world space. Meters for positions, newtons for forces.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    /// +x, screen-right.
    pub const X: Vec2 = Vec2 { x: 1.0, y: 0.0 };
    /// +y, world-up. The frontend flips this when it draws.
    pub const Y: Vec2 = Vec2 { x: 0.0, y: 1.0 };

    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Unit vector at `radians` counterclockwise from +x.
    pub fn from_angle(radians: f64) -> Self {
        Self::new(radians.cos(), radians.sin())
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// 2D cross product, i.e. the z component of the 3D one.
    pub fn cross(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    pub fn len_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn len(self) -> f64 {
        self.len_squared().sqrt()
    }

    /// Unit vector in the same direction. Zero-length stays zero rather than
    /// producing NaN, so a vanishing force vector is still drawable.
    pub fn normalized(self) -> Self {
        let len = self.len();
        if len == 0.0 {
            Self::ZERO
        } else {
            self / len
        }
    }

    /// Rotate counterclockwise by `radians`.
    pub fn rotated(self, radians: f64) -> Self {
        let (s, c) = radians.sin_cos();
        Self::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }

    /// Rotate 90 degrees counterclockwise. Exact, unlike `rotated(PI / 2.0)`.
    pub fn perpendicular(self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// Length of the projection of `self` onto `axis`. `axis` must be unit length.
    pub fn component_along(self, axis: Self) -> f64 {
        self.dot(axis)
    }

    /// Angle counterclockwise from +x, in radians.
    pub fn angle(self) -> f64 {
        self.y.atan2(self.x)
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f64) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2> for f64 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        rhs * self
    }
}

impl Div<f64> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f64) -> Vec2 {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

/// Standard gravity at Earth's surface, m/s^2.
pub const G: f64 = 9.80665;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_ops() {
        let a = Vec2::new(3.0, 4.0);
        assert_eq!(a.len(), 5.0);
        assert_eq!(a.dot(Vec2::new(1.0, 2.0)), 11.0);
        assert_eq!(a + Vec2::new(1.0, 1.0), Vec2::new(4.0, 5.0));
        assert_eq!(a * 2.0, Vec2::new(6.0, 8.0));
        assert_eq!(-a, Vec2::new(-3.0, -4.0));
    }

    #[test]
    fn normalizing_zero_is_zero() {
        assert_eq!(Vec2::ZERO.normalized(), Vec2::ZERO);
        assert!((Vec2::new(0.0, 7.0).normalized() - Vec2::Y).len() < 1e-12);
    }

    #[test]
    fn rotation_preserves_length() {
        let v = Vec2::new(2.0, -1.0);
        let r = v.rotated(0.7);
        assert!((r.len() - v.len()).abs() < 1e-12);
        assert!((v.perpendicular() - v.rotated(std::f64::consts::FRAC_PI_2)).len() < 1e-12);
    }

    #[test]
    fn projection_onto_unit_axis() {
        let axis = Vec2::from_angle(std::f64::consts::FRAC_PI_4);
        let v = Vec2::new(1.0, 1.0);
        assert!((v.component_along(axis) - 2.0f64.sqrt()).abs() < 1e-12);
    }
}
