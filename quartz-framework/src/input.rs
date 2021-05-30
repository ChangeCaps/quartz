use std::collections::HashSet;
pub use winit::event::MouseButton;
pub use winit::event::VirtualKeyCode as Key;

pub trait InputEvent: std::hash::Hash + Eq + Clone {}

impl<T> InputEvent for T where T: std::hash::Hash + Eq + Clone {}

pub struct Input<T: InputEvent> {
    pressed: HashSet<T>,
    held: HashSet<T>,
    released: HashSet<T>,
}

impl<T: InputEvent> Default for Input<T> {
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            held: Default::default(),
            released: Default::default(),
        }
    }
}

impl<T: InputEvent> Input<T> {
    pub fn new() -> Self {
        Self {
            pressed: Default::default(),
            held: Default::default(),
            released: Default::default(),
        }
    }

    pub fn press(&mut self, event: T) {
        self.pressed.insert(event.clone());
        self.held.insert(event);
    }

    pub fn release(&mut self, event: T) {
        self.held.remove(&event);
        self.released.insert(event);
    }

    pub fn update(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }

    pub fn pressed(&self, event: &T) -> bool {
        self.pressed.contains(event)
    }

    pub fn held(&self, event: &T) -> bool {
        self.held.contains(event)
    }

    pub fn released(&self, event: &T) -> bool {
        self.released.contains(event)
    }
}
