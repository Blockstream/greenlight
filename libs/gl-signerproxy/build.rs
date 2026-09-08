fn main() {
    tonic_prost_build::configure()
        .build_client(true)
        .compile_protos(
            &[".resources/proto/glclient/greenlight.proto"],
            &[".resources/proto/glclient"],
        )
        .unwrap();
}
