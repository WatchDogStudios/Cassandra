//! Generated protobuf & gRPC service types.

pub mod agent {
    include!(concat!(env!("OUT_DIR"), "/cassandra.agent.v1.rs"));
}

pub mod orchestration {
    include!(concat!(env!("OUT_DIR"), "/cassandra.orchestration.v1.rs"));
}

pub mod ugc {
    include!(concat!(env!("OUT_DIR"), "/cassandra.ugc.v1.rs"));
}

pub mod messaging {
    include!(concat!(env!("OUT_DIR"), "/cassandra.messaging.v1.rs"));
}

pub use agent::*;
pub use messaging::*;
pub use orchestration::*;
pub use ugc::*;
