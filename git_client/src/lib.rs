use std::{
    path::PathBuf,
    process::{Command, Output},
};

const GIT_COMMAND: &str = "git";

#[derive(Clone, Debug)]
pub struct GitOpts {
    root: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Git {
    pub command: Command,
    pub opts: Option<GitOpts>,
}

impl Git {
    pub fn status() -> Git {
        let mut git = Git {
            command: Command::new(GIT_COMMAND),
            opts: None,
        };
        git.command.arg("status");
        git
    }

    pub fn log() -> Git {
        let mut git = Git {
            command: Command::new(GIT_COMMAND),
            opts: None,
        };
        git.command.arg("log");
        git
    }

    pub fn add_arg(mut self, arg: &str) -> Git {
        self.command.arg(arg);
        println!("add_arg: {:?}", self);
        self
    }

    pub fn done(mut self) -> Output {
        println!("done: {:?}", self);
        if let Some(opt) = self.opts.clone() {
            self.command.current_dir(opt.root.unwrap_or(PathBuf::from(".")));
        }
        self.command.output().expect("command was not failure")
    }
}

pub trait PrintStdout {
    fn print_stdout(&self);
}

impl PrintStdout for Output {
    fn print_stdout(&self) {
        println!("{}", String::from_utf8(self.stdout.clone()).unwrap().to_string());
        eprintln!("{}", String::from_utf8(self.stderr.clone()).unwrap().to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        
    }
}
