use std::env;
mod commands;

#[derive(Debug, PartialEq)]
enum Command {
    Init,
    Add { file: String },
    Commit,
    Checkout { commit_id: String },
}

impl Command {
    fn parse_args(args: &[String]) -> Result<Command, String>{
        if args.len() < 2 {
            return Err("Not enough arguments!".to_string());
        }
        match args[1].to_lowercase().as_str() {
            "init" if args.len() == 2 => Ok(Command::Init),
            "add" if args.len() == 3 => Ok(Command::Add { file: args[2].clone() }),
            "commit" if args.len() == 2 => Ok(Command::Commit),
            "checkout" if args.len() == 3 => Ok(Command::Checkout { commit_id: args[2].clone() }),
            "init" | "commit" => Err("Command does not accept arguments".to_string()),
            "add" | "checkout" => Err("Command expects one argument".to_string()),
            _ => Err("Unknown command!".to_string()),
        }
    }   
}

fn run(command: &Command) -> Result<(), String> {
    match command {
        Command::Init => commands::init::init(),
        Command::Add { file } => commands::add::add(file),
        Command::Commit => commands::commit::commit(),
        Command::Checkout { commit_id } => commands::checkout::checkout(commit_id),
    }
}

fn main() -> Result<(), String>{
    let args: Vec<String> = env::args().collect();
    let config = Command::parse_args(&args)?;
    run(&config)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_args() {
        let args = vec!["target/debug/ferrit.exe".to_string(), "init".to_string()];
        let command = Command::parse_args(&args).unwrap();
        assert!(matches!(command, Command::Init));
    } 

    #[test]
    fn test_not_enough_arguments() {
        let args = vec!["target/debug/ferrit.exe".to_string()];
        let result = Command::parse_args(&args);
        assert!(result.is_err());
    }   
}
