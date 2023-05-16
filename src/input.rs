use gametesting::Collider;
use winit_input_helper::WinitInputHelper;
use std::collections::HashMap;
use std::clone::Clone;
use gilrs::{Gamepad, Button};

use gametesting::Collision;
use gametesting::Sprite;

use crate::World;

struct InputHandler<'a> {
    input_map: &'a HashMap<GameInput, UserInput>,
    keyboard: &'a WinitInputHelper,
    gamepad: Option<&'a Gamepad<'a>>,
    gamepad_events: &'a (Vec<Button>, Vec<Button>),
}

impl<'a> InputHandler<'a> {
    fn check(&self, game_input: &GameInput, input_state: InputState) -> bool {
        let mapped_input = self.input_map.get(game_input);
        
        match mapped_input {
            None => {
                println!("You've done messed up the input process.");
                return false;
            },
            Some(userinput) => {
                match userinput {
                    UserInput::ControllerInput(button) => {
                        if let Some(active_pad) = self.gamepad {
                            match input_state {
                                InputState::Held => return active_pad.is_pressed(button.clone()),
                                InputState::Pressed => return self.gamepad_events.0.contains(button),
                                InputState::Released => return self.gamepad_events.1.contains(button),
                            }
                        }
                    },
                    UserInput::KeyboardInput(keycode) => {
                        match input_state {
                            InputState::Held => return self.keyboard.key_held(keycode.clone()),
                            InputState::Pressed => return self.keyboard.key_pressed(keycode.clone()),
                            InputState::Released => return self.keyboard.key_released(keycode.clone()),
                        }
                    },
                }
            },
        }
        true
    }
}

#[derive(Hash, PartialEq, Eq)]
pub enum GameInput {
    PlayerLeft,
    PlayerRight,
    PlayerUp,
    PlayerDown,
    PlayerAccept,
}

pub enum InputState {
    Pressed,
    Held,
    Released
}

#[derive(Clone)]
pub enum UserInput {
    ControllerInput(gilrs::ev::Button),
    KeyboardInput(winit::event::VirtualKeyCode),
}

pub fn handle_input(world: &mut World, input: &WinitInputHelper, gamepad: Option<&Gamepad>, gamepad_events: &(Vec<Button>, Vec<Button>)) {
    let input_map = &world.input_map;
    let handler = InputHandler {
        input_map: input_map,
        keyboard: input,
        gamepad: gamepad,
        gamepad_events,
    };
    
    let mut entities = world.borrow_component_vec_mut::<Collider>().unwrap();
    let mut sprites = world.borrow_component_vec_mut::<Sprite>().unwrap();
    let player = &mut entities[0];
    let player_sprite = &mut sprites[0];
    
    if let (Some(player_collider), Some(sprite)) = (player, player_sprite) {
        if handler.check(&GameInput::PlayerLeft, InputState::Held) {
            player_collider.vel_x = -0.01;
            sprite.reversed = true;
        }
        
        if handler.check(&GameInput::PlayerLeft, InputState::Released) {
            player_collider.vel_x = 0.00;
        }
        
        if handler.check(&GameInput::PlayerRight, InputState::Held) {
            player_collider.vel_x = 0.01;
            sprite.reversed = false;
        }
        
        if handler.check(&GameInput::PlayerRight, InputState::Released) {
            player_collider.vel_x = 0.00;
        }
        //println!("{:?}", player_collider.grounded);
        //println!("YVEL: {}", player_collider.vel_y);
        //println!("XVEL: {}", player_collider.vel_x);
        if handler.check(&GameInput::PlayerUp, InputState::Pressed) && player_collider.grounded.is_some() {
            //player_collider.vel_y = 0.1;
            //player_collider.grounded = None;
            
            match player_collider.grounded {
                Some(Collision::Left) => {
                    player_collider.vel_y += 0.07;
                    player_collider.vel_x += 0.5;
                },
                Some(Collision::Right) => {
                    player_collider.vel_y += 0.07;
                    player_collider.vel_x -= 0.5;
                },
                Some(Collision::Down) => {
                    player_collider.vel_y = 0.1;
                },
                _ => {println!("HUH")},
            }
        }
        
        if handler.check(&GameInput::PlayerDown, InputState::Held) {
            player_collider.vel_y = -0.1;
        }
        
    }
}
