
pub mod greenlight {
    tonic::include_proto!("greenlight");
}

pub mod scheduler {
    tonic::include_proto!("scheduler");

}
pub use greenlight::*;
pub use cln_grpc::pb as cln;
