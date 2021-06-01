use crate::component::*;
use crate::node::*;
use crate::plugin::*;
use crate::scene::*;
use crate::transform::*;
use crate::tree::*;
use super::Reflect;
use linked_hash_map::LinkedHashMap;
use serde::{
    de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};
use std::sync::RwLock;

impl<'a> Serialize for Scene<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Scene", 2)?;

        state.serialize_field("plugins", self.plugins)?;
        state.serialize_field("tree", self.tree)?;

        state.end()
    }
}

impl Serialize for Plugins {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.plugins.serialize(serializer)
    }
}

impl Serialize for PluginContainer {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.get().unwrap().serialize(serializer)
    }
}

impl Serialize for dyn Plugin {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_serialize().serialize(serializer)
    }
}

impl Serialize for Node {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Node", 3)?;

        state.serialize_field("name", &self.name)?;
        state.serialize_field("transform", &self.transform)?;
        state.serialize_field("component", &self.components)?;

        state.end()
    }
}

impl Serialize for NodeComponents {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.components.serialize(serializer)
    }
}

impl Serialize for NodeContainer {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.node
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .serialize(serializer)
    }
}

impl Serialize for Tree {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Tree", 4)?;

        state.serialize_field("nodes", &self.nodes)?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("parents", &self.parents)?;
        state.serialize_field("base", &self.base)?;

        state.end()
    }
}

impl Serialize for dyn ComponentPod {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_serialize().serialize(serializer)
    }
}

pub(crate) struct SceneDeserializer<'a> {
    pub components: &'a Components,
    pub plugins: &'a mut Plugins,
}

impl<'a, 'de> DeserializeSeed<'de> for SceneDeserializer<'a> {
    type Value = Tree;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Plugins,
            Tree,
        }

        struct SceneVisitor<'a> {
            components: &'a Components,
            plugins: &'a mut Plugins,
        }

        impl<'a, 'de> Visitor<'de> for SceneVisitor<'a> {
            type Value = Tree;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Scene")?;

                Ok(())
            }

            fn visit_seq<V: SeqAccess<'de>>(self, mut seq: V) -> Result<Tree, V::Error> {
                seq.next_element_seed(TreeDeserializer {
                    plugins: self.plugins,
                    components: self.components,
                })?
                .unwrap();

                Ok(seq
                    .next_element_seed(TreeDeserializer {
                        plugins: self.plugins,
                        components: self.components,
                    })?
                    .unwrap())
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Tree, V::Error> {
                let mut plugins = None;
                let mut tree = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Plugins => {
                            if plugins.is_some() {
                                return Err(de::Error::duplicate_field("plugins"));
                            }

                            plugins = Some(map.next_value_seed(PluginsDeserializer {
                                plugins: self.plugins,
                            })?);
                        }
                        Field::Tree => {
                            if tree.is_some() {
                                return Err(de::Error::duplicate_field("tree"));
                            }

                            tree = Some(map.next_value_seed(TreeDeserializer {
                                plugins: self.plugins,
                                components: self.components,
                            })?);
                        }
                    }
                }

                let _ = plugins.ok_or_else(|| de::Error::missing_field("plugins"))?;
                let tree = tree.ok_or_else(|| de::Error::missing_field("tree"))?;

                Ok(tree)
            }
        }

        const FIELDS: &[&str] = &["plugins", "tree"];
        deserializer.deserialize_struct(
            "Scene",
            FIELDS,
            SceneVisitor {
                components: self.components,
                plugins: self.plugins,
            },
        )
    }
}

pub(crate) struct PluginsDeserializer<'a> {
    pub plugins: &'a mut Plugins,
}

impl<'a, 'de> DeserializeSeed<'de> for PluginsDeserializer<'a> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        struct NodesVisitor<'a> {
            plugins: &'a mut Plugins,
        }

        impl<'a, 'de> Visitor<'de> for NodesVisitor<'a> {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("NodeId: Node")?;

                Ok(())
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                while let Some(key) = map.next_key()? {
                    self.plugins
                        .get_mut_dyn(key, |plugin| {
                            map.next_value_seed(PluginDeserializer { plugin })
                        })
                        .unwrap()?;
                }

                Ok(())
            }
        }

        deserializer.deserialize_map(NodesVisitor {
            plugins: self.plugins,
        })
    }
}

pub(crate) struct PluginDeserializer<'a> {
    pub plugin: &'a mut dyn Plugin,
}

impl<'a, 'de> DeserializeSeed<'de> for PluginDeserializer<'a> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.plugin
            .reflect(&mut <dyn erased_serde::Deserializer>::erase(deserializer));
        Ok(())
    }
}

