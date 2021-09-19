fn main() {
    tonic_build::compile_protos("../proto/greenlight.proto").unwrap();
    tonic_build::compile_protos("../proto/scheduler.proto").unwrap();
}
