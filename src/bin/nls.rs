use std::{
    fmt::Debug,
    fs,
    os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt},
};

use chrono::{DateTime, Local};
use clap::Parser;
use colored::*;
use users::{get_group_by_gid, get_user_by_uid};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum FileType {
    File,
    Dir,
    Link,
    CharDevice,
    BlockDevice,
    Fifo,
    Socket,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct FileInfo {
    file_type: FileType,
    permissions: String,
    link: u64,
    owner: String,
    group: String,
    size: u64,
    modified_time: String,
    name: String,
    is_hidden: bool,
}

#[derive(Debug, Parser)]
#[command(
    author = "Tianyi",
    version = "0.0.1",
    about = "A new command line tool written in Rust"
)]
struct LsCli {
    #[arg(short = 'l', help = "show details of files and directories")]
    long: bool,

    #[arg(short = 'a', help = "show hidden files and directories")]
    all: bool,

    #[arg(short = 'H', help = "show human readable file sizes")]
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
    // 'ls -a -l'       => status-3 : calculated by 1 | 2, it will show details of all hidden files and directories
    // 'ls -H'          => status-4 : set status to 4, but do nothing, don't ask why, Linux ls command also do nothing when get '-h' option
    // 'ls -l -H'       => status-5 : calculated by 1 | 4, it will show details of files and directories with human readable file sizes
    // 'ls -a -l -H'    => status-7 : calculated by 1 | 2 | 4, it will show details of all hidden files and directories with human readable file sizes
    // other command    => status-0 : default status
    // Above status were set by the parse function what we implemented in the impl code block.
    //
    // Attention: You must use #[arg(skip)] to skip the hidden field,
    // otherwise it will be shown in help message, and even panic will appear in the program!!!
    #[arg(skip)]
    status: u8,

    // Store files and directories info that from the 'get_files_and_dirs' function.
    #[arg(skip)]
    files: Vec<FileInfo>,
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

        // Set status to 4 if get '-H' option
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
        // Get files and directories info from the target path, and store them to the vec.
        self.get_files_and_dirs();

