use crate::linear::*;

pub trait SDF {
    fn distance(&self, point: &Vec3) -> f64;
}

pub struct FuncSdf<F> {
    func: F,
}

impl<F> FuncSdf<F> {
    pub fn new(f: F) -> Self
        where F: Fn(&Vec3) -> f64 {
        FuncSdf {
            func: f
        }
    }
}

impl<F> SDF for FuncSdf<F> where F: Fn(&Vec3) -> f64 {
    fn distance(&self, point: &Vec3) -> f64 {
        (self.func)(point)
    }
}

impl SDF for Vec3 {
    fn distance(&self, point: &Vec3) -> f64 {
        self.dist(point)
    }
}

#[cfg(test)]
mod tests {
    use crate::linear::*;
    use crate::raymarch::*;

    #[test]
    fn func_sdf() {
        // distance from unit sphere at origin
        let f = FuncSdf::new(|v| v.norm() - 1.0);
        assert_eq!(f.distance(&Vec3::right()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::up()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::down()).to_string(), 0.0.to_string());
        assert_eq!(f.distance(&Vec3::new(2.0, 0.0, 0.0)).to_string(), 1.0.to_string());
        assert_eq!(f.distance(&Vec3::zero()).to_string(), (-1.0).to_string());
    }
}
