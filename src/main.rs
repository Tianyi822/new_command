use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    author = "Tianyi",
    version = "0.0.1",
    about = "A simple command line tool written in Rust"
)]
struct LsCli {
    #[arg(short = 'l', help = "show details of files and directories")]
    long: bool,

    #[arg(short = 'a', help = "show hidden files and directories")]
    all: bool,

    #[arg(long = "hr", help = "show human readable file sizes")]
    human_readable: bool,

    #[arg(default_value = ".", help = "set file or directory path")]
    path: Option<std::path::PathBuf>,

    // This is a hidden field，it will not be shown in help message,
    // but it can be used to store the status of the command.
    // This field just like a state machine to show the status of the command,
    // and to instruct the program what to do next.
    // 'ls'             => status-0 : default status
    // 'ls -l'          => status-1 : show details of files and directories
    // 'ls -a'          => status-2 : show hidden files and directories
    // 'ls -h'          => status-4 : set status to 4, but do nothing, don't ask why, Linux ls command also do nothing when get '-h' option
    // 'ls -a -l'       => status-3 : calculated by 1 | 2, it will show details of all hidden files and directories
    // 'ls -l -h'       => status-5 : calculated by 1 | 4, it will show details of files and directories with human readable file sizes
    // 'ls -a -l -h'    => status-7 : calculated by 1 | 2 | 4, it will show details of all hidden files and directories with human readable file sizes
    // other command    => status-0 : default status
    // Above status were set by the parse function what we implemented in the impl code block.
    //
    // Attention: You must use #[arg(skip)] to skip the hidden field,
    // otherwise it will be shown in help message, and even panic will appear in the program!!!
    #[arg(skip)]
    status: u8,
}

impl LsCli {
    // set status of the command
    fn set_status(&mut self) {
        // set status to 0 by default
        self.status = 0;

        // set status to 1 if get '-l' option
        if self.long {
            self.status |= 1;
        }

        // set status to 2 if get '-a' option
        if self.all {
            self.status |= 2;
        }

        // set status to 4 if get '-h' option
        if self.human_readable {
            self.status |= 4;
        }
    }

    // get status of the command
    fn get_status(&self) -> u8 {
        self.status
    }

    // execute the command
    pub fn execute(&mut self) {
        self.set_status();
        println!("status: {}", self.get_status());
    }
}

// parse ls command
fn _parse_ls_command() {
    let list_cli = LsCli::parse();
    println!("{:#?}", list_cli);

    // get path that user input
    let path = list_cli.path.unwrap();
    let path = path.to_str().unwrap();
    println!("path: {}", path);

    let fds = _get_files_and_dirs(path);
    println!("fds: {:?}", fds);
}

// get files and directories in the path
fn _get_files_and_dirs(path: &str) -> Vec<String> {
    // get path length to remove it from the path string
    let path_len = path.len();
    let mut files_and_dirs = Vec::new();

    // read files and directories from the path
    let paths = std::fs::read_dir(path).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        // remove path from the path string
        let path = &path.to_str().unwrap()[path_len..];
        files_and_dirs.push(path.to_string());
    }
    files_and_dirs
}

fn main() {
    let mut ls = LsCli::parse();
    ls.execute();
}
