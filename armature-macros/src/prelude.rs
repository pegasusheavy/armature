//! Prelude module for common macro imports
//!
//! Add this to easily import all common macros:
//!
//! ```ignore
//! use armature_macros::prelude::*;
//! ```

pub use crate::{
    bad_request,
    created_json,
    guard,
    header,
    internal_error,
    json_object,
    json_response,
    log_error,
    not_found,
    ok_json,
    paginated_response,
    path_param,
    path_params,
    query_param,
    validation_error,
};

