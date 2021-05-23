use std::ops;

use wasm_bindgen::__rt::core::ops::Neg;

use crate::linear::*;
use crate::mat::Material;

pub trait SDF {
    fn distance(&self, point: &Vec3) -> f64;

    fn normal(&self, point: &Vec3) -> Vec3 {
        let epsilon = self.epsilon();
        Vec3::new(
            self.distance(&Vec3::right().scale(epsilon).add(1.0, point))
                - self.distance(&Vec3::left().scale(epsilon).add(1.0, point)),
            self.distance(&Vec3::up().scale(epsilon).add(1.0, point))
                - self.distance(&Vec3::down().scale(epsilon).add(1.0, point)),
            self.distance(&Vec3::forward().scale(epsilon).add(1.0, point))
                - self.distance(&Vec3::backward().scale(epsilon).add(1.0, point)),
        ).normalize()
    }

    fn epsilon(&self) -> f64;

    fn material(&self, _point: &Vec3) -> Option<Material> {
        None
    }

    fn negate(self) -> NegationSDF where Self: Sized + 'static {
        NegationSDF { sdf: Box::new(self) }
    }

    fn union(self, sdf: Box<dyn SDF>) -> UnionSDF where Self: Sized + 'static {
        UnionSDF { a: Box::new(self), b: sdf }
    }

    fn intersection(self, sdf: Box<dyn SDF>) -> IntersectionSDF where Self: Sized + 'static {
        IntersectionSDF { a: Box::new(self), b: sdf }
    }

    fn difference(self, sdf: Box<dyn SDF>) -> DifferenceSDF where Self: Sized + 'static {
        DifferenceSDF { a: Box::new(self), b: sdf }
    }

    fn translate(self, translation: Vec3) -> TranslatedSDF where Self: Sized + 'static {
        TranslatedSDF { sdf: Box::new(self), translation }
    }

    fn scale(self, scale: f64) -> ScaledSDF where Self: Sized + 'static {
        ScaledSDF { sdf: Box::new(self), scale }
    }

    fn rotate(self, angle: f64, axis: Vec3) -> RotatedSDF where Self: Sized + 'static {
        RotatedSDF { sdf: Box::new(self), angle, axis }
    }

    fn shaded(self, mat: Material) -> MatSDF where Self: Sized + 'static {
        MatSDF {
            sdf: Box::new(self),
            mat,
        }
    }
}

pub struct SdfShape {
    sdf: fn(&Vec3) -> f64,
    epsilon: f64,
    mat: Option<Material>,
}

impl SDF for SdfShape {
    fn distance(&self, point: &Vec3) -> f64 {
        (self.sdf)(point)
    }

    fn epsilon(&self) -> f64 {
        self.epsilon
    }

    fn material(&self, point: &Vec3) -> Option<Material> {
        self.mat.clone()
    }
}

#[derive(Clone)]
pub struct Sphere {
    radius: f64,
}

#[derive(Clone)]
pub struct Plane {
    normal: Vec3,
}

#[derive(Clone)]
pub struct Disk {
    normal: Vec3,
    radius: f64,
}

impl Sphere {
    pub fn new(radius: f64) -> Self {
        Self { radius }
    }
}

impl Plane {
    pub fn new(normal: Vec3) -> Self { Self { normal } }
}

impl Disk {
    pub fn new(normal: Vec3, radius: f64) -> Self {
        Self {
            normal,
            radius,
        }
    }
}

impl SDF for Sphere {
    fn distance(&self, point: &Vec3) -> f64 {
        point.norm() - self.radius
    }

    fn epsilon(&self) -> f64 {
        self.radius / 1_000.0
    }
}

impl SDF for Plane {
    fn distance(&self, point: &Vec3) -> f64 {
        self.normal.dot(point)
    }

    fn epsilon(&self) -> f64 {
        0.001
    }
}

impl SDF for Disk {
    fn distance(&self, point: &Vec3) -> f64 {
        // this is really just the intersection of a sphere and an infinite plane
        self.normal.dot(point).max(point.norm() - self.radius)
    }

    fn epsilon(&self) -> f64 {
        self.radius / 1_000.0
    }
}

pub struct MatSDF {
    sdf: Box<dyn SDF>,
    mat: Material,
}

pub struct DynFuncSdf {
    func: Box<dyn Fn(&Vec3) -> f64>,
    epsilon: f64,
}

pub struct FuncSdf {
    func: fn(&Vec3) -> f64,
    epsilon: f64,
}

