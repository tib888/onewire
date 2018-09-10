#![no_std]

extern crate embedded_hal as hal;
//extern crate cortex_m;

pub mod ds18x20;
pub mod iopin;
pub mod temperature;

use hal::blocking::delay::DelayUs;
use hal::digital::{InputPin, OutputPin};
//use cortex_m::interrupt::{self};

///generic 1-wire API:

#[derive(Debug)]
pub enum PortErrors {
    ShortDetected,         //line stays low - probably pullup resistor missing
    NoPresencePulseDetect, //no devices detected at reset
    NoDevices,             //rom enumeration does not see any devices
    CRCMismatch,           //crc check fails
}

pub type Rom = [u8; 8];

pub struct RomIterator {
    last_device_flag: bool,
    last_discrepancy: i32,
    last_family_discrepancy: i32,
    rom: Rom,
}

impl RomIterator {
    /// Setup the search to find the device type 'family_code' on the next call
    /// to search(...) if it is present.
    /// family code = 0 will work on all devices
    pub fn new(family_code: u8) -> RomIterator {
        // set the search state to find SearchFamily type devices
        RomIterator {
            last_discrepancy: 0,
            last_device_flag: false,
            last_family_discrepancy: 0,
            rom: [family_code, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
        }
    }

    //reset counters so next 'search' will be like a first
    pub fn reset(&mut self, family_code: u8) {
        self.last_discrepancy = 0;
        self.last_device_flag = false;
        self.last_family_discrepancy = 0;
        self.rom[0] = family_code;
    }
}

// Compute a Dallas Semiconductor 8 bit CRC directly.
// this is much slower, but much smaller, than the lookup table.
pub fn calculate_crc(buffer: &[u8]) -> u8 {
    let mut crc = 0;
    for b in buffer.iter() {
        let mut d = *b as u32;
        for _ in 0..8 {
            if (crc ^ d) & 1 != 0 {
                crc >>= 1;
                crc ^= 0x8C;
            } else {
                crc >>= 1;
            }
            d >>= 1;
        }
    }
    (crc as u8)
}

pub trait OneWire {
    fn reset(&mut self) -> Result<(), PortErrors>;
    fn send_byte(&mut self, data: u8);
    fn request_byte(&mut self) -> u8;

    fn send_many(&mut self, data: &[u8]) {
        for d in data {
            self.send_byte(*d);
        }
    }

    /// fills the buffer with bytes coming on the 1 wire bus
    fn request_many(&mut self, buffer: &mut [u8]) {
        for d in buffer.iter_mut() {
            *d = self.request_byte();
        }
    }

    /// Do a ROM select
    fn select(&mut self, rom: &Rom) {
        // Choose ROM
        self.send_byte(0x55);
        self.send_many(&rom[..]);
    }

    /// Do a ROM skip
    fn skip(&mut self) {
        // Skip ROM
        self.send_byte(0xCC);
    }

    /// Perform the 1-Wire search algorithm on the 1-Wire bus using the iterator state
    /// normal_search_mode : true = normal search, false = alarm/conditional
    fn iterate_next<'a>(
        &mut self,
        normal_search_mode: bool,
        it: &'a mut RomIterator,
    ) -> Result<Option<&'a Rom>, PortErrors>;
}

pub struct OneWirePort<IOPIN, DELAY>
where
    IOPIN: InputPin + OutputPin, //in opendrain mode the pin also acts as input
    DELAY: DelayUs<u16>,
{
    /// an external 4.7k pullup resistor is needed on this pin
    io: IOPIN,
    delay: DELAY, //TODO add support for a strong pullup pin
}

/// Executes the closure `f` in a preemption free context
/// During the execution of the closure no task can preempt the current task.
/// use this only where the timing is critical, otherwise shall not block.
fn atomic<R, F>(f: F) -> R
where
    F: FnOnce() -> R,
{
    //unsafe { interrupt::disable(); };//TODO
    let r = f();
    //unsafe { interrupt::enable(); };//TODO
    r
}

const DELAY_CALIBRATION: u16 = 3u16;

impl<IOPIN, DELAY> OneWirePort<IOPIN, DELAY>
where
    IOPIN: InputPin + OutputPin,
    DELAY: DelayUs<u16>,
{
    pub fn new(mut io: IOPIN, delay: DELAY) -> Self {
        // initial output state: hi
        io.set_high();
        OneWirePort {
            io: io,
            delay: delay,
        }
    }

    fn send_bit(&mut self, data: bool) {
        //the slave will sample the line 15..60us from the initial falling edge
        //TODO instead of delay_ticks read the counter first then wait until relative positions
        if data {
            //short low pulse = 1
            atomic(|| {
                self.io.set_low();
                self.delay.delay_us(9u16 - DELAY_CALIBRATION);
                self.io.set_high();
            });
            self.delay.delay_us(80u16 - 9u16 - DELAY_CALIBRATION);
        } else {
            //long low pulse = 0
            atomic(|| {
                self.io.set_low();
                self.delay.delay_us(60u16 - DELAY_CALIBRATION);
                self.io.set_high();
            });
            self.delay.delay_us(80u16 - 60u16 - DELAY_CALIBRATION);
        }
    }

    fn request_bit(&mut self) -> bool {
        //the master will sample the line 15us from the initial falling edge
        //TODO instead of delay_ticks read the counter first then wait until relative positions
        {
            let result = atomic(|| {
                //send out a short low pulse
                self.io.set_low();
                self.delay.delay_us(9u16 - DELAY_CALIBRATION);
                self.io.set_high();
                self.delay.delay_us(9u16 - DELAY_CALIBRATION); //6?

                //then sample the port if a slave keeps it pulled down at 15us
                self.io.is_high()
            });

            self.delay.delay_us(80u16 - 9u16 - 9u16 - DELAY_CALIBRATION);
            result
        }
    }
}

