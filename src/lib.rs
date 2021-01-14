#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]
#![feature(core_intrinsics)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(const_fn)]
#![feature(const_in_array_repeat_expressions)]
#![feature(step_trait)]
#![feature(iter_map_while)]
#![feature(asm)]
#![feature(specialization)]
#![allow(incomplete_features)]
#![cfg_attr(not(feature = "_simulator"), no_std)]

const LED_COUNT: usize = 88;

#[allow(unused_imports)]
use byte_copy::*;

#[cfg(all(target_arch = "wasm32", not(feature = "_simulator")))]
use core::panic::PanicInfo;
use core::{
    cell::UnsafeCell,
    future::Future,
    intrinsics::sqrtf32,
    ops::{Index, IndexMut, Mul, MulAssign},
    pin::Pin,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};
#[allow(unused_imports)]
use futures::Stream;
use pin_project::pin_project;

mod programs;
use programs::Program;
use util::interpolate::{Interpolate, Linear};
pub mod projection;
pub mod rand;
pub mod util;

pub struct LedStrip(UnsafeCell<&'static mut [[u8; 3]]>, projection::StripSpace);

impl Clone for LedStrip {
    fn clone(&self) -> Self {
        let slice = unsafe { &mut *self.0.get() };
        let len = slice.len();
        LedStrip(
            UnsafeCell::new(unsafe { core::slice::from_raw_parts_mut(slice.as_mut_ptr(), len) }),
            projection::StripSpace(len),
        )
    }
}

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

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut [u8; 3]> {
        self.0.get_mut().get_mut(idx)
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
    LedStrip(
        unsafe { UnsafeCell::new(strip::STRIP.as_mut()) },
        projection::StripSpace(unsafe { strip::STRIP.len() }),
    )
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

#[cfg(feature = "_simulator")]
mod panic_data {
    #[no_mangle]
    #[used]
    pub(super) static mut PANIC_DATA: *const u8 = std::ptr::null();
    #[no_mangle]
    #[used]
    pub(super) static mut PANIC_LEN: usize = 0;
}

#[no_mangle]
extern "C" fn entry() -> *mut Output {
    #[cfg(feature = "_simulator")]
    {
        std::panic::set_hook(Box::new(|info| {
            let mut data = String::new();
            if let Some(s) = info.payload().downcast_ref::<&str>() {
                data = s.to_string();
            }
            if let Some(s) = info.location() {
                data.push_str(&format!("\r\n{}:{}:{}", s.file(), s.line(), s.column()));
            }
            unsafe {
                panic_data::PANIC_LEN = data.len();
                panic_data::PANIC_DATA = data.as_bytes().as_ptr();
            }
            unsafe { asm!("unreachable") }
        }));
    }

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

#[macro_export]
macro_rules! Receiver {
    (type $ty:ty) => {{
        const fn generate_len() -> usize {
            <$ty as crate::ByteCopy>::MIN_LENGTH
        }
        crate::Receiver!({ generate_len() }).extract::<$ty>()
    }};
    ($len:expr) => {{
        let receiver_data: receiver_hidden::Receiver = receiver_hidden::Receiver(());
        const CONST_LEN: usize = $len;
        pub static mut DATA: [u8; CONST_LEN] = [0u8; CONST_LEN];
        #[doc(hidden)]
        pub mod receiver_hidden {
            #[link_section = "custom_discard"]
            #[no_mangle]
            static mut RECEIVER_MUST_BE_UNIQUE: u8 = 0;
            pub static mut DATA_AVAILABLE: bool = false;
            pub static mut LEN: usize = 0;
            pub(super) struct Receiver(pub(super) ());
        }
        impl crate::Receiver<CONST_LEN> for receiver_hidden::Receiver
        where
            [(); CONST_LEN]: Sized,
        {
            type ExactStream = impl crate::ExactReceiver<CONST_LEN>;
            type ExtractStream<T: Unpin + crate::ByteCopy> = impl crate::Stream<Item = T> + Unpin;

            fn exact(self) -> Self::ExactStream
            where
                Self: Sized,
            {
                struct ExactWrapper(receiver_hidden::Receiver);

                impl ::futures::Stream for ExactWrapper {
                    type Item = [u8; CONST_LEN];

                    fn poll_next<'a>(
                        mut self: ::core::pin::Pin<&'a mut Self>,
                        cx: &mut ::core::task::Context<'_>,
                    ) -> ::core::task::Poll<Option<Self::Item>> {
                        match core::pin::Pin::new(&mut self.0).poll_next(cx) {
                            ::core::task::Poll::Ready(Some((data, len))) => {
                                if len != CONST_LEN {
                                    return ::core::task::Poll::Pending;
                                }
                                ::core::task::Poll::Ready(Some(data))
                            }
                            _ => ::core::task::Poll::Pending,
                        }
                    }
                }

                impl crate::ExactReceiver<CONST_LEN> for ExactWrapper {
                    type Inner = receiver_hidden::Receiver;

                    fn into_inner(self) -> Self::Inner {
                        self.0
                    }
                }

                ExactWrapper(self)
            }

            fn extract<T: Unpin>(self) -> Self::ExtractStream<T>
            where
                T: crate::ByteCopy,
            {
                struct ByteCopyWrapper<T>(
                    receiver_hidden::Receiver,
                    ::core::marker::PhantomData<T>,
                );

                impl<T> Unpin for ByteCopyWrapper<T> {}

                impl<T: crate::ByteCopy> ::futures::Stream for ByteCopyWrapper<T> {
                    type Item = T;

                    fn poll_next<'a>(
                        mut self: ::core::pin::Pin<&'a mut Self>,
                        cx: &mut ::core::task::Context<'_>,
                    ) -> ::core::task::Poll<Option<Self::Item>> {
                        match core::pin::Pin::new(&mut self.0).poll_next(cx) {
                            ::core::task::Poll::Ready(Some((data, len))) => {
                                match crate::ByteCopy::extract(&data[..len]) {
                                    Some((data, _)) => ::core::task::Poll::Ready(Some(data)),
                                    _ => ::core::task::Poll::Pending,
                                }
                            }
                            _ => ::core::task::Poll::Pending,
                        }
                    }
                }

                ByteCopyWrapper(self, ::core::marker::PhantomData)
            }
        }
        impl ::futures::Stream for receiver_hidden::Receiver {
            type Item = ([u8; CONST_LEN], usize);

            fn poll_next<'a>(
                self: ::core::pin::Pin<&'a mut Self>,
                _: &mut ::core::task::Context<'_>,
            ) -> ::core::task::Poll<Option<Self::Item>> {
                unsafe {
                    if receiver_hidden::DATA_AVAILABLE {
                        receiver_hidden::DATA_AVAILABLE = false;
                        ::core::task::Poll::Ready(Some((DATA, receiver_hidden::LEN)))
                    } else {
                        ::core::task::Poll::Pending
                    }
                }
            }
        }
        #[allow(unused_unsafe)]
        unsafe {
            crate::INPUT_HANDLER = |len| {
                if receiver_hidden::DATA_AVAILABLE || len > CONST_LEN {
                    core::ptr::null_mut()
                } else {
                    receiver_hidden::DATA_AVAILABLE = true;
                    receiver_hidden::LEN = len;
                    DATA.as_mut_ptr()
                }
            };
        }
        receiver_data
    }};
}

#[cfg(all(target_arch = "wasm32", not(feature = "_simulator")))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
