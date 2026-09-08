fn main() {
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .type_attribute(
            "TrampolinePayRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile_protos(
            &[".resources/proto/glclient/greenlight.proto"],
            &[".resources/proto/glclient"],
        )
        .unwrap();
}
