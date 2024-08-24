use std::ops::{Add, Div, Index, Mul, Neg, Sub};
// #[serde_as]
#[derive(Debug, Copy, Clone)]
pub struct Vector<T, const N: usize> {
    vals: [T; N],
}

impl<T: Clone, const N: usize> Vector<T, N> {
    pub fn new(vals: [T; N]) -> Vector<T, N> {
        Vector { vals }
    }

    pub fn x(&self) -> T { self.vals[0].clone() }
    pub fn y(&self) -> T { self.vals[1].clone() }
    pub fn z(&self) -> T { self.vals[2].clone() }
}

impl<T: Default + Neg<Output=T> + Copy, const N: usize> Neg for &Vector<T, N> {
    type Output = Vector<T, N>;

    fn neg(self) -> Self::Output {
        let mut vals: [T; N] = [T::default(); N];
        for i in 0..N {
            vals[i] = -self.vals[i];
        }
        return Vector::new(vals);
    }
}

impl<T: Default + Add<T, Output=T> + Mul<T, Output=T> + Copy, const N: usize> Vector<T, N> {
    pub fn dot(&self, rhs: &Vector<T, N>) -> T {
        let mut val = T::default();
        for i in 0..N {
            val = val + self.vals[i] * rhs.vals[i];
        }
        return val;
    }
}

impl<T: Copy, const N: usize> Index<usize> for Vector<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if 0 > index || index >= N { panic!("Index out of bounds.") }
        return &self.vals[index];
    }
}

trait HasSqrt {
    fn sqrt(&self) -> Self;
}

impl HasSqrt for f32 {
    fn sqrt(&self) -> Self {
        (*self).sqrt()
    }
}

impl HasSqrt for f64 {
    fn sqrt(&self) -> Self {
        (*self).sqrt()
    }
}

impl<T: HasSqrt + Default + Add<T, Output=T> + Mul<T, Output=T> + Div<T, Output=T> + Copy, const N: usize> Vector<T, N> {
    pub fn mag(&self) -> T {
        self.dot(self).sqrt()
    }

    pub fn normalise(&self) -> Vector<T, N> {
        let m = self.mag();
        self.div(m)
    }

    pub fn cos_angle_between(&self, other: &Vector<T, N>) -> T {
        self.dot(other) / (self.mag() * other.mag())
    }
}

impl<const N: usize> Vector<f64, N> {
    pub fn signed_angle_to(&self, other: &Vector<f64, N>) -> f64 {
        if N != 2 {
            panic!("Can only be computed for N == 2")
        }
        let cross = self.x() * other.y() - self.y() * other.x();
        // return cross.signum() * self.cos_angle_between(other).acos();
        let dot = self.dot(other);
        return cross.atan2(dot);
    }
}

impl<T: Default + Mul<S, Output=T> + Copy, S: Copy, const N: usize> Mul<S> for &Vector<T, N> {
    type Output = Vector<T, N>;

    fn mul(self, rhs: S) -> Self::Output {
        let mut vals: [T; N] = [T::default(); N];
        for i in 0..N {
            vals[i] = self.vals[i] * rhs;
        }

        Vector { vals }
    }
}

impl<T: Default + Div<S, Output=T> + Copy, S: Copy, const N: usize> Div<S> for &Vector<T, N> {
    type Output = Vector<T, N>;

    fn div(self, rhs: S) -> Self::Output {
        let mut vals: [T; N] = [T::default(); N];
        for i in 0..N {
            vals[i] = self.vals[i] / rhs;
        }

        Vector { vals }
    }
}

impl<T: Mul<f64, Output=T> + Default + Copy, const N: usize> Mul<&Vector<T, N>> for f64 {
    type Output = Vector<T, N>;

    fn mul(self, rhs: &Vector<T, N>) -> Self::Output {
        rhs * self
    }
}

impl<T: Add<T, Output=T> + Copy + Default, const N: usize> Add<&Vector<T, N>> for &Vector<T, N> {
    type Output = Vector<T, N>;

    fn add(self, rhs: &Vector<T, N>) -> Self::Output {
        let mut vals: [T; N] = [T::default(); N];
        for i in 0..N {
            vals[i] = self.vals[i] + rhs.vals[i];
        }

        Vector { vals }
    }
}

impl<T: Add<T, Output=T> + Copy + Default, const N: usize> Add<&Vector<T, N>> for Vector<T, N> {
    type Output = Vector<T, N>;

    fn add(self, rhs: &Vector<T, N>) -> Self::Output {
        &self + rhs
    }
}

impl<T: Add<T, Output=T> + Copy + Default, const N: usize> Add<Vector<T, N>> for Vector<T, N> {
    type Output = Vector<T, N>;

    fn add(self, rhs: Vector<T, N>) -> Self::Output {
        &self + &rhs
    }
}

impl<T: Sub<T, Output=T> + Default + Copy, const N: usize> Sub<&Vector<T, N>> for &Vector<T, N> {
    type Output = Vector<T, N>;

    fn sub(self, rhs: &Vector<T, N>) -> Self::Output {
        let mut vals: [T; N] = [T::default(); N];
        for i in 0..N {
            vals[i] = self.vals[i] - rhs.vals[i];
        }

        Vector { vals }
    }
}
