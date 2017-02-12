extern crate rand;

use std::mem;

use rand::Rng;

// Struct definition for a species
#[derive(Debug)]
struct Species {
    // Name of a species. This should only be one or two characters long.
    name: String,

    // Colour of a species. This only specifies the hue.
    // The saturation and lightness are hardcoded (however lightness changes
    // when a pixel takes over another pixel).
    color: u8,

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
}

impl Species {
    fn new(name: Option<String>, color: Option<u8>) -> Species {
        let mut rng = rand::thread_rng();
        Species {
            name: name.unwrap_or(rng.gen_ascii_chars().take(2).collect()),
            color: color.unwrap_or(rng.gen()),
            health: rng.gen(),
            strength: rng.gen(),
            desire: rng.gen(),
            frequency: rng.gen(),
            expectancy: rng.gen()
        }
    }

    fn new_vec(size: usize) -> Vec<Species> {
        // Generate 2-char strings from AA to ZZ
        let name_iter = (b'A'..b'Z').flat_map(move |a|
            (b'A'..b'Z').map(move |b| format!("{}{}", a as char, b as char))
        );

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
    pixels: Vec<Pixel<'a>>
}

impl<'a> PixelBoard<'a> {
    pub fn new(width: usize, height: usize, species: &'a Vec<Species>) -> PixelBoard<'a> {
        let mut rng = rand::thread_rng();

        //let species: Vec<Species> = Species::new_vec(num_species);
        let mut pixels: Vec<Pixel<'a>> = Vec::with_capacity(width*height);
        for _ in 0..pixels.capacity() {
            let selected_species = rng.choose(&species).expect("Pixel could not choose a species");
            pixels.push(Pixel::new(selected_species));
        }
        PixelBoard{width: 0, pixels: pixels}
    }

    // Return width and height
    /*pub fn get_rect(&self) -> (usize, usize) {
        (self.width, self.pixels.len()/self.width)
    }*/
}


fn main() {
    // For testing purposes, let's do a pixel board that's 2x2
    // and has 4 species
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
}
