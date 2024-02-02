use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

fn main() {
    write_to_collection("lol\n".to_string(), "mdr".to_string());
}

fn write_to_collection(line: String, collection: String) {
    let binding = format!("./data/{}.collection", collection);
    let path = Path::new(&binding);
    let display = path.display();
    println!("{:?}", display);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(&path)
        .unwrap_or_else(|why| panic!("Couldn't open {}: {}", display, why));

    match file.write_all(line.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}