impl<IOPIN, DELAY> OneWire for OneWirePort<IOPIN, DELAY>
where
    IOPIN: InputPin + OutputPin,
    DELAY: DelayUs<u16>,
{
    fn reset(&mut self) -> Result<(), PortErrors> {
        //test few times if the wire is high... just in case
        let mut retry = 128;
        loop {
            retry -= 1;
            if retry == 0 {
                return Err(PortErrors::ShortDetected);
            }
            if self.io.is_high() {
                break;
            }
            self.delay.delay_us(1u16);
        }

        //TODO instead of delay_ticks read the counter first then wait until relative positions
        let device_present = {
            //long (480..640us) low reset pulse:
            self.io.set_low();
            self.delay.delay_us(480u16 - DELAY_CALIBRATION);
            atomic(|| {
                self.io.set_high();
                //wait 15..60us - external pullup brings the line high
                //then sample the line if any device pulls it down to show its presence
                self.delay.delay_us(72u16 - DELAY_CALIBRATION);
                self.io.is_low()
            })
        };

        //then wait for recovery at least 100..180us
        self.delay.delay_us(320u16 - DELAY_CALIBRATION);
        //frame ends here

        if device_present {
            Ok(())
        } else {
            Err(PortErrors::NoPresencePulseDetect)
        }
    }

    fn send_byte(&mut self, data: u8) {
        for i in 0..8 {
            self.send_bit(data & (1 << i) != 0);
        }
    }

    fn request_byte(&mut self) -> u8 {
        let mut result: u8 = 0u8;
        for i in 0..8 {
            if self.request_bit() {
                result |= 1 << i;
            }
        }
        result
    }

    fn iterate_next<'a>(
        &mut self,
        normal_search_mode: bool,
        it: &'a mut RomIterator,
    ) -> Result<Option<&'a Rom>, PortErrors> {
        if let Err(error) = self.reset() {
            return Err(error);
        } else {
            if !it.last_device_flag {
                let mut id_bit_number = 1;
                let mut id_byte_number = 0;
                let mut id_byte_mask = 1u8;
                let mut last_zero = 0;

                // issue the search command
                if normal_search_mode {
                    self.send_byte(0xF0); // NORMAL SEARCH
                } else {
                    self.send_byte(0xEC); // CONDITIONAL SEARCH
                }

                loop {
                    // read a bit and its complement
                    let id_bit = self.request_bit();
                    let cmp_id_bit = self.request_bit();

                    // check for no devices on 1-wire
                    if id_bit && cmp_id_bit {
                        return Err(PortErrors::NoDevices);
                    }

                    let search_direction = if id_bit || cmp_id_bit {
                        // id_bit != cmp_id_bit means
                        // all devices coupled have 0 or 1
                        id_bit
                    } else {
                        // if this discrepancy if before the Last Discrepancy
                        // on a previous next then pick the same as last time
                        let dir = if id_bit_number < it.last_discrepancy {
                            (it.rom[id_byte_number] & id_byte_mask) != 0
                        } else {
                            id_bit_number == it.last_discrepancy
                        };

                        // if 0 was picked then record its position in LastZero
                        if !dir {
                            last_zero = id_bit_number;

                            // check for Last discrepancy in family
                            if last_zero < 9 {
                                it.last_family_discrepancy = last_zero;
                            }
                        }

                        dir
                    };

                    // set or clear the bit in the ROM byte rom_byte_number
                    // with mask rom_byte_mask
                    if search_direction {
                        it.rom[id_byte_number] |= id_byte_mask;
                    } else {
                        it.rom[id_byte_number] &= !id_byte_mask;
                    }
                    self.send_bit(search_direction);

                    id_bit_number += 1;
                    id_byte_mask <<= 1;
                    if id_byte_mask == 0 {
                        id_byte_number += 1;
                        id_byte_mask = 1;
                    }

                    // loop until through all ROM bytes 0-7
                    if id_byte_number >= 8 {
                        it.last_discrepancy = last_zero;

                        if it.last_discrepancy == 0 {
                            it.last_device_flag = true;
                        }

                        if 0 != calculate_crc(&it.rom[..]) {
                            return Err(PortErrors::CRCMismatch);
                        }

                        // if the search was successful then
                        return Ok(Some(&it.rom));
                    }
                }
            }
        }

        // if no (or no more) device found
        Ok(None)
    }
}
