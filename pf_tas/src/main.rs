extern crate serde;
extern crate serde_json;
extern crate vulkano_text;
extern crate vulkano_win;
extern crate winit;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate vulkano;

pub mod graphics;
pub mod buffers;
pub mod input;
pub mod state;
pub mod controller;
pub mod connection;

use std::time::{Instant, Duration};
use std::thread;

use graphics::Graphics;
use input::Input;
use state::State;

fn main() {
    let mut graphics = Graphics::new();
    let mut input = Input::new();
    let mut state = State::new();

    loop {
        let frame_start = Instant::now();

        input.update(graphics.poll_events());
        state.update(&input);
        connection::send(&mut state);
        graphics.draw(&state);

        if input.quit() {
            return;
        }

        let frame_duration = Duration::from_secs(1) / 60;
        let frame_duration_actual = frame_start.elapsed();
        if frame_duration_actual < frame_duration {
            thread::sleep(frame_duration - frame_start.elapsed());
        }
    }
}
