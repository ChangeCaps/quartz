use quartz_engine_core::egui::Ui;
use quartz_engine_core::prelude::*;

mod quartz_engine {
    pub use quartz_engine_core as core;
}

pub const MAX_LIGHTS: u32 = 64;

pub fn register_types(types: &mut Types) {
    types.register_plugin::<Render3dPlugin>();
    types.register_component::<Light3d>();
    types.register_component::<Camera3d>();
    types.register_component::<Mesh3d>();
}

pub struct Render3dPlugin {
    pub pbr_pipeline: RenderPipeline,
    pub main_camera: Option<Mat4>,
    pub lights: UniformBuffer<Vec3, MAX_LIGHTS>,
}

impl Plugin for Render3dPlugin {
    fn init(ctx: PluginInitCtx) -> Self {
        let pbr_shader =
            Shader::from_glsl(include_str!("pbr.vert"), include_str!("pbr.frag")).unwrap();
        let pbr_pipeline = RenderPipeline::new(
            PipelineDescriptor::default_settings(pbr_shader),
            ctx.render_resource,
        )
        .unwrap();

        Self {
            pbr_pipeline,
            main_camera: None,
            lights: UniformBuffer::new(),
        }
    }

    fn update(&mut self, ctx: PluginCtx) {
        self.editor_update(ctx);
    }
    
    fn editor_update(&mut self, _ctx: PluginCtx) {
        self.lights.clear();
    }

    fn render(&mut self, ctx: PluginRenderCtx) {
        // get lights

        // generate shadow_texture

        // generate shadow_map

        //todo!();
    }
}

#[derive(Default, Reflect, Inspect)]
pub struct Light3d {

}

impl Component for Light3d {
    type Plugins = Render3dPlugin;

    fn inspector_ui(&mut self, _: &mut Render3dPlugin, _: ComponentCtx, ui: &mut Ui) {
        self.inspect(ui);
    }

    fn update(&mut self, render: &mut Render3dPlugin, ctx: ComponentCtx) {
        self.editor_update(render, ctx);
    }

    fn editor_update(&mut self, render: &mut Render3dPlugin, ctx: ComponentCtx) {
        render.lights.push(ctx.global_transform.translation).expect("MAX_LIGHTS exceeded");
    }
}

#[derive(Default, Reflect, Inspect)]
pub struct Camera3d {
    pub projection: PerspectiveProjection,
}

impl Component for Camera3d {
    type Plugins = Render3dPlugin;

    fn inspector_ui(&mut self, _: &mut Render3dPlugin, _: ComponentCtx, ui: &mut Ui) {
        self.inspect(ui);
    }

    fn update(&mut self, plugins: &mut Render3dPlugin, ctx: ComponentCtx) {
        self.editor_update(plugins, ctx);
    }

    fn editor_update(&mut self, plugins: &mut Render3dPlugin, ctx: ComponentCtx) {
        let size = ctx.render_resource.target_size();
        self.projection.aspect = size.x / size.y;

        let view_proj = self.projection.matrix() * ctx.global_transform.matrix().inverse();
        plugins.main_camera = Some(view_proj);
    }
}

#[derive(Reflect, Inspect)]
pub struct Mesh3d {
    #[inspect(collapsing)]
    pub mesh: Mesh,
}

impl Default for Mesh3d {
    fn default() -> Self {
        let mut mesh = Mesh::new();

        mesh.add_attribute::<Vec3>("vertex_position");
        mesh.add_attribute::<Vec3>("vertex_normal");
        mesh.add_attribute::<Vec2>("vertex_uv");

        Self { mesh }
    }
}

impl Component for Mesh3d {
    type Plugins = Render3dPlugin;

    fn inspector_ui(&mut self, _: &mut Render3dPlugin, _: ComponentCtx, ui: &mut Ui) {
        self.inspect(ui);
    }

    fn render(&mut self, render: &mut Render3dPlugin, ctx: ComponentRenderCtx) {
        if let Some(view_proj) = &render.main_camera {
            let model = ctx.global_transform.matrix();

            render.pbr_pipeline.bind_uniform("Transform", &model);
            render.pbr_pipeline.bind_uniform("Camera", view_proj);
            render.pbr_pipeline.bind_uniform("Lights", &render.lights);

            ctx.render_pass
                .with_pipeline(&render.pbr_pipeline)
                .draw_mesh(&self.mesh);
        }
    }
}
