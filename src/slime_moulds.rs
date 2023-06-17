use std::sync::Arc;
use vulkano::{
    device::Queue,
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer},
    descriptor_set::{allocator::StandardDescriptorSetAllocator, WriteDescriptorSet, PersistentDescriptorSet},
    image::{ImageUsage, StorageImage},
    format::Format,
    memory::allocator::{MemoryUsage, AllocationCreateInfo}, sync::GpuFuture, buffer::{Buffer, Subbuffer, BufferCreateInfo, BufferUsage},
};
use vulkano_util::{
    renderer::DeviceImageView,
};
use crate::app::SlimeApp;
use crate::{NUM_AGENTS, WIDTH, HEIGHT, SCALE};
use std::f32::consts::FRAC_PI_4;

const NUM_PIXELS: u32 = (WIDTH / SCALE) as u32 * (HEIGHT / SCALE) as u32;

mod slime_shader {
    vulkano_shaders::shader!{
        ty: "compute",
        path: "src/slime_moulds.glsl",
    }
}



pub struct SlimeComputePipeline {
    compute_queue: Arc<Queue>,
    compute_pipeline: Arc<ComputePipeline>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    // life_in: Subbuffer<shader::ty>,
    image: DeviceImageView,
    agent_buffer: Subbuffer<[slime_shader::SlimeAgent]>,
}


impl SlimeComputePipeline {
    pub fn new(
        app: &SlimeApp,
        compute_queue: Arc<Queue>,
        size: [u32; 2]
    ) -> Self {
        let memory_allocator = app.context.memory_allocator();

        let pipeline = {
            let shader = slime_shader::load(compute_queue.device().clone()).unwrap();
            ComputePipeline::new(
                compute_queue.device().clone(),
                shader.entry_point("main").unwrap(),
                &(),
                None,
                |_| {},
            )
            .unwrap()
        };

        let image = StorageImage::general_purpose_image_view(
            memory_allocator,
            compute_queue.clone(),
            size,
            Format::R8G8B8A8_UNORM,
            ImageUsage::SAMPLED | ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
        )
        .unwrap();

        let buffer_data = vec![([100.0, 150.0], FRAC_PI_4)];
        let mut agent_data: Vec<slime_shader::SlimeAgent> = Vec::new();
        for (pos, angle) in buffer_data {
            agent_data.push(slime_shader::SlimeAgent{pos: pos.into(), angle: angle.into()})
        }

        let agent_buffer = Buffer::from_iter(
            app.context.memory_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            agent_data
        ).unwrap();

        SlimeComputePipeline {
            compute_queue: compute_queue,
            compute_pipeline: pipeline,
            command_buffer_allocator: app.command_buffer_allocator.clone(),
            descriptor_set_allocator: app.descriptor_set_allocator.clone(),
            image,
            agent_buffer,
        }

    }

    pub fn colour_image(&self) -> DeviceImageView {
        self.image.clone()
    }

    pub fn init(
        &mut self,
        before_future: Box<dyn GpuFuture>
    ) -> Box<dyn GpuFuture> {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.compute_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();


        self.dispatch(&mut builder, 0, NUM_PIXELS);

        let command_buffer = builder.build().unwrap();
        let after_future = before_future
            .then_execute(self.compute_queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .boxed();

        after_future
    }

    pub fn compute(
        &mut self,
        before_future: Box<dyn GpuFuture>
    ) -> Box<dyn GpuFuture> {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.compute_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();


        self.dispatch(&mut builder, 1, ((NUM_AGENTS - 1) / 64 as u32) * 64 + 64);

        let command_buffer = builder.build().unwrap();
        let after_future = before_future
            .then_execute(self.compute_queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .boxed();

        after_future

    }

    fn dispatch(
        &self,
        builder: &mut AutoCommandBufferBuilder<
        PrimaryAutoCommandBuffer,
        Arc<StandardCommandBufferAllocator>>,
        step: i32,
        num_to_process: u32,
    ) {
        let pipeline_layout = self.compute_pipeline.layout();
        let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            desc_layout.clone(),
            [
                WriteDescriptorSet::image_view(0, self.image.clone()),
                WriteDescriptorSet::buffer(1, self.agent_buffer.clone()),
                // WriteDescriptorSet::buffer(2, self.life_out.clone()),
            ],
        )
        .unwrap();

        let push_constants = slime_shader::PushConstants {
            step,
            num_agents: NUM_AGENTS as i32,
        };
        builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, set)
            .push_constants(pipeline_layout.clone(), 0, push_constants)
            .dispatch([num_to_process / 64, 1, 1])
            .unwrap();
    }


    

    
}