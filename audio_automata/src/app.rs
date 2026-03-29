use std::{sync::Arc, time::{Instant}};

use winit::{
    application::ApplicationHandler,
    event::*, 
    event_loop::ActiveEventLoop, 
    keyboard::{KeyCode, PhysicalKey}, 
    window::Window
};

use crate::{
    automata::{
        Ecosystem, automata_renderer::{AnnihilationAutomataTexturer, AutomataTexturer}
    }, 
    gpu_state::{GPUState, Renderer},
};

pub struct UpdateInfo {
    pub frame: u32,
    pub key_presses: Vec<KeyCode>, 
    pub etc: u32, //More fields here
}

pub struct AutomataHandler {
    ecosystem: Ecosystem,
    automata_renderer: Box<dyn AutomataTexturer>,
    update_info: UpdateInfo,
}

impl AutomataHandler {
    //Standard automata is a Conway automata now
    pub fn new(gpu: &GPUState) -> Self {
        let width = 1024;
        let height = 1024;
        //let ecosystem = Ecosystem::new_conway_automata(width, height, gpu);
        //let ecosystem = Ecosystem::new_spectral_rain_aut(width, height, gpu);
        let ecosystem = Ecosystem::new_annihilation_aut(width, height, gpu);
        let states = ecosystem.get_state_ref();
        //let automata_renderer = Box::new(BasicAutomataTexturer::new(&states[2], width as u32,
        //                                              height as u32, &gpu.device, &gpu.queue));
        let automata_renderer = Box::new(AnnihilationAutomataTexturer::new(states, width as u32,
                                                      height as u32, &gpu.device, &gpu.queue));
        let update_info = UpdateInfo {frame: 0, etc: 0, key_presses: Vec::from([])};

        return AutomataHandler {ecosystem, automata_renderer, update_info};
    }

    pub fn update(&mut self, gpu: &GPUState) {
        let device = &gpu.device;
        let queue = &gpu.queue;

        self.ecosystem.update(&self.update_info, device, queue);
        self.automata_renderer.update_texture(device, queue);

        self.update_info.key_presses.clear();
        self.update_info.frame += 1;
    }

}

pub enum App {
    Uninitialized,
    Initialized(InitializedApp),
}

impl App {
    pub fn new() -> Self {
        return Self::Uninitialized;
    }

}

pub struct InitializedApp {
    gpu_state: GPUState,
    renderer: Renderer,
    automata_handler: AutomataHandler,
    last_render_time: Instant,
}

impl InitializedApp {
    fn update(&mut self) {
        self.automata_handler.update(&self.gpu_state);
    }

    pub fn window_event(&mut self, event: WindowEvent, event_loop: &ActiveEventLoop) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.gpu_state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now - self.last_render_time;
                if dt.as_millis() >= 30 {
                    println!("{}", dt.as_millis());
                    self.update();
                    self.last_render_time = now;
                }
                self.render_app();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => self.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::CursorMoved { device_id: _, position } => 
                self.gpu_state.handle_mouse_moved(position),
            _ => {}
        }
    }

    fn render_app(&mut self) {
        let aut_texture = &self.automata_handler.automata_renderer.get_data().
                                texture.texture_bind_group;
        match self.renderer.render(&self.gpu_state, aut_texture) {
            Ok(_) => {}
            // Reconfigure the surface if it's lost or outdated
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let size = self.gpu_state.window.inner_size();
                self.gpu_state.resize(size.width, size.height);
            }
            Err(e) => {
                log::error!("Unable to render {}", e);
            }
        }
    }



    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            (code, true) => self.automata_handler.update_info.key_presses.push(code),
            _ => (),
        }
    }


}

impl ApplicationHandler<GPUState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        if let App::Uninitialized = self {
            let mut window_attributes = Window::default_attributes();
            //Two unsafe unwraps here, but both crucial to program running at all
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            let gpu_state = pollster::block_on(GPUState::new(window)).unwrap();
            let mut automata_handler = AutomataHandler::new(&gpu_state);

            let tex_bindgroup_layout = &automata_handler.automata_renderer.get_data().
                                        texture.texture_bind_group_layout;          
            let renderer = Renderer::new(&gpu_state, tex_bindgroup_layout);

            *self = App::Initialized(
                        InitializedApp { 
                            gpu_state, 
                            renderer,
                            automata_handler,
                            last_render_time: Instant::now(),
                        }
                    )
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let App::Initialized(app) = self {
            app.window_event(event, event_loop);
        }
    }
}
