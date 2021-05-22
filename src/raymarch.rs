use crate::linear::*;

use std::ops;
use wasm_bindgen::__rt::core::ops::Neg;

pub trait SDF {
    fn distance(&self, point: &Vec3) -> f64;

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


pub struct FuncSdf<F> {
    func: F,
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

impl<F> FuncSdf<F> {
    pub fn new(f: F) -> Self
        where F: Fn(&Vec3) -> f64 {
        FuncSdf {
            func: f
        }
    }
}

impl<A, B> SDF for UnionSDF<A, B> where A: SDF, B: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.a.distance(point).min(self.b.distance(point))
    }
}

impl<A, B> SDF for IntersectionSDF<A, B> where A: SDF, B: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.a.distance(point).max(self.b.distance(point))
    }
}

impl<A> SDF for NegationSDF<A> where A: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        -self.sdf.distance(point)
    }
}

impl<S> SDF for TranslatedSDF<S> where S: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.sdf.distance(&(point - &self.translation))
    }
}

impl<S> SDF for ScaledSDF<S> where S: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.sdf.distance(&point.clone().scale(1.0 / self.scale)) * self.scale
    }
}

impl<S> SDF for RotatedSDF<S> where S: SDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.sdf.distance(&point.clone().rotate(-self.angle, &self.axis))
    }
}

impl<F> SDF for FuncSdf<F> where F: Fn(&Vec3) -> f64 {
    fn distance(&self, point: &Vec3) -> f64 {
        (self.func)(point)
    }
}

impl SDF for Vec3 {
    fn distance(&self, point: &Vec3) -> f64 {
        self.dist(point)
    }
}

#[cfg(test)]
mod tests {
    use crate::linear::*;
    use crate::raymarch::*;

    #[test]
    fn func_sdf() {
        // distance from unit sphere at origin
        let f = FuncSdf::new(|v| v.norm() - 1.0);
        assert_eq!(f.distance(&Vec3::right()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::up()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::down()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::new(2.0, 0.0, 0.0)).to_string(), 1.0.to_string());
        assert_eq!(f.distance(&Vec3::zero()).to_string(), (-1.0).to_string());
    }
}
