# Contributing

Thank you for considering contributing to [rustodo](https://github.com/joaofelipegalvao/rustodo)!

When contributing, please first discuss the change you wish to make via [issue](https://github.com/joaofelipegalvao/rustodo/issues) or any other method with the owner of this repository before making a change.

---

## Setup

1. Fork this repository and create your branch from `main`.
2. Clone your forked repository:

```sh
git clone https://github.com/{username}/rustodo && cd rustodo
```

1. Install [Rust](https://www.rust-lang.org/) 1.70 or later and build the project:

```sh
cargo build
```

---

## Development Workflow

1. Start committing your changes. Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification — this project uses it to automate releases and generate the changelog.

2. Add or update tests according to your changes, then verify they pass:

```sh
cargo test
```

1. Run `clippy` and make sure there are no warnings:

```sh
cargo clippy --tests -- -D warnings
```

1. Check formatting:

```sh
cargo fmt --all -- --check
```

If formatting fails, run:

```sh
cargo fmt --all
```

---

## Commit Convention

This project follows [Conventional Commits](https://www.conventionalcommits.org/). Releases are automated via GitHub Actions:

| Prefix | Effect |
|--------|--------|
| `feat:` | Minor version bump |
| `fix:` | Patch version bump |
| `BREAKING CHANGE` | Major version bump |

---

## Create a Pull Request

1. Ensure your changes are tested and the code is formatted.
2. Fill in the Pull Request description with a clear explanation of what was changed and why.
3. Wait for review. Discuss possible changes and update your Pull Request if necessary.

---

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](./LICENSE).
