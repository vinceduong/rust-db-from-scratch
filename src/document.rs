use serde::de::DeserializeOwned;
use serde::Serialize;
use std::hash::Hash;

pub trait HasId {
    type Id: PartialEq + Copy + Hash + Eq;
    fn id(&self) -> Self::Id;
}

pub trait Document: Serialize + DeserializeOwned + HasId + std::fmt::Debug + Clone {}

impl<T: Serialize + DeserializeOwned + HasId + std::fmt::Debug + Clone> Document for T {}

pub type Filter<T> = fn(d: &T) -> bool;
