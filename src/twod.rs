use js_sys::Math;
use std::ops;

pub trait HasComponents<T> {
    fn size(&self) -> usize;

    fn get_comp(&self, index: usize) -> T;

    fn set_comp(&mut self, index: usize, value: T) -> ();

    fn set_to<V>(&mut self, other: &V)
        where V: HasComponents<T> {
        for i in 0..self.size() {
            self.set_comp(i, other.get_comp(i));
        }
    }
}

pub trait Position: HasComponents<f64> + Clone {
}

pub trait Vector: HasComponents<f64> + Clone {
    fn dot<V>(&self, other: &V) -> f64
        where V: Vector {
        (0..self.size())
            .map(|index| self.get_comp(index) * other.get_comp(index))
            .sum()
    }

    fn norm2(&self) -> f64 {
        self.dot(self)
    }

    fn norm(&self) -> f64 {
        Math::sqrt(self.norm2())
    }

    fn scale(&mut self, scalar: f64) {
        for i in 0..self.size() {
            self.set_comp(i, self.get_comp(i) * scalar);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Pos2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Pos2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn det(&self, other: &Vec2) -> f64 {
        self.x * other.y - other.x * self.y
    }
}

impl<V: Vector> ops::Add<V> for Pos2 {
    type Output = Pos2;

    fn add(self, rhs: V) -> Self::Output {
        let mut result = self.clone();
        for i in 0..self.size() {
            result.set_comp(i, self.get_comp(i) + rhs.get_comp(i));
        }
        result
    }
}

impl<V: Vector> ops::Sub<V> for Pos2 {
    type Output = Pos2;

    fn sub(self, rhs: V) -> Self::Output {
        let mut result = self.clone();
        for i in 0..self.size() {
            result.set_comp(i, self.get_comp(i) - rhs.get_comp(i));
        }
        result
    }
}

impl ops::Mul<Vec2> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl ops::Mul<f64> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl ops::Mul<f64> for Pos2 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl ops::Div<f64> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        self * (1.0 / rhs)
    }
}

impl ops::Div<f64> for Pos2 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        self * (1.0 / rhs)
    }
}

impl ops::Div<Vec2> for Vec2 {
    type Output = Self;

    fn div(self, rhs: Vec2) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl ops::Sub<Pos2> for Pos2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl HasComponents<f64> for Vec2 {
    fn size(&self) -> usize {
        2
    }

    fn get_comp(&self, index: usize) -> f64 {
        match index {
            0 => self.x,
            1 => self.y,
            _ => 0.0,
        }
    }

    fn set_comp(&mut self, index: usize, value: f64) -> () {
        match index {
            0 => {
                self.x = value;
            },
            1 => {
                self.y = value;
            },
            _ => {},
        }
    }
}

impl HasComponents<f64> for Pos2 {
    fn size(&self) -> usize {
        2
    }

    fn get_comp(&self, index: usize) -> f64 {
        match index {
            0 => self.x,
            1 => self.y,
            _ => 0.0,
        }
    }

    fn set_comp(&mut self, index: usize, value: f64) -> () {
        match index {
            0 => {
                self.x = value;
            },
            1 => {
                self.y = value;
            },
            _ => {},
        }
    }
}

impl Vector for Vec2 {}

impl Position for Pos2 {
}

impl From<Pos2> for Vec2 {
    fn from(vec: Pos2) -> Self {
        Self::new(vec.get_comp(0), vec.get_comp(1))
    }
}

impl From<Vec2> for Pos2 {
    fn from(vec: Vec2) -> Self {
        Self::new(vec.get_comp(0), vec.get_comp(1))
    }
}

impl From<Vec2> for (f64, f64) {
    fn from(vec: Vec2) -> Self {
        (vec.get_comp(0), vec.get_comp(1))
    }
}

impl From<Pos2> for (f64, f64) {
    fn from(vec: Pos2) -> Self {
        (vec.get_comp(0), vec.get_comp(1))
    }
}

impl From<(f64, f64)> for Vec2 {
    fn from(tup: (f64, f64)) -> Self {
        Self::new(tup.0, tup.1)
    }
}

impl From<(u32, u32)> for Vec2 {
    fn from(tup: (u32, u32)) -> Self {
        Self::new(tup.0.into(), tup.1.into())
    }
}

impl From<(i32, i32)> for Vec2 {
    fn from(tup: (i32, i32)) -> Self {
        Self::new(tup.0.into(), tup.1.into())
    }
}
