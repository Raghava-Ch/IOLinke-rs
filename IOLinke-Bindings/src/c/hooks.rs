#[cfg(all(target_arch = "arm"))]
use core::panic::PanicInfo;

#[cfg(all(target_arch = "arm"))]
unsafe extern "C" {

}

#[cfg(all(target_arch = "arm"))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        // could also trigger a breakpoint or reset
    }
}
