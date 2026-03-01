use wgpu::{util::{BufferInitDescriptor, DeviceExt}};
use rand::Rng;


pub struct AutomataState {
    pub width: u32,
    pub height: u32,
    pub work_group_size: u32, //Assume work groups are square
    pub automata_bindgroup_even: wgpu::BindGroup,
    pub automata_bindgroup_odd: wgpu::BindGroup,
    pub automata_bindgroup_layout: wgpu::BindGroupLayout,
    pub even_buffer: wgpu::Buffer,
    pub odd_buffer: wgpu::Buffer,
}

impl AutomataState {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> AutomataState {
        let mut rng = rand::rng();
        let input_data: Vec<f32> = (0..(height*width))
                                   .map(|_x| rng.random_range(0..2) as f32).collect(); 
                                    //faster to use a uniform dist,
                                    //but doesnt matter


        let even_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("input"),
            contents: bytemuck::cast_slice(&input_data),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let odd_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output"),
            size: even_buffer.size(),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let compute_bind_group_layout = 
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::COMPUTE,
                                ty: wgpu::BindingType::Buffer { 
                                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                                    has_dynamic_offset: false, //Maybe should be true?
                                    min_binding_size: None,
                                },
                                count: None,
                            },                   
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::COMPUTE,
                                ty: wgpu::BindingType::Buffer { 
                                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                                    has_dynamic_offset: false, //Maybe should be true?
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                        label: Some("compute_bind_group_layout"),
                    });

        let compute_bind_group_even = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: even_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: odd_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_bind_group_odd = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: odd_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: even_buffer.as_entire_binding(),
                },
            ],
        });

        return AutomataState {
            width, 
            height,
            work_group_size: 16,
            automata_bindgroup_even: compute_bind_group_even,
            automata_bindgroup_odd: compute_bind_group_odd, 
            automata_bindgroup_layout: compute_bind_group_layout,
            even_buffer,
            odd_buffer,
        }
    }
}

