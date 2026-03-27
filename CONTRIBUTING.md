# Contributing to Vibe Coding Tracker

First off, thanks for taking the time to contribute!

All types of contributions are encouraged and valued. See the [Table of Contents](#table-of-contents) for different ways to help and details about how this project handles them. Please make sure to read the relevant section before making your contribution. It will make it a lot easier for us maintainers and smooth out the experience for all involved. The community looks forward to your contributions. 

## Table of Contents

- [I Have a Question](#i-have-a-question)
- [I Want To Contribute](#i-want-to-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Your First Code Contribution](#your-first-code-contribution)
  - [Development Guide](#development-guide)
    - [Building from Source](#building-from-source)
    - [Running Tests](#running-tests)
    - [Code Quality](#code-quality)

## I Have a Question

Before you ask a question, it is best to search for existing [Issues](https://github.com/Mai0313/VibeCodingTracker/issues) that might help you. In case you have found a suitable issue and still need clarification, you can write your question in this issue. It is also advisable to search the internet for answers first.

If you then still feel the need to ask a question and need clarification, we recommend the following:

- Open an [Issue](https://github.com/Mai0313/VibeCodingTracker/issues/new).
- Provide as much context as you can about what you're running into.
- Provide project and platform versions (nodejs, mac, linux, rust, etc), depending on what seems relevant.

## I Want To Contribute

### Reporting Bugs

A good bug report shouldn't leave others needing to chase you up for more information. Therefore, we ask you to investigate carefully, collect information and describe the issue in detail in your report.

- Make sure that you are using the latest version.
- Determine if your bug is really a bug and not an error on your side e.g. using incompatible environment components/versions (Make sure that you have read the [documentation](README.md)).
- Check if other users have experienced (and potentially already solved) the same issue you are having. Check if there is not already a bug report existing for your bug or error in the [bug tracker](https://github.com/Mai0313/VibeCodingTracker/issues).

### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion, including completely new features and minor improvements to existing functionality.

- Make sure that you are using the latest version.
- Read the [documentation](README.md) carefully and find out if the functionality is already covered, maybe by an individual configuration.
- Perform a [search](https://github.com/Mai0313/VibeCodingTracker/issues) to see if the enhancement has already been suggested. If it has, add a comment to the existing issue instead of opening a new one.
- Find out whether your idea fits with the scope and aims of the project. It's up to you to make a strong case to convince the project's developers of the merits of this feature.

### Your First Code Contribution

Unsure where to begin contributing? You can start by looking through `good first issue` and `help wanted` issues.

### Development Guide

#### Building from Source

For users who want to customize the build or contribute to development:

```bash
# 1. Clone the repository
git clone https://github.com/Mai0313/VibeCodingTracker.git
cd VibeCodingTracker

# 2. Build release version
cargo build --release

# 3. Binary location
./target/release/vibe_coding_tracker

# 4. Optional: create a short alias
# Linux/macOS:
sudo ln -sf "$(pwd)/target/release/vibe_coding_tracker" /usr/local/bin/vct

# Or install to user directory:
mkdir -p ~/.local/bin
ln -sf "$(pwd)/target/release/vibe_coding_tracker" ~/.local/bin/vct
# Make sure ~/.local/bin is in your PATH
```

**Prerequisites**: [Rust toolchain](https://rustup.rs/) 1.85 or higher

> [!NOTE]
> This project uses **Rust 2024 edition** and requires Rust 1.85+. Update your toolchain with `rustup update` if needed.

#### Running Tests

Before submitting a pull request, ensure all tests pass:

```bash
cargo test
```

#### Code Quality

We use `rustfmt` and `clippy` to ensure code quality. Please run the following commands to check your code before submitting a pull request:

```bash
# Format your code
cargo fmt --all

# Run linting checks
cargo clippy --all-targets --all-features -- -D warnings
```
