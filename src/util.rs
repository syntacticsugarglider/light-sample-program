use crate::STRIP;

pub unsafe fn clear() {
    for color in &mut STRIP[..] {
        *color = [0, 0, 0];
    }
}

pub struct Delay {
    counter: u32,
    duration: u32,
}

impl Delay {
    pub const fn new(duration: u32) -> Self {
        Delay {
            counter: 0,
            duration,
        }
    }

    #[doc(hidden)]
    pub fn step(&mut self) -> bool {
        if self.counter < self.duration {
            self.counter += 1;
            false
        } else {
            true
        }
    }

    pub fn reset(&mut self) {
        self.counter = 0;
    }
}

#[macro_export]
macro_rules! wait {
    ($name:expr) => {
        #[allow(unused_unsafe)]
        unsafe {
            let delay: &mut crate::util::Delay = &mut $name;
            if !crate::util::Delay::step(delay) {
                return;
            }
        };
    };
}

#[macro_export]
macro_rules! Delay {
    ($($name:ident for $length:literal),+) => {
        $(
            static mut $name: Delay = Delay::new($length);
        )+
    };
}

pub use crate::{wait, Delay};
