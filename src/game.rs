extern crate iced;

use iced::{executor};
use iced::{Length, Align};
use iced::{Button, Element, Row, Application, Svg, Text, Command, Subscription};
use iced_native::{
    window::Event,
};

use crate::{control_widget::*, field::*, field_widget::*, message::*, puzzle_creation};

use std::collections::{HashMap};
use std::path::{PathBuf, Path};



pub struct Game {
    field: Option<Field>,
    field_widget: FieldWidget,
    control_widget: ControlWidget,
}

impl Application for Game {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut svgs = HashMap::new();
        svgs.insert(CellType::Meadow, Svg::from_path("images/meadow.svg"));
        svgs.insert(CellType::Tent, Svg::from_path("images/tent.svg"));
        svgs.insert(CellType::Tree, Svg::from_path("images/tree.svg"));


        let field_widget = FieldWidget::new(20, 2, 2, svgs);
        let control_widget = ControlWidget::new(150, 10, 12);

        let game = Game {
            field: None,
            field_widget,
            control_widget,
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
            Message::SolvePuzzle => {
                let field : &mut Field = match &mut self.field {
                    None => panic!("no puzzle man"),
                    Some(f) => f
                };
                field.solve();
            },
                /**
                if let Some(field) = &self.field {
                    let tents = field.tent_coordinates();
                    solve_puzzle(&tents, &field.row_counts, &field.column_counts)
                } else {
                    self.control_widget.add_to_log(
                        "No field available! Drag and drop a tent file or create a custom or random one."
                    );
                };
            },**/
            Message::GridSizeInputChanged{width, height} => {
                self.control_widget.field_creation_widget.update(width, height)
            },
            Message::CreateRandomPuzzle{width , height} => {
                let field = puzzle_creation::create_random_puzzle(height, width).unwrap();
                self.field = Some(field);
            },
            _ => {
                unimplemented!();
            }
        };
        Command::none()
    }


    fn view(&mut self) -> Element<Self::Message> {
        Row::new()
        .align_items(Align::Start)
        .push(self.control_widget.draw())
        .push(self.field_widget.draw_field(&self.field))
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
