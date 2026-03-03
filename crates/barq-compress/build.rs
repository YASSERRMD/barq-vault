use std::env;
use std::path::PathBuf;

fn main() {
    let cpp_dir = "src/cpp";

    // Compile all C++ source files in src/cpp/
    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .opt_level(3)
        .flag_if_supported("-march=native")
        .flag_if_supported("-mavx2")
        .flag_if_supported("-ffast-math")
        .include(cpp_dir)
        .files(
            std::fs::read_dir(cpp_dir)
                .expect("src/cpp directory not found")
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|e| e == "cpp").unwrap_or(false)),
        )
        .compile("barq_compress_cpp");

    // Link system compression libraries
    println!("cargo:rustc-link-lib=lzma");
    println!("cargo:rustc-link-lib=lz4");

    // Rerun if any C++ source or header changes
    println!("cargo:rerun-if-changed={}/codec.h", cpp_dir);
    println!("cargo:rerun-if-changed={}/codec_lzma.cpp", cpp_dir);
    println!("cargo:rerun-if-changed={}/codec_lz4.cpp", cpp_dir);

    // Generate Rust bindings from the C header
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings = bindgen::Builder::default()
        .header(format!("{}/codec.h", cpp_dir))
        .clang_arg(format!("-I{}", cpp_dir))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Failed to generate bindings from codec.h");

    bindings
        .write_to_file(out_path.join("codec_bindings.rs"))
        .expect("Failed to write codec_bindings.rs");
}
