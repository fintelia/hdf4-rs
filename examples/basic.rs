extern crate hdf4;

use std::env;
use std::fs::File;
use std::io::Read;

use hdf4::{HdfFile, Tag};

fn main() {
    let args: Vec<_> = env::args().collect();
    assert!(args.len() > 1, "Please specify hdf file");

    let mut contents = Vec::new();
    let mut file = File::open(&args[1]).unwrap();
    file.read_to_end(&mut contents).unwrap();

    let mut hdf4_file = HdfFile::from_slice(&contents[..]).unwrap();
    hdf4_file.remove_nulls();

    for d in &hdf4_file.descriptors {
        if let Tag::ScientificDataDimension { .. } = d.tag {
            println!("{:?}", d.tag);
        }
    }
}
