//! Control the GPIO pins.
//!
//! Call the [`get_pins`] function to get a struct with the pins labelled as they are on the Arduino
//! board.
//!
//! This module uses type state programming, so all types here are zero-sized and contain no data.
//! All methods defined on them act on constants. This is why every pin has its own set of types.
//!
//! We use traits to classify pins according to their configuration, [`OutputPin`], [`InputPin`] or
//! [`InputPullupPin`]. To declare a function that can use any pin that is configured as output,
//! write `fn<P: OutputPin> with_output_pin(pin: P, ...)`.
//!
//! For details on the addresses for the registers and their functions, see
//! Renesas RA4M1 Group User's Manual: Hardware, p. 351-374.
//! <https://cdn.sparkfun.com/assets/b/1/d/3/6/RA4M1_Datasheet.pdf>
//!
//! Example:
//! ```
//! use arduino_uno_r4_wifi_rt::peripherals::pins::{get_pins, OutputPin};
//! let mut pins = get_pins();
//! let mut led = pins.d13.into_output(); // Pin D13 controls the LED.
//! led.set_high(); // Light up the LED.
//! ```

use super::registers::VolatileBoolOps;

use core::marker::PhantomData;

pub trait PinMode {}

pub struct PinModeUnknown;
impl PinMode for PinModeUnknown {}

pub struct PinModeOutput;
impl PinMode for PinModeOutput {}

pub struct PinModeInput;
impl PinMode for PinModeInput {}

pub struct PinModeInputPullup;
impl PinMode for PinModeInputPullup {}

/// Status of a GPIO pin.
pub enum PinStatus {
    /// Pin is at LOW voltage.
    Low,
    /// Pin is at HIGH voltage.
    High,
}

struct PinWriteProtection;

/// Control if the Pin Function Select registers can be written to.
impl PinWriteProtection {
    /// Pin Write Protection register. Writes to the Pin Function Select registers are only
    /// effective if bit 6 in this register is 1. Writes to that bit only are effective if bit 7 is
    /// set to 0. They really don't want you to change the pin function by accident!
    /// The other bits are reserved.
    const PWPR: *mut u32 = 0x40040d03 as *mut u32;

    fn new() -> Self {
        Self
    }
    /// Unlock the registers by first setting all bits to 0 and then bit 6 to 1.
    #[inline]
    unsafe fn unlock(&mut self) {
        Self::PWPR.write_volatile(0);
        Self::PWPR.write_volatile(1 << 6);
    }

    /// Lock the registers by first setting all bits to 0 and then bit 7 to 1.
    #[inline]
    unsafe fn lock(&mut self) {
        Self::PWPR.write_volatile(0);
        Self::PWPR.write_volatile(1 << 7);
    }
}

/// Control if a pin is input or output.
struct PinFunctionSelect<P: Pin> {
    write_protection: PinWriteProtection,
    _pin: PhantomData<P>,
}

impl<P: Pin> PinFunctionSelect<P> {
    const BASE_ADDRESS: u32 = 0x40040800;

    /// Pin Function Select Register.
    ///
    /// If all bits are 0, the pin is configured as a GPIO input pin without pullup.
    /// Setting bit 2 to 1 configures it as a GPIO output pin. Setting bit 4 to 1 activates the pullup
    /// resistor for an input pin.
    const PFSR: *mut u32 = (Self::BASE_ADDRESS + 4 * (16 * P::PORT_NO + P::PIN_NO)) as *mut u32;

    fn new() -> Self {
        Self {
            write_protection: PinWriteProtection::new(),
            _pin: PhantomData,
        }
    }

    fn set_to_input(&mut self) {
        unsafe {
            self.write_protection.unlock();
            Self::PFSR.write_volatile(0);
            self.write_protection.lock();
        }
    }

    fn set_to_input_pullup(&mut self) {
        unsafe {
            self.write_protection.unlock();
            Self::PFSR.write_volatile(1 << 4);
            self.write_protection.lock();
        }
    }

    fn set_to_output(&mut self) {
        unsafe {
            self.write_protection.unlock();
            Self::PFSR.write(1 << 2);
            self.write_protection.lock();
        }
    }
}

struct PortControl<P: PortNo> {
    _port: PhantomData<P>,
}

impl<P: PortNo> PortControl<P> {
    const BASE_ADDRESS: u32 = 0x40040000;
    const ADDRESS_GAP: u32 = 0x20;
    const PCNTR1: *mut u32 = (Self::BASE_ADDRESS + P::PORT_NO * Self::ADDRESS_GAP) as *mut u32;
    const PCNTR2: *mut u32 = (Self::BASE_ADDRESS + 4 + P::PORT_NO * Self::ADDRESS_GAP) as *mut u32;

    fn new() -> Self {
        Self { _port: PhantomData }
    }

