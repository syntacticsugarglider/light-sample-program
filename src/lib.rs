#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]
#![feature(core_intrinsics)]
#![allow(incomplete_features)]
#![no_std]

use core::{
    cell::UnsafeCell,
    future::Future,
    intrinsics::sqrtf32,
    ops::{Bound, Index, IndexMut, Mul, MulAssign, RangeBounds},
    panic::PanicInfo,
    pin::Pin,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};
use pin_project::pin_project;

mod programs;
use programs::Program;
pub mod rand;
pub mod util;

pub struct LedStrip(UnsafeCell<&'static mut [[u8; 3]]>);

mod sealed {

    pub trait Sealed {}

    impl Sealed for [u8; 3] {}
    impl Sealed for super::LedStrip {}
}

pub struct Scale(pub f32);

impl From<f32> for Scale {
    fn from(item: f32) -> Self {
        Scale(item)
    }
}

impl Mul<Scale> for [u8; 3] {
    type Output = [u8; 3];

    fn mul(self, rhs: Scale) -> Self::Output {
        [
            (self[0] as f32 * rhs.0) as u8,
            (self[1] as f32 * rhs.0) as u8,
            (self[2] as f32 * rhs.0) as u8,
        ]
    }
}

impl MulAssign<Scale> for [u8; 3] {
    fn mul_assign(&mut self, rhs: Scale) {
        *self = self.mul(rhs);
    }
}

pub trait LedExt: sealed::Sealed {
    fn scale(&mut self, factor: f32) -> &mut Self;

    fn normalize(&mut self) -> &mut Self;

    type FadeFuture<'a, T: 'a>: Future<Output = ()> + 'a;

    fn fade_to<'a, T: IntoIterator<Item = [u8; 3]> + 'a>(
        &'a self,
        target: T,
        ticks: u64,
    ) -> Self::FadeFuture<'a, T>;
}

type LedFadeFuture<'a, T: 'a> = impl Future<Output = ()> + 'a;

impl LedExt for [u8; 3] {
    fn scale(&mut self, factor: f32) -> &mut Self {
        for element in self.as_mut() {
            *element = (*element as f32 * factor) as u8;
        }
        self
    }

    type FadeFuture<'a, T: 'a> = LedFadeFuture<'a, T>;

    fn fade_to<'a, T: IntoIterator<Item = [u8; 3]> + 'a>(
        &'a self,
        target: T,
        ticks: u64,
    ) -> Self::FadeFuture<'a, T> {
        use util::next_tick;

        let mut tick = 0;

        let target = target.into_iter().next().unwrap();

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
                let this = unsafe { core::slice::from_raw_parts_mut(self.as_ptr() as *mut _, 3) };
                initial[0] += delta[0];
                initial[1] += delta[1];
                initial[2] += delta[2];
                this[0] = initial[0] as u8;
                this[1] = initial[1] as u8;
                this[2] = initial[2] as u8;
            }
        }
    }

    fn normalize(&mut self) -> &mut Self {
        let _0 = self[0] as f32;
        let _1 = self[1] as f32;
        let _2 = self[2] as f32;
        let norm = unsafe { sqrtf32(_0 * _0 + _1 * _1 + _2 * _2) };
        let factor = 225. / norm;
        *self *= factor.into();
        self
    }
}

type LedStripFadeFuture<'a, T: 'a> = impl Future<Output = ()> + 'a;

impl LedExt for LedStrip {
    fn scale(&mut self, factor: f32) -> &mut Self {
        for led in self.0.get_mut().as_mut() {
            led.scale(factor);
        }
        self
    }

    type FadeFuture<'a, T: 'a> = LedStripFadeFuture<'a, T>;

    fn fade_to<'a, T: IntoIterator<Item = [u8; 3]> + 'a>(
        &'a self,
        target: T,
        ticks: u64,
    ) -> Self::FadeFuture<'a, T> {
        use util::next_tick;
        let this = unsafe { &mut *self.0.get() };

        let mut tick = 0;
        let max_ticks = ticks as f32;

        let mut initial = [[0f32; 3]; 75];
        for (buffer, current) in initial[..this.len()].iter_mut().zip(this.iter_mut()) {
            *buffer = [current[0] as f32, current[1] as f32, current[2] as f32];
        }
        let mut delta = [[0f32; 3]; 75];
        for ((delta, initial), target) in delta[..this.len()]
            .iter_mut()
            .zip(initial.as_mut())
            .zip(target)
        {
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
                    .zip(this.iter_mut())
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

    fn normalize(&mut self) -> &mut Self {
        for light in &mut *self {
            light.normalize();
        }
        self
    }
}

impl LedStrip {
    pub fn len(&self) -> usize {
        unsafe { &*self.0.get() }.len()
    }
    pub fn range<T: RangeBounds<usize>>(&mut self, range: T) -> LedStrip {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => *bound + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(bound) => *bound - 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.0.get_mut().len(),
        };
        LedStrip(unsafe {
            UnsafeCell::new(core::slice::from_raw_parts_mut(
                self.0.get_mut().as_mut_ptr().add(start),
                end - start,
            ))
        })
    }
    pub fn fill(&mut self, color: [u8; 3]) -> &mut Self {
        let buf = self.0.get_mut();
        for led in buf.iter_mut() {
            *led = color;
        }
        self
    }
    pub fn clear(&mut self) -> &mut Self {
        self.fill([0, 0, 0])
    }
}

impl Index<usize> for LedStrip {
    type Output = [u8; 3];

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.0.get() }.index(index)
    }
}

impl IndexMut<usize> for LedStrip {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.get_mut().index_mut(index)
    }
}

impl IntoIterator for LedStrip {
    type Item = &'static mut [u8; 3];

    type IntoIter = core::slice::IterMut<'static, [u8; 3]>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_inner().into_iter()
    }
}

impl<'a> IntoIterator for &'a mut LedStrip {
    type Item = &'a mut [u8; 3];

    type IntoIter = core::slice::IterMut<'a, [u8; 3]>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.get_mut().into_iter()
    }
}

mod strip {
    pub(super) static mut STRIP: [[u8; 3]; 75] = [[0u8; 3]; 75];
}

pub fn leds() -> LedStrip {
    LedStrip(unsafe { UnsafeCell::new(strip::STRIP.as_mut()) })
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
            PROGRAM = Some(Executor(programs::program()));
        }
    };
    unsafe { &mut OUTPUT }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
