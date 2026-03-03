use std::env;
use std::path::PathBuf;

fn main() {
    let cpp_dir = "src/cpp";

    // Detect Homebrew prefix for macOS header resolution
    let homebrew_prefix = std::process::Command::new("brew")
        .arg("--prefix")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "/opt/homebrew".to_string());

    let xz_include = format!("{}/opt/xz/include", homebrew_prefix);
    let lz4_include = format!("{}/opt/lz4/include", homebrew_prefix);
    let xz_lib = format!("{}/opt/xz/lib", homebrew_prefix);
    let lz4_lib = format!("{}/opt/lz4/lib", homebrew_prefix);

    // Compile all C++ source files in src/cpp/
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .opt_level(3)
        .flag_if_supported("-ffast-math")
        .include(cpp_dir)
        .include(&xz_include)
        .include(&lz4_include);

    // AVX2 only on x86_64
    if cfg!(target_arch = "x86_64") {
        build.flag_if_supported("-mavx2");
    }

    let cpp_files: Vec<_> = std::fs::read_dir(cpp_dir)
        .expect("src/cpp directory not found")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "cpp").unwrap_or(false))
        .collect();

    for f in &cpp_files {
        build.file(f);
        println!(
            "cargo:rerun-if-changed={}",
            f.display()
        );
    }
    println!("cargo:rerun-if-changed={}/codec.h", cpp_dir);

    build.compile("barq_compress_cpp");

    // Link system compression libraries
    println!("cargo:rustc-link-search=native={}", xz_lib);
    println!("cargo:rustc-link-search=native={}", lz4_lib);
    println!("cargo:rustc-link-lib=lzma");
    println!("cargo:rustc-link-lib=lz4");

    // Generate Rust bindings from the C header
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings = bindgen::Builder::default()
        .header(format!("{}/codec.h", cpp_dir))
        .clang_arg(format!("-I{}", cpp_dir))
        .clang_arg(format!("-I{}", xz_include))
        .clang_arg(format!("-I{}", lz4_include))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Failed to generate bindings from codec.h");

    bindings
        .write_to_file(out_path.join("codec_bindings.rs"))
        .expect("Failed to write codec_bindings.rs");
}
