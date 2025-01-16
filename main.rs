use argh::FromArgs;
use image::{ImageError, Luma, Rgb, RgbImage, Pixel};

#[derive(Debug, Clone, PartialEq, FromArgs)]
/// Convertit une image en monochrome ou vers une palette réduite de couleurs.
struct DitherArgs {

    /// le fichier d’entrée
    #[argh(positional)]
    input: String,

    /// le fichier de sortie (optionnel)
    #[argh(positional)]
    output: Option<String>,

    /// le mode d’opération
    #[argh(subcommand)]
    mode: Mode
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand)]
enum Mode {
    Seuil(OptsSeuil),
    Palette(OptsPalette),
    Dithering(OptsDithering),
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="seuil")]
/// Rendu de l’image par seuillage monochrome.
struct OptsSeuil {}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de l’image avec une palette contenant un nombre limité de couleurs
struct OptsPalette {

    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="dithering")]
/// Rendu de l’image en dithering.
struct OptsDithering {}

const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
const GREY: Rgb<u8> = Rgb([127, 127, 127]);
const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
const BLUE: Rgb<u8> = Rgb([0, 0, 255]);
const RED: Rgb<u8> = Rgb([255, 0, 0]);
const GREEN: Rgb<u8> = Rgb([0, 255, 0]);
const YELLOW: Rgb<u8> = Rgb([255, 255, 0]);
const MAGENTA: Rgb<u8> = Rgb([255, 0, 255]);
const CYAN: Rgb<u8> = Rgb([0, 255, 255]);

fn get_image(path: String) -> Result<RgbImage, ImageError> {
    let img = image::open(path)?;
    let img = img.to_rgb8();
    Ok(img)
}

fn modify_image_seuil(mut img: RgbImage) -> Result<RgbImage, ImageError> {
    let (width, height) = img.dimensions();
    for x in 0..width {
        for y in 0..height {
            let Luma(luminosite_) = img.get_pixel(x, y).to_luma();
            if luminosite_[0] > 127 {
                img.put_pixel(x, y, WHITE);
            } else {
                img.put_pixel(x, y, BLACK);
            }
        }
    }
    Ok(img)
}

fn modify_image_palette(mut img: RgbImage, n_couleurs: usize) -> Result<RgbImage, ImageError> {
    let (width, height) = img.dimensions();
    
    // Original palette with 9 colors
    let mut palette = vec![BLACK, GREY, WHITE, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA];
    
    // Clamp n_couleurs to the size of the palette
    let n_couleurs = n_couleurs.min(palette.len());
    
    // Reduce the palette to n_couleurs colors
    palette = palette.drain(0..n_couleurs).collect::<Vec<Rgb<u8>>>();
    
    for x in 0..width {
        for y in 0..height {
            let pixel = img.get_pixel(x, y);
            let mut best_distance = f64::INFINITY;
            let mut best_color = BLACK;
            for color in palette.iter() {
                let distance = (color[0] as f64 - pixel[0] as f64).powi(2) + (color[1] as f64 - pixel[1] as f64).powi(2) + (color[2] as f64 - pixel[2] as f64).powi(2);
                if distance < best_distance {
                    best_distance = distance;
                    best_color = *color;
                }
            }
            img.put_pixel(x, y, best_color);
        }
    }

    Ok(img)
}

fn modify_image_dithering(mut img: RgbImage) -> Result<RgbImage, ImageError> {
    let (width, height) = img.dimensions();

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let avg_color = (pixel[0] as f64 + pixel[1] as f64 + pixel[2] as f64) / 3.0;
            let new_color = if avg_color > 128.0 { WHITE } else { BLACK };

            let error = [
                pixel[0] as f64 - new_color[0] as f64,
                pixel[1] as f64 - new_color[1] as f64,
                pixel[2] as f64 - new_color[2] as f64,
            ];

            img.put_pixel(x, y, new_color);

            // Floyd-Steinberg error diffusion
            if x + 1 < width {
                let neighbor = img.get_pixel_mut(x + 1, y);
                neighbor[0] = (neighbor[0] as f64 + error[0] * 7.0 / 16.0) as u8;
                neighbor[1] = (neighbor[1] as f64 + error[1] * 7.0 / 16.0) as u8;
                neighbor[2] = (neighbor[2] as f64 + error[2] * 7.0 / 16.0) as u8;
            }
            if x > 0 && y + 1 < height {
                let neighbor = img.get_pixel_mut(x - 1, y + 1);
                neighbor[0] = (neighbor[0] as f64 + error[0] * 3.0 / 16.0) as u8;
                neighbor[1] = (neighbor[1] as f64 + error[1] * 3.0 / 16.0) as u8;
                neighbor[2] = (neighbor[2] as f64 + error[2] * 3.0 / 16.0) as u8;
            }
            if y + 1 < height {
                let neighbor = img.get_pixel_mut(x, y + 1);
                neighbor[0] = (neighbor[0] as f64 + error[0] * 5.0 / 16.0) as u8;
                neighbor[1] = (neighbor[1] as f64 + error[1] * 5.0 / 16.0) as u8;
                neighbor[2] = (neighbor[2] as f64 + error[2] * 5.0 / 16.0) as u8;
            }
            if x + 1 < width && y + 1 < height {
                let neighbor = img.get_pixel_mut(x + 1, y + 1);
                neighbor[0] = (neighbor[0] as f64 + error[0] * 1.0 / 16.0) as u8;
                neighbor[1] = (neighbor[1] as f64 + error[1] * 1.0 / 16.0) as u8;
                neighbor[2] = (neighbor[2] as f64 + error[2] * 1.0 / 16.0) as u8;
            }
        }
    }

    Ok(img)
}

fn main() -> Result<(), ImageError>{
    let args: DitherArgs = argh::from_env();
    let path_in = args.input;
    let path_out = args.output.unwrap_or_else(|| "out.png".to_string());
    let mode = args.mode;
    
    let img = get_image(path_in)?;

    match mode {
        Mode::Seuil(_) => {
            let image = modify_image_seuil(img)?;
            image.save(path_out)?;
        }
        Mode::Palette(opts) => {
            let image = modify_image_palette(img, opts.n_couleurs)?;
            image.save(path_out)?;
        }
        Mode::Dithering(_) => {
            let image = modify_image_dithering(img)?;
            image.save(path_out)?;
        }
    }

    Ok(())
}
