//!this helper can turn two 'one directional' pins into one 'bidirectional' pin which is needed for OneWirePort implementation

#![deny(unsafe_code)]
#![deny(warnings)]

use hal::digital::{InputPin, OutputPin};

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
    OPIN: OutputPin,
    IPIN: InputPin,
{
    fn is_high(&self) -> bool {
        self.input.is_high()
    }

    fn is_low(&self) -> bool {
        self.input.is_low()
    }
}

impl<OPIN, IPIN> OutputPin for IOPin<OPIN, IPIN>
where
    OPIN: OutputPin,
    IPIN: InputPin,
{
    fn set_high(&mut self) {
        self.output.set_high();
    }

    fn set_low(&mut self) {
        self.output.set_low();
    }
}
