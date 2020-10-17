#![windows_subsystem = "windows"]

extern crate sdl2;

use sdl2::event::Event;
use sdl2::pixels::Color;
use serde::Deserialize;
use std::fs::File;
use std::io;
use std::io::Read;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

mod twitch;
use twitch::*;

mod text;
use text::TextRenderer;

#[derive(Deserialize)]
struct Config {
    channel: String,
    font: String,
    font_size: u16,
    window_width: u32,
    window_height: u32,
}

fn default_config() -> Config {
    Config {
        channel: String::from("#cantdrown"),
        font: String::from("Silver.ttf"),
        font_size: 36,
        window_width: 320,
        window_height: 720,
    }
}

fn load_config() -> Result<Config, io::Error> {
    let mut f = File::open("config.toml")?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    let config: Config = toml::from_str(&s)?;
    Ok(config)
}

fn main() {
    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            println!("Couldn't read config: {}", e);
            default_config()
        }
    };

    let ctx = sdl2::init().unwrap();
    let video_ctx = ctx.video().unwrap();
    let ttf_ctx = sdl2::ttf::init().expect("Failed to initialize ttf");

    let window = video_ctx
        .window("Chat", config.window_width, config.window_height)
        .position_centered()
        .build()
        .expect("Failed to create window");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("Failed to create canvas");

    let texture_creator = canvas.texture_creator();

    let mut text_renderer = TextRenderer::new(
        &ttf_ctx,
        &texture_creator,
        &config.font,
        config.font_size,
        config.window_width,
        config.window_height,
        4,
    );

    let (tx, rx) = channel::<TwitchCommand>();
    let _child = thread::spawn(move || {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let future = connect_to_chat(&config.channel, tx);
        runtime.block_on(future).unwrap();
    });

    let mut event_pump = ctx.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        let message = rx.try_recv();
        match message {
            Ok(message) => match message {
                TwitchCommand::Message {
                    id: _,
                    username,
                    message,
                    color,
                } => {
                    let mut s = String::from(username);
                    s += ": ";
                    s += &message;
                    text_renderer.push_line(&s, &color);
                }
                _ => {}
            },
            _ => {}
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        text_renderer.render(&mut canvas);
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
