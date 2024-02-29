use std::sync::Arc;

use elgan_math::linalg::*;

use crate::{material::Material, Ray};

#[derive(Clone)]
pub struct HitRecord {
    pub point: ColumnVec<3>,
    pub normal: ColumnVec<3>,
    pub t: f64,
    pub front_face: bool,
    pub material: Arc<dyn Material>,
}

impl HitRecord {
    fn new(ray: Ray, normal: ColumnVec<3>, t: f64, material: Arc<dyn Material>) -> Self {
        let front_face = ray.direction * normal < 0.;
        Self {
            point: ray.at(t),
            normal: if front_face { normal } else { -normal },
            t,
            front_face,
            material,
        }
    }
}

pub trait Hittable: Sync {
    /// If the ray hits the object within the specified bounds, returns a record of the hit.
    /// If the ray does not hit returns none.
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

#[derive(Clone)]
pub struct Sphere {
    pub center: ColumnVec<3>,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl Hittable for Sphere {
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction * ray.direction;
        let half_b = oc * ray.direction;
        let c = oc * oc - self.radius * self.radius;
        let discrim = half_b * half_b - a * c;
        if discrim < 0. {
            return None;
        }
        let root = (-half_b - discrim.sqrt()) / a;
        if t_min < root && root < t_max {
            return Some(HitRecord::new(
                ray,
                (ray.at(root) - self.center).normalized(),
                root,
                self.material.clone(),
            ));
        }
        let root = (-half_b + discrim.sqrt()) / a;
        if t_min < root && root < t_max {
            return Some(HitRecord::new(
                ray,
                (ray.at(root) - self.center).normalized(),
                root,
                self.material.clone(),
            ));
        }
        None
    }
}

#[derive(Clone)]
pub struct Triangle {
    pub vertices: [ColumnVec<3>; 3],
    pub material: Arc<dyn Material>,
}

impl Triangle {
    pub fn normal(&self) -> ColumnVec<3> {
        (self.vertices[1] - self.vertices[0])
            .cross(self.vertices[2] - self.vertices[0])
            .normalized()
    }
}

impl Hittable for Triangle {
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let new_ray = Ray {
            origin: ray.origin - self.vertices[0],
            direction: ray.direction,
        };
        let transform = Matrix::from_columns([
            self.vertices[1] - self.vertices[0],
            self.vertices[2] - self.vertices[0],
            new_ray.direction,
        ]);
        let transform_inv = transform.inverse()?;
        let new_ray = Ray {
            origin: transform_inv * new_ray.origin,
            direction: transform_inv * new_ray.direction,
        };
        if new_ray.origin[0] > 0.
            && new_ray.origin[1] > 0.
            && new_ray.origin[0] + new_ray.origin[1] < 1.
            && -new_ray.origin[2] > t_min
            && -new_ray.origin[2] < t_max
        {
            Some(HitRecord::new(
                ray,
                self.normal(),
                -new_ray.origin[2],
                self.material.clone(),
            ))
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Parallelogram {
    pub vertices: [ColumnVec<3>; 3],
    pub material: Arc<dyn Material>,
}

impl Parallelogram {
    pub fn normal(&self) -> ColumnVec<3> {
        (self.vertices[1] - self.vertices[0])
            .cross(self.vertices[2] - self.vertices[0])
            .normalized()
    }
}

impl Hittable for Parallelogram {
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let new_ray = Ray {
            origin: ray.origin - self.vertices[0],
            direction: ray.direction,
        };
        let transform = Matrix::from_columns([
            self.vertices[1] - self.vertices[0],
            self.vertices[2] - self.vertices[0],
            new_ray.direction,
        ]);
        let transform_inv = transform.inverse()?;
        let new_ray = Ray {
            origin: transform_inv * new_ray.origin,
            direction: transform_inv * new_ray.direction,
        };
        if new_ray.origin[0] > 0.
            && new_ray.origin[1] > 0.
            && new_ray.origin[0] < 1.
            && new_ray.origin[1] < 1.
            && -new_ray.origin[2] > t_min
            && -new_ray.origin[2] < t_max
        {
            Some(HitRecord::new(
                ray,
                self.normal(),
                -new_ray.origin[2],
                self.material.clone(),
            ))
        } else {
            None
        }
    }
}

pub struct World {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl World {
    pub fn new() -> Self {
        Self { objects: vec![] }
    }
    pub fn push(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl Hittable for World {
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut result = None;
        let mut closest = t_max;
        for object in &self.objects {
            if let Some(rec) = object.hit(ray, t_min, closest) {
                result = Some(rec.clone());
                closest = rec.t;
            }
        }
        result
    }
}
