mod slime_moulds;
mod app;
mod render_pass;

use std::time::Instant;
use rand::{self, Rng};
use app::{SlimeApp, handle_window_events, compute_then_render};
use winit::event_loop::EventLoop;
use std::f32::consts::PI;


const HEIGHT: f32 = 1080.0;
const WIDTH: f32 = 1910.0;
const SCALE: f32 = 1.0;

const TURN_SPEED: f32 = 0.1;

fn main() {
    let mut event_loop = EventLoop::new();
    let mut slime_agents = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..500 {
        slime_agents.push(([WIDTH / (SCALE * 2.0), HEIGHT / (SCALE * 2.0)], rng.gen::<f32>() * PI))
    }

    let mut app = SlimeApp::default();
    app.open(&event_loop, slime_agents);

    let mut time = Instant::now();

    loop {
        if !handle_window_events(&mut event_loop, &mut app) {break;}

        if (Instant::now() - time).as_secs_f32() > 1.0 / 60.0 {
            compute_then_render(&mut app);
            time = Instant::now();
        }
    }
}



