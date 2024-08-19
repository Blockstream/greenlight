fn main() {
    tonic_build::configure()
        .build_client(true)
        .compile(
            &[".resources/proto/glclient/greenlight.proto"],
            &[".resources/proto/glclient"],
        )
        .unwrap();
}
