extern crate rand;
extern crate palette;
extern crate piston;
extern crate graphics;
extern crate image;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate texture;

use std::mem;

use rand::Rng;
use palette::{Rgb, Hsl};
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use image::{ImageBuffer, Rgba, Pixel as imgPixel};
use texture::TextureSettings;

// Struct definition for a species
#[derive(Debug)]
struct Species {
    // Name of a species. This should only be one or two characters long.
    name: String,

    // Colour of a species. This only specifies the hue.
    // The saturation and lightness are hardcoded (however lightness changes
    // when a pixel takes over another pixel).
    color: i16,

    // How much health a pixel of a species starts with.
    // each pixel will lose health on successful attacks
    health: u16,

    // How powerful an attack is against an enemy.
    // If an enemy has more strength, they are more likely to win.
    // Within a certain threshold, a fight might result in a tie.
    // Additionally, if the strength plus a random value differs enough,
    // One pixel's strength might be immediately depleted.
    strength: u16,

    // How much a pixel wants to attack an enemy.
    // This value plus a random value must be above a threshold to
    // perform an attack against an enemy.
    desire: u16,

    // How often a pixel will attack an enemy.
    frequency: u16,

    // How long a pixel is expected to live.
    // A random value can be rolled which can cause a pixel to live
    // longer or shorter, however the threshold increases as the pixel
    // lives beyond its expectancy.
    expectancy: u16,

    // For speed and caching purposes, store the piston colour
    // along with the original colour
    piston_color: graphics::types::Color,
    image_color: Rgba<u8>
}

impl Species {
    fn new(name: Option<String>, color: Option<i16>) -> Species {
        let mut rng = rand::thread_rng();
        let color = color.unwrap_or(rng.gen_range(-180,180));
        let palette_color = Rgb::from(Hsl::new((color as f32).into(), 1.0, 0.5));
        let piston_color = [palette_color.red, palette_color.green, palette_color.blue, 1.0];
        let image_color = Rgba::from_channels(
            (palette_color.red * 256.0) as u8,
            (palette_color.green * 256.0) as u8,
            (palette_color.blue * 256.0) as u8,
            255
        );

        Species {
            name: name.unwrap_or(rng.gen_ascii_chars().take(2).collect()),
            color: color,
            health: rng.gen(),
            strength: rng.gen(),
            desire: rng.gen(),
            frequency: rng.gen(),
            expectancy: rng.gen(),

            piston_color: piston_color,
            image_color: image_color
        }
    }

    fn new_vec(size: usize) -> Vec<Species> {
        // Generate 2-char strings from AA to ZZ
        /*let name_iter = (b'A'..b'Z').flat_map(move |a|
            (b'A'..b'Z').map(move |b| format!("{}{}", a as char, b as char))
        );*/

        // Generate 1-char strings from A to Z
        let name_iter = (b'A'..b'Z').map(move |a| format!("{}", a as char));

        // Generate species with random colours and values
        name_iter.take(size).map(|n| Species::new(Some(n), None)).collect()
    }
}

// Struct definition for a pixel within a species
#[derive(Debug)]
struct Pixel<'a> {
    // Species that owns a pixel.
    // When a pixel is defeated, the health modifiers are random walked
    // from the successful pixel.
    species: &'a Species,

    // Each of the species' values has a modifier
    // that causes variation in all of the pixels.
    // when attacks occur, a random walk between
    // -5 and +5 occurs for the captured pixel.
    health_modifier: i8,
    strength_modifier: i8,
    desire_modifier: i8,
    frequency_modifier: i8,
    expectancy_modifier: i8
}

impl<'a> Pixel<'a> {
    fn new(species: &'a Species) -> Pixel<'a> {
        let mut rng = rand::thread_rng();
        Pixel{
            species: species,
            health_modifier: rng.gen_range(-10, 10),
            strength_modifier: rng.gen_range(-10, 10),
            desire_modifier: rng.gen_range(-10, 10),
            frequency_modifier: rng.gen_range(-10, 10),
            expectancy_modifier: rng.gen_range(-10, 10),
        }
    }
}

// Struct definition for the board that holds all the species and pixels
// (0,0) is the top-left corner
#[derive(Debug)]
struct PixelBoard<'a> {
    width: usize,
    height: usize,
    pixels: Vec<Pixel<'a>>
}

impl<'a> PixelBoard<'a> {
    pub fn new(width: usize, height: usize, species: &'a Vec<Species>) -> PixelBoard<'a> {
        let mut rng = rand::thread_rng();

        let mut pixels: Vec<Pixel<'a>> = Vec::with_capacity(width*height);
        for _ in 0..pixels.capacity() {
            let selected_species = rng.choose(&species).expect("Pixel could not choose a species");
            pixels.push(Pixel::new(selected_species));
        }
        PixelBoard{width: width, height: height, pixels: pixels}
    }

    fn get_rect(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn get_imagebuffer(&self) -> Option<ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        let (width, height) = self.get_rect();
        let storage = vec![0; 4 * self.pixels.len()];
        ImageBuffer::from_raw(width as u32, height as u32, storage)
            .map(|mut buf| {
                for (b, p) in buf.pixels_mut().zip(&self.pixels) {
                    *b = p.species.image_color;
                }
                buf
            })
    }
}

struct DisplayApp<'a> {
    gl: GlGraphics,
    pixel_board: &'a PixelBoard<'a>
}

impl<'a> DisplayApp<'a> {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        let pixel_board = self.pixel_board;

        self.gl.draw(args.viewport(), |c, gl| {
            clear(WHITE, gl);

            if let Some(board_image) = pixel_board.get_imagebuffer() {
                let image = Image::new().rect([0.0, 0.0, 800.0, 600.0]);
                let settings = TextureSettings::new();
                let texture = Texture::from_image(&board_image, &settings);
                image.draw(&texture, &DrawState::default(), c.transform, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {

    }
}

fn main() {
    let species = Species::new_vec(16);
    let pixel_board = PixelBoard::new(800, 600, &species);
    for s in &species {
        println!("{:?}", s);
    }
    /*for pixel in pixel_board.pixels {
        println!("{:?}", pixel);
    }*/
    println!("sizes: board: {}, species: {}, pixel: {}",
             mem::size_of::<PixelBoard>(),
             mem::size_of::<Species>(),
             mem::size_of::<Pixel>()
    );

    // Display the initial board
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new(
        "evo-thing",
        [1920,1080]
    )
    .opengl(opengl)
    .exit_on_esc(true)
    .build()
    .expect("Could not build OpenGL window");

    let mut display_app = DisplayApp {
        gl: GlGraphics::new(opengl),
        pixel_board: &pixel_board
    };

    let mut events = Events::new(EventSettings::new().max_fps(60));
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            display_app.render(&r);
        }
        if let Some(u) = e.update_args() {
            display_app.update(&u);
        }
    }
}
