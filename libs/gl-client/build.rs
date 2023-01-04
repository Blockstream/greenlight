use std::process::Command;

const NOBODY_CRT: &'static str = "../../tls/users-nobody.pem";
const NOBODY_KEY: &'static str = "../../tls/users-nobody-key.pem";

const DEFAULT_NOBODY_CERT: &'static str = "../../tls/default/users-nobody.pem";
const DEFAULT_NOBODY_KEY: &'static str = "../../tls/default/users-nobody-key.pem";

fn main() {
    // Do we have a custom `NOBODY` user?
    let nobody_cert = match option_env!("GL_CUSTOM_NOBODY_CERT") {
        Some(s) => s,
        None => {
            println!("cargo:warning=Using default NOBODY cert. Set \"GL_CUSTOM_NOBODY_CERT\" to use a custom cert.");
            DEFAULT_NOBODY_CERT
        }
    };
    let nobody_key = match option_env!("GL_CUSTOM_NOBODY_KEY") {
        Some(s) => s,
        None => {
            println!("cargo:warning=Using default NOBODY key. Set \"GL_CUSTOM_NOBODY_KEY\" to use a custom key.");
            DEFAULT_NOBODY_KEY
        }
    };

    // Only one of the custom `NOBODY` env vars is set. This can not be intended.
    // Better panic and throw an error!
    if !(nobody_cert.is_empty() && nobody_key.is_empty())
        && (nobody_cert.is_empty() || nobody_key.is_empty())
    {
        panic!("Only one of GL_CUSTOM_NOBODY_CERT and GL_CUSTOM_NOBODY_KEY is set. Set both or none.")
    }

    // Write the `NOBODY` user cert and key
    let mut cp = Command::new("cp");
    cp.args([nobody_cert, NOBODY_CRT]);
    cp.output().unwrap();

    let mut cp = Command::new("cp");
    cp.args([nobody_key, NOBODY_KEY]);
    cp.output().unwrap();

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
