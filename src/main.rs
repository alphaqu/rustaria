use std::fs::File;
use std::io::Read;
use std::ops::Div;
use std::time::{Duration, Instant};

use crate::client::ClientHandler;
use crate::player::Player;
use crate::world::World;

pub mod player;
pub mod world;
pub mod client;
pub mod settings;
mod network;
mod misc;
mod gen;
mod local;

fn main() {
    profiler_service::profile();
    run_rustaria();
    profiler_service::print();
}

#[profiler_macro::profile]
fn test() {
    std::thread::sleep(Duration::from_millis(1000));
}

#[profiler_macro::profile]
fn test_return() -> u32 {
    std::thread::sleep(Duration::from_millis(1000));
    69420
}


const MS_PER_UPDATE: f64 = 1000.0 / 60.0;
const MS_PER_PROFILE_PRINT: u128 = 1000;

fn run_rustaria() {
    println!("Launching Rustaria. This is gonna be rusty.");
    let mut client: ClientHandler = client::ClientHandler::create();
    let world = World::default();
    client.join_world(world);

    let mut profiler = Profiler {
        updates: 0,
        frames: 0,
        update_time: 0.0,
        frame_time: 0.0,
        last_update: Instant::now(),
    };

    let mut previous_update = Instant::now();
    let mut lag = 0f64;
    loop {
        let elapsed = previous_update.elapsed();
        previous_update = Instant::now();
        lag += elapsed.as_micros() as f64 / 1000.0;

        client.input_tick();

        while lag >= MS_PER_UPDATE {
            let time = Instant::now();
            client.tick();
            lag -= MS_PER_UPDATE;
            profiler.update_time += time.elapsed().as_nanos() as f64 / 1000000.0;
            profiler.updates += 1;
        }

        let time = Instant::now();
        client.draw();
        profiler.frame_time += time.elapsed().as_nanos() as f64 / 1000000.0;
        profiler.frames += 1;
        profiler.update();
    }
}


fn read_asset_string(path: &str) -> String {
    let mut file = File::open("./assets/".to_owned() + path).unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).expect("Could not read file");
    string
}

struct Profiler {
    updates: u32,
    frames: u32,
    update_time: f64,
    frame_time: f64,
    last_update: Instant,
}

impl Profiler {
    pub fn update(&mut self) {
        if self.last_update.elapsed().as_millis() > MS_PER_PROFILE_PRINT {
            profiler_service::print();

            let multiplier = (1000 / MS_PER_PROFILE_PRINT) as u32;
            println!("(fps/mspf|ups/mspu) {fps}/{mspf}ms | {ups}/{mspu}ms",
                     // Frames per second
                     fps = self.frames * multiplier,

                     // Milliseconds per frame
                     mspf = self.frame_time / self.frames as f64,
                     // Updates per second
                     ups = self.updates * multiplier,
                     // Milliseconds per update
                     mspu = self.update_time / self.updates as f64,
            );
            self.updates = 0;
            self.frames = 0;
            self.update_time = 0.0;
            self.frame_time = 0.0;
            self.last_update = Instant::now();
        }
    }

    fn div(v0: u32, v1: u32) -> f32 {
        ((v0 as f64 / v1 as f64 * 100f64) as u32) as f32 / 100f32
    }
}