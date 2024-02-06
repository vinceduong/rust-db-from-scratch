use bincode::ErrorKind;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::os::unix::prelude::FileExt;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct MyStruct {
    field1: u32,
    field2: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct CollectionPageHeader {
    page_number: u64,
    number_of_documents: u64,
    free_space_offset: u64,
    free_space_available: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DocumentPointer {
    offset: u16,
    size: u8,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct CollectionPage {
    header: CollectionPageHeader,
    document_pointers: Vec<DocumentPointer>,
    data: Vec<u8>,
}

struct Collection {
    number_of_pages: u64,
    file: File,
}

#[derive(Debug)]
enum ReadPageError {
    PageNumberTooHighError,
    IoError(std::io::Error),
    DeserializeError(Box<ErrorKind>),
}

#[derive(Debug)]
enum WritePageError {
    PageNumberTooHighError,
    IoError(std::io::Error),
    SerializeError(Box<ErrorKind>),
}

impl Collection {
    fn new(name: &str, dir: &str) -> Result<Self, Box<dyn Error>> {
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

        Ok(Collection {
            number_of_pages: page_number,
            file,
        })
    }

    fn read_page(self: &Self, page_number: u64) -> Result<CollectionPage, ReadPageError> {
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

    fn write_page(&mut self, page: &CollectionPage) -> Result<(), WritePageError> {
        if page.header.page_number > self.number_of_pages + 1 {
            return Err(WritePageError::PageNumberTooHighError);
        }

        if page.header.page_number == self.number_of_pages + 1 {
            self.number_of_pages += 1;
        }

        let offset = COLLECTION_PAGE_SIZE * page.header.page_number;

        let binary = bincode::serialize(page).map_err(|e| WritePageError::SerializeError(e))?;

        self.file
            .write_all_at(&binary, offset)
            .map_err(|e| WritePageError::IoError(e))?;
        Ok(())
    }
}

const COLLECTION_PAGE_SIZE: u64 = 64_000;

fn main() {
    let collection_page_0 = CollectionPage {
        header: CollectionPageHeader {
            page_number: 0,
            number_of_documents: 0,
            free_space_offset: 0,
            free_space_available: 45000,
        },
        document_pointers: vec![],
        data: vec![],
    };

    let collection_page_1 = CollectionPage {
        header: CollectionPageHeader {
            page_number: 1,
            number_of_documents: 0,
            free_space_offset: 0,
            free_space_available: 45000,
        },
        document_pointers: vec![],
        data: vec![],
    };

    let mut collection = Collection::new("collection", "./data").unwrap();
    collection.write_page(&collection_page_0).unwrap();
    collection.write_page(&collection_page_1).unwrap();

    let collection_page_from_file_0 = collection.read_page(0).unwrap();
    let collection_page_from_file_1 = collection.read_page(1).unwrap();

    println!("{:?}", collection_page_from_file_0);
    println!("{:?}", collection_page_from_file_1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_and_read_from_collection() {
        let dir = tempdir().unwrap();
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();
        let mut collection = Collection::new("collection", dir_name).unwrap();

        let collection_page = CollectionPage {
            header: CollectionPageHeader {
                page_number: 0,
                number_of_documents: 0,
                free_space_offset: 0,
                free_space_available: 45000,
            },
            document_pointers: vec![DocumentPointer { offset: 0, size: 7 }],
            data: vec![1, 2, 3, 4, 5, 7],
        };

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
        let mut collection = Collection::new("collection", binding.to_str().unwrap()).unwrap();

        let collection_page_0 = CollectionPage {
            header: CollectionPageHeader {
                page_number: 0,
                number_of_documents: 0,
                free_space_offset: 0,
                free_space_available: 45000,
            },
            document_pointers: vec![],
            data: vec![],
        };

        let collection_page_1 = CollectionPage {
            header: CollectionPageHeader {
                page_number: 1,
                number_of_documents: 0,
                free_space_offset: 0,
                free_space_available: 45000,
            },
            document_pointers: vec![],
            data: vec![],
        };

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
        let mut collection = Collection::new("collection", binding.to_str().unwrap()).unwrap();

        let collection_page_0 = CollectionPage {
            header: CollectionPageHeader {
                page_number: 0,
                number_of_documents: 0,
                free_space_offset: 0,
                free_space_available: 45000,
            },
            document_pointers: vec![],
            data: vec![],
        };

        let collection_page_1 = CollectionPage {
            header: CollectionPageHeader {
                page_number: 1,
                number_of_documents: 0,
                free_space_offset: 0,
                free_space_available: 45000,
            },
            document_pointers: vec![],
            data: vec![],
        };

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

        let collection_page_0_updated = CollectionPage {
            header: CollectionPageHeader {
                page_number: 0,
                number_of_documents: 1,
                free_space_offset: 2,
                free_space_available: 45000,
            },
            document_pointers: vec![],
            data: vec![],
        };

        collection.write_page(&collection_page_0_updated).unwrap();

        let collection_page_from_file_0_updated = collection
            .read_page(0)
            .unwrap_or_else(|why| panic!("{:?}", why));

        assert_eq!(
            collection_page_0_updated,
            collection_page_from_file_0_updated
        );
    }
}
