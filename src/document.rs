use serde::de::DeserializeOwned;
use serde::Serialize;
use std::hash::Hash;

pub trait HasId {
    type Id: PartialEq + Copy + Hash + Eq;
    fn id(&self) -> Self::Id;
}

pub trait Document: Serialize + DeserializeOwned + HasId + std::fmt::Debug {}

impl<T: Serialize + DeserializeOwned + HasId + std::fmt::Debug> Document for T {}
