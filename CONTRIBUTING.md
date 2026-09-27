# Contributing to AeroOS

Thanks for your interest! Here's how to contribute.

## Code of Conduct

Be respectful. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## How to contribute

### 1. Report bugs

Open an [issue](https://github.com/user123141/AeroOS/issues) with:
- Windows version (`winver`)
- AeroOS version (`aeroos.exe --version`)
- Steps to reproduce
- Expected vs actual behavior
- Logs (from `%TEMP%\aeroos.log`)

### 2. Suggest features

Open an [issue](https://github.com/user123141/AeroOS/issues) with the `enhancement` label.

### 3. Submit code

    # Fork & clone
    git clone https://github.com/<your-username>/AeroOS.git
    cd AeroOS

    # Create branch
    git checkout -b feature/my-feature

    # Make changes
    # ...

    # Verify
    cargo fmt
    cargo clippy -- -D warnings
    cargo test

    # Commit
    git commit -m "feat: add my feature"

    # Push
    git push origin feature/my-feature

    # Open Pull Request

## Commit conventions

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` — new feature
- `fix:` — bug fix
- `docs:` — documentation
- `refactor:` — refactoring
- `test:` — tests
- `chore:` — build/config

Examples:
- `feat(virtio): add virtio-fs support`
- `fix(snapshot): handle partial block writes`
- `docs(readme): add benchmarks section`

## Code style

- Run `cargo fmt` before committing
- No `clippy` warnings (`cargo clippy -- -D warnings`)
- Use `anyhow` for errors in application code
- Use `thiserror` for library errors
- Document all public APIs with `///`
- Add tests for new functionality

## Testing

    cd AeroOS
    cargo test              # Unit + integration
    cargo test --release    # Release-mode tests
    cargo bench             # Benchmarks (if available)

## Project structure

See [README.md](README.md#-project-structure).

## Questions?

Open a [discussion](https://github.com/user123141/AeroOS/discussions).