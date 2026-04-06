mod assets;
mod constants;
mod entities;
mod game;
mod input;
mod render;
mod world;

use assets::Assets;
use assets::{load_map_data, load_result_sound, load_splash_screen, load_theme_music};
use game::Game;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: constants::WINDOW_TITLE.to_owned(),
        window_width: 1280,
        window_height: 736,
        high_dpi: false,
        sample_count: 1,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let skip_splash = std::env::args().any(|arg| arg == "--skip-splash");
    let mut game = Game::new(skip_splash);

    loop {
        let frame_dt = get_frame_time().min(0.25);
        game.frame(frame_dt);
        game.draw();

        if game.needs_splash_load() {
            let splash = load_splash_screen().await;
            game.finish_splash_load(splash);
        }

        if game.needs_theme_load() {
            let theme_music = load_theme_music().await;
            game.finish_theme_load(theme_music);
        }

        if game.needs_runtime_load() {
            let assets = Assets::load().await;
            let (map_json, spritesheet_bytes) = load_map_data().await;
            game.finish_loading(assets, &map_json, &spritesheet_bytes);
        }

        if let Some(result) = game.take_pending_result_sound() {
            let sound = load_result_sound(result).await;
            game.play_result_sound(sound);
        }

        next_frame().await;
    }
}
