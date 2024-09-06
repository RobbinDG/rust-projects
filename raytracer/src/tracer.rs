use crate::composition::{Colour, Light, Object, Scene};
use crate::hit::Hit;
use crate::ray::Ray;

fn simple_illumination(ray: &Ray, hit: &Hit) -> Option<Colour> {
    let n = hit.normal;
    let cos_theta = (-&n).cos_angle_between(&ray.d);
    if cos_theta >= 0.0 {  // Front side
        Some(&hit.material.colour * cos_theta.sqrt())
    } else { // Back side
        None
    }
}

fn phong_illumination(ray: &Ray, hit: &Hit, lights: &Vec<Box<dyn Light>>) -> Option<Colour> {
    let ka = hit.material.ka;
    let kd = hit.material.kd;
    let ks = hit.material.ks;
    let alpha = hit.material.alpha;

    let mut c = &hit.material.colour * ka;
    for light in lights {
        let v = -&ray.d;
        let l = light.vec(&hit.loc);
        let ln = l.dot(&hit.normal);
        let r = (&(&hit.normal * (2.0 * ln)) - &l).normalise();

        let c_hit = &hit.material.colour * &light.colour();
        let c_diffuse = &c_hit * (kd * ln);
        let spec = r.dot(&v).max(0.0);
        let c_specular = &Colour::new_rgba([255, 255, 255, 255]) * (ks * spec.powf(alpha));
        c = c + c_diffuse + c_specular;
    }
    Some(c)
}

fn hit_object(ray: &Ray, object: &Box<dyn Object>, lights: &Vec<Box<dyn Light>>) -> Option<(f64, Colour)> {
    if let Some(hit) = object.clone().intersect(&ray) {
        let i = phong_illumination(&ray, &hit, lights)?;
        Some((hit.t, i))
    } else {
        None
    }
}
pub fn trace(ray: Ray, scene: &Scene) -> image::Rgb<u8> {
    let mut closest: Option<(f64, Colour)> = None;
    for object in &scene.objects {
        let hit = hit_object(&ray, object, &scene.lights);
        if let Some((tn, cn)) = hit {
            closest = match closest {
                Some((tc, _)) if tc < tn => { Some((tn, cn)) }
                None => { Some((tn, cn)) }
                _ => { closest }
            };
        }
    }

    match closest {
        Some((_, c)) => image::Rgb([c.r(), c.g(), c.b()]),
        None => image::Rgb([0, 0, 0]),
    }
}