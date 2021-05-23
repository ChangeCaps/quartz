use crate::render::*;
use spirv_reflect::types::{
    descriptor::ReflectDescriptorType, image::ReflectFormat, variable::ReflectDimension,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use wgpu::{
    BlendState, ColorWrite, CompareFunction, DepthBiasState, FrontFace, PolygonMode,
    PrimitiveState, PrimitiveTopology, StencilState,
};

/// Descriptor binding location.
pub struct Location {
    pub set: usize,
    pub binding: usize,
}

/// The type of a binding in a shader.
#[derive(Clone, PartialEq, Eq)]
pub enum BindingType {
    Texture {
        view_dimension: wgpu::TextureViewDimension,
        multisampled: bool,
    },
    Sampler,
    Buffer,
    UniformBuffer,
}

/// Implemented for anything that can be bound
pub trait Binding: Any {
    fn prepare_resource(&mut self, _render_resource: &RenderResource) {}
    fn binding_resource(&self) -> wgpu::BindingResource;
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    fn binding_clone(&self) -> Box<dyn Binding>;
}

impl Clone for Box<dyn Binding> {
    fn clone(&self) -> Self {
        self.binding_clone()
    }
}

fn downcast_mut<T: Binding>(binding: &mut dyn Binding) -> Option<&mut T> {
    if TypeId::of::<T>() == Binding::type_id(binding) {
        // SAFETY: just checked that binding has the same type_id as T, which means casting is safe
        Some(unsafe { &mut *(binding as *mut _ as *mut T) })
    } else {
        None
    }
}

#[derive(Clone)]
pub struct Bindings {
    bindings: HashMap<String, Box<dyn Binding>>,
}

impl std::fmt::Debug for Bindings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Bindings")?;

        Ok(())
    }
}

impl Bindings {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn bind(&mut self, ident: impl Into<String>, binding: impl Binding + 'static) {
        self.bindings.insert(ident.into(), Box::new(binding));
    }

    pub fn get(&self, ident: &String) -> Option<&Box<dyn Binding>> {
        self.bindings.get(ident)
    }

    pub fn get_mut(&mut self, ident: &String) -> Option<&mut Box<dyn Binding>> {
        self.bindings.get_mut(ident)
    }

    pub fn generate_groups<C: TextureFormat, D: TextureFormat>(
        &mut self,
        pipeline: &RenderPipeline<C, D>,
        render_resource: &RenderResource,
    ) -> Vec<Arc<wgpu::BindGroup>> {
        pipeline
            .shader_layout
            .bind_groups
            .iter()
            .map(|bind_group| {
                for (_binding, entry) in bind_group.bindings.iter() {
                    self.get_mut(&entry.ident)
                        .expect(format!("{} not bound", entry.ident).as_str())
                        .prepare_resource(render_resource);
                }

                let entries = bind_group
                    .bindings
                    .iter()
                    .map(|(_binding, entry)| {
                        let binding = self.get(&entry.ident).unwrap();

                        wgpu::BindGroupEntry {
                            binding: entry.binding,
                            resource: binding.binding_resource(),
                        }
                    })
                    .collect::<Vec<_>>();

                Arc::new(
                    render_resource
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Bind Group"),
                            layout: bind_group.layout.as_ref().unwrap(),
                            entries: &entries,
                        }),
                )
            })
            .collect::<Vec<_>>()
    }
}

/// Shader side descriptor set binding.
#[derive(Clone, PartialEq, Eq)]
pub struct BindGroupEntry {
    pub ident: String,
    pub binding: u32,
    pub ty: BindingType,
}

/// Shader side descriptor set.
#[derive(Clone)]
pub struct BindGroup {
    /// Entries.
    pub bindings: HashMap<u32, BindGroupEntry>,
    /// Wgpu internal [`wgpu::BindGroupLayout`].
    pub layout: Option<Arc<wgpu::BindGroupLayout>>,
}

