use crate::Image;
use std::collections::HashMap;
use std::fs;

/// Returns HashMap of images read from the sprites folder, with the numbers in the filename removed by "_"
pub fn load_images() -> HashMap<String, Image> {
    let mut images = HashMap::new();
    let paths = fs::read_dir("sprites/").unwrap();

    for path in paths {
        let path_string = path.unwrap().path().into_os_string().into_string().unwrap();
        let path_vec: Vec<&str> = path_string.split("_").collect();

        let image = Image::new(String::from(format!("{}",&path_string)), path_vec[1].parse::<u32>().unwrap());
        
        images.insert(format!("{}.png", path_vec[0].replace("sprites/","")), image);
    }
    if cfg!(debug_assertions) {
        println!("Images Loaded!");
    } 
    return images;
}
