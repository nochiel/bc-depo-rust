# BlockchainCommons Server

BlockchainCommons Server (BC-Server) is the reusable lightweight server codebase for all BlockchainCommons projects.
The goal of this project is to allow any BlockchainCommons project to expose its API as a HTTP service in a standardized and easy way.
BlockchainCommons has several projects as commandline tools which could be made more accessible for testing and useful if a server is running that exposes their functionality.
BC-Server will expose its functionality using a JSON-RPC interface
To this end, we have the following requirements:

- A BlockchainCommons command line tool should be able to describe a simple manifest specifying
  - named endpoints (path and parameters)
  - the command to be executed when an endpoint is called.
- The manifest should be written as a rust library and can contain arbitrary logic to process input and generate output.
  - This library can then be put into a "modules" directory and the server will automatically make its functionality available.

## The bc-server API (How to write a module for bc-server)

- In modules, copy and rename the `example` directory.
  - Rename `example/example.rs`
  - Ensure that you fill in the following required module functions in `example.rs`:
    - `make_routes()`: This function generates the routes that will be exposed by your module.
    - `start_server()`:
      - Put any startup code required by your module here e.g. initializing a database.
      - Check for and configure any required dependencies here.
- In modules/mod.rs, change `mod example` to refer to the name of your module from the previous step.
- Run `cargo add example-crate` and replace `example-crate` with the name(s) of the crate(s) that your module will use for its functionality.

## Modules

The following APIs are implemented:

- bc-depository: secure storage and retrieval of binary objects.

## References

- [Core Lightning](https://github.com/ElementsProject/lightning) (aka `lightningd`) has a module/plugin system that will server as a model for BC-Server.
  - See [A day in the life of a plugin
    ](https://github.com/ElementsProject/lightning/blob/master/doc/developers-guide/plugin-development/a-day-in-the-life-of-a-plugin.md)

## Bugs

- During compilation you might have to wrestle with: <https://users.rust-lang.org/t/error-e0635-unknown-feature-stdsimd/106445/2>

```
  Error[E0635]: unknown feature `stdsimd`
  --> /home/nik/.cargo/registry/src/index.crates.io-6f17d22bba15001f/ahash-0.8.3/src/lib.rs:99:4
```

- To fix:
  - `rustup default nightly`
  - `rm Cargo.lock` (because it will have conflicting versions of ahash)
