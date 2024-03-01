use iced::{executor};
use iced::widget::{column, container, image, text_input};
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

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String)
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
            Message::InputChanged(input) => {
                self.station_input = input;
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        // Construct map viewer
        let map_path = format!("{}/assets/standard-tube-map-tube-only.png",
                               env!("CARGO_MANIFEST_DIR"));
        let map_handle = image::Handle::from_path(map_path);
        let map_viewer = viewer(map_handle)
            .width(Length::Fill);

        // Construct input field
        let input_field = text_input("Guess a station", &self.station_input)
            .on_input(Message::InputChanged)
            .width(Length::Fill);

        // === Layout ===
        let column_layout = column![
            input_field,
            map_viewer
        ];

        container(
            column_layout
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}