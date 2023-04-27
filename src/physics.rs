use gametesting::Collision;
use gametesting::Coordinates;
use gametesting::Collider;

use crate::{Entity, Camera};
use std::time::Instant;
use std::cell::RefMut;
use std::collections::HashMap;

pub fn simulate_frame(last_updated: &Instant, colliders: &mut RefMut<Vec<Option<Collider>>>, coordinates: &mut RefMut<Vec<Option<Coordinates>>>, renderable_entities: &HashMap<i32,i32>, camera: &Camera) {
    let mut id_grid: [[Vec<usize>; 5]; 6] = Default::default();
    
    let zip = colliders.iter_mut().zip(coordinates.iter_mut());
    let mut entities: Vec<(&mut Collider, &mut Coordinates)> = zip.filter_map(|(health, name)| Some((health.as_mut()?, name.as_mut()?))).collect();
    //todo: make the id_grid have objects properly placed in their respective boxes
    
    //Likely inefficient, test if you see this
    for (index, (collider, coordinate)) in  entities.iter().enumerate() {
            let x_rel = coordinate.coord_x as i32 - camera.x;
            let y_rel = coordinate.coord_y as i32 - camera.y;
            
            let col = x_rel / 71;
            let row = y_rel / 48;
            //println!("num: {}", index);
            if row < 6 && col < 6 {
                id_grid[col as usize][row as usize].push(index.clone());
            }
    }
    
    for x in 0..6 {
        for y in 0..5 {
            if !id_grid[x][y].is_empty() && id_grid[x][y][0] == 0 {
                for element in id_grid[x][y].iter() {
                    if element.clone() != 0 as usize {
                        if  entities[0].1.coord_x < entities[element.clone()].1.coord_x + entities[element.clone()].0.boundary.2 &&
                            entities[0].1.coord_x + entities[0].0.boundary.2 > entities[element.clone()].1.coord_x &&
                            entities[0].1.coord_y < entities[element.clone()].1.coord_y + entities[element.clone()].0.boundary.3 &&
                            entities[0].1.coord_y + entities[0].0.boundary.3 > entities[element.clone()].1.coord_y 
                         {
                            let player_half_w = entities[0].0.boundary.2/2.0;
                            let player_half_h = entities[0].0.boundary.3/2.0;
                            let object_half_w = entities[element.clone()].0.boundary.2/2.0;
                            let object_half_h = entities[element.clone()].0.boundary.3/2.0;
                            let player_center_x =  entities[0].1.coord_x + player_half_w;
                            let player_center_y =  entities[0].1.coord_y + player_half_h;
                            let object_center_x = entities[element.clone()].1.coord_x + object_half_w;
                            let object_center_y = entities[element.clone()].1.coord_y + object_half_h;
                            
                            let diff_x = player_center_x - object_center_x;
                            let diff_y = player_center_y - object_center_y;
                            
                            let min_x_distance = player_half_w + object_half_w;
                            let min_y_distance = player_half_h + object_half_h;
                            
                            let depth_x = if diff_x > 0.0 {min_x_distance - diff_x} else {-min_x_distance - diff_x};
                            let depth_y = if diff_y > 0.0 {min_y_distance - diff_y} else {-min_y_distance - diff_y};
                            
                            if depth_x != 0.0 && depth_x != 0.0 {
                                if depth_x.abs() < depth_y.abs() {
                                    
                                    if depth_x > 0.0 {
                                        //println!("LEFT COLLISION");
                                        entities[0].0.vel_x = 0.0;
                                        entities[0].1.coord_x = entities[element.clone()].1.coord_x + entities[element.clone()].0.boundary.2;
                                        entities[0].0.grounded = Some(Collision::Left);
                                    } else {
                                        //println!("RIGHT COLLISION");
                                        entities[0].0.vel_x = 0.0;
                                        entities[0].1.coord_x = entities[element.clone()].1.coord_x - entities[0].0.boundary.2 - 0.01;
                                        entities[0].0.grounded = Some(Collision::Right);
                                    }
                                } else {
                                    if depth_y > 0.0 {
                                        //println!("BOTTOM COLLISION");
                                        entities[0].0.vel_y = 0.0;
                                        entities[0].1.coord_y = entities[element.clone()].1.coord_y + entities[element.clone()].0.boundary.3 + 0.01;
                                        entities[0].0.grounded = Some(Collision::Down);
                                        //println!("GROUNDED");
                                    } else {
                                        //println!("TOP COLLISION");
                                        entities[0].0.vel_y = 0.0;
                                        entities[0].1.coord_y = entities[element.clone()].1.coord_y - entities[0].0.boundary.3;
                                    }
                                }
                            } 
                        } else {
                            entities[0].1.coord_x += entities[0].0.vel_x;
                            entities[0].1.coord_y += entities[0].0.vel_y;
                            entities[0].0.vel_y -= 0.0001;
                            //entities[0].0.grounded = None;
                        }
                    } else {
                        entities[0].1.coord_x += entities[0].0.vel_x;
                        entities[0].1.coord_y += entities[0].0.vel_y;
                        entities[0].0.vel_y -= 0.0000;
                        //entities[0].0.grounded = None;
                    }
                }
            }
        }
    }
}