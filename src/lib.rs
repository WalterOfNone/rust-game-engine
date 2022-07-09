pub trait Object {
    /// Returns pixels relative to the current camera
    fn get_pixels(&self, camera: &Camera, mouse_pos: &(i32,i32)) -> Vec<Pixel>;

    fn update(&mut self);
}

#[derive(Debug, Copy, Clone)]
pub struct Pixel {
    pub x: i32,
    pub y: i32,
    pub rgba: [u8; 4],
}

/// Player controlled object
pub struct Player {
    pub health: i8,
    pub coord_x: f64,
    pub coord_y: f64,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub grounded: bool,
    pub grappled: bool,
    pub grapple_loc: (i32, i32),
}

/// World camera, coordinates are the bottom left of camera
pub struct Camera {
    pub x: i32,
    pub y: i32,
}

impl Camera {
    pub fn update(&mut self, player: &Player) {
        if (player.coord_y as i32) > 90 {
            self.y = player.coord_y as i32 - 90;
        } else {
            self.y = 0;
        }

        if player.coord_x as i32 > 160 {
            self.x = player.coord_x as i32 - 160;
            
        } else {
            self.x = 0;
        }
    }
}

impl Object for Player {
    fn get_pixels(&self, camera: &Camera, mouse_pos: &(i32,i32)) -> Vec<Pixel> {

        let mut block = Vec::new();

        //draws player as block
        for x in 0..9 {
            for y in 0..9 {
                //if self.coord_x as i32 + x -4 < 320 && self.coord_y as i32 + y - 4 > 0 {
                    block.push(Pixel {
                        x: self.coord_x as i32 - camera.x + x - 4,
                        y: self.coord_y as i32 - camera.y + y - 4,
                        rgba: [x as u8, 80, 80, 255],
                    })
                //}
                
            }
        }

        //draws cursor position in green
        block.push(Pixel {
            x: mouse_pos.0 % 320,
            y: mouse_pos.1 - 1,
            rgba: [0,255,0,255],
        });

        //draws grapple spot
        if self.grappled{
            let x_relative = self.grapple_loc.0 - camera.x;
            let y_relative = self.grapple_loc.1 - camera.y;

            //checks if coordinates are in frame
            if x_relative > 0 && x_relative < 320 && y_relative < 180 && y_relative > 0 {
                block.push(Pixel {
                    x: x_relative,
                    y: y_relative,
                    rgba: [255, 0, 0, 255],
                });
            }
        }

        //draws grapple hook and rope when fired
        let mut y_diff = (self.coord_y - mouse_pos.1 as f64 - camera.y as f64) as f64;
        let mut x_diff = (self.coord_x - mouse_pos.0 as f64 - camera.x as f64) as f64;

        let mut slope = 1.0;

        if self.grappled {
            y_diff = (self.coord_y - self.grapple_loc.1 as f64) as f64;
            x_diff = (self.coord_x - self.grapple_loc.0  as f64) as f64;
            if x_diff != 0.0 {
                slope = y_diff/x_diff;
            }
        }

        //angle of the grapple hook relative to the player
        let mut mouse_angle = (y_diff/x_diff).atan();

        //increase the mouse angle by 1 rad to make a full circle
        if x_diff >= 0.0 {
            mouse_angle += 3.141;
        }

        let grapple_hook_x = (mouse_angle.cos() * 15.0) as i32 + self.coord_x as i32 - camera.x;
        let grapple_hook_y = (mouse_angle.sin() * 15.0) as i32 + self.coord_y as i32 - camera.y;

        //draws grapple hook
        if grapple_hook_x > 0 && grapple_hook_x < 320 && grapple_hook_y < 180 && grapple_hook_y > 0 {
            block.push(Pixel {
                x: grapple_hook_x as i32,
                y: grapple_hook_y as i32,
                rgba: [0,0,255,255],
            });
        }

        //draws grapple hook rope
        if self.grappled {
            let grapple_x_diff = self.grapple_loc.0 - grapple_hook_x - camera.x;
            for mut x in 0..grapple_x_diff.abs() {
                //goes in opposite direction if grapple spot is left of player
                if grapple_x_diff < 0 {
                    x *= -1;
                }
                let rope_x = grapple_hook_x as i32 + x;
                let rope_y = grapple_hook_y as i32 + (x as f64 * slope) as i32;

                if rope_x > 0 && rope_x < 320 && rope_y < 180 && rope_y > 0 {
                    block.push(Pixel {
                        x: rope_x,
                        y: rope_y,
                        rgba: [0,0,255,255],
                    });
                }
            }
        }

        return block;
    }

    fn update(&mut self) {
        //gravity and ground
        self.coord_x += self.velocity_x;
        self.coord_y += self.velocity_y;

        //println!("X: {} Y: {}", self.coord_x, self.coord_y);
        //println!("XVEL: {} YVEL: {}", self.velocity_x, self.velocity_y);

        //TODO: clean up this mess
        if self.coord_x < 4.0 {
            self.coord_x = 4.0;
        }

        if self.coord_y <= 4.0 {
            self.coord_y = 4.0;
            self.velocity_y = 0.0;
            self.grounded = true;
            //println!("grounded")
        } else {
            self.grounded = false;
            //println!("not grounded")
        }

        if self.grappled {
            let y_diff = self.coord_y - self.grapple_loc.1 as f64;
            let x_diff = self.coord_x - self.grapple_loc.0  as f64 ;
            self.velocity_x -= x_diff * 0.0005;
            self.velocity_y -= y_diff * 0.0007;
        }

        if self.coord_y > 4.1 {
            self.velocity_y -= 0.025;
        }

        if self.grounded {
            self.velocity_x *= 0.99;
        }
    }
}