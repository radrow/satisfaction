use iced::{Button, HorizontalAlignment, Length, VerticalAlignment};
use iced::{Column, Element, Row, Svg, Text};
use iced::svg::Handle;
use crate::{
    field::*,
    message::*,
};

use crate::game::{Config, FieldState};

lazy_static!{
    static ref TENT_SVG: Svg   = Svg::new(Handle::from_memory(include_bytes!("../../images/tent.svg").to_vec()));
    static ref TREE_SVG: Svg   = Svg::new(Handle::from_memory(include_bytes!("../../images/tree.svg").to_vec()));
    static ref MEADOW_SVG: Svg = Svg::new(Handle::from_memory(include_bytes!("../../images/meadow.svg").to_vec()));
}

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
    fn draw_cell(&self, cell: &CellType) -> Element<'static, Message> {
        match cell {
            CellType::Tent => TENT_SVG.clone(),
            CellType::Tree => TREE_SVG.clone(),
            CellType::Meadow => MEADOW_SVG.clone(),
        }.width(self.cell_size)
            .height(self.cell_size)
            .into()
    }
    fn draw_number(&self, number: usize) -> Element<'static, Message> {
        Text::new(number.to_string())
            .width(self.cell_size)
            .height(self.cell_size)
            .size(self.font_size)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center)
            .into()
    }

    pub fn view<'a,'b,'c>(&'a self, field: &'b Field, state: &'c mut FieldState) -> Element<'c, Message> {
        let mut cells = field.cells.iter()
            .map(|row| row.iter()
                .map(|cell| self.draw_cell(cell))
                .collect()
                ).collect::<Vec<Vec<_>>>();


        match state {
            FieldState::Playable(_, button_states) => {
                for ((row, col), state) in button_states.iter_mut() {
                    let cell = self.draw_cell(&field.get_cell(*row, *col));
                    cells[*row][*col] = Button::new(state, cell)
                        .padding(0)
                        .on_press(Message::FieldButtonPressed(*row, *col))
                        .into();
                }
            },
            _ => {},
        }

        cells.into_iter()
            .zip(field.row_counts.iter())
            .fold(Column::new().spacing(self.cell_spacing), |vertical, (row, row_count)| {
                vertical.push(row.into_iter()
                    .fold(Row::new().spacing(self.cell_spacing), |horizontal, cell| horizontal.push(cell)
                    ).push(self.draw_number(*row_count)))
            }).push(field.column_counts.iter()
                .fold(Row::new().spacing(self.cell_spacing), |horizontal, col_count|
                    horizontal.push(self.draw_number(*col_count)))
            ).spacing(self.cell_spacing).into()
    }
}
