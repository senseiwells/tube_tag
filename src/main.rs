mod render_overlay;
mod stations;

use std::fs;
use std::fs::File;
use std::ops::{Add, Mul, Sub};
use iced::{Color, executor, Font, font, Pixels, Point, Rectangle, Renderer, Vector};
use iced::widget::{column, container, row, image, text_input, text, Column, canvas, Image};
use iced::{Application, Command, Element, Length, Settings, Theme};
use iced::font::{Family, Weight};
use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Geometry, Path, Program, Text};
use iced::widget::image::viewer;
use json_comments::StripComments;
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
            "{}/assets/station_locations.json5",
            env!("CARGO_MANIFEST_DIR")
        );
        let file = File::open(station_locations_path)
            .expect("Missing station_locations.json5");
        let stations: Vec<Station> = serde_json::from_reader(StripComments::new(file))
            .expect("station_locations.json5 was invalid");

        (
            Self {
                all_stations: stations,
                guessed_stations: Vec::new(),
                target_station: None,
                station_input: String::new(),
                render_cache: Cache::new()
            },
            font::load(fs::read(format!(
                "{}/fonts/P22UndergroundPro-Bold.ttf",
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

const UNDERGROUND_FONT: Font = Font {
    family: Family::Name("P22 Underground Pro"),
    weight: Weight::Bold,
    ..Font::DEFAULT
};

impl Program<Message> for TubeTagApp {
    // We have the state of the image viewer
    // So we can translate our guessed positions correctly...
    type State = viewer::State;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor
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

            let context = DrawContext::new(frame.width(), frame.height(), exposed.scale);

            for station in &self.all_stations {
                for (index, offsets) in station.station_positions.iter().enumerate() {
                    let relative_x = offsets.0 * DrawContext::REL_X;
                    let relative_y = offsets.1 * DrawContext::REL_Y;
                    let point = Point::new(
                        context.x_dist_percent(relative_x - 0.5),
                        context.y_dist_percent(relative_y - 0.5)
                    ).add(offset);

                    let circle = Path::circle(point, context.x_dist_pixels(32.0));

                    // TODO: Determine colour based on distance
                    frame.fill(&circle, Color::from_rgb8(0, 255, 0));

                    if index == station.name_data.station_position {
                        for name in station.get_render_names(&point, &context) {
                            frame.fill_text(name)
                        }
                    }
                }
            }
        });
        vec![geometry]
    }
}

struct DrawContext {
    frame_width: f32,
    frame_height: f32,
    scale: f32
}

impl DrawContext {
    // 8k Image resolution: 8262×5803
    const REL_X: f32 = 1.0 / 8262.0;
    const REL_Y: f32 = 1.0 / 5803.0;

    fn new(frame_width: f32, frame_height: f32, scale: f32) -> Self {
        Self {
            frame_width,
            frame_height,
            scale
        }
    }

    fn x_dist_pixels(&self, dist: f32) -> f32 {
        self.x_dist_percent(dist * Self::REL_X)
    }

    fn y_dist_pixels(&self, dist: f32) -> f32 {
        self.y_dist_percent(dist * Self::REL_Y)
    }

    fn x_dist_percent(&self, percent: f32) -> f32 {
        percent * self.frame_width * self.scale
    }

    fn y_dist_percent(&self, percent: f32) -> f32 {
        percent * self.frame_height * self.scale
    }
}