pub mod routing;
pub mod routes;
pub use routing::route;
pub use routing::generate_response;
pub use routing::generate_static_response;
pub use routing::is_requesting_static_resource;