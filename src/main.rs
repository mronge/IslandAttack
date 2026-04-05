mod assets;
mod constants;
mod entities;
mod game;
mod input;
mod render;
mod world;

use assets::Assets;
use assets::{load_result_sound, load_splash_screen, load_theme_music};
use game::Game;
use macroquad::audio::{PlaySoundParams, play_sound};
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
    let splash_screen = load_splash_screen().await;
    let theme_music = load_theme_music().await;
    play_sound(
        &theme_music,
        PlaySoundParams {
            looped: true,
            volume: 0.6,
        },
    );
    let mut game = Game::new(splash_screen, theme_music, skip_splash);

    loop {
        if game.needs_runtime_load() {
            let assets = Assets::load().await;
            game.finish_loading(assets);
            continue;
        }

        if let Some(result) = game.take_pending_result_sound() {
            let sound = load_result_sound(result).await;
            game.play_result_sound(sound);
            continue;
        }

        let frame_dt = get_frame_time().min(0.25);
        game.frame(frame_dt);
        game.draw();
        next_frame().await;
    }
}
