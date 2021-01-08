#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]
#![feature(core_intrinsics)]
#![allow(incomplete_features)]
#![no_std]

const LED_COUNT: usize = 76;

use core::{
    cell::UnsafeCell,
    future::Future,
    intrinsics::sqrtf32,
    iter::repeat,
    ops::{Bound, Index, IndexMut, Mul, MulAssign, RangeBounds},
    panic::PanicInfo,
    pin::Pin,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};
use futures::Stream;
use pin_project::pin_project;

mod programs;
use programs::Program;
use util::interpolate::{Interpolate, Linear};
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

    type FadeFuture<'a, T: 'a, I: 'a>: Future<Output = ()> + 'a;

    fn fade_to<'a, T: IntoIterator<Item = [u8; 3]> + 'a, I: Interpolate + 'a>(
        &'a self,
        target: T,
        ticks: u64,
    ) -> Self::FadeFuture<'a, T, I>;
}

type LedFadeFuture<'a, T: 'a, I: 'a> = impl Future<Output = ()> + 'a;

impl LedExt for [u8; 3] {
    fn scale(&mut self, factor: f32) -> &mut Self {
        for element in self.as_mut() {
            *element = (*element as f32 * factor) as u8;
        }
        self
    }

    type FadeFuture<'a, T: 'a, I: 'a> = LedFadeFuture<'a, T, I>;

