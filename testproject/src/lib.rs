use quartz_engine::egui::*;
use quartz_engine::prelude::*;

#[derive(Reflect, Inspect, Default)]
pub struct Terrain {

}

impl Terrain {
    pub fn generate(&self, ctx: ComponentCtx) {
        for child in ctx.tree.get_children(ctx.node_id) {
            let mut child = ctx.tree.get_node(child).unwrap();

            if let Some(mesh) = child.get_component_mut::<Mesh3d>() {
                println!("generating");

                let mut positions = Vec::new();
                let mut indices = Vec::new();
                
                for x in -20..20 {
                    for z in -20..20 {
                        let mut pos = Vec3::new(x as f32, 0.0, z as f32);
                        pos.y = pos.x * pos.z * 0.03;

                        positions.push(pos);
                    }
                }

                for x in 0..39 {
                    for z in 0..39 {
                        let index = z * 40 + x;

                        indices.push(index);
                        indices.push(index + 1);
                        indices.push(index + 40);
                        indices.push(index + 41);
                        indices.push(index + 40);
                        indices.push(index + 1);
                    }
                }

                let mut normals = vec![Vec3::ZERO; positions.len()];

                for tri in 0..indices.len() / 3 {
                    let i0 = indices[tri * 3 + 0];
                    let i1 = indices[tri * 3 + 1];
                    let i2 = indices[tri * 3 + 2];

                    let v0 = positions[i0 as usize];
                    let v1 = positions[i1 as usize];
                    let v2 = positions[i2 as usize];
                
                    let normal = (v1 - v0).cross(v2 - v0);

                    normals[i0 as usize] += normal;
                    normals[i1 as usize] += normal;
                    normals[i2 as usize] += normal;
                }

                for norm in &mut normals {
                    *norm = norm.normalize();
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