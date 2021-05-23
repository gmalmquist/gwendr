use crate::sdf::SDF;
use crate::linear::*;
use crate::mat::Material;

pub struct RayHit {
    pub ray: Ray,
    pub point: Vec3,
    pub distance: f64,
    pub normal: Vec3,
    pub material: Material,
}

pub fn raymarch<S: SDF>(ray: Ray, sdf: &S) -> Option<RayHit> {
    let mut point = ray.origin.clone();
    let direction = ray.direction.clone().normalize();
    let mut distance = sdf.distance(&point);
    let epsilon = sdf.epsilon();
    while distance > epsilon {
        point = point.add(distance, &direction);
        let last_distance = distance;
        distance = sdf.distance(&point);
        if distance > last_distance {
            return None
        }
    }
    let normal = sdf.normal(&point);
    let material = sdf.material(&point);
    let material = material.unwrap_or_else(|| Material::new());
    Some(RayHit {
        ray,
        point,
        distance,
        normal,
        material,
    })
}