    fn fade_to<'a, T: IntoIterator<Item = [u8; 3]> + 'a, I: Interpolate + 'a>(
        &'a self,
        target: T,
        ticks: u64,
    ) -> Self::FadeFuture<'a, T, I> {
        use util::next_tick;

        let mut tick = 0;
        let max_ticks = ticks as f32;

        let target = target.into_iter().next().unwrap();

        let initial = *self;

        async move {
            while tick < ticks {
                next_tick().await;
                tick += 1;
                let this = unsafe { core::slice::from_raw_parts_mut(self.as_ptr() as *mut _, 3) };
                this.copy_from_slice(&I::interpolate(&initial, &target, tick as f32 / max_ticks));
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

type LedStripFadeFuture<'a, T: 'a, I> = impl Future<Output = ()> + 'a;

impl LedExt for LedStrip {
    fn scale(&mut self, factor: f32) -> &mut Self {
        for led in self.0.get_mut().as_mut() {
            led.scale(factor);
        }
        self
    }

    type FadeFuture<'a, T: 'a, I: 'a> = LedStripFadeFuture<'a, T, I>;

    fn fade_to<'a, T: IntoIterator<Item = [u8; 3]> + 'a, I: Interpolate + 'a>(
        &'a self,
        iter: T,
        ticks: u64,
    ) -> Self::FadeFuture<'a, T, I> {
        use util::next_tick;
        let this = unsafe { &mut *self.0.get() };

        let mut tick = 0;
        let max_ticks = ticks as f32;

        let mut initial = [[0u8; 3]; LED_COUNT];
        initial.copy_from_slice(this);
        let mut target = [[0u8; 3]; LED_COUNT];
        for (place, item) in target.iter_mut().zip(iter) {
            *place = item;
        }

        async move {
            while tick < ticks {
                next_tick().await;
                tick += 1;
                for ((initial, target), this) in initial
                    .iter_mut()
                    .zip(target.iter_mut())
                    .zip(this.iter_mut())
                {
                    *this = Linear::interpolate(initial, target, tick as f32 / max_ticks);
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
    pub fn rotate_left(&self, by: usize) -> &Self {
        unsafe { &mut *self.0.get() }.rotate_left(by);
        self
    }
    pub fn rotate_right(&self, by: usize) -> &Self {
        unsafe { &mut *self.0.get() }.rotate_right(by);
        self
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
    pub fn fill_from<T: IntoIterator<Item = [u8; 3]>>(&mut self, iter: T) -> &mut Self {
        for (led, color) in self.0.get_mut().iter_mut().zip(iter) {
            *led = color;
        }
        self
    }
    pub fn fill(&mut self, color: [u8; 3]) -> &mut Self {
        self.fill_from(repeat(color))
    }
    pub fn clear(&mut self) -> &mut Self {
        self.fill_from(repeat([0, 0, 0]))
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

impl<'a> IntoIterator for &'a LedStrip {
    type Item = &'a mut [u8; 3];

    type IntoIter = core::slice::IterMut<'a, [u8; 3]>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe { &mut *self.0.get() }.into_iter()
    }
}

mod strip {
    pub(super) static mut STRIP: [[u8; 3]; crate::LED_COUNT] = [[0u8; 3]; crate::LED_COUNT];
}

pub fn leds() -> LedStrip {
    LedStrip(unsafe { UnsafeCell::new(strip::STRIP.as_mut()) })
}

/// Array of arrays, each sub-array is one light, colors in order RGB.
static mut OUTPUT: Output = Output {
    buffered: true,
    data: OutputData {
        buffered: (0, (crate::LED_COUNT - 1) as u8, core::ptr::null_mut()),
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

static mut INPUT_HANDLER: fn(len: usize) -> *mut u8 = |_| core::ptr::null_mut();

#[no_mangle]
extern "C" fn handle_input(len: usize) -> *mut u8 {
    unsafe { INPUT_HANDLER(len) }
}

pub trait Receiver<const LEN: usize>: Stream<Item = ([u8; LEN], usize)> {
    type ExactStream: ExactReceiver<LEN> + Unpin;

    fn exact(self) -> Self::ExactStream
    where
        Self: Sized;
}

pub trait ExactReceiver<const LEN: usize>: Stream<Item = [u8; LEN]> {
    type Inner: Receiver<LEN> + Unpin;

    fn into_inner(self) -> Self::Inner
    where
        Self: Sized;
}

#[macro_export]
macro_rules! Receiver {
    ($len:expr) => {{
        let receiver_data: receiver_hidden::Receiver = receiver_hidden::Receiver(());
        #[doc(hidden)]
        mod receiver_hidden {
            #[no_mangle]
            static mut RECEIVER_MUST_BE_UNIQUE: () = ();
            pub static mut DATA_AVAILABLE: bool = false;
            pub static mut DATA: [u8; $len] = [0u8; $len];
            pub static mut LEN: usize = 0;
            pub(super) struct Receiver(pub(super) ());
        }
        impl crate::Receiver<$len> for receiver_hidden::Receiver {
            type ExactStream = impl crate::ExactReceiver<$len>;

            fn exact(self) -> Self::ExactStream
            where
                Self: Sized,
            {
                struct ExactWrapper(receiver_hidden::Receiver);

                impl ::futures::Stream for ExactWrapper {
                    type Item = [u8; $len];

                    fn poll_next<'a>(
                        mut self: ::core::pin::Pin<&'a mut Self>,
                        cx: &mut ::core::task::Context<'_>,
                    ) -> ::core::task::Poll<Option<Self::Item>> {
                        match core::pin::Pin::new(&mut self.0).poll_next(cx) {
                            ::core::task::Poll::Ready(Some((data, $len))) => {
                                ::core::task::Poll::Ready(Some(data))
                            }
                            _ => ::core::task::Poll::Pending,
                        }
                    }
                }

                impl crate::ExactReceiver<$len> for ExactWrapper {
                    type Inner = receiver_hidden::Receiver;

                    fn into_inner(self) -> Self::Inner {
                        self.0
                    }
                }

                ExactWrapper(self)
            }
        }
        impl ::futures::Stream for receiver_hidden::Receiver {
            type Item = ([u8; $len], usize);

            fn poll_next<'a>(
                self: ::core::pin::Pin<&'a mut Self>,
                _: &mut ::core::task::Context<'_>,
            ) -> ::core::task::Poll<Option<Self::Item>> {
                unsafe {
                    if receiver_hidden::DATA_AVAILABLE {
                        receiver_hidden::DATA_AVAILABLE = false;
                        ::core::task::Poll::Ready(Some((
                            receiver_hidden::DATA,
                            receiver_hidden::LEN,
                        )))
                    } else {
                        ::core::task::Poll::Pending
                    }
                }
            }
        }
        #[allow(unused_unsafe)]
        unsafe {
            crate::INPUT_HANDLER = |len| {
                if receiver_hidden::DATA_AVAILABLE || len > $len {
                    core::ptr::null_mut()
                } else {
                    receiver_hidden::DATA_AVAILABLE = true;
                    receiver_hidden::LEN = len;
                    receiver_hidden::DATA.as_mut_ptr()
                }
            };
        }
        receiver_data
    }};
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
