use rand::prelude::*;
use rand::rngs::SmallRng;
use std::{thread, time};
use crate::display_tiles;

#[derive(Clone, Debug)]
pub struct Tile {
    pub up: u8,
    pub down: u8,
    pub left: u8,
    pub right: u8
}

// Generates a tiled maze
pub fn gen_maze(seed: &mut u64, height: usize, width: usize, rand_tiles: f64) -> Vec<Vec<Option<Tile>>>{
    let mut tiles = vec![vec![None; width]; height];
    let mut active: Vec<(usize, usize)> = vec![(0,0)];

    tiles[0][0] = Some(Tile {
        up: 0,
        down: 0,
        left: 0,
        right: 0,
        });

    while !active.is_empty() {
        //display_tiles(&tiles).unwrap();
        //let ten_millis = time::Duration::from_millis(7);
        //thread::sleep(ten_millis);
        let mut rng = SmallRng::seed_from_u64(seed.clone());
        *seed += 1;

        let num = rng.gen_range(0.0..1.0);
        let mut active_index = active.len() - 1 ;
        if num < rand_tiles {
            rng = SmallRng::seed_from_u64(seed.clone());
            *seed += 1;
            active_index = rng.gen_range(0..active.len());
        }
        let active_pos = active[active_index];

        let empty_tiles = get_adjacent_empty(active_pos, &tiles);
        if !empty_tiles.is_empty() {
            *seed += 1;
            let mut rng = SmallRng::seed_from_u64(seed.clone());
            let empty_tile_index = rng.gen_range(0..empty_tiles.len());
            let empty_tile_location = empty_tiles[empty_tile_index];

            match empty_tile_location {
                (x, y) if x < active_pos.0 => {
                    tiles[active_pos.0][active_pos.1].as_mut().unwrap().left = 1;
                    tiles[x][y] = Some(Tile {up: 0, down: 0, left: 0, right: 1}); },
                (x, y) if x > active_pos.0 => {
                    tiles[active_pos.0][active_pos.1].as_mut().unwrap().right = 1;
                    tiles[x][y] = Some(Tile {up: 0, down: 0, left: 1, right: 0}); },
                (x, y) if y < active_pos.1 => {
                    tiles[active_pos.0][active_pos.1].as_mut().unwrap().down = 1;
                    tiles[x][y] = Some(Tile {up: 1, down: 0, left: 0, right: 0}); },
                (x, y) if y > active_pos.1 => {
                    tiles[active_pos.0][active_pos.1].as_mut().unwrap().up = 1;
                    tiles[x][y] = Some(Tile {up: 0, down: 1, left: 0, right: 0}); },
                _=> unreachable!(),
            }
            active.push(empty_tile_location);
        } else {
            active.remove(active_index);
        }
    }
    return tiles;
}

// Returns a list of positions for the tiles empty and adjacent to the given tile
fn get_adjacent_empty(tilepos: (usize, usize), tiles: &Vec<Vec<Option<Tile>>>) -> Vec<(usize, usize)>{
    let mut tiles_adjacent = vec![
        (tilepos.0 + 1, tilepos.1),
        (tilepos.0, tilepos.1 + 1),
    ];
    
    if tilepos.0 > 0 {
        tiles_adjacent.push((tilepos.0 - 1, tilepos.1));
    }
    
    if tilepos.1 > 0 {
        tiles_adjacent.push((tilepos.0, tilepos.1 - 1));
    }
    
    let mut empty_tiles = Vec::new();

    for adjacent_tile in tiles_adjacent {
        if adjacent_tile.0 >= 0 && 
           adjacent_tile.0 < tiles.len() && 
           adjacent_tile.1 >= 0 && 
           adjacent_tile.1 < tiles[adjacent_tile.0].len() {
            match tiles[adjacent_tile.0][adjacent_tile.1] {
                Some(_) => {},
                None => empty_tiles.push((adjacent_tile.0, adjacent_tile.1)),
            }
        }
    }
    return empty_tiles;
}