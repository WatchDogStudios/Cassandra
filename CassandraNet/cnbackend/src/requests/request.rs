use std::collections::HashMap;

use cncommon::profile::cnprofile::{CnProfile, Profiler};

/// Represents the type of a CassandraNet request.
#[derive(Debug)]
pub enum CnRequestType {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
    Trace,
    Connect,
    Custom,
    ///> (Mikael A.) Custom request type for any other request type that is not listed here. could be evaluated as a class of its own.
    Unknown,
}

/// Represents the HTTP method of a request.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Method {
    Get,
}

/// A trait for registering custom requests.
pub trait CnCustomRequest {
    /// Registers a custom request type.
    fn register_custom_request(&mut self, request_type: Method);
}

/// Represents a CassandraNet request.
pub struct CnRequest {
    pub request_id: u64,
    pub request_type: Method,
    pub request_data: Option<String>,
    ///> optional (basically a nullable field, can be None/Null or a String)
    pub headers: HashMap<String, String>,
}

impl CnRequest {
    /// Creates a new CnRequest instance.
    pub fn new(
        request_id: u64,
        request_type: Method,
        request_data: Option<String>,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            request_id,
            request_type,
            request_data,
            headers,
        }
    }
}

impl CnCustomRequest for CnRequest {
    /// Registers a custom request type.
    fn register_custom_request(&mut self, request_type: Method) {
        CnProfile::start_event("register_custom_request");
        self.request_type = request_type;
    }
}
