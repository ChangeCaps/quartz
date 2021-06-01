use crate::transform::*;
use egui::*;
use std::collections::HashMap;
pub use quartz_engine_derive::Inspect;
use quartz_render::prelude::{Vec2, *};
use quartz_render::wgpu;
use crate::node::*;

pub trait Inspect {
    fn inspect(&mut self, ui: &mut Ui) -> bool;
}

macro_rules! impl_inspect {
    (drag => $ty:path) => {
        impl Inspect for $ty {
            fn inspect(&mut self, ui: &mut Ui) -> bool {
                ui.add(DragValue::new(self).speed(0.1)).changed()
            }
        }
    };
}

macro_rules! inspect {
    (drag $(($speed:expr))? $ui:ident => $value:expr) => {
        {
            $ui.add(DragValue::new($value)$(.speed($speed))?).changed()
        }
    };
    (inspect $ui:ident => $value:expr) => {
        $value.inspect($ui)
    };
    (field $ui:ident => $self:ident.$field:ident: $($tt:tt)*) => {
        {
            let $field = &mut $self.$field;
            $ui.horizontal(|ui| {
                ui.label(stringify!($field));
                inspect!($($tt)* ui => $field)
            }).inner
        }
    };
}

impl_inspect!(drag => i16);
impl_inspect!(drag => i32);
impl_inspect!(drag => i64);
impl_inspect!(drag => isize);

impl_inspect!(drag => f32);
impl_inspect!(drag => f64);

impl_inspect!(drag => u8);
impl_inspect!(drag => u16);
impl_inspect!(drag => u32);
impl_inspect!(drag => u64);
impl_inspect!(drag => usize);

impl Inspect for bool {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let prev = *self;

        ui.checkbox(self, "");

        *self != prev
    }
}

impl Inspect for String {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let prev = self.clone();
        ui.text_edit_singleline(self);
        *self == prev
    }
}

impl Inspect for Vec2 {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let prev = self.clone();

        ui.columns(2, |columns| {
            columns[0].add(DragValue::new(&mut self.x).speed(0.1));
            columns[1].add(DragValue::new(&mut self.y).speed(0.1));
        });

        *self == prev
    }
}

impl Inspect for Vec3 {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let prev = self.clone();

        ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.x).speed(0.1));
            columns[1].add(DragValue::new(&mut self.y).speed(0.1));
            columns[2].add(DragValue::new(&mut self.z).speed(0.1));
        });

        *self == prev
    }
}

impl Inspect for Vec4 {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let prev = self.clone();

        ui.columns(4, |columns| {
            columns[0].add(DragValue::new(&mut self.x).speed(0.1));
            columns[1].add(DragValue::new(&mut self.y).speed(0.1));
            columns[2].add(DragValue::new(&mut self.z).speed(0.1));
            columns[3].add(DragValue::new(&mut self.w).speed(0.1));
        });

        *self == prev
    }
}

impl Inspect for IVec2 {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let prev = self.clone();

        ui.columns(2, |columns| {
            columns[0].add(DragValue::new(&mut self.x).speed(0.1));
            columns[1].add(DragValue::new(&mut self.y).speed(0.1));
        });

        *self == prev
    }
}

impl Inspect for IVec3 {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let prev = self.clone();

        ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.x).speed(0.1));
            columns[1].add(DragValue::new(&mut self.y).speed(0.1));
            columns[2].add(DragValue::new(&mut self.z).speed(0.1));
        });

        *self == prev
    }
}

impl Inspect for IVec4 {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let prev = self.clone();

        ui.columns(4, |columns| {
            columns[0].add(DragValue::new(&mut self.x).speed(0.1));
            columns[1].add(DragValue::new(&mut self.y).speed(0.1));
            columns[2].add(DragValue::new(&mut self.z).speed(0.1));
            columns[3].add(DragValue::new(&mut self.w).speed(0.1));
        });

        *self == prev
    }
}

impl Inspect for NodeId {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        ui.add(DragValue::new(&mut self.0).speed(0.1)).changed()
    }
}

impl Inspect for Color {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let prev = self.clone();

        let mut color = [self.r, self.g, self.b, self.a];

        ui.color_edit_button_rgba_unmultiplied(&mut color);

        self.r = color[0];
        self.g = color[1];
        self.b = color[2];
        self.a = color[3];

        *self == prev
    }
}

impl Inspect for PerspectiveProjection {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let mut mutated = false;

        ui.vertical(|ui| {
            mutated |= inspect!(field ui => self.aspect: drag(0.05));
            mutated |= inspect!(field ui => self.fov: drag(0.1));
            mutated |= inspect!(field ui => self.far: drag(0.1));
            mutated |= inspect!(field ui => self.near: drag(0.1));
        });

        mutated
    }
}

impl Inspect for OrthographicProjection {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let mut mutated = false;

