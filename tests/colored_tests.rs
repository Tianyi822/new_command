#[cfg(test)]
mod tests {
    use colored::*;

    #[test]
    fn test_colored_func() {
        println!("{}", "this is blue".blue());
        println!("{}", "this is red".red());
        println!("{}", "this is red on blue".red().on_blue());
        println!("{}", "this is also red on blue".on_blue().red());
        println!("{}", "you can use truecolor values too!".truecolor(0, 255, 136));
        println!("{}", "background truecolor also works :)".on_truecolor(135, 28, 167));
        println!("{}", "bright colors are welcome as well".on_bright_blue().bright_red());
        println!("{}", "you can also make bold comments".bold());
        println!("{} {} {}", "or use".cyan(), "any".italic().yellow(), "string type".cyan());

        println!("{}", "or change advice. This is red".yellow().blue().red());
        println!("{}", "or clear things up. This is default color and style".red().bold().clear());
        println!("{}", "purple and magenta are the same".purple().magenta());
        println!("{}", "and so are normal and clear".normal().clear());

        println!("{}", format!("{:30}", "format works as expected. This will be padded".blue()));
        println!("{}", format!("{:.3}", "and this will be green but truncated to 3 chars".green()));
    }
}
