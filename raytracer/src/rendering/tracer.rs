use crate::composition::{Colour, Light, Object, Scene};
use crate::rendering::hit::Hit;
use crate::rendering::ray::Ray;

fn simple_illumination(ray: &Ray, hit: &Hit) -> Option<Colour> {
    let n = hit.normal;
    let cos_theta = (-&n).cos_angle_between(&ray.d);
    if cos_theta >= 0.0 {  // Front side
        Some(&hit.material.colour * cos_theta.sqrt())
    } else { // Back side
        None
    }
}

fn phong_illumination(ray: &Ray, hit: &Hit, lights: &Vec<Box<dyn Light>>) -> Colour {
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
    c
}

fn closest_object(ray: &Ray, scene: &Scene) -> Option<Hit> {
    let mut closest: Option<Hit> = None;

    for object in &scene.objects {
        // let hit = hit_object(&ray, object, &scene.lights);
        if let Some(hit) = object.intersect(&ray) {
            closest = match closest {
                Some(c) if hit.t >= 0.0 && hit.t < c.t => Some(hit),
                None if hit.t >= 0.0 => Some(hit),
                _ => closest,
            }
        }
    }
    closest
}

fn reflect_ray(ray: &Ray, hit: &Hit, hit_colour: Colour) -> Ray {
    Ray {
        s: hit.loc,
        d: (&ray.d - &(&hit.normal * (2.0 * ray.d.dot(&hit.normal)))).normalise(),
        c: hit_colour,
    }
}

pub fn trace(ray: Ray, scene: &Scene) -> image::Rgb<u8> {
    let c = trace_reflect(ray, scene, 1);
    image::Rgb([c.r(), c.g(), c.b()])
}

pub fn trace_reflect(ray: Ray, scene: &Scene, depth: u8) -> Colour {
    // Determine the closest intersecting object
    let closest = closest_object(&ray, scene);

    match closest {
        Some(hit) => {
            match hit.material.ref_coef {
                Some(coef) if depth > 0 => {
                    let reflection = reflect_ray(&ray, &hit, ray.c.clone());
                    trace_reflect(reflection, scene, depth - 1)
                }
                _ => phong_illumination(&ray, &hit, &scene.lights) // Not reflective material
            }
        }
        None => scene.background.clone(),
    }
}