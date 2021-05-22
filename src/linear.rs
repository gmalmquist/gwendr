use std::f64;
use std::fmt;
use std::ops;

#[derive(Clone)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone)]
pub struct Basis {
    pub axes: (Vec3, Vec3, Vec3),
}

#[derive(Clone)]
pub struct Frame {
    pub origin: Vec3,
    pub basis: Basis,
}

#[derive(Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
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

    pub fn dist2(&self, other: &Vec3) -> f64 {
        let (x, y, z) = (other.x - self.x, other.y - self.y, other.z - self.z);
        x * x + y * y + z * z
    }

    pub fn dist(&self, other: &Vec3) -> f64 {
        self.dist2(other).sqrt()
    }
}

impl Basis {
    pub fn new(i: Vec3, j: Vec3, k: Vec3) -> Self {
        Self {
            axes: (i, j, k),
        }
    }

    pub fn identity() -> Self {
        Self::new(Vec3::right(), Vec3::up(), Vec3::forward())
    }

    pub fn into_frame(self, origin: Vec3) -> Frame {
        Frame {
            origin,
            basis: self,
        }
    }

    pub fn project(&self, local: &Vec3) -> Vec3 {
        Vec3::zero()
            .add(local.x, &self.axes.0)
            .add(local.y, &self.axes.1)
            .add(local.z, &self.axes.2)
    }

    pub fn unproject(&self, global: &Vec3) -> Vec3 {
        Vec3::new(
            global * &self.axes.0 / self.axes.0.norm2(),
            global * &self.axes.1 / self.axes.1.norm2(),
            global * &self.axes.2 / self.axes.2.norm2(),
        )
    }
}

impl Frame {
    pub fn new(origin: Vec3, i: Vec3, j: Vec3, k: Vec3) -> Self {
        Self {
            origin,
            basis: Basis::new(i, j, k),
        }
    }

    pub fn identity() -> Self {
        Self::new(Vec3::zero(), Vec3::right(), Vec3::up(), Vec3::forward())
    }

    pub fn translate(mut self, by: &Vec3) {
        self.origin = self.origin.add(1.0, by);
    }

    pub fn project_vec(&self, local: &Vec3) -> Vec3 {
        self.basis.project(local)
    }

    pub fn unproject_vec(&self, global: &Vec3) -> Vec3 {
        self.basis.unproject(global)
    }

    pub fn project_point(&self, local: &Vec3) -> Vec3 {
        self.basis.project(local).add(1.0, &self.origin)
    }

    pub fn unproject_point(&self, global: &Vec3) -> Vec3 {
        self.basis.unproject(&(global - &self.origin))
    }
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "<{:.2}, {:.2}, {:.2}>", self.x, self.y, self.z)
    }
}

impl From<(f64, f64, f64)> for Vec3 {
    fn from(tup: (f64, f64, f64)) -> Self {
        Self::new(tup.0, tup.1, tup.2)
    }
}

impl From<(usize, usize, usize)> for Vec3 {
    fn from(tup: (usize, usize, usize)) -> Self {
        Self::new(tup.0 as f64, tup.1 as f64, tup.2 as f64)
    }
}

impl From<Vec3> for (f64, f64, f64) {
    fn from(v: Vec3) -> Self {
        (v.x, v.y, v.z)
    }
}

impl From<Vec3> for (usize, usize, usize) {
    fn from(v: Vec3) -> Self {
        (v.x as usize, v.y as usize, v.z as usize)
    }
}

impl From<Frame> for Basis {
    fn from(frame: Frame) -> Self {
        frame.basis
    }
}

impl fmt::Display for Basis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "Basis(I={}, J={}, K={})", self.axes.0, self.axes.1, self.axes.2)
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Frame(O={}, I={}, J={}, K={})",
            self.origin,
            self.basis.axes.0,
            self.basis.axes.1,
            self.basis.axes.2
        )
    }
}

impl fmt::Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Ray(origin={}, direction={})",
            self.origin,
            self.direction,
        )
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

    #[test]
    fn basis() {}
}
