use std::sync::Arc;
mod gen;
pub use crate::gen::*;

uniffi::include_scaffolding!("glclient");


