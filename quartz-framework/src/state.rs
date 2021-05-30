use crate::event::*;
use crate::input::*;
use crate::mouse::*;
use crate::window::*;
use quartz_render::prelude::*;
use std::time::Instant;

pub enum Trans {
    None,
}

pub struct UpdateCtx<'a> {
    pub instance: &'a Instance,
    pub delta_time: f32,
    pub window: &'a mut WindowDescriptor,
    pub keyboard: &'a Input<Key>,
    pub mouse: &'a MouseInput,
}

pub trait State {
    fn start(&mut self, _instance: &Instance) {}

    fn update(&mut self, _ctx: UpdateCtx<'_>) -> Trans {
        Trans::None
    }

    fn handle_event(&mut self, _instance: &Instance, _event: &winit::event::Event<()>) -> Trans {
        Trans::None
    }

    fn render(&mut self, _instance: &Instance, _target: TextureView) {}
}

pub struct StateMachine {
    last_update: Instant,
    states: Vec<Box<dyn State>>,
}

impl StateMachine {
    pub fn new<T: State + 'static>(instance: &Instance, mut state: T) -> Self {
        state.start(instance);

        Self {
            last_update: Instant::now(),
            states: vec![Box::new(state)],
        }
    }

    pub fn transition(&mut self, trans: Trans) {
        match trans {
            Trans::None => {}
        }
    }

    pub fn update(
        &mut self,
        instance: &Instance,
        window_descriptor: &mut WindowDescriptor,
        keyboard: &Input<Key>,
        mouse: &mut MouseInput,
    ) {
        let time = Instant::now();

        mouse.pre_update();

        let ctx = UpdateCtx {
            instance,
            delta_time: (time - self.last_update).as_secs_f32(),
            window: window_descriptor,
            keyboard,
            mouse,
        };

        self.last_update = time;

        let trans = self.states.last_mut().unwrap().update(ctx);

        mouse.post_update();

        self.transition(trans);
    }

    pub fn handle_event(&mut self, instance: &Instance, event: &winit::event::Event<()>) {
        let trans = self
            .states
            .last_mut()
            .unwrap()
            .handle_event(instance, event);

        self.transition(trans);
    }

    pub fn render(&mut self, instance: &Instance, target: TextureView) {
        self.states.last_mut().unwrap().render(instance, target);
    }
}
