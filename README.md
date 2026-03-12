## autoschematic-connector-template

This crate is a minimal starter template for building a new Autoschematic connector in Rust. 

Start with these changes:

1. Rename the package in `Cargo.toml`.
2. Rename `DummyConnector` everywhere.
3. Start following the guide at [https://autoschematic.sh/guide/building-your-own-connectors-in-rust/a-starter-template/](https://autoschematic.sh/guide/building-your-own-connectors-in-rust/a-starter-template/)

Test import with `autoschematic import` .


This repo includes autoschematic.ron already hooked up to load the local crate.


```rust
AutoschematicConfig(
    prefixes: {
        "main": Prefix(
            connectors: [
                Connector(
                    shortname: "mything",
                    spec: CargoLocal(
                        // Absolute paths are also supported.
                        // path: "/home/chef/prog/ottercorp-connector-mything",
                        path: "./",
                    ),
                )
            ]
        )
    }
)

```
