use std::fs::File;
use std::path::Path;

use crate::scene::Scene;
use crate::tracer::trace;

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
mod scene_loader;
mod light;

fn main() {
    let scene = load_scene("scene.json");

    let imgx = 100;
    let imgy = 100;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let ray = scene.camera.ray_for_pixel(x, y);
        *pixel = trace(ray, &scene);
    }

    // Save the image as “fractal.png”, the format is deduced from the path
    imgbuf.save("result.png").unwrap();
}

fn load_scene(file_name: &str) -> Scene {
    let file = File::open(&Path::new("scene.json")).unwrap();
    let scene_loaded: Scene = serde_json::from_reader(file).unwrap();
    scene_loaded
}