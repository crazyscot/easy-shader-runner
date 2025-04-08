use structopt::StructOpt;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen::{self, prelude::*};

mod controller;

#[derive(StructOpt, Clone, Copy)]
pub struct Options {
    /// Starts in debug mode and with speed set to 0
    #[structopt(short, long)]
    debug: bool,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    let options = Options::from_args();
    let controller = controller::Controller::new(&options);
    runner::start(
        controller,
        #[cfg(not(target_arch = "wasm32"))]
        "shader/shader",
    );
}
