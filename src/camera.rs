use elgan_math::linalg::*;

use crate::Ray;

#[derive(Clone, Copy)]
pub struct Camera {
    pub aspect: f64,
    pub width: u32,
    pub height: u32,
    pub view_height: f64,
    pub view_width: f64,
    pub focal_length: f64,
    pub origin: ColumnVec<3>,
    pub horizontal: ColumnVec<3>,
    pub vertical: ColumnVec<3>,
    pub lower_left: ColumnVec<3>,
}

impl Camera {
    pub fn new(
        width: u32,
        height: u32,
        view_height: f64,
        focal_length: f64,
        origin: ColumnVec<3>,
    ) -> Self {
        let aspect = width as f64 / height as f64;
        let view_width = view_height * aspect;
        let horizontal = ColumnVec([view_width, 0., 0.]);
        let vertical = ColumnVec([0., view_height, 0.]);
        let lower_left =
            origin - horizontal / 2. - vertical / 2. - ColumnVec([0., 0., focal_length]);
        Self {
            aspect,
            width,
            height,
            view_height,
            view_width,
            focal_length,
            origin,
            horizontal,
            vertical,
            lower_left,
        }
    }

    pub fn get_ray(self, u: f64, v: f64) -> Ray {
        Ray {
            origin: self.origin,
            direction: self.lower_left + u * self.horizontal + v * self.vertical - self.origin,
        }
    }
}
