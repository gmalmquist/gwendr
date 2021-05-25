use crate::linear::*;
use crate::mat::Material;
use wasm_bindgen::__rt::core::f64::consts::PI;
use crate::log;

const MAX_FLOAT: f64 = (1u64 << 53u64) as f64;

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

    fn smooth_union(self, sdf: Box<dyn SDF>, s: Option<SmoothUnionType>) -> SmoothUnionSDF where Self: Sized + 'static {
        SmoothUnionSDF::new(Box::new(self), sdf, s)
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

    fn transformed(self, func: Box<dyn Fn(&Vec3, &Box<dyn SDF>) -> f64>) -> TransformedSDF
        where Self: Sized + 'static {
        TransformedSDF::new(Box::new(self), func)
    }

    fn raymarch(&self, ray: &Ray, far_plane: f64) -> Option<RayHit> {
        let mut point = ray.origin.clone();
        let direction = ray.direction.clone().normalize();
        let mut distance = self.distance(&point);
        let epsilon = self.epsilon();
        while distance > epsilon {
            point = point.add(distance, &direction);
            distance = self.distance(&point);
            if point.dist2(&ray.origin) >= far_plane * far_plane {
                return None;
            }
        }
        let normal = self.normal(&point);
        let material = self.material(&point);
        let material = material.unwrap_or_else(|| Material::new());
        Some(RayHit {
            ray: ray.clone(),
            point,
            distance,
            normal,
            material,
        })
    }
}

#[derive(Debug)]
pub struct RayHit {
    pub ray: Ray,
    pub point: Vec3,
    pub distance: f64,
    pub normal: Vec3,
    pub material: Material,
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

#[derive(Clone)]
pub struct PolyFace {
    normal: Vec3,
    centroid: Vec3,
    vertices: Vec<Vec3>,
    epsilon: f64,
}

#[derive(Clone)]
pub struct EmptySDF {}

impl UnionSDF {
    pub fn new(a: Box<dyn SDF>, b: Box<dyn SDF>) -> Self {
        Self { a, b }
    }
}

impl SmoothUnionSDF {
    pub fn new(a: Box<dyn SDF>, b: Box<dyn SDF>, k: Option<SmoothUnionType>) -> Self {
        Self { a, b, smooth: k.unwrap_or(SmoothUnionType::Exp(32.)) }
    }
}

impl IntersectionSDF {
    pub fn new(a: Box<dyn SDF>, b: Box<dyn SDF>) -> Self {
        Self { a, b }
    }
}

impl DifferenceSDF {
    pub fn new(a: Box<dyn SDF>, b: Box<dyn SDF>) -> Self {
        Self { a, b }
    }
}

impl NegationSDF {
    pub fn new(sdf: Box<dyn SDF>) -> Self {
        Self { sdf }
    }
}

impl<'a> NegatedRefSDF<'a> {
    pub fn new(sdf: &'a Box<dyn SDF>) -> Self {
        Self { sdf }
    }
}

impl TransformedSDF {
    pub fn new(sdf: Box<dyn SDF>, func: Box<dyn Fn(&Vec3, &Box<dyn SDF>) -> f64>) -> Self {
        Self { sdf, func }
    }
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

impl PolyFace {
    pub fn new(vertices: Vec<Vec3>) -> Self {
        let (centroid, normal) = if vertices.len() >= 3 {
            let centroid = vertices.iter()
                .fold(Vec3::zero(), |a, b| &a + b)
                .scale(1. / (vertices.len() as f64));
            let a = &vertices[0] - &centroid;
            let b = &vertices[1] - &centroid;
            (centroid, (&b ^ &a).normalize())
        } else {
            (Vec3::zero(), Vec3::zero())
        };
        let epsilon = {
            let mut eps: f64 = 1.;
            for i in 0..vertices.len() {
                let d = vertices[i].dist(&vertices[(i + 1) % vertices.len()]);
                eps = eps.min(d / 1000.);
            }
            eps
        };
        Self {
            normal,
            centroid,
            vertices,
            epsilon,
        }
    }
}

impl SDF for Sphere {
    fn distance(&self, point: &Vec3) -> f64 {
        point.norm() - self.radius
    }

