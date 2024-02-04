use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::os::unix::prelude::FileExt;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct MyStruct {
    field1: u32,
    field2: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct CollectionPageHeader {
    page_number: u32,
    number_of_documents: u32,
    free_space_offset: u16,
    free_space_available: u16,
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

const COLLECTION_PAGE_SIZE: u64 = 64_000;

fn write_page_to_collection(dir: &str, page: &CollectionPage, collection: &str) {
    let binding = format!("{}/{}.collection", dir, collection);
    let path = Path::new(&binding);
    let display = path.display();
    let offset = COLLECTION_PAGE_SIZE * page.header.page_number as u64;

    let binary = bincode::serialize(page).unwrap();
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path)
        .unwrap_or_else(|why| panic!("Couldn't open {}: {}", display, why));

    file.write_all_at(&binary, offset)
        .expect("Failed to write data")
}

fn read_page_from_collection(
    dir: &str,
    collection: &str,
    page_number: u32,
) -> Result<CollectionPage, Box<dyn std::error::Error>> {
    let binding = format!("{}/{}.collection", dir, collection);
    let path = Path::new(&binding);
    let display = path.display();
    let offset = COLLECTION_PAGE_SIZE * page_number as u64;
    let mut encoded = vec![0u8; COLLECTION_PAGE_SIZE as usize];
    let file = OpenOptions::new()
        .read(true)
        .open(&path)
        .unwrap_or_else(|why| panic!("Couldn't open {}: {}", display, why));

    file.read_at(&mut encoded, offset)?;

    let decoded: CollectionPage = bincode::deserialize(&encoded[..])?;

    Ok(decoded)
}

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

    write_page_to_collection("./data/", &collection_page_0, "test_1");
    write_page_to_collection("./data/", &collection_page_1, "test_1");

    let collection_page_from_file_0 =
        read_page_from_collection("./data/", "test_1", 0).unwrap_or_else(|why| panic!("{}", why));
    let collection_page_from_file_1 =
        read_page_from_collection("./data/", "test_1", 1).unwrap_or_else(|why| panic!("{}", why));

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

        write_page_to_collection(dir_name, &collection_page_0, "test_1");
        write_page_to_collection(dir_name, &collection_page_1, "test_1");

        let collection_page_from_file_0 = read_page_from_collection(dir_name, "test_1", 0)
            .unwrap_or_else(|why| panic!("{}", why));
        let collection_page_from_file_1 = read_page_from_collection(dir_name, "test_1", 1)
            .unwrap_or_else(|why| panic!("{}", why));

        assert_eq!(collection_page_0, collection_page_from_file_0);
        assert_eq!(collection_page_1, collection_page_from_file_1);
    }

    #[test]
    fn test_write_read_update_from_collection() {
        let dir = tempdir().unwrap();
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();

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

        write_page_to_collection(dir_name, &collection_page_0, "test_1");
        write_page_to_collection(dir_name, &collection_page_1, "test_1");

        let collection_page_from_file_0 = read_page_from_collection(dir_name, "test_1", 0)
            .unwrap_or_else(|why| panic!("{}", why));
        let collection_page_from_file_1 = read_page_from_collection(dir_name, "test_1", 1)
            .unwrap_or_else(|why| panic!("{}", why));

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

        write_page_to_collection(dir_name, &collection_page_0_updated, "test_1");

        let collection_page_from_file_0_updated = read_page_from_collection(dir_name, "test_1", 0)
            .unwrap_or_else(|why| panic!("{}", why));

        assert_eq!(
            collection_page_0_updated,
            collection_page_from_file_0_updated
        );
    }
}
