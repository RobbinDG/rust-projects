use crate::colour::Colour;
use crate::hit::Hit;
use crate::light::Light;
use crate::object::Object;
use crate::ray::Ray;
use crate::scene::Scene;

fn ambient_illumination(ray: &Ray, hit: &Hit) -> Option<Colour> {
    let n = hit.normal;
    let cos_theta = (-&n).cos_angle_between(&ray.d);
    if cos_theta >= 0.0 {  // Front side
        Some(&hit.material * cos_theta.sqrt())
    } else { // Back side
        None
    }
}

fn phong_illumination(ray: &Ray, hit: &Hit, lights: &Vec<Box<dyn Light>>) -> Option<Colour> {
    let ks = 0.0;
    let kd = 0.0;
    let ka = 0.0;
    let alpha = 0.0;

    let ia = ambient_illumination(ray, hit)?;

    let mut i = &ia * ka;
    for light in lights {
        let hit_col = &hit.material * &light.colour();
        let l = light.vec(&hit.loc);
        let ln = l.dot(&hit.normal);
        let r = &(&hit.normal * (2.0 * ln)) - &l;

        let i_ln = &hit_col * (kd * ln);
        let v = -&ray.d;
        let i_rv = &Colour::new_rgba([255, 255, 255, 255]) * (ks * r.dot(&v).powf(alpha));
        i = i + i_ln + i_rv;
    }
    return Some(i);
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