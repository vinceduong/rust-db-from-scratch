use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait HasId {
    type Id: PartialEq + Copy;
    fn id(&self) -> Self::Id;
}

pub trait Document: Serialize + DeserializeOwned + HasId {}

impl<T: Serialize + DeserializeOwned + HasId> Document for T {}
