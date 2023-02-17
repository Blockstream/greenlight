tonic::include_proto!("greenlight");

pub mod scheduler {
    tonic::include_proto!("scheduler");

}

pub use cln_grpc::pb as cln;
