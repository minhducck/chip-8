use std::{fs::File, io::{self, Read, Write}};
use chip8_lib::*;
use sdl2::event::Event;
use std::env;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICK_PER_LOOP: u8 = 5;



fn render_screen(processor: &mut Processor, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
    canvas.clear();

    let screen_buf = processor.get_display();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i% SCREEN_WIDTH) as u32;
            let y = (i/SCREEN_WIDTH) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }

    canvas.present();
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    let sdl_context = sdl2::init().unwrap();
    let video_system = sdl_context.video().unwrap();

    let window = video_system.window("Chip8 Emulator", WINDOW_WIDTH.try_into().unwrap(), WINDOW_HEIGHT.try_into().unwrap())
    .position_centered()
    .opengl()
    .build()
    .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut processor = Processor::new();
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    processor.load_rom(&buffer);

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit{..} => {
                    break 'gameloop;
                },
                Event::KeyDown{keycode: Some(key), ..} => {
                    if let Some(k) = key_map(key) {
                        processor.press_key(k, true);
                    }
                },
                Event::KeyUp{keycode: Some(key), ..} => {
                    if let Some(k) = key_map(key) {
                        processor.press_key(k, false);
                    }
                },
                _ => ()
            }
        }

        for _i in 0..TICK_PER_LOOP {
            processor.tick();
        }

        processor.timer_tick();
        render_screen(&mut processor, &mut canvas);
    }
}

fn key_map (key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
