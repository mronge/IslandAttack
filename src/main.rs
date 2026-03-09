mod assets;
mod constants;
mod editor;
mod entities;
mod game;
mod input;
mod render;
mod replay;
mod world;

use assets::Assets;
use game::Game;
use macroquad::prelude::*;
use std::path::Path;

fn window_conf() -> Conf {
    Conf {
        window_title: constants::WINDOW_TITLE.to_owned(),
        window_width: 1920,
        window_height: 1080,
        high_dpi: false,
        sample_count: 1,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = Assets::load().await;
    let mut game = Game::new(assets);

    loop {
        let frame_dt = get_frame_time().min(0.25);
        game.frame(frame_dt);
        game.draw();
        for path in game.drain_capture_paths() {
            if let Some(parent) = Path::new(&path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            get_screen_data().export_png(&path);
            println!("Captured screenshot: {path}");
        }
        if let Some(summary) = game.take_run_summary() {
            println!("{summary}");
        }
        if game.should_exit_after_frame() {
            break;
        }
        next_frame().await;
    }
}
