use crate::{
    util::{clear, wait, Delay},
    STRIP,
};

Delay!(
    SHORT for 50,
    OTHER_SHORT for 50
);

#[allow(dead_code)]
pub unsafe fn pulse() {
    clear();
    for i in &mut STRIP[0..36] {
        *i = [255, 255, 0];
    }
    for i in &mut STRIP[36..=74] {
        *i = [255, 0, 255];
    }
    wait!(SHORT);

    for i in &mut STRIP[0..36] {
        *i = [255, 0, 255];
    }
    for i in &mut STRIP[36..=74] {
        *i = [255, 255, 0];
    }
    wait!(OTHER_SHORT);
    SHORT.reset();
    OTHER_SHORT.reset();
}
