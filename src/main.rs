extern crate iced;

use iced::button;
use iced::Length;
use iced::{Button, Column, Element, Row, Sandbox, Settings, Space, Svg, Text};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum CellType {
    Unknown,
    Tent,
    Tree,
    Meadow,
}

fn main() -> iced::Result {
    Game::run(Settings::default())
}

struct Game {
    cells: Vec<Vec<CellType>>,
    svgs: HashMap<CellType, Svg>,
    width: u16,
    height: u16,
}

#[derive(Debug, Copy, Clone)]
enum Message {
    Pressed,
}

impl Sandbox for Game {
    type Message = Message;

    fn new() -> Self {
        let cells = vec![
            vec![CellType::Tent, CellType::Meadow],
            vec![CellType::Tree, CellType::Unknown],
        ];

        let mut svgs = HashMap::new();
        let _ = svgs.insert(CellType::Meadow, Svg::from_path("images/meadow.svg"));
        let _ = svgs.insert(CellType::Tent, Svg::from_path("images/tent.svg"));
        let _ = svgs.insert(CellType::Tree, Svg::from_path("images/tree.svg"));

        Game {
            cells,
            svgs,
            width: 30,
            height: 30,
        }
    }

    fn title(&self) -> String {
        String::from("Tents")
    }

    fn view(&mut self) -> Element<Self::Message> {
        let mut grid = Column::new();
        for row in self.cells.iter() {
            let mut columns = Row::new();
            for column in row.iter() {
                columns = columns.push({
                    let element: Element<Self::Message> = if let Some(svg) = self.svgs.get(column) {
                        svg.clone()
                            .width(Length::Units(self.width))
                            .height(Length::Units(self.height))
                            .into()
                    } else {
                        Space::new(Length::Units(self.width), Length::Units(self.height)).into()
                    };

                    element
                }).spacing(2)
            }
            grid = grid.push(columns).spacing(2)
        }
        grid.into()
    }

    fn update(&mut self, _: Self::Message) {}
}
