use std::env;
mod commands;

#[derive(Debug, PartialEq)]
enum Command {
    Init,
    Add { file: String },
    Commit { message: String },
    Checkout { commit_id: String },
    Log,
    Status,
}

impl Command {
    /// Parses raw CLI arguments into a typed command.
    ///
    /// The parser receives the full `env::args()` vector, including the binary
    /// path at index 0. Once parsed, the rest of the program can work with

    /// command variants instead of raw strings.
    fn parse_args(args: &[String]) -> Result<Command, String>{
        if args.len() < 2 {
            return Err("Not enough arguments!".to_string());
        }
        match args[1].to_lowercase().as_str() {
            "init" if args.len() == 2 => Ok(Command::Init),
            "add" if args.len() == 3 => Ok(Command::Add { file: args[2].clone() }),
            "commit" if args.len() >= 3 => Ok(Command::Commit { message: args[2..].join(" ") }),
            "checkout" if args.len() == 3 => Ok(Command::Checkout { commit_id: args[2].clone() }),
            "log" if args.len() == 2 => Ok(Command::Log),
            "status" if args.len() == 2 => Ok(Command::Status),
            "init" | "log" | "status" => Err("Command does not accept arguments".to_string()),
            "commit" => Err("Command expects a commit message".to_string()),
            "add" | "checkout" => Err("Command expects one argument".to_string()),
            _ => Err("Unknown command!".to_string()),
        }
    }   
}

/// Dispatches a parsed command to the matching command module.
/// This function serves as the main entry point for executing commands after they've been parsed.
fn run(command: &Command) -> Result<(), String> {
    match command {
        Command::Init => commands::init::init(),
        Command::Add { file } => commands::add::add(file),
        Command::Commit { message } => commands::commit::commit(message),
        Command::Checkout { commit_id } => commands::checkout::checkout(commit_id),
        Command::Log => commands::log::log(),
        Command::Status => commands::status::status(),
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
