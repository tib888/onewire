use core::convert::From;
use core::ops::Add;
use core::ops::Sub;

/// temperature in 1/16 Celsius
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct Temperature(i16);

impl Temperature {
    pub fn from_celsius(degrees: i16, degrees_div_16: i16) -> Self {
        Temperature((degrees << 4) | degrees_div_16)
    }

    pub fn whole_degrees(&self) -> i16 {
        self.0 >> 4
    }

    pub fn fraction_degrees(&self) -> i16 {
        if self.0 >= 0 {
            self.0 & 0xF
        } else {
            (-self.0) & 0xF
        }
    }

    //format a temperature given in 1/16 celsius to a string -02.3
    pub fn format(&self) -> [u8; 5] {
        let mut result = [20u8; 5];
        result[0] = if self.0 < 0 { '-' as u8 } else { ' ' as u8 };
        let temp: u8 = self.0.abs() as u8;
        result[1] = '0' as u8 + (temp / 160);
        result[2] = '0' as u8 + (temp >> 4);
        result[3] = '.' as u8;
        //round fraction to one digit:
        // 0	0.000
        // 1	0.063
        // 2	0.125
        // 3	0.188
        // 4	0.250
        // 5	0.313
        // 6	0.375
        // 7	0.438
        // 8	0.500
        // 9	0.563
        // 10	0.625
        // 11	0.688
        // 12	0.750
        // 13	0.813
        // 14	0.875
        // 15	0.938
        let table: &[u8] = b"0112334456678899";
        result[4] = table[(temp & 0xf) as usize];
        result
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
