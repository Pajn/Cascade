use std::env;

fn main() {
  let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

  cbindgen::generate(crate_dir)
    .map(|bindings| {
      bindings.write_to_file("rust_wm.h.new");
    })
    .or_else(|err| {
      match err {
        // Let Rust compiler handle parse errors
        cbindgen::Error::ParseSyntaxError {
          crate_name: _,
          src_path: _,
          error: _,
        } => Ok(()),
        _ => Err(err),
      }
    })
    .expect("Unable to generate bindings");
}
