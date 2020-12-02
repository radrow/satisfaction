extern crate iced;

use iced::{button, executor};
use iced::{Length, HorizontalAlignment, VerticalAlignment, Align};
use iced::{Button, Column, Element, Row, Application, Settings, Space, Svg, Text, Command, Subscription};
use iced_native::{
    window::Event,
};

use crate::field::*;
use std::collections::{HashMap};
use std::path::PathBuf;


enum Exception {
    FileNotFound(PathBuf),
    IllFormatedFile(PathBuf),
}

pub struct Game {
    field: Option<Field>,
    settings: LayoutSettings,
    exception: Option<Exception>,
    solve_button: button::State,
}

struct LayoutSettings {
    rect_size: Length,
    vertical_spacing: u16,
    horizontal_spacing: u16,
    svgs: HashMap<CellType, Svg>,
}

impl LayoutSettings {
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

    fn draw_field(&self, field: &Option<Field>) -> Element<Message> {
        if let Some(field) = field {
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
                )
            ).spacing(self.horizontal_spacing)
                .into()
        } else {
            Space::new(self.rect_size, self.rect_size).into()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    FileDropped(PathBuf),
    SolvePuzzle,
}

impl Game {
    
    
    //fn draw_control(self, )
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

        let settings = LayoutSettings {
            rect_size: Length::Units(20),
            vertical_spacing: 2,
            horizontal_spacing: 2,
            svgs,
        };

        let game = Game {
            field: Some(field),
            settings,
            exception: None,
            solve_button: button::State::new()
        };
        (game, Command::none())
    }

    fn title(&self) -> String {
        String::from("Solving Tents")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::FileDropped(path) => {
                // TODO: Handle exception appropriately
                self.field = Field::from_file(&path).ok();
            },
            Message::SolvePuzzle => println!("Solving puzzle ..."),
        };
        Command::none()
    }


    fn view(&mut self) -> Element<Self::Message> {
        Row::new()
        .align_items(Align::Center)
        .push(self.settings.draw_field(&self.field))
        .push(Text::new("Drag and drop tent puzzle, please!"))
        .push(Button::new(&mut self.solve_button, Text::new("Solve Puzzle!"))
            .on_press(Message::SolvePuzzle)
        )

        .into()
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