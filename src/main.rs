mod assets;
mod constants;
mod editor;
mod entities;
mod game;
mod input;
mod render;
mod world;

use assets::Assets;
use game::Game;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: constants::WINDOW_TITLE.to_owned(),
        window_width: 1280,
        window_height: 720,
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
        next_frame().await;
    }
}
