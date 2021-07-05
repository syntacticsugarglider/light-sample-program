use core::future::Future;

use crate::{
    projection::{
        Cartesian2dExt, CartesianRange, CartesianSpatialExt, LinearSpatialExt, Spatial, SpatialExt,
        SwitchbackGrid,
    },
    util::delay,
};

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn valentine() -> Program {
    async move {
        let mut leds = crate::leds();
        let mut leds = crate::leds().project(
            SwitchbackGrid::<_, u8>::new(8)
                .constrain_height(13)
                .unwrap()
                .flip_y()
                .unwrap(),
        );

        let mut h = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((0, 2)..=(4, 2)).unwrap().fill(co).unwrap();
            leds.range((0, 0)..=(0, 5)).unwrap().fill(co).unwrap();
            leds.range((4, 0)..=(4, 5)).unwrap().fill(co).unwrap();
        };

        let mut a = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((1, 0)..=(3, 0)).unwrap().fill(co).unwrap();
            leds.range((0, 1)..=(0, 5)).unwrap().fill(co).unwrap();
            leds.range((4, 1)..=(4, 5)).unwrap().fill(co).unwrap();
            leds.range((1, 2)..=(3, 2)).unwrap().fill(co).unwrap();
        };

        let mut p = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((0, 0)..=(3, 0)).unwrap().fill(co).unwrap();
            leds.range((0, 1)..=(0, 5)).unwrap().fill(co).unwrap();
            leds.range((1, 2)..=(3, 2)).unwrap().fill(co).unwrap();
            leds[(4, 1)] = co;
        };

        let mut y = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((0, 0)..=(0, 1)).unwrap().fill(co).unwrap();
            leds[(1, 2)] = co;
            leds[(3, 2)] = co;
            leds.range((4, 0)..=(4, 1)).unwrap().fill(co).unwrap();
            leds.range((2, 3)..=(2, 5)).unwrap().fill(co).unwrap();
        };

        let mut v = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((0, 0)..=(0, 2)).unwrap().fill(co).unwrap();
            leds.range((4, 0)..=(4, 2)).unwrap().fill(co).unwrap();
            leds.range((1, 3)..=(1, 4)).unwrap().fill(co).unwrap();
            leds.range((3, 3)..=(3, 4)).unwrap().fill(co).unwrap();
            leds[(2, 5)] = co;
        };

        let mut l = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((0, 0)..=(0, 5)).unwrap().fill(co).unwrap();
            leds.range((1, 5)..=(4, 5)).unwrap().fill(co).unwrap();
        };

        let mut e = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((0, 0)..=(4, 0)).unwrap().fill(co).unwrap();
            leds.range((0, 1)..=(0, 5)).unwrap().fill(co).unwrap();
            leds.range((1, 5)..=(4, 5)).unwrap().fill(co).unwrap();
            leds.range((1, 2)..=(3, 2)).unwrap().fill(co).unwrap();
        };

        let mut n = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((0, 0)..=(0, 5)).unwrap().fill(co).unwrap();
            leds.range((4, 0)..=(4, 5)).unwrap().fill(co).unwrap();
            leds[(1, 1)] = co;
            leds[(2, 2)] = co;
            leds[(3, 3)] = co;
            leds[(3, 4)] = co;
        };

        let mut t = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((0, 0)..=(4, 0)).unwrap().fill(co).unwrap();
            leds.range((2, 1)..=(2, 5)).unwrap().fill(co).unwrap();
        };

        let mut i = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((2, 0)..=(2, 5)).unwrap().fill(co).unwrap();
        };

        let mut s = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((1, 0)..=(4, 0)).unwrap().fill(co).unwrap();
            leds[(0, 1)] = co;
            leds.range((1, 2)..=(3, 2)).unwrap().fill(co).unwrap();
            leds.range((4, 3)..=(4, 4)).unwrap().fill(co).unwrap();
            leds.range((0, 5)..=(3, 5)).unwrap().fill(co).unwrap();
        };

        let mut d = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((0, 0)..=(3, 0)).unwrap().fill(co).unwrap();
            leds.range((0, 1)..=(0, 5)).unwrap().fill(co).unwrap();
            leds.range((1, 5)..=(3, 5)).unwrap().fill(co).unwrap();
            leds.range((4, 1)..=(4, 4)).unwrap().fill(co).unwrap();
        };

        let mut o = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((1, 0)..=(3, 0)).unwrap().fill(co).unwrap();
            leds.range((0, 1)..=(0, 4)).unwrap().fill(co).unwrap();
            leds.range((1, 5)..=(3, 5)).unwrap().fill(co).unwrap();
            leds.range((4, 1)..=(4, 4)).unwrap().fill(co).unwrap();
        };

        let mut z = |co, leds: &mut CartesianRange<_, _>| {
            leds.range((0, 0)..=(4, 0)).unwrap().fill(co).unwrap();
            leds[(4, 1)] = co;
            leds[(3, 2)] = co;
            leds[(2, 3)] = co;
            leds[(1, 4)] = co;
            leds.range((0, 5)..=(4, 5)).unwrap().fill(co).unwrap();
        };

        let mut heart = |co, leds: &mut CartesianRange<_, _>| {
            leds[(1, 7)] = co;
            leds[(2, 7)] = co;
            leds[(3, 8)] = co;
            leds[(4, 7)] = co;
            leds[(5, 7)] = co;
            leds[(0, 8)] = co;
            leds[(0, 9)] = co;
            leds[(1, 10)] = co;
            leds[(2, 11)] = co;
            leds[(3, 12)] = co;
            leds[(4, 11)] = co;
            leds[(5, 10)] = co;
            leds[(6, 9)] = co;
            leds[(6, 8)] = co;
        };

        let mut tleds = leds.range((1, 0)..=(7, 6)).unwrap();
        let co = [255, 0, 128];

        heart(co, &mut tleds);

        let mut leds = leds.range((0, 0)..=(7, 6)).unwrap();

        loop {
            h(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            a(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            p(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            p(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            y(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            v(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            a(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            l(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            e(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            n(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            t(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            i(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            n(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            e(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            s(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            d(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            a(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            y(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(20).await;
            l(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            o(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            v(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            e(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            i(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            z(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            z(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(10).await;
            y(co, &mut leds);
            delay(10).await;
            leds.clear();
            delay(20).await;
        }
    }
}
