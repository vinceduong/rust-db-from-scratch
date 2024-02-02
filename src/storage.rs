use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn write_to_collection(dir: &str, document: &str, collection: &str) {
    let binding = format!("{}/{}.collection", dir, collection);
    let path = Path::new(&binding);
    let display = path.display();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .unwrap_or_else(|why| panic!("Couldn't open {}: {}", display, why));

    let document_line = format!("{}\n", document);

    match file.write_all(document_line.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use tempfile::tempdir;

    fn read_file_content(path: &Path) -> String {
        let mut file = File::open(path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        content
    }

    #[test]
    fn test_write_to_new_collection() {
        let dir = tempdir().unwrap();
        let collection_name = "new_collection";
        let document = "New document content";
        let collection_path = dir.path().join(format!("{}.collection", collection_name));

        // Change the path in the actual function or use any method to redirect output to the temp directory
        // For the sake of this example, assume the function writes to the temp directory
        write_to_collection(dir.into_path().to_str().unwrap(), document, collection_name);

        assert!(collection_path.exists());
        let content = read_file_content(&collection_path);
        assert_eq!(content, format!("{}\n", document));
    }

    #[test]
    fn test_append_to_existing_collection() {
        let dir = tempdir().unwrap();
        let collection_name = "existing_collection";
        let initial_document = "Initial content";
        let appended_document = "Appended content";
        let collection_path = dir.path().join(format!("{}.collection", collection_name));
        let binding = dir.into_path();
        let dir_name = binding.to_str().unwrap();

        // Write initial content
        write_to_collection(dir_name, initial_document, collection_name);
        // Append new content
        write_to_collection(dir_name, appended_document, collection_name);

        assert!(collection_path.exists());
        let content = read_file_content(&collection_path);
        assert_eq!(
            content,
            format!("{}\n{}\n", initial_document, appended_document)
        );
    }

    // Add more tests here for error handling, etc.
}
