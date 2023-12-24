use clap::Parser;
use colored::*;

#[derive(Debug)]
struct FileInfo {
    permissions: String,
    link: u32,
    owner: String,
    group: String,
    size: u64,
    modified_time: String,
    name: String,
}

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
    //
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
    // Set status of the command
    fn set_status(&mut self) {
        // Set status to 0 by default
        self.status = 0;

        // Set status to 1 if get '-l' option
        if self.long {
            self.status |= 1;
        }

        // Set status to 2 if get '-a' option
        if self.all {
            self.status |= 2;
        }

        // Set status to 4 if get '-h' option
        if self.human_readable {
            self.status |= 4;
        }
    }

    // Get status of the command
    fn get_status(&self) -> u8 {
        self.status
    }

    // Execute the command
    pub fn execute(&mut self) {
        self.set_status();
        println!("status: {}", self.get_status());

        match self.get_status() {
            0 => self.print_files_and_dirs(),
            1 => todo!(),
            2 => todo!(),
            3 => todo!(),
            4 => todo!(),
            5 => todo!(),
            6 => todo!(),
            7 => todo!(),
            _ => self.print_files_and_dirs(),
        }
    }

    // Just print files and dirs name in the path
    fn print_files_and_dirs(&self) {
        // First check if the path is exist.
        if self.path.is_none() {
            let msg = format!("Error: path is not exist").red();
            panic!("{}", msg);
        }

        // Second check if the path is a file.
        if self.path.as_ref().unwrap().is_file() {
            // Get PathBuf of file.
            let file = self.path.as_ref().unwrap();
            // Get file metadata, such as file size, modified time, etc.
            let metadata = file.metadata().unwrap();

            let file_name = file.file_name().unwrap().to_str().unwrap();
            let msg = format!("{} {}", file_name, metadata.len()).green();
            println!("{}", msg);
        }
    }
}

fn main() {
    let mut ls = LsCli::parse();
    ls.execute();
}
