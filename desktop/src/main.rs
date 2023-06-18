use sdl2::event::Event;

use chip8_core::*;
use std::env;
use std::fs::File;
use std::io::Read;
use sdl2::keyboard::Keycode;
use crate::input::key2btn;

const SCALE: u32 = 15;
const TICKS_PER_FRAME: usize = 10;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

mod input;
mod rendering;
mod sound;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }
    let rom_path = &args[1];
    let mut emu = Emu::new();
    emu.reset();

    let mut rom = File::open(rom_path).expect("Unable to open the file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    emu.load(&buffer);

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    // Setup SDL
    let sdl_context = sdl2::init().unwrap();
    let mut canvas = rendering::build_window(&sdl_context, WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut event_pump = sdl_context.event_pump().unwrap();

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..}=> {
                    break 'gameloop;
                },
                Event::KeyDown {keycode: Some(key), ..} => {
                    if let Some(k) = key2btn(key) {
                        emu.keypress(k, true);
                    }
                },
                Event::KeyUp {keycode: Some(key), ..} => {
                    if let Some(k) = key2btn(key) {
                        emu.keypress(k, false);
                    }
                }
              _ => ()
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            emu.tick();
        }
        let needs_to_beep = emu.tick_timers();
        rendering::draw_screen(&emu, &mut canvas);
        sound::make_sound(needs_to_beep, &stream_handle);
    }
}
