use crate::component::*;
use crate::node::*;
use crate::plugin::*;
use quartz_render::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

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

    pub fn get_node<'a>(&self, node_id: &NodeId) -> Option<NodeGuard<'a>> {
        if let Some(container) = self.nodes.get(node_id) {
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
        render_resource: &RenderResource,
    ) {
        if let Some(mut node) = self.get_node(node_id) {
            node.despawn(plugins, node_id, self, render_resource);

            for child in self.get_children(*node_id).clone() {
                self.despawn_recursive(&child, plugins, render_resource);
            }
        }
    }

    pub fn update(&mut self, plugins: &Plugins, render_resource: &RenderResource) {
        for node_id in self.nodes.keys().cloned().collect::<Vec<_>>() {
            if let Some(mut node) = self.get_node(&node_id) {
                node.update(plugins, &node_id, self, render_resource);
            }
        }
    }

    pub fn editor_update(&mut self, plugins: &Plugins, render_resource: &RenderResource) {
        for node_id in self.nodes.keys().cloned().collect::<Vec<_>>() {
            if let Some(mut node) = self.get_node(&node_id) {
                node.editor_update(plugins, &node_id, self, render_resource);
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
        render_resource: &RenderResource,
        render_pass: &mut EmptyRenderPass<'_, '_, format::TargetFormat, format::Depth32Float>,
    ) {
        for id in self.nodes.keys().cloned().collect::<Vec<_>>() {
            if let Some(mut node) = self.get_node(&id) {
                node.render(plugins, &id, self, render_resource, render_pass);
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
