use crate::component::*;
use crate::node::*;
use crate::plugin::*;
use crate::transform::*;
use linked_hash_map::LinkedHashMap;
use quartz_render::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub struct Tree {
    pub(crate) nodes: LinkedHashMap<NodeId, NodeContainer>,
    pub(crate) parents: HashMap<NodeId, NodeId>,
    pub(crate) children: HashMap<NodeId, Vec<NodeId>>,
    pub(crate) base: HashSet<NodeId>,
    pub(crate) next_node_id: NodeId,
    pub(crate) despawn: Vec<NodeId>,
    pub(crate) added: Vec<NodeId>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            nodes: LinkedHashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
            base: HashSet::new(),
            next_node_id: NodeId(0),
            despawn: Vec::new(),
            added: Vec::new(),
        }
    }

    pub fn len(&mut self) -> usize {
        self.nodes.len()
    }

    pub fn generate_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id.0 += 1;
        id
    }

    pub fn nodes(&self) -> Vec<NodeId> {
        self.nodes.keys().cloned().collect()
    }

    pub fn spawn(&mut self) -> NodeId {
        let id = self.generate_id();
        let node = Node::new();
        self.nodes.insert(id, NodeContainer::new(node));
        self.base.insert(id);
        self.children.insert(id, Vec::new());
        self.added.push(id);

        id
    }

    pub fn spawn_child(&mut self, parent_id: impl Into<NodeId>) -> Option<NodeId> {
        let parent_id = parent_id.into();

        if self.nodes.contains_key(&parent_id) {
            let child_id = self.spawn();
            self.set_parent(child_id, parent_id);

            Some(child_id)
        } else {
            None
        }
    }

    pub fn despawn(&mut self, node: impl Into<NodeId>) {
        self.despawn.push(node.into());
    }

    pub(crate) fn remove_recursive(&mut self, node: impl Into<NodeId>) {
        let node = node.into();
        self.nodes.remove(&node);

        if let Some(parent) = self.parents.remove(&node) {
            if let Some(children) = self.children.get_mut(&parent) {
                children.retain(|n| *n != node);
            }
        }

        for child in self.children.remove(&node).unwrap() {
            self.remove_recursive(child);
        }
    }

    pub fn set_parent(&mut self, child: impl Into<NodeId>, parent: impl Into<Option<NodeId>>) {
        let parent = parent.into();
        let child = child.into();

        if let Some(parent) = parent {
            if let Some(parent) = self.parents.remove(&child) {
                if let Some(children) = self.children.get_mut(&parent) {
                    children.retain(|c| *c != child);
                }
            }

            self.base.remove(&child);
            self.parents.insert(child, parent);
            self.children
                .entry(parent)
                .or_insert(Vec::new())
                .push(child);
        } else {
            if let Some(parent) = self.parents.remove(&child) {
                if let Some(children) = self.children.get_mut(&parent) {
                    children.retain(|c| *c != child);
                }
            }

            self.base.insert(child);
        }
    }

    pub fn get_children(&self, parent: impl Into<NodeId>) -> &Vec<NodeId> {
        let parent = parent.into();
        self.children.get(&parent).unwrap()
    }

    pub fn get_parent(&self, child: impl Into<NodeId>) -> Option<NodeId> {
        let child = child.into();
        self.parents.get(&child).cloned()
    }

    pub fn get_node<'a>(&self, node_id: impl Into<NodeId>) -> Option<NodeGuard<'a>> {
        let node_id = node_id.into();
        if let Some(container) = self.nodes.get(&node_id) {
            container.guard()
        } else {
            None
        }
    }
}

#[cfg(feature = "editor_bridge")]
impl Tree {
    pub(crate) fn despawn_recursive(
        &mut self,
        node_id: &NodeId,
        plugins: &Plugins,
        instance: &Instance,
    ) {
        if let Some(mut node) = self.get_node(node_id) {
            self.base.remove(node_id);

            node.despawn(plugins, node_id, self, instance);

            for child in self.get_children(*node_id).clone() {
                self.despawn_recursive(&child, plugins, instance);
            }
        }
    }

