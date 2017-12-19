# Feature: Server

The server feature provides a readonly RESTful API on port 8000

# Example

- [ascii-hub](https://github.com/ascii-dresden/ascii-hub)

# Setup

You'll need the latest [Rust](https://rust-lang.rs) nightly build.

**NOTE:** See README.md for information how to install rustup

---

Use

```sh
$ rustup install nightly
```

to install the latest nightly build.

You may also use.

```sh
$ rustup default nightly
```

Visit [rustup.rs](https://github.com/rust-lang-nursery/rustup.rs#installation) for more information.

**NOTE:** While I'm writing this documentation (Jan. 2018), i receive compilation errors. See [Ring Issue #598](https://github.com/briansmith/ring/issues/598) for a temporary fix.

## Run

Run the following command to compile your project and make use of the server feature.

``sh
$ cargo run --example server --features server
``

This will create an local instance of a [Rocket](https://github.com/SergioBenitez/Rocket) server in port 8000.

Since rocket runs on port 8000 by default, your browser will block Cross-Origin request. You'll have to allow Cross-Origin requests to use the API.

# HTTP Endpoints

Content-Type: application/json

- GET `/api/projects` \=\> all project identifiers
- GET `/api/projects/year` \=\> all years
- GET `/api/projects/year/<year>` \=\> all project identifiers of `<year>`
- GET `/api/projects/<identifier>` \=\> project with `<identifier>`
- GET `/api/full_projects` \=\> all projects
- GET `/api/full_projects/year/<year>`  \=\> all projects of `<year>`