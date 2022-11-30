fn main() {
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
