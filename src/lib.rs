use std::io::{self,BufWriter};
use std::fs::File;
use std::path::Path;
use std::convert::{From,TryFrom,TryInto};
use png::{self,ColorType,BitDepth};

mod kmeans;
use crate::kmeans::{kmeans,Vec4};

#[derive(Debug,Default,Copy,Clone)]
pub struct RgbPixel {
    value: (u8,u8,u8),
}

#[derive(Debug,Default,Copy,Clone)]
pub struct HsvPixel {
    value: (f32,f32,f32),
}


pub trait PixelType {}
impl PixelType for RgbPixel {}
impl PixelType for HsvPixel {}

impl From<RgbPixel> for HsvPixel {
    fn from(p: RgbPixel) -> Self {
        let (r,g,b) = p.value;

        let r = (r as f32) / 255.0;
        let g = (g as f32) / 255.0;
        let b = (b as f32) / 255.0;

        let (h,cmax,del) = if r >= g && r >= b {
            let cmax = r;
            let cmin = if g >= b { b } else { g };
            let del = cmax - cmin;
            let h = if del == 0.0 {
                0.0
            } else if g >= b {
                0.0 + (g-b)/del
            } else {
                6.0 + (g-b)/del
            };
            (h,cmax,del)
        } else if g >= r && g >= b {
            let cmax = g;
            let cmin = if r >= b { b } else { r };
            let del = cmax - cmin;
            let h = if del == 0.0 {
                0.0
            } else {
                2.0 + (b-r)/del
            };
            (h,cmax,del)
        } else {
            let cmax = b;
            let cmin = if r >= g { g } else { r };
            let del = cmax - cmin;
            let h = if del == 0.0 {
                0.0
            } else {
                4.0 + (r-g)/del
            };
            (h,cmax,del)
        };

        let v = cmax;
        let s = if cmax > 0.0 {
            del/cmax
        } else {
            0.0
        };
        HsvPixel {value: (h,s,v)}
    }
}

impl TryFrom<HsvPixel> for RgbPixel {
    type Error = &'static str;
    fn try_from(p: HsvPixel) -> Result<Self,Self::Error> {
        let (h,s,v) = p.value;
        let hp = h;
        let c = s*v;
        let x = c*(1.0-(((hp/2.0)-(hp/2.0).floor())*2.0-1.0).abs());
        let (r,g,b) = match hp {
            y if y == 0.0 => (0.0,0.0,0.0),
            y if y > 0.0 && y <= 1.0 => (c,x,0.0),
            y if y > 1.0 && y <= 2.0 => (x,c,0.0),
            y if y > 2.0 && y <= 3.0 => (0.0,c,x),
            y if y > 3.0 && y <= 4.0 => (0.0,x,c),
            y if y > 4.0 && y <= 5.0 => (x,0.0,c),
            y if y > 5.0 && y <= 6.0 => (c,0.0,x),
            _ => {
                dbg!(hp);
                return Err("H prime out of range")
            }
        };
        let m = v-c;
        let r = ((r + m)*255.0) as u8;
        let g = ((g + m)*255.0) as u8;
        let b = ((b + m)*255.0) as u8;
        Ok(RgbPixel{value: (r,g,b)})
    }
}

impl From<Image<RgbPixel>> for Image<HsvPixel> {
    fn from(image: Image<RgbPixel>) -> Self {
        let image_data = image.image_data.iter().map(|&p| HsvPixel::from(p)).collect::<Vec<HsvPixel>>();
        Image {image_data, width: image.width, height: image.height}
    }
}

impl TryFrom<Image<HsvPixel>> for Image<RgbPixel> {
    type Error = &'static str;
    fn try_from(image: Image<HsvPixel>) -> Result<Self,Self::Error> {
        let image_data = image.image_data.iter().map(|&p| RgbPixel::try_from(p)).collect::<Result<Vec<RgbPixel>,Self::Error>>()?;
        Ok(Image {image_data, width: image.width, height: image.height})
    }
}

#[derive(Debug,Clone)]
pub struct Image<T: PixelType> {
    image_data: Vec<T>,
    width: usize,
    height: usize,
}

impl<T: PixelType> Default for Image<T> {
    fn default() -> Image<T> {
        Image {
            image_data: Vec::new(),
            width: 0,
            height: 0,
        }
    }
}

impl<R: io::Read> TryFrom<png::Decoder<R>> for Image<RgbPixel> {
    type Error = &'static str;

