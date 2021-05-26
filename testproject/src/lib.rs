use quartz_engine::egui::*;
use quartz_engine::prelude::*;

#[derive(Reflect, Inspect, Default)]
pub struct Terrain {
    pub size: u32,
    pub scale: f32,
}

impl Terrain {
    pub fn generate(&self, ctx: ComponentCtx) {
        for child in ctx.tree.get_children(ctx.node_id) {
            let mut child = ctx.tree.get_node(child).unwrap();

            if let Some(mesh) = child.get_component_mut::<ProceduralMesh3d>() {
                println!("generating");

                let mut positions = Vec::new();
                let mut indices = Vec::new();
                let mut normals = Vec::new();

                for x in 0..self.size {
                    for z in 0..self.size {
                        let mut pos = Vec3::new(x as f32, 0.0, z as f32);
                        pos -= Vec3::splat(self.size as f32 / 2.0);
                        pos *= self.scale;

                        let h = |p: Vec3| p.x.sin() * p.z.sin();
                        let e = 0.01;

                        let y = h(pos);
                        pos.y = y;
                        let normal = Vec3::new(
                            y - h(pos + Vec3::new(e, 0.0, 0.0)),
                            e,
                            y - h(pos + Vec3::new(0.0, 0.0, e)),
                        )
                        .normalize();

                        positions.push(pos);
                        normals.push(normal);
                    }
                }

                for x in 0..self.size - 1 {
                    for z in 0..self.size - 1 {
                        let index = z * self.size + x;

                        indices.push(index);
                        indices.push(index + 1);
                        indices.push(index + self.size);
                        indices.push(index + self.size + 1);
                        indices.push(index + self.size);
                        indices.push(index + 1);
                    }
                }

                mesh.mesh.set_attribute("vertex_position", positions);
                mesh.mesh.set_attribute("vertex_normal", normals);
                mesh.mesh.set_indices(indices);
            }
        }
    }
}

impl Component for Terrain {
    type Plugins = ();

    fn inspector_ui(&mut self, _: (), ctx: ComponentCtx, ui: &mut Ui) {
        if ui.button("Generate Mesh").clicked() {
            self.generate(ctx);
        }

        self.inspect(ui);
    }
}

fn register_types(types: &mut Types) {
    types.register_component::<Terrain>();
}

quartz_engine::register_types!(register_types);
