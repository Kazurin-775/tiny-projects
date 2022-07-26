use std::{fs::File, io::Write};

use flash_lso::types::Lso;

fn main() {
    let file = std::env::args().nth(1).unwrap();
    if file.ends_with(".sol") {
        println!("Converting sol to yaml");

        let data = std::fs::read(&file).unwrap();
        let mut reader = flash_lso::read::Reader::default();
        let contents = reader.parse(&data).unwrap();

        let out_file = File::create(format!("{}.yaml", &file[..file.len() - 4])).unwrap();
        serde_yaml::to_writer(out_file, &contents).unwrap();
    } else if file.ends_with(".yaml") {
        println!("Converting yaml to sol");

        let mut contents: Lso = serde_yaml::from_reader(File::open(&file).unwrap()).unwrap();

        let mut out_file = File::create(format!("{}.sol", &file[..file.len() - 5])).unwrap();
        let data = flash_lso::write::write_to_bytes(&mut contents).unwrap();
        out_file.write_all(&data).unwrap();
    } else {
        panic!("unknown file type");
    }
}