impl BindGroup {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            layout: None,
        }
    }
}

/// Shader side input descriptor.
#[derive(Clone)]
pub struct VertexAttributeLayout {
    /// Buffer offset.
    pub offset: u64,
    /// Location in shader.
    pub shader_location: u32,
    /// Vertex format.
    pub format: wgpu::VertexFormat,
}

/// Layout of the shader side of a [`RenderPipeline`].
#[derive(Clone)]
pub struct PipelineLayout {
    /// Bind groups.
    pub bind_groups: Vec<BindGroup>,
    /// Vertex attributes.
    pub vertex_attributes: HashMap<String, VertexAttributeLayout>,
}

pub struct ColorTargetState<F: TextureFormat> {
    pub blend: Option<BlendState>,
    pub write_mask: ColorWrite,
    _marker: std::marker::PhantomData<F>,
}

impl<F: TextureFormat> Default for ColorTargetState<F> {
    fn default() -> Self {
        Self {
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrite::ALL,
            _marker: Default::default(),
        }
    }
}

pub struct DepthStencilState<F: TextureFormat> {
    pub depth_write_enabled: bool,
    pub depth_compare: CompareFunction,
    pub stencil: StencilState,
    pub bias: DepthBiasState,
    _marker: std::marker::PhantomData<F>,
}

impl<F: TextureFormat> Default for DepthStencilState<F> {
    fn default() -> Self {
        Self {
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
            _marker: Default::default(),
        }
    }
}

/// Used to create a [`RenderPipeline`].
pub struct PipelineDescriptor<
    C: TextureFormat = format::TargetFormat,
    D: TextureFormat = format::Depth32Float,
> {
    /// The shader for the pipeline
    pub shader: Shader,
    pub targets: Vec<ColorTargetState<C>>,
    pub depth_stencil: Option<DepthStencilState<D>>,
    pub primitive: PrimitiveState,
}

