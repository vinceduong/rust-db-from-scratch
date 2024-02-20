use crate::{
    collection_file::{CollectionFile, CollectionFileError},
    collection_indexer::{index_collection_id, IdToPageMap},
    collection_page::{CollectionPage, CollectionPageError},
    document::{Document, Filter, HasId},
    COLLECTION_PAGE_DATA_SIZE,
};

struct Collection<T: Document> {
    id_to_page_map: IdToPageMap<T>,
    collection_file: CollectionFile<T>,
}

pub enum CollectionInsertError {
    FileError(CollectionFileError),
    PageError(CollectionPageError),
    DocumentTooBig,
    SerializeError(Box<bincode::ErrorKind>),
}

impl From<CollectionFileError> for CollectionInsertError {
    fn from(err: CollectionFileError) -> Self {
        CollectionInsertError::FileError(err)
    }
}
impl From<CollectionPageError> for CollectionInsertError {
    fn from(err: CollectionPageError) -> Self {
        CollectionInsertError::PageError(err)
    }
}
impl From<Box<bincode::ErrorKind>> for CollectionInsertError {
    fn from(err: Box<bincode::ErrorKind>) -> Self {
        CollectionInsertError::SerializeError(err)
    }
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

    fn write_document_to_page(
        &mut self,
        doc: T,
        collection_page: &mut CollectionPage<T>,
    ) -> Result<(), CollectionInsertError> {
        let doc_id = doc.id();
        collection_page.insert_document(doc)?;

        self.collection_file.write_page(&collection_page)?;
        self.id_to_page_map.insert(doc_id, 0);
        Ok(())
    }

    fn get_first_page_with_enough_space(
        &self,
        doc_size: u64,
    ) -> Result<CollectionPage<T>, CollectionInsertError> {
        let number_of_pages = self.collection_file.number_of_pages();

        if number_of_pages == 0 {
            return Ok(CollectionPage::<T>::new(0));
        }

        for i in 0..number_of_pages {
            let collection_page_header = self.collection_file.read_page_header(i)?;

            if collection_page_header.space_available() >= doc_size {
                return Ok(self.collection_file.read_page(i)?);
            }
        }

        return Ok(CollectionPage::<T>::new(number_of_pages));
    }

    fn insert_one(&mut self, doc: T) -> Result<(), CollectionInsertError> {
        let doc_id = doc.id();
        let document_size = bincode::serialized_size(&doc)?;

        if document_size > COLLECTION_PAGE_DATA_SIZE {
            return Err(CollectionInsertError::DocumentTooBig);
        }

        let mut page = self.get_first_page_with_enough_space(document_size)?;

        self.write_document_to_page(doc, &mut page)?;

        Ok(())
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
