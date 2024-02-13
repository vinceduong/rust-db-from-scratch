use serde::{Deserialize, Serialize};
mod collection_file;
mod collection_page;
mod document;
use collection_file::CollectionFile;
use collection_page::CollectionPage;
use document::HasId;

const COLLECTION_PAGE_DATA_SIZE: u64 = 62_000;

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
struct MyDocument {
    id: u64,
}

impl HasId for MyDocument {
    type Id = u64;

    fn id(&self) -> u64 {
        self.id
    }
}

fn main() {
    let collection_page_0: CollectionPage<MyDocument> = CollectionPage::new(0);
    let collection_page_1: CollectionPage<MyDocument> = CollectionPage::new(1);

    let mut collection = CollectionFile::new("collection", "./data").unwrap();
    collection.write_page(&collection_page_0).unwrap();
    collection.write_page(&collection_page_1).unwrap();

    let collection_page_from_file_0 = collection.read_page(0).unwrap();
    let collection_page_from_file_1 = collection.read_page(1).unwrap();

    println!("{:?}", collection_page_from_file_0);
    println!("{:?}", collection_page_from_file_1);
}
