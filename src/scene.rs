use crate::linear::*;
use crate::mat::{Color, Material};

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
