use std::f32::consts::SQRT_2;
use std::ops::{Add, Mul};
use iced::{Color, Pixels, Point, Vector};
use iced::advanced::text::{LineHeight, Shaping};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas::Text;
use serde::{Deserialize};
use crate::{CoordinateSystem, UNDERGROUND_FONT};

#[derive(Debug, Deserialize)]
pub struct Station {
    pub name: String,
    pub station_positions: Vec<(f32, f32)>,
    #[serde(default)]
    pub name_data: NameData
}

impl Station {
    pub fn get_render_lines(&self, point: &Point, context: &CoordinateSystem) -> Vec<Text> {
        let size = context.y_dist_pixels(36.0);
        let unit_offset = self.name_data.offset;
        let unit_vec = self.name_data.anchor.unit_vec();
        let mut offset = unit_vec.add(Vector::new(unit_offset.0, unit_offset.1)).mul(size);
        let (horizontal, vertical) = self.name_data.anchor.alignments();

        if let Some(names) = self.name_data.name_lines.as_ref() {
            // Multiline name
            let lines = names.len() as f32;
            let mut shift = size * 0.5 * lines;

            // If our name is above (north) then we shift by the number of lines
            // If our name is beside (east / west) then we shift by half number of lines
            // If our name is below (south) then we do not shift
            offset = offset.add(Vector::new(0.0, (unit_vec.y.round() - 1.0) * 0.5 * shift));

            names.iter().rev().map(|name| {
                let new_offset = offset.add(Vector::new(0.0, shift));
                shift -= size;
                Self::name_to_text(name, point, new_offset, size, horizontal, vertical)
            }).collect()
        } else {
            // Single line name
            vec![Self::name_to_text(&self.name, point, offset, size, horizontal, vertical)]
        }
    }

    fn name_to_text(
        name: &String,
        point: &Point,
        offset: Vector,
        size: f32,
        horizontal: Horizontal,
        vertical: Vertical
    ) -> Text {
        Text {
            content: name.clone(),
            position: point.add(offset),
            color: Color::from_rgb8(0x1B, 0x40, 0x94),
            size: Pixels(size),
            line_height: LineHeight::Relative(1.0),
            font: UNDERGROUND_FONT,
            horizontal_alignment: horizontal,
            vertical_alignment: vertical,
            shaping: Shaping::Basic,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct NameData {
    #[serde(default)]
    pub anchor: Anchor,
    #[serde(default)]
    pub name_lines: Option<Vec<String>>,
    #[serde(default)]
    pub offset: (f32, f32)
}

#[derive(Debug, Default, Deserialize)]
pub enum Anchor {
    #[default]
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest
}

impl Anchor {
    fn unit_vec(&self) -> Vector {
        const INV_SQRT_2: f32 = 1.0 / SQRT_2;

        match self {
            Anchor::North => Vector::new(0.0, -1.0),
            Anchor::NorthEast => Vector::new(INV_SQRT_2, -INV_SQRT_2),
            Anchor::East => Vector::new(1.0, 0.0),
            Anchor::SouthEast => Vector::new(INV_SQRT_2, INV_SQRT_2),
            Anchor::South => Vector::new(0.0, 1.0),
            Anchor::SouthWest => Vector::new(-INV_SQRT_2, INV_SQRT_2),
            Anchor::West => Vector::new(-1.0, 0.0),
            Anchor::NorthWest => Vector::new(-INV_SQRT_2, -INV_SQRT_2)
        }
    }

    fn alignments(&self) -> (Horizontal, Vertical) {
        match self {
            Anchor::North => (Horizontal::Center, Vertical::Bottom),
            Anchor::NorthEast => (Horizontal::Left, Vertical::Bottom),
            Anchor::East => (Horizontal::Left, Vertical::Center),
            Anchor::SouthEast => (Horizontal::Left, Vertical::Top),
            Anchor::South => (Horizontal::Center, Vertical::Top),
            Anchor::SouthWest => (Horizontal::Right, Vertical::Top),
            Anchor::West => (Horizontal::Right, Vertical::Center),
            Anchor::NorthWest => (Horizontal::Right, Vertical::Bottom)
        }
    }
}