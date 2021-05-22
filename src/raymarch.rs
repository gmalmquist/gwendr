use crate::sdf::SDF;
use crate::linear::*;

pub struct RayHit {
    ray: Ray,
    point: Vec3,
    distance: f64,
    normal: Vec3,
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
    let normal = sdf.normal(&point, epsilon);
    Some(RayHit {
        ray,
        point,
        distance,
        normal,
    })
}
