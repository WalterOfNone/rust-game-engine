use rand::prelude::*;
use rand::rngs::SmallRng;
use std::collections::{BinaryHeap, HashMap, HashSet};

#[derive(Clone, Debug)]
// Tile that represents a room. 0 Indicates an open pathway for a given direction.
pub struct Tile {
    pub up: u8,
    pub down: u8,
    pub left: u8,
    pub right: u8
}

// Generates a tiled maze
pub fn gen_maze(seed: &mut u64, height: usize, width: usize, rand_level: f64) -> Vec<Vec<Option<Tile>>>{
    let mut tiles = vec![vec![None; width]; height];
    let mut active: Vec<(usize, usize)> = vec![(0,0)];

    tiles[0][0] = Some(Tile { up: 0, down: 0, left: 0, right: 0});

    while !active.is_empty() {
        let mut rng = SmallRng::seed_from_u64(seed.clone());
        *seed += 1;

        // The active tile is chosen from a random tile or from the last active tile
        // rand_level is the % chance a random tile is chosen
        let active_index = if rand_level > rng.gen_range(0.0..1.0) {
            rng = SmallRng::seed_from_u64(seed.clone());
            *seed += 1;
            rng.gen_range(0..active.len())
        } else {
            active.len() - 1
        };
        let active_pos = active[active_index];
        
        let empty_tiles = get_adjacent_empty(active_pos, &tiles);
        if !empty_tiles.is_empty() {
            *seed += 1;
            rng = SmallRng::seed_from_u64(seed.clone());
            // Neighboring empty tile is chosen at random
            let empty_tile_index = rng.gen_range(0..empty_tiles.len());
            let empty_tile_location = empty_tiles[empty_tile_index];

            // The empty tile is assigned a tile, and a path is made to it
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

// Returns a list of positions for the tiles empty and adjacent to the given tile //refactor this
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

// Gets all tiles that a tile at the given position is able to navigate to
fn get_adjacent_accessible(nodepos: &(usize, usize), nodes: &Vec<Vec<Option<Tile>>>) -> Vec<(usize, usize)> {
    let mut adjacent_nodes = Vec::new();
    
    let tile= nodes[nodepos.0][nodepos.1].as_ref().unwrap();
    // Adds positions of other nodes to adjacent_nodes if the tile has a path to them
    if tile.left > 0    {adjacent_nodes.push((nodepos.0 - 1, nodepos.1))};
    if tile.right > 0   {adjacent_nodes.push((nodepos.0 + 1, nodepos.1))};
    if tile.up > 0      {adjacent_nodes.push((nodepos.0, nodepos.1 + 1))};
    if tile.down > 0    {adjacent_nodes.push((nodepos.0, nodepos.1 - 1))};
    
    return adjacent_nodes;
}

// Returns manhattan distance of given points
fn manhattan_dist(start: &(usize, usize), end: &(usize, usize)) -> usize {
    return end.0.abs_diff(start.0) + end.1.abs_diff(start.1);
}

// Returns a vec of node positions that the A* algorithm pathed
fn reconstruct_path(came_from: &HashMap<(usize, usize), (usize, usize)>, end: (usize, usize)) -> Vec<(usize, usize)> {
    let mut total_path = Vec::new();
    let mut current = end.clone();
    
    while let Some(&previous_node) = came_from.get(&current) {
        total_path.insert(0, previous_node);
        current = previous_node;
    }
    
    total_path.push(end);
    
    return total_path;
}

// Paths maze using A* algorithm
pub fn path_maze(maze: &Vec<Vec<Option<Tile>>>, start: &(usize, usize), end: &(usize, usize)) -> Vec<(usize, usize)> {
    // Creates open set to manage currently open nodes, open_set_set is to manage which nodes are in such a set
    let mut open_set: BinaryHeap<(usize, (usize, usize))> = BinaryHeap::new();
    let mut open_set_set: HashSet<(usize, usize)> = HashSet::new();
    open_set.push((0, *start));
    open_set_set.insert(*start);
    
    // Map of nodes to their previous node
    let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    
    // Map of costs to reach a given node
    let mut g_score: HashMap<(usize, usize), usize> = HashMap::new();
    g_score.insert(*start, 0);
    
    // Map of (g_score + heuristic estimate) - used to determine which node it picks
    let mut f_score = HashMap::new();
    f_score.insert(*start, manhattan_dist(&start, &end));
    
    // Picks open node with smallest f_score
    while let Some((_, current)) = open_set.pop() {
        if current == *end {
            return reconstruct_path(&came_from, current);
        }
        
        for accessible in get_adjacent_accessible(&current, maze) {
            // Sets tentative_g_score to distance from starting node to neighbor through the current node
            let tentative_g_score = g_score.get(&current).unwrap_or_else(|| &usize::MAX) + 1;
            // With a lower g_score, the g_score + f_score + came_from map is updated to reflect the shortest path to that neighbor
            if tentative_g_score < *g_score.get(&accessible).unwrap_or_else(|| &usize::MAX) {
                
                came_from.insert(accessible, current);
                g_score.insert(accessible, tentative_g_score);
                
                let heuristic = manhattan_dist(&accessible, end);
                f_score.insert(accessible, tentative_g_score + heuristic);

                if !open_set_set.contains(&accessible) {
                    open_set.push((f_score[&accessible], accessible));
                    open_set_set.insert(accessible);
                }
            }
        }
    }
    //Returns blank vector if no path found
    return Vec::new();
}