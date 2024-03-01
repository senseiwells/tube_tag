use std::fmt::format;
use std::io::Read;
use iced::{executor, theme};
use iced::widget::{button, column, container, row, text, svg, image};
use iced::{Application, Command, Element, Length, Settings, Theme};
use iced::widget::image::viewer;

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
        let map_path = format!(
            "{}/assets/standard-tube-map-tube-only.png",
            env!("CARGO_MANIFEST_DIR")
        );

        let map_handle = image::Handle::from_path(map_path);

        let map_viewer = viewer(map_handle);

        // Construct svg object from handle
        // let tube_img = svg(tube_img_handle).width(Length::Fill).height(Length::Fill).style(
        //     theme::Svg::Default
        // );

        let column_layout = column![
            map_viewer
        ];



        container(
            column_layout
            .height(Length::Fill),
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}