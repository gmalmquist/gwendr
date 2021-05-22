use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::raymarch;
use crate::sdf;
use crate::linear::{Frame, Vec3, Basis, Ray};

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
        }
    }

    pub fn update(&mut self) {
        //self.context.clear_rect(0., 0., self.canvas.width().into(), self.canvas.height().into());
        for _ in 0..500 {
            self.render_next_point();
        }
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

        let scene = sdf::Sphere::new(Vec3::new(0., 0., 5.), 1.);

        let hit = raymarch::raymarch(ray, &scene);

        if let Some(hit) = hit {
            self.context.set_fill_style(&JsValue::from_str("#ffffff"));
            self.context.fill_rect(x, y, 1., 1.);
        } else {
            self.context.set_fill_style(&JsValue::from_str("#ff00ff"));
            self.context.fill_rect(x, y, 1., 1.);
        }

        self.index = (self.index + 1) % (width * height);
    }
}

impl ViewportApi for Viewport {
    fn handle_key_down(&mut self, key: &str) {
        // TODO
    }
}
