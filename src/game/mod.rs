//! Handle games in the database as well as game logic.
mod database;
mod logic;
mod response;
mod routes;

pub use database::Game;
pub use response::Response;
pub use routes::routes;
