use serde::{Deserialize, Serialize};
mod storage;

#[derive(Serialize, Deserialize, Debug)]
struct MyStruct {
    field1: u32,
    field2: String,
}

fn main() {
    let lol = MyStruct {
        field1: 1,
        field2: "fdsf".to_string(),
    };

    storage::write_to_collection::write_to_collection(
        "./data",
        bincode::serialize(&lol).unwrap(),
        "user",
    );
}
