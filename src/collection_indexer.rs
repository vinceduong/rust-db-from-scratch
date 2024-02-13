use std::collections::HashMap;

use crate::{
    collection_file::{CollectionFile, ReadPageError},
    document::{Document, HasId},
};

fn index_collection_hash<T: Document>(
    collection_file: CollectionFile<T>,
) -> Result<HashMap<<T as HasId>::Id, u64>, ReadPageError> {
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
    use crate::document::HasId;
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
    fn test_collection_hash() {
        let dir = tempdir().unwrap();
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();

        let mut collection_file = CollectionFile::<MyDocument>::new("test", dir_name).unwrap();

        let mut collection_page = collection_file.read_page(0).unwrap();

        collection_page
            .insert_document(MyDocument { id: 1 })
            .unwrap();
        collection_file.write_page(&collection_page).unwrap();

        let hash = index_collection_hash(collection_file).unwrap();

        let mut expected_hash_map = HashMap::new();
        expected_hash_map.insert(1, 0);

        assert_eq!(hash, expected_hash_map)
    }
}