    fn try_from(decoder: png::Decoder<R>) -> Result<Self,Self::Error> {

        let (info, mut reader) = decoder.read_info().expect("Error in reading png info.");

        assert_eq!(info.color_type,ColorType::RGB);
        assert_eq!(info.bit_depth,BitDepth::Eight);
        assert_eq!(info.line_size, (info.width as usize)*3);

        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf).expect("Error in reading next frame.");

        let width: usize = info.width.try_into().unwrap();
        let height: usize = info.height.try_into().unwrap();
        let mut image_data: Vec<RgbPixel> = Vec::with_capacity(width*height);

        for i in 0..width*height {
            image_data.push(RgbPixel {value: (buf[i*3],buf[i*3 + 1], buf[i*3 + 2])});
        }

        Ok(Image {
            image_data,
            width,
            height,
        })
    }
}

impl Image<RgbPixel> {
    pub fn pixelate(self, factor: usize,doit: bool) -> Result<Self,&'static str> {

        let image = if doit {
            Image::<HsvPixel>::from(self).pixelate(factor)?.kmeans_reduce().expect("Reduce failed.")
        } else {
            Image::<HsvPixel>::from(self).pixelate(factor)?
        };

        Ok(Image::<RgbPixel>::try_from(image)?)
    }
}

impl Image<HsvPixel> {
    pub fn kmeans_reduce(&self) -> Result<Self,&'static str> {
        println!("kmeans");
        let width = self.width;
        let height = self.height;
        let image_data = kmeans(self.image_data.iter().map(|p| {
            let x = (p.value.0*std::f32::consts::FRAC_PI_3).cos();
            let y = (p.value.0*std::f32::consts::FRAC_PI_3).sin();
            Vec4::new(x,y,p.value.1,p.value.2)
        }).collect(),8).iter().map(|vec4| {
            let (x,y,s,v) = vec4.get();
            let mut h = y.atan2(x)/std::f32::consts::FRAC_PI_3;
            if h < 0.0 {
                h += 6.0;
            }
            HsvPixel {value: (h,s,v)}
        }).collect();
        Ok(Image::<HsvPixel> {
            image_data,
            width,
            height,
        })
    }
}

impl Image<HsvPixel> {
    fn pixelate(&self, factor: usize) -> Result<Self,&'static str> {
        if !(self.width % factor == 0 && self.height % factor == 0) {
            return Err("Image size is not divisible by factor.");
        }

        let width = self.width;
        let height = self.height;
        let mut image_data: Vec<HsvPixel> = Vec::with_capacity(width*height);
        image_data.resize(width*height,HsvPixel::default());
        let mut selection: Vec<HsvPixel> = Vec::with_capacity(factor*factor);
        for i in 0..self.width/factor {
            for j in 0..self.height/factor {
                selection.clear();
                for ii in 0..factor {
                    for jj in 0..factor {
                        selection.push(self.image_data[(i*factor + ii)*self.height + (j*factor + jj)]);
                    }
                }
                let p = avg_hsv_pixels(&selection);
                for ii in 0..factor {
                    for jj in 0..factor {
                        image_data[(i*factor + ii)*self.height + (j*factor + jj)] = p;
                    }
                }
            }
        }

        Ok(Image::<HsvPixel> {
            image_data,
            width,
            height,
        })

    }
}

fn avg_hsv_pixels(pixels: &[HsvPixel]) -> HsvPixel {

    let n: f32 = u16::try_from(pixels.len()).unwrap().into();
    let (mut x, mut y, mut s, mut v) = (0.0,0.0,0.0,0.0);

    for p in pixels {
        x += (p.value.0*std::f32::consts::FRAC_PI_3).cos();
        y += (p.value.0*std::f32::consts::FRAC_PI_3).sin();
        s += p.value.1;
        v += p.value.2;
    }

    x /= n;
    y /= n;
    s /= n;
    v /= n;

    let mut h = y.atan2(x)/std::f32::consts::FRAC_PI_3;
    if h < 0.0 {
        h += 6.0;
    }

    HsvPixel {
        value: (h,s,v),
    }

}

pub fn write_image(image: Image<RgbPixel>, path: &Path) -> Result<(),io::Error> {

    let mut buf: Vec<u8> = Vec::with_capacity(image.image_data.len()*3);
    for i in 0..image.image_data.len() {
        buf.push(image.image_data[i].value.0);
        buf.push(image.image_data[i].value.1);
        buf.push(image.image_data[i].value.2);
    }

    let mut encoder = png::Encoder::new(BufWriter::new(File::create(path)?),image.width.try_into().unwrap(),image.height.try_into().unwrap());
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(buf.as_slice())?;

    Ok(())
}
