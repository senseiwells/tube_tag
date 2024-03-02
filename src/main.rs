mod render_overlay;

use std::ops::{Add, Mul, Sub};
use iced::{Color, executor, Point, Rectangle, Renderer, Size, Vector};
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
            "{}/assets/tube-map-8k.png",
            env!("CARGO_MANIFEST_DIR")
        );

        let map_handle = image::Handle::from_path(map_path);
        let map_viewer = image::viewer(map_handle)
            .width(Length::Fill)
            // TODO: We have a scale limiter
            //  but we also need to limit the panning?
            .min_scale(1.0);

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

#[derive(Debug, Clone, Copy)]
pub struct PubState {
    pub scale: f32,
    pub starting_offset: Vector,
    pub current_offset: Vector,
    pub cursor_grabbed_at: Option<Point>,
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
        let copied = state.clone();
        // We need access to the offset and scale
        let exposed: PubState = unsafe {
            // This is totally safe and not jank
            std::mem::transmute(copied)
        };

        // TODO: We should only clear this if the state has changed
        //   since the last time we drew on our canvas
        self.guessed_cache.clear();
        let geometry = self.guessed_cache.draw(renderer, bounds.size(), |frame| {
            let circle = Path::circle(
                frame.center().sub(exposed.current_offset),
                50.0 * exposed.scale
            );
            frame.fill(&circle, Color::from_rgb8(255, 0, 0));

            // FIXME:
            //  The following won't work because the offset and scale isn't
            //  calculated properly and I don't have the brainpower to do the maths
            // frame.fill(
            //     &Path::circle(
            //         Point::new(500.0, 0.0).sub(exposed.current_offset),
            //         10.0 * exposed.scale
            //     ),
            //     Color::from_rgb8(0, 255, 0)
            // )
        });
        vec![geometry]
    }
}