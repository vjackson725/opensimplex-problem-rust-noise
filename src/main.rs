
use image::{ColorType, ImageError};
use image::png::PngEncoder;

use mp4::{Mp4Config, Mp4Writer};

use noise::NoiseFn;

use glam::{DVec2, const_dvec2};

use std::f64::consts as f64;
use std::fs::File;
use std::iter;
use std::io::{Cursor, Write};
use std::path::Path;

mod colormap;

use colormap::{MAGMA, TWILIGHT};

const R_DIR : DVec2 = const_dvec2!([ 1.0,  0.0]);
const U_DIR : DVec2 = const_dvec2!([ 0.0,  1.0]);
const L_DIR : DVec2 = const_dvec2!([-1.0,  0.0]);
const D_DIR : DVec2 = const_dvec2!([ 0.0, -1.0]);

#[derive(Debug, Clone)]
pub struct Map {
  width : usize,
  height : usize,
  data : Vec<f64>,
  deriv : Vec<DVec2>,
}

impl Map {
  pub fn new(width : usize, height : usize, time : f64) -> Self {
    use noise::{OpenSimplex, Perlin, ScalePoint};

    // calculate the raw data
    let mut data = Vec::new();
    let data_noise = {
      ScalePoint::new(OpenSimplex::new()).set_scale(8.0)
    };

    // sample noise from a cylinder
    for j in 0..height {
      let y = j as f64 / height as f64;
      for i in 0..width {
        let x = i as f64 / width as f64;
        data.push(data_noise.get([x,y,time]) as f64);
      }
    }

    // calculate a local derivative
    let mut deriv : Vec<DVec2> = Vec::new();

    for j in 0..height {
      for i in 0..width {
        let mut vel = DVec2::new(0.0, 0.0);

        let c = data[i + j*width];
        if j > 0 {
          let u = data[i + (j-1)*width];
          vel += (u - c) * U_DIR;
        }
        if j+1 < height {
          let d = data[i + (j+1)*width];
          vel += (d - c) * D_DIR;
        }
        if i > 0 {
          let l = data[i-1 + j*width];
          vel += (l - c) * L_DIR;
        }
        if i+1 < width {
          let r = data[i+1 + j*width];
          vel += (r - c) * R_DIR;
        }

        deriv.push(vel);
      }
    }

    Map { width, height, data, deriv }
  }

  pub fn encode_data_png<W: Write>(&self, encoder : PngEncoder<W>) -> Result<(),ImageError> {
    let data = {
      self.data.iter()
        .flat_map(|&v| {
          let [r,g,b] = MAGMA.interpolate((v + 1.0)*0.5);
          iter::once(r)
            .chain(iter::once(g))
            .chain(iter::once(b))
        })
        .map(|v| ((v + 1.0) * 127.0) as u8)
        .collect::<Vec<u8>>()
    };
    encoder.encode(
      data.as_slice(),
      self.width as u32,
      self.height as u32,
      ColorType::Rgb8,
    )
  }

  pub fn encode_deriv_png<W: Write>(&self, encoder : PngEncoder<W>) -> Result<(),ImageError> {
    let data = {
      self.deriv.iter()
        .flat_map(|d| {
          let ang = d.angle_between(const_dvec2!([1.0,0.0])) / f64::PI;
          let [fr,fg,fb] = TWILIGHT.interpolate((ang + 1.0)/2.0);
          let r = (fr * 255.0) as u8;
          let g = (fg * 255.0) as u8;
          let b = (fb * 255.0) as u8;

          iter::once(r)
            .chain(iter::once(g))
            .chain(iter::once(b))
        })
        .collect::<Vec<u8>>()
    };
    encoder.encode(
      data.as_slice(),
      self.width as u32,
      self.height as u32,
      ColorType::Rgb8,
    )
  }
}


fn main() {
  let map = Map::new(800, 600, 0.0);

  let f_deriv = File::create(Path::new("deriv.png")).expect("failed to make file");
  let _encode_res = map.encode_deriv_png(PngEncoder::new(f_deriv));

  let f_data = File::create(Path::new("data.png")).expect("failed to make file");
  let _encode_res = map.encode_data_png(PngEncoder::new(f_data));
}