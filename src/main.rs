mod render_overlay;
mod stations;
mod coordinate_system;
mod resource_util;

use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::ops::{Add, Deref};
use std::time::{Duration, Instant};
use iced::{Color, executor, Font, font, Pixels, Point, Rectangle, Renderer, Size, Vector};
use iced::widget::{container, row, image, text_input, Column, canvas, button};
use iced::{Application, Command, Element, Length, Settings, Theme};
use iced::alignment::{Horizontal, Vertical};
use iced::font::{Family, Weight};
use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Geometry, Path, Program, Text};
use iced::widget::image::viewer;
use json_comments::StripComments;
use simsearch::{SearchOptions, SimSearch};
use regex::Regex;
use rand::{random, Rng};
use rand::seq::SliceRandom;
use crate::render_overlay::RenderOverlay;
use crate::stations::Station;
use crate::coordinate_system::CoordinateSystem;
use crate::resource_util::convert_relative_path;

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.antialiasing = true;
    TubeTagApp::run(settings)
}

struct Title {
    message: Text,
    overlay: Option<Color>,
    duration: Duration,
    instant: Instant
}

impl Title {
    fn new(
        message: &str,
        color: Color,
        overlay: Option<Color>,
        duration: Duration
    ) -> Self {
        return Self {
            message: Text {
                content: message.to_string(),
                color,
                size: Pixels(60.0),
                font: UNDERGROUND_FONT,
                ..Default::default()
            },
            overlay,
            duration,
            instant: Instant::now(),
        }
    }
}

#[derive(Default)]
struct TubeTagApp {
    // Backend
    all_stations: Vec<Station>,
    guessed_stations: HashSet<usize>,
    target_station: Option<usize>,
    search_engine: SimSearch<usize>,

    // Frontend
    station_input: String,
    render_cache: Cache,
    title: Option<Title>,

    viewing_map: bool
}

#[derive(Debug, Clone)]
enum Message {
    FontLoaded(Result<(), font::Error>),
    GuessInputChanged(String),
    GuessSubmitted,
    Restart,
    GiveUp,
    ShowMap
}

impl Application for TubeTagApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        // Load station locations and deserialize
        let station_locations_path = convert_relative_path("assets/station_locations.json5");
        let station_locations_file = File::open(station_locations_path)
            .expect("Missing station_locations.json5");
        let stations: Vec<Station> = serde_json::from_reader(StripComments::new(station_locations_file))
            .expect("station_locations.json5 was invalid");

        // Initialize search engine
        let mut search_engine = SimSearch::new_with(
            SearchOptions::new().threshold(0.75).stop_whitespace(false).levenshtein(true)
        );
        let regex = Regex::new(r"\((.*?)\)").unwrap();
        for station_idx in 0..stations.len() {
            let name = &stations[station_idx].name;
            if regex.is_match(name) {
                // We insert the name without brackets too
                search_engine.insert_tokens(station_idx, &[name, &regex.replace_all(name, "")]);
            } else {
                search_engine.insert(station_idx, name);
            }
        }

        // Create a command to load the font
        let font_filepath = convert_relative_path("fonts/P22UndergroundPro-Bold.ttf");
        let load_font_command = font::load(fs::read(font_filepath).unwrap()).map(Message::FontLoaded);

        // Construct a TubeTagApp object
        let mut ret = Self {
            all_stations: stations,
            guessed_stations: HashSet::new(),
            target_station: None,
            search_engine,
            station_input: String::new(),
            render_cache: Cache::new(),
            title: None,
            viewing_map: false
        };
        ret.restart_game();

        // Return the constructed object and the command
        (ret, load_font_command)
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
                self.guess_submitted()
            }
            Message::Restart => {
                self.restart_game()
            }
            Message::GiveUp => {
                for idx in 0..self.all_stations.len() {
                    self.guessed_stations.insert(idx);
                }
            }
            Message::ShowMap => {
                self.viewing_map = !self.viewing_map
            }
            _ => { }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        // Construct map viewer
        let map_path = convert_relative_path("assets/tube-map-8k.png");
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

        let clear_guesses = button("Restart")
            .on_press(Message::Restart);
        let give_up = button("Give Up")
            .on_press(Message::GiveUp);
        let show_map = button(if self.viewing_map { "Hide Map" } else { "Show Map" })
            .on_press(Message::ShowMap);

        let overlaid = RenderOverlay::new(map_viewer, canvas(self));

        // === Layout ===
        let input_row = row![
            guess_input,
            clear_guesses,
            give_up,
            show_map
        ].padding(5).spacing(5);

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

impl TubeTagApp {
    fn restart_game(&mut self) {
        self.guessed_stations.clear();

        let mut rng = rand::thread_rng();
        let random_idx = rng.gen_range(0..self.all_stations.len());
        self.target_station = Some(random_idx);
        self.title = None;
    }

    fn search_approx(&self, query : &str) -> Vec<usize>{
        // Search with search engine
        let results: Vec<usize> = self.search_engine.search(query);
        if results.is_empty() {
            return results
        }

        let first_idx = results[0];
        let first_station = &self.all_stations[first_idx];

        if !first_station.name.contains("Edgware Road") {
            return vec![first_idx]
        }

        // Edgware Roads is a special case since we have 2 Edgware Roads
        let mut duplicates: Vec<usize> = vec![];
        for (idx, station) in self.all_stations.iter().enumerate() {
            if station.name.contains("Edgware Road") {
                duplicates.push(idx);

                if duplicates.len() == 2 {
                    break
                }
            }
        }
        return duplicates
    }

