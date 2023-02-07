use gloo::console::log;
use serde::{Deserialize, Serialize};

use crate::types::{Hsl, Index, JSTimer, Rand, Rgba};

/// Rules
/// - neighbour generation: hsl range of h = [-20, 20], l = [-10, 10]
/// - possible neighbours
///
///     [0]120,1,55        [3]95,1,50*1
///     [1]100,1,50        [2]90,1,45
///     [6]*2              [4]80,1,35
///     [5]70,1,30         [5]70,1,30
///
/// *1(90..120,1,45..55)
/// *2(min(all)..max(all),1, min(all)..max(all))

const BASIC_RANGES: ((i32, i32), (f64, f64)) = ((-20, 20), (-0.1, 0.1));

pub enum PixelType {
    RGBA(Rgba),
    HSL(Hsl),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pixel {
    pub rgba: Rgba,
    pub hsl: Hsl,
}

impl Pixel {
    pub fn random() -> Self {
        let f = || Rand::gen_rangei32(0..255);
        let rgba = [f() as u8, f() as u8, f() as u8, 1];
        let hsl = Pixel::rgb2hsl(rgba[0], rgba[1], rgba[2]);
        Self { rgba, hsl }
    }

    pub fn new() -> Self {
        let rgba = [0, 0, 0, 1];
        let hsl = Pixel::rgb2hsl(rgba[0], rgba[1], rgba[2]);
        Self { rgba, hsl }
    }

    pub fn set_data(&mut self, set: PixelType) {
        use PixelType::*;
        match set {
            RGBA(v) => {
                self.rgba = v;
                self.hsl = Pixel::rgb2hsl(v[0], v[1], v[2]);
            }
            HSL(v) => {
                self.hsl = v;
                self.rgba = Pixel::hsl2rgb(v[0], v[1], v[2]);
            }
        }
    }

    pub fn rgb2hsl(r: u8, g: u8, b: u8) -> Hsl {
        let cmax = *[r, g, b].iter().max().unwrap();
        let cmin = *[r, g, b].iter().min().unwrap();

        let (r, g, b) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
        let (cmax, cmin) = (cmax as f64 / 255.0, cmin as f64 / 255.0);

        let delta = cmax - cmin;
        let mut hue = 0.0;

        if delta != 0.0 {
            hue = match cmax {
                x if x == r => ((g - b) / delta) % 6.0,
                x if x == g => (b - r) / delta + 2.0,
                x if x == b => (r - g) / delta + 4.0,
                _ => unreachable!(),
            };
        }

        hue = hue * 60.0;
        if hue < 0.0 {
            hue += 360.0
        }

        let l = (cmax + cmin) / 2.0;
        let s = if l > 0.5 {
            let lfh = 2.0 - cmax - cmin;
            if lfh == 0.0 {
                0.0
            } else {
                delta / (2.0 - cmax - cmin)
            }
        } else {
            let lfh = cmax + cmin;
            if lfh == 0.0 {
                0.0
            } else {
                delta / (cmax + cmin)
            }
        };

        [hue, s, l]
    }

    pub fn hsl2rgb(h: f64, s: f64, l: f64) -> Rgba {
        let a = s * l.min(1.0 - l);
        let f = |n: f64| {
            let k = (n + h / 30.0) % 12.0;

            let min = {
                let (a, b, c) = (k - 3.0, 9.0 - k, 1.0);
                let mut min = a;
                if b < min {
                    min = b
                }
                if c < min {
                    min = c
                }
                min
            };
            (l - a * min.max(-1.0)) * 255.0
        };
        [f(0.0) as u8, f(8.0) as u8, f(4.0) as u8, 1]
    }
}

#[derive(Serialize, Deserialize)]
pub struct Cell {
    pub px: Pixel,
    pub collapsed: bool,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            px: Pixel::new(),
            collapsed: false,
        }
    }
}

pub struct WFCField {
    pub data: Box<[Cell]>,
    pub dim: usize,
    pub epoch_idx: usize,
    pub collapsed_cnt: usize,
    visited: Vec<Index>,
    last: Index,
    neighbours: Box<[Box<[Index]>]>,
    timer: JSTimer,
}

