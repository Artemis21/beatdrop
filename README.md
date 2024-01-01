# Beatdrop

Beatdrop is a "name that tune" music guessing game, strongly inspired by the discontinued
Heardle game.

## Development

This app is implemented as a JSON API server written in Rust, alongside an SPA frontend
built using React.

For development, you will need to have Yarn and Cargo installed. You also need to have a
PostgreSQL database available.

To run the server, first create a config file named `beatdrop.toml` in the current
directory, like so:

```toml
db_url = "postgres://..."
session_key = "some arbitrary secret for signing session tokens"
media_dir = "path/to/directory_for_caching_media"
```

Then, run the development-mode server using `cargo run` (if you want to put
`beatdrop.toml` somewhere else, just pass it's path as the first argument, like
`cargo run -- path/to/myconfig.toml`).

By default, the development-mode server does HMR, meaning that any changes you make to the
web frontend (but not the Rust backend) will be automatically sent to the browser on save.
You can turn this off by adding `dev = false` to the config - this will require you to
rebuild the server every time you change frontend code.

For production, build in release mode - either `cargo run --release` or
`cargo build --release` and then run the resulting binary. HMR is not supported in
production (frontend files are embedded in the binary), but it is otherwise the same.

For frontend development, you should also
[enable Yarn editor SDKs](https://yarnpkg.com/getting-started/editor-sdks).
