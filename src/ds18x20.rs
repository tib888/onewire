//! Read the temperature from DS18B20 1-wire temperature sensors connected to B6 GPIO

#![deny(unsafe_code)]
//#![deny(warnings)]

use OneWire;
use PortErrors;
use calculate_crc;

pub enum DS18x20Devices {
    DS18S20, // or old DS1820
    DS18B20,
    DS1822,
}

pub trait DS18x20 {
    ///starts the temperature measurement, returns the time in milliseconds 
    ///required to wait before reading back the result.
    fn start_temperature_measurement(&mut self, rom: &[u8; 8]) -> Result<u16, PortErrors>;

    ///returns the temperature in 1/16 celsius
    fn read_temperature_measurement_result(&mut self, rom: &[u8; 8]) -> Result<i16, PortErrors>;
}

pub fn detect_18x20_devices(factory_code: u8) -> Option<DS18x20Devices> {
    match factory_code {
        0x10 => Some(DS18x20Devices::DS18S20),  // or old DS1820
        0x28 => Some(DS18x20Devices::DS18B20),
        0x22 => Some(DS18x20Devices::DS1822),
        _ => None,
    }
}

impl<T: OneWire> DS18x20 for T {
    /// Measure the temperature with 18x20, returns temp in celsius * 16
    /// returs the time in ms required for conversion
    fn start_temperature_measurement(&mut self, rom: &[u8; 8]) -> Result<u16, PortErrors> {
        
        if let Err(error) = self.reset() {
            return Err(error);
        }

        self.select(&rom);

        //start conversion
        self.send_byte(0x44);

        //with parasite power ON
        //self.strong_pullup(true);

        Ok(800) //TODO may shorten this if lower bit resolution was chosen.
    }

    fn read_temperature_measurement_result(&mut self, rom: &[u8; 8]) -> Result<i16, PortErrors> {
        //with parasite power OFF
        //self.strong_pullup(false);

        if let Err(err) = self.reset() {
            return Err(err);
        }

        self.select(&rom);

        // Read Scratchpad
        self.send_byte(0xBE);

        let mut scratchpad = [0u8; 9];
        self.request_many(&mut scratchpad[..]);

        if calculate_crc(&scratchpad[..]) == 0 {
            let mut rawtemp: u16 = scratchpad[0] as u16 | ((scratchpad[1] as u16) << 8);

            if rom[0] == 0x10 {
                //DS18S20 or old DS1820
                rawtemp <<= 3; // 9 bit resolution default
                //[7] = count per celsius
                //[6] = remaining count
                //t = temp_read - 0.25 + (count_pre_celsius-remaining_count)/count_pre_celsius
                if scratchpad[7] == 0x10 {
                    // "count remain" gives full 12 bit resolution
                    rawtemp = (rawtemp & 0xFFF0) - 4u16 + 16u16 - (scratchpad[6] as u16);
                }
            } else {
                // at lower res, the low bits are undefined, so let's zero them
                rawtemp &= match scratchpad[4] & 0x60 {
                    0x00 => !7,  // 9 bit resolution, 93.75 ms
                    0x20 => !3,  // 10 bit res, 187.5 ms
                    0x40 => !1,  // 11 bit res, 375 ms
                    _ => !0,     // default is 12 bit resolution, 750 ms conversion time
                }
            }
            
            Ok(rawtemp as i16) //celsius = rawtemp/16.0
        } else {
            Err(PortErrors::CRCMismatch)
        }
    }
}