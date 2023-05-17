pub mod lib;
pub mod worldinit;
pub mod physics;
pub mod econtainer;
pub mod worldgen;
use worldgen::Tile;
mod render;
use std::io::{self, Write};

mod input;
use econtainer::EContainer;
//mod render;
use input::{GameInput, UserInput};
use input::handle_input;
use gilrs::{Gilrs, Button};
use gametesting::Collider;
use gametesting::Coordinates;
use gametesting::Sprite;
use gilrs::EventType::{ButtonPressed, ButtonReleased};
use worldgen::gen_maze;

use lib::{Camera, Entity, Image, Object, ComponentVec};
use log::error;
use physics::simulate_frame;
use pixels::wgpu::{PowerPreference, RequestAdapterOptions};
use pixels::{Error, PixelsBuilder, SurfaceTexture};
use worldinit::load_images;
use std::borrow::BorrowMut;
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

use std::time::Duration;
use rodio::{Decoder, OutputStream, Sink};
use rodio::source::{SineWave, Source};

use crate::worldgen::path_maze;


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

    fn draw(&mut self, frame: &mut [u8]) {    
        let mut sprites = self.borrow_component_vec_mut::<Sprite>().unwrap();
        let coordinates = self.borrow_component_vec::<Coordinates>().unwrap();
        
        render::render_frame(&self.last_updated, &mut sprites, &coordinates, &self.sprites, frame, &self.camera);
    }
}

fn main() -> Result<(), Error> {
    let mut container = EContainer::new();
    container.new_entity();
    container.add_component_to_entity(0, 3);
    container.new_entity();
    container.add_component_to_entity(1, 5);
    container.remove_entity(0);
    
    println!("{:?}", container.free_entities);
    
    let newen = container.new_entity();
    container.add_component_to_entity(newen, 4);
    
    println!("{:?}", container.free_entities);
    
    let jod = container.new_entity();
    container.add_component_to_entity(jod, 300);
    
    
    let mut container_i32 = container.borrow_component_vec_mut::<i32>().unwrap();
    
    for num in container_i32.borrow_mut().iter() {
        println!("{:?}", num);
    }

    let start = Instant::now();
    let input = gen_maze(&mut 333334456, 50, 50, 0.25);
    let gen_duration = start.elapsed();
    let start = Instant::now();
    let paths = path_maze(&input, &(0, 0), &(49,49));
    println!("Path length: {}", paths.len());
    let path_duration = start.elapsed();

    println!("Generation: {:?} Pathing: {:?}", gen_duration, path_duration);

    display_path(50, &paths);
    //println!("Paths: {:?}", paths);
    //display_tiles(&input).unwrap();
    
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    
    
    //creates window with specified game width, scales to higher res
    let window = {
        let min_size = LogicalSize::new(426, 240);
        let size = LogicalSize::new(1280, 720);
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
        .enable_vsync(true)
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
        sprite: String::from("robot"),
        sprite_state: 0,
    };
    
    world.new_entity();
    world.add_component_to_entity(0, Sprite {
        visible: true,
        fade: false,
        sprite: "tileset",
        sprite_state: (1,0),
        time_left: 0.0,
        reversed: false,
    });
    world.add_component_to_entity(0, Coordinates { 
        coord_x: 20.0,
        coord_y: 50.0,
    });
    world.add_component_to_entity(0, Collider {
        sticky: false,
        rigid_body: true,
        active: true,
        collision: true,
        boundary: (0.0, 0.0, 16.0, 16.0),
        vel_x: 0.0,
        vel_y: 0.0,
        grounded: None,
    });
        
    world.new_entity();
    world.add_component_to_entity(1, Sprite {
        visible: true,
        fade: false,
        sprite: "tileset",
        sprite_state: (0,0),
        time_left: 0.0,
        reversed: false,
    });
    world.add_component_to_entity(1, Coordinates { 
        coord_x: 50.0,
        coord_y: 16.0
    });
    world.add_component_to_entity(1, Collider {
        sticky: false,
        rigid_body: false,
        active: false,
        collision: true,
        boundary: (0.0, 0.0, 16.0, 16.0),
        vel_x: 0.0,
        vel_y: 0.0,
        grounded: None,
    });
        
        
    for i in 2..28 {
        world.new_entity();
        world.add_component_to_entity(i, Sprite {
            visible: true,
            fade: false,
            sprite: "tileset",
            sprite_state: (0,0),
            time_left: 100000.0,
            reversed: false,
        });
        world.add_component_to_entity(i, Coordinates { 
            coord_x: (16.0 * i as f64 -80.0),
            coord_y: 0.0
        });
        world.add_component_to_entity(i, Collider {
            sticky: false,
            rigid_body: false,
            active: false,
            collision: true,
            boundary: (0.0, 0.0, 16.0, 16.0),
            vel_x: 0.0,
            vel_y: 0.0,
            grounded: None,
        });
    }
        
    world.new_entity();
    world.add_component_to_entity(28, Sprite {
        visible: true,
        fade: false,
        sprite: "textbox",
        sprite_state: (0,0),
        time_left: 0.0,
        reversed: false,
    });
    world.add_component_to_entity(28, Coordinates { 
        coord_x: 250.0,
        coord_y: 100.0
    });
    
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    // Add a dummy source of the sake of the example.
    let source = SineWave::new(18000.0).take_duration(Duration::from_secs_f32(10.25)).amplify(100.90);
    //sink.append(source);
    
    
    let lookuptable = world.sprites.get("lookuptable").unwrap();
    
    let textbox = render::create_textbox(lookuptable, &String::from("TEST"));
    world.sprites.insert(textbox.name.clone(), textbox);
    
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
        
        if input.update(&event) {
            
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
            
            window.request_redraw();
        }
        world.update();
        //println!("NEWFRAME");
    });
}


// GPT'd
pub fn display_tiles(tiles: &Vec<Vec<Option<Tile>>>) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for row in tiles {
        // print the top edge of each tile in the row
        for tile in row {
            match tile {
                Some(t) => {
                    write!(handle, "+")?;

                    if t.up == 1 {
                        write!(handle, "----+")?;
                    } else {
                        write!(handle, "    +")?;
                    }
                },
                None => {
                    write!(handle, "+     ")?;
                },
            }
        }

        write!(handle, "\n")?;

        // print the left and right edges of each tile in the row
        for tile in row {
            match tile {
                Some(t) => {
                    if t.left == 1 {
                        write!(handle, " ")?;
                    } else {
                        write!(handle, "|")?;
                    }

                    if t.up == 1 {
                        write!(handle, "  ")?;
                    } else {
                        write!(handle, "--")?;
                    }

                    if t.down == 1 {
                        write!(handle, "  ")?;
                    } else {
                        write!(handle, "--")?;
                    }

                    if t.right == 1 {
                        write!(handle, " ")?;
                    } else {
                        write!(handle, "|")?;
                    }
                },
                None => {
                    write!(handle, "      ")?;
                },
            }
        }

        write!(handle, "\n")?;
    }

    // print the bottom edge of the grid
    for _ in 0..GRID_SIZE {
        write!(handle, "+-----")?;
    }

    write!(handle, "+")?;

    Ok(())
}

const GRID_SIZE: usize = 4;

// Also GPT'd
fn display_path(grid_size: usize, path: &Vec<(usize, usize)>) {
    for y in 0..grid_size {
        for x in 0..grid_size {
            let position = (x, y);
            
            if path.contains(&position) {
                print!("X");  // Print 'X' for path cell
            } else {
                print!(".");  // Print '.' for non-path cell
            }
        }
        println!();  // Move to the next row after printing a complete row
    }
}
