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

#[derive(Debug)]
pub enum CollectionError {
    FileError(CollectionFileError),
    PageError(CollectionPageError),
    NotFoundError,
    DocumentTooBig,
    DuplicateError,
    SerializeError(Box<bincode::ErrorKind>),
}

impl From<CollectionFileError> for CollectionError {
    fn from(err: CollectionFileError) -> Self {
        CollectionError::FileError(err)
    }
}
impl From<CollectionPageError> for CollectionError {
    fn from(err: CollectionPageError) -> Self {
        CollectionError::PageError(err)
    }
}
impl From<Box<bincode::ErrorKind>> for CollectionError {
    fn from(err: Box<bincode::ErrorKind>) -> Self {
        CollectionError::SerializeError(err)
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
        doc: &T,
        collection_page: &mut CollectionPage<T>,
    ) -> Result<(), CollectionError> {
        let doc_id = doc.id();
        collection_page.insert_document(&doc)?;

        self.collection_file.write_page(&collection_page)?;
        self.id_to_page_map.insert(doc_id, 0);
        Ok(())
    }

    fn get_first_page_with_enough_space(
        &self,
        doc_size: u64,
    ) -> Result<CollectionPage<T>, CollectionError> {
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

    fn insert_one(&mut self, doc: &T) -> Result<(), CollectionError> {
        let doc_id = doc.id();
        let document_size = bincode::serialized_size(&doc)?;

        if self.id_to_page_map.contains_key(&doc_id) {
            return Err(CollectionError::DuplicateError);
        }

        if document_size > COLLECTION_PAGE_DATA_SIZE {
            return Err(CollectionError::DocumentTooBig);
        }

        let mut page = self.get_first_page_with_enough_space(document_size)?;

        self.write_document_to_page(&doc, &mut page)?;

        Ok(())
    }

    fn find_by_id(&self, id: <T as HasId>::Id) -> Option<T> {
        let page_number = self.id_to_page_map.get(&id)?;

        let page = self.collection_file.read_page(*page_number).ok()?;

        page.find_document(id)
    }

    fn find_by(&self, filter: Filter<T>) -> Vec<T> {
        let mut matching_docs: Vec<T> = vec![];
        let mut page_number = 0;
        while let Ok(page) = self.collection_file.read_page(page_number) {
            for document in page.documents().iter() {
                if filter(document) {
                    matching_docs.push(document.to_owned());
                }
            }
            page_number += 1;
        }

        matching_docs
    }

    fn update_one(&mut self, doc_update: &T) -> Result<(), CollectionError> {
        let doc_id = doc_update.id();
        let page_number = self
            .id_to_page_map
            .get(&doc_id)
            .ok_or(CollectionError::NotFoundError)?;

        let mut page = self.collection_file.read_page(*page_number)?;

        let update = page.update_document(&doc_update);

        match update {
            Ok(_) => Ok(()),
            Err(CollectionPageError::NoFreeSpaceAvailable) => {
                page.remove_document(doc_id)?;
                self.insert_one(doc_update)?;
                Ok(())
            }
            Err(e) => Err(CollectionError::PageError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::HasId;
    use serde_derive::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
    struct MyDocument {
        id: u64,
        name: String,
    }

    impl HasId for MyDocument {
        type Id = u64;

        fn id(&self) -> u64 {
            self.id
        }
    }

    #[test]
    fn test_insert_one_find_one_by_id() {
        let dir = tempdir().unwrap();
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();
        let mut collection = Collection::<MyDocument>::new("test", dir_name);

        let document: MyDocument = MyDocument {
            id: 0,
            name: String::from("test1"),
        };

        collection.insert_one(&document).unwrap();

        let doc_from_collection = collection.find_by_id(0).unwrap();

        assert_eq!(document, doc_from_collection);
    }

    #[test]
    fn test_insert_find_all_collection() {
        let dir = tempdir().unwrap();
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();
        let mut collection = Collection::<MyDocument>::new("test", dir_name);

        let documents: Vec<MyDocument> = vec![
            MyDocument {
                id: 0,
                name: String::from("test1"),
            },
            MyDocument {
                id: 1,
                name: String::from("test2"),
            },
        ];

        for document in &documents {
            collection.insert_one(&document).unwrap();
        }

        let doc_from_collection = collection.find_by(|_| true);

        assert_eq!(documents, doc_from_collection);
    }

    #[test]
    fn test_insert_find_by_collection() {
        let dir = tempdir().unwrap();
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();
        let mut collection = Collection::<MyDocument>::new("test", dir_name);

        let documents: Vec<MyDocument> = vec![
            MyDocument {
                id: 0,
                name: String::from("test1"),
            },
            MyDocument {
                id: 1,
                name: String::from("test2"),
            },
            MyDocument {
                id: 2,
                name: String::from("test3"),
            },
            MyDocument {
                id: 3,
                name: String::from("test4"),
            },
        ];

        for document in &documents {
            collection.insert_one(&document).unwrap();
        }

        let doc_from_collection = collection.find_by(|doc| doc.id() % 2 == 0);

        assert_eq!(
            vec![
                MyDocument {
                    id: 0,
                    name: String::from("test1"),
                },
                MyDocument {
                    id: 2,
                    name: String::from("test3"),
                },
            ],
            doc_from_collection
        );
    }
}
