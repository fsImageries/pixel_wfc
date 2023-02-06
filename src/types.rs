use std::ops::Range;

use gloo::console::log;

pub type Settings = (usize,);
pub type Index = (usize, usize);
pub type Hsl = [f64; 3];
pub type Rgba = [u8; 4];

pub struct Rand {}
impl Rand {
    pub fn map_range(v: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
        let slope = (out_max - out_min) / (in_max - in_min);
        out_min + slope * (v - in_min)
    }

    pub fn gen_rangei32(range: Range<i32>) -> f64 {
        let iter = range.into_iter();
        let max = iter.clone().max().unwrap();
        let min = iter.min().unwrap();
        let v = js_sys::Math::random();
        Rand::map_range(v, 0.0, 1.0, min as f64, max as f64)
    }

    pub fn gen_rangef64(start:f64, end:f64) -> f64 {
        let max = end;
        let min = start;
        let v = js_sys::Math::random();
        Rand::map_range(v, 0.0, 1.0, min as f64, max as f64)
    }
}

pub struct JSTimer {
    start: f64,
    epochs: Vec<f64>
}

impl JSTimer {
    pub fn new() -> Self {
        Self { start: 0.0, epochs:vec![] }
    }

    pub fn start_time(&mut self) {
        self.start = js_sys::Date::now();
    }

    pub fn epoch_from_start(&mut self, msg:&str) {
        let epoch = js_sys::Date::now();
        let time = (epoch - self.start) / 1000.0;
        log!(format!("{} took: {:.2} s", msg, time));
        // self.epochs.push(epoch);
    }

    pub fn epoch_from_last(&mut self, msg:&str) {
        let epoch = js_sys::Date::now();
        let start = match self.epochs.last(){
            Some(v) => *v,
            None => self.start
        };
        let time = (epoch - start) / 1000.0;
        log!(format!("{} took: {:.2} s", msg, time));
        self.epochs.push(epoch);
    }
}