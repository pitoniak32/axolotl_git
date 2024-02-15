<p align="center">
   <img src="https://github.com/pitoniak32/axolotl_git/assets/84917393/8d0d9970-5ffb-469f-b382-7d0de50cccb9" width="200"/>
</p>

# Axolotl Git

A repo management cli that will allow your projects to grow, and regenerate like an axolotl!

## Getting Started

(Requires tmux, and fzf for majority of features)
- [tmux](https://github.com/tmux/tmux/wiki)
- [fzf](https://github.com/junegunn/fzf?tab=readme-ov-file#installation)

### Install

(Currently not supported on windows)

#### Using cargo
```
cargo install --locked axolotl_git
```

#### Using Github Releases

Download a release from [here](https://github.com/pitoniak32/axolotl_git/releases), and add it to a directory on your path.


### Running

Add a projects directory file
example:
path: `~/.config/axl/personal_projects.yml`
```yml
path: "/home/your_user/Projects"
projects: 
  - git@github.com:your_github/your_project.git
  - git@github.com:your_github/your_other_project.git
```
now let `axl` know you would like to use this file with:
 - an env var `export PROJECTS_DIRECTORY_FILE=~/.config/axl/personal_projects.yml`
 - a flag `--projects-directory-file=~/.config/axl/personal_projects.yml`

Check the available commands
```
$ axl --help
project management cli

Usage: axl [OPTIONS] [COMMAND]

Commands:
  project  Commands for managing projects
  help     Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...                 Increase logging verbosity
  -q, --quiet...                   Decrease logging verbosity
  -c, --config-path <CONFIG_PATH>  Override '$XDG_CONFIG_HOME/config.yml' or '$HOME/.axlrc.yml' defaults
  -h, --help                       Print help
  -V, --version                    Print version
```

### Helpful Commands
open
```
axl project open --help 
```

```
axl project open -m tmux
```
