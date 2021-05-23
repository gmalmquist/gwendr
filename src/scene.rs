use wasm_bindgen::prelude::*;
use wasm_bindgen::__rt::core::f64::consts::PI;

use crate::linear::*;
use crate::mat::Color;
use crate::sdf::{RayHit, SDF};

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub atten: f64,
}

impl Light {
    pub fn new(position: Vec3, color: Color, atten: f64) -> Self {
        Self { position, color, atten }
    }

    pub fn shadow_ray(&self, point: &Vec3) -> Ray {
        Ray::new(point.clone(), &self.position - point)
    }

    pub fn color(&self, point: &Vec3) -> Color {
        let dist2 = point.dist2(&self.position);
        let atten = ((self.atten * self.atten) / dist2).min(1.0);
        self.color.clone().scale(atten)
    }
}

pub struct Scene {
    pub sdf: Box<dyn SDF>,
    pub lights: Vec<Light>,
    pub view: ViewTransform,
    pub far_plane: f64,
}

pub struct OrthoView {
    pub frame: Frame,
}

pub struct PerspView {
    pub eye_frame: Frame,
    pub near: f64,
    pub fov_degrees: f64,
}

pub enum ViewTransform {
    Ortho(OrthoView),
    Persp(PerspView),
}

impl ViewTransform {
    pub fn project(&self, local: &Vec3) -> Ray {
        // local point should be in (-1, -1, 0) to (+1, +1, 0)
        match self {
            ViewTransform::Ortho(ortho) => {
                Ray::new(
                    ortho.frame.project_point(local),
                    ortho.frame.project_vec(&Vec3::forward()),
                )
            }
            ViewTransform::Persp(persp) => {
                let fov = persp.fov_degrees * PI / 180.;
                let near_plane_width = 2. * (fov / 2.).tan() * persp.near;
                let near_plane_height = 2. * (fov / 2.).tan() * persp.near;
                let near_plane = Frame::new(
                    persp.eye_frame.project_point(&Vec3::new(0., 0., persp.near)),
                    persp.eye_frame.project_vec(&Vec3::new(near_plane_width / 2., 0., 0.)),
                    persp.eye_frame.project_vec(&Vec3::new(0., near_plane_height / 2., 0.)),
                    persp.eye_frame.project_vec(&Vec3::forward()),
                );

                let point_on_near_plane = near_plane.project_point(local);

                Ray::new(
                    point_on_near_plane.clone(),
                    (&point_on_near_plane - &persp.eye_frame.origin).normalize(),
                )
            }
        }
    }
}

impl Scene {
    pub fn raycast_pixel(&self, pixel: (usize, usize), width: usize, height: usize) -> Option<Color> {
        let x = pixel.0 as f64;
        let y = pixel.1 as f64;
        let width = width as f64;
        let height = height as f64;
        let x = (x - width / 2.) / (width / 2.);
        let y = (height / 2. - y) / (height / 2.);
        let local = Vec3::new(x, y, 0.);
        let ray = self.view.project(&local);
        self.raycast(ray)
    }

    pub fn raycast(&self, ray: Ray) -> Option<Color> {
        let ray_count = 1;
        let mut color = None;
        for _ in 0..ray_count {
            let hit = self.sdf.raymarch(&perturb(&ray, 0.01), self.far_plane);
            if let Some(col) = hit.map(|hit| self.get_color(&hit)) {
                color = color.map(|c| &c + &col).or(Some(col))
            }
        }
        color.map(|c| c.scale(1. / (ray_count as f64)))
    }

    fn get_color(&self, hit: &RayHit) -> Color {
        let mut color = hit.material.ambient.clone();

        // ray pointing toward eye
        let v = hit.ray.direction.clone().normalize().scale(-1.);

        // hit point pushed out a little bit to avoid self-collisions
        let adjusted_hit = hit.point.clone().add(self.sdf.epsilon(), &hit.normal);

        for light in &self.lights {
            let lc = light.color(&hit.point);
            let mut shadow_ray = light.shadow_ray(&adjusted_hit);
            let ld = shadow_ray.direction.clone().normalize();

            let shadow_ray_count = 1;
            let mut shadow_hit_count = 0;
            for _ in 0..shadow_ray_count {
                let hit = self.sdf.raymarch(
                    &perturb(&shadow_ray, 0.),
                    shadow_ray.direction.norm(),
                );
                if hit.is_some() {
                    shadow_hit_count += 1;
                }
            }
            if shadow_hit_count == shadow_ray_count {
                continue; // fully in shadow.
            }
            let shadow_amount = (shadow_hit_count as f64) / (shadow_ray_count as f64);

            // reflection of direction to light
            let lr = ld.clone().add(-2., &ld.clone().off_axis(&hit.normal));

            let diffuse_strength = (&ld * &hit.normal).max(0.);
            let specular_strength = (&lr * &v).max(0.).powf(hit.material.phong);
            color = color
                .add(diffuse_strength * (1. - shadow_amount), &(&hit.material.diffuse * &lc))
                .add(specular_strength * (1. - shadow_amount), &hit.material.specular)
        }

        if hit.material.reflectivity > 0. {
            let refl_ray = Ray::new(
                adjusted_hit,
                v.clone().add(-2., &v.clone().off_axis(&hit.normal)),
            );
            if let Some(refl_color) = self.raycast(refl_ray) {
                color = color.add(hit.material.reflectivity, &refl_color);
            }
        }

        color
    }
}

fn perturb(ray: &Ray, degrees: f64) -> Ray {
    let random_spread = degrees * PI / 180.0;
    let mut ray = ray.clone();

    let r = ray.direction.clone().normalize();
    let axis1 = Vec3::cross(&r, &Vec3::right());
    let axis1 = if axis1.norm2() == 0. {
        Vec3::cross(&r, &Vec3::up())
    } else {
        axis1
    };
    let axis1 = axis1.normalize();
    let axis2 = Vec3::cross(&axis1, &r).normalize();

    ray.direction = r
        .rotate((random() * 2. - 1.) * random_spread, &axis1)
        .rotate((random() * 2. - 1.) * random_spread, &axis2);
    ray
}