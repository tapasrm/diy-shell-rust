use std::{
    env,
    io::{stdin, stdout, Write},
    path::Path,
    process::{Child, Command, Stdio},
};

fn main() -> std::io::Result<()> {
    loop {
        print!("> ");
        stdout().flush().expect("clear output failed");

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        // split by pipes to support unix pipe(|) functionality.
        // must be peekable so we know we are on the the last command.
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            let mut parts: std::str::SplitWhitespace<'_> = command.trim().split_whitespace();
            // everything after the first whitespace is considered as argument.
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "cd" => {
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                }
                "exit" => return Ok(()),
                command => {
                    let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| {
                        Stdio::from(output.stdout.unwrap())
                    });

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there are no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };

                    let mut command = Command::new(command);
                    if let Ok(mut child) = command.args(args).stdin(stdin).stdout(stdout).spawn() {
                        child.wait().expect("command wasn't running");
                        previous_command = Some(child);
                        println!("Child has finished its execution!");
                    } else {
                        eprintln!("Previous Command Failed");
                        previous_command = None;
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            // block until the final command has finished
            final_command.wait().expect("Command failed");
        }
    }
}
