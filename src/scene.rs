use std::iter::Iterator;

use regex::Regex;
use wasm_bindgen::__rt::core::f64::consts::PI;
use wasm_bindgen::prelude::*;

use crate::linear::*;
use crate::mat::{Color, Material, RefractionConstants};
use crate::sdf::{EmptySDF, RayHit, SDF, Sphere, UnionSDF, PolyFace, NegatedRefSDF};

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
    pub debugging: bool,
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
                    persp.eye_frame.origin.clone(),
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
        self.raycast(ray, 10)
    }

    fn raycast(&self, ray: Ray, refl_count: usize) -> Option<Color> {
        if ray.origin.is_nan() || ray.direction.is_nan() {
            panic!("Cannot raycast with nan ray: {:#?}", ray);
        }
        let ray_count = 1;
        let mut color = None;
        for _ in 0..ray_count {
            let hit = self.sdf.raymarch(&perturb(&ray, 0.01), self.far_plane);
            if let Some(col) = hit.map(|hit| self.get_color(&hit, refl_count)) {
                color = color.map(|c| &c + &col).or(Some(col))
            }
        }
        color.map(|c| c.scale(1. / (ray_count as f64)))
    }

    fn get_color(&self, hit: &RayHit, refl_count: usize) -> Color {
        if hit.point.is_nan() {
            panic!("Cannot get color for an NaN point! {:#?}", hit);
        }
        let mut color = hit.material.ambient.clone().scale(hit.material.opacity);

        // ray pointing toward eye
        let v = hit.ray.direction.clone().normalize().scale(-1.);

        // hit point pushed out a little bit to avoid self-collisions
        let adjusted_hit = hit.point.clone().add(self.sdf.epsilon(), &hit.normal);

        if self.debugging {
            log(&format!("hit: {:#?}, refl_count: {}", hit, refl_count));
            log(&format!("v_eye: {}", v));
            log(&format!("adjusted_hit: {}", adjusted_hit));
        }

        for light in &self.lights {
            let lc = light.color(&hit.point);
            let mut shadow_ray = light.shadow_ray(&adjusted_hit);
            let ld = shadow_ray.direction.clone().normalize();

            if self.debugging {
                log(&format!("lc {} ld {}", lc, ld));
                log(&format!("shadow ray {}", shadow_ray));
            }

            let mut light_filter = Color::white();

            for _ in 0..refl_count {
                let shadow_dir = shadow_ray.direction.clone().normalize();
                let hit = self.sdf.raymarch(
                    &perturb(&shadow_ray, 0.),
                    shadow_ray.direction.norm(),
                );
                if hit.is_none() {
                    break;
                }
                let hit = hit.unwrap();
                if &hit.normal * &shadow_ray.direction < 0. {
                    if hit.material.opacity < 1.0 {
                        light_filter = light_filter.lerp(hit.material.opacity, &hit.material.diffuse);
                        shadow_ray = light.shadow_ray(&hit.point);
                        shadow_ray.origin = shadow_ray.origin.add(self.sdf.epsilon() * 2., &shadow_dir);
                        let inverse_sdf = NegatedRefSDF::new(&self.sdf);
                        // NB: this doesn't take refraction into account. not sure if I actually can
                        // do that with this lighting method; might have to do some kind of fancy
                        // photon simulation thing.
                        let refr_hit = inverse_sdf.raymarch(&shadow_ray, shadow_ray.direction.norm());
                        if refr_hit.is_none() {
                            break;
                        }
                        let refr_hit = refr_hit.unwrap();
                        shadow_ray.origin = refr_hit.point.clone().add(self.sdf.epsilon() * 2., &shadow_dir);
                    } else {
                        light_filter = Color::black();
                    }
                    break;
                }
                shadow_ray.origin = hit.point.clone()
                    .add(self.sdf.epsilon() * 2., &shadow_dir);
            }

            if light_filter.is_black() {
                continue; // fully in shadow.
            }
            let lc = &light_filter * &lc;

            // reflection of direction to light
            let lr = ld.clone().add(-2., &ld.clone().off_axis(&hit.normal));

            let diffuse_strength = (&ld * &hit.normal).max(0.);
            let specular_strength = (&lr * &v).max(0.).powf(hit.material.phong);
            color = color
                .add(diffuse_strength * hit.material.opacity, &(&hit.material.diffuse * &lc))
                // nb: intentionally not scaling specular highlights by opacity.
                .add(specular_strength, &hit.material.specular)
        }

        if hit.material.reflectivity > 0. && refl_count > 0 {
            let refl_ray = Ray::new(
                adjusted_hit,
                v.clone().add(-2., &v.clone().off_axis(&hit.normal)),
            );
            if refl_ray.origin.is_nan() || refl_ray.direction.is_nan() {
                panic!("cannot cast NaN reflection ray: {:#?} v={}, normal={}", refl_ray,
                       v, hit.normal);
            }
            if let Some(refl_color) = self.raycast(refl_ray, refl_count - 1) {
                color = color.add(hit.material.reflectivity * hit.material.opacity, &refl_color);
            }
        }

        if hit.material.opacity < 1. && refl_count > 0 {
            if self.debugging {
                log(&format!("firing refraction ray for material with opacity {}",
                             hit.material.opacity));
            }
            let refr_ray = Ray::new(
                hit.point.clone().add(-self.sdf.epsilon() * 2., &hit.normal),
                refract(
                    &hit.ray.direction,
                    &hit.normal,
                    RefractionConstants::VACUUM,
                    hit.material.index_of_refraction,
                ),
            );
            if self.debugging {
                log(&format!("refraction ray: {}", refr_ray));
            }
            let inverse_sdf = NegatedRefSDF::new(&self.sdf);
            if let Some(farside_hit) = inverse_sdf.raymarch(&refr_ray, self.far_plane) {
                // hit the far side of the interior of this shape.
                let refr_ray = Ray::new(
                    farside_hit.point.clone().add(-self.sdf.epsilon() * 2., &farside_hit.normal),
                    refract(
                        &farside_hit.ray.direction,
                        &farside_hit.normal,
                        hit.material.index_of_refraction,
                        RefractionConstants::VACUUM,
                    ),
                );
                if self.debugging {
                    log(&format!("refraction interior hit: {:#?}", farside_hit));
                    log(&format!("hit interior of shape with refraction ray, firing again: {}", refr_ray));
                }
                // TODO: apply coloration based on how far we went through the material?
                if refr_ray.origin.is_nan() || refr_ray.direction.is_nan() {
                    panic!("cannot cast NaN refraction ray: {:#?} farside hit: {:#?}", refr_ray,
                           farside_hit);
                }
                if let Some(refr_color) = self.raycast(refr_ray, refl_count - 1) {
                    if self.debugging {
                        log(&format!("transparency color: {}", refr_color));
                    }
                    // hit from ray shooting out the other side of this shape
                    color = color.add(1.0 - hit.material.opacity, &refr_color);
                }
            }
        }

        color
    }

    pub fn parse<'a, L>(lines: L) -> Self where L: Iterator<Item=&'a str> {
        let mut objects: Vec<Box<dyn SDF>> = vec![];
        let mut lights: Vec<Light> = vec![];
        let mut view: ViewTransform = ViewTransform::Ortho(OrthoView { frame: Frame::identity() });
        let mut far_plane = 1_000.;

        let default_attenuation = 50.;

        let space = Regex::new(r"\s+").unwrap();

        let mut material = Material::new();
        let mut poly_vertices: Vec<Vec3> = vec![];

        for line in lines {
            let line: &str = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = space.split(line).collect();
            let command = parts[0];
            let arguments = &parts[1..];
            match (command, arguments) {
                ("fov", [fov]) => {
                    view = ViewTransform::Persp(PerspView {
                        eye_frame: Frame::new(
                            Vec3::new(0., 0., 0.),
                            Vec3::right(),
                            Vec3::up(),
                            Vec3::backward(),
                        ),
                        near: 1.0,
                        fov_degrees: fov.parse().unwrap(),
                    })
                }
                ("light", [x, y, z, r, g, b]) => {
                    let light = Light::new(
                        Vec3::new(x.parse().unwrap(), y.parse().unwrap(), z.parse().unwrap()),
                        Color::new(r.parse().unwrap(), g.parse().unwrap(), b.parse().unwrap()),
                        default_attenuation,
                    );
                    lights.push(light);
                }
                ("surface", [
                dr, dg, db,
                ar, ag, ab,
                sr, sg, sb,
                phong, krefl
                ]) => {
                    material = Material::new();
                    material.ambient = Color::new(ar.parse().unwrap(), ag.parse().unwrap(), ab.parse().unwrap());
                    material.diffuse = Color::new(dr.parse().unwrap(), dg.parse().unwrap(), db.parse().unwrap());
                    material.specular = Color::new(sr.parse().unwrap(), sg.parse().unwrap(), sb.parse().unwrap());
                    material.phong = phong.parse().unwrap();
                    material.reflectivity = krefl.parse().unwrap();
                }
                ("sphere", [radius, x, y, z]) => {
                    objects.push(Box::new(
                        Sphere::new(radius.parse().unwrap())
                            .translate(Vec3::new(
                                x.parse().unwrap(),
                                y.parse().unwrap(),
                                z.parse().unwrap(),
                            ))
                            .shaded(material.clone())
                    ));
                }
                ("background", [r, g, b]) => {
                    objects.push(Box::new(
                        Sphere::new(100.)
                            .negate()
                            .shaded({
                                let mut m = Material::new();
                                m.ambient = Color::new(
                                    r.parse().unwrap(),
                                    g.parse().unwrap(),
                                    b.parse().unwrap(),
                                );
                                m.diffuse = Color::black();
                                m.specular = Color::black();
                                m
                            })
                    ));
                }
                ("begin", []) => {
                    poly_vertices.clear();
                }
                ("end", []) => {
                    objects.push(Box::new(
                        PolyFace::new(poly_vertices.clone())
                            .shaded(material.clone())
                    ));
                }
                ("vertex", [x, y, z]) => {
                    poly_vertices.push(Vec3::new(
                        x.parse().unwrap(),
                        y.parse().unwrap(),
                        z.parse().unwrap(),
                    ));
                }
                ("write", [filepath]) => {
                    // ignored
                }
                (command, args) => {
                    log(&format!("ignoring unknown command: {} {:?}", command, args))
                }
            }
        }

        let mut sdf: Option<Box<dyn SDF>> = None;
        for obj in objects {
            sdf = match sdf {
                Some(sdf) => Some(Box::new(UnionSDF::new(sdf, obj))),
                None => Some(obj),
            };
        }
        let sdf = sdf.unwrap_or(Box::new(EmptySDF {}));

        Self {
            sdf,
            lights,
            view,
            far_plane,
            debugging: false,
        }
    }
}

