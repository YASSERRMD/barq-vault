use std::path::PathBuf;

fn main() {
    let proto_dir = "../../proto";
    let proto_file = format!("{}/barq_vault.proto", proto_dir);

    println!("cargo:rerun-if-changed={}", proto_file);

    let out_dir = PathBuf::from("src/generated");
    std::fs::create_dir_all(&out_dir).expect("Failed to create src/generated directory");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(&out_dir)
        .compile_protos(&[&proto_file], &[proto_dir])
        .expect("Failed to compile barq_vault.proto");
}
