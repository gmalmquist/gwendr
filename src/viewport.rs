use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::linear::{Frame, Vec3};

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
    fn update(&mut self);
    fn handle_key_down(&mut self, key: &str);
}

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
}

impl ViewportApi for Viewport {
    fn update(&mut self) {
        //self.context.clear_rect(0., 0., self.canvas.width().into(), self.canvas.height().into());
        let width = self.canvas.width() as usize;
        let height = self.canvas.height() as usize;

        // TODO: do the projection when you're not hungry :)
        let canvas_frame = Frame::new(
            Vec3::new(width as f64 / 2.0, height as f64 / 2.0, 0.0),
            Vec3::right().scale(width/2 as f64),
            Vec3::up().scale(-height/2 as f64),
            Vec3::forward(),
        );

        let x = self.index % width;
        let y = self.index / width;

        let world_frame = Frame::identity();

        let near_plane = -1.;



        self.index = (self.index + 1) % (width * height);
    }

    fn handle_key_down(&mut self, key: &str) {
        // TODO
    }
}
