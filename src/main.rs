mod render_overlay;

use iced::{Color, executor, Point, Rectangle, Renderer, Size};
use iced::widget::{column, container, row, image, text_input, text, Text, Column, canvas};
use iced::{Application, Command, Element, Length, Settings, Theme};
use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Geometry, Path, Program};
use iced::widget::image::viewer;
use crate::render_overlay::RenderOverlay;

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.antialiasing = true;
    TubeTagApp::run(settings)
}

#[derive(Default)]
struct TubeTagApp {
    station_input: String,
    guessed_cache: Cache
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

        let overlaid = RenderOverlay::new(map_viewer, canvas(self));

        // === Layout ===
        let input_row = row![
            guess_input
        ].padding(5);

        let column_layout = Column::new()
            .push(input_row)
            .push(overlaid);

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

impl Program<Message> for TubeTagApp {
    // We have the state of the image viewer
    // So we can translate our guessed positions correctly...
    type State = viewer::State;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor
    ) -> Vec<Geometry> {
        let geometry = self.guessed_cache.draw(renderer, bounds.size(), |frame| {
            println!("Center: {:?}", frame.center());
            let circle = Path::circle(frame.center(), 50.0);
            frame.fill(&circle, Color::from_rgb8(255, 0, 0));

            let rect = Path::rectangle(frame.center(), Size::new(100.0, 200.0));
            frame.fill(&rect, Color::from_rgb8(0, 0, 255));
        });
        vec![geometry]
    }
}