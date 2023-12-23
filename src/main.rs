use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    author = "Tianyi",
    version = "0.0.1",
    about = "A simple command line tool written in Rust"
)]
struct ListCli {
    #[arg(
        short = 'l',
        help = "show details of files and directories",
    )]
    long: bool,

    #[arg(
        short = 'a',
        help = "show hidden files and directories",
    )]
    all: bool,

    #[arg(
        long = "hr",
        help = "show human readable file sizes",
    )]
    human_readable: bool,

    #[arg(
        default_value = ".",
        help = "set file or directory path",
    )]
    path: Option<std::path::PathBuf>,
}

fn parse_ls_command() {
    let list_cli = ListCli::parse();
    println!("{:#?}", list_cli);

    // get path that user input
    let path = list_cli.path.unwrap();
    let path = path.to_str().unwrap();
    println!("path: {}", path);

    let fds = get_files_and_dirs(path);
    println!("fds: {:?}", fds);
}

fn get_files_and_dirs(path: &str) -> Vec<String> {
    let path_len = path.len();
    let mut files_and_dirs = Vec::new();
    let paths = std::fs::read_dir(path).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let path = &path.to_str().unwrap()[path_len..];
        files_and_dirs.push(path.to_string());
    }
    files_and_dirs
}

fn main() {
    parse_ls_command();
}
