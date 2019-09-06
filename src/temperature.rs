use core::convert::From;
use core::ops::Add;
use core::ops::Sub;

/// temperature in 1/16 Celsius
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct Temperature(i16);

impl Temperature {
    pub fn from_celsius(degrees: i16, degrees_div_16: i16) -> Self {
        Temperature(degrees * 16 + degrees_div_16)
    }

    pub fn is_negative(&self) -> bool {
        self.0 < 0i16
    }

    ///ignores the sign
    pub fn whole_degrees(&self) -> u16 {
        (self.0.abs() >> 4) as u16
    }

    ///ignores the sign
    pub fn fraction_degrees(&self) -> u8 {
        (self.0.abs() & 0xF) as u8
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
