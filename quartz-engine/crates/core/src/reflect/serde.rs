use crate::component::*;
use crate::node::*;
use crate::plugin::*;
use crate::tree::*;
use quartz_render::transform::Transform;
use serde::{
    de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor},
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};
use std::collections::HashMap;

impl Serialize for Node {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Node", 3)?;

        state.serialize_field("name", &self.name)?;
        state.serialize_field("transform", &self.transform)?;
        state.serialize_field("component", &self.component)?;

        state.end()
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

impl<'a> Serialize for dyn ComponentPod {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Component", 2)?;

        state.serialize_field("type", self.name())?;
        state.serialize_field("component", self.as_serialize())?;

        state.end()
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
    type Value = HashMap<NodeId, NodeContainer>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        struct NodesVisitor<'a> {
            components: &'a Components,
            plugins: &'a Plugins,
        }

        impl<'a, 'de> Visitor<'de> for NodesVisitor<'a> {
            type Value = HashMap<NodeId, NodeContainer>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("NodeId: Node")?;

                Ok(())
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut nodes = HashMap::with_capacity(map.size_hint().unwrap_or(0));

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

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut name = None;
                let mut transform = None;
                let mut component = None;

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
                            if component.is_some() {
                                return Err(de::Error::duplicate_field("component"));
                            }

                            component = Some(map.next_value_seed(ComponentPodDeserializer {
                                components: self.components,
                                plugins: self.plugins,
                            })?);
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let transform = transform.ok_or_else(|| de::Error::missing_field("transform"))?;
                let component = component.ok_or_else(|| de::Error::missing_field("component"))?;

                Ok(Node {
                    name,
                    transform,
                    global_transform: Transform::IDENTITY,
                    component,
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

pub struct ComponentPodDeserializer<'a> {
    plugins: &'a Plugins,
    components: &'a Components,
}

impl<'a, 'de> DeserializeSeed<'de> for ComponentPodDeserializer<'a> {
    type Value = Box<dyn ComponentPod>;

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

        struct ComponentVisitor<'a> {
            plugins: &'a Plugins,
            components: &'a Components,
        }

        impl<'a, 'de> Visitor<'de> for ComponentVisitor<'a> {
            type Value = Box<dyn ComponentPod>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Component")?;

                Ok(())
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut ty = None;
                let mut component = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Type => {
                            if ty.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }

                            ty = Some(map.next_value()?);
                        }
                        Field::Component => {
                            if let Some(ty) = ty {
                                component = Some(map.next_value_seed(ComponentDeserializer {
                                    name: ty,
                                    plugins: self.plugins,
                                    components: self.components,
                                })?);
                            } else {
                                return Err(de::Error::missing_field("type"));
                            }
                        }
                    }
                }

                let component = component.ok_or_else(|| de::Error::missing_field("component"))?;

                Ok(component)
            }
        }

        const FIELDS: &[&str] = &["type", "component"];
        deserializer.deserialize_struct(
            "Component",
            FIELDS,
            ComponentVisitor {
                plugins: self.plugins,
                components: self.components,
            },
        )
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
        println!("loading component: {}", self.name);
        let mut component = self.components.init(self.name, self.plugins).unwrap();

        component.reflect(&mut <dyn erased_serde::Deserializer>::erase(deserializer));

        Ok(component)
    }
}
