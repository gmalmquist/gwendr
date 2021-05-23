use std::fmt;
use std::ops;

#[derive(Clone, Debug)]
pub struct Material {
    pub ambient: Color,
    pub diffuse: Color,
    pub specular: Color,
    pub phong: f64,
    pub reflectivity: f64,
}

impl Material {
    pub fn new() -> Self {
        Self {
            ambient: Color::black(),
            diffuse: Color::white(),
            specular: Color::black(),
            phong: 1.,
            reflectivity: 0.,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    pub fn from_irgb(r: usize, g: usize, b: usize) -> Self {
        Self::new(
            (r as f64) / 255.,
            (g as f64) / 255.,
            (b as f64) / 255.,
        )
    }

    pub fn from_hexstring(color: &str) -> Self {
        let color = if color.starts_with('#') {
            &color[1..]
        } else {
            &color
        };

        assert_eq!(color.len(), 6, "color string must contain six characters.");

        let r = usize::from_str_radix(&color[0..2], 16).unwrap();
        let g = usize::from_str_radix(&color[2..4], 16).unwrap();
        let b = usize::from_str_radix(&color[4..6], 16).unwrap();

        Self::from_irgb(r, g, b)
    }

    pub fn black() -> Self {
        Self::new(0., 0., 0.)
    }

    pub fn white() -> Self {
        Self::new(1., 1., 1.)
    }

    pub fn add(mut self, scale: f64, other: &Color) -> Self {
        self.r += scale * other.r;
        self.g += scale * other.g;
        self.b += scale * other.b;
        self
    }

    pub fn scale(mut self, scale: f64) -> Self {
        self.r *= scale;
        self.g *= scale;
        self.b *= scale;
        self
    }

    pub fn multiply(mut self, other: &Color) -> Self {
        self.r *= other.r;
        self.g *= other.g;
        self.b *= other.b;
        self
    }

    pub fn as_hexstring(&self) -> String {
        let r = convert_to_255(self.r);
        let g = convert_to_255(self.g);
        let b = convert_to_255(self.b);
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

fn convert_to_255(f: f64) -> usize {
    (f.max(0.).min(1.0) * 255.) as usize
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.as_hexstring())
    }
}

impl ops::Add<&Color> for &Color {
    type Output = Color;

    fn add(self, rhs: &Color) -> Self::Output {
        self.clone().add(1.0, rhs)
    }
}

impl ops::Sub<&Color> for &Color {
    type Output = Color;

    fn sub(self, rhs: &Color) -> Self::Output {
        self.clone().add(-1.0, rhs)
    }
}

impl ops::Mul<&Color> for &Color {
    type Output = Color;

    fn mul(self, rhs: &Color) -> Self::Output {
        self.clone().multiply(rhs)
    }
}

impl ops::Mul<f64> for &Color {
    type Output = Color;

    fn mul(self, rhs: f64) -> Self::Output {
        self.clone().scale(rhs)
    }
}

impl ops::Mul<&Color> for f64 {
    type Output = Color;

    fn mul(self, rhs: &Color) -> Self::Output {
        rhs.clone().scale(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::mat::*;

    #[test]
    fn hexstring() {
        assert_eq!("#0fff08", Color::from_irgb(15, 255, 8).as_hexstring());
        assert_eq!(Color::white().to_string(), "#ffffff");
        assert_eq!(Color::black().to_string(), "#000000");
        assert_eq!(Color::from_hexstring("#ffffff").to_string(), "#ffffff");
        assert_eq!(Color::from_hexstring("#000000").to_string(), "#000000");
    }
}
