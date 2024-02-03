use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn write_to_collection(dir: &str, document: Vec<u8>, collection: &str) {
    let binding = format!("{}/{}.collection", dir, collection);
    let path = Path::new(&binding);
    let display = path.display();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .unwrap_or_else(|why| panic!("Couldn't open {}: {}", display, why));

    file.write_all(&document).expect("Failed to write data")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use tempfile::tempdir;

    fn read_file_content(path: &Path) -> Vec<u8> {
        let mut file = File::open(path).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        println!("CONTENT: {:?}", content);
        content
    }

    #[test]
    fn test_write_to_new_collection() {
        let dir = tempdir().unwrap();
        let collection_name = "new_collection";
        let document = vec![1, 2, 3, 4, 5];
        let collection_path = dir.path().join(format!("{}.collection", collection_name));

        // Change the path in the actual function or use any method to redirect output to the temp directory
        // For the sake of this example, assume the function writes to the temp directory
        write_to_collection(
            dir.into_path().to_str().unwrap(),
            document.clone(),
            collection_name,
        );

        assert!(collection_path.exists());
        let content = read_file_content(&collection_path);
        assert_eq!(content, document);
    }

    #[test]
    fn test_append_to_existing_collection() {
        let dir = tempdir().unwrap();
        let collection_name = "existing_collection";
        let initial_document = vec![1, 2, 3, 4];
        let appended_document = vec![1, 2, 3, 4];

        let collection_path = dir.path().join(format!("{}.collection", collection_name));
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();
        let merged_vec: Vec<u8> =
            [initial_document.as_slice(), appended_document.as_slice()].concat();
        // Write initial content
        write_to_collection(dir_name, initial_document, collection_name);
        // Append new content
        write_to_collection(dir_name, appended_document, collection_name);

        assert!(collection_path.exists());
        let content = read_file_content(&collection_path);
        assert_eq!(content, merged_vec);
    }

    // Add more tests here for error handling, etc.
}
