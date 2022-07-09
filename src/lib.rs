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

        //draws grapple spot
        if self.grappled{
            let x_relative = self.grapple_loc.0 - camera.x;
            let y_relative = self.grapple_loc.1 - camera.y;
            if x_relative > 0 && x_relative < 320 && y_relative < 180 && y_relative > 0 {
                block.push(Pixel {
                    x: x_relative,
                    y: y_relative,
                    rgba: [255, 0, 0, 255],
                })
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

        let mut mouse_angle = (y_diff/x_diff).atan();

        if x_diff >= 0.0 {
            mouse_angle += 3.141;
        }

        let circ_x = (mouse_angle.cos() * 15.0) + self.coord_x - camera.x as f64;
        let circ_y = (mouse_angle.sin() * 15.0) + self.coord_y - camera.y as f64;

        //draws grapple hook
        block.push(Pixel {
            x: circ_x as i32,
            y: circ_y as i32,
            rgba: [0, 0, 255, 255],
        });

        //draws grapple hook rope
        if self.grappled {
            let y_diff_grapple = circ_y - self.grapple_loc.1 as f64;
            let x_diff_grapple = self.grapple_loc.0 as f64 - circ_x;

            println!("Slope: {}", slope);
            println!("Xdiff: {}", x_diff_grapple);

            for x in 1..x_diff_grapple.abs() as i32 {
                block.push(Pixel {
                    x: circ_x as i32 + x,
                    y: (slope * x as f64) as i32 + circ_y as i32,
                    rgba: [0, 0, 255, 255],
                });
            }
        }

        return block;
    }

    fn update(&mut self) {
        //gravity and ground
        self.coord_x += self.velocity_x;
        self.coord_y += self.velocity_y;

        println!("X: {} Y: {}", self.coord_x, self.coord_y);
        println!("XVEL: {} YVEL: {}", self.velocity_x, self.velocity_y);

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