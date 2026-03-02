use std::{sync::Arc, time::{Instant}};

use wgpu::{SurfaceError, util::{DeviceExt}};

use winit::{
    application::ApplicationHandler, dpi::{PhysicalPosition, PhysicalSize}, event::*, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::Window
};

use crate::{
    automata::{
        automata_rule::AutomataRule, 
        automata_state::AutomataState, 
        automata_renderer::AutomataRenderer,
        Automata
    }, 
    data::{Vertex}
};

// This will store the state of our "renderer"
pub struct GPUState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    //compute_pipeline: wgpu::ComputePipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    clear_color: wgpu::Color,

    //In practice will contain a reference to the bindgroup of a automata texturer
    //which is computed by a automata render
    texture_bind_group_layout: wgpu::BindGroupLayout,

    window: Arc<Window>,
}

impl GPUState {
    // We don't need this to be async right now,
    // but we will in the next tutorial
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let _ = window.request_inner_size(PhysicalSize::new(1600, 1600));
        let size = window.inner_size();
        //Instance: used to create other "GPU objects"
        //Surface: the GPU draws too this
        //Adapter: Interface with our actual GPU, get information
        //Device: Logical device allowing us to interact with physical device?
        //Queue: Command queue
        //Surface Config: Clear, how the surface creates, stores, and present/sync
        //surface textures?

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };


        //OBS: We never "apply" the surface config to the surface here

        let clear_color = wgpu::Color {
                            r: 0.7,
                            g: 0.2,
                            b: 0.5,
                            a: 1.0,
                          };

        let texture_bind_group_layout =
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                // This should match the filterable field of the
                                // corresponding Texture entry above.
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                        ],
                        label: Some("texture_bind_group_layout"),
                    });


        //Begin setup of render pipeline
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[], //What do these two do?? Something with buffers?
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), //Shader in shader.wgsl
                buffers: &[
                    Vertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // Target is our surface, note
                                                         // config.format is a surface format!
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // Not currently in use.
            multisample: wgpu::MultisampleState {
                count: 1, // Apparently very complicated, dont know what this does.
                mask: !0, 
                alpha_to_coverage_enabled: false, // Something with anti-aliasing, not used
            },
            multiview: None, // Dont know, but isn't used
            cache: None, // Cache shader compilation, not used.

        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(crate::data::VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(crate::data::INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );


        let num_indices = crate::data::INDICES.len() as u32;


        
        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            render_pipeline,
            clear_color,
            window,
            vertex_buffer,
            index_buffer,
            num_indices,
            texture_bind_group_layout,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }
    
    pub fn render(&mut self, texture_bindgroup: &wgpu::BindGroup) -> Result<(), SurfaceError> {
        //OBS: state.render() is called from window.request_redraw()
        //creating a "loop".
        self.window.request_redraw();


        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }


        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        //We now create a command buffer that will be put into the command queue...
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        //Extra block here to free _render_pass as it borrows encoder mutably
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, texture_bindgroup, &[]);
            //Use a slice for the buffer as the buffer could contain more than
            //one "set" of data.
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }


        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();


        Ok(())
    }

    fn handle_mouse_moved(&mut self, position: PhysicalPosition<f64>) {
        let x_prop = position.x / f64::from(self.window.inner_size().width);
        let y_prop = position.y / f64::from(self.window.inner_size().height);
        let clear_color = wgpu::Color {
                            r: x_prop,
                            g: y_prop,
                            b: 0.5,
                            a: 1.0,
                          };
        self.clear_color = clear_color;
    }
}


pub struct AutomataHandler {
    automata: Automata,
    automata_renderer: AutomataRenderer,
    even_frame: bool,
}

impl AutomataHandler {
    //Standard automata is a Conway automata now
    pub fn new(gpu: &GPUState) -> Self {
        let aut_width = 1024;
        let aut_height = 1024;
        let state = AutomataState::new(&gpu.device, aut_width, aut_height);
        let states = vec![state];
        let mut rules: Vec<AutomataRule> = vec![];
        rules.push(AutomataRule::conway_rule(&states, 0, &gpu.device));
        let automata = Automata {
            states,
            rules,
        };

        let automata_renderer = AutomataRenderer::new(&automata.states, &gpu.texture_bind_group_layout, aut_width, aut_height, &gpu.device, &gpu.queue);

        return AutomataHandler {automata, automata_renderer, even_frame: true};
    }

    pub fn update(&mut self, gpu: &GPUState) {
        let device = &gpu.device;
        let queue = &gpu.queue;

        self.automata.update(device, queue, self.even_frame);
        self.automata_renderer.update_texture(device, queue, self.even_frame);
        self.even_frame = !self.even_frame;
    }

    fn temp_update_conway_prm(&mut self, gpu: &GPUState) {
        let width = self.automata.states[0].width;
        let height = self.automata.states[0].height;
        let new_prm = vec![width, height, 30, 12, 13];
        self.automata.rules[0].update_prm_bindgroup(new_prm, &gpu.queue);
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
            WindowEvent::CursorMoved { device_id: _, position } => self.gpu_state.handle_mouse_moved(position),
            _ => {}
        }
    }

    fn render_app(&mut self) {
        let aut_texture = &self.automata_handler.automata_renderer.texture_bind_group;
        match self.gpu_state.render(aut_texture) {
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
            (KeyCode::KeyU, true) => self.automata_handler.temp_update_conway_prm(&self.gpu_state),
            _ => {}
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

            let automata_handler = AutomataHandler::new(&gpu_state);

            *self = App::Initialized(
                        InitializedApp { 
                            gpu_state, 
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
