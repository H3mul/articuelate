use iced::widget::{center, text};
use iced::{Element, Task, Theme};

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title(|_: &App| String::from("Articuelate"))
        .theme(|_: &App| Theme::Dark)
        .window_size(iced::Size::new(1280.0, 720.0))
        .window(iced::window::Settings {
            min_size: Some(iced::Size::new(800.0, 600.0)),
            ..iced::window::Settings::default()
        })
        .run()
}

#[derive(Default)]
struct App;

#[derive(Debug, Clone)]
enum Message {}

impl App {
    fn boot() -> (App, Task<Message>) {
        (App::default(), Task::none())
    }

    fn update(_state: &mut App, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(_state: &App) -> Element<'_, Message> {
        center(text("Articuelate")).into()
    }
}