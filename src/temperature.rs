use core::convert::From;
use core::ops::Add;
use core::ops::Sub;

/// temperature in 1/16 Celsius
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct Temperature(i16);

impl Temperature {
    pub fn from_celsius(degree: i16, degree_div_16: i16) -> Self {
        Temperature((degree << 4) | degree_div_16)
    }
}

impl Add for Temperature {
    type Output = Temperature;
    fn add(self, rhs: Self) -> Self::Output {
        Temperature(self.0 + rhs.0)
    }
}

impl Sub for Temperature {
    type Output = Temperature;
    fn sub(self, rhs: Self) -> Self::Output {
        Temperature(self.0 - rhs.0)
    }
}

impl From<i16> for Temperature {
    fn from(original: i16) -> Temperature {
        Temperature(original)
    }
}
