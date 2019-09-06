//!this helper can turn two 'one directional' pins into one 'bidirectional' pin which is needed for OneWirePort implementation

#![deny(unsafe_code)]
#![deny(warnings)]

use embedded_hal::digital::v2::{InputPin, OutputPin};

pub struct IOPin<OPIN, IPIN>
where
    OPIN: OutputPin,//in opendrain mode
    IPIN: InputPin,//in floating input mode
{
    output: OPIN,  
    input: IPIN,
}

impl<OPIN, IPIN> InputPin for IOPin<OPIN, IPIN>
where
    IPIN: InputPin,
    OPIN: OutputPin,
{
    type Error = IPIN::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.input.is_high()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.input.is_low()
    }
}

impl<OPIN, IPIN> OutputPin for IOPin<OPIN, IPIN>
where
    IPIN: InputPin,
    OPIN: OutputPin,
{
    type Error = OPIN::Error;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.output.set_high()
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.output.set_low()
    }
}
