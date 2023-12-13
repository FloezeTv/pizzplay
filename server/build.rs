use std::{env, fs};

use npm_rs::{NodeEnv, NpmEnv};

type Return = Result<(), Box<dyn std::error::Error>>;

/// Prints all reruns directly in the passed `root` directory,
/// while ignoring the files/directories passed in `ignore`.
///
/// Will not recurse into any directories
/// (and therefore not ignore any files in subdirectories),
/// but instead relies on cargo to recursively check modification times.
fn print_reruns<const N: usize>(root: &str, ignore: [&'static str; N]) -> Return {
    for f in fs::read_dir(root)? {
        let f = f?;
        if ignore.iter().all(|i| f.file_name() != *i) {
            println!("cargo:rerun-if-changed={}", f.path().to_str().ok_or("")?,);
        }
    }
    Ok(())
}

/// Automatically builds the client in `../client`.
/// The build directory of the client can be gotten using `env!("CLIENT_BUILD_DIR")`.
fn build_client() -> Return {
    // Paths
    let client_path = env::var("CARGO_MANIFEST_DIR")? + "/../client/";
    let out_path = env::var("OUT_DIR")? + "/client";

    print_reruns(&client_path, ["node_modules"])?;

    let npm = NpmEnv::default()
        .with_node_env(&NodeEnv::from_cargo_profile().unwrap_or_default())
        .set_path(client_path)
        .init_env();

    npm.custom("ci", None)
        .run(format!("build -- --outDir=\"{out_path}\"").as_str())
        .exec()?;

    // Output path can now be gotten at compile-time using `env!("CLIENT_BUILD_DIR")`
    println!("cargo:rustc-env=CLIENT_BUILD_DIR={out_path}");

    Ok(())
}

fn main() -> Return {
    build_client()?;
    Ok(())
}
