# MokaPot

![CI - GitHub Actions](https://img.shields.io/github/actions/workflow/status/henryhchchc/mokapot/ci.yml?style=flat-square&logo=githubactions&logoColor=white&label=CI)
![Codecov](https://img.shields.io/codecov/c/github/henryhchchc/mokapot?style=flat-square&logo=codecov&logoColor=white&label=Coverage)
![Crates.io](https://img.shields.io/crates/v/mokapot?style=flat-square&logo=rust&logoColor=white)
![docs.rs](https://img.shields.io/docsrs/mokapot?style=flat-square&logo=docsdotrs&logoColor=white&label=docs%2Frelease)

MokaPot is a Java bytecode analysis library written in Rust.

> [!WARNING]
> **API Stability:** This project is in an early development stage and breaking changes can happen before v1.0.0.
> Documentations are incomplete, which will be added when the basic functionalities works.
> Using this project for production is currently NOT RECOMMENDED.

## Documentation

The documentation of the released version is available at [docs.rs](https://docs.rs/mokapot).
The documentation of the latest commit is available at [github.io](https://henryhchchc.github.io/mokapot/mokapot/)

## Usage

### Adding the dependency

Add the following line to the `[dependencies]` section in your `Cargo.toml`.

```toml
mokapot = "0.12"
```

Alternatively, to follow the latest commit version, add the following line instead.
Before building your project, run `cargo update` to fetch the latest commit.

```toml
mokapot = { git = "https://github.com/henryhchchc/mokapot.git" }
```

### Parsing a class

```rust
use mokapot::jvm::class::Class;

fn parse_class() -> Result<Class, Box<dyn std::error::Error>> {
    let reader: std::io::Read = todo!("Some reader for the byte code");
    let class = Class::from_reader(reader)?;
    Ok(class)
}
```

### MokaIR

MokaIR is an intermediate representation of JVM bytecode in [mokapot](https://github.com/henryhchchc/mokapot).
To learn more, please refer to [docs/MokaIR.md](docs/MokaIR.md)

## Building

Make sure you have the following tools installed:

- The latest stable version of Rust
- The latest release version of JDK

Compile the project and run the tests with the following command.

```bash
cargo build --all-features
cargo test --all-features
```

## Contributing

Cool. Contributions are welcomed. See the [contribution guide](docs/CONTRIBUTING.md) for more information.
