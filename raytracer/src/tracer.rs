use crate::colour::Colour;
use crate::object::Object;
use crate::ray::Ray;
use crate::scene::Scene;

fn hit_object(ray: &Ray, object: &Box<dyn Object>) -> Option<(f64, Colour)> {
    if let Some(hit) = object.clone().intersect(&ray) {
        let n = hit.normal;
        let cos_theta = (-&n).cos_angle_between(&ray.d);
        println!("{} {:?} {:?} {:?}", cos_theta, n, ray.d, hit.material.r);
        if cos_theta >= 0.0 {  // Front side
            Some((hit.t, &hit.material * cos_theta.sqrt()))
        } else { // Back side
            None
        }
    } else {
        None
    }
}
pub fn trace(ray: Ray, scene: &Scene) -> image::Rgb<u8> {
    let mut closest: Option<(f64, Colour)> = None;
    for object in &scene.objects {
        let hit = hit_object(&ray, object);
        if let Some((tn, cn)) = hit {
            closest = match closest {
                Some((tc, _)) if tc < tn => { Some((tn, cn)) }
                None => { Some((tn, cn)) }
                _ => { closest }
            };
        }
    }
    println!("{:?}", closest);

    match closest {
        Some((_, c)) => image::Rgb([c.r, c.g, c.b]),
        None => image::Rgb([0, 0, 0]),
    }
}