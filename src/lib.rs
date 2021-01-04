#![no_std]

use core::panic::PanicInfo;

static mut SEED: u32 = 123456789;
const M: u32 = 4294967295;
const M_F: f32 = M as f32;
const M_2: u32 = M / 2;
const A: u32 = 1103515245;
const C: u32 = 12345;

unsafe fn rand() -> u32 {
    SEED = (A * SEED + C) % M;
    SEED
}

unsafe fn rand_bool(threshold: Option<f32>) -> bool {
    if let Some(threshold) = threshold {
        rand() > (M_F * (1. - threshold)) as u32
    } else {
        rand() > M_2
    }
}

static mut FRAMES: u8 = 0;
static mut CO_BUFFER: [[u8; 3]; 75] = [[0u8; 3]; 75];
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

#[derive(Clone, Copy)]
pub struct Twinkle {
    brightness: f32,
    state: bool,
    rate: f32,
    extra_co: u8,
}

impl Twinkle {
    fn update(&mut self) -> Option<u8> {
        if self.state {
            self.brightness -= self.rate;
            if self.brightness <= 0. {
                return None;
            }
            Some(self.brightness as u8)
        } else {
            self.brightness += self.rate;
            if self.brightness >= 255. {
                self.brightness = 255.;
                self.state = true;
            }
            Some(self.brightness as u8)
        }
    }
}

static mut ACTIVE: &'static mut [Option<Twinkle>; 75] = &mut [None; 75];

#[no_mangle]
extern "C" fn entry() -> *mut Output {
    unsafe {
        OUTPUT.data.buffered.2 = CO_BUFFER.as_mut_ptr();
        for color in &mut CO_BUFFER[..] {
            *color = [0, 0, 0];
        }
        let mut idx = 0;
        for twinkle in &mut ACTIVE[..] {
            let mut r = false;
            *CO_BUFFER.get_unchecked_mut(idx) = if let Some(twinkle) = twinkle {
                if let Some(c) = twinkle.update() {
                    let b = (c as u32 * twinkle.extra_co as u32) / 255;
                    [b as u8, b as u8, c]
                } else {
                    r = true;
                    [0, 0, 0]
                }
            } else {
                if rand_bool(Some(0.01)) {
                    *twinkle = Some(Twinkle {
                        brightness: 0.,
                        rate: (rand() as f32 / M_F) * 10. + 6.,
                        state: false,
                        extra_co: ((rand() as f32 / M_F) * 255.) as u8,
                    })
                }
                [0, 0, 0]
            };
            if r {
                *twinkle = None;
            }
            idx += 1;
        }
        FRAMES += 1;
        if FRAMES > 74 {
            FRAMES = 0;
        }
    };
    unsafe { &mut OUTPUT }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
