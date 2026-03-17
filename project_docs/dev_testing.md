# Local development testing

`ruv.toml` and `ruv.lock` are gitignored in this repo because ruv itself has no
R package dependencies — they are only created when manually testing ruv commands
in the development directory.

To test `ruv` against a real project, create a `ruv.toml` in the repo root:

```toml
[project]
name = "test_project"
version = "0.1.0-alpha.1"
r-version = "4.3.2"
dependencies = [
    "ggplot2 >= 3.1",
    "dplyr",
    "tidymodels == 1.1.0"
]
```

Then run:

```sh
cargo run -- lock    # resolve and write ruv.lock
cargo run -- sync    # install packages + set up .ruv/bin symlinks
cargo run -- run -e "library(ggplot2); sessionInfo()"
```

The `.ruv/` directory (library + bin symlinks) is also gitignored.
