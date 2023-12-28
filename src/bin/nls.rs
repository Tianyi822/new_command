use libc::getgrgid;
use std::{
    fmt::Debug,
    fs,
    os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt},
};

use std::ffi::CStr;

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

    #[arg(short = 'a', long = "all", help = "show hidden files and directories")]
    all: bool,

    #[arg(
        short = 'H',
        long = "human-readable",
        help = "show human readable file sizes"
    )]
    human_readable: bool,

    #[arg(default_value = ".", help = "set file or directory path")]
    path: Option<std::path::PathBuf>,

    #[arg(short = 's', long = "size", help = "sort by file size")]
    sort_by_size: bool,

    #[arg(short = 't', long = "time", help = "sort by modified time")]
    sort_by_time: bool,

    #[arg(short = 'r', long = "reverse", help = "reverse sort")]
    resort: bool,

    #[arg(
        short = 'T',
        long = "tree",
        help = "show files and directories as a tree"
    )]
    tree: bool,

    #[arg(
        short = 'd',
        long = "depth",
        help = "set the depth of the tree, default is 10",
        default_value = "10"
    )]
    depth: Option<u8>,

    // This is a hidden field，it will not be shown in help message,
    // but it can be used to store the status of the command.
    //
    // This field just like a state machine to show the status of the command,
    // and to instruct the program what to do next.
    // 'ls'                     => status-0 : default status
    // 'ls -l'                  => status-1 : show details of files and directories
    // 'ls -a'                  => status-2 : show hidden files and directories
    // 'ls -a -l'               => status-3 : calculated by 1 | 2, it will show details of all hidden files and directories
    // 'ls -H'                  => status-4 : set status to 4, but do nothing, don't ask why, Linux ls command also do nothing when get '-h' option
    // 'ls -l -H'               => status-5 : calculated by 1 | 4, it will show details of files and directories with human readable file sizes
    // 'ls -a -l -H'            => status-7 : calculated by 1 | 2 | 4, it will show details of all hidden files and directories with human readable file sizes
    // 'ls -t' of 'ls --tree'   => status-8 : show files and directories as a tree
    // other command            => status-0 : default status
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

        if self.tree {
            self.status |= 8;
        }
    }

    // Get status of the command
    fn get_status(&self) -> u8 {
        self.status
    }

    // Execute the command
    pub fn execute(&mut self) {
        // Check if the path is exist.
        if self.path.is_none() {
            let msg = format!("Error: path is not exist").red();
            panic!("{}", msg);
        } else {
            // If the path is exist, get the canonical path
            // Convert the path to an absolute path because the path may be a relative path.
            // The relative path may cause panic when use fs::PathBuf.file_name() would return None.
            self.path = Some(self.path.as_ref().unwrap().canonicalize().unwrap());
        }

        self.set_status();
        // Get files and directories info from the target path, and store them to the vec.
        self.get_files_and_dirs();

        let _v = match self.get_status() {
            0 | 2 | 4 => self.show_names(),
            1 | 3 | 5 | 7 => self.show_infos(),
            8 => self.show_as_tree(),
            _ => self.show_names(),
        };
    }

    // Show files and directories as a tree.
    fn show_as_tree(&mut self) {
        let cur_path = self.path.as_ref().unwrap();
        self.show_as_tree_recursively(cur_path, 0);
    }

    // Show files and directories as a tree recursively.
    fn show_as_tree_recursively(&self, path: &std::path::PathBuf, depth: u8) {
        if !path.exists() {
            println!(
                "{:indent$}| - {}",
                "",
                "No such file or directory".red(),
                indent = (depth * 5) as usize
            );
            return;
        }

        if depth > self.depth.unwrap() {
            return;
        }

        // Get file info.
        let file_info = self.get_file_info(path);

        // Get file name with color.
        let file_name_with_color = self.color_file_names(&file_info);

        // Print file name with color.
        println!(
            "{:indent$}| - {}",
            "",
            file_name_with_color,
            indent = (depth * 5) as usize
        );

        // If the file is a directory, get all files and directories in it.
        if file_info.file_type == FileType::Dir {
            let paths = match fs::read_dir(path) {
                Ok(paths) => paths,
                Err(_) => {
                    println!(
                        "{:indent$}| - {}",
                        "",
                        "Permission denied".red(),
                        indent = (depth * 5) as usize
                    );
                    return;
                }
            };
            for path in paths {
                let path = path.unwrap().path();
                self.show_as_tree_recursively(&path, depth + 1);
            }
        }
    }

    // If don't get any option or use other options that don't define,
    // just show non-hidden files name.
    fn show_names(&self) {
        for file in self.files.iter() {
            if !self.all && file.is_hidden {
                continue;
            }

            print!("{:<20}", self.color_file_names(&file));
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

            let size = if self.human_readable {
                self.human_readable_size(file.size)
            } else {
                file.size.to_string()
            };

            let file_name_with_color = self.color_file_names(&file);

            println!(
                "{} {:>3} {:>8} {:>8} {:>8} {:>20} {}",
                file.permissions,
                file.link,
                file.owner,
                file.group,
                size,
                file.modified_time,
                file_name_with_color
            );
        }
    }

    // Color file name by file type when show file names.
    fn color_file_names(&self, file: &FileInfo) -> ColoredString {
        match file.file_type {
            FileType::File => file.name.white(),
            FileType::Dir => file.name.cyan(),
            FileType::Link => file.name.blue(),
            FileType::CharDevice | FileType::BlockDevice | FileType::Fifo | FileType::Socket => {
                file.name.green()
            }
        }
    }

    // Turn file size to human readable size.
    fn human_readable_size(&self, size: u64) -> String {
        let mut size = size as f64;
        let mut unit = "B";

        if size > 1024.0 {
            size /= 1024.0;
            unit = "K";
        }

        if size > 1024.0 {
            size /= 1024.0;
            unit = "M";
        }

        if size > 1024.0 {
            size /= 1024.0;
            unit = "G";
        }

        if size > 1024.0 {
            size /= 1024.0;
            unit = "T";
        }

        if size > 1024.0 {
            size /= 1024.0;
            unit = "P";
        }

        format!("{:.2}{}", size, unit)
    }

    #[cfg(unix)]
    // Just print files and dirs name in the path
    fn get_files_and_dirs(&mut self) {
        // Get PathBuf of file.
        let path_buf: &std::path::PathBuf = self.path.as_ref().unwrap();

        // Check if the path is a file.
        if !path_buf.is_dir() {
            // If it is a file, just get file info and return.
            self.files.push(self.get_file_info(path_buf));
            return;
        } else {
            // If it is a directory, get all files and directories in it.
            // And store them to the vec.
            let paths = match fs::read_dir(path_buf) {
                Ok(paths) => paths,
                Err(_) => {
                    let msg = format!("Error: Permission denied").red();
                    panic!("{}", msg);
                }
            };
            for path in paths {
                let path = path.unwrap().path();
                self.files.push(self.get_file_info(&path));
            }
        }

        // Sort by option
        if self.sort_by_size {
            self.files.sort_by(|f1, f2| f1.size.cmp(&f2.size));
        } else if self.sort_by_time {
            self.files
                .sort_by(|f1, f2: &FileInfo| f1.modified_time.cmp(&f2.modified_time));
        } else {
            self.files.sort_by(|f1, f2| f1.name.cmp(&f2.name));
        }

        // Reverse sort if get '-r' option.
        if self.resort {
            self.files.reverse();
        }
    }

    #[cfg(unix)]
    // Get file info, such as file size, modified time, etc.
    fn get_file_info(&self, path_buf: &std::path::PathBuf) -> FileInfo {
        // Get file metadata, include file size, modified time, etc.
        let metadata = match fs::symlink_metadata(path_buf) {
            Ok(metadata) => metadata,
            Err(_) => path_buf.metadata().unwrap(),
        };

        // Get file basic info include: permissions, type, name and is not hidden.
        let (permission, file_type) = self.analysis_mode(&metadata);

        // Get file name and judge if it is hidden.
        let file_name = path_buf.file_name().unwrap().to_string_lossy().to_string();
        let is_hidden = file_name.starts_with(".");

        // println!("{}", format!("{} - {}", file_name, permission).red());

        // Get file link number.
        let link_num = metadata.nlink();

        // Get modified time of file.
        let modify_time: DateTime<Local> = metadata.modified().unwrap().into();
        let modify_time = modify_time.format("%Y-%m-%d %H:%M:%S").to_string();

        // Get owner and group name.
        let (owner_name, group_name) = self.get_owner_and_group_name(&metadata, &file_type);

        // Store these infos to FileInfo struct and add it to vec.
        let fi = FileInfo {
            permissions: permission,
            file_type: file_type,
            link: link_num,
            owner: owner_name,
            group: group_name,
            size: metadata.len(),
            modified_time: modify_time,
            name: file_name,
            is_hidden,
        };

        fi
    }

    // Get owner and group name.
    #[cfg(unix)]
    fn get_owner_and_group_name(
        &self,
        metadata: &fs::Metadata,
        file_type: &FileType,
    ) -> (String, String) {
        let group_name: String;

        let uid = metadata.uid();
        let gid = metadata.gid();

        // If the file type is not file, dir or link, just one way to get group name by libc.
        // It's so difficult to get group name by std::os::unix::fs::MetadataExt and users crate.
        // Because The method in the 'user crate' for converting a gid to a group name
        // can cause the program to panic due to memory alignment issues.
        // So it is necessary to use libc to call the C language implementation to accomplish this functionality.
        if file_type != &FileType::File
            || file_type != &FileType::Dir
            || file_type != &FileType::Link
        {
            // 获取用户组名
            let group_info = unsafe { getgrgid(gid) };
            group_name = if !group_info.is_null() {
                let group_name_cstr = unsafe { CStr::from_ptr((*group_info).gr_name) };
                group_name_cstr.to_string_lossy().into_owned()
            } else {
                "".to_string()
            }
        } else {
            group_name = get_group_by_gid(gid)
                .map(|g| g.name().to_string_lossy().into_owned())
                .unwrap_or_else(|| "Unknown".to_string());
        }

        let owner_name = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unknown".to_string());

        // println!("{} - {}", owner_name, group_name);

        return (owner_name, group_name);
    }

    #[cfg(unix)]
    // Analysis file mode from metadata.
    fn analysis_mode(&self, metadata: &fs::Metadata) -> (String, FileType) {
        // Get file permissions.
        let mode: u32 = metadata.permissions().mode();

        // Turn permission number to string.
        let perms_str = format!(
            "{}{}{}",
            self.turn_permission_num_to_str((mode >> 6) & 0o007),
            self.turn_permission_num_to_str((mode >> 3) & 0o007),
            self.turn_permission_num_to_str(mode & 0o007)
        );

        // Get file type, and add it to the msg.
        let file_type = metadata.file_type();
        let result = match file_type {
            _ if file_type.is_dir() => (format!("d{perms_str}"), FileType::Dir),
            _ if file_type.is_file() => (format!("-{perms_str}"), FileType::File),
            _ if file_type.is_symlink() => (format!("l{perms_str}"), FileType::Link),
            _ if file_type.is_char_device() => (format!("c{perms_str}"), FileType::CharDevice),
            _ if file_type.is_block_device() => (format!("b{perms_str}"), FileType::BlockDevice),
            _ if file_type.is_fifo() => (format!("p{perms_str}"), FileType::Fifo),
            _ if file_type.is_socket() => (format!("s{perms_str}"), FileType::Socket),
            _ => (format!("?{perms_str}"), FileType::File),
        };

        result
    }

    #[cfg(unix)]
    // Turn permission number to string.
    // For example: 0o755 => rwxr-xr-x
    fn turn_permission_num_to_str(&self, num: u32) -> String {
        let mut result = String::from("");

        if num & 4 == 4 {
            result.push_str("r");
        } else {
            result.push_str("-");
        }

        if num & 2 == 2 {
            result.push_str("w");
        } else {
            result.push_str("-");
        }

        if num & 1 == 1 {
            result.push_str("x");
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
