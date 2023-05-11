use gametesting::Collision;
use gametesting::Coordinates;
use gametesting::Collider;
use crate::Camera;
use std::borrow::BorrowMut;
use std::time::Instant;
use std::cell::RefMut;
use std::collections::HashMap;

/// Simulates one frame of physics
pub fn simulate_frame(
    last_updated: &Instant,
    colliders: &mut RefMut<Vec<Option<Collider>>>,
    coordinates: &mut RefMut<Vec<Option<Coordinates>>>,
    renderable_entities: &HashMap<i32,i32>,
    camera: &Camera) {
    let mut id_grid: [[Vec<usize>; 5]; 6] = Default::default();
    let mut id_active = Vec::new();
    
    let zip = colliders.iter_mut().zip(coordinates.iter_mut());
    let mut entities= zip.filter_map(|(health, name)| Some((health.as_mut()?, name.as_mut()?)));
    
    // puts colliders into a grid system
    for (index, (collider, coordinate)) in  entities.borrow_mut().enumerate() {
            let width = collider.boundary.2;
            let height = collider.boundary.3;
            
            let x_rel = coordinate.coord_x as i32 - camera.x;
            let y_rel = coordinate.coord_y as i32 - camera.y;
            
            let start_col = x_rel / 71;
            let start_row = y_rel / 48;
            
            let end_col = (x_rel + width as i32) / 71;
            let end_row = (y_rel + height as i32) / 48;
            
            for col in start_col..=end_col {
                for row in start_row..=end_row {
                    if col < 6 {
                        id_grid[col as usize][row as usize].push(index);
                    }
                }
            }
            if collider.active {
                id_active.push(index);
            }
    }
    
    let zip = coordinates.iter_mut().zip(colliders.iter_mut());
    let mut entities: Vec<(&mut Coordinates, &mut Collider)>= zip.filter_map(|(health, name)| Some((health.as_mut()?, name.as_mut()?))).into_iter().collect();

    // performs gravity on rigid_bodies and applies velocity to active colliders
    for element in id_active {
        let element = element.clone();
            if entities[element].1.rigid_body {
                entities[element].1.vel_y -= 0.0001;
            }
            if entities[element].1.active {
                entities[element].0.coord_x += entities[element].1.vel_x;
                entities[element].0.coord_y += entities[element].1.vel_y;
            }
    }         

    entities[0].1.grounded = None;
    
    // Checks if the player is colliding with any of the colliders in that grid square
    for x in 0..6 {
        for y in 0..5 {
            if !id_grid[x][y].is_empty() && id_grid[x][y][0] == 0 {
                for element in id_grid[x][y].iter() {
                    if element.clone() != 0 as usize {
                        let entity1:(&Coordinates, &Collider) = (entities[0].0, entities[0].1);
                        let entity2:(&Coordinates, &Collider) = (entities[element.clone()].0, entities[element.clone()].1);
                        match box_collision(entity1, entity2) {
                            Some(Collision::Left) => {
                                entities[0].0.coord_x = entity2.0.coord_x + entity2.1.boundary.2;
                                entities[0].1.vel_x = 0.0;
                                entities[0].1.grounded = Some(Collision::Left);
                            },
                            Some(Collision::Right) => {
                                entities[0].0.coord_x = entity2.0.coord_x - entities[0].1.boundary.2;
                                entities[0].1.vel_x = 0.0;
                                entities[0].1.grounded = Some(Collision::Right);
                            },
                            Some(Collision::Up) => {
                                entities[0].0.coord_y = entity2.0.coord_y - entities[0].1.boundary.3;
                                entities[0].1.vel_y = 0.0;
                                entities[0].1.grounded = Some(Collision::Up);
                            },
                            Some(Collision::Down) => {
                                entities[0].0.coord_y = entity2.0.coord_y + entity2.1.boundary.3;
                                entities[0].1.vel_y = 0.0;
                                entities[0].1.grounded = Some(Collision::Down);
                            },
                            None => {

                            },
                        }
                    }
                }
            }
        }
    }
}

// checks if two boxes are colliding, and returns the side they are colliding on if true
fn box_collision(entity1: (&Coordinates, &Collider), entity2: (&Coordinates, &Collider)) -> Option<Collision>{
    if  entity1.0.coord_x < entity2.0.coord_x + entity2.1.boundary.2 &&
        entity1.0.coord_x + entity1.1.boundary.2 > entity2.0.coord_x &&
        entity1.0.coord_y < entity2.0.coord_y + entity2.1.boundary.3 &&
        entity1.0.coord_y + entity1.1.boundary.3 > entity2.0.coord_y {
        
        let player_half_w = entity1.1.boundary.2/2.0;
        let player_half_h = entity1.1.boundary.3/2.0;
        let object_half_w = entity2.1.boundary.2/2.0;
        let object_half_h = entity2.1.boundary.3/2.0;
        let player_center_x =  entity1.0.coord_x + player_half_w;
        let player_center_y =  entity1.0.coord_y + player_half_h;
        let object_center_x = entity2.0.coord_x + object_half_w;
        let object_center_y = entity2.0.coord_y + object_half_h;
                            
        let diff_x = player_center_x - object_center_x;
        let diff_y = player_center_y - object_center_y;
                            
        let min_x_distance = player_half_w + object_half_w;
        let min_y_distance = player_half_h + object_half_h;
                            
        let depth_x = if diff_x > 0.0 {min_x_distance - diff_x} else {-min_x_distance - diff_x};
        let depth_y = if diff_y > 0.0 {min_y_distance - diff_y} else {-min_y_distance - diff_y};
                            
        if depth_x != 0.0 && depth_x != 0.0 {
            if depth_x.abs() < depth_y.abs() {          
                if depth_x > 0.0 {
                    return Some(Collision::Left)
                } else {
                    return Some(Collision::Right)
                }
            } else {
                if depth_y > 0.0 {
                    return Some(Collision::Down)
                } else {
                    return Some(Collision::Up)
                }
            }
        } else {
            println!("COLLISION ERROR");
        }
    }
    return None;
}