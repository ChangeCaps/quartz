mod editor_state;
mod project;
mod ui;

use quartz_render::framework::App;

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .with_module_level("quartz_editor", log::LevelFilter::Info)
        .init()
        .unwrap();

    log::info!("Starting editor");

    App::new()
        .title("Quartz Editor")
        .run(editor_state::EditorState::new)
        .unwrap();
}
