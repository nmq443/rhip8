use rhip8::cpu::{CPU, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::PixelFormat;
use sdl3::render::ScaleMode;
use sdl3::render::TextureAccess;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cpu = CPU::new();
    let err = cpu.load_rom_from_file("roms/3-corax+.ch8");
    if err.is_ok() {
        const SCALE: u32 = 10;

        let sdl_context = sdl3::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window(
                "CHIP-8",
                DISPLAY_WIDTH as u32 * SCALE,
                DISPLAY_HEIGHT as u32 * SCALE,
            )
            .position_centered()
            .build()?;

        let mut canvas = window.into_canvas();
        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator.create_texture(
            Some(PixelFormat::RGBA8888),
            TextureAccess::Streaming,
            DISPLAY_WIDTH as u32,
            DISPLAY_HEIGHT as u32,
        )?;
        texture.set_scale_mode(ScaleMode::Nearest);

        let mut rgba = vec![0u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * 4];
        let pitch = DISPLAY_WIDTH * 4;
        let mut event_pump = sdl_context.event_pump()?;

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            let display = &cpu.display;
            for y in 0..DISPLAY_HEIGHT {
                for x in 0..DISPLAY_WIDTH {
                    let idx = (y * DISPLAY_WIDTH + x) * 4;
                    if display[y * DISPLAY_WIDTH + x] {
                        rgba[idx..idx + 4].copy_from_slice(&[255, 255, 255, 255]);
                    } else {
                        rgba[idx..idx + 4].copy_from_slice(&[0, 0, 0, 0]);
                    }
                }
            }

            texture.update(None, &rgba, pitch)?;
            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();

            cpu.step();

            std::thread::sleep(Duration::from_millis(16));
        }
    }

    Ok(())
}
