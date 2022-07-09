pub mod lib;
use lib::{Object, Pixel, Camera};
use log::error;
use pixels::{Error, Pixels, SurfaceTexture, PixelsBuilder};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use pixels::wgpu::{PowerPreference, RequestAdapterOptions};
use std::time::Instant;
use winit::window::{Fullscreen, CursorIcon};

struct World {
    player_1: lib::Player,
    mouse_pos: (i32,i32),
    camera: lib::Camera,
    frames: u64,
    start_time: Instant,
}

impl World {
    /// Spawns a basic world with player
    fn new() -> Self {
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
            camera: lib::Camera {x: 0, y: 0},
            mouse_pos: (160, 90),
            frames: 0,
            start_time: Instant::now(),
        }
    }

    /// Updates world movement
    fn update(&mut self) {
        //make sure no frame drops
        self.frames += 1;

        if self.frames % 10000 == 0 {
            println!("{} FPS", (10000.0 /self.start_time.elapsed().as_secs_f64()));
        }

        self.player_1.update();
        self.camera.update(&self.player_1);
    }

    fn draw(&self, frame: &mut [u8]) {
        
        let mut pre_buffer: [[Pixel; 180]; 320] = [[Pixel {x: 0, y: 0, rgba: [255,255,255,255]}; 180]; 320];

        // grabs pixels from player's display output and assigns them to the pre_buffer
        for pixel in self.player_1.get_pixels(&self.camera, &self.mouse_pos).iter() {
            pre_buffer[pixel.x as usize][pixel.y as usize] = pixel.clone();
        }

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % 320 as usize) as i16;
            let y = (i / 320 as usize) as i16;

            let rgba = pre_buffer[x as usize][179 - y  as usize].rgba;

            pixel.copy_from_slice(&rgba);
        }
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let min_size = LogicalSize::new(320,180);
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

    let mut pixels = PixelsBuilder::new(320, 180, surface_texture)
    .request_adapter_options(RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    })
    .enable_vsync(true)
    .build()?;

    let mut world = World::new();

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
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::W) && world.player_1.grounded{
                world.player_1.velocity_y += 1.5;
                world.player_1.grounded = false;
            }

            if input.key_held(VirtualKeyCode::S) || input.quit(){
                world.player_1.velocity_y -= 0.1;
            }

            if input.key_held(VirtualKeyCode::A) || input.quit(){
                if world.player_1.velocity_x <= -0.5 && world.player_1.velocity_x <= 0.0{
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

            if input.key_held(VirtualKeyCode::D) || input.quit(){
                if world.player_1.velocity_x <= 0.5 && world.player_1.velocity_x >= 0.0{
                    world.player_1.velocity_x = 0.5;
                } else if world.player_1.velocity_x <= 0.5{
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
                    None => print!("cheese"),
                    Some(coord) => {
                        world.mouse_pos.0 = (coord.0 / 6.0) as i32;
                        world.mouse_pos.1 = 180 - (coord.1 / 6.0) as i32;
                    },
                }
            }

            if input.mouse_pressed(0) {
                world.player_1.grappled = true;
                world.player_1.grapple_loc = (world.mouse_pos.0 + world.camera.x, world.mouse_pos.1 + world.camera.y);
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