pub struct UnionSDF {
    a: Box<dyn SDF>,
    b: Box<dyn SDF>,
}

pub struct IntersectionSDF {
    a: Box<dyn SDF>,
    b: Box<dyn SDF>,
}


pub struct DifferenceSDF {
    a: Box<dyn SDF>,
    b: Box<dyn SDF>,
}

pub struct NegationSDF {
    sdf: Box<dyn SDF>,
}

pub struct TranslatedSDF {
    sdf: Box<dyn SDF>,
    translation: Vec3,
}

pub struct ScaledSDF {
    sdf: Box<dyn SDF>,
    scale: f64,
}

pub struct RotatedSDF {
    sdf: Box<dyn SDF>,
    angle: f64,
    axis: Vec3,
}

impl SDF for MatSDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.sdf.distance(point)
    }

    fn epsilon(&self) -> f64 {
        self.sdf.epsilon()
    }

    fn material(&self, _: &Vec3) -> Option<Material> {
        Some(self.mat.clone())
    }
}

impl FuncSdf {
    pub fn new(func: fn(&Vec3) -> f64, epsilon: f64) -> Self {
        FuncSdf {
            func,
            epsilon,
        }
    }
}

impl SDF for DynFuncSdf {
    fn distance(&self, point: &Vec3) -> f64 {
        (self.func)(point)
    }

    fn epsilon(&self) -> f64 {
        self.epsilon
    }
}

impl SDF for UnionSDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.a.distance(point).min(self.b.distance(point))
    }

    fn epsilon(&self) -> f64 {
        self.a.epsilon().min(self.b.epsilon())
    }

    fn material(&self, p: &Vec3) -> Option<Material> {
        if self.a.distance(p) < self.b.distance(p) {
            self.a.material(p)
        } else {
            self.b.material(p)
        }
    }
}

impl SDF for IntersectionSDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.a.distance(point).max(self.b.distance(point))
    }

    fn epsilon(&self) -> f64 {
        self.a.epsilon().min(self.b.epsilon())
    }

    fn material(&self, p: &Vec3) -> Option<Material> {
        if self.a.distance(p) < self.b.distance(p) {
            self.a.material(p)
        } else {
            self.b.material(p)
        }
    }
}

impl SDF for DifferenceSDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.a.distance(point).max(-self.b.distance(point))
    }

    fn epsilon(&self) -> f64 {
        self.a.epsilon().min(self.b.epsilon())
    }

    fn material(&self, p: &Vec3) -> Option<Material> {
        self.a.material(p)
    }
}

impl SDF for NegationSDF {
    fn distance(&self, point: &Vec3) -> f64 {
        -self.sdf.distance(point)
    }

    fn epsilon(&self) -> f64 {
        self.sdf.epsilon()
    }
}

impl SDF for TranslatedSDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.sdf.distance(&(point - &self.translation))
    }

    fn epsilon(&self) -> f64 {
        self.sdf.epsilon()
    }

    fn material(&self, p: &Vec3) -> Option<Material> {
        self.sdf.material(p)
    }
}

impl SDF for ScaledSDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.sdf.distance(&point.clone().scale(1.0 / self.scale)) * self.scale
    }

    fn epsilon(&self) -> f64 {
        self.sdf.epsilon()
    }

    fn material(&self, p: &Vec3) -> Option<Material> {
        self.sdf.material(p)
    }
}

impl SDF for RotatedSDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.sdf.distance(&point.clone().rotate(-self.angle, &self.axis))
    }

    fn epsilon(&self) -> f64 {
        self.sdf.epsilon()
    }

    fn material(&self, p: &Vec3) -> Option<Material> {
        self.sdf.material(p)
    }
}

impl SDF for FuncSdf {
    fn distance(&self, point: &Vec3) -> f64 {
        (self.func)(point)
    }

    fn epsilon(&self) -> f64 {
        self.epsilon
    }
}

impl SDF for Vec3 {
    fn distance(&self, point: &Vec3) -> f64 {
        self.dist(point)
    }

    fn epsilon(&self) -> f64 {
        0.0001
    }
}

#[cfg(test)]
mod tests {
    use crate::linear::*;
    use crate::sdf::*;

    #[test]
    fn sphere_sdf() {
        // distance from unit sphere at origin
        let f = Sphere::new(1.0);
        assert_eq!(f.distance(&Vec3::right()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::up()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::down()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::new(2.0, 0.0, 0.0)).to_string(), 1.0.to_string());
        assert_eq!(f.distance(&Vec3::zero()).to_string(), (-1.0).to_string());
    }
}
