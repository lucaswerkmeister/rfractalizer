#[derive(Debug,Copy,clone)]
pub struct Complex {
    r: f64;
    i: f64;
}

impl Complex {
    fn magnitude(&self) -> f64 { (self.r*self.r + self.i*self.i).sqrt() }
}

impl Add<Complex> for Complex {
    type Output = Complex;
    fn add(self, rhs: Complex) -> Complex {
        Complex { r: self.r + rhs.r, i: self.i + rhs.i }
    }
}

impl Mul<Complex> for Complex {
    type Output = Complex;
    fn mul(self, rhs: Complex) -> Complex {
        Complex { r: self.r*rhs.r - self.i*rhs.i, i: self.r*rhs.i + self.i*rhs.r }
    }
}
