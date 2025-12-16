use std::{sync::Arc, time::{Instant}};

use wgpu::{SurfaceError, util::{DeviceExt}};

use winit::{
    application::ApplicationHandler, dpi::PhysicalPosition, event::*, event_loop::{ActiveEventLoop}, keyboard::{KeyCode, PhysicalKey}, window::Window
};

use crate::data::{ComputeTexture, Vertex};

// This will store the state of our "renderer"
pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    clear_color: wgpu::Color,
    compute_texture: ComputeTexture,
    even_frame: bool,
    window: Arc<Window>,
}

impl State {
    // We don't need this to be async right now,
    // but we will in the next tutorial
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
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

        let compute_texture = ComputeTexture::new(&device, &queue, 10, 10).unwrap();

        //OBS: We never "apply" the surface config to the surface here

        let clear_color = wgpu::Color {
                            r: 0.7,
                            g: 0.2,
                            b: 0.5,
                            a: 1.0,
                          };

        //Begin setup of render pipeline
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline layout"),
            bind_group_layouts: &[&compute_texture.texture_bind_group_layout],
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

        let compute_shader = device.create_shader_module(wgpu::include_wgsl!("automata.wgsl"));
        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline layout"),
            bind_group_layouts: &[&compute_texture.compute_bind_group_layout],
            push_constant_ranges: &[], //What does this one do?? Something with buffers?
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("cp_main"), //Correct name?
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        
        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            render_pipeline,
            compute_pipeline,
            clear_color,
            window,
            vertex_buffer,
            index_buffer,
            num_indices,
            compute_texture,
            even_frame: true,
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
    
    pub fn render(&mut self) -> Result<(), SurfaceError> {
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
            render_pass.set_bind_group(0, &self.compute_texture.texture_bind_group, &[]);
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

    fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
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

    fn update(&mut self) {

        //Compute
        let num_dispatches_x = self.compute_texture.width.div_ceil(16) as u32;
        let num_dispatches_y = self.compute_texture.height.div_ceil(16) as u32;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
           label: Some("Compute Encoder"),
        });

        {
           let mut compute_pass = encoder.begin_compute_pass(&Default::default());
           compute_pass.set_pipeline(&self.compute_pipeline);
           if self.even_frame {
               compute_pass.set_bind_group(0, &self.compute_texture.compute_bind_group_even, &[]);
               self.even_frame = false;
           } else {
               compute_pass.set_bind_group(0, &self.compute_texture.compute_bind_group_odd, &[]);
               self.even_frame = true;
           }
           compute_pass.dispatch_workgroups(num_dispatches_x, num_dispatches_y, 1);
        }
        self.queue.submit([encoder.finish()]);
    }

}






pub struct App {
    state: Option<State>,
    last_render_time: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            last_render_time: Instant::now(),
        }
    }
}


impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.state = Some(pollster::block_on(State::new(window)).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now - self.last_render_time;
                if dt.as_millis() >= 30 {
                    //println!("{}", dt.as_millis());
                    state.update();
                    self.last_render_time = now;
                }
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::CursorMoved { device_id: _, position } => state.handle_mouse_moved(position),
            _ => {}
        }
    }
}
