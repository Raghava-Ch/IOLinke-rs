//! Module: hooks
//!
//! This module provides platform-specific hooks for ARM architectures,
//! including a custom panic handler that invokes the `hard_fault` routine
//! on panic. This is useful for embedded systems where standard panic
//! behavior may not be appropriate.
//!
//! # Features
//! - Declares an external `hard_fault` function for ARM targets.
//! - Implements a panic handler that calls `hard_fault` on panic.
//!
//! # Platform Support
//! This module is only active when compiling for ARM architectures.

#[cfg(all(target_arch = "arm"))]
use core::panic::PanicInfo;

#[cfg(all(target_arch = "arm"))]
unsafe extern "C" {
    fn hard_fault();
}

#[cfg(all(target_arch = "arm"))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { hard_fault() }
    loop {}
}
