fn main() {
    // Migrations are embedded into the binary, so we need to re-build if they change
    println!("cargo:rerun-if-changed=migrations");

    // Bundled web assets are the same:
    println!("cargo:rerun-if-changed=web");

    // Build web assets...
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::Path::new(&out_dir);
    let dist = out_dir.join("webdist");
    build_assets(&dist);

    // ...and generate Rocket routes to serve them
    let routes_src = asset_routes(&dist);
    let routes_path = out_dir.join("webdist.rs");
    std::fs::write(routes_path, routes_src).expect("failed to write webdist.rs");
}

fn build_assets(dist: &std::path::Path) {
    // Clear old bundled assets
    if dist.try_exists().unwrap_or(false) {
        std::fs::remove_dir_all(dist).expect("failed to remove old dist");
    }
    // Install parcel
    std::process::Command::new("yarn")
        .args(["install", "--frozen-lockfile"])
        .status()
        .expect("failed to run `yarn install`");
    // Build web assets
    std::process::Command::new("yarn")
        .args([
            "run", "parcel", "build",
            "--dist-dir", &dist.to_string_lossy(),
            "--no-source-maps",
        ])
        .status()
        .expect("failed to run `yarn build`");
}

fn asset_routes(dist: &std::path::Path) -> String {
    let mut branches = vec![];
    for entry in std::fs::read_dir(dist).expect("failed to read dist") {
        let entry = entry.expect("failed to read dist entry");
        let path = entry.path();
        if !path.is_file() {
            panic!("unexpected non-file in dist: {path:?}");
        }
        let file = path.file_name().expect("file has no name").to_string_lossy();
        let ext = path.extension().expect("file has no extension").to_string_lossy();
        let rel_path = dist.join(file.to_string());
        let rel_path = rel_path.to_string_lossy();
        branches.push(format!(
            "\"{file}\" => Ok((ContentType::from_extension(\"{ext}\").unwrap(), include_str!(\"{rel_path}\"))),"
        ));
    }
    let index = dist.join("index.html");
    let index = index.to_string_lossy();
    let branches = branches.join("\n");
    format!(
        r#"
        #[get("/")]
        fn embedded_index() -> (ContentType, &'static str) {{
            (ContentType::HTML, include_str!("{index}"))
        }}

        #[get("/static/<file>")]
        fn embedded_static_file(file: &str) -> Result<(ContentType, &'static str), (Status, &'static str)> {{
            match file {{
                {branches}
                _ => Err((Status::NotFound, "file not found")),
            }}
        }}
        "#,
    )
}
