use crate::linear::*;
use crate::mat::{Color, Material};

pub struct Light {
    pub position: Vec3,
    pub color: Color,
}

impl Light {
    pub fn new(position: Vec3, color: Color) -> Self {
        Self { position, color }
    }

    pub fn shadow_ray(&self, point: &Vec3) -> Ray {
        Ray::new(point.clone(), &self.position - point)
    }

    pub fn color(&self, _point: &Vec3) -> Color {
        self.color.clone()
    }
}
