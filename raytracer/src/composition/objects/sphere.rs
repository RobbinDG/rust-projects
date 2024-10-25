use std::f64::consts::PI;
use serde::Deserialize;
use crate::composition::colour::Colour;
use crate::rendering::hit::Hit;
use crate::composition::material::Material;
use crate::composition::objects::object::Object;
use crate::composition::objects::uv_mapped::UVMapped;
use crate::rendering::ray::Ray;
use crate::vector::Vector;

#[derive(Deserialize)]
pub struct Sphere {
    pub c: Vector<f64, 3>,
    pub r: f64,
    pub material: Material,
}

impl Sphere {
    fn normal(&self, at: &Vector<f64, 3>) -> Vector<f64, 3> {
        (at - &self.c).normalise()
    }

    fn material(&self) -> Material {
        self.material.clone()
    }
}

impl Object for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        // || x - c ||^2  = r^2
        let v = &ray.s - &self.c;
        let vd = v.dot(&ray.d);
        let det = vd * vd - &v.dot(&v) + self.r * self.r;

        if det < 0.0 {
            return None;
        }

        let sqrt_det = f64::sqrt(det);
        let t1 = -vd + sqrt_det;
        let t2 = -vd - sqrt_det;
        let t = if t1 <= t2 { t1 } else { t2 };
        let h = ray.at(t);
        let normal = self.normal(&h);

        let n = (&h - &self.c).normalise();
        let uv = (n[0].atan2(-n[2]) / (2.0 * PI) + 0.5, -n[1] * 0.5 + 0.5);
        Some(Hit {
            loc: h,
            t,
            normal,
            material: self.material(),
            back_side: normal.dot(&ray.d) > 0.0,
            uv: Some(uv),
        })
    }
}

impl UVMapped for Sphere {
    fn get_uv_coords(&self, hit: &Hit) -> (f64, f64) {
        let n = (&hit.loc - &self.c).normalise();
        (n[0].atan2(n[1]) / (2.0 * PI) + 0.5, n[1] * 0.5 + 0.5)
    }
}
