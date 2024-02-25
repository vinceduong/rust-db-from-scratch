use crate::collection_page::{CollectionPage, CollectionPageHeader, COLLECTION_PAGE_SIZE};
use crate::document::Document;
use bincode::ErrorKind;
use std::fs::{File, OpenOptions};
use std::marker::PhantomData;
use std::os::unix::prelude::FileExt;
use std::path::Path;

#[derive(Debug)]
pub struct CollectionFile<T: Document> {
    number_of_pages: u64,
    file: File,
    _marker: PhantomData<T>,
}

#[derive(Debug)]
pub enum CollectionFileError {
    PageNumberTooHighError,
    FileError(std::io::Error),
    SerializationError(Box<ErrorKind>),
}

impl From<std::io::Error> for CollectionFileError {
    fn from(err: std::io::Error) -> Self {
        CollectionFileError::FileError(err)
    }
}

impl From<Box<ErrorKind>> for CollectionFileError {
    fn from(err: Box<ErrorKind>) -> Self {
        CollectionFileError::SerializationError(err)
    }
}

impl<T: Document> CollectionFile<T> {
    pub fn new(name: &str, dir: &str) -> Result<Self, CollectionFileError> {
        let binding = format!("{}/{}.collection", dir, name);
        let path = Path::new(&binding);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&path)?;
        let mut page_number: u64 = 0;
        let mut encoded = vec![0u8; 1];

        while let Ok(bytes_read) = file.read_at(&mut encoded, page_number * COLLECTION_PAGE_SIZE) {
            if bytes_read < 1 {
                break;
            }

            page_number += 1;
        }

        let mut collection = CollectionFile {
            number_of_pages: page_number,
            file,
            _marker: PhantomData,
        };

        if page_number == 0 {
            let first_page = CollectionPage::<T>::new(0);
            collection.write_page(&first_page)?;

            collection.number_of_pages = 1;
        }

        Ok(collection)
    }

    pub fn read_page(
        self: &Self,
        page_number: u64,
    ) -> Result<CollectionPage<T>, CollectionFileError> {
        if page_number >= self.number_of_pages {
            return Err(CollectionFileError::PageNumberTooHighError);
        }

        let offset = COLLECTION_PAGE_SIZE * page_number;
        let mut encoded = vec![0u8; COLLECTION_PAGE_SIZE as usize];
        self.file.read_at(&mut encoded, offset)?;

        let collection_page = bincode::deserialize::<CollectionPage<T>>(&encoded[..])?;

        Ok(collection_page)
    }

    pub fn read_page_header(
        self: &Self,
        page_number: u64,
    ) -> Result<CollectionPageHeader, CollectionFileError> {
        if page_number >= self.number_of_pages {
            return Err(CollectionFileError::PageNumberTooHighError);
        }

        let offset = COLLECTION_PAGE_SIZE * page_number;

        let header_size: usize = std::mem::size_of::<CollectionPageHeader>();

        let mut encoded = vec![0u8; header_size];
        self.file.read_at(&mut encoded, offset)?;

        let page_header = bincode::deserialize::<CollectionPageHeader>(&encoded[..])?;

        Ok(page_header)
    }

    pub fn write_page(&mut self, page: &CollectionPage<T>) -> Result<(), CollectionFileError> {
        if page.get_page_number() > self.number_of_pages + 1 {
            return Err(CollectionFileError::PageNumberTooHighError);
        }

        if page.get_page_number() == self.number_of_pages {
            self.number_of_pages += 1;
        }

        let offset = COLLECTION_PAGE_SIZE * page.get_page_number();

        let binary = bincode::serialize(page)?;

        self.file.write_all_at(&binary, offset)?;
        Ok(())
    }

    pub fn number_of_pages(&self) -> u64 {
        self.number_of_pages
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
    fn test_write_and_read_from_collection() {
        let dir = tempdir().unwrap();
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();
        let mut collection = CollectionFile::<MyDocument>::new("collection", dir_name).unwrap();

        let collection_page = CollectionPage::new(0);

        collection.write_page(&collection_page).unwrap();

        let collection_page_from_file = collection
            .read_page(0)
            .unwrap_or_else(|why| panic!("{:?}", why));

        assert_eq!(collection_page, collection_page_from_file);
    }

    #[test]
    fn test_write_and_read_two_pages_from_collection() {
        let dir = tempdir().unwrap();

        let binding = dir.into_path();
        let mut collection =
            CollectionFile::<MyDocument>::new("collection", binding.to_str().unwrap()).unwrap();

        let collection_page_0 = CollectionPage::new(0);

        let collection_page_1 = CollectionPage::new(1);

        collection.write_page(&collection_page_0).unwrap();
        collection.write_page(&collection_page_1).unwrap();

        let collection_page_from_file_0 = collection
            .read_page(0)
            .unwrap_or_else(|why| panic!("{:?}", why));
        let collection_page_from_file_1 = collection
            .read_page(1)
            .unwrap_or_else(|why| panic!("{:?}", why));

        assert_eq!(collection_page_0, collection_page_from_file_0);
        assert_eq!(collection_page_1, collection_page_from_file_1);
    }

    #[test]
    fn test_write_read_update_from_collection() {
        let dir = tempdir().unwrap();

        let binding = dir.into_path();
        let mut collection =
            CollectionFile::<MyDocument>::new("collection", binding.to_str().unwrap()).unwrap();

        let mut collection_page_0 = CollectionPage::new(0);

        let collection_page_1 = CollectionPage::new(1);

        collection.write_page(&collection_page_0).unwrap();
        collection.write_page(&collection_page_1).unwrap();

        let collection_page_from_file_0 = collection
            .read_page(0)
            .unwrap_or_else(|why| panic!("{:?}", why));
        let collection_page_from_file_1 = collection
            .read_page(1)
            .unwrap_or_else(|why| panic!("{:?}", why));

        assert_eq!(collection_page_0, collection_page_from_file_0);
        assert_eq!(collection_page_1, collection_page_from_file_1);

        collection_page_0
            .insert_document(MyDocument { id: 1 })
            .unwrap();

        collection.write_page(&collection_page_0).unwrap();

        let collection_page_from_file_0_updated = collection
            .read_page(0)
            .unwrap_or_else(|why| panic!("{:?}", why));

        assert_eq!(collection_page_0, collection_page_from_file_0_updated);
    }
}
