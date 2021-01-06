use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::STRIP;

pub unsafe fn clear() {
    for color in &mut STRIP[..] {
        *color = [0, 0, 0];
    }
}

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
