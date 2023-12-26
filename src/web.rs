//! Serve static web assets.
use rocket::{get, routes, Route, http::{ContentType, Status}};
#[cfg(debug_assertions)]
use std::path::PathBuf;

/// Get the routes for serving static web assets.
#[cfg_attr(debug_assertions, allow(unused_variables))]  // dev_mode ignored in release
pub fn routes(dev_mode: bool) -> Vec<Route> {
    #[cfg(debug_assertions)]
    if dev_mode {
        std::thread::spawn(watch_frontend);
        return routes![filesystem_index, filesystem_static_file];
    }
    routes![embedded_index, embedded_static_file]
}

// Routes serving static files embedded in the binary, generated by build.rs.
include!(concat!(env!("OUT_DIR"), "/webdist.rs"));

/// Run `parcel watch` to rebuild the frontend on changes, and add HMR code.
#[cfg(debug_assertions)]
fn watch_frontend() {
    let mut parcel = std::process::Command::new("yarn")
        .args(["run", "parcel", "watch"])
        .spawn()
        .expect("failed to run `yarn run parcel watch`");
    let status = parcel.wait().expect("failed to wait for `yarn run parcel watch`");
    if !status.success() {
        eprintln!("`yarn run parcel watch` failed - changes to the frontend will not be rebuilt");
    }
}

/// Serve the index page from the filesystem.
#[cfg(debug_assertions)]
#[get("/")]
fn filesystem_index() -> (ContentType, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("dist")
        .join("index.html");
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("failed to read {path:?}"));
    (ContentType::HTML, content)
}

/// Serve a static file from the filesystem.
#[cfg(debug_assertions)]
#[rocket::get("/static/<file>")]
fn filesystem_static_file(file: &str) -> Result<(ContentType, String), (Status, &'static str)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("dist")
        .join(file);
    if !path.try_exists().unwrap_or(false) {
        return Err((Status::NotFound, "file not found"));
    }
    let ext = path
        .extension()
        .ok_or((Status::InternalServerError, "file has no extension"))?
        .to_string_lossy();
    let mime_type = match ext.as_ref() {
        "js" => ContentType::JavaScript,
        "css" => ContentType::CSS,
        "html" => ContentType::HTML,
        "map" => ContentType::JSON,
        _ => panic!("unknown file type"),
    };
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("failed to read {path:?}"));
    Ok((mime_type, content))
}
