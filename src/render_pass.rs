use std::sync::Arc;
use vulkano::{
    buffer::{BufferContents, Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    pipeline::{
        graphics::{
            vertex_input::Vertex,
            input_assembly::InputAssemblyState,
            viewport::{ViewportState, Viewport}
        },
        GraphicsPipeline,
        PipelineBindPoint,
        Pipeline
    },
    device::Queue,
    render_pass::{RenderPass, Subpass, Framebuffer, FramebufferCreateInfo},
    command_buffer::{allocator::StandardCommandBufferAllocator,SecondaryAutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferUsage, CommandBufferInheritanceInfo, RenderPassBeginInfo, SubpassContents},
    format::Format,
    memory::allocator::{AllocationCreateInfo, MemoryUsage},
    image::ImageViewAbstract,
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator},
    sampler::{Sampler, SamplerCreateInfo, Filter, SamplerAddressMode, SamplerMipmapMode},
    sync::GpuFuture,
    image::{ImageAccess},
};
use vulkano_util::renderer::{DeviceImageView, SwapchainImageView};
use crate::app::SlimeApp;



#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct TexturedVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
    #[format(R32G32_SFLOAT)]
    pub tex_coords: [f32; 2],
}


fn textured_quad(width: f32, height: f32) -> (Vec<TexturedVertex>, Vec<u32>) {
    (
        vec![
            TexturedVertex {
                position: [-(width / 2.0), -(height / 2.0)],
                tex_coords: [0.0, 1.0],
            },
            TexturedVertex {
                position: [-(width / 2.0), height / 2.0],
                tex_coords: [0.0, 0.0],
            },
            TexturedVertex {
                position: [width / 2.0, height / 2.0],
                tex_coords: [1.0, 0.0],
            },
            TexturedVertex {
                position: [width / 2.0, -(height / 2.0)],
                tex_coords: [1.0, 1.0],
            },
        ],
        vec![0, 2, 1, 0, 3, 2],
    )
}


pub struct RenderPassOverFrame{
    queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
    pixels_draw_pipeline: PixelDrawPipeline,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
}

impl RenderPassOverFrame {
    pub fn new(
        app: &SlimeApp,
        queue: Arc<Queue>,
        output_format: Format,
    ) -> RenderPassOverFrame {
        let render_pass = vulkano::single_pass_renderpass!(
            queue.device().clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: output_format,
                    samples: 1,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .unwrap();
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
        let pixels_draw_pipeline = PixelDrawPipeline::new(app, queue.clone(), subpass);

        RenderPassOverFrame {
            queue,
            render_pass,
            pixels_draw_pipeline,
            command_buffer_allocator: app.command_buffer_allocator.clone(),
        }
    }

    /// Places the view exactly over the target swapchain image. The texture draw pipeline uses a
    /// quad onto which it places the view.
    pub fn render<F>(
        &self,
        before_future: F,
        view: DeviceImageView,
        target: SwapchainImageView,
    ) -> Box<dyn GpuFuture>
    where
        F: GpuFuture + 'static,
    {
        // Get the dimensions.
        let img_dims = target.image().dimensions();

        // Create the framebuffer.
        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![target],
                ..Default::default()
            },
        )
        .unwrap();

        // Create a primary command buffer builder.
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // Begin the render pass.
        command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0; 4].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassContents::SecondaryCommandBuffers,
            )
            .unwrap();

        // Create a secondary command buffer from the texture pipeline & send draw commands.
        let cb = self
            .pixels_draw_pipeline
            .draw(img_dims.width_height(), view);

        // Execute above commands (subpass).
        command_buffer_builder.execute_commands(cb).unwrap();

        // End the render pass.
        command_buffer_builder.end_render_pass().unwrap();

        // Build the command buffer.
        let command_buffer = command_buffer_builder.build().unwrap();

        // Execute primary command buffer.
        let after_future = before_future
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap();

        after_future.boxed()
    }
}


struct PixelDrawPipeline {
    queue: Arc<Queue>,
    subpass: Subpass,
    pipeline: Arc<GraphicsPipeline>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    vertices: Subbuffer<[TexturedVertex]>,
    indices: Subbuffer<[u32]>,
}

impl PixelDrawPipeline {
    pub fn new(app: &SlimeApp, queue: Arc<Queue>, subpass: Subpass) -> PixelDrawPipeline {
        let (vertices, indices) = textured_quad(2.0, 2.0);
        let memory_allocator = app.context.memory_allocator();
        let vertex_buffer = Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            vertices,
        )
        .unwrap();
        let index_buffer = Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            indices,
        )
        .unwrap();

        let pipeline = {
            let vs = vs::load(queue.device().clone()).expect("failed to create shader module");
            let fs = fs::load(queue.device().clone()).expect("failed to create shader module");
            GraphicsPipeline::start()
                .vertex_input_state(TexturedVertex::per_vertex())
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                .input_assembly_state(InputAssemblyState::new())
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .render_pass(subpass.clone())
                .build(queue.device().clone())
                .unwrap()
        };

        PixelDrawPipeline {
            queue,
            subpass,
            pipeline,
            command_buffer_allocator: app.command_buffer_allocator.clone(),
            descriptor_set_allocator: app.descriptor_set_allocator.clone(),
            vertices: vertex_buffer,
            indices: index_buffer,
        }
    }

    fn create_image_sampler_nearest(
        &self,
        image: Arc<dyn ImageViewAbstract>,
    ) -> Arc<PersistentDescriptorSet> {
        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let sampler = Sampler::new(
            self.queue.device().clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                address_mode: [SamplerAddressMode::Repeat; 3],
                mipmap_mode: SamplerMipmapMode::Nearest,
                ..Default::default()
            },
        )
        .unwrap();

        PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::image_view_sampler(
                0,
                image.clone(),
                sampler,
            )],
        )
        .unwrap()
    }

    /// Draws input `image` over a quad of size -1.0 to 1.0.
    pub fn draw(
        &self,
        viewport_dimensions: [u32; 2],
        image: Arc<dyn ImageViewAbstract>,
    ) -> SecondaryAutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::secondary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(self.subpass.clone().into()),
                ..Default::default()
            },
        )
        .unwrap();
        let desc_set = self.create_image_sampler_nearest(image);
        builder
            .set_viewport(
                0,
                [Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [viewport_dimensions[0] as f32, viewport_dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }],
            )
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                desc_set,
            )
            .bind_vertex_buffers(0, self.vertices.clone())
            .bind_index_buffer(self.indices.clone())
            .draw_indexed(self.indices.len() as u32, 1, 0, 0, 0)
            .unwrap();
        builder.build().unwrap()
    }
}



mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450
            layout(location=0) in vec2 position;
            layout(location=1) in vec2 tex_coords;

            layout(location = 0) out vec2 f_tex_coords;

            void main() {
                gl_Position =  vec4(position, 0.0, 1.0);
                f_tex_coords = tex_coords;
            }
        ",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450
            layout(location = 0) in vec2 v_tex_coords;

            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 0) uniform sampler2D tex;

            void main() {
                f_color = texture(tex, v_tex_coords);
            }
        ",
    }
}