    pub fn start(&mut self, plugins: &Plugins, instance: &Instance) {
        for node_id in self.nodes() {
            if let Some(mut node) = self.get_node(&node_id) {
                node.start(plugins, &node_id, self, instance);
            }
        }
    }

    pub fn editor_start(&mut self, plugins: &Plugins, instance: &Instance) {
        for node_id in self.nodes() {
            if let Some(mut node) = self.get_node(&node_id) {
                node.editor_start(plugins, &node_id, self, instance);
            }
        }
    }

    pub fn update(&mut self, plugins: &Plugins, instance: &Instance) {
        for node_id in self.nodes() {
            if let Some(mut node) = self.get_node(&node_id) {
                node.update(plugins, &node_id, self, instance);
            }
        }
    }

    pub fn editor_update(&mut self, plugins: &Plugins, instance: &Instance) {
        for node_id in self.nodes() {
            if let Some(mut node) = self.get_node(&node_id) {
                node.editor_update(plugins, &node_id, self, instance);
            }
        }
    }

    pub fn update_transforms(&self) {
        for node_id in &self.base {
            self.update_transform(Transform::IDENTITY, node_id);
        }
    }

    pub fn update_transform(&self, parent_transform: Transform, node_id: &NodeId) {
        if let Some(mut node) = self.get_node(node_id) {
            let global_transform = parent_transform * node.transform.clone();
            node.global_transform = global_transform.clone();

            drop(node);

            for child in self.get_children(*node_id) {
                self.update_transform(global_transform.clone(), child);
            }
        }
    }

    pub fn render(
        &mut self,
        plugins: &Plugins,
        viewport_camera: &Option<Mat4>,
        instance: &Instance,
        render_pass: &mut EmptyRenderPass<'_, '_, '_, format::TargetFormat, format::Depth32Float>,
    ) {
        for id in self.nodes() {
            if let Some(mut node) = self.get_node(&id) {
                node.render(plugins, &id, self, viewport_camera, instance, render_pass);
            }
        }
    }

    pub fn viewport_render(
        &mut self,
        plugins: &Plugins,
        viewport_camera: &Option<Mat4>,
        instance: &Instance,
        render_pass: &mut EmptyRenderPass<'_, '_, '_, format::TargetFormat, format::Depth32Float>,
    ) {
        for id in self.nodes() {
            if let Some(mut node) = self.get_node(&id) {
                node.viewport_render(plugins, &id, self, viewport_camera, instance, render_pass);
            }
        }
    }

    pub fn viewport_pick_render(
        &mut self,
        plugins: &Plugins,
        viewport_camera: &Mat4,
        render_pipeline: &RenderPipeline<format::R32Uint>,
        instance: &Instance,
        render_pass: &mut RenderPass<'_, '_, '_, format::R32Uint, format::Depth32Float>,
    ) {
        for node_id in self.nodes() {
            if let Some(mut node) = self.get_node(&node_id) {
                render_pipeline.bind_uniform("NodeId", &(node_id.0 as u32));
                render_pipeline.bind_uniform("Camera", viewport_camera);
                render_pipeline.bind_uniform("Transform", &node.global_transform.matrix());

                node.viewport_pick_render(
                    plugins,
                    &node_id,
                    self,
                    viewport_camera,
                    instance,
                    render_pass,
                );
            }
        }
    }
}

pub struct NodeContainer {
    pub(crate) node: Arc<Mutex<Option<Node>>>,
}

impl NodeContainer {
    pub fn new(node: Node) -> Self {
        Self {
            node: Arc::new(Mutex::new(Some(node))),
        }
    }

    pub fn guard<'a>(&self) -> Option<NodeGuard<'a>> {
        if let Some(node) = self.node.lock().unwrap().take() {
            Some(NodeGuard {
                container: self.node.clone(),
                node: Some(node),
                _marker: Default::default(),
            })
        } else {
            None
        }
    }
}

pub struct NodeGuard<'a> {
    container: Arc<Mutex<Option<Node>>>,
    node: Option<Node>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl std::ops::Deref for NodeGuard<'_> {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        self.node.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for NodeGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.node.as_mut().unwrap()
    }
}

impl Drop for NodeGuard<'_> {
    fn drop(&mut self) {
        *self.container.lock().unwrap() = self.node.take();
    }
}
