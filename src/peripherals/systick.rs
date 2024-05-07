//! A module exposing the SysTick timer functions of the Arm CPU.
//!
//! See Armv7-M Architecture Reference Manual, p. 620-623.

use super::registers::VolatileBoolOps;

/// System timer of the ARM CPU.
///
/// When enabled, the timer value ticks down until it reaches 0, then wraps around to a configurable
/// reset value. A status bit is set when it wraps around.
pub struct SysTick {
    enabled: bool,
}

impl SysTick {
    /// SysTick Control and Status Register.
    /// The most important bits are:
    /// * b0: Set to 0/1 to enable/disable the timer.
    /// * b1: Enable SysTick interrupt (never got that to work...)
    /// * b16: Has `cvr` reached 0 since we last read this register?
    ///        Note: This bit is set to 0 every time we read this register.
    const CSR: *mut u32 = 0xe000e010 as *mut u32;

    /// SysTick Reset Value Register. Stores the value that cvr is set to when it ticks down to 0.
    /// The upper 8 bits are reserved, only the lower 24 bits act as the reset value.
    const RVR: *mut u32 = 0xe000e014 as *mut u32;

    /// SysTick Current Value Register. When the timer is enabled, this ticks down to 0 and then wraps
    /// to the reset value. Any write resets the register to 0. It will take the value from `rvr`
    /// on the next clock cycle.
    const CVR: *mut u32 = 0xe000e018 as *mut u32;

    /// SysTick Calibration Register. The most important part are the lower 24 bits: They show how many
    /// ticks correspond to 10ms.
    const CALIB: *const u32 = 0xe000e01c as *mut u32;

    #[inline]
    fn new() -> Self {
        SysTick { enabled: false }
    }

    /// Create a SysTick instance if no instance has been created yet.
    pub fn instance() -> Option<Self> {
        unsafe {
            static mut SYSTICK_CREATED: bool = false;
            if !SYSTICK_CREATED {
                Some(SysTick::new())
            } else {
                None
            }
        }
    }

    /// Returns the number of ticks per 10ms.
    #[inline]
    pub fn get_ticks_per_10ms(&self) -> u32 {
        unsafe { Self::CALIB.read_volatile() & 0x00ffffff }
    }

    /// Run the timer.
    ///
    /// Calling this function clears the status bit that checks if the timer wrapped.
    #[inline]
    pub fn enable(&mut self) {
        unsafe {
            Self::CSR.volatile_or(1);
        }
        self.enabled = true;
    }

    /// Stop the timer.
    ///
    /// Calling this function clears the status bit that checks if the timer wrapped.
    #[inline]
    pub fn disable(&mut self) {
        unsafe {
            Self::CSR.volatile_and(!1);
        }
        self.enabled = false;
    }

    /// Returns true if the timer is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns true if the timer has wrapped since it was checked last time.
    ///
    /// This takes `&mut self` because reading csr clears the bit.
    #[inline]
    pub fn timer_wrapped(&mut self) -> bool {
        unsafe { Self::CSR.read_volatile() & (1 << 16) != 0 }
    }

    /// Read the current value.
    #[inline]
    pub fn get_current_value(&self) -> u32 {
        unsafe { Self::CVR.read_volatile() }
    }

    /// Reset the timer.
    #[inline]
    pub fn reset(&mut self) {
        unsafe {
            Self::CVR.write(0);
        }
    }

    /// Set a new reset value. This has no impact on the current timer value.
    /// Ignores the top 8 bits of `reset value`.
    #[inline]
    pub fn set_reset_value(&mut self, reset_value: u32) {
        let clamped_reset_value = reset_value & 0x00ffffff;
        unsafe {
            Self::RVR.write_volatile(clamped_reset_value);
        }
    }

    /// Read the reset value.
    #[inline]
    pub fn get_reset_value(&self) -> u32 {
        unsafe { Self::RVR.read_volatile() & 0x00ffffff }
    }
}
