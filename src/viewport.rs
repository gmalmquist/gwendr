use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::mat;
use crate::raymarch;
use crate::sdf;
use crate::linear::{Frame, Vec3, Basis, Ray};
use crate::sdf::{DynFuncSdf, SDF, Sphere, UnionSDF};
use crate::raymarch::RayHit;
use crate::scene::Light;
use crate::mat::Color;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct Viewport {
    canvas: web_sys::HtmlCanvasElement,
    context: web_sys::CanvasRenderingContext2d,
    index: usize,
    seed: u64,
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

        let near_plane = 1.;

        let canvas_frame = Frame::new(
            Vec3::new(width as f64 / 2.0, height as f64 / 2.0, 0.),
            Vec3::right().scale(width as f64 / 2.0),
            Vec3::up().scale(-(height as f64 / 2.0)),
            Vec3::forward(),
        );

        let x = (self.index % width) as f64;
        let y = (self.index / width) as f64;

        let world_frame = Frame::identity();

        let canvas_point = Vec3::new(x, y, 0.);
        let local_point = canvas_frame.unproject_point(&canvas_point);
        let world_point = world_frame.project_point(&local_point);

        let eye = Vec3::zero().add(near_plane, &Vec3::backward());
        let ray = Ray::new(eye.clone(), (&world_point - &eye).normalize());

        let scene = self.get_scene();

        let hit = raymarch::raymarch(ray, &scene);

        if let Some(hit) = hit {
            let color = &self.get_color(&hit, &scene);
            self.context.set_fill_style(&color.into());
            self.context.fill_rect(x, y, 1., 1.);
        } else {
            let rand = ((self.index as u64 ^ self.seed) as f64);
            let color = mat::Color::new(rand.sin(), (rand + 23.).sin(), (rand + 7.).cos());
            self.context.set_fill_style(&JsValue::from_str(&color.to_string()));
            self.context.fill_rect(x, y, 1., 1.);
        }

        self.index = (self.index + 1) % (width * height);
    }

    fn get_color<F>(&self, hit: &RayHit, scene: &F) -> mat::Color where F: sdf::SDF {
        let light = Light::new(
            Vec3::new(-10.0, 10.0, -10.0),
            Color::from_hexstring("#ff88ff"),
        );

        let lc = light.color(&hit.point);
        let shadow_ray = light.shadow_ray(&hit.point);
        let ld = shadow_ray.direction.clone().normalize();
        // TODO shadow rays.

        hit.material.ambient.clone()
            .add((&ld * &hit.normal).max(0.), &hit.material.diffuse)
        // TODO: specular etc
    }

    fn get_scene(&self) -> UnionSDF<Sphere, Sphere> {
        let a = sdf::Sphere::new(Vec3::new(0., 0., 5.), 1.);
        let b = sdf::Sphere::new(Vec3::new(-3., 3., 5.), 1.);
        let scene = sdf::SDF::union(a, b);
        scene
    }
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
