use sphere::Sphere;

use crate::colour::Colour;
use crate::cube::Cube;
use crate::scene::{Camera, Scene};
use crate::tracer::trace;
use crate::vector::Vector;

mod scene;
mod ray;
mod tracer;
mod vector;
mod sphere;
mod colour;
mod plane;
mod object;
mod cube;
mod hit;

fn main() {
    let imgx = 100;
    let imgy = 100;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    let scene_spheres = Scene {
        camera: Camera {
            eye_pos: Vector::new([0.0, 0.0, 0.0]),
            dir: Vector::new([-1.0, 0.0, 1.0]),
            fov: 2.0,
            width: imgy,
            height: imgx,
        },
        objects: vec![
            Box::new(Sphere {
                c: Vector::new([0.5, 0.5, 3.5]),
                r: 2.0,
                colour: Colour {r: 255, g: 0, b: 0, a: 255},
            }),
            Box::new(Sphere {
                c: Vector::new([0.0, -0.5, 2.5]),
                r: 1.0,
                colour: Colour {r: 0, g: 0, b: 255, a: 255},
            }),
        ],
    };

    let scene_box = Scene {
        camera: Camera {
            eye_pos: Vector::new([-1.5, 0.0, 0.0]),
            dir: Vector::new([0.2, 0.0, 1.0]),
            fov: 2.0,
            width: imgy,
            height: imgx,
        },
        objects: vec![
            Box::new(Cube {
                c: Vector::new([0.0, 0.0, 3.5]),
                d: 1.0,
                colour: Colour {r: 255, g: 0, b: 0, a: 255},
            }),
        ],
    };

    let scene = scene_box;

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let ray = scene.camera.ray_for_pixel(x, y);
        *pixel = trace(ray, &scene)
    }

    // Save the image as “fractal.png”, the format is deduced from the path
    imgbuf.save("result.png").unwrap();
}