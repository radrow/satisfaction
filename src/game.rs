extern crate iced;

use iced::{button, executor};
use iced::Length;
use iced::{Button, Column, Element, Row, Application, Settings, Space, Svg, Text, Command, Subscription};
use iced_native::{
    window::Event,
};

use crate::field::*;
use std::collections::{HashMap};
use std::path::PathBuf;

pub struct Game {
    field: Field,

    svgs: HashMap<CellType, Svg>,
    width: u16,
    height: u16,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileDropped(PathBuf),
    SolvePuzzle,
}

impl Application for Game {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let cells = vec![
            vec![CellType::Tent, CellType::Meadow],
            vec![CellType::Tree, CellType::Unknown],
        ];

        let field = Field {
            cells: cells,
            row_counts: vec![2, 1],
            column_counts: vec![0,1],
        };

        let mut svgs = HashMap::new();
        let _ = svgs.insert(CellType::Meadow, Svg::from_path("images/meadow.svg"));
        let _ = svgs.insert(CellType::Tent, Svg::from_path("images/tent.svg"));
        let _ = svgs.insert(CellType::Tree, Svg::from_path("images/tree.svg"));

        let game = Game {
            field,
            svgs,
            width: 30,
            height: 30,
        };
        (game, Command::none())
    }

    fn title(&self) -> String {
        String::from("Solving Tents")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::FileDropped(path) => println!("{:?}", path),
            Message::SolvePuzzle => println!("Solving puzzle ..."),
        };
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let mut grid = Column::new();
        for (i, row) in self.field.cells.iter().enumerate() {
            let mut columns = Row::new();
            for (j, column) in row.iter().enumerate() {
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
            grid = grid.push(columns.push(Text::new(self.field.column_counts[i].to_string())))
                .spacing(2)
        }
        grid.push({
            let elements = self.field.row_counts.iter().map(|number|
                Text::new(number.to_string())
                    .width(Length::Units(self.width))
                    .height(Length::Units(self.height))
                    .into()
            ).collect();
            Row::with_children(elements)
        }).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events_with(
            |event, _| {
                match event {
                    iced_native::Event::Window(Event::FileDropped(path)) => Some(Message::FileDropped(path)),
                    _ => None
                }
            })
    }
}