    fn pin_is_set_high(&self, pin: u32) -> bool {
        unsafe { Self::PCNTR1.read() & (1 << (pin + 16)) != 0 }
    }

    fn pin_is_high(&self, pin: u32) -> bool {
        unsafe { Self::PCNTR2.read() & (1 << pin) != 0 }
    }

    fn set_pin_high(&mut self, pin: u32) {
        unsafe {
            Self::PCNTR1.volatile_or(1 << (pin + 16));
        }
    }

    fn set_pin_low(&mut self, pin: u32) {
        unsafe {
            Self::PCNTR1.volatile_and(!(1 << (pin + 16)));
        }
    }

    fn toggle_pin_output(&mut self, pin: u32) {
        unsafe {
            Self::PCNTR1.volatile_xor(1 << (pin + 16));
        }
    }
}

trait PortNo {
    const PORT_NO: u32;
}

/// Common trait for all pins.
pub trait Pin {
    type PinTypeUnknown: Pin;
    type PinTypeOutput: OutputPin;
    type PinTypeInput: InputPin;
    type PinTypeInputPullup: InputPullupPin;

    const PORT_NO: u32;
    const PIN_NO: u32;

    /// "Forget" the configuration of this pin.
    fn into_unknown(self) -> Self::PinTypeUnknown;

    /// Configure this pin into an output pin.
    fn into_output(self) -> Self::PinTypeOutput;

    /// Configure this pin into an input pin.
    fn into_input(self) -> Self::PinTypeInput;

    /// Configure this pin into an input pin with pull-up.
    fn into_input_pullup(self) -> Self::PinTypeInputPullup;
}

/// A digital output pin.
pub trait OutputPin: Pin {
    /// Is the pin currently set to output HIGH?
    fn is_set_high(&self) -> bool;

    /// Make the pin output HIGH.
    fn set_high(&mut self);

    /// Make the pin output LOW.
    fn set_low(&mut self);

    /// Set the pin status.
    fn set(&mut self, status: PinStatus) {
        match status {
            PinStatus::Low => self.set_low(),
            PinStatus::High => self.set_high(),
        };
    }

    /// Toggle the output of the pin.
    fn toggle(&mut self);
}

/// An input pin.
pub trait InputPin: Pin {
    /// Returns true if this pin has a pull-up.
    fn is_input_pullup(&self) -> bool;

    /// Returns true if this pin receives HIGH voltage.
    fn is_high(&self) -> bool;

    /// Returns true if this pin receives LOW voltage.
    fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Return the input status.
    fn get_status(&self) -> PinStatus {
        if self.is_high() {
            PinStatus::High
        } else {
            PinStatus::Low
        }
    }
}

/// An input pin with a pull-up resistor.
///
/// The value read on an input pin may fluctuate if it is not connected to anything. A pull-up
/// resistor makes the pin always read HIGH in that situation. It only becomes LOW when connected
/// to ground.
pub trait InputPullupPin: InputPin {}

macro_rules! make_port_pins {
    ($port_no:literal, $port_x:ident, $port_x_pins:ident, $($pin_no: literal, $pin_var:ident, $pin_type:ident),*) => {
        $(
            pub struct $pin_type<M: PinMode> {
                port_control: PortControl<$port_x>,
                pin_function_select: PinFunctionSelect<Self>,
                _mode: PhantomData<M>,
            }

            impl<M: PinMode> $pin_type<M> {
                fn new() -> Self {
                    Self {
                        port_control: PortControl::new(),
                        pin_function_select: PinFunctionSelect::new(),
                        _mode: PhantomData,
                    }
                }
            }

            impl<M: PinMode> Pin for $pin_type<M> {
                type PinTypeUnknown = $pin_type<PinModeUnknown>;
                type PinTypeOutput = $pin_type<PinModeOutput>;
                type PinTypeInput = $pin_type<PinModeInput>;
                type PinTypeInputPullup = $pin_type<PinModeInputPullup>;

                const PORT_NO: u32 = $port_no;
                const PIN_NO: u32 = $pin_no;

                #[inline]
                fn into_unknown(self) -> Self::PinTypeUnknown {
                    Self::PinTypeUnknown::new()
                }

                #[inline]
                fn into_output(mut self) -> Self::PinTypeOutput {
                    self.pin_function_select.set_to_output();
                    Self::PinTypeOutput::new()
                }

                #[inline]
                fn into_input(mut self) -> Self::PinTypeInput {
                    self.pin_function_select.set_to_input();
                    Self::PinTypeInput::new()
                }

                #[inline]
                fn into_input_pullup(mut self) -> Self::PinTypeInputPullup {
                    self.pin_function_select.set_to_input_pullup();
                    Self::PinTypeInputPullup::new()
                }
            }

            impl OutputPin for $pin_type<PinModeOutput> {
                #[inline]
                fn is_set_high(&self) -> bool {
                    self.port_control.pin_is_set_high($pin_no)
                }

                #[inline]
                fn set_high(&mut self) {
                    self.port_control.set_pin_high($pin_no);
                }

                #[inline]
                fn set_low(&mut self) {
                    self.port_control.set_pin_low($pin_no);
                }

                #[inline]
                fn toggle(&mut self) {
                    self.port_control.toggle_pin_output($pin_no);
                }
            }

            impl InputPin for $pin_type<PinModeInput> {
                #[inline]
                fn is_high(&self) -> bool {
                    self.port_control.pin_is_high($pin_no)
                }

                #[inline]
                fn is_input_pullup(&self) -> bool {
                    false
                }
            }

            impl InputPin for $pin_type<PinModeInputPullup> {
                #[inline]
                fn is_high(&self) -> bool {
                    self.port_control.pin_is_high($pin_no)
                }

                #[inline]
                fn is_input_pullup(&self) -> bool {
                    true
                }
            }

            impl InputPullupPin for $pin_type<PinModeInputPullup> {}
        )*

        /// A struct containing the pins accessible on this port.
        pub struct $port_x_pins {
            $( pub $pin_var: $pin_type<PinModeUnknown>, )*
        }

        pub struct $port_x {
            _phantom_data: PhantomData<()>
        }

        impl PortNo for $port_x {
            const PORT_NO: u32 = $port_no;
        }

        impl $port_x {
            const fn new() -> Self {
                Self { _phantom_data: PhantomData, }
            }

            /// Split up the port and return a struct with the individual pins.
            pub fn split(self) -> $port_x_pins {
                $port_x_pins {
                    $( $pin_var: $pin_type::<PinModeUnknown>::new(), )*
                }
            }
        }
    };
}

