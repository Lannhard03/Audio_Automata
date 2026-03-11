pub mod automata_rule;
pub mod automata_state;
pub mod automata_renderer;

use crate::app::UpdateInfo;
use crate::automata::automata_renderer::AutomataTexturer;
use crate::gpu_state::GPUState;
use crate::automata::automata_state::AutomataState;
use crate::automata::automata_rule::{AutomataInteraction, ConwayInteraction, RainInteraction};

pub struct AutomataHandler {
    ecosystem: Ecosystem,
    automata_renderer: AutomataTexturer,
    update_info: UpdateInfo,
}

impl AutomataHandler {
    //Standard automata is a Conway automata now
    pub fn new(gpu: &GPUState) -> Self {
        let width = 1024;
        let height = 1024;
        let ecosystem = Ecosystem::new_conway_automata(width, height, gpu);
        //let ecosystem = Ecosystem::new_spectral_rain_aut(width, height, gpu);
        let states = ecosystem.get_state_ref();
        let automata_renderer = AutomataTexturer::new(states, width as u32,
                                                      height as u32, &gpu.device, &gpu.queue);
        let update_info = UpdateInfo {frame: 0, etc: 0, key_presses: Vec::from([])}; //Temp values for now

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


pub struct Ecosystem {
    states: Vec<AutomataState>,
    interactions: Vec<Box<dyn AutomataInteraction>>,
}

impl Ecosystem {
    pub fn get_state_ref(&self) -> &Vec<AutomataState> {
        return &self.states;
    }

    pub fn update(&mut self, update_info: &UpdateInfo, device: &wgpu::Device, queue: &wgpu::Queue) {
        for state in self.states.iter_mut() {
            state.even_frame = !state.even_frame;
        }

        for interaction in self.interactions.iter_mut() {
            interaction.update_prm(update_info, queue);
        }

        //Do this here so that a rules are applied in the same compute pass,
        //don't actually know if this is a good or bad idea?
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());

            for interaction in self.interactions.iter() {
                interaction.apply_interaction(&self.states, &mut compute_pass);
            }
        }
        queue.submit([encoder.finish()]);

    }

    fn std_bindgroup_layout(gpu: &GPUState) -> wgpu::BindGroupLayout {
        let compute_bind_group_layout = 
            gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        return compute_bind_group_layout;
    }

    pub fn new_conway_automata(width: u32, height: u32, gpu: &GPUState) -> Self {
        let bind_group_layout = Self::std_bindgroup_layout(gpu);
        let state = AutomataState::new(&gpu.device, width, height);
        let states = vec![state];

        let mut interactions: Vec<Box<dyn AutomataInteraction>> = vec![];
        let state_indicies: Vec<usize> = vec![0];
        let conway_interaction = Box::new(ConwayInteraction::new(width, height, state_indicies, 
                                   bind_group_layout, &gpu.device));

        interactions.push(conway_interaction);

        Ecosystem {
            states,
            interactions,
        }
    }

    
    pub fn new_spectral_rain_aut(width: u32, height: u32, gpu: &GPUState) -> Self {
        let bind_group_layout = Self::std_bindgroup_layout(gpu);
        let state = AutomataState::new(&gpu.device, width, height);
        let states = vec![state];

        let mut interactions: Vec<Box<dyn AutomataInteraction>> = vec![];
        let state_indicies: Vec<usize> = vec![0];
        let rain_interaction = Box::new(RainInteraction::new(width, height, state_indicies, 
                                   bind_group_layout, &gpu.device));

        interactions.push(rain_interaction);

        Ecosystem {
            states,
            interactions,
        }
    }
    
}
