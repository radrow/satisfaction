use iced::{Length, HorizontalAlignment, VerticalAlignment};
use iced::{Column, Element, Row, Svg, Text};
use iced::svg::Handle;
use crate::{
    field::*,
    message::*,
};

lazy_static!{
    static ref TENT_SVG_SOURCE: Svg   = Svg::new(Handle::from_memory(include_bytes!("../images/tent.svg").to_vec()));
    static ref TREE_SVG_SOURCE: Svg   = Svg::new(Handle::from_memory(include_bytes!("../images/tree.svg").to_vec()));
    static ref MEADOW_SVG_SOURCE: Svg = Svg::new(Handle::from_memory(include_bytes!("../images/meadow.svg").to_vec()));
}

pub struct FieldWidget {
    rect_size: Length,
    vertical_spacing: u16,
    horizontal_spacing: u16,

    pub arrows: Vec<((usize, usize), (usize, usize))>,
}

impl FieldWidget {
    pub fn new(rect_size: u16, vertical_spacing: u16, horizontal_spacing: u16) -> FieldWidget {
        FieldWidget {
            rect_size: Length::Units(rect_size),

            vertical_spacing,
            horizontal_spacing,

            arrows: Vec::new(),
        }
    }

    fn draw_cell(&self, cell: &CellType) -> Element<Message> {
        match cell {
            CellType::Tent => TENT_SVG_SOURCE.clone(),
            CellType::Tree => TREE_SVG_SOURCE.clone(),
            CellType::Meadow => MEADOW_SVG_SOURCE.clone(),
        }.width(self.rect_size)
            .height(self.rect_size)
            .into()
    }
    fn draw_number(&self, number: usize) -> Element<Message> {
        Text::new(number.to_string())
            .width(self.rect_size)
            .height(self.rect_size)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center)
            .into()
    }

    pub fn view(&self, field: &Field) -> Element<Message> {
        Column::with_children(
            field.cells.iter()
                .zip(field.row_counts.iter())
                .map(|(rows, row_count)| {
                    Row::with_children(
                        rows.iter()
                            .map(|cell| self.draw_cell(cell))
                            .collect()
                    ).spacing(self.vertical_spacing)
                        .push(self.draw_number(*row_count))
                        .into()
                }).collect()
        ).push(
            Row::with_children(
                field.column_counts.iter()
                    .map(|number| self.draw_number(*number))
                    .collect()
            ).spacing(self.horizontal_spacing)
        ).spacing(self.horizontal_spacing)
            .into()
    }
}