        ui.vertical(|ui| {
            mutated |= inspect!(field ui => self.left: drag(0.1));
            mutated |= inspect!(field ui => self.right: drag(0.1));
            mutated |= inspect!(field ui => self.bottom: drag(0.1));
            mutated |= inspect!(field ui => self.top: drag(0.1));
            mutated |= inspect!(field ui => self.far: drag(0.1));
            mutated |= inspect!(field ui => self.near: drag(0.1));
        });

        mutated
    }
}

impl Inspect for Transform {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let mut mutated = false;

        ui.vertical(|ui| {
            mutated |= self.translation.inspect(ui);

            let (mut yaw, mut pitch, mut roll) = self.rotation.to_euler(EulerRot::XYZ);
            yaw *= 180.0 / std::f32::consts::PI;
            pitch *= 180.0 / std::f32::consts::PI;
            roll *= 180.0 / std::f32::consts::PI;

            ui.horizontal(|ui| {
                ui.columns(3, |columns| {
                    let ui = &mut columns[0];
                    mutated |= inspect!(drag(0.02) ui => &mut yaw);
                    let ui = &mut columns[1];
                    mutated |= inspect!(drag(0.02) ui => &mut pitch);
                    let ui = &mut columns[2];
                    mutated |= inspect!(drag(0.02) ui => &mut roll);
                });
            });

            yaw /= 180.0 / std::f32::consts::PI;
            pitch /= 180.0 / std::f32::consts::PI;
            roll /= 180.0 / std::f32::consts::PI;

            self.rotation = Quat::from_euler(EulerRot::XYZ, yaw, pitch, roll);

            mutated |= self.scale.inspect(ui);
        });

        mutated
    }
}

impl<K: Inspect + Clone, V: Inspect> Inspect for HashMap<K, V> {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let mut mutated = false;

        for (k, v) in self.iter_mut() {
            ui.vertical(|ui| {
                let mut k = k.clone();
                k.inspect(ui);
                mutated |= v.inspect(ui);
            });
        }

        mutated
    }
}

impl<T: Inspect> Inspect for [T] {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let mut mutated = false;

        for item in self {
            mutated |= item.inspect(ui);
        }

        mutated
    }
}

impl<T: Inspect + Default> Inspect for Vec<T> {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let mut mutated = false;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    self.push(T::default());
                    mutated = true;
                }

                if ui.button("-").clicked() {
                    self.pop();
                    mutated = true;
                }
            });

            ui.indent("vec", |ui| {
                for item in self {
                    mutated |= item.inspect(ui);
                }
            });
        });

        mutated
    }
}

impl Inspect for Mesh {
    fn inspect(&mut self, ui: &mut Ui) -> bool {
        let mut mutated = false;

        ui.vertical(|ui| {
            ui.label("attributes");
            ui.indent("mesh_attributes", |ui| {
                for name in self.vertex_data().keys().cloned().collect::<Vec<_>>() {
                    let mut attribute_mutated = false;

                    let vertex_data = self.vertex_data_mut().get_mut(&name).unwrap();
                    let size = vertex_data.format.size();
                    let format = vertex_data.format;
                    let len = vertex_data.data.len();
                    let default = || match format {
                        wgpu::VertexFormat::Float32x2 => Vec2::ZERO.to_bytes(),
                        wgpu::VertexFormat::Float32x3 => Vec3::ZERO.to_bytes(),
                        wgpu::VertexFormat::Float32x4 => Vec4::ZERO.to_bytes(),
                        _ => unimplemented!(),
                    };
                    let data = &mut vertex_data.data;

                    ui.horizontal(|ui| {
                        ui.label(&name);
                        if ui.button("+").clicked() {
                            data.append(&mut default().to_vec());
                            attribute_mutated = true;
                        }

                        if ui.button("-").clicked() {
                            for _ in 0..size {
                                data.pop();
                            }
                            attribute_mutated = true;
                        }
                    });

                    if len > 0 {
                        ui.indent(&name, |ui| {
                            let m = match self.vertex_data().get(&name).unwrap().format {
                                wgpu::VertexFormat::Float32x2 => self
                                    .get_attribute_mut_unmarked::<Vec2>(&name)
                                    .unwrap()
                                    .inspect(ui),
                                wgpu::VertexFormat::Float32x3 => self
                                    .get_attribute_mut_unmarked::<Vec3>(&name)
                                    .unwrap()
                                    .inspect(ui),
                                wgpu::VertexFormat::Float32x4 => self
                                    .get_attribute_mut_unmarked::<Vec4>(&name)
                                    .unwrap()
                                    .inspect(ui),
                                _ => unimplemented!(),
                            };

                            attribute_mutated |= m;
                        });
                    }

                    if attribute_mutated {
                        self.invalidate_vertex_buffer(&name);

                        mutated = true;
                    }
                }
            });

            ui.label("indices");
            if self.indices_mut_unmarked().inspect(ui) {
                self.invalidate_index_buffer();

                mutated = true;
            }
        });

        mutated
    }
}
