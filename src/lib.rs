use elgan_math::linalg::*;
use image::Rgb;

pub mod camera;
pub mod hittable;
pub mod material;
pub mod render;

pub fn to_color(vec: ColumnVec<3>) -> Rgb<u8> {
    //Rgb(vec.0.map(|i: f64| (i.clamp(0., 1.) * 255.) as u8))
    Rgb(vec.0.map(|i: f64| (i.clamp(0., 1.).sqrt() * 255.) as u8))
}

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: ColumnVec<3>,
    pub direction: ColumnVec<3>,
}

impl Ray {
    pub fn new(origin: ColumnVec<3>, direction: ColumnVec<3>) -> Self {
        Self { origin, direction }
    }

    pub fn at(self, t: f64) -> ColumnVec<3> {
        self.origin + t * self.direction
    }
}