make_port_pins!(0, Port0, Port0Pins, 0, p000, P000, 1, p001, P001, 2, p002, P002, 14, p014, P014);
make_port_pins!(
    1, Port1, Port1Pins, 0, p100, P100, 1, p101, P101, 2, p102, P102, 3, p103, P103, 4, p104, P104,
    5, p105, P105, 6, p106, P106, 7, p107, P107, 11, p111, P111, 12, p112, P112
);
make_port_pins!(3, Port3, Port3Pins, 1, p301, P301, 2, p302, P302, 3, p303, P303, 4, p304, P304);
make_port_pins!(4, Port4, Port4Pins, 10, p410, P410, 11, p411, P411);

/// Pins that are exposed on the Arduino.
///
/// Pin d13 controls the LED.
pub struct ArduinoPins {
    pub d0: P301<PinModeUnknown>,
    pub d1: P302<PinModeUnknown>,
    pub d2: P104<PinModeUnknown>,
    pub d3: P105<PinModeUnknown>,
    pub d4: P106<PinModeUnknown>,
    pub d5: P107<PinModeUnknown>,
    pub d6: P111<PinModeUnknown>,
    pub d7: P112<PinModeUnknown>,
    pub d8: P304<PinModeUnknown>,
    pub d9: P303<PinModeUnknown>,
    pub d10: P103<PinModeUnknown>,
    pub d11: P411<PinModeUnknown>,
    pub d12: P410<PinModeUnknown>,
    pub d13: P102<PinModeUnknown>, // LED
    pub a0: P014<PinModeUnknown>,
    pub a1: P000<PinModeUnknown>,
    pub a2: P001<PinModeUnknown>,
    pub a3: P002<PinModeUnknown>,
    pub a4: P101<PinModeUnknown>,
    pub a5: P100<PinModeUnknown>,
}

pub struct Ports {
    port0: Port0,
    port1: Port1,
    port3: Port3,
    port4: Port4,
}

pub static mut PORTS: Option<Ports> = Some(Ports {
    port0: Port0::new(),
    port1: Port1::new(),
    port3: Port3::new(),
    port4: Port4::new(),
});

/// Get the pins that are exposed on the Arduino board.
pub fn get_pins() -> Option<ArduinoPins> {
    let ports = unsafe { PORTS.take()? };
    let port0_pins = ports.port0.split();
    let port1_pins = ports.port1.split();
    let port3_pins = ports.port3.split();
    let port4_pins = ports.port4.split();

    Some(ArduinoPins {
        d0: port3_pins.p301,
        d1: port3_pins.p302,
        d2: port1_pins.p104,
        d3: port1_pins.p105,
        d4: port1_pins.p106,
        d5: port1_pins.p107,
        d6: port1_pins.p111,
        d7: port1_pins.p112,
        d8: port3_pins.p304,
        d9: port3_pins.p303,
        d10: port1_pins.p103,
        d11: port4_pins.p411,
        d12: port4_pins.p410,
        d13: port1_pins.p102,
        a0: port0_pins.p014,
        a1: port0_pins.p000,
        a2: port0_pins.p001,
        a3: port0_pins.p002,
        a4: port1_pins.p101,
        a5: port1_pins.p100,
    })
}
