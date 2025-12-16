pub mod automata;
pub mod app;
pub mod data;

use crate::app::App;



use winit::{
    event_loop::EventLoop
};


pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new();
    //Add automata to app
    event_loop.run_app(&mut app)?;

    Ok(())
}



fn main() {
    match run() {
       Err(_) => {},
       Ok(_) => {}
    }
}