fn refract(incoming: &Vec3, normal: &Vec3, n1: f64, n2: f64) -> Vec3 {
    // n1 sin theta1 = n2 sin theta2
    // sin theta2 =  (n1 sin theta1) / n2
    // theta2 = asin ((n1 sin theta1) / n2)

    if (n1 - n2).abs() < 0.0001 {
        return incoming.clone();
    }

    let incoming = incoming.clone().normalize();
    let cos_theta1 = -(&incoming * normal);
    if (cos_theta1.abs() - 1.).abs() < 0.0001 {
        // if the incoming ray is orthogonal to the surface, there isn't any refraction to do.
        return incoming.clone();
    }

    let theta1 = cos_theta1.clamp(-1., 1.).acos().abs();
    let theta2 = ((n1 * theta1.sin()) / n2).clamp(-1., 1.).asin();

    let axis_of_rotation = (&incoming ^ normal).normalize();

    let result = normal.clone()
        .scale((&incoming * normal).signum())
        .rotate(theta2, &axis_of_rotation);

    if result.is_nan() {
        panic!(
            "refracted to NaN?! I={}, N={}, n1={}, n2={}, I^N={}, theta1={}, theta2={}",
            incoming, normal, n1, n2, axis_of_rotation, theta1, theta2
        );
    }

    result
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

#[cfg(test)]
mod tests {
    use crate::linear::Vec3;
    use crate::scene::refract;
    use wasm_bindgen::__rt::core::f64::consts::PI;

    #[test]
    fn refraction() {
        let incoming = Vec3::new(-1., -1., 0.).normalize();
        let normal = Vec3::new(0., 1., 0.);

        // no refraction
        let refracted = check_snell(&incoming, &normal, 1., 1.);

        // no refraction when the indices are equal
        assert_eq!(
            incoming.to_string(),
            refracted.to_string(),
            "expected no refraction (should maintain 45° angle)"
        );

        // should bend 45/2 degrees toward normal
        check_snell(&incoming, &normal, 1., 2.);

        // parallel to normal
        check_snell(&normal, &normal, 1., 2.);

        // orthogonal to normal
        check_snell(&Vec3::right(), &normal, 1., 2.);

        // check for consistency with incoming vector from different quadrants
        check_snell(&Vec3::new(1., 1., 0.).normalize(), &normal, 1., 2.);
        check_snell(&Vec3::new(1., -1., 0.).normalize(), &normal, 1., 2.);
        check_snell(&Vec3::new(-1., 1., 0.).normalize(), &normal, 1., 2.);
    }

    fn check_snell(incoming: &Vec3, normal: &Vec3, n1: f64, n2: f64) -> Vec3 {
        let refracted = refract(incoming, normal, n1, n2);
        let theta1 = (incoming * normal).acos();
        let theta2 = (&refracted * normal).acos();
        assert!(
            ((n1 * theta1.sin()) - (n2 * theta2.sin())).abs() < 0.0001,
            format!(
                "snell's equation was violated:\n{} sin({}°) =/= {} sin({}°)\n\
                I: {}, R: {}, N: {}",
                n1, theta1 * 180. / PI, n2, theta2 * 180. / PI,
                incoming, normal, refracted,
            )
        );

        if (incoming * normal).abs() > 0.0001 {
            // won't hold true when incoming is orthogonal to normal
            assert_eq!(
                (incoming * normal).signum(),
                (&refracted * normal).signum(),
                "refracted ray should be on the same front/back side of normal as incoming ray \
                incoming ray:\nI: {}, R: {}, N: {}\ntheta1: {:.2}°, theta2: {:.2}°\n",
                incoming, refracted, normal, theta1 * 180. / PI, theta2 * 180. / PI,
            );
        }

        if (incoming * normal).abs() < 1.0 {
            // won't hold true when incoming is parallel to normal
            let axis_of_rotation = &(incoming ^ normal);
            assert_eq!(
                (incoming * axis_of_rotation).signum(),
                (&refracted * axis_of_rotation).signum(),
                "refracted ray should be on the same left/right of normal as the incoming ray \
                incoming ray:\nI: {}, R: {}, N: {}\ntheta1: {:.2}°, theta2: {:.2}°\n",
                incoming, refracted, normal, theta1 * 180. / PI, theta2 * 180. / PI,
            );
        }

        refracted
    }
}
