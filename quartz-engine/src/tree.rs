use crate::component::*;
use crate::node::*;
use crate::plugin::*;
use crate::state::Components;
use egui::*;
use quartz_render::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub struct Tree {
    pub(crate) nodes: HashMap<NodeId, NodeContainer>,
    pub(crate) parents: HashMap<NodeId, NodeId>,
    pub(crate) children: HashMap<NodeId, Vec<NodeId>>,
    pub(crate) base: HashSet<NodeId>,
    pub(crate) next_node_id: NodeId,
    pub(crate) despawn: Vec<NodeId>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
            base: HashSet::new(),
            next_node_id: NodeId(0),
            despawn: Vec::new(),
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

    pub fn spawn(&mut self, component: impl ToPod) -> NodeId {
        let id = self.generate_id();
        let node = Node::new(component.to_pod());
        self.nodes.insert(id, NodeContainer::new(node));
        self.base.insert(id);
        self.children.insert(id, Vec::new());

        id
    }

    pub fn spawn_child(&mut self, component: impl ToPod, parent_id: &NodeId) -> Option<NodeId> {
        if self.nodes.contains_key(parent_id) {
            let child_id = self.spawn(component);
            self.set_parent(*parent_id, child_id);

            Some(child_id)
        } else {
            None
        }
    }

    pub fn despawn(&mut self, node: NodeId) {
        self.despawn.push(node);
    }

    pub(crate) fn remove_recursive(&mut self, node: NodeId) {
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

    pub fn set_parent(&mut self, parent: NodeId, child: NodeId) {
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
    }

    pub fn get_children(&self, parent: NodeId) -> &Vec<NodeId> {
        self.children.get(&parent).unwrap()
    }

    pub fn get_parent(&self, child: NodeId) -> Option<NodeId> {
        self.parents.get(&child).cloned()
    }

    pub fn nodes_ui(
        &mut self,
        ui: &mut Ui,
        components: &Components,
        plugins: &Plugins,
        selected_node: &mut Option<NodeId>,
    ) {
        for id in self.base.clone() {
            self.node_ui(&id, components, plugins, ui, selected_node);
        }
    }

    pub fn node_ui(
        &mut self,
        node_id: &NodeId,
        components: &Components,
        plugins: &Plugins,
        ui: &mut Ui,
        selected_node: &mut Option<NodeId>,
    ) {
        if let Some(node) = self.get_node(node_id) {
            let selected = *selected_node == Some(*node_id);

            let children = self.get_children(*node_id).clone();

            let response = if !children.is_empty() {
                let response =
                    CollapsingHeader::new(&node.name)
                        .id_source(node_id)
                        .show(ui, |ui| {
                            for child in children {
                                self.node_ui(&child, components, plugins, ui, selected_node);
                            }
                        });

                response.header_response
            } else {
                ui.add(Button::new(&node.name))
            };

            if response.double_clicked() {
                *selected_node = Some(*node_id);
            }

            let popup_id = ui.make_persistent_id("add_component_id");

            if ui.input().key_pressed(Key::A) && ui.input().modifiers.ctrl && selected {
                ui.memory().toggle_popup(popup_id);
            }

            if selected {
                popup::popup_below_widget(ui, popup_id, &response, |ui| {
                    ui.set_max_width(200.0);

                    ScrollArea::from_max_height(300.0).show(ui, |ui| {
                        for component in components.components() {
                            if ui.button(component).clicked() {
                                let component = components.init(component, plugins).unwrap();

                                self.spawn_child(component, &selected_node.unwrap());
                            }
                        }
                    });
                });
            }
        }
    }

    pub fn update(&mut self, plugins: &Plugins, render_resource: &RenderResource) {
        for id in self.base.clone() {
            self.update_node(plugins, Transform::IDENTITY, &id, render_resource);
        }
    }

    pub fn update_node(
        &mut self,
        plugins: &Plugins,
        parent_transform: Transform,
        node_id: &NodeId,
        render_resource: &RenderResource,
    ) {
        if let Some(mut node) = self.get_node(node_id) {
            node.update(plugins, node_id, self, render_resource);

            node.global_transform = parent_transform * node.transform.clone();
            let global_transform = node.global_transform.clone();
            let children = self.get_children(*node_id).clone();

            drop(node);

            for child in children {
                self.update_node(plugins, global_transform.clone(), &child, render_resource);
            }
        }
    }

    pub fn render(
        &mut self,
        plugins: &Plugins,
        render_resource: &RenderResource,
        render_pass: &mut EmptyRenderPass<'_, '_, format::Rgba8UnormSrgb, format::Depth32Float>,
    ) {
        for id in self.nodes.keys().cloned().collect::<Vec<_>>() {
            if let Some(mut node) = self.get_node(&id) {
                node.render(plugins, &id, self, render_resource, render_pass);
            }
        }
    }

    pub fn get_node<'a>(&self, node_id: &NodeId) -> Option<NodeGuard<'a>> {
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
