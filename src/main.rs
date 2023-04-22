pub mod lib;
pub mod worldinit;
pub mod physics;

mod input;

use input::{GameInput, UserInput};
use input::handle_input;
use gilrs::{Gilrs, Button};
use gametesting::Collider;
use gametesting::Coordinates;
use gametesting::Sprite;
use gilrs::EventType::{ButtonPressed, ButtonReleased};

use lib::{Camera, Entity, Image, Object, Pixel, ComponentVec};
use log::error;
use physics::simulate_frame;
use pixels::wgpu::{PowerPreference, RequestAdapterOptions};
use pixels::{Error, PixelsBuilder, SurfaceTexture};
use worldinit::load_images;
use std::any::Any;
use std::cell::RefCell;
use std::cell::RefMut;
use std::cell::Ref;
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

pub struct World {
    player_1: lib::Player,
    mouse_pos: (i32, i32),
    camera: lib::Camera,
    last_updated: Instant,
    renderable_entities: HashMap<i32, i32>, //This is likely not the best data type to be using
    entities: HashMap<i32, Entity>,
    sprites: HashMap<String, Image>,
    entities_count: usize,
    component_vecs: Vec<Box<dyn ComponentVec>>,
    input_map: HashMap<input::GameInput, input::UserInput>,
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
            last_updated: Instant::now(),
            renderable_entities: HashMap::new(),
            entities: HashMap::new(),
            sprites: default_images,
            entities_count: 0,
            component_vecs: Vec::new(),
            input_map: HashMap::new(),
        }
    }
    
    fn new_entity(&mut self) -> usize {
        let entity_id = self.entities_count;
        for component_vec in self.component_vecs.iter_mut() {
            component_vec.push_none();
        }
        self.entities_count += 1;
        entity_id
    }
    
    // We've changed the return type to be a `RefMut`. 
    // That's what `RefCell` returns when `borow_mut` is used to borrow from the `RefCell`
    // When `RefMut` is dropped the `RefCell` is alerted that it can be borrowed from again.
    fn borrow_component_vec_mut<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                // Here we use `borrow_mut`. 
                // If this `RefCell` is already borrowed from this will panic.
                return Some(component_vec.borrow_mut());
            }
        }
        None
    }
    
    // We've changed the return type to be a `RefMut`. 
    // That's what `RefCell` returns when `borow_mut` is used to borrow from the `RefCell`
    // When `RefMut` is dropped the `RefCell` is alerted that it can be borrowed from again.
    fn borrow_component_vec<ComponentType: 'static>(
        &self,
    ) -> Option<Ref<Vec<Option<ComponentType>>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                // Here we use `borrow_mut`. 
                // If this `RefCell` is already borrowed from this will panic.
                return Some(component_vec.borrow());
            }
        }
        None
    }
    
    fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) {
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                component_vec.get_mut()[entity] = Some(component);
                return;
            }
        }
        // No matching component storage exists yet, so we have to make one.
        let mut new_component_vec: Vec<Option<ComponentType>> =
            Vec::with_capacity(self.entities_count);
    
        // All existing entities don't have this component, so we give them `None`
        for _ in 0..self.entities_count {
            new_component_vec.push(None);
        }
    
        // Give this Entity the Component.
        new_component_vec[entity] = Some(component);
        self.component_vecs.push(Box::new(RefCell::new(new_component_vec)));   
    }
        
    fn spawn(&mut self, entity: Entity) {
        self.entities.insert(entity.id, entity);
    }

    /// Updates world movement
    fn update(&mut self) {
        self.last_updated = Instant::now();
        for entity in self.entities.iter_mut() {
            entity.1.update(&self.camera, &mut self.renderable_entities);
        }

        self.player_1.update(&self.camera, &mut self.renderable_entities);
        
        self.camera.update(&self.entities.get(&0).expect("player gone"));
        
        let mut colliders = self.borrow_component_vec_mut::<Collider>().unwrap();
        let mut coordinates = self.borrow_component_vec_mut::<Coordinates>().unwrap();
        
        simulate_frame(&self.last_updated, &mut colliders, &mut coordinates, &self.renderable_entities, &self.camera);
    }

    fn draw(&self, frame: &mut [u8]) {
        let mut pre_buffer: [[[u8; 4]; GAME_HEIGHT]; GAME_WIDTH] = [[[100, 100, 100, 255]; GAME_HEIGHT]; GAME_WIDTH];
        //Iterates through all visible entities and places visible pixels on pre_buffer
        //TODO: multithread
        let sprites = self.borrow_component_vec::<Sprite>().unwrap();
        let coordinates = self.borrow_component_vec::<Coordinates>().unwrap();
        let zip = sprites.iter().zip(coordinates.iter());
        let iter = zip.filter_map(|(health, name)| Some((health.as_ref()?, name.as_ref()?)));
        
        for (sprite, coordinates) in iter
        {
            if sprite.visible {
                let sprite = &self.sprites[sprite.sprite];
                let x_rel = coordinates.coord_x as i32 - self.camera.x;
                let y_rel = coordinates.coord_y as i32 - self.camera.y;

                let mut sprite_start_x = 0;
                if x_rel < 0 {
                    sprite_start_x = x_rel.abs();
                }
                let mut sprite_end_x = sprite.sprite_width as i32 - 0;
                if sprite_end_x + x_rel + 1 > GAME_WIDTH as i32 {
                    sprite_end_x = sprite.sprite_width as i32 - 0 - ((sprite_end_x + x_rel) - GAME_WIDTH as i32);
                }
                //println!("sprite_end_x: {}", sprite_end_x);
                let mut sprite_start_y: i32 = 0;
                if y_rel < 0 {
                    sprite_start_y = y_rel.abs();
                }
                let mut sprite_end_y = sprite.sprite_height as i32 - 0;
                if sprite_end_y + y_rel + 1 > GAME_HEIGHT as i32 {
                    //perhaps change from -1 to - 0 here?
                    sprite_end_y = sprite.sprite_height as i32 - 1 - ((sprite_end_y + y_rel) - GAME_HEIGHT as i32);
                }

                for x in (sprite_start_x + x_rel)..(sprite_end_x + x_rel) {
                    for y in (sprite_start_y + y_rel)..(sprite_end_y + y_rel) {
                        let location = (x - x_rel + sprite.sprite_width as i32 * (y - y_rel)) * 4;
                        if sprite.bytes[(location + 3) as usize] == 255 {
                            //perhaps calc pixel location once, then add 1,2,3 for rgb value later?
                            pre_buffer[x as usize][y as usize][0] = sprite.bytes[location as usize];
                            pre_buffer[x as usize][y as usize][1] = sprite.bytes[(location + 1 ) as usize];
                            pre_buffer[x as usize][y as usize][2] = sprite.bytes[(location + 2 ) as usize];
                            pre_buffer[x as usize][y as usize][3] = 255;
                        } else if sprite.bytes[(location + 3) as usize] == 0 {
                            //adds no pixel
                        } else {
                            let src: &[u8; 4] = &pre_buffer[x as usize][y as usize][0..4].try_into().unwrap();
                            let dst = &sprite.bytes[location as usize..(location + 4) as usize].try_into().unwrap();
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

// Blends alpha between 2 pixels quickly. Not a correct implementation, as it ignores the background pixel's alpha.
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
    
    world.input_map.insert(GameInput::PlayerLeft, UserInput::KeyboardInput(VirtualKeyCode::A));
    world.input_map.insert(GameInput::PlayerRight, UserInput::KeyboardInput(VirtualKeyCode::D));
    world.input_map.insert(GameInput::PlayerUp, UserInput::KeyboardInput(VirtualKeyCode::W));
    world.input_map.insert(GameInput::PlayerDown, UserInput::KeyboardInput(VirtualKeyCode::S));

    let player = Entity {
        id: 0,
        visible: true,
        collision: false,
        hitbox_x: 16.0,
        hitbox_y: 16.0,
        coord_x: 0.0,
        coord_y: 0.0,
        sprite: String::from("steve.png"),
        sprite_state: 0,
    };
    
    world.new_entity();
    world.add_component_to_entity(0, Sprite {
        visible: true,
        sprite: "steve.png",
        sprite_state: 0,
    });
    world.add_component_to_entity(0, Coordinates { 
        coord_x: 50.0,
        coord_y: 50.0});
    world.add_component_to_entity(0, Collider {
        rigid_body: true,
        active: true,
        collision: true,
        boundary: (0.0, 0.0, 9.0, 9.0),
        vel_x: 0.0,
        vel_y: 0.12,
    });
        
    world.new_entity();
    world.add_component_to_entity(1, Sprite {
        visible: true,
        sprite: "coord.png",
        sprite_state: 0,
    });
    world.add_component_to_entity(1, Coordinates { 
        coord_x: 50.0,
        coord_y: 110.0});
    world.add_component_to_entity(1, Collider {
        rigid_body: true,
        active: true,
        collision: true,
        boundary: (0.0, 0.0, 16.0, 16.0),
        vel_x: 0.00,
        vel_y: 0.00,
    });
        
    world.new_entity();
    world.add_component_to_entity(2, Sprite {
        visible: true,
        sprite: "coord.png",
        sprite_state: 0,
    });
    world.add_component_to_entity(2, Coordinates { 
        coord_x: 100.0,
        coord_y: 50.0});
    world.add_component_to_entity(2, Collider {
        rigid_body: true,
        active: true,
        collision: true,
        boundary: (0.0, 0.0, 16.0, 16.0),
        vel_x: 0.00,
        vel_y: 0.00,
    });
        
    world.new_entity();
    world.add_component_to_entity(3, Sprite {
        visible: true,
        sprite: "platform.png",
        sprite_state: 0,
    });
    world.add_component_to_entity(3, Coordinates { 
        coord_x: 40.0,
        coord_y: 0.0});
    world.add_component_to_entity(3, Collider {
        rigid_body: true,
        active: true,
        collision: true,
        boundary: (0.0, 0.0, 32.0, 6.0),
        vel_x: 0.00,
        vel_y: 0.00,
    });
        
    world.new_entity();
    world.add_component_to_entity(4, Sprite {
        visible: true,
        sprite: "floor.png",
        sprite_state: 0,
    });
    world.add_component_to_entity(4, Coordinates { 
        coord_x: 80.0,
        coord_y: 0.0});
    world.add_component_to_entity(4, Collider {
        rigid_body: true,
        active: true,
        collision: true,
        boundary: (0.0, 0.0, 256.0, 16.0),
        vel_x: 0.00,
        vel_y: 0.00,
    });
        
    for i in 5..1000 {
       world.new_entity();
        world.add_component_to_entity(i, Sprite {
            visible: true,
            sprite: "floor.png",
            sprite_state: 0,
        });
        world.add_component_to_entity(i, Coordinates { 
            coord_x: 480.0,
            coord_y: 0.0});
        world.add_component_to_entity(i, Collider {
            rigid_body: true,
            active: true,
            collision: true,
            boundary: (0.0, 0.0, 256.0, 16.0),
            vel_x: 0.00,
            vel_y: 0.00,
        }); 
    }
        
    world.spawn(player);
    
    let mut gilrs = Gilrs::new().unwrap();
    
    let mut active_gamepad = None;
    
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }   
    
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
        
        let mut gamepad_events: (Vec<Button>, Vec<Button>)  = (Vec::new(), Vec::new());
        while let Some(gilrs::Event { id, event, time }) = gilrs.next_event() {
            //println!("{:?} New event from {}: {:?}", time, id, event);
            match event {
                ButtonPressed(button, code) => gamepad_events.0.push(button),
                ButtonReleased(button, code) => gamepad_events.1.push(button),
                _ => {},
            }
            active_gamepad = Some(id);
        }
        
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            //println!("Button South is pressed (XBox - A, PS - X)");
            handle_input(&mut world, &input, Some(&gamepad), &gamepad_events);
        } else {
            handle_input(&mut world, &input, None, &gamepad_events);
        }
        
        // I should probably figure out a better way to do things than this but oh well for now
        // perhaps separate file to handle these inputs? //no
        if input.update(&event) {
            
            //println!("event: {:?}", input);
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
        
    });
}
