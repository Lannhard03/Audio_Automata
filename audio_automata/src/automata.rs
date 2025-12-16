
pub struct AutomataState {
    width: u32,
    height: u32,
    work_group_size: u32, //Assume work groups are square
    automata_bindgroup: wgpu::BindGroup,
    automata_bind_group_layout: wgpu::BindGroupLayout,
    cell_buffer: wgpu::Buffer,
}

pub struct AutomataRenderer {
    //Probably needed, but should also combine several automata?
    //create a bindgroup using cell_buffers from several automata
}


pub struct AutomataRule {
    pipeline: wgpu::ComputePipeline,
    prm_bindgroup: wgpu::BindGroup,
}

impl AutomataRule {
    //A rule must be created for a particular automata as the pipeline needs
    //the bindgroup layout of the automata. Alt. this bind group layout is 
    //the same for all automata and
    //gotten from a 3rd struct "AutomataStateBGLayout"

    //Somehow update parameters based on sound, and other things??
}

pub struct Automata {
    state: AutomataState,
    rules: Vec<AutomataRule>,
}

impl Automata {
    pub fn update(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let wg_size = self.state.work_group_size;
        let num_dispatches_x = self.state.width.div_ceil(wg_size) as u32;
        let num_dispatches_y = self.state.height.div_ceil(wg_size) as u32;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
           label: Some("Compute Encoder"),
        });

        {
           let mut compute_pass = encoder.begin_compute_pass(&Default::default());

            for rule in &self.rules {
                compute_pass.set_pipeline(&rule.pipeline);
                compute_pass.set_bind_group(0, &rule.prm_bindgroup, &[]);
                compute_pass.set_bind_group(1, &self.state.automata_bindgroup, &[]);
                compute_pass.dispatch_workgroups(num_dispatches_x, num_dispatches_y, 1);
            }
        }
        queue.submit([encoder.finish()]);
    }
}
