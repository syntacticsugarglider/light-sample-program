#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]
#![allow(incomplete_features)]
#![no_std]

use core::{
    future::Future,
    ops::{Bound, Index, IndexMut, RangeBounds},
    panic::PanicInfo,
    pin::Pin,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};
use pin_project::pin_project;

mod programs;
use programs::Program;
pub mod rand;
pub mod util;

pub struct LedStrip(&'static mut [[u8; 3]]);

mod sealed {

    pub trait Sealed {}

    impl Sealed for [u8; 3] {}
    impl Sealed for super::LedStrip {}
}

pub trait LedExt: sealed::Sealed {
    fn scale(&mut self, factor: f32);

    type FadeFuture<'a>: Future<Output = ()> + 'a;

    fn fade_to<'a>(&'a mut self, target: [u8; 3], ticks: u64) -> Self::FadeFuture<'a>;
}

type LedFadeFuture<'a> = impl Future<Output = ()> + 'a;

impl LedExt for [u8; 3] {
    fn scale(&mut self, factor: f32) {
        for element in self {
            *element = (*element as f32 * factor) as u8;
        }
    }

    type FadeFuture<'a> = LedFadeFuture<'a>;

    fn fade_to<'a>(&'a mut self, target: [u8; 3], ticks: u64) -> Self::FadeFuture<'a> {
        use util::next_tick;

        let mut tick = 0;

        let mut initial = [self[0] as f32, self[1] as f32, self[2] as f32];
        let delta = [
            target[0] as f32 - self[0] as f32,
            target[1] as f32 - self[1] as f32,
            target[2] as f32 - self[2] as f32,
        ];
        let delta = [
            delta[0] / ticks as f32,
            delta[1] / ticks as f32,
            delta[2] / ticks as f32,
        ];

        async move {
            while tick < ticks {
                next_tick().await;
                tick += 1;
                initial[0] += delta[0];
                initial[1] += delta[1];
                initial[2] += delta[2];
                self[0] = initial[0] as u8;
                self[1] = initial[1] as u8;
                self[2] = initial[2] as u8;
            }
        }
    }
}

type LedStripFadeFuture<'a> = impl Future<Output = ()> + 'a;

impl LedExt for LedStrip {
    fn scale(&mut self, factor: f32) {
        for led in self {
            led.scale(factor)
        }
    }

    type FadeFuture<'a> = LedStripFadeFuture<'a>;

    fn fade_to<'a>(&'a mut self, target: [u8; 3], ticks: u64) -> Self::FadeFuture<'a> {
        use util::next_tick;

        let mut tick = 0;
        let max_ticks = ticks as f32;

        let mut initial = [[0f32; 3]; 75];
        for (buffer, current) in initial[..self.0.len()].iter_mut().zip(self.0.iter_mut()) {
            *buffer = [current[0] as f32, current[1] as f32, current[2] as f32];
        }
        let mut delta = [[0f32; 3]; 75];
        for (delta, initial) in delta[..self.0.len()].iter_mut().zip(initial.as_mut()) {
            *delta = [
                (target[0] as f32 - initial[0]) / max_ticks,
                (target[1] as f32 - initial[1]) / max_ticks,
                (target[2] as f32 - initial[2]) / max_ticks,
            ];
        }

        async move {
            while tick < ticks {
                next_tick().await;
                tick += 1;
                for ((initial, delta), this) in initial
                    .iter_mut()
                    .zip(delta.iter_mut())
                    .zip(self.0.iter_mut())
                {
                    initial[0] += delta[0];
                    initial[1] += delta[1];
                    initial[2] += delta[2];
                    this[0] = initial[0] as u8;
                    this[1] = initial[1] as u8;
                    this[2] = initial[2] as u8;
                }
            }
        }
    }
}

impl LedStrip {
    pub fn range<T: RangeBounds<usize>>(&mut self, range: T) -> LedStrip {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => *bound + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(bound) => *bound - 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.0.len(),
        };
        LedStrip(unsafe {
            core::slice::from_raw_parts_mut(self.0.as_mut_ptr().add(start), end - start)
        })
    }
    pub fn fill(&mut self, color: [u8; 3]) {
        for led in self.0.iter_mut() {
            *led = color;
        }
    }
    pub fn clear(&mut self) {
        self.fill([0, 0, 0])
    }
}

impl Index<usize> for LedStrip {
    type Output = [u8; 3];

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for LedStrip {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl IntoIterator for LedStrip {
    type Item = &'static mut [u8; 3];

    type IntoIter = core::slice::IterMut<'static, [u8; 3]>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut LedStrip {
    type Item = &'a mut [u8; 3];

    type IntoIter = core::slice::IterMut<'a, [u8; 3]>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

mod strip {
    pub(super) static mut STRIP: [[u8; 3]; 75] = [[0u8; 3]; 75];
}

pub fn leds() -> LedStrip {
    LedStrip(unsafe { strip::STRIP.as_mut() })
}

/// Array of arrays, each sub-array is one light, colors in order RGB.
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
        OUTPUT.data.buffered.2 = strip::STRIP.as_mut_ptr();

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
