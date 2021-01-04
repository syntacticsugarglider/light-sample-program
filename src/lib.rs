#![no_std]

use core::panic::PanicInfo;

mod programs;
pub mod rand;
pub mod util;

///  Array of arrays, type is [[u8; 3]; 75].
/// Each sub-array is one light, colors in order RGB.
static mut STRIP: [[u8; 3]; 75] = [[0u8; 3]; 75];
static mut OUTPUT: Output = Output {
    buffered: true,
    data: OutputData {
        buffered: (0, 74, core::ptr::null_mut()),
    },
};

#[repr(C)]
union OutputData {
    unbuffered: (u8, u8, [u8; 3]),
    buffered: (u8, u8, *mut [u8; 3]),
}

#[repr(C)]
struct Output {
    buffered: bool,
    data: OutputData,
}

#[no_mangle]
extern "C" fn entry() -> *mut Output {
    unsafe {
        OUTPUT.data.buffered.2 = STRIP.as_mut_ptr();

        // choose program here
        programs::twinkle();
    };
    unsafe { &mut OUTPUT }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
