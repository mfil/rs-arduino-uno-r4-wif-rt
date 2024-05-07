use core::ops::{BitAnd, BitOr, BitXor};

/// Trait for the types that have registers associated: `u8`, `u16` and `u32`.
pub trait RegisterType:
    Copy + BitOr<Output = Self> + BitAnd<Output = Self> + BitXor<Output = Self>
{
}

impl RegisterType for u8 {}
impl RegisterType for u16 {}
impl RegisterType for u32 {}

/// Operations to update volatile registers with a bitmask.
pub trait VolatileBoolOps<T: RegisterType>: Copy {
    unsafe fn volatile_or(self, bitmask: T);
    unsafe fn volatile_and(self, bitmask: T);
    unsafe fn volatile_xor(self, bitmask: T);
}

impl<T: RegisterType> VolatileBoolOps<T> for *mut T {
    /// OR bitmask into the register.
    #[inline]
    unsafe fn volatile_or(self, bitmask: T) {
        self.write_volatile(self.read_volatile() | bitmask);
    }

    /// AND bitmask into the register.
    #[inline]
    unsafe fn volatile_and(self, bitmask: T) {
        self.write_volatile(self.read_volatile() & bitmask);
    }

    /// XOR bitmask into the register.
    #[inline]
    unsafe fn volatile_xor(self, bitmask: T) {
        self.write_volatile(self.read_volatile() ^ bitmask);
    }
}
