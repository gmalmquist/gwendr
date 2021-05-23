use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::linear::{Frame, Ray, Vec3};
use crate::mat;
use crate::mat::{Color, Material};
use crate::scene::{Light, OrthoView, PerspView, Scene, ViewTransform};
use crate::sdf;
use crate::sdf::SDF;
use crate::utils::current_time_millis;

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
        if self.frame > 0 {
            return;
        }

        //self.context.clear_rect(0., 0., self.canvas.width().into(), self.canvas.height().into());
        let width = self.canvas.width() as usize;
        let height = self.canvas.height() as usize;

        let mut scene = self.get_scene();

        if self.frame == 0 && self.index == 0 {
            scene.debugging = true;
            if let Some(c) = scene.raycast_pixel((width / 2, height / 2), width, height) {
                log(&format!("debugged pixel color: {}", c));
            }
            scene.debugging = false;
        }

        let time_budget_millis = 100.;
        let start_time_millis = current_time_millis();

        while current_time_millis() - start_time_millis < time_budget_millis {
            self.render_next_point(&scene);
        }

        self.seed += 29;
    }

    fn render_next_point(&mut self, scene: &Scene) {
        let width = self.canvas.width() as usize;
        let height = self.canvas.height() as usize;

        let x = self.index % width;
        let y = self.index / width;
        let pixel = (x, y);
        let x = x as f64;
        let y = y as f64;

        let black = Color::black().to_string().into();

        if let Some(color) = scene.raycast_pixel(pixel, width, height) {
            let color = &color;
            self.context.set_fill_style(&color.into());
            self.context.fill_rect(x, y, 1., 1.);

            if self.frame == 0 {
                // log(&format!("eye: {}", &hit.ray));
                // log(&format!("hit: {:#?}", &hit.distance));
            }
        } else {
            self.context.set_fill_style(&black);
            self.context.fill_rect(x, y, 1., 1.);
        }

        self.index = (self.index + 1) % (width * height);
        if self.index == 0 {
            self.frame += 1;
        }
    }


    fn get_scene(&self) -> Scene {
        if false {
            let scene = "
# four reflective spheres

fov 60

# blue background

background 0.2 0.2 1

# one light source

light 0.4 0 0  .5 .5 .5

# four reflective spheres

surface  .7 .7 .7  0 0 0  0 0 0  20 0.7

sphere 1.41421356   1  1 -1
sphere 1.41421356  -1 -1 -1
sphere 1.41421356   1 -1  1
sphere 1.41421356  -1  1  1

write c5.png
            ";
//             let scene = "
// # two spheres, one shiny
//
// fov 60
// light -5 2 5 1 1 1
//
// surface   0.5 0 0   0.2 0 0   0.7 0.7 0.7  20  0.7
// sphere 1  0 0 -4
//
// surface   0 0.5 0   0 0.2 0   0 0 0   1 0
// sphere .3  1 0.6 -3
//
// write t3.png
//
// ";
            return Scene::parse(scene.split("\n"));
        }

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
            .translate(Vec3::new(1., -2., 4. - 2.))
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
        let sdf = floor
            .union(Box::new(a))
            .union(Box::new(b))
            .union(Box::new(c))
            .union(Box::new(d));

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

        Scene {
            sdf: Box::new(sdf),
            lights,
            view: ViewTransform::Persp(PerspView {
                eye_frame: Frame::identity().translate(&(&Vec3::backward() * 5.)),
                near: 1.,
                fov_degrees: 60.,
            }),
            // view: ViewTransform::Ortho(OrthoView {
            //     frame: Frame::new(
            //         Vec3::new(0., 0., 1.),
            //         Vec3::new(10., 0., 0.),
            //         Vec3::new(0., 10., 0.),
            //         Vec3::new(0., 0., 1.),
            //     )
            // }),
            far_plane: 1_000.,
            debugging: false,
        }
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
