use crate::component::*;
use crate::node::*;
use egui::*;
use quartz_render::prelude::*;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub struct Tree {
    pub(crate) nodes: HashMap<NodeId, NodeContainer>,
    pub(crate) base: Vec<NodeId>,
    pub(crate) next_node_id: NodeId,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            base: Vec::new(),
            next_node_id: NodeId(0),
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

    pub fn spawn(&mut self, component: impl Component) -> NodeId {
        let id = self.generate_id();
        let node = Node::new(component);
        self.nodes.insert(id, NodeContainer::new(node));
        self.base.push(id);

        id
    }

    pub fn nodes_ui(&self, ui: &mut Ui, selected_node: &mut Option<NodeId>) {
        for id in &self.base {
            self.node_ui(id, ui, selected_node);
        }
    }

    pub fn node_ui(&self, node_id: &NodeId, ui: &mut Ui, selected_node: &mut Option<NodeId>) {
        if let Some(node) = self.get_node(node_id) {
            let response = if !node.children.is_empty() {
                let response = ui.collapsing(&node.name, |ui| {
                    for child in &node.children {
                        self.node_ui(child, ui, selected_node);
                    }
                });

                response.header_response
            } else {
                ui.button(&node.name)
            };

            if response.double_clicked() {
                *selected_node = Some(*node_id);
            }
        }
    }

    pub fn update(&self, render_resource: &RenderResource) {
        for id in &self.base {
            self.update_node(id, render_resource);
        }
    }

    pub fn update_node(&self, node_id: &NodeId, render_resource: &RenderResource) {
        if let Some(mut node) = self.get_node(node_id) {
            node.update(render_resource);

            let children = node.children.clone();

            drop(node);

            for child in children {
                self.update_node(&child, render_resource);
            }
        }
    }

    pub fn render(
        &self,
        render_resource: &RenderResource,
        render_pass: &mut EmptyRenderPass<'_, '_, format::Rgba8UnormSrgb, format::Depth32Float>,
    ) {
        for (_id, node) in &self.nodes {
            if let Some(mut node) = node.guard() {
                node.render(render_resource, render_pass);
            }
        }
    }

    pub fn get_node<'a>(&'a self, node_id: &NodeId) -> Option<NodeGuard<'a>> {
        if let Some(container) = self.nodes.get(node_id) {
            container.guard()
        } else {
            None
        }
    }
}

pub struct NodeContainer {
    node: Arc<Mutex<Option<Node>>>,
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
