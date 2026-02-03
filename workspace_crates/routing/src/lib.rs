pub mod router;
pub mod routes;
pub use router::RouterData;
pub use router::generate_response_data;
pub use router::generate_static_response;
pub use router::is_requesting_static_resource;
pub use router::route;
