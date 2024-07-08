fn main() {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .type_attribute("TrampolinePayRequest", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(
            &["../proto/glclient/greenlight.proto"],
            &["../proto/glclient"],
        )
        .unwrap();
}
