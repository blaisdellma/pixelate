use std::fs::File;
use std::path::Path;
use std::convert::TryFrom;
use png::{self,Decoder};

extern crate pixelate;
use pixelate::*;

fn main() {
    println!("PIXELATE");

    let filein = "lenna.png";
    let fileout1 = "lenna1.png";
    let fileout2 = "lenna2.png";

    let decoder = Decoder::new(File::open(filein).expect("File not found."));

    let image = Image::<RgbPixel>::try_from(decoder).expect("Error decoding image.");

    write_image(image.clone().pixelate(16,false).expect("Pixelate failed.")
        ,Path::new(fileout1)).expect("Writing png out failed.");

    write_image(image.clone().pixelate(16,true).expect("Pixelate failed.")
        ,Path::new(fileout2)).expect("Writing png out failed.");

}
