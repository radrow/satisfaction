extern crate iced;

use iced::{button, executor};
use iced::{Length, HorizontalAlignment, VerticalAlignment};
use iced::{Button, Column, Element, Row, Application, Settings, Space, Svg, Text, Command, Subscription, Align};
use iced_native::{
    window::Event,
};

use crate::field::*;
use std::collections::{HashMap};
use std::path::PathBuf;

pub struct Game {
    field: Field,

    width: Length,
    height: Length,

    svgs: HashMap<CellType, Svg>,
    vertical_spacing: u16,
    horizontal_spacing: u16,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileDropped(PathBuf),
    SolvePuzzle,
}

impl Game {
    fn draw_cell(&self, cell: &CellType) -> Element<Message> {
        match self.svgs.get(cell) {
            Some(svg) => svg.clone()
                            .width(self.width)
                            .height(self.height)
                            .into(),
            None => Space::new(self.width, self.height).into(),
        }
    }

    fn draw_number(&self, number: usize) -> Element<Message> {
        Text::new(number.to_string())
            .width(self.width)
            .height(self.height)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center)
            .into()
    }

    fn draw_field(&self) -> Element<Message> {
        Column::with_children(
            self.field.cells.iter()
                .zip(self.field.row_counts.iter())
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
                self.field.column_counts.iter()
                    .map(|number| self.draw_number(*number))
                    .collect()
            )
        ).spacing(self.horizontal_spacing)
            .into()
    }
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
        svgs.insert(CellType::Meadow, Svg::from_path("images/meadow.svg"));
        svgs.insert(CellType::Tent, Svg::from_path("images/tent.svg"));
        svgs.insert(CellType::Tree, Svg::from_path("images/tree.svg"));


        let game = Game {
            field,

            width: Length::Units(30),
            height: Length::Units(30),
            vertical_spacing: 2,
            horizontal_spacing: 2,

            svgs,
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
        self.draw_field()
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