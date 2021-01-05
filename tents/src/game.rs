extern crate iced;

use iced::{Length, Align};
use iced::{Element, Row, Application, Text, Command, Subscription, HorizontalAlignment, VerticalAlignment, Container};
use iced_native::window::Event;

use crate::{
    message::Message, 
    field::*, 
    log::Log,
    control_widget::*, 
    field_widget::FieldWidget,
    puzzle_creation,
};


pub struct Game {
    field: Option<Field>,
    puzzle_solved: bool,
    log: Log,

    field_widget: FieldWidget,
    control_widget: ControlWidget,
}

impl Game {
    fn set_field(&mut self, field: Field) {
        self.field = Some(field);
        self.puzzle_solved = false;
    }

}

impl Application for Game {
    type Executor = iced_futures::executor::Tokio;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let field_widget = FieldWidget::new(15, 2, 2);
        let control_widget = ControlWidget::new(180);

        let game = Game {
            field: None,
            puzzle_solved: false,
            log: Log::new(),

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
                return Command::perform(
                    Field::from_file(path),
                    |result| result.map(Message::FieldLoaded)
                        .unwrap_or_else(|error| Message::ErrorOccurred(error.to_string()))
                )
            },
            Message::FieldLoaded(field) => self.set_field(field),
            Message::SolvePuzzle => {
                let field = self.field.as_mut().unwrap();
                self.field_widget.arrows = field.solve();
                self.puzzle_solved = true;
            },
            Message::GridSizeInputChanged{width, height} => {
                self.control_widget.field_creation_widget.update(width, height)
            },
            Message::CreateRandomPuzzle{width , height} => {
                let field = puzzle_creation::create_random_puzzle(height, width).unwrap();
                self.set_field(field);
            },
            Message::ErrorOccurred(error) => {
                self.log.add_error(error);
            }
        };
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Row::new()
        .align_items(Align::Start)
        .push(self.control_widget.view(!self.puzzle_solved && self.field.is_some(), &self.log))
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
