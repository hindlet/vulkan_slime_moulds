mod slime_moulds;
mod app;
mod render_pass;

use std::time::Instant;

use app::{SlimeApp, handle_window_events, compute_then_render};
use winit::event_loop::EventLoop;


const HEIGHT: f32 = 768.0;
const WIDTH: f32 = 1024.0;
const SCALE: f32 = 1.0;
const NUM_AGENTS: u32 = 1;

const TURN_SPEED: f32 = 0.1;

fn main() {
    let mut event_loop = EventLoop::new();

    let mut app = SlimeApp::default();
    app.open(&event_loop);

    let mut time = Instant::now();

    loop {
        if !handle_window_events(&mut event_loop, &mut app) {break;}

        if (Instant::now() - time).as_secs_f32() > 1.0 / 60.0 {
            compute_then_render(&mut app);
            time = Instant::now();
        }
    }
}



