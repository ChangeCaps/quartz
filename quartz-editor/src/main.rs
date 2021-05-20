mod editor_state;
mod project;

use quartz_render::framework::App;

fn main() {
    simple_logger::SimpleLogger::from_env().init().unwrap();

    log::debug!("Starting editor");

    App::new()
        .title("Quartz Editor")
        .run(editor_state::EditorState::new)
        .unwrap();
}
