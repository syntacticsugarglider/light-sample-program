use crate::{
    rand::{rand_bool, rand_logit, rand_u8},
    util::clear,
    STRIP,
};

#[derive(Clone, Copy)]
pub struct Twinkle {
    brightness: f32,
    state: bool,
    rate: f32,
    extra_co: u8,
}

impl Twinkle {
    fn update(&mut self) -> Option<(u8, u8)> {
        if self.state {
            self.brightness -= self.rate;
            if self.brightness <= 0. {
                return None;
            }
            Some((
                self.brightness as u8,
                ((self.brightness / 255.) * self.extra_co as f32) as u8,
            ))
        } else {
            self.brightness += self.rate;
            if self.brightness >= 255. {
                self.brightness = 255.;
                self.state = true;
            }
            Some((
                self.brightness as u8,
                ((self.brightness / 255.) * self.extra_co as f32) as u8,
            ))
        }
    }
}

static mut ACTIVE: &'static mut [Option<Twinkle>; 75] = &mut [None; 75];

#[allow(dead_code)]
pub unsafe fn twinkle() {
    clear();

    for (twinkle, color) in (&mut ACTIVE[..]).into_iter().zip(STRIP.iter_mut()) {
        let mut r = false;
        *color = if let Some(twinkle) = twinkle {
            if let Some((c, b)) = twinkle.update() {
                [b, c, b]
            } else {
                r = true;
                [0, 0, 0]
            }
        } else {
            if rand_bool(Some(0.01)) {
                *twinkle = Some(Twinkle {
                    brightness: 0.,
                    rate: rand_logit() * 10. + 6.,
                    state: false,
                    extra_co: rand_u8(),
                })
            }
            [0, 0, 0]
        };
        if r {
            *twinkle = None;
        }
    }
}