impl WFCField {
    pub fn new(dim: usize) -> Self {
        let x = Rand::gen_rangei32(0..(dim + 1) as i32) as usize;
        let y = Rand::gen_rangei32(0..(dim + 1) as i32) as usize;

        let idx = x * dim + y;
        let mut visited = Vec::with_capacity(dim * 4);
        visited.push((x, y));

        let mut data = (0..dim * dim)
            .map(|_| Cell::new())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        data[idx].collapsed = true;
        data[idx].px = Pixel::random();
        let l = data.len();

        let neighbours = WFCField::gen_neighbours(l, dim);

        Self {
            data,
            dim,
            epoch_idx: 0,
            visited,
            neighbours,
            last: (x, y),
            collapsed_cnt: 0,
            timer: JSTimer::new(),
        }
    }

    pub fn new_with_data(mut data: Box<[Cell]>, dim: usize) -> Self {
        let x = Rand::gen_rangei32(0..(dim + 1) as i32) as usize;
        let y = Rand::gen_rangei32(0..(dim + 1) as i32) as usize;

        let idx = x * dim + y;
        let mut visited = Vec::with_capacity(dim * 4);
        visited.push((x, y));
        data[idx].collapsed = true;
        data[idx].px = Pixel::random();
        let neighbours = WFCField::gen_neighbours(data.len(), dim);

        Self {
            data,
            dim,
            epoch_idx: 0,
            collapsed_cnt: 0,
            visited,
            last: (x, y),
            neighbours,
            timer: JSTimer::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn init(&mut self) {
        let x = Rand::gen_rangei32(0..(self.dim + 1) as i32) as usize;
        let y = Rand::gen_rangei32(0..(self.dim + 1) as i32) as usize;

        let idx = x * self.dim + y;
        let d = &mut self.data[idx];
        d.collapsed = true;
        d.px = Pixel::random();

        self.visited.push((x, y));
    }

    pub fn gen_neighbours(len: usize, dim: usize) -> Box<[Box<[Index]>]> {
        let mut neighs = Vec::with_capacity(len);
        for x in 0..dim as i32 {
            for y in 0..dim as i32 {
                // let idx = x * dim as i32 + y;
                let iter = (-1..=1)
                    .map(|x1| {
                        (-1..=1).filter_map(move |y1| {
                            let xn = x as i32 + x1;
                            let yn = y as i32 + y1;
                            if (xn < 0 || yn < 0)
                                || (xn >= dim as i32 || yn >= dim as i32)
                                || (x == xn && y == yn)
                            {
                                return None;
                            }
                            Some((xn as usize, yn as usize))
                        })
                    })
                    .flatten()
                    .collect::<Box<[Index]>>();
                neighs.push(iter);
            }
        }

        neighs.into_boxed_slice()
    }

    fn gen_value(&self, (x, y): Index) -> Hsl {
        let mut cols = vec![];
        let cur_idx = x * self.dim + y;
        for (_x, _y) in self.neighbours[cur_idx].iter() {
            let idx = _x * self.dim + _y;
            let cell = &self.data[idx];
            if cell.collapsed {
                cols.push(cell.px.hsl);
            }
        }
        // let cols2 = cols.clone();
        let cnt = cols.len() as f64;
        let sum = cols
            .into_iter()
            .reduce(|mut acc, v| {
                acc[0] += v[0];
                acc[2] += v[2];
                acc
            })
            .unwrap();

        let new = [sum[0] / cnt, sum[1], sum[2] / cnt];

        let h_rang = BASIC_RANGES.0 .0..BASIC_RANGES.0 .1;
        let h = new[0] + Rand::gen_rangei32(h_rang);
        let l = new[2] + Rand::gen_rangef64(BASIC_RANGES.1 .0, BASIC_RANGES.1 .1);

        // log!(format!("cols.len: {:?}",cnt));
        // log!(format!("Cols: {:#?}",cols2));
        // log!(format!("Hsl: {:?}",new));
        // log!(format!("Rgba: {:?}",Pixel::hsl2rgb(h, 1.0, l)));
        // log!("--------------------------------------------------");
        // self.data[idx].px.rgba = Pixel::hsl2rgb(h, 1.0, l);
        // self.data[cur_idx].px.set_data(PixelType::HSL([h, 1.0, l]));
        // self.data[cur_idx].collapsed = true;
        [h, 1.0, l]
    }

    pub fn epoch(&mut self) {
        log!("Visited len: ", self.visited.len());
        for (_x, _y) in self.visited.clone() {
            for (x, y) in self.neighbours[_x * self.dim + _y].iter() {
                let idx = x * self.dim + y;
                if !self.visited.contains(&(*x, *y)) {
                    self.visited.push((*x, *y));
                }

                if !self.data[idx].collapsed {
                    let col = self.gen_value((*x, *y));
                    self.data[idx].collapsed = true;
                    self.data[idx].px.set_data(PixelType::HSL(col));
                    self.collapsed_cnt += 1;
                }
                // return;
            }
            let idx = self
                .visited
                .iter()
                .position(|idx| *idx == (_x, _y))
                .unwrap();
            self.visited.remove(idx);
        }

        self.visited = self
            .visited
            .iter()
            .enumerate()
            .filter_map(|(n, x)| {
                // if Rand::gen_rangef64(0.0, 1.0) < 0.5 {
                //     return None;
                // }
                // if n > 500 {return None};
                if self.is_blank(*x) {
                    return None;
                };
                Some(*x)
            })
            .collect::<Vec<_>>();
    }

    pub fn epoch2(&mut self) {
        let (x, y) = self.last;
        let idx = x * self.dim + y;
        let neighs = &self.neighbours[idx];
        let i = Rand::gen_rangei32(0..neighs.len() as i32) as usize;

        for (n, (x, y)) in neighs.iter().enumerate() {
            let idx = x * self.dim + y;

            if !self.data[idx].collapsed {
                let col = self.gen_value((*x, *y));
                self.data[idx].collapsed = true;
                self.data[idx].px.set_data(PixelType::HSL(col));
                self.collapsed_cnt += 1;
            }

            if n == i {
                self.last = (*x, *y);
            }
        }
    }

    pub fn is_blank(&self, (x, y): Index) -> bool {
        for (x, y) in self.neighbours[x * self.dim + y].iter() {
            if !self.data[x * self.dim + y].collapsed {
                return false;
            }
        }
        true
    }

    pub fn epoch3(&mut self) {
        // log!("Visited len: ", self.visited.len());

        // self.timer.start_time();
        for (_x, _y) in self.visited.clone() {
            for (x, y) in self.neighbours[_x * self.dim + _y].iter() {
                let idx = x * self.dim + y;
                if !self.visited.contains(&(*x, *y)) {
                    self.visited.push((*x, *y));
                }

                if !self.data[idx].collapsed {
                    let col = self.gen_value((*x, *y));
                    self.data[idx].collapsed = true;
                    self.data[idx].px.set_data(PixelType::HSL(col));
                    self.collapsed_cnt += 1;
                }
                // return;
            }
            let idx = self
                .visited
                .iter()
                .position(|idx| *idx == (_x, _y))
                .unwrap();
            self.visited.remove(idx);
        }
        // self.timer.epoch_from_start("Neighs took");

        // self.timer.start_time();
        let l = self.visited.len();
        if l > 100 {
            let mut cnt = 0;
            self.visited = self
                .visited
                .iter()
                .filter_map(|x| {
                    if Rand::gen_rangef64(0.0, 1.0) < 0.5 {
                        return None;
                    }
                    if self.is_blank(*x) {
                        return None;
                    };
                    Some(*x)
                })
                .filter(|_| {
                    if cnt < 100 {
                        {
                            cnt += 1;
                            true
                        }
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();

            if self.visited.len() < 2 {
                self.visited = self
                    .data
                    .iter()
                    .enumerate()
                    .filter_map(|(n, x)| {
                        // p.x = index / 3;
                        // p.y = index % 3;
                        if !x.collapsed {
                            let x = n / self.dim;
                            let y = n % self.dim;
                            return Some((x, y));
                        }
                        None
                    })
                    .collect();
            }
        }
        // self.timer.epoch_from_start("Visited reset took");
    }
}
