fn main() {
    // Use vendored protoc so developers don't need it installed
    let protoc_path = protoc_bin_vendored::protoc_bin_path().expect("protoc vendored");
    std::env::set_var("PROTOC", &protoc_path);

    let proto_dir = "proto";
    let mut config = prost_build::Config::new();
    config.enable_type_names();
    let protos: Vec<_> = std::fs::read_dir(proto_dir)
        .unwrap()
        .filter_map(|e| {
            let p = e.ok()?.path();
            if p.extension().and_then(|s| s.to_str()) == Some("proto") {
                Some(p)
            } else {
                None
            }
        })
        .collect();
    if protos.is_empty() {
        println!("cargo:rerun-if-changed={proto_dir}");
        return;
    }
    for p in &protos {
        println!("cargo:rerun-if-changed={}", p.display());
    }
    tonic_build::configure()
        .compile_protos_with_config(config, &protos, &[proto_dir])
        .unwrap();
}
