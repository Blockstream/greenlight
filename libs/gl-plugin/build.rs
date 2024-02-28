fn main() {
    tonic_build::compile_protos("../proto/glclient/greenlight.proto").unwrap();
}
