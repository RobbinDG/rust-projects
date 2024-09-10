use std::fs::File;
use std::path::Path;

use composition::Scene;
use rendering::tracer::trace;
use crate::rendering::viewport::Viewport;

mod vector;
mod composition;
mod rendering;

fn main() {
    let scene = load_scene("scene.json");

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(scene.camera.width, scene.camera.height);
    let viewport = Viewport::new(&scene.camera);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let ray = viewport.ray_for_pixel(x as usize, y as usize);
        *pixel = trace(ray, &scene);
    }

    // Save the image as “fractal.png”, the format is deduced from the path
    imgbuf.save("result.png").unwrap();
}

fn load_scene(file_name: &str) -> Scene {
    let file = File::open(&Path::new(file_name)).unwrap();
    let scene_loaded: Scene = serde_json::from_reader(file).unwrap();
    scene_loaded
}