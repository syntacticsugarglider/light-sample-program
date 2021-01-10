use core::future::Future;

use crate::{
    projection::{
        Cartesian2d, Cartesian2dExt, CartesianSpatialExt, Spatial, SpatialExt, SwitchbackGrid,
    },
    util::delay,
};

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn grid() -> Program {
    async move {
        let mut leds =
            crate::leds().project(SwitchbackGrid::<_, u8>::new(8).constrain_height(9).unwrap());

        loop {
            for x in 0..leds.space().width().unwrap() - 1 {
                for y in 0..leds.space().height().unwrap() - 1 {
                    leds.clear();
                    let mut range = leds.range((x, y)..(x + 2, y + 2)).unwrap();
                    range.fill([255, 255, 255]).unwrap();
                    delay(2).await;
                }
            }
        }
    }
}
