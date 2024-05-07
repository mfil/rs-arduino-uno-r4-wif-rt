//! Blink the LED slowly if D10 is connected to ground and quickly otherwise.

#![no_main]
#![no_std]
extern crate arduino_uno_r4_wifi_rt;

use arduino_uno_r4_wifi_rt::peripherals::pins::{get_pins, InputPin, OutputPin, Pin};
use arduino_uno_r4_wifi_rt::peripherals::systick;

arduino_uno_r4_wifi_rt::entry!(main);

fn main() -> ! {
    let mut systick = match systick::SysTick::instance() {
        Some(systick) => systick,
        None => loop {},
    };

    let pins = match get_pins() {
        Some(pins) => pins,
        None => loop {},
    };

    let mut led_builtin = pins.d13.into_output();
    let d10 = pins.d10.into_input_pullup();

    systick.enable();
    systick.set_reset_value(systick.get_ticks_per_10ms() * 10);
    systick.reset();

    loop {
        if systick.timer_wrapped() {
            led_builtin.toggle();
        }
        if d10.is_low() {
            systick.set_reset_value(systick.get_ticks_per_10ms() * 100);
        } else {
            systick.set_reset_value(systick.get_ticks_per_10ms() * 10);
        }
    }
}
