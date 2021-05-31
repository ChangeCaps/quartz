use crate::component::*;
use crate::reflect::*;
use crate::tree::*;
use egui::Ui;
use quartz_render::prelude::*;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct PluginGuard<'a, P: Plugin> {
    taken: Arc<AtomicBool>,
    plugin: &'a mut P,
}

impl<'a, P: Plugin> std::ops::Deref for PluginGuard<'a, P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        self.plugin
    }
}

impl<'a, P: Plugin> std::ops::DerefMut for PluginGuard<'a, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.plugin
    }
}

impl<'a, P: Plugin> Drop for PluginGuard<'a, P> {
    fn drop(&mut self) {
        self.taken.store(false, Ordering::SeqCst);
    }
}

pub(crate) struct PluginContainer {
    taken: Arc<AtomicBool>,
    plugin: UnsafeCell<Box<dyn Plugin>>,
}

impl PluginContainer {
    pub fn new(plugin: Box<dyn Plugin>) -> Self {
        Self {
            taken: Arc::new(AtomicBool::new(false)),
            plugin: UnsafeCell::new(plugin),
        }
    }

    pub fn get(&self) -> Option<&dyn Plugin> {
        if self.taken.load(Ordering::SeqCst) {
            None
        } else {
            // SAFETY: only accessed when not taken
            Some(unsafe { &*self.plugin.get() }.as_ref())
        }
    }

    pub fn get_mut(&mut self) -> &mut dyn Plugin {
        self.plugin.get_mut().as_mut()
    }

    pub fn lock<'a, P: Plugin>(&'a self) -> Option<PluginGuard<'a, P>> {
        if let Some(plugin) = self.take() {
            if let Some(plugin) = plugin.as_any_mut().downcast_mut() {
                Some(PluginGuard {
                    taken: self.taken.clone(),
                    plugin,
                })
            } else {
                None
            }
        } else {
            None
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
    pub(crate) plugins: HashMap<String, PluginContainer>,
}

impl Plugins {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn start(&self, ctx: PluginCtx) {
        for id in self.plugins.keys() {
            let ctx = PluginCtx {
                tree: ctx.tree,
                plugins: ctx.plugins,
                instance: ctx.instance,
                target_format: ctx.target_format,
            };

            self.get_mut_dyn(id, |plugin| {
                plugin.start(ctx);
            })
            .unwrap();
        }
    }

    pub fn editor_start(&self, ctx: PluginCtx) {
        for id in self.plugins.keys() {
            let ctx = PluginCtx {
                tree: ctx.tree,
                plugins: ctx.plugins,
                instance: ctx.instance,
                target_format: ctx.target_format,
            };

            self.get_mut_dyn(id, |plugin| {
                plugin.editor_start(ctx);
            })
            .unwrap();
        }
    }

    pub fn update(&self, ctx: PluginCtx) {
        for id in self.plugins.keys() {
            let ctx = PluginCtx {
                tree: ctx.tree,
                plugins: ctx.plugins,
                instance: ctx.instance,
                target_format: ctx.target_format,
            };

            self.get_mut_dyn(id, |plugin| {
                plugin.update(ctx);
            })
            .unwrap();
        }
    }

    pub fn editor_update(&self, ctx: PluginCtx) {
        for id in self.plugins.keys() {
            let ctx = PluginCtx {
                tree: ctx.tree,
                plugins: ctx.plugins,
                instance: ctx.instance,
                target_format: ctx.target_format,
            };

            self.get_mut_dyn(id, |plugin| {
                plugin.editor_update(ctx);
            })
            .unwrap();
        }
    }

    pub fn render(&self, ctx: PluginRenderCtx) {
        for id in self.plugins.keys() {
            let ctx = PluginRenderCtx {
                tree: ctx.tree,
                plugins: ctx.plugins,
                instance: ctx.instance,
                render_ctx: ctx.render_ctx,
                target: ctx.target,
            };

            self.get_mut_dyn(id, |plugin| {
                plugin.render(ctx);
            })
            .unwrap();
        }
    }

    pub fn viewport_render(&self, ctx: PluginRenderCtx) {
        for id in self.plugins.keys() {
            let ctx = PluginRenderCtx {
                tree: ctx.tree,
                plugins: ctx.plugins,
                instance: ctx.instance,
                render_ctx: ctx.render_ctx,
                target: ctx.target,
            };

            self.get_mut_dyn(id, |plugin| {
                plugin.viewport_render(ctx);
            })
            .unwrap();
        }
    }

    pub fn get<P: Plugin>(&self) -> Option<&P> {
        let id = P::long_name_const().to_string();

        if let Some(plugin) = self.plugins.get(&id) {
            if let Some(plugin) = plugin.get() {
                Some(plugin.as_any().downcast_ref().unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_mut<P: Plugin>(&self) -> Option<PluginGuard<P>> {
        let id = P::long_name_const();

        if let Some(plugin) = self.plugins.get(id) {
            plugin.lock()
        } else {
            None
        }
    }

    pub fn plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    pub fn get_mut_dyn<O>(&self, id: &str, f: impl FnOnce(&mut dyn Plugin) -> O) -> Result<O, ()> {
        if let Some(plugin) = self.plugins.get(id) {
            if let Some(p) = plugin.take() {
                let out = f(p);

                unsafe { plugin.put() };

                Ok(out)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn init<C: InitComponent>(&self) -> C {
        C::Plugins::fetch(self, |plugins| C::init(plugins))
    }

    pub fn register_plugin<P: Plugin>(&mut self, init_ctx: PluginInitCtx) {
        let id = P::long_name_const().to_string();
        self.plugins
            .insert(id, PluginContainer::new(Box::new(P::init(init_ctx))));
    }

    pub fn take<'a, P: Plugin>(&'a self) -> Option<&'a mut P> {
        let id = P::long_name_const();

        if let Some(plugin) = self.plugins.get(id) {
            <dyn Any>::downcast_mut(plugin.take().unwrap().as_any_mut())
        } else {
            None
        }
    }

    pub unsafe fn put<P: Plugin>(&self) {
        let id = P::long_name_const();

        if let Some(plugin) = self.plugins.get(id) {
            plugin.put();
        }
    }
}

pub trait PluginAny: Any {
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
    pub instance: &'a Instance,
    pub target_format: format::TargetFormat,
}

pub struct PluginCtx<'a> {
    pub tree: &'a mut Tree,
    pub plugins: &'a Plugins,
    pub instance: &'a Instance,
    pub target_format: format::TargetFormat,
}

pub struct PluginRenderCtx<'a, 'b, 'c> {
    pub tree: &'a mut Tree,
    pub plugins: &'a Plugins,
    pub instance: &'a Instance,
    pub render_ctx: &'a mut RenderCtx<'b>,
    pub target: &'a TextureView<'c>,
}

#[allow(unused_variables)]
pub trait Plugin: PluginAny + Reflect + ReflectName {
    fn init(ctx: PluginInitCtx) -> Self
    where
        Self: Sized;

    fn inspector_ui(&mut self, ctx: PluginCtx, ui: &mut Ui) {}

    fn start(&mut self, ctx: PluginCtx) {}

    fn editor_start(&mut self, ctx: PluginCtx) {}

    fn update(&mut self, ctx: PluginCtx) {}

    fn editor_update(&mut self, ctx: PluginCtx) {}

    fn render(&mut self, ctx: PluginRenderCtx) {}

    fn viewport_render(&mut self, ctx: PluginRenderCtx) {
        self.render(ctx);
    }
}

pub trait PluginFetch<'a>: Sized {
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
