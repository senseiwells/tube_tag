use iced::{executor, theme};
use iced::widget::{button, column, container, row, text, svg, image, text_input};
use iced::{Application, Command, Element, Length, Settings, Theme};

pub fn main() -> iced::Result {
    TubeTagApp::run(Settings::default())
}

#[derive(Default)]
struct TubeTagApp {
    current_guess: String
}

#[derive(Debug, Clone)]
enum Message {
    GuessInputChanged(String),
    GuessSubmitted
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
            Message::GuessInputChanged(input) => {
                self.current_guess = input
            }
            Message::GuessSubmitted => {
                // TODO: Check that it's a valid tube station
                //  If it is and the user has not guessed it we
                //  reset the current guess, otherwise nothing
                self.current_guess = "".to_string()
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let map_path = format!(
            "{}/assets/tube-map-4k.png",
            env!("CARGO_MANIFEST_DIR")
        );

        let map_handle = image::Handle::from_path(map_path);
        let map_viewer = image::viewer(map_handle)
            .width(Length::Fill);

        let guess_input = text_input("Tube Station", &self.current_guess)
            .on_input(Message::GuessInputChanged)
            .on_submit(Message::GuessSubmitted);

        let input_row = row![
            guess_input
        ].padding(5);

        let column_layout = column![
            input_row,
            map_viewer
        ];

        container(
            column_layout,
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}