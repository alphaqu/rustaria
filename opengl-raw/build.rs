extern crate gl_generator;

use std::fs::File;
use std::io::Write;
use std::path::Path;

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};

fn main() {
    let path = Path::new(&"./src/raw_gl.rs");
    println!("DOING STUFF");
    if !path.exists() {
        let mut file = File::create(&path).unwrap();

        let mut bindings = Vec::new();
        Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::None, [])
            .write_bindings(GlobalGenerator, &mut bindings)
            .unwrap();


        file.write_all(bindings.as_ref()).unwrap();
    }
}