    fn epsilon(&self) -> f64 {
        self.radius / 10_000.0
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

impl SDF for EmptySDF {
    fn distance(&self, _: &Vec3) -> f64 {
        MAX_FLOAT
    }

    fn epsilon(&self) -> f64 {
        1.
    }
}

impl SDF for PolyFace {
    fn distance(&self, point: &Vec3) -> f64 {
        if self.vertices.len() < 3 {
            return MAX_FLOAT;
        }

        let thickness = 0.1;

        let mut sd = self.normal.dot(&(point - &self.centroid));
        sd = sd.max(-self.normal.dot(&(point - &self.centroid)) - thickness);
        for i in 0..self.vertices.len() {
            let a = &self.vertices[i];
            let b = &self.vertices[(i + 1) % self.vertices.len()];
            let edge_normal = (b - a).rotate(PI / 2.0, &self.normal).normalize();
            sd = sd.max(&(point - a) * &edge_normal - self.epsilon)
        }
        sd
    }

    fn normal(&self, _: &Vec3) -> Vec3 {
        self.normal.clone()
    }


    fn epsilon(&self) -> f64 {
        self.epsilon
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

pub struct SmoothUnionSDF {
    a: Box<dyn SDF>,
    b: Box<dyn SDF>,
    smooth: SmoothUnionType,
}

pub enum SmoothUnionType {
    /** exponential smoothing, default parameter = 32 */
    Exp(f64),
    /** polynomial smoothing, default parameter = 0.1 */
    Poly(f64),
    /** power smoothing, default parameter = 8 */
    Pow(f64),
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

pub struct NegatedRefSDF<'a> {
    sdf: &'a Box<dyn SDF>,
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

pub struct TransformedSDF {
    sdf: Box<dyn SDF>,
    func: Box<dyn Fn(&Vec3, &Box<dyn SDF>) -> f64>,
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

impl SDF for SmoothUnionSDF {
    fn distance(&self, point: &Vec3) -> f64 {
        self.smooth.smooth(self.a.distance(point), self.b.distance(point))
    }

    fn epsilon(&self) -> f64 {
        self.a.epsilon().min(self.b.epsilon()) / 10.
    }
}

impl SmoothUnionType {
    pub fn smooth(&self, a: f64, b: f64) -> f64 {
        // https://iquilezles.org/www/articles/smin/smin.htm
        match self {
            SmoothUnionType::Exp(k) => {
                let res = (-k * a).exp2() + (-k * b).exp2();
                -(res.log2() / k)
            }
            SmoothUnionType::Poly(k) => {
                let k = 0.1;
                let h = (k - (a - b).abs()).max(0.0) / k;
                a.min(b) - h * h * k * (1.0 / 4.0)
            }
            SmoothUnionType::Pow(k) => {
                let a = a.powf(*k);
                let b = b.powf(*k);
                ((a * b) / (a + b)).powf(1.0 / k)
            }
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

    fn normal(&self, p: &Vec3) -> Vec3 {
        self.sdf.normal(p).scale(-1.)
    }
}

impl<'a> SDF for NegatedRefSDF<'a> {
    fn distance(&self, point: &Vec3) -> f64 {
        -self.sdf.distance(point)
    }

    fn normal(&self, point: &Vec3) -> Vec3 {
        self.sdf.normal(point).scale(-1.)
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

impl SDF for TransformedSDF {
    fn distance(&self, point: &Vec3) -> f64 {
        (self.func)(point, &self.sdf)
    }

    fn epsilon(&self) -> f64 {
        self.sdf.epsilon()
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
