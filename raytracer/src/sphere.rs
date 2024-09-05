use serde::Deserialize;
use crate::colour::Colour;
use crate::hit::Hit;
use crate::object::Object;
use crate::ray::Ray;
use crate::vector::Vector;

#[derive(Deserialize)]
pub struct Sphere {
    pub c: Vector<f64, 3>,
    pub r: f64,
    pub colour: Colour,
}

impl Sphere {
    fn normal(&self, at: &Vector<f64, 3>) -> Vector<f64, 3> {
        (at - &self.c).normalise()
    }

    fn material(&self) -> Colour {
        self.colour.clone()
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
        Some(Hit {
            loc: h,
            t,
            normal: self.normal(&h),
            material: self.material(),
        })
    }
}
