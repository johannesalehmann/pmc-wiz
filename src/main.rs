mod room;

use crate::room::RoomGenerator;
use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use probabilistic_models::traits::{ReadChoiceLabelsMaybe, ReadValuationsMaybe};
use probabilistic_models::{Index, StateIndex};
use std::collections::HashMap;
use tiny_pmc::ProcessingOptions;

#[macroquad::main("MyGame")]
async fn main() {
    let Ok((files, consts)) = parse_args() else {
        return;
    };
    let inputs =
        tiny_pmc::parsing::Inputs::from_cli_args(&files).expect("failed to parse input files");
    let consts = if let Some(consts) = consts.first() {
        tiny_pmc::parsing::parse_const_assignments(&consts)
            .expect("failed to parse constant assignments")
    } else {
        HashMap::new()
    };
    let (model, _queries) = tiny_pmc::build_model(inputs, consts, &ProcessingOptions::default())
        .expect("failed to build model");
    let model = model
        .base_into_mdp()
        .expect("Only MDPs are supported")
        .try_with_choice_labels()
        .unwrap()
        .try_with_valuations()
        .unwrap();
    let mut room = RoomGenerator::generate(&model, StateIndex::default());
    // room.carve_platform(5, 3, 3, 6, 6);
    // room.carve_platform(3, 7, 3, 10, 6);
    // room.carve_platform(5, 11, 1, 16, 8);
    // room.carve_platform(4, 11, 8, 16, 9);
    // room.build_stair(5, 4, 4, 2, false);
    // room.build_stair(11, 4, 4, 2, true);
    // room.build_bridge(16, 5, 1, 5, 0.05);
    // room.build_bridge(16, 5, 4, 3, 0.2);
    // room.build_bridge(16, 5, 7, 9, 0.5);
    // room.build_pillar(21, 5, 1);
    // room.build_pillar(19, 5, 4);
    // room.build_pillar(25, 5, 7);
    // room.build_tube(18, 6, 1);
    // room.build_tube(17, 6, 4);
    // room.build_tube(20, 6, 7);
    // room.build_sign(14, 4, 1, false);
    // room.build_sign(14, 4, 4, false);
    // room.build_sign(14, 4, 7, false);
    // room.place_player(13, 4, 4);
    let texture = load_texture("data/spritesheet.png").await.unwrap();
    texture.set_filter(FilterMode::Nearest);
    let mut camera = room::Camera {
        scale: 1.0,
        offset: Vec2::default(),
    };
    let mut black_fade = 0.0;
    loop {
        clear_background(BLACK);
        if let Some(to) = room.teleport_to {
            black_fade += get_frame_time() * 1.3;
            if black_fade >= 1.0 {
                black_fade = 1.0;
                room = RoomGenerator::generate(&model, StateIndex::from_raw(to));
            }
        } else {
            black_fade -= get_frame_time() * 5.0;
            if black_fade < 0.0 {
                black_fade = 0.0;
            }
        }
        room.update();
        camera.update(screen_size(), room.player_pos());
        room.draw(&texture, &camera);
        let actual_fade = if room.teleport_to.is_some() {
            (black_fade * 3.0 - 2.0).max(0.0)
        } else {
            (black_fade * 2.0).min(1.0)
        };
        if actual_fade > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, actual_fade),
            );
        }

        // draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        // draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        //
        // draw_text("Hello, Macroquad!", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}

fn parse_args() -> Result<(Vec<String>, Vec<String>), ()> {
    let mut next_is_const = false;
    let mut files = Vec::new();
    let mut consts = Vec::new();
    for arg in std::env::args().skip(1) {
        if arg == "-h" || arg == "--help" {
            print_help();
            return Err(());
        }
        if arg == "-c" {
            next_is_const = true;
        }
        if arg.starts_with("-") {
            println!("unexpected argument `{arg}`");
            return Err(());
        }
        if next_is_const {
            if consts.len() > 0 {
                println!(
                    "constants specified twice (combine all constants into a single argument instead)"
                );
                return Err(());
            }
            consts.push(arg);
        } else {
            files.push(arg);
        }
    }
    if files.is_empty() {
        print_help();
        return Err(());
    }
    Ok((files, consts))
}

fn print_help() {
    let help = r#"                      .
            /^\     .
       /\   "V"                 ***********
      /__\   I      O  o        * PMC-WIZ *
     //..\\  I     .            ***********
     \].`[/  I
     /l\/j\  (]    .  O         | Call with:
    /. ~~ ,\/I          .       |     pmc-wiz model_file.prism props_file.props -c "K=5,n=12"
    \\L__j^\/I       o          |
     \/--v}  I     o   .        | Model and property files can be in any order.
     |    |  I   _________
     |    |  I c(`       ')o
     |    l  I   \.     ,/
   _/j  L l\_!  _//^---^\\_     This ASCII art is by Rowan Crawford
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

            "#;
    println!("{help}")
}
