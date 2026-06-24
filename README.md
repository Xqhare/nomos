# Nomos

The base for my project management solution

It follows my "All code written by me or part of rust's standard library and libc" philosophy.
You can learn more about that [here](https://blog.xqhare.net/posts/why_solve_problems/).

> [!note]
> As of right now, Nomos is a work in progress.
> The parser is stableish, but consider the entire project a minimum viable product.

## Roadmap

`Nomos` uses my [nomos](https://github.com/xqhare/nomos) project management system.

The roadmap for this project can be found in the [nomos.md](nomos.md) file.

All nomos files follow the syntax defined [here](https://github.com/Xqhare/nomos/blob/master/spec/).

## Features

- _**No dependencies**_: All code is written by me or part of std.

## Naming

As with all my projects, Nomos is named after an ancient deity.
Learn more about my naming scheme [here](https://blog.xqhare.net/posts/explaining_the_pantheon/).

Nomos was a lesser Greek deity of laws, statutes, and ordinances.

## Usage

Nomos is both available to be used as a library and as a command line tool.

### Command line tool

Clone the repository and run `cargo install --path .`

First, run `nomos` by itself to create the config file.
Then, confirm the setup with `nomos validate`.
If successful, run `nomos all` or `nomos next` to get started.

Run `nomos --help` for more information.

### Importing

Add the following to your `Cargo.toml`:

```toml
[dependencies]
nomos = { git = "https://github.com/xqhare/nomos" }
```

### Example

```rust

```

## License

Nomos is distributed under the [MIT](https://github.com/xqhare/nomos/blob/master/LICENSE) license.

## Contributing

See [CONTRIBUTING](https://github.com/xqhare/nomos/blob/master/CONTRIBUTING.md) for contribution guidelines.