pub(crate) struct TreeDeserializer<'a> {
    pub components: &'a Components,
    pub plugins: &'a Plugins,
}

impl<'a, 'de> DeserializeSeed<'de> for TreeDeserializer<'a> {
    type Value = Tree;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Nodes,
            Parents,
            Children,
            Base,
        }

        struct TreeVisitor<'a> {
            components: &'a Components,
            plugins: &'a Plugins,
        }

        impl<'a, 'de> Visitor<'de> for TreeVisitor<'a> {
            type Value = Tree;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Tree")?;

                Ok(())
            }

            fn visit_seq<V: SeqAccess<'de>>(self, mut seq: V) -> Result<Tree, V::Error> {
                let nodes = seq
                    .next_element_seed(NodesDeserializer {
                        plugins: self.plugins,
                        components: self.components,
                    })?
                    .ok_or(de::Error::invalid_length(0, &self))?;

                let next_node_id = nodes
                    .keys()
                    .max_by(|a, b| a.0.cmp(&b.0))
                    .map(|id| NodeId(id.0 + 1))
                    .unwrap_or(NodeId(0));

                Ok(Tree {
                    nodes,
                    children: seq
                        .next_element()?
                        .ok_or(de::Error::invalid_length(1, &self))?,
                    parents: seq
                        .next_element()?
                        .ok_or(de::Error::invalid_length(2, &self))?,
                    base: seq
                        .next_element()?
                        .ok_or(de::Error::invalid_length(3, &self))?,
                    next_node_id,
                    despawn: Vec::new(),
                    added: Vec::new(),
                })
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Tree, V::Error> {
                let mut nodes = None;
                let mut parents = None;
                let mut children = None;
                let mut base = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Nodes => {
                            if nodes.is_some() {
                                return Err(de::Error::duplicate_field("base"));
                            }

                            nodes = Some(map.next_value_seed(NodesDeserializer {
                                components: self.components,
                                plugins: self.plugins,
                            })?);
                        }
                        Field::Parents => {
                            if parents.is_some() {
                                return Err(de::Error::duplicate_field("parents"));
                            }

                            parents = Some(map.next_value()?);
                        }
                        Field::Children => {
                            if children.is_some() {
                                return Err(de::Error::duplicate_field("children"));
                            }

                            children = Some(map.next_value()?);
                        }
                        Field::Base => {
                            if base.is_some() {
                                return Err(de::Error::duplicate_field("base"));
                            }

                            base = Some(map.next_value()?);
                        }
                    }
                }

                let nodes = nodes.ok_or_else(|| de::Error::missing_field("nodes"))?;
                let parents = parents.ok_or_else(|| de::Error::missing_field("parents"))?;
                let children = children.ok_or_else(|| de::Error::missing_field("children"))?;
                let base = base.ok_or_else(|| de::Error::missing_field("base"))?;

                let next_node_id = nodes
                    .keys()
                    .max_by(|a, b| a.0.cmp(&b.0))
                    .map(|id| NodeId(id.0 + 1))
                    .unwrap_or(NodeId(0));

                Ok(Tree {
                    nodes,
                    parents,
                    children,
                    base,
                    next_node_id,
                    despawn: Vec::new(),
                    added: Vec::new(),
                })
            }
        }

        const FIELDS: &[&str] = &["nodes", "parents", "children", "base"];
        deserializer.deserialize_struct(
            "Tree",
            FIELDS,
            TreeVisitor {
                components: self.components,
                plugins: self.plugins,
            },
        )
    }
}

pub(crate) struct NodesDeserializer<'a> {
    pub components: &'a Components,
    pub plugins: &'a Plugins,
}

impl<'a, 'de> DeserializeSeed<'de> for NodesDeserializer<'a> {
    type Value = LinkedHashMap<NodeId, NodeContainer>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        struct NodesVisitor<'a> {
            components: &'a Components,
            plugins: &'a Plugins,
        }

        impl<'a, 'de> Visitor<'de> for NodesVisitor<'a> {
            type Value = LinkedHashMap<NodeId, NodeContainer>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("NodeId: Node")?;

                Ok(())
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut nodes = LinkedHashMap::with_capacity(map.size_hint().unwrap_or(0));

                while let Some(key) = map.next_key()? {
                    let value = map.next_value_seed(NodeContainerDeserializer {
                        components: self.components,
                        plugins: self.plugins,
                    })?;

                    nodes.insert(key, value);
                }

                Ok(nodes)
            }
        }

        deserializer.deserialize_map(NodesVisitor {
            components: self.components,
            plugins: self.plugins,
        })
    }
}

pub(crate) struct NodeContainerDeserializer<'a> {
    pub components: &'a Components,
    pub plugins: &'a Plugins,
}

