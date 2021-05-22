use crate::sdf::SDF;
use crate::linear::*;

pub struct RayHit {
    ray: Ray,
    point: Vec3,
    distance: f64,
    normal: Vec3,
}

pub fn raymarch<S: SDF>(ray: Ray, sdf: S) -> RayHit {
    let mut point = ray.origin.clone();
    let direction = ray.direction.clone().normalize();
    let mut distance = sdf.distance(&point);
    let mut last_distance = -1.;
    let epsilon = sdf.epsilon();
    while distance > epsilon && (last_distance < 0. || distance < last_distance) {
        point = point.add(distance, &direction);
        last_distance = distance;
        distance = sdf.distance(&point);
    }
    let normal = sdf.normal(&point, epsilon);
    RayHit {
        ray,
        point,
        distance,
        normal,
    }
}
