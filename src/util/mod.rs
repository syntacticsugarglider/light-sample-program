use core::{
    future::Future,
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

pub fn lerp(a: [u8; 3], b: [u8; 3], factor: f32) -> [u8; 3] {
    let delta = [
        b[0] as f32 - a[0] as f32,
        b[1] as f32 - a[1] as f32,
        b[2] as f32 - a[2] as f32,
    ];
    [
        (a[0] as f32 + (delta[0] * factor)) as u8,
        (a[1] as f32 + (delta[1] * factor)) as u8,
        (a[2] as f32 + (delta[2] * factor)) as u8,
    ]
}

#[derive(Clone)]
pub struct Gradient {
    state: [f32; 3],
    delta: [f32; 3],
    left: usize,
}

impl Iterator for Gradient {
    type Item = [u8; 3];

    fn next(&mut self) -> Option<Self::Item> {
        if self.left == 0 {
            return None;
        }
        let ret = [
            self.state[0] as u8,
            self.state[1] as u8,
            self.state[2] as u8,
        ];
        self.left -= 1;
        self.state[0] += self.delta[0];
        self.state[1] += self.delta[1];
        self.state[2] += self.delta[2];
        Some(ret)
    }
}

pub fn gradient(start: [u8; 3], end: [u8; 3], steps: usize) -> Gradient {
    let steps = steps - 1;
    Gradient {
        state: [start[0] as f32, start[1] as f32, start[2] as f32],
        delta: [
            (end[0] as f32 - start[0] as f32) / steps as f32,
            (end[1] as f32 - start[1] as f32) / steps as f32,
            (end[2] as f32 - start[2] as f32) / steps as f32,
        ],
        left: steps + 1,
    }
}

#[macro_export]
macro_rules! gradient {
    ($a:expr => $b:expr, $len:expr; $($c:expr => $d:expr, $e:expr;)*) => {
        crate::util::gradient($a, $b, $len) $(.chain(crate::util::gradient($c, $d, $e)))*
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

#[allow(non_upper_case_globals)]
pub mod colors;
