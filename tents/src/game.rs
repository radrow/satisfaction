extern crate iced;

use solver::CadicalSolver;
use iced::{executor};
use iced::{Length, Align};
use iced::{Element, Row, Application, Svg, Text, Command, Subscription, HorizontalAlignment, VerticalAlignment, Container};
use iced_native::{
    window::Event,
};

use crate::{control_widget::*, field::*, field_widget::*, message::*, puzzle_creation};

use std::collections::{HashMap};
use std::path::Path;

pub struct Game {
    field: Option<Field>,
    puzzle_solved: bool,
    field_widget: FieldWidget,
    control_widget: ControlWidget,
}

impl Application for Game {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let image_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("images/");

        let mut svgs = HashMap::new();
        svgs.insert(CellType::Meadow, Svg::from_path(image_directory.join("meadow.svg")));
        svgs.insert(CellType::Tent, Svg::from_path(image_directory.join("tent.svg")));
        svgs.insert(CellType::Tree, Svg::from_path(image_directory.join("tree.svg")));


        let field_widget = FieldWidget::new(15, 2, 2, svgs);
        let control_widget = ControlWidget::new(180);

        let game = Game {
            field: None,
            puzzle_solved: false,
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
                let field = Field::from_file(&path)
                    .unwrap();
                self.field = Some(field);
                self.puzzle_solved = false;
            },
            Message::SolvePuzzle => {
                let field = self.field.as_mut().unwrap();
                if field.solve(&CadicalSolver) {
                    self.puzzle_solved = true;
                }
            },
            Message::GridSizeInputChanged{width, height} => {
                self.control_widget.field_creation_widget.update(width, height)
            },
            Message::CreateRandomPuzzle{width , height} => {
                let field = puzzle_creation::create_random_puzzle(height, width).unwrap();
                self.field = Some(field);
                self.puzzle_solved = false;
            },
        };
        Command::none()
    }


    fn view(&mut self) -> Element<Self::Message> {
        Row::new()
        .align_items(Align::Start)
        .push(self.control_widget.view(!self.puzzle_solved && self.field.is_some()))
        .push(Container::new(
            match &mut self.field {
                None => Element::from(
                    Text::new("Drag and drop a file!")
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center)
                ),
                Some(field) => self.field_widget.view(field).into(),
            }).center_x()
                .center_y()
                .width(Length::Fill)
                .height(Length::Fill)
        ).padding(10)
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