    fn guess_submitted(&mut self) {
        let station_indices = self.search_approx(&self.station_input);

        // Input was not a valid station
        if station_indices.is_empty() {
            self.title = Some(Title::new(
                "Unknown Station",
                Color::from_rgb8(200, 0,0),
                None,
                Duration::from_secs(2)
            ));
            return;
        }

        // Station is valid
        for station_idx in station_indices {
            self.guessed_stations.insert(station_idx);

            if let Some(target_idx) = self.target_station {
                if station_idx == target_idx {
                    self.game_won()
                }
            }
        }
        self.station_input = String::new();
    }

    fn game_won(&mut self) {
        self.title = Some(Title::new(
            "You Won!",
            Color::from_rgb8(0, 255,0),
            Some(Color::from_rgba8(255, 255, 255, 0.5)),
            Duration::from_secs(15)
        ));

        for idx in 0..self.all_stations.len() {
            self.guessed_stations.insert(idx);
        }
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

fn average_position(positions: &Vec<(f32, f32)>) -> (f32, f32) {
    let mut x = 0.0;
    let mut y = 0.0;
    for position in positions {
        x += position.0;
        y += position.1;
    }
    (x / positions.len() as f32, y / positions.len() as f32)
}

fn lerp_colour(start: &Color, end: &Color, delta: f32) -> Color {
    Color::from_rgb(
        start.r * (1.0 - delta) + end.r * delta,
        start.g * (1.0 - delta) + end.g * delta,
        start.b * (1.0 - delta) + end.b * delta
    )
}

impl Program<Message> for TubeTagApp {
    type State = viewer::State;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor
    ) -> Vec<Geometry> {
        // Clone the state and cast to PubState to extract offsets
        let exposed_state: PubState = unsafe { std::mem::transmute(state.clone()) };

        self.render_cache.clear();

        // Rendering
        let geometry = self.render_cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            let offset = Vector::new(
                center.x - exposed_state.current_offset.x,
                center.y - exposed_state.current_offset.y
            );

            let coords = CoordinateSystem::new(frame.width(), frame.height(), exposed_state.scale);

            let stations = if !self.viewing_map {
                let mut guessed = vec![];
                for station in &self.guessed_stations {
                    guessed.push(&self.all_stations[station.clone()])
                }
                guessed
            } else {
                let mut all = vec![];
                for station in &self.all_stations {
                    all.push(station)
                }
                all
            };
            for station in stations {
                for (index, offsets) in station.station_positions.iter().enumerate() {
                    let relative_x = offsets.0 * CoordinateSystem::REL_X;
                    let relative_y = offsets.1 * CoordinateSystem::REL_Y;
                    let point = Point::new(
                        coords.x_dist_percent(relative_x - 0.5),
                        coords.y_dist_percent(relative_y - 0.5)
                    ).add(offset);

                    // Render station name text
                    if index == 0 {
                        // Loop over each line in the name and render it
                        for mut name in station.get_render_lines(&point, &coords) {
                            if let Some(target_idx) = self.target_station {
                                let target_station = &self.all_stations[target_idx];
                                if !self.viewing_map && station.name == target_station.name {
                                    name.color = Color::from_rgb8(0, 255, 0);
                                    let mut shadow = name.clone();
                                    shadow.color = Color::BLACK;
                                    shadow.position = shadow.position.add(Vector::new(
                                        coords.x_dist_pixels(2.0),
                                        coords.y_dist_pixels(2.0)
                                    ));
                                    frame.fill_text(shadow)
                                }
                            }
                            frame.fill_text(name)
                        }
                    }

                    if self.viewing_map {
                        continue
                    }

                    if let Some(target_idx) = self.target_station {
                        let target_station = &self.all_stations[target_idx];
                        let average_target_position = average_position(&target_station.station_positions);
                        let dx = offsets.0 - average_target_position.0;
                        let dy = offsets.1 - average_target_position.1;

                        let distance = (dx * dx + dy * dy).sqrt() * CoordinateSystem::REL_Y;

                        let red = Color::from_rgb8(255, 0, 0);
                        let yellow = Color::from_rgb8(255, 255, 0);
                        let green = Color::from_rgb8(0, 255, 0);

                        let colour = if distance > 0.7 {
                            red
                        } else if distance > 0.2 {
                            lerp_colour(&yellow, &red, (distance * 2.0) - 0.4)
                        } else {
                            lerp_colour(&green, &yellow, distance * 5.0)
                        };


                        let circle = Path::circle(point, coords.x_dist_pixels(32.0));
                        frame.fill(&circle, Color::BLACK);
                        let circle = Path::circle(point, coords.x_dist_pixels(25.0));
                        frame.fill(&circle, colour);
                    }
                }
            }

            // Title rendering
            if let Some(title) = &self.title {
                let since = Instant::now().duration_since(title.instant);
                if since > title.duration {
                    return
                }

                if let Some(overlay) = title.overlay {
                    frame.fill_rectangle(
                        Point::new(0.0, 0.0),
                        Size::new(frame.width(), frame.height()),
                        overlay
                    )
                }

                let mut message = title.message.clone();
                message.position = center;
                message.horizontal_alignment = Horizontal::Center;
                message.vertical_alignment = Vertical::Center;
                let mut shadow = message.clone();
                shadow.color = Color::BLACK;
                shadow.position = shadow.position.add(Vector::new(4.0, 4.0));

                frame.fill_text(shadow);
                frame.fill_text(message)
            }
        });

        // Place geometry in a vector and return
        vec![geometry]
    }
}