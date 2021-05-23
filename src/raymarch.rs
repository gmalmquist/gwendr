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

pub fn raymarch<S: SDF>(ray: &Ray, sdf: &S, far_plane: f64) -> Option<RayHit> {
    let mut point = ray.origin.clone();
    let direction = ray.direction.clone().normalize();
    let mut distance = sdf.distance(&point);
    let epsilon = sdf.epsilon();
    while distance > epsilon {
        point = point.add(distance, &direction);
        distance = sdf.distance(&point);
        if point.dist2(&ray.origin) >= far_plane*far_plane {
            return None
        }
    }
    let normal = sdf.normal(&point);
    let material = sdf.material(&point);
    let material = material.unwrap_or_else(|| Material::new());
    Some(RayHit {
        ray: ray.clone(),
        point,
        distance,
        normal,
        material,
    })
}
