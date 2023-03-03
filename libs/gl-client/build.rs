const NOBODY_CRT: &'static str = "../../tls/users-nobody.pem";
const NOBODY_KEY: &'static str = "../../tls/users-nobody-key.pem";

use std::env::var;

fn main() {
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
    println!("cargo:rustc-env=GL_NOBODY_KEY={}", vars.0);
    println!("cargo:rustc-env=GL_NOBODY_CRT={}", vars.1);

    // Setting a custom certificate causes rebuilds of this crate
    println!("cargo:rerun-if-env-changed=GL_CUSTOM_NOBODY_CERT");
    println!("cargo:rerun-if-env-changed=GL_CUSTOM_NOBODY_KEY");

    let builder = tonic_build::configure();

    builder
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &[
                "../proto/greenlight.proto",
                "../proto/scheduler.proto",
                "../proto/node.proto",
            ],
            &["../proto"],
        )
        .unwrap();
}
