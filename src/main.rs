extern crate ole;

use ole::Reader;

fn main() {
    let parser = Reader::from_path("assets/oleObject1.bin").unwrap();
    for entry in parser.iterate() {
        match entry.name() {
            "Equation Native" => {
                let slice = parser.get_entry_slice(entry).unwrap();
                println!("Equation Native: {} bytes.", slice.len());
                return;
            }
            _ => ()
        }
    }
    panic!("No Equation Found!");
}
