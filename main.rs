use argh::FromArgs;
use image::ImageError;

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
 
const WHITE: image::Rgb<u8> = image::Rgb([255, 255, 255]);
const GREY: image::Rgb<u8> = image::Rgb([127, 127, 127]);
const BLACK: image::Rgb<u8> = image::Rgb([0, 0, 0]);
const BLUE: image::Rgb<u8> = image::Rgb([0, 0, 255]);
const RED: image::Rgb<u8> = image::Rgb([255, 0, 0]);
const GREEN: image::Rgb<u8> = image::Rgb([0, 255, 0]);
const YELLOW: image::Rgb<u8> = image::Rgb([255, 255, 0]);
const MAGENTA: image::Rgb<u8> = image::Rgb([255, 0, 255]);
const CYAN: image::Rgb<u8> = image::Rgb([0, 255, 255]);




fn get_image(path: String) -> Result<image::RgbImage, ImageError> {
    let img = image::open(path)?;
    let img = img.to_rgb8();
    Ok(img)
}

fn modify_image_seuil(mut img: image::RgbImage) -> Result<image::RgbImage, ImageError> {
    for pixel in img.pixels_mut(){
        let r = pixel[0];
        let g = pixel[1];
        let b = pixel[2];
        if r as u32+g as u32+b as u32 > 383 as u32 {
            *pixel = WHITE;
        } else {
            *pixel = BLACK;
        }
    }

    Ok(img)
}

fn modify_image_palette(mut img: image::RgbImage) -> Result<image::RgbImage, ImageError> {
    Ok(img)
}



fn main() -> Result<(), ImageError>{
    let args: DitherArgs = argh::from_env();
    let path_in = args.input;
    let path_out;
    if args.output.is_none() {
        path_out = "out.png".to_string();
    } else {
        path_out = args.output.unwrap();
    }
    let mode = args.mode;
    
    let img = get_image(path_in)?;

    match mode {
        Mode::Seuil(_) => {
            let image = modify_image_seuil(img)?;
            image.save(path_out)?;
        }
        Mode::Palette(opts) => {
            let image = modify_image_palette(img)?;
            image.save(path_out)?;
        }
    }

    Ok(())
}

