use walkdir::WalkDir;

fn main() {
    let proto_list = WalkDir::new("../proto")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().unwrap_or_default() == "proto")
        .map(|e| e.into_path())
        .collect::<Vec<_>>();

    tonic_build::configure()
        .compile_protos(&proto_list, &["../proto"])
        .unwrap();
}
