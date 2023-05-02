//use gametesting::Camera;
use gametesting::Coordinates;
use gametesting::Sprite;

use std::cell::RefMut;
use std::cell::Ref;
use std::collections::HashMap;
use std::time::Instant;
use std::borrow::BorrowMut;
const GAME_HEIGHT: usize = 240;
const GAME_WIDTH: usize = 426;
use crate::Image;
use crate::Camera;

pub fn render_frame(
    last_updated: &Instant,
    sprites: &mut RefMut<Vec<Option<Sprite>>>,
    coordinates: & Ref<Vec<Option<Coordinates>>>,
    images: &HashMap<String, Image>,
    frame: &mut [u8],
    camera: &Camera,
) {
        let mut pre_buffer: Vec<u8> = vec![80; GAME_WIDTH * GAME_HEIGHT * 4]; 
        
        let zip = sprites.iter_mut().zip(coordinates.iter());
        let mut both = zip.filter_map(|(health, name)| Some((health.as_mut()?, name.as_ref()?)));
        
        for (sprite, coordinates) in both.borrow_mut()
        {
            if sprite.visible {
                let image = &images[sprite.sprite];
                
                //Sets where the sprite exists relative to the camera
                let mut x_rel = coordinates.coord_x as i32 - camera.x;
                let mut y_rel = coordinates.coord_y as i32 - camera.y;
                
                let mut sprite_start_x = (sprite.sprite_state.0 * image.sprite_width) as i32;
                let mut sprite_end_x = image.sprite_width as i32 + sprite_start_x;
                
                if x_rel < 0 {
                    sprite_start_x += x_rel.abs();
                }
                
                if x_rel + image.sprite_width as i32 + 1 > GAME_WIDTH as i32 {
                    sprite_end_x -= x_rel + image.sprite_width as i32 - GAME_WIDTH as i32;
                }
                
                let mut sprite_start_y: i32 = (sprite.sprite_state.1 * image.sprite_height) as i32;
                let mut sprite_end_y = image.sprite_height as i32 + sprite_start_y;
                
                if y_rel < 0 {
                    sprite_start_y += y_rel.abs();
                }
                
                if y_rel + image.sprite_height as i32 + 1 > GAME_HEIGHT as i32 {
                    sprite_end_y -= y_rel + image.sprite_height as i32 - GAME_HEIGHT as i32;
                }
                
                x_rel -= (sprite.sprite_state.0 * image.sprite_width) as i32;
                y_rel -= (sprite.sprite_state.1 * image.sprite_height) as i32;

                for x in (sprite_start_x + x_rel)..(sprite_end_x + x_rel - 0) {
                    for y in (sprite_start_y + y_rel)..(sprite_end_y + y_rel) {
                        let index = ((y * GAME_WIDTH as i32 + x) * 4) as usize;
                        let mut location = ((x - x_rel + image.image_width as i32 * (y - y_rel)) * 4) as usize;
                        if sprite.reversed {
                            location = ((x_rel - x - 1+ image.image_width as i32 * (y - y_rel + 1)) * 4) as usize;
                        }
                        //println!("{}", location)
                        if !sprite.fade{
                            if image.bytes[location + 3] == 255 {
                                //Copies pixel value directly from sprite
                                pre_buffer[index..index + 4].copy_from_slice(&image.bytes[location..(location + 4)]);
                            } else if image.bytes[location + 3] == 0 {
                                //Adds no pixel
                            } else {
                                //Adds pixel with alpha calculation
                                let src: &[u8; 4] = &pre_buffer[index..index + 4].try_into().unwrap();
                                let dst = &image.bytes[location..(location + 4)].try_into().unwrap();
                                let blended = blend_alpha_fast(src, dst);
                                pre_buffer[index..index + 4].copy_from_slice(&blended);
                            }
                        } else {
                            let src: &[u8; 4] = &pre_buffer[index..index + 4].try_into().unwrap();
                            let mut dst: [u8; 4] = [0; 4];
                            let slice = &image.bytes[location..(location + 3)];
                            dst[0..3].copy_from_slice(slice);
                            dst[3] = (sprite.time_left * image.bytes[location + 3] as f64 / 10.0) as u8;
                            let blended = blend_alpha_fast(src, &dst);
                            pre_buffer[index..index + 4].copy_from_slice(&blended);
                        }
                    }
                }
                sprite.time_left -= 0.001;
                if sprite.time_left <= 0.0 {
                    sprite.sprite_state.0 += 1;
                    sprite.sprite_state.0 %= image.row_length[sprite.sprite_state.1 as usize];
                    sprite.time_left = image.row_time[sprite.sprite_state.1 as usize];
                }
                
            }
        }

        //copies pixel array into current frame
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = (i % GAME_WIDTH as usize) as usize;
        let y = GAME_HEIGHT - 1 - i / GAME_WIDTH as usize;
        
        let index = ((y as i32 * GAME_WIDTH as i32 + x as i32) * 4) as usize;
    
        let rgba = &pre_buffer[index..index+4];
    
        pixel.copy_from_slice(rgba);
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

pub fn create_textbox(lookuptable: &Image, text: &String) -> Image{
    if !text.is_ascii() {
        println!("Invalid String!");
    }
    
    let lookupbytes = &lookuptable.bytes;
    let mut bytes: Vec<u8> = Vec::new();
    let mut textbox_width = 0;
    let mut textbox_height = 2;
    
    let words = text.split("\n");

    for line in words.clone() {
        let line_width = (line.as_bytes().len() * 4) as u32;
        if line_width > textbox_width {
            textbox_width = line_width;
        }
    }
    
    for line in words {
        let line_width = (line.as_bytes().len() * 4) as u32;
        for y in 0..7 {
            for char in line.as_bytes() {
                let lookup_val = (((char.clone() as u32 - 32) * 12) + (y * lookuptable.image_width) * 4) as usize;
                //adds that row of the character
                bytes.extend_from_slice(&lookupbytes[lookup_val..lookup_val+12]);
                //adds space after character
                bytes.append(&mut vec![0u8; 4]);
            }
            if line_width < textbox_width {
                    let spacer = textbox_width - line_width;
                    println!("Spacer: {}", spacer);
                    bytes.append(&mut vec![0u8; (4*(textbox_width - line_width)) as usize]);
            }
        }
        textbox_height += 7;
    }
    textbox_width += 1;
    
    let mut textbox_sprite_bytes: Vec<u8> = Vec::new();
    for y in 0..textbox_height {
        let horizontal_wall: bool = y == 0 || y == textbox_height - 1;
        
        for x in 0..textbox_width {
            let vertical_wall: bool = x == 0 || x == textbox_width - 1;

            if horizontal_wall && vertical_wall {
                textbox_sprite_bytes.append(&mut vec![0u8; 4]);
            } else if vertical_wall || horizontal_wall {
                textbox_sprite_bytes.append(&mut vec![120,120,120,255]);
            } else {
                if x > 0 && x < textbox_width - 1 && y > 0 && y < textbox_height - 1{
                    let text_x = x - 1;
                    let text_y = y - 1;
                    if bytes[((text_x * 4) + text_y * (textbox_width - 1) * 4) as usize + 3] != 0 {
                        textbox_sprite_bytes.append(&mut vec![0,0,0,255]);
                    } else {
                        textbox_sprite_bytes.append(&mut vec![70,70,70,255]);
                    }
                } else {
                    textbox_sprite_bytes.append(&mut vec![255u8; 4]);
                }
            }

        }
    }
    
    let textbox_image = Image {
        name: String::from("textbox"),
        bytes: textbox_sprite_bytes,
        sprite_width: textbox_width,
        sprite_height: textbox_height,
        image_width: textbox_width,
        row_time: vec![1.0],
        row_length: vec![1],
    };
    
    return textbox_image
}