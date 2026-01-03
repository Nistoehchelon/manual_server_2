pub mod routes;
pub mod router;
pub use router::generate_response;
pub use router::generate_static_response;
pub use router::is_requesting_static_resource;
pub use router::route;
