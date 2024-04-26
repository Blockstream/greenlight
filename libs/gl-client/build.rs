const NOBODY_CRT: &'static str = "../../tls/users-nobody.pem";
const NOBODY_KEY: &'static str = "../../tls/users-nobody-key.pem";

use std::env::var;
use std::path::Path;
use std::process::Command;

fn main() {
    // It's a lot easier to help users if we have the exact version of
    // the Rust bindings that were used.
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // Either both are set or none is :-) We set an env-var for
    // `rustc` to pick up and include using `env!`.
    let vars = match (var("GL_CUSTOM_NOBODY_KEY"), var("GL_CUSTOM_NOBODY_CERT")) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(_), Err(_)) => {
            println!("cargo:warning=Using default NOBODY cert.");
            println!("cargo:warning=Set \"GL_CUSTOM_NOBODY_KEY\" and \"GL_CUSTOM_NOBODY_CERT\" to use a custom cert.");
            (NOBODY_KEY.to_owned(), NOBODY_CRT.to_owned())
        }
        (Ok(_), Err(_)) => {
            println!("Missing GL_CUSTOM_NOBODY_CERT, since you are using GL_CUSTOM_NOBODY_KEY");
            std::process::exit(1);
        }
        (Err(_), Ok(_)) => {
            println!("Missing GL_CUSTOM_NOBODY_KEY, since you are using GL_CUSTOM_NOBODY_CERT");
            std::process::exit(1);
        }
    };

    // This actually sets the GL_NOBODY_KEY and GL_NOBODY_CRT env to the
    // path of the given certs.
    println!("cargo:rustc-env=GL_NOBODY_KEY={}", vars.0);
    println!("cargo:rustc-env=GL_NOBODY_CRT={}", vars.1);

    // We check that these exist before we compile.
    let key_path = Path::new(&vars.0);
    let cert_path = Path::new(&vars.1);

    match (key_path.exists(), cert_path.exists()) {
        (true, true) => (),
        (_, _) => {
            // We could not find either the key or the cert.
            println!(
                "Could not find cert and key files: {:?}, {:?}",
                key_path, cert_path
            );
            std::process::exit(1);
        }
    }

    // Setting a custom certificate causes rebuilds of this crate
    println!("cargo:rerun-if-env-changed=GL_CUSTOM_NOBODY_CERT");
    println!("cargo:rerun-if-env-changed=GL_CUSTOM_NOBODY_KEY");

    let builder = tonic_build::configure();

    builder
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &[
                "../proto/glclient/greenlight.proto",
                "../proto/glclient/scheduler.proto",
                "../proto/node.proto",
            ],
            &["../proto"],
        )
        .unwrap();
}
