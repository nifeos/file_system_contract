# file_system_contract

This repository holds two crates of one filesystem stack, as one Cargo workspace:

- [`file_system_contract`](file_system_contract/README.md): the wire protocols, as code. The
  block protocol runs between a filesystem server and a block server, and the file-service protocol
  between a client and the filesystem server.
- [`file_system_fixture`](file_system_fixture/README.md): the test data both sides of those
  protocols agree on. It never publishes.

Each crate's README says how to build and test it. From this directory, `cargo test` runs both.

## License

Each crate is licensed under either of the [Apache License, Version 2.0](file_system_contract/LICENSE-APACHE)
or the [MIT license](file_system_contract/LICENSE-MIT), at your option.
