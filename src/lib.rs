#![no_std]

pub mod peripherals;

use core::panic::PanicInfo;
use core::ptr;

#[derive(Clone, Copy)]
/// Entry for the vector table. Usually a function pointer, but some are reserved and must be set
/// to 0.
pub union VectorTableEntry {
    reserved: u32,
    handler: unsafe fn(),
}

#[macro_export]
/// Macro to set the entry point (or main function) of the program. The type of
/// the function must be `fn() -> !`, i.e., it takes no arguments and doesn't return.
macro_rules! entry {
    ($path:path) => {
        #[export_name = "__main"]
        pub unsafe fn __main() -> ! {
            // Type-check the given path.
            let f: fn() -> ! = $path;

            // Execute the main function.
            f();
        }
    };
}

#[panic_handler]
/// Dummy panic handler, loops infinitely.
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

/// Initializes the RAM by setting the .bss section to 0 and copying the initial
/// values of static variables from FLASH to the .data section.
unsafe fn initialize_ram() {
    // Get the symbols defined in link.x to mark the memory sections.
    // We need to take the address of those symbols to get the corresponding
    // memory locations.
    extern "C" {
        static mut _sbss: u8;
        static mut _ebss: u8;
        static mut _sdata: u8;
        static mut _edata: u8;
        static mut _sidata: u8;
    }

    let sbss = ptr::addr_of_mut!(_sbss);
    let ebss = ptr::addr_of!(_ebss);
    let sdata = ptr::addr_of_mut!(_sdata);
    let edata = ptr::addr_of!(_edata);
    let sidata = ptr::addr_of!(_sidata);

    let count_bss = ebss as usize - sbss as usize;
    ptr::write_bytes(sbss, 0, count_bss);
    let count_data = edata as usize - sdata as usize;
    ptr::copy_nonoverlapping(sidata, sdata, count_data);
}

/// Dummy exception handler, loops infinitely. It is public so that it can't be optimized away.
pub fn default_exception_handler() {
    loop {}
}

#[no_mangle]
/// Reset handler, calls the main function defined with the `entry` macro.
/// This is public and extern so we can mark it as the entry point in `link.x`.
pub unsafe extern "C" fn Reset() -> ! {
    extern "Rust" {
        fn __main() -> !;
    }
    initialize_ram();
    __main();
}

#[link_section = ".vector_table.reset_vector"]
/// Pointer to [`Reset`], second entry in the vector table.
///
/// ARMv7M CPUs expect a vector table at the start of the flash memory. The first entry is the inital
/// value of the stack pointer which we set in the linker script. The rest are function pointers for
/// various exception/interrupt handlers. We define these in this file, and ensure in the linker
/// script that they are placed in the correct location. All the parts of the vector table are
/// marked as public so they can't be optimized away.
pub static RESET_VECTOR: unsafe extern "C" fn() -> ! = Reset;

#[link_section = ".vector_table.exceptions"]
#[no_mangle]
/// Array of pointers to the exception/interrupt handler functions. Some are reserved and set to 0.
/// The others are set to [`default_exception_handler`]. Comes after the reset pointer in the vector
/// table.
pub static EXCEPTIONS: [VectorTableEntry; 14] = [
    // 2: NMI
    VectorTableEntry {
        handler: default_exception_handler,
    },
    // 3: HardFault
    VectorTableEntry {
        handler: default_exception_handler,
    },
    // 4: MemManage
    VectorTableEntry {
        handler: default_exception_handler,
    },
    // 5: BusFault
    VectorTableEntry {
        handler: default_exception_handler,
    },
    // 6: UsageFault
    VectorTableEntry {
        handler: default_exception_handler,
    },
    // 7-10: reserved.
    VectorTableEntry { reserved: 0 },
    VectorTableEntry { reserved: 0 },
    VectorTableEntry { reserved: 0 },
    VectorTableEntry { reserved: 0 },
    // 11: SVCall
    VectorTableEntry {
        handler: default_exception_handler,
    },
    // 12: DebugMonitor
    VectorTableEntry {
        handler: default_exception_handler,
    },
    // 13: reserved
    VectorTableEntry { reserved: 0 },
    // 14: PendSV
    VectorTableEntry {
        handler: default_exception_handler,
    },
    // 15: SysTick
    VectorTableEntry {
        handler: default_exception_handler,
    },
];

/// The number of external interrupts is implementation-defined. For the Arduino UNO R4 WIFI, the
/// number is 32. This number can be calculated from the ICTR register (`0xe000e004`).
const NUM_EXTERNAL_INTERRUPTS: usize = 32;

#[link_section = ".vector_table.external_interrupts"]
#[no_mangle]
/// Pointers to the handlers of external interrupts, all set to [`default_exception_handler`].
/// Placed in the vector table after [`EXCEPTIONS`].
pub static EXTERNAL_INTERRUPTS: [VectorTableEntry; NUM_EXTERNAL_INTERRUPTS] = [VectorTableEntry {
    handler: default_exception_handler,
};
    NUM_EXTERNAL_INTERRUPTS];
