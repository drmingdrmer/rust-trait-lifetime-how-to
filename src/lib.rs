#![feature(impl_trait_in_assoc_type)]

use std::collections::BTreeMap;
use std::sync::Arc;

pub mod data;
pub mod map_api;
pub mod map_api_impl;
pub mod util;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Val(u64);

#[derive(Debug, Default)]
pub struct Level {
    pub kv: BTreeMap<String, Val>,
}

#[derive(Debug, Default, Clone)]
pub struct StaticLevels {
    levels: Vec<Arc<Level>>,
}

#[derive(Debug)]
pub struct Ref<'d> {
    writable: &'d Level,
    frozen: &'d StaticLevels,
}

#[derive(Debug)]
pub struct RefMut<'d> {
    writable: &'d mut Level,
    frozen: &'d StaticLevels,
}

#[derive(Debug)]
pub struct LevelMap {
    writable: Level,
    frozen: StaticLevels,
}
