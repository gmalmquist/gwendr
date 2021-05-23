use crate::linear::*;

use std::ops;
use wasm_bindgen::__rt::core::ops::Neg;
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

    fn material(&self, point: &Vec3) -> Option<Material> {
        None
    }

    fn from<F>(func: F, epsilon: f64) -> FuncSdf<F>
        where F: Fn(&Vec3) -> f64 {
        FuncSdf::new(func, epsilon)
    }

    fn boxed_from(func: Box<dyn Fn(&Vec3) -> f64>, epsilon: f64) -> DynFuncSdf {
        DynFuncSdf { func, epsilon }
    }

    fn negate<T: SDF>(sdf: T) -> NegationSDF<T> {
        NegationSDF { sdf }
    }

    fn union<A: SDF, B: SDF>(a: A, b: B) -> UnionSDF<A, B> {
        UnionSDF { a, b }
    }

    fn intersection<A: SDF, B: SDF>(a: A, b: B) -> IntersectionSDF<A, B> {
        IntersectionSDF { a, b }
    }

    fn difference<A: SDF, B: SDF>(a: A, b: B) -> IntersectionSDF<A, NegationSDF<B>> {
        Self::intersection(a, Self::negate(b))
    }

    fn translate<S: SDF>(sdf: S, translation: Vec3) -> TranslatedSDF<S> {
        TranslatedSDF { sdf, translation }
    }

    fn scale<S: SDF>(sdf: S, scale: f64) -> ScaledSDF<S> {
        ScaledSDF { sdf, scale }
    }

    fn rotate<S: SDF>(sdf: S, angle: f64, axis: Vec3) -> RotatedSDF<S> {
        RotatedSDF { sdf, angle, axis }
    }
}

pub struct Sphere {
    center: Vec3,
    radius: f64,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64) -> Self {
        Self { center, radius }
    }
}

impl SDF for Sphere {
    fn distance(&self, point: &Vec3) -> f64 {
        self.center.dist(point) - self.radius
    }

    fn epsilon(&self) -> f64 {
        self.radius / 10_000.0
    }
}

pub struct MatSDF<S> {
    sdf: S,
    mat: Material,
}

pub struct DynFuncSdf {
    func: Box<dyn Fn(&Vec3) -> f64>,
    epsilon: f64,
}

pub struct FuncSdf<F> {
    func: F,
    epsilon: f64,
}

pub struct UnionSDF<A, B> {
    a: A,
    b: B,
}

pub struct IntersectionSDF<A, B> {
    a: A,
    b: B,
}

pub struct DifferenceSDF<A, B> {
    a: A,
    b: B,
}

pub struct NegationSDF<S> {
    sdf: S,
}

pub struct TranslatedSDF<S> {
    sdf: S,
    translation: Vec3,
}

pub struct ScaledSDF<S> {
    sdf: S,
    scale: f64,
}

pub struct RotatedSDF<S> {
    sdf: S,
    angle: f64,
    axis: Vec3,
}

impl<S> SDF for MatSDF<S> where S: SDF {
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

impl<F> FuncSdf<F> {
    pub fn new(func: F, epsilon: f64) -> Self
        where F: Fn(&Vec3) -> f64 {
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

impl<A, B> SDF for UnionSDF<A, B> where A: SDF, B: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.a.distance(point).min(self.b.distance(point))
    }

    fn epsilon(&self) -> f64 {
        self.a.epsilon().min(self.b.epsilon())
    }
}

impl<A, B> SDF for IntersectionSDF<A, B> where A: SDF, B: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.a.distance(point).max(self.b.distance(point))
    }

    fn epsilon(&self) -> f64 {
        self.a.epsilon().min(self.b.epsilon())
    }
}

impl<A> SDF for NegationSDF<A> where A: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        -self.sdf.distance(point)
    }

    fn epsilon(&self) -> f64 {
        self.sdf.epsilon()
    }
}

impl<S> SDF for TranslatedSDF<S> where S: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.sdf.distance(&(point - &self.translation))
    }

    fn epsilon(&self) -> f64 {
        self.sdf.epsilon()
    }
}

impl<S> SDF for ScaledSDF<S> where S: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.sdf.distance(&point.clone().scale(1.0 / self.scale)) * self.scale
    }

    fn epsilon(&self) -> f64 {
        self.sdf.epsilon()
    }
}

impl<S> SDF for RotatedSDF<S> where S: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.sdf.distance(&point.clone().rotate(-self.angle, &self.axis))
    }

    fn epsilon(&self) -> f64 {
        self.sdf.epsilon()
    }
}

impl<F> SDF for FuncSdf<F> where F: Fn(&Vec3) -> f64 {
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
    fn func_sdf() {
        // distance from unit sphere at origin
        let f = FuncSdf::new(|v| v.norm() - 1.0, 0.001);
        assert_eq!(f.distance(&Vec3::right()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::up()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::down()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::new(2.0, 0.0, 0.0)).to_string(), 1.0.to_string());
        assert_eq!(f.distance(&Vec3::zero()).to_string(), (-1.0).to_string());
    }
}
