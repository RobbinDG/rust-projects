use crate::colour::Colour;
use crate::ray::Ray;
use crate::vector::Vector;

pub struct Sphere {
    pub c: Vector<f64, 3>,
    pub r: f64,
    pub colour: Colour,
}

impl Sphere {
    pub fn intersect(&self, ray: &Ray) -> Option<(Vector<f64, 3>, f64)> {
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
        return Some((ray.at(t), t));
    }

    pub fn normal(&self, at: &Vector<f64, 3>) -> Vector<f64, 3> {
        return (at - &self.c).normalise();
    }
}