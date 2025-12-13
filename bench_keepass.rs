use keepass::db::Group;
use keepass::{Database, DatabaseKey};
use std::fs::File;
use std::path::Path;
use std::time::Instant;

fn main() {
    let path = Path::new("testpass.kdbx");
    let password = "testpasswd"; // The password content, not the file path

    // Read password from file
    let password = std::fs::read_to_string("testpasswd")
        .unwrap()
        .trim()
        .to_string();

    let start = Instant::now();
    let mut file = File::open(path).unwrap();
    let key = DatabaseKey::new().with_password(&password);
    let db = Database::open(&mut file, key).unwrap();
    println!("keepass crate opened DB in: {:?}", start.elapsed());
    let root = &db.root;
    println!("Root type: {:?}", std::any::type_name_of_val(root));
}
