// THIS FILE IS AUTO-GENERATED, DO NOT EDIT 

mod gradients;
#[cfg(feature = "gradients")]
pub use gradients::{gradients as program, Program};
mod example;
#[cfg(feature = "example")]
pub use example::{example as program, Program};
mod oops;
#[cfg(feature = "oops")]
pub use oops::{oops as program, Program};
mod twinkle;
#[cfg(feature = "twinkle")]
pub use twinkle::{twinkle as program, Program};
mod flow;
#[cfg(feature = "flow")]
pub use flow::{flow as program, Program};

#[cfg(not(any(
  feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow"
)))]
mod _editor_shim {
    pub type Program = impl core::future::Future<Output = ()>;

    #[allow(dead_code)]
    pub unsafe fn program() -> Program {
        async move { panic!() }
    }
}

#[cfg(not(any(
  feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow"
)))]
pub use _editor_shim::*;
