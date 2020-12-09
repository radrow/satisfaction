use iced::{button, executor};
use iced::{Length, HorizontalAlignment, VerticalAlignment, Align};
use iced::{Button, Column, Element, Row, Space, Svg, Text};
use std::collections::{HashMap};
use crate::{
    field::*,
    message::*,
};

pub struct FieldWidget {
    rect_size: Length,
    vertical_spacing: u16,
    horizontal_spacing: u16,
    svgs: HashMap<CellType, Svg>,
}

impl FieldWidget {
    pub fn new(rect_size: u16, vertical_spacing: u16, horizontal_spacing: u16, svgs: HashMap<CellType, Svg>) -> FieldWidget {
        FieldWidget {
            rect_size: Length::Units(rect_size),
            vertical_spacing,
            horizontal_spacing,
            svgs,
        }
    }

    fn draw_cell(&self, cell: &CellType) -> Element<Message> {
        match self.svgs.get(cell) {
            Some(svg) => svg.clone()
                            .width(self.rect_size)
                            .height(self.rect_size)
                            .into(),
            None => Space::new(self.rect_size, self.rect_size).into(),
        }
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