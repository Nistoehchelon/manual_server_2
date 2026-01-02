pub mod routes;
pub mod routing;
pub use routing::generate_response;
pub use routing::generate_static_response;
pub use routing::is_requesting_static_resource;
pub use routing::route;
