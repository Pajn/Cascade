use std::env;
use std::path::PathBuf;

fn main() {
  // Tell cargo to tell rustc to link the system miral
  // shared library.
  println!("cargo:rustc-link-lib=miral");

  let bindings = bindgen::Builder::default()
    // The input header we would like to generate
    // bindings for.
    .header("mir.h")
    .enable_cxx_namespaces()
    .opaque_type("const_pointer")
    .opaque_type("std::vector")
    .opaque_type("miral::.*")
    .whitelist_type("wl_..*")
    .whitelist_type("Mir.*")
    .whitelist_type("mir::.*")
    .whitelist_type("miral::.*")
    .whitelist_function("wl_.*")
    .whitelist_function("mir_.*")
    .whitelist_function("miral::.*")
    .no_copy("miral::.*")
    .default_enum_style(bindgen::EnumVariation::ModuleConsts)
    .clang_args(vec!["-x", "c++"])
    .clang_args(vec!["-I", "mir/include/client"])
    .clang_args(vec!["-I", "mir/include/common"])
    .clang_args(vec!["-I", "mir/include/core"])
    .clang_args(vec!["-I", "mir/include/miral"])
    .clang_args(vec!["-I", "mir/include/server"])
    // Finish the builder and generate the bindings.
    .generate()
    // Unwrap the Result and panic on failure.
    .expect("Unable to generate bindings");

  // Write the bindings to the $OUT_DIR/bindings.rs file.
  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("mir.rs"))
    .expect("Couldn't write bindings!");
}
