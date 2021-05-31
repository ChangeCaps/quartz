use crate::plugin::*;
use crate::tree::*;

pub struct Scene<'a> {
    pub plugins: &'a Plugins,
    pub tree: &'a Tree,
}
