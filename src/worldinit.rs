use crate::Image;
use std::collections::HashMap;
use std::fs;

/// Returns HashMap of images read from the sprites folder
pub fn load_images() -> HashMap<String, Image> {
    let mut images = HashMap::new();
    let paths = fs::read_dir("sprites/").unwrap();

    //Iterates over every file and if it ends in png, creates a new Image with that filename
    for path in paths {
        let path_string = path.unwrap().path().into_os_string().into_string().unwrap();
        if &path_string[path_string.len()-3..path_string.len()] == "png"{
            let path_vec: Vec<&str> = path_string.split(".").collect();

            let image = Image::new(String::from(format!("{}",&path_vec[0])));
        
            images.insert(format!("{}", path_vec[0].replace("sprites/","")), image);
        }
    }
    if cfg!(debug_assertions) {
        println!("Images Loaded!");
    } 
    return images;
}
