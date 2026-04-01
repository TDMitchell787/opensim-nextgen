# Contributing to OpenSim NextGen

Thank you for your interest in contributing to OpenSim NextGen.

## Getting Started

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Follow the build instructions in `BUILDING.md`
4. Make your changes
5. Run the test suite (`cargo test`)
6. Commit your changes with a clear message
7. Push to your fork and open a pull request

## Code Style

- **Rust**: Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- **Zig**: Follow Zig standard style
- **Dart/Flutter**: Follow Dart conventions (`dart format`)
- Keep comments minimal — code should be self-documenting
- Follow existing patterns and conventions in the codebase

## Architecture Rules

- **Physics**: All physics, collision, and math work goes in the Zig engine (`zig/src/physics/`). No Rust physics crates.
- **FFI Boundary**: Rust calls Zig via FFI (`rust/src/ffi/mod.rs`). Zig owns simulation state; Rust owns game logic.
- **Flutter**: All desktop UI targets macOS native builds. No web builds.
- **Database**: Support all four backends (PostgreSQL, MySQL, MariaDB, SQLite).

## Security

- Never commit credentials, API keys, or personal data
- Never log secrets or sensitive information
- Follow OWASP guidelines for input validation
- Report security vulnerabilities privately

## Pull Requests

- Keep PRs focused on a single change
- Include a clear description of what and why
- Ensure all tests pass
- Update documentation if behavior changes

## License

By contributing, you agree that your contributions will be licensed under the BSD 3-Clause License.
