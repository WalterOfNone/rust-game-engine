pub mod lib;
pub mod worldinit;

use lib::{Camera, Entity, Image, Object, Pixel};
use log::error;
use pixels::wgpu::{PowerPreference, RequestAdapterOptions};
use pixels::{Error, Pixels, PixelsBuilder, SurfaceTexture};
use worldinit::load_images;
use std::collections::HashMap;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit::window::{CursorIcon, Fullscreen};
use winit_input_helper::WinitInputHelper;

const GAME_HEIGHT: usize = 240;
const GAME_WIDTH: usize = 426;

struct World {
    player_1: lib::Player,
    mouse_pos: (i32, i32),
    camera: lib::Camera,
    frames: u64,
    start_time: Instant,
    entities: HashMap<i32, Entity>,
    sprites: HashMap<String, Image>,
}

impl World {
    /// Spawns a basic world with player
    fn new() -> Self {
        let default_images = load_images();
        
        Self {
            player_1: lib::Player {
                health: 100,
                coord_x: 20.0,
                coord_y: 20.0,
                velocity_x: 0.0,
                velocity_y: 0.0,
                grounded: false,
                grappled: false,
                grapple_loc: (0, 0),
            },
            camera: lib::Camera { x: 0, y: 0 },
            mouse_pos: (160, 90),
            frames: 0,
            start_time: Instant::now(),
            entities: HashMap::new(),
            sprites: default_images,
        }
    }

    fn spawn(&mut self, entity: Entity) {
        self.entities.insert(entity.id, entity);
    }

    /// Updates world movement
    fn update(&mut self) {
        for entity in self.entities.iter_mut() {
            entity.1.update(&self.camera);
        }

        self.player_1.update(&self.camera);
        self.camera.update(&self.player_1);
    }

    fn draw(&self, frame: &mut [u8]) {
        let mut pre_buffer: [[[u8; 4]; GAME_HEIGHT]; GAME_WIDTH] = [[[100, 100, 100, 255]; GAME_HEIGHT]; GAME_WIDTH];
        //Iterates through all visible entities and places visible pixels on pre_buffer
        //TODO: multithread
        for entity in self.entities.iter() {
            if entity.1.visible {
                let x_rel = entity.1.coord_x as i32 - self.camera.x;
                let y_rel = entity.1.coord_y as i32 - self.camera.y;

                let mut sprite_start_x = 0;
                if x_rel < 0 {
                    sprite_start_x = x_rel.abs();
                }
                let mut sprite_end_x = self.sprites[&entity.1.sprite].sprite_width as i32 - 0;
                if sprite_end_x + x_rel + 1 > GAME_WIDTH as i32 {
                    sprite_end_x = self.sprites[&entity.1.sprite].sprite_width as i32 - 0 - ((sprite_end_x + x_rel) - GAME_WIDTH as i32);
                }
                //println!("sprite_end_x: {}", sprite_end_x);
                let mut sprite_start_y: i32 = 0;
                if y_rel < 0 {
                    sprite_start_y = y_rel.abs();
                }
                let mut sprite_end_y = self.sprites[&entity.1.sprite].sprite_height as i32 - 0;
                if sprite_end_y + y_rel + 1 > GAME_HEIGHT as i32 {
                    //perhaps change from -1 to - 0 here?
                    sprite_end_y = self.sprites[&entity.1.sprite].sprite_height as i32 - 1 - ((sprite_end_y + y_rel) - GAME_HEIGHT as i32);
                }

                let sprite = &self.sprites[&entity.1.sprite];

                for x in (sprite_start_x + x_rel)..(sprite_end_x + x_rel) {
                    for y in (sprite_start_y + y_rel)..(sprite_end_y + y_rel) {
                        if sprite.bytes[((x - x_rel + sprite.sprite_width as i32 * (y - y_rel)) * 4+ 3) as usize] == 255 {
                            pre_buffer[x as usize][y as usize][0] = sprite.bytes[((x - x_rel + sprite.sprite_width as i32 * (y - y_rel))* 4+ 0)as usize];
                            pre_buffer[x as usize][y as usize][1] = sprite.bytes[((x - x_rel + sprite.sprite_width as i32 * (y - y_rel))* 4+ 1)as usize];
                            pre_buffer[x as usize][y as usize][2] = sprite.bytes[((x - x_rel+ sprite.sprite_width as i32 * (y - y_rel))* 4+ 2)as usize];
                            pre_buffer[x as usize][y as usize][3] = 255;
                        } else if sprite.bytes[((x - x_rel+ sprite.sprite_width as i32 * (y - y_rel))* 4+ 3) as usize] == 0 {
                            //adds no pixel
                        } else {
                            let src: &[u8; 4] = &pre_buffer[x as usize][y as usize][0..4].try_into().unwrap();
                            let dst = &sprite.bytes[((x - x_rel + sprite.sprite_width as i32 * (y - y_rel))* 4+ 0) as usize..((x - x_rel + sprite.sprite_width as i32 * (y - y_rel)) * 4 + 4)as usize].try_into().unwrap();
                            let blended = blend_alpha_fast(src, dst);
                            pre_buffer[x as usize][y as usize][0] = blended[0];
                            pre_buffer[x as usize][y as usize][1] = blended[1];
                            pre_buffer[x as usize][y as usize][2] = blended[2];
                            pre_buffer[x as usize][y as usize][3] = blended[3];
                        }
                    }
                }
            }
        }

        //copies pixel array into current frame
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % GAME_WIDTH as usize) as i16;
            let y = (i / GAME_WIDTH as usize) as i16;

