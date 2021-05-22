use std::f64;
use std::fmt;
use std::ops;
use wasm_bindgen::__rt::core::fmt::Formatter;
use wasm_bindgen::__rt::core::ops::{Add, BitXor};

#[derive(Clone)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0., 0., 0.)
    }

    pub fn up() -> Self {
        Self::new(0., 1., 0.)
    }

    pub fn down() -> Self {
        Self::new(0., -1., 0.)
    }

    pub fn left() -> Self {
        Self::new(-1., 0., 0.)
    }

    pub fn right() -> Self {
        Self::new(1., 0., 0.)
    }

    pub fn forward() -> Self {
        Self::new(0., 0., 1.)
    }

    pub fn backward() -> Self {
        Self::new(0., 0., -1.)
    }

    pub fn cross(a: &Self, b: &Self) -> Vec3 {
        // i     j     k
        // a.x   a.y   a.z
        // b.x   b.y   b.z
        Vec3::new(
            a.y * b.z - a.z * b.y,
            a.z * b.x - a.x * b.z,
            a.x * b.y - a.y * b.x,
        )
    }

    pub fn set(mut self, x: f64, y: f64, z: f64) -> Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn add(mut self, scale: f64, other: &Vec3) -> Self {
        self.x += scale * other.x;
        self.y += scale * other.y;
        self.z += scale * other.z;
        self
    }

    pub fn scale(mut self, scale: f64) -> Self {
        self.x *= scale;
        self.y *= scale;
        self.z *= scale;
        self
    }

    pub fn scale_vec(mut self, scale: &Vec3) -> Self {
        self.x *= scale.x;
        self.y *= scale.y;
        self.z *= scale.z;
        self
    }

    pub fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn norm2(&self) -> f64 {
        self.dot(&self)
    }

    pub fn norm(&self) -> f64 {
        self.norm2().sqrt()
    }

    pub fn normalize(mut self) -> Self {
        let mag2 = self.norm2();
        if mag2 == 0.0 {
            return self;
        }
        let mag = mag2.sqrt();
        self.x /= mag;
        self.y /= mag;
        self.z /= mag;
        self
    }
}

impl ops::Add<&Vec3> for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: &Vec3) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::Sub<&Vec3> for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl ops::Mul<&Vec3> for &Vec3 {
    type Output = f64;

    fn mul(self, rhs: &Vec3) -> Self::Output {
        self.dot(&rhs)
    }
}

impl ops::Mul<f64> for &Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        self.clone().scale(rhs)
    }
}

impl ops::BitXor<&Vec3> for &Vec3 {
    type Output = Vec3;

    fn bitxor(self, rhs: &Vec3) -> Self::Output {
        Vec3::cross(self, rhs)
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "<{:.2}, {:.2}, {:.2}>", self.x, self.y, self.z)
    }
}

impl From<(f64, f64, f64)> for Vec3 {
    fn from(tup: (f64, f64, f64)) -> Self {
        Self::new(tup.0, tup.1, tup.2)
    }
}

impl From<Vec3> for (f64, f64, f64) {
    fn from(v: Vec3) -> Self {
        (v.x, v.y, v.z)
    }
}

#[cfg(test)]
mod tests {
    use crate::linear::*;

    #[test]
    fn cross_product() {
        // right-hand rule ...
        assert_eq!(
            Vec3::forward().to_string(),
            (&Vec3::right() ^ &Vec3::up()).to_string(),
        )
    }
}
