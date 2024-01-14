# Beatdrop

![Screenshot showing beatdrop gameplay](https://github.com/Artemis21/beatdrop/assets/57376638/4d8c6c1b-4b19-4e18-83bb-ec0c800ee348)

Beatdrop is a "name that tune" music guessing game, strongly inspired by the discontinued
Heardle game.

## Deployment

1. Install [Postgres](https://postgresql.org), create a database, and create a user with
   access to that database. You should end up with a database connection URL in the form
   `postgres://username:password@host:port/database_name`.
2. Create a configuration file. This should be a TOML file with the following keys:

    - `db_url` -- the database connection URL from step 1
    - `session_key` -- a random string used to sign session tokens
    - `media_dir` -- a relative path to a directory where media files will be cached

    See the development section below for an example.

3. Obtain a copy of the `beatdrop` binary. Currently, you must build it yourself:
    1. Install [Node.js](https://nodejs.org/),
       [Yarn Modern](https://yarnpkg.com/getting-started/install) and
       [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).
    2. Clone this repository.
    3. Run `cargo build --release` in the repository root. The binary will be in
       `target/release/beatdrop`.
4. Run the binary, passing the path to the configuration file as the first argument. If
   the configuration file is named `beatdrop.toml` and is in the current directory, you
   can just run the binary with no arguments: `./beatdrop`.

## Development

This app is implemented as a JSON API server written in Rust, alongside an SPA frontend
built using React.

For development, you will need to have [Yarn](https://yarnpkg.com/getting-started/install)
and [Cargo](https://rustup.rs/) installed.

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

For backend development, if you are touching database queries, you should set up a local
Postgres database and allow SQLx to connect to it. From
[the SQLx docs](https://docs.rs/sqlx/latest/sqlx/macro.query.html#requirements):

> The `DATABASE_URL` environment variable must be set at build-time to point to a database
> server with the schema that the query string will be checked against. All variants of
> `query!()` use dotenv so this can be in a `.env` file instead.

If you are not touching database queries, metadata prepared by `cargo sqlx prepare` should
be present in the repository, which will allow SQLx to compile offline.

Helpful commands:

-   `yarn install` -- install dependencies and development tools for JS
-   `yarn fmt` -- format JS code
-   `yarn check` -- check and lint JS code
-   `cargo fmt` -- format Rust code
-   `cargo clippy` -- check and lint Rust code
-   `cargo run` -- run the server in development mode
-   `cargo build --release` -- build the server in release mode
-   `cargo sqlx prepare` -- generate metadata for SQL queries (run this after adding or
    changing any SQLx queries)
-   `cargo sqlx migrate add <description>` -- create a new database migration template
