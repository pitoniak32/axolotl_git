<p align="center">
   <img src="https://github.com/pitoniak32/axolotl_git/assets/84917393/8d0d9970-5ffb-469f-b382-7d0de50cccb9" width="200"/>
</p>

# Axolotl Git
Tmux session management CLI

## Getting Started
Requires tmux, fzf, and zoxide.
- [tmux](https://github.com/tmux/tmux/wiki)
- [fzf](https://github.com/junegunn/fzf?tab=readme-ov-file#installation)
- [zoxide](https://github.com/ajeetdsouza/zoxide?tab=readme-ov-file#getting-started)

### Install
(Currently not supported on windows)

#### Using cargo
```
cargo install --locked axolotl_git
```

#### Using Github Releases
Download a release from [here](https://github.com/pitoniak32/axolotl_git/releases), and add it to a directory on your path.

### Running
Check the available commands
```
$ axl --help
```

## Future Enhancements
- Fancy `fzf` custom prompts like [zoxide](https://github.com/ajeetdsouza/zoxide).

## Tracing
This cli is instrumented with tokio tracing. If you increase the verbosity of the cli you will see more logs with details that can help with troubleshooting.

To modify the verbosity of the logs, use:
- `-v` or `-q` for the console output.
- `RUST_LOG=<trace,debug,info,warn,error>` for the traces that are shipped to the optional OTEL collector. (see [this](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives) for more advanced options)

### References:
- https://github.com/tokio-rs/tracing | https://docs.rs/tracing/latest/tracing/index.html

## Inspiration
- for `zoxide` integration: https://github.com/omerxx/tmux-sessionx
