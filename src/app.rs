use std::{sync::Arc};
use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
};
use vulkano_util::{
    context::{VulkanoContext, VulkanoConfig},
    window::{VulkanoWindows, WindowDescriptor},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    platform::run_return::EventLoopExtRunReturn
};
use crate::slime_moulds::SlimeComputePipeline;
use crate::render_pass::RenderPassOverFrame;
use crate::{HEIGHT, WIDTH, SCALE};




pub struct SlimeApp {
    pub context: VulkanoContext,
    pub windows: VulkanoWindows,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    pub pipeline: Option<(SlimeComputePipeline, RenderPassOverFrame)>,
}

impl SlimeApp {
    pub fn open(&mut self, event_loop: &EventLoop<()>) {
        let window_id = self.windows.create_window(
            event_loop,
            &self.context,
            &WindowDescriptor {
                width: WIDTH,
                height: HEIGHT,
                title: "Slime Mould Simulation".to_string(),
                ..Default::default()
            },
            |_| {},
        );

        let mut pipeline = SlimeComputePipeline::new(
            self,
            self.context.graphics_queue().clone(),
            [(WIDTH / SCALE) as u32, (HEIGHT / SCALE) as u32]
        );
        let render_pass = RenderPassOverFrame::new(
            self,
            self.context.graphics_queue().clone(),
            self.windows.get_renderer(window_id).unwrap().swapchain_format(),
        );

        

        let window_renderer = self.windows.get_primary_renderer_mut().unwrap();
        match window_renderer.window_size() {
            [w, h] => {
                if w == 0.0 || h == 0.0 {
                    return;
                }
            }
        }

        let before_pipeline_future = match window_renderer.acquire() {
            Err(e) => {
                println!("{e}");
                return;
            }
            Ok(future) => future,
        };

        let after_compute = pipeline.init(before_pipeline_future);

        let color_image = pipeline.colour_image();
        let target_image = window_renderer.swapchain_image_view();

        let after_render = render_pass
            .render(after_compute, color_image, target_image);

        window_renderer.present(after_render, true);

        self.pipeline = Some((pipeline, render_pass));
    }
}



pub fn handle_window_events (
    event_loop: &mut EventLoop<()>,
    app: &mut SlimeApp,
) -> bool {
    let mut running = true;

    event_loop.run_return(|event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match &event {
            Event::WindowEvent{
                window_id, event
            } => match event {
                WindowEvent::CloseRequested => {
                    running = false;
                    app.windows.remove_renderer(*window_id);
                },
                _ => ()
            },
            Event::MainEventsCleared => *control_flow = ControlFlow::Exit,

            _ => ()
        }
    });


    running
}

// pub fn draw_slime(
//     app: &mut SlimeApp
// ) {
//     let window_renderer = app.windows.get_primary_renderer().unwrap();
//     let compute_pipeline = &mut app.pipeline.as_mut().unwrap().0;

//     let size = window_renderer.window_size();
//     let image_size = compute_pipeline
//         .colour_image()
//         .image()
//         .dimensions()
//         .width_height();
    
//     compute_pipeline.d

// }

pub fn compute_then_render(
    app: &mut SlimeApp
) {
    let window_renderer = app.windows.get_primary_renderer_mut().unwrap();
    match window_renderer.window_size() {
        [w, h] => {
            if w == 0.0 || h == 0.0 {
                return;
            }
        }
    }

    let (compute_pipeline, render_pipeline) = app.pipeline.as_mut().unwrap();

    let before_pipeline_future = match window_renderer.acquire() {
        Err(e) => {
            println!("{e}");
            return;
        }
        Ok(future) => future,
    };

    let after_compute = compute_pipeline.compute(before_pipeline_future);

    let color_image = compute_pipeline.colour_image();
    let target_image = window_renderer.swapchain_image_view();

    let after_render = render_pipeline
        .render(after_compute, color_image, target_image);

    window_renderer.present(after_render, true);
}




impl Default for SlimeApp {
    fn default() -> Self {
        let context = VulkanoContext::new(VulkanoConfig::default());
        let command_allocator = Arc::new(StandardCommandBufferAllocator::new(
            context.device().clone(),
            Default::default()
        ));
        let descript_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            context.device().clone()
        ));
        

        SlimeApp {
            context,
            windows: VulkanoWindows::default(),
            command_buffer_allocator: command_allocator,
            descriptor_set_allocator: descript_allocator,
            pipeline: None
        }
    }
}




