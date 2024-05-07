//! LED lights up if the pin struct has 0 size.

#![no_main]
#![no_std]
extern crate arduino_uno_r4_wifi_rt;

use arduino_uno_r4_wifi_rt::peripherals::pins::{get_pins, ArduinoPins, OutputPin, Pin};
use core::mem::size_of;

arduino_uno_r4_wifi_rt::entry!(main);

fn main() -> ! {
    let pins = get_pins();
    let mut led_builtin = pins.d13.into_output();
    if size_of::<ArduinoPins>() == 0 {
        led_builtin.set_high();
    }

    loop {}
}
