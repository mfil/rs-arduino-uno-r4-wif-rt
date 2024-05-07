# arduino_uno_r4_wifi_rt

## About
This is a toy project to build a Rust runtime for the Arduino Uno R4 Wifi "from scratch", using only
the core library. It currently exposes the GPIO pins and the Systick timer feature.

I don't have plans to expand this to a comprehensive library, and it is not intended for production
use. Nevertheless, if anything here seems useful, feel free to take it under the MIT license. No
warranties though!

## Building
Use `rustup target add thumbv7em-none-eabi` to install the compiler for the AMD Cortex M4 CPU.
The .cargo/config file in this repository is set up to compile for this architecture by default,
so you can just use `cargo build`.

## Using
Since this isn't a serious library, I'm not putting it on crates.io. If you clone this repository,
you can use the crate by adding the following to your Cargo.toml:
```
[dependencies]
arduino_uno_wifi_rt = { /path/to/arduino_uno_r4_wifi_rt }
```

Copy the .cargo/config from this crate to yours. This is to ensure that the linker script from this
crate is used.

In your application, write
```
#[no_std]
#[no_main]

extern crate arduino_uno_wifi_rt;

use arduino_uno_wifi_rt::entry;

entry!(main);

fn main() -> ! {
    // Your code here.
    ...
}
```

Check the doc comments and examples for how to access the GPIO pins.

To get your code onto your Arduino, you first need to make a raw binary from the ELF file and then
upload it using `arduino-cli`:
```
cargo objcopy -- -O binary test.bin
arduino-cli upload --input-file test.bin -p /dev/ttyACM0
```

## Further reading
The following resources were very helpful with this project. If you're interested in embedded Rust
for this platform or in general, I recommend checking them out.

General embedded Rust on Arm:
* [Embedonomicon](https://docs.rust-embedded.org/embedonomicon/)
* [The Embedded Rust Book](https://docs.rust-embedded.org/book/)

Documentation for the Armv7-M architecture:
* [Armv7-M Architecture Reference Manual](https://developer.arm.com/documentation/ddi0403/ee)

Documentation for the Renesas RA4M1 microcontroller:
* [Renesas RA4M1 Group User's Manual: Hardware](https://cdn.sparkfun.com/assets/b/1/d/3/6/RA4M1_Datasheet.pdf)
