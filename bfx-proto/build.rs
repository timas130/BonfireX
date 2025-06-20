use std::env;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

fn main() {
    println!("cargo::rerun-if-changed=../proto");

    let proto_list = WalkDir::new("../proto")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().unwrap_or_default() == "proto")
        .map(DirEntry::into_path)
        .collect::<Vec<_>>();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_prost_build::configure()
        .type_attribute(
            ".bfx.markdown",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .file_descriptor_set_path(out_dir.join("file_descriptor_set.bin"))
        .compile_protos(&proto_list, &["../proto".into()])
        .unwrap();
}
