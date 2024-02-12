use crate::collection_page::CollectionPage;
use crate::document::Document;
use bincode::ErrorKind;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::marker::PhantomData;
use std::os::unix::prelude::FileExt;
use std::path::Path;

const COLLECTION_PAGE_SIZE: u64 = 64_000;

pub struct CollectionFile<T: Document> {
    number_of_pages: u64,
    file: File,
    _marker: PhantomData<T>,
}

#[derive(Debug)]
pub enum ReadPageError {
    PageNumberTooHighError,
    IoError(std::io::Error),
    DeserializeError(Box<ErrorKind>),
}

#[derive(Debug)]
pub enum WritePageError {
    PageNumberTooHighError,
    IoError(std::io::Error),
    SerializeError(Box<ErrorKind>),
}

impl<T: Document> CollectionFile<T> {
    pub fn new(name: &str, dir: &str) -> Result<Self, Box<dyn Error>> {
        let binding = format!("{}/{}.collection", dir, name);
        let path = Path::new(&binding);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&path)
            .map_err(|e| {
                println!("could not open {}: {}", binding, e);
                e
            })?;
        let mut page_number: u64 = 0;
        let mut encoded = vec![0u8; 1];

        while let Ok(bytes_read) = file.read_at(&mut encoded, page_number * COLLECTION_PAGE_SIZE) {
            if bytes_read < 1 {
                break;
            }

            page_number += 1;
        }

        let collection = CollectionFile {
            number_of_pages: page_number,
            file,
            _marker: PhantomData,
        };
        Ok(collection)
    }

    pub fn read_page(self: &Self, page_number: u64) -> Result<CollectionPage<T>, ReadPageError> {
        if page_number > self.number_of_pages {
            return Err(ReadPageError::PageNumberTooHighError);
        }

        let offset = COLLECTION_PAGE_SIZE * page_number;
        let mut encoded = vec![0u8; COLLECTION_PAGE_SIZE as usize];
        self.file
            .read_at(&mut encoded, offset)
            .map_err(|e| ReadPageError::IoError(e))?;

        bincode::deserialize(&encoded[..]).map_err(|e| ReadPageError::DeserializeError(e))
    }

    pub fn write_page(&mut self, page: &CollectionPage<T>) -> Result<(), WritePageError> {
        if page.get_page_number() > self.number_of_pages + 1 {
            return Err(WritePageError::PageNumberTooHighError);
        }

        if page.get_page_number() == self.number_of_pages + 1 {
            self.number_of_pages += 1;
        }

        let offset = COLLECTION_PAGE_SIZE * page.get_page_number();

        let binary = bincode::serialize(page).map_err(|e| WritePageError::SerializeError(e))?;

        self.file
            .write_all_at(&binary, offset)
            .map_err(|e| WritePageError::IoError(e))?;
        Ok(())
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
