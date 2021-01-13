use core::future::Future;

use crate::{
    projection::LinearSpatialExt,
    rand,
    util::{gradient, interpolate::Linear},
    LedExt,
};

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn gradients() -> Program {
    async move {
        let mut leds = crate::leds();
        let gradient = {
            let len = leds.len();
            move || gradient::<Linear>(*rand::color().normalize(), *rand::color().normalize(), len)
        };
        leds.fill_from(gradient());
        loop {
            leds.fade_to::<_, Linear>(gradient(), 100).await;
        }
    }
}
