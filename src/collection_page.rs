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
    data: Vec<T>,
}

#[derive(Debug)]
pub enum InsertDocumentError {
    NoFreeSpaceAvailable,
    SerializeError(Box<ErrorKind>),
}

impl<T: Document> CollectionPage<T> {
    pub fn new(page_number: u64) -> CollectionPage<T> {
        CollectionPage {
            header: CollectionPageHeader {
                page_number,
                number_of_documents: 0,
                free_space_available: COLLECTION_PAGE_DATA_SIZE,
            },
            data: vec![],
        }
    }

    pub fn get_page_number(&self) -> u64 {
        self.header.page_number
    }

    pub fn insert_document(&mut self, document: T) -> Result<(), InsertDocumentError> {
        let document_size = bincode::serialized_size(&document)
            .map_err(|e| InsertDocumentError::SerializeError(e))?;

        if self.header.free_space_available < document_size as u64 {
            return Err(InsertDocumentError::NoFreeSpaceAvailable);
        }

        self.data.push(document);

        self.header.free_space_available -= document_size as u64;

        Ok(())
    }

    pub fn find_document(self, id: <T as HasId>::Id) -> Option<T> {
        self.data.into_iter().find(|d| d.id() == id)
    }
}
