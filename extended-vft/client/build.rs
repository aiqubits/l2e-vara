use sails_client_gen::ClientGenerator;
use std::{env, path::PathBuf};

fn main() {
    let out_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let idl_file_path = out_dir_path.join("vft.idl");

    // Generate IDL file for the program
    sails_idl_gen::generate_idl_to_file::<extended_vft_app::Program>(&idl_file_path).unwrap();

    // Generate client code from IDL file
    ClientGenerator::from_idl_path(&idl_file_path)
        .with_mocks("with_mocks")
        .generate_to(PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/vft_client.rs"))
        .unwrap();
}
