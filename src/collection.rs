use core::panic;

use crate::{
    collection_file::CollectionFile,
    collection_indexer::{index_collection_id, IdToPageMap},
    collection_page::CollectionPage,
    document::{Document, Filter, HasId},
    COLLECTION_PAGE_DATA_SIZE,
};

struct Collection<T: Document> {
    id_to_page_map: IdToPageMap<T>,
    collection_file: CollectionFile<T>,
}

impl<T: Document> Collection<T> {
    fn new(name: &str, dir: &str) -> Collection<T> {
        let collection_file = CollectionFile::new(name, dir).unwrap();
        let collection_id_idx = index_collection_id(&collection_file).unwrap();

        Collection {
            id_to_page_map: collection_id_idx,
            collection_file,
        }
    }

    fn write_document_to_page(&mut self, doc: T, collection_page: &mut CollectionPage<T>) {
        let doc_id = doc.id();
        collection_page.insert_document(doc).unwrap();

        self.collection_file.write_page(&collection_page).unwrap();
        self.id_to_page_map.insert(doc_id, 0);
    }

    fn get_first_page_with_enough_space(&self, doc_size: u64) -> CollectionPage<T> {
        let number_of_pages = self.collection_file.number_of_pages();

        if number_of_pages == 0 {
            return CollectionPage::<T>::new(0);
        }

        for i in 0..number_of_pages {
            let collection_page_header = self.collection_file.read_page_header(i).unwrap();

            if collection_page_header.space_available() >= doc_size {
                return self.collection_file.read_page(i).unwrap();
            }
        }

        return CollectionPage::<T>::new(number_of_pages);
    }

    fn insert_one(&mut self, doc: T) {
        let doc_id = doc.id();
        let document_size = bincode::serialized_size(&doc).unwrap();

        if document_size > COLLECTION_PAGE_DATA_SIZE {
            panic!("Document too big")
        }

        let mut page = self.get_first_page_with_enough_space(document_size);

        self.write_document_to_page(doc, &mut page);
    }

    fn find_by_id(&self, id: <T as HasId>::Id) -> Option<T> {
        let page_number = self.id_to_page_map.get(&id)?;

        let page = self.collection_file.read_page(*page_number).ok()?;

        page.find_document(id)
    }

    fn find_by(&self, filter: Filter<T>) -> Vec<T> {
        let mut matching_docs: Vec<T> = vec![];
        let page_number = 0;
        while let Ok(page) = self.collection_file.read_page(page_number) {
            for document in page.documents().iter() {
                if filter(document) {
                    matching_docs.push(document.to_owned());
                }
            }
        }

        matching_docs
    }
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
    fn test_insert_one_find_one_collection() {
        let dir = tempdir().unwrap();
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();
        let mut collection = Collection::<MyDocument>::new("test", dir_name);

        let document: MyDocument = MyDocument { id: 0 };

        collection.insert_one(document);

        let doc_from_collection = collection.find_by_id(0).unwrap();

        assert_eq!(document, doc_from_collection);
    }
}
