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

fn phong_illumination(ray: &Ray, hit: &Hit, light: &Box<dyn Light>, in_shadow: bool) -> Colour {
    let ka = hit.material.ka;
    let kd = hit.material.kd;
    let ks = hit.material.ks;
    let alpha = hit.material.alpha;

    let c = &hit.material.colour * ka;
    let v = -&ray.d;
    let l = light.vec(&hit.loc);
    let ln = l.dot(&hit.normal);
    let r = (&(&hit.normal * (2.0 * ln)) - &l).normalise();

    let light_col = &light.colour() * in_shadow as i32 as f64;
    let c_hit = &hit.material.colour * &light_col;
    let c_diffuse = &c_hit * (kd * ln);
    let spec = r.dot(&v).max(0.0);
    let c_specular = &light_col * (ks * spec.powf(alpha));
    c + c_diffuse + c_specular
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

fn reflect_ray(ray: &Ray, hit: &Hit) -> Ray {
    let dir = (&ray.d - &(&hit.normal * (2.0 * ray.d.dot(&hit.normal)))).normalise();
    Ray {
        s: hit.loc + &dir * 0.0001,
        d: dir,
        in_material: hit.material.clone(),
    }
}

fn refract_ray(ray: &Ray, hit: &Hit) -> Option<Ray> {
    let n1 = ray.in_material.refractive_index.unwrap_or(1.0);
    let n2 = hit.material.refractive_index.unwrap_or(1.0);
    let n12 = n1 / n2;
    let cos_theta_1 = -ray.d.dot(&hit.normal);
    let cos_theta_2 = (1.0 - n12 * n12 * (1.0 - cos_theta_1 * cos_theta_1)).sqrt();
    let d_rfr = (&ray.d * n12 + &hit.normal * (n12 * cos_theta_1 - cos_theta_2)).normalise();

    let asin_n21 = (n2 / n1).asin();
    let cos_theta_crit = (1.0 - asin_n21 * asin_n21).sqrt();
    if cos_theta_1 >= cos_theta_crit {
        return None;
    }
    // println!("{:?} {:?} {} {} {} {}", ray.d, d_rfr, n12, cos_theta_1, cos_theta_2, (cos_theta_1.acos().sin() * n1 / n2).asin().cos());
    Some(Ray {
        s: hit.loc + &d_rfr * 0.00001,
        d: d_rfr,
        in_material: hit.material.clone(),
    })
}

pub fn trace(ray: Ray, scene: &Scene) -> image::Rgb<u8> {
    let c = trace_reflect(ray, scene, 1);
    image::Rgb([c.r(), c.g(), c.b()])
}

pub fn illuminate(ray: &Ray, hit: &Hit, scene: &Scene) -> Colour {
    let mut c = Colour::black();
    for light in &scene.lights {
        let shadow_d = light.vec(&hit.loc);
        let shadow_ray = Ray::new(hit.loc + &shadow_d * 0.001, shadow_d);
        let shad = closest_object(&shadow_ray, scene);
        c = c + phong_illumination(&ray, &hit, &light, shad.is_none());
    }
    c
}

pub fn trace_reflect(ray: Ray, scene: &Scene, depth: u8) -> Colour {
    // Determine the closest intersecting object
    let mut closest = closest_object(&ray, scene);

    match closest {
        Some(mut hit) => {
            if depth > 0 {
                let mut leftover = 1.0;
                let mut colour = Colour::black();
                let hit_colour = hit.colour_at();
                if let Some(reflectivity) = hit.material.reflectivity {
                    if !hit.back_side {
                        let reflection = reflect_ray(&ray, &hit);
                        let c = trace_reflect(reflection, scene, depth - 1);
                        colour = colour + &(&hit_colour * &c) * reflectivity;
                        leftover -= reflectivity;
                    }
                } else {
                    colour = colour + hit_colour.clone();
                }
                if let Some(transmittance) = hit.material.transmittance {
                    if let Some(refraction) = refract_ray(&ray, &hit) {
                        let c = trace_reflect(refraction, scene, depth - hit.back_side as u8);
                        colour = colour + &(&hit_colour * &c) * transmittance;
                        leftover -= transmittance;
                    }
                }
                hit.material.colour = colour + &hit_colour * leftover;
            }
            illuminate(&ray, &hit, &scene)
        }
        None => scene.background.clone(),
    }
}