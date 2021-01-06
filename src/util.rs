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
