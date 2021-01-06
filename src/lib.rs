#![feature(type_alias_impl_trait)]
#![no_std]

use core::{
    future::Future,
    panic::PanicInfo,
    pin::Pin,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};
use pin_project::pin_project;

mod programs;
use programs::Program;
pub mod rand;
pub mod util;

/// Array of arrays, each sub-array is one light, colors in order RGB.
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

fn clone(_: *const ()) -> RawWaker {
    RawWaker::new(core::ptr::null(), &WAKER_VTABLE)
}

fn wake(_: *const ()) {}

fn wake_by_ref(_: *const ()) {}

fn drop(_: *const ()) {}

static WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

#[pin_project]
struct Executor<T: Future<Output = ()>>(#[pin] T);

impl<T: Future<Output = ()>> Executor<T> {
    fn run(self: Pin<&mut Self>) {
        let this = self.project();
        let waker = unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &WAKER_VTABLE)) };
        let _ = this.0.poll(&mut Context::from_waker(&waker));
    }
}

static mut PROGRAM: Option<Executor<Program>> = None;

#[no_mangle]
extern "C" fn entry() -> *mut Output {
    unsafe {
        OUTPUT.data.buffered.2 = STRIP.as_mut_ptr();

        util::CURRENT_TICK = !util::CURRENT_TICK;
        util::TICKS_ELAPSED += 1;

        if let Some(executor) = PROGRAM.as_mut() {
            Pin::new_unchecked(executor).run();
        } else {
            PROGRAM = Some(Executor(programs::twinkle()));
        }
    };
    unsafe { &mut OUTPUT }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