        let _v = match self.get_status() {
            0 => self.show_names(),
            1 => self.show_infos(),
            2 => self.show_names(),
            3 => self.show_infos(),
            4 => println!("do nothing at now"),
            5 => println!("do nothing at now"),
            6 => println!("do nothing at now"),
            7 => println!("do nothing at now"),
            _ => self.show_names(),
        };
    }

    // If don't get any option or use other options that don't define, just show non-hidden files name.
    fn show_names(&self) {
        for file in self.files.iter() {
            if !self.all && file.is_hidden {
                continue;
            }

            match file.file_type {
                FileType::File => print!("{:<20}", file.name),
                FileType::Dir => print!("{:<20}", file.name),
                FileType::Link => print!("{:<20}", file.name),
                FileType::CharDevice => print!("{:<20}", file.name),
                FileType::BlockDevice => print!("{:<20}", file.name),
                FileType::Fifo => print!("{:<20}", file.name),
                FileType::Socket => print!("{:<20}", file.name),
            }
        }
        // Add a new line at the end of the output.
        println!();
    }

    // Show details of files and directories
    fn show_infos(&self) {
        for file in self.files.iter() {
            if !self.all && file.is_hidden {
                continue;
            }

            println!(
                "{} {:>3} {:>8} {:>8} {:>8} {:>20} {}",
                file.permissions,
                file.link,
                file.owner,
                file.group,
                file.size,
                file.modified_time,
                file.name
            );
        }
    }

    #[cfg(unix)]
    // Just print files and dirs name in the path
    fn get_files_and_dirs(&mut self) {
        // Check if the path is exist.
        if self.path.is_none() {
            let msg = format!("Error: path is not exist").red();
            panic!("{}", msg);
        }

        // Get PathBuf of file.
        let path_buf: &std::path::PathBuf = self.path.as_ref().unwrap();

        // Check if the path is a file.
        if !path_buf.is_dir() {
            // If it is a file, just get file info and return.
            self.files.push(self.get_file_info(path_buf));
        } else {
            // If it is a directory, get all files and directories in it.
            // And store them to the vec.
            let paths = fs::read_dir(path_buf).unwrap();
            for path in paths {
                let path = path.unwrap().path();
                self.files.push(self.get_file_info(&path));
            }
        }

        self.files.sort_by(|f1, f2| f1.name.cmp(&f2.name));
    }

    #[cfg(unix)]
    // Get file info, such as file size, modified time, etc.
    fn get_file_info(&self, path_buf: &std::path::PathBuf) -> FileInfo {
        // Get file info, such as file size, modified time, etc.
        let metadata = path_buf.metadata().unwrap();

        // Get file basic info include: permissions, type, name and is not hidden.
        let file_basic_info = self.analysis_mode(&path_buf);

        // Get file link number.
        let link_num = metadata.nlink();

        // Get modified time of file.
        let modify_time: DateTime<Local> = metadata.modified().unwrap().into();
        let modify_time = modify_time.format("%Y-%m-%d %H:%M:%S").to_string();

        // get owner and group name by uid and gid.
        let uid = metadata.uid();
        let gid = metadata.gid();

        let owner_name = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unknown".to_string());

        let group_name = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unknown".to_string());

        // Store these infos to FileInfo struct and add it to vec.
        let fi = FileInfo {
            permissions: file_basic_info.0,
            file_type: file_basic_info.1,
            link: link_num,
            owner: owner_name,
            group: group_name,
            size: metadata.len(),
            modified_time: modify_time,
            name: file_basic_info.2,
            is_hidden: file_basic_info.3,
        };

        fi
    }

    #[cfg(unix)]
    // Analysis file mode from metadata.
    fn analysis_mode(&self, path_buf: &std::path::PathBuf) -> (String, FileType, String, bool) {
        // Get file permissions.
        let metadata = path_buf.metadata().unwrap();
        let mode: u32 = metadata.permissions().mode();

        // Turn permission number to string.
        let permission_str = format!(
            "{}{}{}",
            self.turn_permission_num_to_str((mode >> 6) & 0o007),
            self.turn_permission_num_to_str((mode >> 3) & 0o007),
            self.turn_permission_num_to_str(mode & 0o007)
        );
        // Get file name.
        let file_name = path_buf.file_name().unwrap().to_string_lossy().to_string();

        // Get file type, and add it to the msg.
        let file_type = metadata.file_type();
        match file_type {
            _ if file_type.is_dir() => {
                return (
                    format!("d{permission_str}"),
                    FileType::Dir,
                    file_name.cyan().to_string(),
                    file_name.starts_with("."),
                );
            }
            _ if file_type.is_file() => {
                return (
                    format!("-{permission_str}"),
                    FileType::File,
                    file_name.white().to_string(),
                    file_name.starts_with("."),
                );
            }
            _ if file_type.is_symlink() => {
                return (
                    format!("l{permission_str}"),
                    FileType::Link,
                    file_name.blue().to_string(),
                    file_name.starts_with("."),
                );
            }
            _ if file_type.is_char_device() => {
                return (
                    format!("d{permission_str}"),
                    FileType::CharDevice,
                    file_name.yellow().to_string(),
                    file_name.starts_with("."),
                );
            }
            _ if file_type.is_block_device() => {
                return (
                    format!("b{permission_str}"),
                    FileType::BlockDevice,
                    file_name.yellow().to_string(),
                    file_name.starts_with("."),
                );
            }
            _ if file_type.is_fifo() => {
                return (
                    format!("p{permission_str}"),
                    FileType::Fifo,
                    file_name.yellow().to_string(),
                    file_name.starts_with("."),
                );
            }
            _ if file_type.is_socket() => {
                return (
                    format!("s{permission_str}"),
                    FileType::Socket,
                    file_name.yellow().to_string(),
                    file_name.starts_with("."),
                );
            }
            _ => {
                return (
                    format!("?{permission_str}"),
                    FileType::File,
                    file_name.white().to_string(),
                    file_name.starts_with("."),
                );
            }
        }
    }

    #[cfg(unix)]
    // Turn permission number to string.
    // For example: 0o755 => rwxr-xr-x
    fn turn_permission_num_to_str(&self, num: u32) -> String {
        let mut result = String::from("");

        if num & 1 == 1 {
            result.push_str("x");
        } else {
            result.push_str("-");
        }

        if num & 2 == 2 {
            result.push_str("w");
        } else {
            result.push_str("-");
        }

        if num & 4 == 4 {
            result.push_str("r");
        } else {
            result.push_str("-");
        }

        result
    }
}

fn main() {
    let mut ls = LsCli::parse();
    ls.execute();
}
