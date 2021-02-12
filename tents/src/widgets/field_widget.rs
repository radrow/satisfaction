use iced::{Length, HorizontalAlignment, VerticalAlignment};
use iced::{Column, Element, Row, Svg, Text};
use iced::svg::Handle;
use crate::{
    field::*,
    message::*,
};

use crate::game::Config;

// For each type a Tents tile can have an SVG is loaded.
// This is done during compilation so the necessary resources cannot lack.
lazy_static!{
    static ref TENT_SVG: Svg   = Svg::new(Handle::from_memory(include_bytes!("../../images/tent.svg").to_vec()));
    static ref TREE_SVG: Svg   = Svg::new(Handle::from_memory(include_bytes!("../../images/tree.svg").to_vec()));
    static ref MEADOW_SVG: Svg = Svg::new(Handle::from_memory(include_bytes!("../../images/meadow.svg").to_vec()));
}

/// A widget that gathers parameters
/// that determine the way a field should be drawn.
pub struct FieldWidget {
    cell_size: Length,
    cell_spacing: u16,
    font_size: u16,
}

impl FieldWidget {
    pub fn new(config: &Config) -> FieldWidget {
        FieldWidget {
            cell_size: config.cell_size,
            cell_spacing: config.cell_spacing,
            font_size: config.count_font_size,
        }
    }
}


impl FieldWidget {
    /// Draws a single tile/cell of a Tent field,
    /// i.e. a Tent, Tree or Meadow.
    fn draw_cell(&self, cell: &CellType) -> Element<Message> {
        match cell {
            CellType::Tent => TENT_SVG.clone(),
            CellType::Tree => TREE_SVG.clone(),
            CellType::Meadow => MEADOW_SVG.clone(),
        }.width(self.cell_size)
            .height(self.cell_size)
            .into()
    }

    /// Draws the number for the row/column counts.
    fn draw_number(&self, number: usize) -> Element<Message> {
        Text::new(number.to_string())
            .width(self.cell_size)
            .height(self.cell_size)
            .size(self.font_size)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center)
            .into()
    }

    /// A field is simply a grid of SVGs.
    pub fn view(&self, field: &Field) -> Element<Message> {
        Column::with_children(
            field.cells.iter()
                .zip(field.row_counts.iter())
                .map(|(rows, row_count)| {
                    Row::with_children(
                    // Stack the cells of each row to a horizontal bar
                        rows.iter()
                            .map(|cell| self.draw_cell(cell))
                            .collect()

                    // Append row count
                    ).spacing(self.cell_spacing)
                        .push(self.draw_number(*row_count))
                        .into()
                }).collect()
        ).push(
            // Append column counts
            Row::with_children(
                field.column_counts.iter()
                    .map(|number| self.draw_number(*number))
                    .collect()
            ).spacing(self.cell_spacing)
        ).spacing(self.cell_spacing)
            .into()
    }
}
