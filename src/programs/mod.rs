// THIS FILE IS AUTO-GENERATED, DO NOT EDIT 

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
  feature = "example", feature = "oops", feature = "twinkle", feature = "flow"
)))]
mod _editor_shim {
    pub type Program = impl core::future::Future<Output = ()>;

    #[allow(dead_code)]
    pub unsafe fn program() -> Program {
        async move { panic!() }
    }
}
pub use _editor_shim::*;
