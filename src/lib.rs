use std::fs::File;
use std::collections::HashMap;
const GAME_HEIGHT: i32 = 240;
const GAME_WIDTH: i32 = 426;
use std::cell::RefCell;
use std::io::Read;
use serde::{Serialize, Deserialize};


pub trait ComponentVec {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn push_none(&mut self);
    fn set_none(&mut self, index: usize);
}

impl<T: 'static> ComponentVec for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn push_none(&mut self) {
        self.get_mut().push(None)
    }
    
    fn set_none(&mut self, index: usize) {
        self.get_mut()[index] = None;
    }
}

/// Rectangular collider with optional collision
///
/// Boundary box defined as x1, y1, x2, y2
pub struct Collider {
    pub sticky: bool,
    pub rigid_body: bool,
    pub active: bool,
    pub collision: bool,
    pub boundary: (f64, f64, f64, f64),
    pub vel_x: f64,
    pub vel_y: f64,
    pub grounded: Option<Collision>
}

#[derive(Debug)]
pub enum Collision {
    Left,
    Right,
    Down,
    Up,
}

pub struct Sprite {
    pub visible: bool,
    pub sprite: &'static str,
    pub sprite_state: (u32, u32),
    pub time_left: f64,
    pub fade: bool,
    pub reversed: bool,
}

pub struct Text {
    pub text: &'static str,
    pub speed: f64,
}

pub struct Coordinates {
    pub coord_x: f64,
    pub coord_y: f64,
}

pub trait Object {
    /// Returns pixels relative to the current camera
    fn get_pixels(&self, camera: &Camera, mouse_pos: &(i32, i32)) -> Vec<Pixel>;
    fn update(&mut self, camera: &Camera, renderable_entities: &mut HashMap<i32,i32>);
}

pub struct Entity {
    pub id: i32,
    pub visible: bool,
    pub collision: bool,
    pub hitbox_x: f64,
    pub hitbox_y: f64,
    pub coord_x: f64,
    pub coord_y: f64,
    pub sprite: String,
    pub sprite_state: u32,
}

pub struct Image {
    pub name: String,
    pub bytes: Vec<u8>,
    pub sprite_height: u32,
    pub sprite_width: u32,
    pub image_width: u32,
    pub row_length: Vec<u32>,
    pub row_time: Vec<f64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ImageInfo {
    pub row_length: Vec<u32>,
    pub row_time: Vec<f64>,
    pub sprite_height: u32,
    pub sprite_width: u32,
}

impl Image {
    //Creates a new Image with a given file name, and amount of divions in the spritesheet
    pub fn new(path: String) -> Self {
        //Excellent code
        let png_name = format!("{}.png", path);

        let decoder = png::Decoder::new(File::open(png_name).unwrap());
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let bytes_arr = &buf[..info.buffer_size()];

        let mut bytes = bytes_arr.to_vec();

        flip_pixels_x_axis(&mut bytes, info.width as usize, info.height as usize);
        
        let info_name = format!("{}.info", path);

        let mut file = File::open(info_name).unwrap();
        let mut contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut contents).unwrap();
    
        let image_info: ImageInfo = bincode::deserialize(&contents[..]).unwrap();

        Self {
            name: path,
            bytes: bytes,
            sprite_height: image_info.sprite_height,
            image_width: info.width,
            sprite_width: image_info.sprite_width,
            row_length: image_info.row_length,
            row_time: image_info.row_time,
        }
    }
}

fn flip_pixels_x_axis(pixels: &mut Vec<u8>, width: usize, height: usize) {
    for y in 0..height / 2 {
        for x in 0..width {
            let top_index = (y * width + x) * 4;
            let bottom_index = ((height - y - 1) * width + x) * 4;
            for i in 0..4 {
                pixels.swap(top_index + i, bottom_index + i);
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Pixel {
    pub x: i32,
    pub y: i32,
    pub rgba: [u8; 4],
}

/// World camera, coordinates are the bottom left of camera
pub struct Camera {
    pub x: i32,
    pub y: i32,
}
