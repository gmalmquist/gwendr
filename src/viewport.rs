use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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
        }
    }
}

impl ViewportApi for Viewport {
    fn update(&mut self) {
        self.context.clear_rect(0., 0., self.canvas.width().into(), self.canvas.height().into());
    }

    fn handle_key_down(&mut self, key: &str) {
        // TODO
    }
}
