use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use pin_project::pin_project;

pub(super) static mut CURRENT_TICK: bool = false;
pub(super) static mut TICKS_ELAPSED: u64 = 0;

pub fn next_tick() -> NextTick {
    NextTick::new()
}

pub fn delay(ticks: u64) -> Delay {
    Delay::new(ticks)
}

pub struct Delay {
    end: u64,
}

impl Delay {
    fn new(duration: u64) -> Self {
        Delay {
            end: unsafe { TICKS_ELAPSED } + duration,
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if self.end > unsafe { TICKS_ELAPSED } {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

pub struct NextTick {
    initial: bool,
    complete: bool,
}

impl NextTick {
    fn new() -> Self {
        unsafe {
            NextTick {
                initial: CURRENT_TICK,
                complete: false,
            }
        }
    }
}

impl Future for NextTick {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if self.complete {
            Poll::Ready(())
        } else {
            if self.initial == unsafe { CURRENT_TICK } {
                Poll::Pending
            } else {
                self.complete = true;
                Poll::Ready(())
            }
        }
    }
}

#[pin_project]
pub struct Select<T, U>(#[pin] T, #[pin] U);

impl<T: Future<Output = ()>, U: Future<Output = ()>> Future for Select<T, U> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut pending = false;
        if let Poll::Pending = this.0.poll(cx) {
            pending = true;
        }
        if let Poll::Pending = this.1.poll(cx) {
            pending = true;
        }
        if pending {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

pub fn select<T: Future<Output = ()>, U: Future<Output = ()>>(a: T, b: U) -> Select<T, U> {
    Select(a, b)
}

#[derive(Clone)]
pub struct Gradient<I> {
    start: [u8; 3],
    end: [u8; 3],
    max: usize,
    current: usize,
    _marker: PhantomData<I>,
}

impl<I: Interpolate> Iterator for Gradient<I> {
    type Item = [u8; 3];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.max {
            return None;
        }
        let ratio = self.current as f32 / self.max as f32;
        let ret = I::interpolate(&self.start, &self.end, ratio);
        self.current += 1;
        Some(ret)
    }
}

pub fn gradient<I: Interpolate>(start: [u8; 3], end: [u8; 3], max: usize) -> Gradient<I> {
    Gradient {
        start,
        end,
        max,
        current: 0,
        _marker: PhantomData,
    }
}

#[macro_export]
macro_rules! gradient {
    ($a:expr => $b:expr, $interp:ty, $len:expr; $($c:expr => $d:expr, $e:ty, $f:expr;)*) => {
        crate::util::gradient::<$interp>($a, $b, $len) $(.chain(crate::util::gradient::<$e>($c, $d, $f)))*
    };
}

pub use crate::gradient;

#[macro_export]
macro_rules! select {
    ($a:expr, $($c:expr),+) => {{
        trait SelectExt {
            fn _ext_select<U: Future<Output = ()>>(self, other: U) -> crate::util::Select<Self, U> where Self: Sized;
        }
        impl<T: Future<Output = ()>> SelectExt for T {
            fn _ext_select<U: Future<Output = ()>>(self, other: U) -> crate::util::Select<T, U> {
                crate::util::select(self, other)
            }
        }
        $a$(._ext_select($c))+
    }};
}

pub use crate::select;

use self::interpolate::Interpolate;

#[allow(non_upper_case_globals)]
pub mod colors;
pub mod interpolate;
