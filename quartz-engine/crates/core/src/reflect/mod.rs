pub(crate) mod serde;
pub use quartz_engine_derive::Reflect;

pub trait Reflect: erased_serde::Serialize {
    fn reflect<'de>(&mut self, deserializer: &mut dyn erased_serde::Deserializer<'de>);
    fn as_serialize(&self) -> &dyn erased_serde::Serialize;

    fn short_name_const() -> &'static str
    where
        Self: Sized;
    fn long_name_const() -> &'static str
    where
        Self: Sized,
    {
        std::any::type_name::<Self>()
    }
}
