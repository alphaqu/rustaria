extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks, GlobalGenerator};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let path = Path::new(&"C:\\Program Files (x86)\\inkscape\\gl-rs-bindings\\bindings.rs");
    if !path.exists() {
        let mut file = File::create(&path).unwrap();
        Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [])
            .write_bindings(GlobalGenerator, &mut file)
            .unwrap();
    }
}