            let rgba = pre_buffer[x as usize][GAME_HEIGHT - 1 - y as usize];

            pixel.copy_from_slice(&rgba);
        }
    }
}

//Blends alpha between 2 pixels quickly. Not a correct implementation, as it ignores the background pixel's alpha.
fn blend_alpha_fast(&src: &[u8; 4], &dst: &[u8; 4]) -> [u8; 4] {
    let mut blended = [255 as u8; 4];
    blended[0] = (dst[0] as f64 * (dst[3] as f64) / 255.0) as u8 + (src[0] as f64 * (255.0 - dst[3] as f64) / 255.0) as u8;
    blended[1] = (dst[1] as f64 * (dst[3] as f64) / 255.0) as u8 + (src[1] as f64 * (255.0 - dst[3] as f64) / 255.0) as u8;
    blended[2] = (dst[2] as f64 * (dst[3] as f64) / 255.0) as u8 + (src[2] as f64 * (255.0 - dst[3] as f64) / 255.0) as u8;
    blended[3] = 255;

    return blended;
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    //creates window with specified game width, scales to higher res
    let window = {
        let min_size = LogicalSize::new(426, 240);
        let size = LogicalSize::new(1920, 1080);
        WindowBuilder::new()
            .with_title("Verified Game Testing Moment")
            .with_inner_size(size)
            .with_min_inner_size(min_size)
            .build(&event_loop)
            .unwrap()
    };

    window.set_cursor_icon(CursorIcon::Crosshair);

    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);

    let mut pixels = PixelsBuilder::new(426, 240, surface_texture)
        .request_adapter_options(RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .enable_vsync(false)
        .build()?;

    let mut world = World::new();

    let player = Entity {
        id: 0,
        visible: true,
        collision: false,
        hitbox_x: 16.0,
        hitbox_y: 16.0,
        coord_x: 20.0,
        coord_y: 20.0,
        sprite: String::from("steve.png"),
        sprite_state: 0,
    };

    world.spawn(player);
    
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // I should probably figure out a better way to do things than this but oh well for now
        // perhaps separate file to handle these inputs?
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::W) && world.player_1.grounded {
                world.player_1.velocity_y += 1.5;
                world.player_1.grounded = false;
            }

            if input.key_held(VirtualKeyCode::S) || input.quit() {
                world.player_1.velocity_y -= 0.1;
            }

            if input.key_held(VirtualKeyCode::A) || input.quit() {
                if world.player_1.velocity_x <= -0.5 && world.player_1.velocity_x <= 0.0 {
                    world.player_1.velocity_x = -0.5;
                } else {
                    world.player_1.velocity_x -= 0.5;
                }
            }

            if input.key_released(VirtualKeyCode::A) {
                if world.player_1.velocity_x >= 0.5 {
                    world.player_1.velocity_x += 0.5;
                }
            }

            if input.key_held(VirtualKeyCode::D) || input.quit() {
                if world.player_1.velocity_x <= 0.5 && world.player_1.velocity_x >= 0.0 {
                    world.player_1.velocity_x = 0.5;
                } else if world.player_1.velocity_x <= 0.5 {
                    world.player_1.velocity_x += 0.5;
                }
            }

            if input.key_released(VirtualKeyCode::D) {
                if world.player_1.velocity_x >= 0.5 {
                    world.player_1.velocity_x += 0.5;
                }
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            let mouse_diff = input.mouse_diff();
            if mouse_diff != (0.0, 0.0) {
                match input.mouse() {
                    None => print!("Something with the mouse became unholy! Check line 303 for more details! :)"),
                    Some(coord) => {
                        world.mouse_pos.0 = (coord.0 / 6.0) as i32;
                        world.mouse_pos.1 = 240 - (coord.1 / 6.0) as i32;
                    }
                }
            }

            if input.mouse_pressed(0) {
                world.player_1.grappled = true;
                world.player_1.grapple_loc = (
                    world.mouse_pos.0 + world.camera.x,
                    world.mouse_pos.1 + world.camera.y,
                );
            }

            if input.mouse_released(0) {
                world.player_1.grappled = false;
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}
