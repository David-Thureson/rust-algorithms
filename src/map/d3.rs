// #![allow(dead_code)]

use serde::Serialize;
use crate::map::polygon_map::*;
// use serde_json;

use super::*;

pub const COLOR_CLEAR: D3Color = D3Color { r: 255, g: 255, b: 255, a: 0.0 };
pub const COLOR_WHITE: D3Color = D3Color { r: 255, g: 255, b: 255, a: 1.0 };
pub const COLOR_RED: D3Color = D3Color { r: 255, g: 0, b: 0, a: 1.0 };
pub const COLOR_GREEN: D3Color = D3Color { r: 0, g: 255, b: 0, a: 1.0 };
pub const COLOR_BLUE: D3Color = D3Color { r: 0, g: 0, b: 255, a: 1.0 };
pub const COLOR_BLACK: D3Color = D3Color { r: 0, g: 0, b: 0, a: 1.0 };
pub const COLOR_DARK_GRAY: D3Color = D3Color { r: 63, g: 63, b: 63, a: 1.0 };
pub const COLOR_GRAY: D3Color = D3Color { r: 127, g: 127, b: 127, a: 1.0 };
pub const COLOR_LIGHT_GRAY: D3Color = D3Color { r: 191, g: 191, b: 191, a: 1.0 };

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct D3Map {
    pub width: UD3,
    pub height: UD3,
    pub h_padding: UD3,
    pub v_padding: UD3,
    pub text: Vec<D3Text>,
    pub lines: Vec<D3Line>,
    pub circles: Vec<D3Circle>,
    pub polygons: Vec<D3Polygon>,
}

#[derive(Copy, Clone)]
pub enum D3FontFamily {
    SansSerif,
}

#[derive(Copy, Clone)]
pub enum D3TextAnchor {
    Start,
    Middle,
    End,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct D3Text {
    pub key: String,
    pub text: String,
    pub x: ID3,
    pub y: ID3,
    pub font_family: String,
    pub font_size: UD3,
    pub fill: String,
    pub text_anchor: String,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct D3Line {
    pub key: String,
    // pub role: String,
    pub x1: ID3,
    pub y1: ID3,
    pub x2: ID3,
    pub y2: ID3,
    pub stroke_width: UD3,
    pub stroke: String,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct D3Circle {
    pub key: String,
    // pub role: String,
    pub cx: ID3,
    pub cy: ID3,
    pub r: UD3,
    pub fill: String,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct D3Polygon {
    pub key: String,
    pub points: String,
    pub stroke_width: UD3,
    pub stroke: String,
    pub fill: String,
}

#[derive(Clone, Copy)]
pub struct D3Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl D3Map {
    pub fn new(width: UD3, height: UD3, h_padding: UD3, v_padding: UD3) -> Self {
        Self {
            width,
            height,
            h_padding,
            v_padding,
            text: vec![],
            lines: vec![],
            circles: vec![],
            polygons: vec![],
        }
    }
}

impl D3Text {
    pub fn new(key: &str, text: &String, x: ID3, y: ID3, font_family: D3FontFamily, font_size: UD3, fill: &D3Color, text_anchor: D3TextAnchor) -> Self {
        Self {
            key: key.to_string(),
            text: text.to_string(),
            x,
            y,
            font_family: font_family.to_html(),
            font_size,
            fill: fill.to_html(),
            text_anchor: text_anchor.to_html(),
        }
    }
}

impl D3Line {
    pub fn new(key: &str, x1: ID3, y1: ID3, x2: ID3, y2: ID3, stroke_width: UD3, stroke: &D3Color) -> Self {
        Self {
            key: key.to_string(),
            x1,
            y1,
            x2,
            y2,
            stroke_width,
            stroke: stroke.to_html(),
        }
    }
}

impl D3Circle {
    pub fn new(key: &str, cx: ID3, cy: ID3, r: UD3, fill: &D3Color) -> Self {
        Self {
            key: key.to_string(),
            cx,
            cy,
            r,
            fill: fill.to_html(),
        }
    }
}

impl D3Polygon {
    pub fn new(key: &str, points: &str, stroke_width: UD3, stroke: &D3Color, fill: &D3Color) -> Self {
        Self {
            key: key.to_string(),
            points: points.to_string(),
            stroke_width,
            stroke: stroke.to_html(),
            fill: fill.to_html(),
        }
    }
}

impl D3FontFamily {
    pub fn to_html(&self) -> String {
        match self {
            D3FontFamily::SansSerif => "sans-serif",
        }.to_string()
    }
}

impl D3TextAnchor {
    pub fn to_html(&self) -> String {
        match self {
            D3TextAnchor::Start => "start",
            D3TextAnchor::Middle => "middle",
            D3TextAnchor::End => "end",
        }.to_string()
    }
}

impl D3Color {
    pub fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        D3Color { r, g, b, a: 1.0 }
    }

    pub fn new_rgba(r: u8, g: u8, b: u8, a:f32) -> Self {
        D3Color { r, g, b, a }
    }

    pub fn mult_a(&self, a: f32) -> Self {
        let mut c = self.clone();
        c.a *= a;
        c
    }

    pub const fn a(&self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: a,
        }
    }

    pub fn interpolate(&self, other: &D3Color, pct: Pct) -> Self {
        Self::new_rgba(
            self.r + (*((f(other.r) - f(self.r)) * pct)) as u8,
            self.g + (*((f(other.g) - f(self.g)) * pct)) as u8,
            self.b + (*((f(other.b) - f(self.b)) * pct)) as u8,
            self.a + (*((f(other.a) - f(self.a)) * pct)) as f32
        )
    }

    pub fn to_html(&self) -> String {
        format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a)
    }
}



