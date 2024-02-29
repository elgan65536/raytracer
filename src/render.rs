use std::sync::{Arc, Mutex};

use elgan_math::linalg::ColumnVec;
use image::{ImageBuffer, RgbImage};
use rand::Rng;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    camera::Camera,
    hittable::{Hittable, Sphere, World},
    material::{ColorType, Dielectric, Emissive, Lambertian, Metal},
    to_color, Ray,
};

fn ray_color(r: Ray, world: &World, depth: i32) -> ColumnVec<3> {
    if depth <= 0 {
        return ColumnVec::zero();
    }
    if let Some(rec) = world.hit(r, 0.00069420, f64::INFINITY) {
        match rec.material.clone().scatter(r, rec) {
            (Some(scattered), Some(color)) => {
                return color.component_mul(ray_color(scattered, world, depth - 1));
            }
            (None, Some(color)) => return color,
            _ => (),
        }
    }
    let unit_dir = r.direction.normalized();
    let t = 0.5 * unit_dir[1] + 0.5;
    // (1. - t) * ColumnVec([1.; 3]) + t * ColumnVec([0.5, 0.7, 1.0])
    (1. - t) * ColumnVec([0.08, 0.1, 0.2]) + t * ColumnVec([0.032, 0.04, 0.08])
}

pub fn ray_diffuse_glass() {
    let mut world = World::new();
    world.push(Box::new(Sphere {
        center: ColumnVec([0., -100.5, -1.]),
        radius: 100.,
        material: Arc::new(Lambertian {
            color: ColorType::Checker(ColumnVec([0.4, 0.8, 0.4]), ColumnVec([0.6, 1., 0.6]), 0.25),
        }),
    }));
    world.push(Box::new(Sphere {
        center: ColumnVec([-3., 2., -5.]),
        radius: 1.,
        material: Arc::new(Emissive {
            color: ColorType::Solid(ColumnVec([5., 0.2, 0.3])),
        }),
    }));
    world.push(Box::new(Sphere {
        center: ColumnVec([0., 2., -5.]),
        radius: 1.,
        material: Arc::new(Emissive {
            color: ColorType::Solid(ColumnVec([0.3, 5., 0.2])),
        }),
    }));
    world.push(Box::new(Sphere {
        center: ColumnVec([3., 2., -5.]),
        radius: 1.,
        material: Arc::new(Emissive {
            color: ColorType::Solid(ColumnVec([0.3, 0.2, 5.])),
        }),
    }));
    world.push(Box::new(Sphere {
        center: ColumnVec([3., 2., -5.]),
        radius: 1.01,
        material: Arc::new(Dielectric {
            color: ColorType::Solid(ColumnVec([1.; 3])),
            ir: 1.3,
        }),
    }));
    world.push(Box::new(Sphere {
        center: ColumnVec([0., 2., -5.]),
        radius: 1.01,
        material: Arc::new(Dielectric {
            color: ColorType::Solid(ColumnVec([1.; 3])),
            ir: 1.3,
        }),
    }));
    world.push(Box::new(Sphere {
        center: ColumnVec([-3., 2., -5.]),
        radius: 1.01,
        material: Arc::new(Dielectric {
            color: ColorType::Solid(ColumnVec([1.; 3])),
            ir: 1.3,
        }),
    }));
    for i in 0..40 {
        world.push(Box::new(Sphere {
            center: ColumnVec([
                rand::thread_rng().gen_range(-11.0..11.),
                rand::thread_rng().gen_range(-1.0..7.5),
                rand::thread_rng().gen_range(-14.0..-2.),
            ]),
            radius: rand::thread_rng().gen_range(0.25..0.75),
            material: Arc::new(Lambertian {
                color: ColorType::Checker(
                    ColumnVec::random_box(0.3, 1.),
                    ColumnVec::random_box(0.3, 1.),
                    rand::thread_rng().gen_range(0.1..0.25),
                ),
            }),
        }));
        world.push(Box::new(Sphere {
            center: ColumnVec([
                rand::thread_rng().gen_range(-11.0..11.),
                rand::thread_rng().gen_range(-1.0..8.),
                rand::thread_rng().gen_range(-14.0..-2.),
            ]),
            radius: rand::thread_rng().gen_range(0.25..0.75),
            material: Arc::new(Metal {
                color: ColorType::Solid(ColumnVec::random_box(0.6, 0.8)),
                fuzz: rand::random::<f64>().powf(3.),
            }),
        }));

        if i % 2 == 0 {
            world.push(Box::new(Sphere {
                center: ColumnVec([
                    rand::thread_rng().gen_range(-11.0..11.),
                    rand::thread_rng().gen_range(-1.0..8.),
                    rand::thread_rng().gen_range(-13.0..-2.),
                ]),
                radius: rand::thread_rng().gen_range(0.25..0.75),
                material: Arc::new(Dielectric {
                    ir: 1.3,
                    color: ColorType::Solid(ColumnVec::random_box(0.9, 1.)),
                }),
            }));
        }
    }

    let camera = Camera::new(3000, 1920, 2., 1., ColumnVec([0., 1., 0.]));

    render(
        world,
        camera,
        512,
        &format!(
            "diffuse_glass_{}.png",
            rand::thread_rng().gen_range(0..1000000)
        ),
    )
}

pub fn render(world: World, camera: Camera, samples_per_pixel: u32, filename: &str) {
    let img: Arc<Mutex<RgbImage>> =
        Arc::new(Mutex::new(ImageBuffer::new(camera.width, camera.height)));
    let count = Arc::new(Mutex::new(0));

    rayon::ThreadPoolBuilder::new()
        .num_threads(6)
        .build_global()
        .unwrap();

    (0..camera.width)
        .collect::<Vec<_>>()
        .par_iter()
        .for_each(|i| {
            for j in 0..camera.height {
                let mut color = ColumnVec([0.; 3]);
                for _ in 0..samples_per_pixel {
                    let u = (*i as f64 + rand::random::<f64>()) / (camera.width - 1) as f64;
                    let v = ((camera.height - j) as f64 + rand::random::<f64>())
                        / (camera.height - 1) as f64;
                    let ray = camera.get_ray(u, v);
                    color = color + ray_color(ray, &world, 16);
                }
                let mut image = img.lock().unwrap();
                image.put_pixel(*i, j, to_color(color / samples_per_pixel as f64));
            }
            *count.lock().unwrap() += 1;
            println!("{}", count.lock().unwrap());
        });

    if img.lock().unwrap().save(filename).is_ok() {
        println!("saved image as {}", filename)
    } else {
        println!("error saving image")
    };
}
