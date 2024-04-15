mod render_overlay;
mod stations;
mod coordinate_system;
mod resource_util;

use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::ops::{Add};
use iced::{Color, executor, Font, font, Point, Rectangle, Renderer, Vector};
use iced::widget::{container, row, image, text_input, Column, canvas, button};
use iced::{Application, Command, Element, Length, Settings, Theme};
use iced::font::{Family, Weight};
use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Geometry, Path, Program};
use iced::widget::image::viewer;
use json_comments::StripComments;
use simsearch::{SearchOptions, SimSearch};
use regex::Regex;
use crate::render_overlay::RenderOverlay;
use crate::stations::Station;
use crate::coordinate_system::CoordinateSystem;
use crate::resource_util::convert_relative_path;

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.antialiasing = true;
    TubeTagApp::run(settings)
}

#[derive(Default)]
struct TubeTagApp {
    // Backend
    all_stations: Vec<Station>,
    guessed_stations: HashSet<usize>,
    target_station: Option<Station>,
    search_engine: SimSearch<usize>,

    // Frontend
    station_input: String,
    render_cache: Cache,
}

#[derive(Debug, Clone)]
enum Message {
    FontLoaded(Result<(), font::Error>),
    GuessInputChanged(String),
    GuessSubmitted,
    ClearGuesses
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
        let ret = Self {
            all_stations: stations,
            guessed_stations: HashSet::new(),
            target_station: None,
            search_engine,
            station_input: String::new(),
            render_cache: Cache::new()
        };

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
            Message::ClearGuesses => {
                self.guessed_stations.clear()
            }
            _ => { }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        // Construct map viewer
        let map_path = convert_relative_path("assets/tube-map-4k.png");
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

        let clear_guesses = button("Clear")
            .on_press(Message::ClearGuesses);

        let overlaid = RenderOverlay::new(map_viewer, canvas(self));

        // === Layout ===
        let input_row = row![
            guess_input,
            clear_guesses
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

impl TubeTagApp {
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
            // TODO: Some sort of user feedback for "Station doesn't exist"
            return;
        }

        // Station is valid
        for station_idx in station_indices {
            self.guessed_stations.insert(station_idx);
        }
        self.station_input = String::new();
    }

    fn find_collisions(self) {
        // TODO: Remove this
        let mut collisions: HashSet<usize> = HashSet::new();
        for (idx, station) in self.all_stations.iter().enumerate() {
            if collisions.contains(&idx) {
                continue
            }
            let result = self.search_engine.search(&station.name);
            if result.len() == 0 {
                println!("--------------------");
                println!("Uhhhh.... {:?}", station.name)
            }
            if result.len() > 1 {
                println!("--------------------");
                println!("More than 1 for {:?}:", station.name);
                for result in result {
                    println!("{:?}", self.all_stations[result].name);
                    collisions.insert(result);
                }
            }
        }
        println!("--------------------");
        println!("{:?} collisions", collisions.len());
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

        // TODO: We should only clear this if the state has changed
        //   since the last time we drew on our canvas
        self.render_cache.clear();

        // Rendering
        let geometry = self.render_cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            let offset = Vector::new(
                center.x - exposed_state.current_offset.x,
                center.y - exposed_state.current_offset.y
            );

            let coords = CoordinateSystem::new(frame.width(), frame.height(), exposed_state.scale);

            for station_idx in &self.guessed_stations {
                let station = &self.all_stations[station_idx.clone()];
                for (index, offsets) in station.station_positions.iter().enumerate() {
                    let relative_x = offsets.0 * CoordinateSystem::REL_X;
                    let relative_y = offsets.1 * CoordinateSystem::REL_Y;
                    let point = Point::new(
                        coords.x_dist_percent(relative_x - 0.5),
                        coords.y_dist_percent(relative_y - 0.5)
                    ).add(offset);

                    let circle = Path::circle(point, coords.x_dist_pixels(32.0));

                    // TODO: Determine colour based on distance
                    // frame.fill(&circle, Color::from_rgb8(0, 255, 0));

                    // Render station name text
                    if index == 0
                    {
                        // Loop over each line in the name and render it
                        for name in station.get_render_lines(&point, &coords) {
                            frame.fill_text(name)
                        }
                    }
                }
            }
        });

        // Place geometry in a vector and return
        vec![geometry]
    }
}