use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::mat;
use crate::raymarch;
use crate::sdf;
use crate::linear::{Frame, Vec3, Basis, Ray};
use crate::sdf::{DynFuncSdf, SDF, Sphere, UnionSDF};
use crate::raymarch::RayHit;
use crate::scene::Light;
use crate::mat::{Color, Material};
use wasm_bindgen::__rt::core::f64::consts::PI;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

#[wasm_bindgen]
pub struct Viewport {
    canvas: web_sys::HtmlCanvasElement,
    context: web_sys::CanvasRenderingContext2d,
    index: usize,
    seed: u64,
    frame: u64,
}

pub trait ViewportApi {
    fn handle_key_down(&mut self, key: &str);
}

#[wasm_bindgen]
impl Viewport {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        Self {
            canvas,
            context,
            index: 0,
            seed: 0,
            frame: 0,
        }
    }

    pub fn update(&mut self) {
        //self.context.clear_rect(0., 0., self.canvas.width().into(), self.canvas.height().into());
        let width = self.canvas.width() as usize;
        let height = self.canvas.height() as usize;

        for _ in 0..(width * height / 64) {
            self.render_next_point();
        }
        self.seed += 29;
    }

    fn render_next_point(&mut self) {
        let width = self.canvas.width() as usize;
        let height = self.canvas.height() as usize;

        let eye_distance = 1.;

        let canvas_frame = Frame::new(
            Vec3::new(width as f64 / 2.0, height as f64 / 2.0, 0.),
            Vec3::right().scale(width as f64 / 2.0),
            Vec3::up().scale(-(height as f64 / 2.0)),
            Vec3::forward(),
        );

        let x = (self.index % width) as f64;
        let y = (self.index / width) as f64;

        let world_frame = Frame::identity();//.scale(6.);

        let canvas_point = Vec3::new(x, y, 0.);
        let local_point = canvas_frame.unproject_point(&canvas_point);
        let world_point = world_frame.project_point(&local_point);

        // TODO: pretty sure this perspective math is slightly wrong
        let eye = Vec3::zero().add(eye_distance, &Vec3::backward());
        let eye_dir = (&world_point - &eye).normalize()
            .rotate(0. * PI / 180., &Vec3::up());
        let ray = Ray::new(eye, eye_dir);

        if let Some(color) = self.raycast(ray) {
            let color = &color;
            self.context.set_fill_style(&color.into());
            self.context.fill_rect(x, y, 1., 1.);

            if self.frame == 0 {
                // log(&format!("eye: {}", &hit.ray));
                // log(&format!("hit: {:#?}", &hit.distance));
            }
        } else {
            self.context.fill_rect(x, y, 0., 0.);
        }

        self.index = (self.index + 1) % (width * height);
        if self.index == 0 {
            self.frame += 1;
        }
    }

    fn raycast(&self, ray: Ray) -> Option<mat::Color> {
        let far_plane = 1_000.;
        let scene = self.get_scene();
        let ray_count = 1;
        let mut color = None;
        for _ in 0..ray_count {
            let hit = raymarch::raymarch(&perturb(&ray, 0.01), &scene, far_plane);
            if let Some(col) = hit.map(|hit| self.get_color(&hit, &scene, far_plane)) {
                color = color.map(|c| &c + &col).or(Some(col))
            }
        }
        color.map(|c| c.scale(1. / (ray_count as f64)))
    }

    fn get_color<F>(&self, hit: &RayHit, scene: &F, far_plane: f64) -> mat::Color where F: sdf::SDF {
        let lights = vec![
            Light::new(
                Vec3::new(-10.0, 10.0, 5.0),
                Color::from_hexstring("#ffffff"),
                10.,
            ),
            Light::new(
                Vec3::new(10.0, 0.0, 0.0),
                Color::from_hexstring("#ff88ff").scale(0.1),
                10.,
            ),
            Light::new(
                Vec3::new(-10.0, 0.0, 3.),
                Color::from_hexstring("#ffffff"),
                10.,
            ),
        ];

        let mut color = hit.material.ambient.clone();

        // ray pointing toward eye
        let v = hit.ray.direction.clone().normalize().scale(-1.);

        // hit point pushed out a little bit to avoid self-collisions
        let adjusted_hit = hit.point.clone().add(scene.epsilon(), &hit.normal);

        for light in lights {
            let lc = light.color(&hit.point);
            let mut shadow_ray = light.shadow_ray(&adjusted_hit);
            let ld = shadow_ray.direction.clone().normalize();

            let shadow_ray_count = 1;
            let mut shadow_hit_count = 0;
            for _ in 0..shadow_ray_count {
                let hit = raymarch::raymarch(
                    &perturb(&shadow_ray, 0.),
                    scene,
                    shadow_ray.direction.norm()
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
                v.clone().add(-2., &v.clone().off_axis(&hit.normal))
            );
            if let Some(refl_color) = self.raycast(refl_ray) {
                color = color.add(hit.material.reflectivity, &refl_color);
            }
        }

        color
    }

    fn get_scene(&self) -> UnionSDF {
        let a = sdf::Sphere::new(1.)
            .translate(Vec3::new(0., 0., 5.))
            .shaded({
                let mut m = Material::new();
                m.diffuse = Color::from_hexstring("#ffffff");
                m.ambient = m.diffuse.clone().scale(0.01);
                m.specular = Color::from_hexstring("#ffffff");
                m.phong = 10.;
                m.reflectivity = 1.0;
                m
            });
        let b = sdf::Sphere::new(1.)
            .translate(Vec3::new(-3., 3., 7.))
            .shaded({
                let mut m = Material::new();
                m.diffuse = Color::from_hexstring("#ff88ff");
                m.ambient = m.diffuse.clone().scale(0.01);
                m.specular = Color::from_hexstring("#ffffff");
                m.phong = 10.;
                m.reflectivity = 1.0;
                m
            });
        let c = sdf::Sphere::new(0.5)
            .translate(Vec3::new(1., -2., 4.))
            .shaded({
                let mut m = Material::new();
                m.diffuse = Color::from_hexstring("#8888ff");
                m.ambient = m.diffuse.clone().scale(0.01);
                m.specular = Color::from_hexstring("#ffffff");
                m.phong = 10.;
                m
            });
        let d = sdf::Sphere::new(0.2)
            .translate(Vec3::new(-1., 0.6, 4.5))
            .shaded({
                let mut m = Material::new();
                m.diffuse = Color::from_hexstring("#ffffff");
                m.ambient = m.diffuse.clone().scale(0.01);
                m.specular = Color::from_hexstring("#ffffff");
                m.phong = 10.;
                m
            });
        let floor = sdf::Disk::new(Vec3::up(), 30.0)
            .translate(Vec3::new(0., -10., 0.))
            .shaded({
                let mut m = Material::new();
                m.diffuse = Color::from_hexstring("#ffffff");
                m.ambient = m.diffuse.clone().scale(0.01);
                m
            });
        let scene = floor
            .union(Box::new(a))
            .union(Box::new(b))
            .union(Box::new(c))
            .union(Box::new(d));
        scene
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

impl ViewportApi for Viewport {
    fn handle_key_down(&mut self, key: &str) {
        // TODO
    }
}

impl From<&mat::Color> for JsValue {
    fn from(c: &Color) -> Self {
        JsValue::from_str(&c.to_string())
    }
}
