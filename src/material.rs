use elgan_math::linalg::*;

use crate::{hittable::HitRecord, Ray};

pub trait Material: Send + Sync {
    fn scatter(&self, ray: Ray, rec: HitRecord) -> (Option<Ray>, Option<ColumnVec<3>>);
}

#[derive(Clone, Copy)]
pub enum ColorType {
    Solid(ColumnVec<3>),
    Normal,
    Checker(ColumnVec<3>, ColumnVec<3>, f64),
}

impl ColorType {
    pub fn color(&self, rec: HitRecord) -> ColumnVec<3> {
        match self {
            ColorType::Solid(x) => *x,
            ColorType::Normal => rec.normal * 0.5 + ColumnVec([0.5; 3]),
            ColorType::Checker(x, y, f) => {
                if ((rec.point[0] / f).floor() as i64
                    + (rec.point[1] / f).floor() as i64
                    + (rec.point[2] / f).floor() as i64)
                    % 2
                    == 0
                {
                    *x
                } else {
                    *y
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Lambertian {
    pub color: ColorType,
}

impl Material for Lambertian {
    fn scatter(&self, _ray: Ray, rec: HitRecord) -> (Option<Ray>, Option<ColumnVec<3>>) {
        let mut scatter_direction = ColumnVec::random_in_hemisphere(rec.normal);
        if scatter_direction.close_enough(ColumnVec::zero()) {
            scatter_direction = rec.normal
        }
        (
            Some(Ray {
                origin: rec.point,
                direction: scatter_direction,
            }),
            Some(self.color.color(rec)),
        )
    }
}

#[derive(Clone, Copy)]
pub struct Metal {
    pub color: ColorType,
    pub fuzz: f64,
}

impl Material for Metal {
    fn scatter(&self, ray: Ray, rec: HitRecord) -> (Option<Ray>, Option<ColumnVec<3>>) {
        let reflected = Matrix::reflection_normal_vec(rec.normal) * ray.direction.normalized();
        (
            Some(Ray {
                origin: rec.point,
                direction: reflected + ColumnVec::random_inside_sphere() * self.fuzz,
            }),
            Some(self.color.color(rec)),
        )
    }
}

fn refract(vec: ColumnVec<3>, normal: ColumnVec<3>, ratio: f64) -> ColumnVec<3> {
    let vec = vec.normalized();
    let cos_theta = -(vec * normal);
    let perpendicular = ratio * (vec + cos_theta * normal);
    let parallel = -(1. - perpendicular.length().powf(2.)).sqrt() * normal;
    perpendicular + parallel
}

fn reflect(vec: ColumnVec<3>, normal: ColumnVec<3>) -> ColumnVec<3> {
    let vec = vec.normalized();
    vec - 2. * (vec * normal) * normal
}

fn refelctance(cosine: f64, ref_index: f64) -> f64 {
    let r0 = (1. - ref_index) / (1. + ref_index);
    let r0 = r0 * r0;
    r0 + (1. - r0) * (1. - cosine).powi(5)
}

#[derive(Clone, Copy)]
pub struct Dielectric {
    pub ir: f64,
    pub color: ColorType,
}

impl Material for Dielectric {
    fn scatter(&self, ray: Ray, rec: HitRecord) -> (Option<Ray>, Option<ColumnVec<3>>) {
        let ratio = if rec.front_face {
            1. / self.ir
        } else {
            self.ir
        };
        let cos_theta = -(ray.direction.normalized() * rec.normal);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();
        let refracted = if ratio * sin_theta > 1. || refelctance(cos_theta, ratio) > rand::random()
        {
            reflect(ray.direction, rec.normal)
        } else {
            refract(ray.direction, rec.normal, ratio)
        };
        (
            Some(Ray {
                origin: rec.point,
                direction: refracted,
            }),
            Some(self.color.color(rec)),
        )
    }
}

pub struct Emissive {
    pub color: ColorType,
}

impl Material for Emissive {
    fn scatter(&self, _ray: Ray, rec: HitRecord) -> (Option<Ray>, Option<ColumnVec<3>>) {
        (None, Some(self.color.color(rec)))
    }
}
