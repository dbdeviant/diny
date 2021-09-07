# Contributing to diny

**diny** welcomes contribution from everyone in the form of suggestions, bug
reports, pull requests, and feedback. This document gives some guidance if you
are thinking of helping us.

## Getting help

**diny** is very much a work in progress and there are many pending design decisions before it can be considered fully implemented and documented, let alone stable. It is intended to only be used for experimentation and design feedback at this point.  In fact, only a limited subset of data structures are currently supported, and won't be available until the project hits v0.1.0.  (This is to enable faster iteration on design changes)  If you're adventurous and would like to provide some feedback, please hit us up with your ideas [here](https://github.com/dbdeviant/diny/discussions).

## Opening issues

**diny** development is currently very young and we have some very specific
short-term goals we are trying to achieve.  It's also incomplete, so it's
expected that there are areas that don't work just yet, and may not for a while.
However, we're very open to suggestions and feature requests, especially around
the core design and api aspects. The GitHub Discussions board is currently the
best choice for asking questions and bouncing ideas around.  We'll selectively
open issues on an as-needed basis from there.

## Running the workspace tests

**diny** currently requires the nightly rust toolchain >= 1.56.0. Cargo is our
defacto test suite runner, and all tests can be run by executing the following
in the main project directory:

```sh
cargo +nightly test
```

[`serde`]: https://github.com/serde-rs/serde/tree/master/serde
[`test_suite`]: https://github.com/serde-rs/serde/tree/master/test_suite

## Conduct

**diny** follows the [Rust Code of Conduct] and [GitHub Community Guidelines].  Abusive behavior
of any kind will not be tolerated and will be strictly moderated.  Be kind, be inclusive,
and have fun!

[Rust Code of Conduct]: https://www.rust-lang.org/policies/code-of-conduct
[GitHub Community Guidelines]: https://docs.github.com/en/github/site-policy/github-community-guidelines