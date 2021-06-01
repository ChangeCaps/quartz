use quartz_framework::prelude::*;

pub struct GameState {
    pipeline: RenderPipeline,
    mesh: Mesh,
    count: usize,
}

impl GameState {
    pub fn new(instance: &Instance, target_format: format::TargetFormat) -> Self {
        let shader = Shader::load("examples/shader.vert", "examples/shader.frag").unwrap();
        let pipeline = RenderPipeline::new(
            PipelineDescriptor {
                depth_stencil: None,
                ..PipelineDescriptor::default_settings(shader, target_format)
            },
            instance,
        )
        .unwrap();

        let mut mesh = Mesh::new();
        mesh.set_attribute("vertex_position", vec![Vec3::splat(1.0); 10000]);
        mesh.set_indices(vec![0; 10000]);

        pipeline
            .bind_uniform("transform", &Mat4::default());

        Self { pipeline, mesh, count: 0 }
    }
}

impl State for GameState {
    fn render(&mut self, instance: &Instance, main_view: TextureView) {
        self.count += 1;
        
        let mut render_ctx = instance.render();

        let desc = RenderPassDescriptor::default_settings(main_view);

        let mut pass = render_ctx.render_pass(&desc, &self.pipeline);

        let n = 10000;

        //self.mesh
        //    .set_attribute("vertex_position", vec![Vec3::splat(1.0); n]);
        //self.mesh.set_indices(vec![0; n]);

        //pass.set_pipeline_bindings();
        for _ in 0..100 {
            self.pipeline.generate_groups(instance);

            //pass.draw_mesh(&self.mesh);
        }
    }
}

fn main() {
    App::new().run(GameState::new).unwrap()
}
