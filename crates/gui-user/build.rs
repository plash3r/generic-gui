use std::{env, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let linker = manifest.join("linker.ld");
    println!("cargo:rerun-if-changed={}", linker.display());
    println!("cargo:rustc-link-arg-bin=generic-gui-user=-T{}", linker.display());
}
