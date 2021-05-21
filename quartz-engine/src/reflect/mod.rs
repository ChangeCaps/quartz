pub(crate) mod serde;

pub trait Reflect: erased_serde::Serialize {
    fn reflect(&mut self, deserializer: &mut dyn erased_serde::Deserializer);
    fn as_serialize(&self) -> &dyn erased_serde::Serialize;
}