impl<'a, 'de> DeserializeSeed<'de> for NodeContainerDeserializer<'a> {
    type Value = NodeContainer;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let node = NodeDeserializer {
            components: self.components,
            plugins: self.plugins,
        }
        .deserialize(deserializer)?;

        Ok(NodeContainer::new(node))
    }
}

pub(crate) struct NodeDeserializer<'a> {
    plugins: &'a Plugins,
    components: &'a Components,
}

impl<'a, 'de> DeserializeSeed<'de> for NodeDeserializer<'a> {
    type Value = Node;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Name,
            Transform,
            Component,
        }

        struct NodeVisitor<'a> {
            plugins: &'a Plugins,
            components: &'a Components,
        }

        impl<'a, 'de> Visitor<'de> for NodeVisitor<'a> {
            type Value = Node;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Component")?;

                Ok(())
            }

            fn visit_seq<V: SeqAccess<'de>>(self, mut seq: V) -> Result<Self::Value, V::Error> {
                Ok(Node {
                    name: seq
                        .next_element()?
                        .ok_or(de::Error::invalid_length(0, &self))?,
                    transform: seq
                        .next_element()?
                        .ok_or(de::Error::invalid_length(1, &self))?,
                    global_transform: Transform::IDENTITY,
                    components: NodeComponents {
                        components: seq
                            .next_element_seed(ComponentsDeserializer {
                                plugins: self.plugins,
                                components: self.components,
                            })?
                            .ok_or(de::Error::invalid_length(2, &self))?,
                    },
                })
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut name = None;
                let mut transform = None;
                let mut components = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }

                            name = Some(map.next_value()?);
                        }
                        Field::Transform => {
                            if transform.is_some() {
                                return Err(de::Error::duplicate_field("transform"));
                            }

                            transform = Some(map.next_value()?);
                        }
                        Field::Component => {
                            if components.is_some() {
                                return Err(de::Error::duplicate_field("component"));
                            }

                            components = Some(map.next_value_seed(ComponentsDeserializer {
                                components: self.components,
                                plugins: self.plugins,
                            })?);
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let transform = transform.ok_or_else(|| de::Error::missing_field("transform"))?;
                let components = components.ok_or_else(|| de::Error::missing_field("component"))?;

                Ok(Node {
                    name,
                    transform,
                    global_transform: Transform::IDENTITY,
                    components: NodeComponents { components },
                })
            }
        }

        const FIELDS: &[&str] = &["type", "component"];
        deserializer.deserialize_struct(
            "Node",
            FIELDS,
            NodeVisitor {
                components: self.components,
                plugins: self.plugins,
            },
        )
    }
}

pub struct ComponentsDeserializer<'a> {
    plugins: &'a Plugins,
    components: &'a Components,
}

impl<'a, 'de> DeserializeSeed<'de> for ComponentsDeserializer<'a> {
    type Value = LinkedHashMap<String, RwLock<Box<dyn ComponentPod>>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Type,
            Component,
        }

        struct ComponentsVisitor<'a> {
            plugins: &'a Plugins,
            components: &'a Components,
        }

        impl<'a, 'de> Visitor<'de> for ComponentsVisitor<'a> {
            type Value = LinkedHashMap<String, RwLock<Box<dyn ComponentPod>>>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("map of components")?;

                Ok(())
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut components = LinkedHashMap::new();

                while let Some(key) = map.next_key()? {
                    let component = map.next_value_seed(ComponentDeserializer {
                        name: key,
                        plugins: self.plugins,
                        components: self.components,
                    })?;

                    components.insert(key.to_string(), RwLock::new(component));
                }

                Ok(components)
            }
        }

        deserializer.deserialize_map(ComponentsVisitor {
            plugins: self.plugins,
            components: self.components,
        })
    }
}

pub struct ComponentDeserializer<'a> {
    name: &'a str,
    plugins: &'a Plugins,
    components: &'a Components,
}

impl<'a, 'de> DeserializeSeed<'de> for ComponentDeserializer<'a> {
    type Value = Box<dyn ComponentPod>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        log::debug!("loading component: {}", self.name);
        let mut component = self
            .components
            .init_long_name(self.name, self.plugins)
            .unwrap();

        component.reflect(&mut <dyn erased_serde::Deserializer>::erase(deserializer));

        Ok(component)
    }
}

pub struct ReflectDeserializer<'a> {
    pub reflect: &'a mut dyn Reflect,
}

impl<'a, 'de> DeserializeSeed<'de> for ReflectDeserializer<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.reflect.reflect(&mut <dyn erased_serde::Deserializer>::erase(deserializer));

        Ok(())
    }
}