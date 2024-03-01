use iced::{executor};
use iced::widget::{column, container, row, image, text_input};
use iced::{Application, Command, Element, Length, Settings, Theme};
use iced::widget::image::viewer;

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.antialiasing = true;
    TubeTagApp::run(settings)
}

#[derive(Default)]
struct TubeTagApp {
    station_input : String
}

#[derive(Debug, Clone, Copy)]
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
                self.station_input = input
            }
            Message::GuessSubmitted => {
                // TODO: Check that it's a valid tube station
                //  If it is and the user has not guessed it we
                //  reset the current guess, otherwise nothing
                self.station_input = "".to_string()
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        // Construct map viewer
        let map_path = format!(
            "{}/assets/tube-map-4k.png",
            env!("CARGO_MANIFEST_DIR")
        );
        let map_handle = image::Handle::from_path(map_path);
        let map_viewer = image::viewer(map_handle)
            .width(Length::Fill);

        // Construct input field
        let guess_input = text_input("Guess a station", &self.station_input)
            .on_input(Message::GuessInputChanged)
            .on_submit(Message::GuessSubmitted);

        // === Layout ===
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