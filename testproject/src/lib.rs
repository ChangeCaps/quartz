use quartz_engine::egui::*;
use quartz_engine::prelude::*;

#[derive(Reflect, Inspect)]
pub struct MeshComponent {
    mesh: Mesh,
}

impl InitComponent for MeshComponent {
    fn init(_render: &mut Render) -> Self {
        let mut mesh = Mesh::new();

        mesh.add_attribute::<Vec3>("vertex_position");

        Self { mesh }
    }
}

impl Component for MeshComponent {
    type Plugins = Render;

    fn name() -> &'static str {
        "Mesh"
    }

    fn inspector_ui(&mut self, _: &mut Render, _ctx: ComponentCtx, ui: &mut Ui) {
        self.inspect(ui);
    }

    fn update(&mut self, _: &mut Render, ctx: ComponentCtx) {
        ctx.transform.rotation *= Quat::from_rotation_y(0.01);
    }

    fn render(&mut self, render: &mut Render, ctx: ComponentRenderCtx) {
        if let Some(view_proj) = render.camera {
            render.pipeline.bind_uniform("ViewProj", view_proj);
            render
                .pipeline
                .bind_uniform("Model", ctx.global_transform.matrix());

            ctx.render_pass
                .with_pipeline(&render.pipeline)
                .draw_mesh(&self.mesh);
        }
    }
}

impl MeshComponent {
    pub fn new(mesh: Mesh) -> Self {
        Self { mesh }
    }
}

#[derive(Default, Reflect, Inspect)]
pub struct Camera {
    pub projection: PerspectiveProjection,
}

impl Camera {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for Camera {
    type Plugins = Render;

    fn name() -> &'static str {
        "Camera"
    }

    fn inspector_ui(&mut self, _: &mut Render, _ctx: ComponentCtx, ui: &mut Ui) {
        self.inspect(ui);
    }

    fn update(&mut self, render: &mut Render, ctx: ComponentCtx) {
        let size = ctx.render_resource.target_size();
        self.projection.aspect = size.x / size.y;

        let view_proj = self.projection.matrix() * ctx.global_transform.matrix().inverse();

        render.camera = Some(view_proj);
    }

    fn editor_update(&mut self, render: &mut Render, ctx: ComponentCtx) {
        Component::update(self, render, ctx);
    }

    fn despawn(&mut self, render: &mut Render, _ctx: ComponentCtx) {
        //render.camera = None;
    }
}

pub struct Render {
    pub pipeline: RenderPipeline,
    pub camera: Option<Mat4>,
}

impl Plugin for Render {
    fn init(ctx: PluginInitCtx) -> Self {
        let shader =
            Shader::from_glsl(include_str!("shader.vert"), include_str!("shader.frag")).unwrap();
        let pipeline = RenderPipeline::new(
            PipelineDescriptor::default_settings(shader),
            ctx.render_resource,
        )
        .unwrap();

        Self {
            pipeline: pipeline,
            camera: None,
        }
    }
}

quartz_engine::bridge! {
    components: { MeshComponent, Camera }
    plugins: { Render }
}
