use iced::executor;
use iced::widget::{button, column, container, row, text};
use iced::{Application, Command, Element, Length, Settings, Theme};

pub fn main() -> iced::Result {
    TubeTagApp::run(Settings::default())
}

#[derive(Default)]
struct TubeTagApp {
    clicks : u32
}

#[derive(Debug, Clone, Copy)]
enum Message {
    BtnClicked
}

impl Application for TubeTagApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("TubeTag")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::BtnClicked => {
                self.clicks += 1;
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let content = column!(
            button("Click Me!").on_press(Message::BtnClicked),
            text(self.clicks).size(50)
        );

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}