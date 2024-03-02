mod render_overlay;
mod stations;

use std::fs;
use std::fs::File;
use std::ops::{Add, Mul, Sub};
use iced::{Color, executor, Font, font, Pixels, Point, Rectangle, Renderer, Vector};
use iced::widget::{column, container, row, image, text_input, text, Column, canvas};
use iced::{Application, Command, Element, Length, Settings, Theme};
use iced::alignment::{Horizontal, Vertical};
use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Geometry, Path, Program, Text};
use iced::widget::image::viewer;
use iced::widget::text::Shaping;
use crate::render_overlay::RenderOverlay;
use crate::stations::Station;

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.antialiasing = true;
    TubeTagApp::run(settings)
}

#[derive(Default)]
struct TubeTagApp {
    // Backend
    all_stations: Vec<Station>,
    guessed_stations: Vec<Station>,
    target_station: Option<Station>,

    // Frontend
    station_input: String,
    render_cache: Cache,
}

#[derive(Debug, Clone)]
enum Message {
    FontLoaded(Result<(), font::Error>),
    GuessInputChanged(String),
    GuessSubmitted
}

impl Application for TubeTagApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let station_locations_path = format!(
            "{}/assets/station_locations.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let file = File::open(station_locations_path)
            .expect("Missing station_locations.json");
        let stations: Vec<Station> = serde_json::from_reader(file)
            .expect("station_locations.json was invalid");

        (
            Self {
                all_stations: stations,
                guessed_stations: Vec::new(),
                target_station: None,
                station_input: String::new(),
                render_cache: Cache::new()
            },
            font::load(fs::read(format!(
                "{}/fonts/P22 Underground.ttf",
                env!("CARGO_MANIFEST_DIR")
            )).unwrap()).map(Message::FontLoaded)
        )
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
            _ => { }
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

const UNDERGROUND_FONT: Font = Font::with_name("P22 Underground");

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
        self.render_cache.clear();
        let geometry = self.render_cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            let offset = Vector::new(
                center.x - exposed.current_offset.x,
                center.y - exposed.current_offset.y
            );

            for station in &self.all_stations {
                for (index, offsets) in station.station_offsets.iter().enumerate() {
                    // 8k Image resolution: 8262×5803
                    let relative_x = offsets.0 / 8262.0;
                    let relative_y = offsets.1 / 5803.0;
                    let point = Point::new(
                        (relative_x - 0.5) * frame.width() * exposed.scale,
                         (relative_y - 0.5) * frame.height() * exposed.scale
                    ).add(offset);

                    let circle = Path::circle(point, 4.0 * exposed.scale);
                    // TODO: Determine colour based on distance
                    frame.fill(&circle, Color::from_rgb8(0, 255, 0));

                    if index == station.name_data.station_offset {
                        // TODO: Shift point by anchor
                        let name = Text {
                            content: station.name.clone(),
                            position: point.add(Vector::new(5.0 * exposed.scale, -5.0 * exposed.scale)),
                            color: Color::from_rgb8(0, 0, 0),
                            size: Pixels(8.0 * exposed.scale),
                            line_height: Default::default(),
                            font: UNDERGROUND_FONT,
                            horizontal_alignment: Horizontal::Left,
                            vertical_alignment: Vertical::Top,
                            shaping: Shaping::Basic,
                        };
                        frame.fill_text(name)
                    }
                }
            }
        });
        vec![geometry]
    }
}