use crate::colour::Colour;
use crate::object::Object;
use crate::ray::Ray;
use crate::scene::Scene;
use crate::sphere::Sphere;

fn hit_object(ray: &Ray, object: &Box<dyn Object>) -> Option<(f64, Colour)> {
    if let Some((hit, t)) = object.clone().intersect(&ray) {
        let n = object.clone().normal(&hit);
        let cos_theta = -n.dot(&ray.d) / (n.mag() * ray.d.mag());
        if cos_theta >= 0.0 {  // Front side
            Some((t, &object.material() * cos_theta))
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
            if let Some((tc, _)) = closest {
                if tn < tc {
                    closest = Some((tn, cn));
                }
            } else {
                closest = Some((tn, cn));
            }
        }
    }

    match closest {
        Some((_, c)) => image::Rgb([c.r, c.g, c.b]),
        None => image::Rgb([0, 0, 0]),
    }
}