impl<C: TextureFormat, D: TextureFormat> PipelineDescriptor<C, D> {
    pub fn default_settings(shader: Shader) -> Self {
        Self {
            shader,
            targets: vec![ColorTargetState {
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrite::ALL,
                ..Default::default()
            }],
            depth_stencil: Some(DepthStencilState {
                ..Default::default()
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
        }
    }
}

/// Needed for rendering a renderpass.
///
/// Keeps track of [`Bindings`].
pub struct RenderPipeline<
    C: TextureFormat = format::TargetFormat,
    D: TextureFormat = format::Depth32Float,
> {
    pub(crate) descriptor: PipelineDescriptor<C, D>,
    pub(crate) shader_layout: PipelineLayout,
    pub(crate) vs_module: wgpu::ShaderModule,
    pub(crate) fs_module: wgpu::ShaderModule,
    pub(crate) bindings: Mutex<Bindings>,
    pub(crate) bind_groups: Mutex<Vec<Arc<wgpu::BindGroup>>>,
    pub(crate) layout: Option<wgpu::PipelineLayout>,
    pub(crate) pipeline: Arc<wgpu::RenderPipeline>,
    pub(crate) bindings_changed: AtomicBool,
}

impl<C: TextureFormat, D: TextureFormat> RenderPipeline<C, D> {
    /// Creates a pipeline.
    pub fn new(
        descriptor: PipelineDescriptor<C, D>,
        render_resource: &RenderResource,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let limis = render_resource.device.limits();

        let mut bind_groups = (0..limis.max_bind_groups as usize)
            .into_iter()
            .map(|_| BindGroup::new())
            .collect::<Vec<_>>();

        let vs_reflection =
            spirv_reflect::ShaderModule::load_u8_data(descriptor.shader.vs_spirv.as_binary_u8())?;
        let fs_reflection =
            spirv_reflect::ShaderModule::load_u8_data(descriptor.shader.fs_spirv.as_binary_u8())?;

        for binding in vs_reflection
            .enumerate_descriptor_bindings(None)?
            .into_iter()
            .chain(fs_reflection.enumerate_descriptor_bindings(None)?)
        {
            let entry = BindGroupEntry {
                ident: match binding.descriptor_type {
                    ReflectDescriptorType::UniformBuffer => {
                        binding.type_description.unwrap().type_name
                    }
                    _ => binding.name,
                },
                binding: binding.binding,
                ty: match binding.descriptor_type {
                    ReflectDescriptorType::StorageBuffer => BindingType::Buffer,
                    ReflectDescriptorType::UniformBuffer => BindingType::UniformBuffer,
                    ReflectDescriptorType::SampledImage => BindingType::Texture {
                        view_dimension: match binding.image.dim {
                            ReflectDimension::Type1d => wgpu::TextureViewDimension::D1,
                            ReflectDimension::Type2d => {
                                if binding.image.arrayed > 0 {
                                    wgpu::TextureViewDimension::D2Array
                                } else {
                                    wgpu::TextureViewDimension::D2
                                }
                            }
                            ReflectDimension::Type3d => wgpu::TextureViewDimension::D3,
                            ReflectDimension::Cube => wgpu::TextureViewDimension::Cube,
                            _ => panic!("Texture type unsupported"),
                        },
                        multisampled: binding.image.ms > 1,
                    },
                    ReflectDescriptorType::Sampler => BindingType::Sampler,
                    _ => return Err("Descriptor type unsupported".into()),
                },
            };

            if let Some(existing_entry) = bind_groups[binding.set as usize]
                .bindings
                .get(&binding.binding)
            {
                if *existing_entry != entry {
                    return Err("Overlapping bindings".into());
                }
            } else {
                bind_groups[binding.set as usize]
                    .bindings
                    .insert(binding.binding, entry);
            }
        }

        let vertex_attributes = vs_reflection
            .enumerate_input_variables(None)?
            .into_iter()
            .filter(|input| {
                !input
                    .decoration_flags
                    .contains(spirv_reflect::types::variable::ReflectDecorationFlags::BUILT_IN)
            })
            .map(|input| {
                let layout = VertexAttributeLayout {
                    offset: 0,
                    shader_location: input.location,
                    format: match input.format {
                        ReflectFormat::R32_SFLOAT => wgpu::VertexFormat::Float32,
                        ReflectFormat::R32G32_SFLOAT => wgpu::VertexFormat::Float32x2,
                        ReflectFormat::R32G32B32_SFLOAT => wgpu::VertexFormat::Float32x3,
                        ReflectFormat::R32G32B32A32_SFLOAT => wgpu::VertexFormat::Float32x4,
                        ReflectFormat::R32_SINT => wgpu::VertexFormat::Sint32,
                        ReflectFormat::R32G32_SINT => wgpu::VertexFormat::Sint32x2,
                        ReflectFormat::R32G32B32_SINT => wgpu::VertexFormat::Sint32x3,
                        ReflectFormat::R32G32B32A32_SINT => wgpu::VertexFormat::Sint32x4,
                        _ => panic!("Unsupported input format {:?}", input),
                    },
                };

                (input.name, layout)
            })
            .collect::<HashMap<_, _>>();

        let mut attributes = vertex_attributes
            .iter()
            .map(|(_, attr)| {
                vec![wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: attr.shader_location,
                    format: attr.format,
                }]
            })
            .collect::<Vec<_>>();

        attributes.sort_by(|a, b| a[0].shader_location.cmp(&b[0].shader_location));

        let buffers = attributes
            .iter()
            .map(|attr| wgpu::VertexBufferLayout {
                array_stride: attr[0].format.size(),
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &attr,
            })
            .collect::<Vec<_>>();

        let (vs_module, fs_module) = descriptor.shader.to_modules(render_resource);

        let layouts = bind_groups
            .iter_mut()
            .map(|bind_group| {
                let entries = bind_group
                    .bindings
                    .iter()
                    .map(|(_binding, entry)| wgpu::BindGroupLayoutEntry {
                        binding: entry.binding,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: match entry.ty {
                            BindingType::Sampler => wgpu::BindingType::Sampler {
                                filtering: true,
                                comparison: false,
                            },
                            BindingType::Texture {
                                view_dimension,
                                multisampled,
                            } => wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension,
                                multisampled,
                            },
                            BindingType::UniformBuffer => wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            _ => panic!(),
                        },
                        count: None,
                    })
                    .collect::<Vec<_>>();

                let layout = render_resource.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: Some("Bind Group Layout"),
                        entries: &entries,
                    },
                );

                bind_group.layout = Some(Arc::new(layout));

                &**bind_group.layout.as_ref().unwrap()
            })
            .collect::<Vec<_>>();

        let layout =
            render_resource
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline Layout"),
                    bind_group_layouts: &layouts,
                    push_constant_ranges: &[],
                });

        let target_format = render_resource.target_format();

        let targets = descriptor
            .targets
            .iter()
            .map(|target| wgpu::ColorTargetState {
                format: C::format(target_format),
                blend: target.blend.clone(),
                write_mask: target.write_mask.clone(),
            })
            .collect::<Vec<_>>();

        let depth_stencil =
            descriptor
                .depth_stencil
                .as_ref()
                .map(|depth_stencil| wgpu::DepthStencilState {
                    format: D::format(target_format),
                    depth_write_enabled: depth_stencil.depth_write_enabled,
                    depth_compare: depth_stencil.depth_compare.clone(),
                    stencil: depth_stencil.stencil.clone(),
                    bias: depth_stencil.bias.clone(),
                });

        let pipeline =
            render_resource
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &vs_module,
                        entry_point: "main",
                        buffers: &buffers,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &fs_module,
                        entry_point: "main",
                        targets: &targets,
                    }),
                    primitive: descriptor.primitive.clone(),
                    depth_stencil,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                });

        Ok(Self {
            descriptor,
            shader_layout: PipelineLayout {
                bind_groups,
                vertex_attributes,
            },
            vs_module,
            fs_module,
            bindings: Mutex::new(Bindings::new()),
            bind_groups: Mutex::new(Vec::new()),
            layout: Some(layout),
            pipeline: Arc::new(pipeline),
            bindings_changed: AtomicBool::new(true),
        })
    }

    /// Sets entire [`Bindings`].
    pub fn set_bindings(&self, bindings: Bindings) {
        *self.bindings.lock().unwrap() = bindings;
        self.bindings_changed.store(true, Ordering::SeqCst);
    }

    /// Binds a binding.
    pub fn bind(&self, ident: impl Into<String>, binding: impl Binding + 'static) {
        self.bindings.lock().unwrap().bind(ident, binding);
        self.bindings_changed.store(true, Ordering::SeqCst);
    }

    /// Binds a uniform.
    pub fn bind_uniform(&self, ident: impl Into<String>, uniform: impl Uniform) {
        let mut bindings = self.bindings.lock().unwrap();
        let ident = ident.into();

        if let Some(binding) = bindings.get_mut(&ident) {
            if let Some(uniform_buffer) = downcast_mut::<UniformBuffer>(binding.as_mut()) {
                uniform_buffer.set_uniform(uniform);

                self.bindings_changed.store(true, Ordering::SeqCst);
            } else {
                drop(bindings);
                self.bind(ident, UniformBuffer::new(uniform));
            }
        } else {
            drop(bindings);
            self.bind(ident, UniformBuffer::new(uniform));
        }
    }

    /// Updates the bindings on the gpu.
    pub fn submit_bindings(&self, render_resource: &RenderResource) {
        if !self.bindings_changed.swap(false, Ordering::SeqCst) {
            return;
        }

        let bind_groups = self
            .bindings
            .lock()
            .unwrap()
            .generate_groups(self, render_resource);

        *self.bind_groups.lock().unwrap() = bind_groups;
    }
}
