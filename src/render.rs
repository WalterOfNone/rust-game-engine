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
    let mut pre_buffer: [[[u8; 4]; GAME_HEIGHT]; GAME_WIDTH] = [[[100, 100, 100, 255]; GAME_HEIGHT]; GAME_WIDTH];
        //Iterates through all visible entities and places visible pixels on pre_buffer
        //TODO: multithread
    
        
        let zip = sprites.iter_mut().zip(coordinates.iter());
        let mut iter = zip.filter_map(|(health, name)| Some((health.as_mut()?, name.as_ref()?)));
        
        for (sprite, coordinates) in iter.borrow_mut()
        {
            if sprite.visible {
                let image = &images[sprite.sprite];
                //let image = sprites[sprite.sprite];
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
                //println!("{}", y_rel);

                for x in (sprite_start_x + x_rel)..(sprite_end_x + x_rel - 0) {
                    for y in (sprite_start_y + y_rel)..(sprite_end_y + y_rel) {
                        //can't I just memcpy the whole damn thing??? //no
                        let location = ((x - x_rel + image.image_width as i32 * (y - y_rel)) * 4) as usize;
                        //println!("{}", location);
                        if !sprite.fade{
                            if image.bytes[location + 3] == 255 {
                                //Copies pixel value directly from sprite
                                pre_buffer[x as usize][y as usize].copy_from_slice(&image.bytes[location..(location + 4)]);
                            } else if image.bytes[location + 3] == 0 {
                                //Adds no pixel
                            } else {
                                //Adds pixel with alpha calculation
                                let src: &[u8; 4] = &pre_buffer[x as usize][y as usize][0..4].try_into().unwrap();
                                let dst = &image.bytes[location..(location + 4)].try_into().unwrap();
                                let blended = blend_alpha_fast(src, dst);
                                pre_buffer[x as usize][y as usize][0] = blended[0];
                                pre_buffer[x as usize][y as usize][1] = blended[1];
                                pre_buffer[x as usize][y as usize][2] = blended[2];
                                pre_buffer[x as usize][y as usize][3] = blended[3];
                            }
                        } else {
                            let src: &[u8; 4] = &pre_buffer[x as usize][y as usize ][0..4].try_into().unwrap();
                            let mut dst: [u8; 4] = [0; 4]; // Initialize the array to zero
                            let slice = &image.bytes[location..(location + 3)];
                            dst[0..3].copy_from_slice(slice);
                            dst[3] = (sprite.time_left * image.bytes[location + 3] as f64 / 10.0) as u8;
                            let blended = blend_alpha_fast(src, &dst);
                            pre_buffer[x as usize][y as usize][0] = blended[0];
                            pre_buffer[x as usize][y as usize][1] = blended[1];
                            pre_buffer[x as usize][y as usize][2] = blended[2];
                            pre_buffer[x as usize][y as usize][3] = blended[3];
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
            let x = (i % GAME_WIDTH as usize) as i16;
            let y = (i / GAME_WIDTH as usize) as i16;

            let rgba = pre_buffer[x as usize][GAME_HEIGHT - 1 - y as usize];

            pixel.copy_from_slice(&rgba);
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