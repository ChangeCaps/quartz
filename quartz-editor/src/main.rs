mod editor_state;
mod project;
mod ui;

use quartz_framework::app::App;

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .with_module_level("quartz_editor", log::LevelFilter::Debug)
        .with_module_level("quartz_render", log::LevelFilter::Warn)
        .init()
        .unwrap();

    App::new()
        .title("Quartz Editor")
        .run(editor_state::EditorState::new)
        .unwrap();
}
