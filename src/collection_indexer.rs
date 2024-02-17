use std::collections::HashMap;

use crate::{
    collection_file::{CollectionFile, ReadPageError},
    document::{Document, HasId},
};

pub type IdToPageMap<T> = HashMap<<T as HasId>::Id, u64>;

pub fn index_collection_id<T: Document>(
    collection_file: &CollectionFile<T>,
) -> Result<IdToPageMap<T>, ReadPageError> {
    let mut collection_index = HashMap::<<T>::Id, u64>::new();
    println!("{:?}", collection_file);

    for i in 0..collection_file.number_of_pages() {
        let page = collection_file.read_page(i)?;
        println!("{:?}", page);

        let documents = page.documents();
        println!("{:?}", documents);

        for document in documents.iter() {
            collection_index.insert(document.id(), i);
        }
    }

    Ok(collection_index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{collection_page::CollectionPage, document::HasId};
    use serde_derive::{Deserialize, Serialize};
    use tempfile::tempdir;

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

    #[test]
    fn test_collection_hash_one_document() {
        let dir = tempdir().unwrap();
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();

        let mut collection_file = CollectionFile::<MyDocument>::new("test", dir_name).unwrap();

        let mut collection_page = collection_file.read_page(0).unwrap();

        collection_page
            .insert_document(MyDocument { id: 1 })
            .unwrap();
        collection_file.write_page(&collection_page).unwrap();

        let index_hash_map = index_collection_id(&collection_file).unwrap();

        let mut expected_hash_map = HashMap::new();
        expected_hash_map.insert(1, 0);

        assert_eq!(index_hash_map, expected_hash_map)
    }

    #[test]
    fn test_collection_hash_two_document() {
        let dir = tempdir().unwrap();
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();

        let mut collection_file = CollectionFile::<MyDocument>::new("test", dir_name).unwrap();

        let mut collection_page = collection_file.read_page(0).unwrap();

        collection_page
            .insert_document(MyDocument { id: 1 })
            .unwrap();
        collection_page
            .insert_document(MyDocument { id: 2 })
            .unwrap();
        collection_file.write_page(&collection_page).unwrap();

        let index_hash_map = index_collection_id(&collection_file).unwrap();

        let mut expected_hash_map = HashMap::new();
        expected_hash_map.insert(1, 0);
        expected_hash_map.insert(2, 0);

        assert_eq!(index_hash_map, expected_hash_map)
    }

    #[test]
    fn test_collection_hash_two_document_in_two_pages() {
        let dir = tempdir().unwrap();
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();

        let mut collection_file = CollectionFile::<MyDocument>::new("test", dir_name).unwrap();

        let mut collection_page_0 = collection_file.read_page(0).unwrap();
        let mut collection_page_1 = CollectionPage::<MyDocument>::new(1);

        collection_page_0
            .insert_document(MyDocument { id: 1 })
            .unwrap();
        collection_page_1
            .insert_document(MyDocument { id: 2 })
            .unwrap();
        collection_file.write_page(&collection_page_0).unwrap();
        collection_file.write_page(&collection_page_1).unwrap();

        let index_hash_map = index_collection_id(&collection_file).unwrap();

        let mut expected_hash_map = HashMap::new();
        expected_hash_map.insert(1, 0);
        expected_hash_map.insert(2, 1);

        assert_eq!(index_hash_map, expected_hash_map)
    }
}
