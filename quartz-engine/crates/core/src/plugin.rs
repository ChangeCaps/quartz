use crate::component::*;
use crate::tree::*;
use quartz_render::prelude::*;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

struct PluginContainer {
    taken: AtomicBool,
    plugin: UnsafeCell<Box<dyn Plugin>>,
}

impl PluginContainer {
    pub fn new(plugin: Box<dyn Plugin>) -> Self {
        Self {
            taken: AtomicBool::new(false),
            plugin: UnsafeCell::new(plugin),
        }
    }

    pub fn get_mut(&mut self) -> &mut dyn Plugin {
        self.plugin.get_mut().as_mut()
    }

    pub fn _get(&self) -> Option<&dyn Plugin> {
        if self.taken.load(Ordering::SeqCst) {
            None
        } else {
            // SAFETY: only accessed when not taken
            Some(unsafe { &*self.plugin.get() }.as_ref())
        }
    }

    pub fn take<'a>(&'a self) -> Option<&'a mut dyn Plugin> {
        if self.taken.swap(true, Ordering::SeqCst) {
            None
        } else {
            // SAFETY: plugin can only be accessed when taken is false
            let plugin = unsafe { &mut *self.plugin.get() };
            Some(plugin.as_mut())
        }
    }

    pub unsafe fn put(&self) {
        self.taken.store(false, Ordering::SeqCst);
    }
}

pub struct Plugins {
    plugins: HashMap<TypeId, PluginContainer>,
}

impl Plugins {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn start(&mut self, ctx: PluginCtx) {
        for plugin in self.plugins.values_mut() {
            let ctx = PluginCtx {
                tree: ctx.tree,
                render_resource: ctx.render_resource,
            };

            plugin.get_mut().start(ctx);
        }
    }

    pub fn editor_start(&mut self, ctx: PluginCtx) {
        for plugin in self.plugins.values_mut() {
            let ctx = PluginCtx {
                tree: ctx.tree,
                render_resource: ctx.render_resource,
            };

            plugin.get_mut().editor_start(ctx);
        }
    }

    pub fn update(&mut self, ctx: PluginCtx) {
        for plugin in self.plugins.values_mut() {
            let ctx = PluginCtx {
                tree: ctx.tree,
                render_resource: ctx.render_resource,
            };

            plugin.get_mut().update(ctx);
        }
    }

    pub fn editor_update(&mut self, ctx: PluginCtx) {
        for plugin in self.plugins.values_mut() {
            let ctx = PluginCtx {
                tree: ctx.tree,
                render_resource: ctx.render_resource,
            };

            plugin.get_mut().editor_update(ctx);
        }
    }

    pub fn init<C: InitComponent>(&self) -> C {
        C::Plugins::fetch(self, |plugins| C::init(plugins))
    }

    pub fn register_plugin<P: Plugin>(&mut self, init_ctx: PluginInitCtx) {
        let id = TypeId::of::<P>();
        self.plugins
            .insert(id, PluginContainer::new(Box::new(P::init(init_ctx))));
    }

    pub fn take<'a, P: Plugin>(&'a self) -> Option<&'a mut P> {
        if let Some(plugin) = self.plugins.get(&TypeId::of::<P>()) {
            <dyn Any>::downcast_mut(plugin.take().unwrap().as_any_mut())
        } else {
            None
        }
    }

    pub unsafe fn put<P: Plugin>(&self) {
        if let Some(plugin) = self.plugins.get(&TypeId::of::<P>()) {
            plugin.put();
        }
    }
}

pub trait PluginAny: Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl<T: Any> PluginAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct PluginInitCtx<'a> {
    pub render_resource: &'a RenderResource,
}

pub struct PluginCtx<'a> {
    pub tree: &'a mut Tree,
    pub render_resource: &'a RenderResource,
}

#[allow(unused_variables)]
pub trait Plugin: PluginAny {
    fn init(ctx: PluginInitCtx) -> Self
    where
        Self: Sized;

    fn start(&mut self, ctx: PluginCtx) {}

    fn editor_start(&mut self, ctx: PluginCtx) {}

    fn update(&mut self, ctx: PluginCtx) {}

    fn editor_update(&mut self, ctx: PluginCtx) {}
}

pub trait PluginFetch<'a> {
    type Item;

    fn fetch<O>(plugins: &'a Plugins, f: impl FnOnce(Self::Item) -> O) -> O;
}

impl<'a> PluginFetch<'a> for () {
    type Item = ();

    #[inline(always)]
    fn fetch<O>(_plugins: &'a Plugins, f: impl FnOnce(Self::Item) -> O) -> O {
        f(())
    }
}

macro_rules! impl_fetch {
    ($($ident:ident),+) => {
        #[allow(unused_parens)]
        impl<'a, $($ident),+> PluginFetch<'a> for ($($ident),+)
        where $($ident: Plugin),+
        {
            #[allow(unused_parens)]
            type Item = ($(&'a mut $ident),+);

            #[inline(always)]
            fn fetch<O>(plugins: &'a Plugins, f: impl FnOnce(Self::Item) -> O) -> O {
                $(
                    #[allow(non_snake_case)]
                    let $ident = plugins.take::<$ident>().unwrap();
                )+

                let out = f(($($ident),+));

                $(
                    unsafe { plugins.put::<$ident>(); }
                )+

                out
            }
        }
    };
}

impl_fetch!(A);
impl_fetch!(A, B);
impl_fetch!(A, B, C);
impl_fetch!(A, B, C, D);
impl_fetch!(A, B, C, D, E);
impl_fetch!(A, B, C, D, E, F);
impl_fetch!(A, B, C, D, E, F, G);
impl_fetch!(A, B, C, D, E, F, G, H);
impl_fetch!(A, B, C, D, E, F, G, H, I);
impl_fetch!(A, B, C, D, E, F, G, H, I, J);
impl_fetch!(A, B, C, D, E, F, G, H, I, J, K);
impl_fetch!(A, B, C, D, E, F, G, H, I, J, K, L);
