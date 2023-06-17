mod slime_moulds;
mod app;
mod render_pass;

use std::time::Instant;
use rand::{self, Rng};
use app::{SlimeApp, handle_window_events, compute_then_render};
use winit::event_loop::EventLoop;
use std::f32::consts::{PI, SQRT_2};


const HEIGHT: f32 = 1080.0;
const WIDTH: f32 = 1920.0;
const SCALE: f32 = 1.0;

const TURN_SPEED: f32 = 0.7;
const MOVE_SPEED: f32 = 1.0;
const SENSE_DISTANCE: f32 = 15.0;
const SENSE_ANGLE: f32 = 73.0 * PI / 180.0;
const SENSE_SIZE: i32 = 7;

const DECAY_RATE: f32 = 0.002;
const DIFFUSE_RATE: f32 = 0.05;

const START_CIRCLE_SIZE: f32 = 500.0;

fn main() {
    let mut event_loop = EventLoop::new();
    let mut slime_agents = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..50000 {
        let angle = rng.gen::<f32>() * PI * 2.0;
        // let position = [(WIDTH / SCALE) * rng.gen::<f32>(), (HEIGHT / SCALE) * rng.gen::<f32>()];
        let position = [(WIDTH / SCALE) * 0.5 + rng.gen::<f32>() * angle.cos() * START_CIRCLE_SIZE, (HEIGHT / SCALE) * 0.5 - rng.gen::<f32>() + rng.gen::<f32>() * angle.sin() * START_CIRCLE_SIZE];

        slime_agents.push((position, angle))
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



