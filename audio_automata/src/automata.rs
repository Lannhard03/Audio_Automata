pub mod automata_rule;
pub mod automata_state;
pub mod automata_renderer;

use crate::app::UpdateInfo;
use crate::gpu_state::GPUState;
use crate::automata::automata_state::AutomataState;
use crate::automata::automata_rule::{AnnihilationInteraction, AutomataInteraction, ConwayInteraction, RainInteraction};

pub struct Ecosystem {
    states: Vec<AutomataState>,
    interactions: Vec<Box<dyn AutomataInteraction>>,
}

impl Ecosystem {
    pub fn get_state_ref(&self) -> &Vec<AutomataState> {
        return &self.states;
    }

    pub fn update(&mut self, update_info: &UpdateInfo, device: &wgpu::Device, queue: &wgpu::Queue) {
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
                interaction.apply_interaction(&mut self.states, &mut compute_pass);
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
        let state = AutomataState::new(&gpu.device, width, height, true);
        let states = vec![state];

        let mut interactions: Vec<Box<dyn AutomataInteraction>> = vec![];
        let state_indicies: Vec<usize> = vec![0];
        let conway_interaction = Box::new(ConwayInteraction::new(width, height, state_indicies, [3, 12, 13],
                                   bind_group_layout, &gpu.device));

        interactions.push(conway_interaction);

        Ecosystem {
            states,
            interactions,
        }
    }

    
    pub fn new_spectral_rain_aut(width: u32, height: u32, gpu: &GPUState) -> Self {
        let bind_group_layout = Self::std_bindgroup_layout(gpu);
        let state = AutomataState::new(&gpu.device, width, height, false);
        let states = vec![state];

        let mut interactions: Vec<Box<dyn AutomataInteraction>> = vec![];
        let state_indicies: Vec<usize> = vec![0];
        let rain_interaction = Box::new(RainInteraction::new(width, height, false, state_indicies, 
                                   bind_group_layout, &gpu.device));

        interactions.push(rain_interaction);

        Ecosystem {
            states,
            interactions,
        }
    }

    pub fn new_annihilation_aut(width: u32, height: u32, gpu: &GPUState) -> Self {
        let state1 = AutomataState::new(&gpu.device, width, height, false);
        let state2 = AutomataState::new(&gpu.device, width, height, false);
        let state3 = AutomataState::new(&gpu.device, width, height, false);
        let states = vec![state1, state2, state3];

        let mut interactions: Vec<Box<dyn AutomataInteraction>> = vec![];

        let state_indicies: Vec<usize> = vec![0, 1, 2];
        let bindgroup_layout = Self::std_bindgroup_layout(gpu);
        let annihilation_inter = Box::new(AnnihilationInteraction::new(width, height, state_indicies,
                                     bindgroup_layout, &gpu.device));

        let state_indicies: Vec<usize> = vec![0];
        let bindgroup_layout = Self::std_bindgroup_layout(gpu);
        let rain_inter1 =  Box::new(RainInteraction::new(width, height, true, state_indicies, 
                                   bindgroup_layout, &gpu.device));

        let state_indicies: Vec<usize> = vec![1];
        let bindgroup_layout = Self::std_bindgroup_layout(gpu);
        let rain_inter2 =  Box::new(RainInteraction::new(width, height, false, state_indicies, 
                                   bindgroup_layout, &gpu.device));

        let state_indicies: Vec<usize> = vec![2];
        let bindgroup_layout = Self::std_bindgroup_layout(gpu);
        let conway_inter = Box::new(ConwayInteraction::new(width, height, state_indicies, [30, 13, 14], 
                                   bindgroup_layout, &gpu.device));

        interactions.push(annihilation_inter);
        interactions.push(rain_inter1);
        interactions.push(rain_inter2);
        interactions.push(conway_inter);

        Ecosystem {
            states,
            interactions,
        }
    }
}
