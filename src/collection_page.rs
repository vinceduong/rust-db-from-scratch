use crate::document::{Document, HasId};
use bincode::ErrorKind;

use serde::{Deserialize, Serialize};

const COLLECTION_PAGE_DATA_SIZE: u64 = 62_000;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CollectionPageHeader {
    page_number: u64,
    number_of_documents: u64,
    free_space_available: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CollectionPage<T> {
    header: CollectionPageHeader,
    documents: Vec<T>,
}

#[derive(Debug)]
pub enum InsertDocumentError {
    NoFreeSpaceAvailable,
    SerializeError(Box<ErrorKind>),
}

#[derive(Debug)]
pub enum UpdateDocumentError {
    NotFound,
    NoFreeSpaceAvailable,
    SerializeError(Box<ErrorKind>),
}

#[derive(Debug)]
pub enum RemoveDocumentError {
    NotFound,
}

impl<T: Document> CollectionPage<T> {
    pub fn new(page_number: u64) -> CollectionPage<T> {
        CollectionPage {
            header: CollectionPageHeader {
                page_number,
                number_of_documents: 0,
                free_space_available: COLLECTION_PAGE_DATA_SIZE,
            },
            documents: vec![],
        }
    }

    pub fn get_page_number(&self) -> u64 {
        self.header.page_number
    }

    pub fn insert_document(&mut self, document: T) -> Result<(), InsertDocumentError> {
        let document_size = bincode::serialized_size(&document)
            .map_err(|e| InsertDocumentError::SerializeError(e))?;

        println!("Document size: {:?}", document_size);
        println!(
            "Free space available: {:?}",
            self.header.free_space_available
        );

        if self.header.free_space_available < document_size as u64 {
            return Err(InsertDocumentError::NoFreeSpaceAvailable);
        }

        self.documents.push(document);

        self.header.free_space_available -= document_size as u64;
        self.header.number_of_documents += 1;

        Ok(())
    }

    pub fn find_document(&self, id: <T as HasId>::Id) -> Option<T> {
        self.documents.iter().find(|d| d.id() == id).cloned()
    }

    pub fn documents(&self) -> &Vec<T> {
        &self.documents
    }

    pub fn update_document(&mut self, new_doc: T) -> Result<(), UpdateDocumentError> {
        for (index, value) in self.documents.iter().enumerate() {
            if value.id() == new_doc.id() {
                let old_version_size = bincode::serialized_size(&value)
                    .map_err(|e| UpdateDocumentError::SerializeError(e))?;
                let new_vesion_size = bincode::serialized_size(&new_doc)
                    .map_err(|e| UpdateDocumentError::SerializeError(e))?;

                if self.header.free_space_available - old_version_size + new_vesion_size
                    > COLLECTION_PAGE_DATA_SIZE
                {
                    return Err(UpdateDocumentError::NoFreeSpaceAvailable);
                }

                self.header.free_space_available -= old_version_size + new_vesion_size;

                self.documents[index] = new_doc;

                return Ok(());
            }
        }
        return Err(UpdateDocumentError::NotFound);
    }

    pub fn remove_document(&mut self, id: <T as HasId>::Id) -> Result<T, RemoveDocumentError> {
        let index = self
            .documents
            .iter()
            .position(|e| e.id() == id)
            .ok_or_else(|| RemoveDocumentError::NotFound)?;

        Ok(self.documents.swap_remove(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::HasId;
    use serde_derive::{Deserialize, Serialize};

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
    fn insert_one_document() {
        let mut collection_page = CollectionPage::<MyDocument>::new(0);

        collection_page
            .insert_document(MyDocument { id: 1 })
            .unwrap();

        assert_eq!(collection_page.documents, vec![MyDocument { id: 1 }]);
        assert_eq!(collection_page.header.number_of_documents, 1);
        assert_eq!(
            collection_page.header.free_space_available,
            COLLECTION_PAGE_DATA_SIZE - 8
        )
    }

    #[test]
    fn insert_multiple_document() {
        let mut collection_page = CollectionPage::<MyDocument>::new(0);

        collection_page
            .insert_document(MyDocument { id: 1 })
            .unwrap();

        collection_page
            .insert_document(MyDocument { id: 2 })
            .unwrap();

        assert_eq!(
            collection_page.documents,
            vec![MyDocument { id: 1 }, MyDocument { id: 2 }]
        );
        assert_eq!(collection_page.header.number_of_documents, 2);
        assert_eq!(
            collection_page.header.free_space_available,
            COLLECTION_PAGE_DATA_SIZE - 8 * 2
        )
    }

    #[test]
    fn find_one_document() {
        let mut collection_page = CollectionPage::<MyDocument>::new(0);

        collection_page
            .insert_document(MyDocument { id: 1 })
            .unwrap();

        let document = collection_page.find_document(1);
        assert_eq!(document.unwrap(), MyDocument { id: 1 })
    }

    #[test]
    fn do_not_find_document() {
        let mut collection_page = CollectionPage::<MyDocument>::new(0);

        collection_page
            .insert_document(MyDocument { id: 1 })
            .unwrap();

        let document = collection_page.find_document(2);

        assert_eq!(true, document.is_none())
    }

    #[test]
    fn update_one_document() {
        #[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
        struct UserDocument {
            id: u64,
            name: String,
        }

        impl HasId for UserDocument {
            type Id = u64;

            fn id(&self) -> u64 {
                self.id
            }
        }

        let mut collection_page = CollectionPage::<UserDocument>::new(0);
        let user_document = UserDocument {
            id: 1,
            name: "lol".to_string(),
        };

        collection_page.insert_document(user_document).unwrap();

        collection_page
            .update_document(UserDocument {
                id: 1,
                name: "mdr".to_string(),
            })
            .unwrap();

        assert_eq!(
            collection_page.documents,
            vec![UserDocument {
                id: 1,
                name: "mdr".to_string(),
            }]
        )
    }

    #[test]
    fn delete_one_document() {
        #[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
        struct UserDocument {
            id: u64,
            name: String,
        }

        impl HasId for UserDocument {
            type Id = u64;

            fn id(&self) -> u64 {
                self.id
            }
        }

        let mut collection_page = CollectionPage::<UserDocument>::new(0);
        let user_document = UserDocument {
            id: 1,
            name: "lol".to_string(),
        };

        collection_page.insert_document(user_document).unwrap();
        collection_page.remove_document(1).unwrap();

        assert_eq!(collection_page.documents, vec![])
    }
}
