
mod parser;
mod command;
use std::io::Write;

use std::collections::HashMap;
use super::shell::parser::Parser;

pub trait Host {
    fn open_file(&mut self);
    fn run_tests(&mut self, filename: &str);
    fn run_test_from_file(&mut self, file_name: &str, test_name: &str);
    fn step(&mut self);
}

#[derive(Debug)]
pub enum ShellErr {
}

pub struct Shell<'a, T> where T: Host {
    host: &'a mut T,
    prompt: String,
}

impl<'a, T> Shell<'a, T> where T: Host {
    pub fn new(host_: &'a mut T) -> Self {
        Self {
            host: host_,
            prompt: String::from("(cadmus)> "),
        }
    }

    pub fn start(&mut self) -> Result<(), ShellErr> {
        self.main_loop()
    }

    fn print_prompt(&self) {
        print!("{}", self.prompt);
        std::io::stdout().flush().unwrap();
    }

    pub fn main_loop(&mut self) -> Result<(), ShellErr> {
        let mut parser = Parser {};
        loop {
            self.print_prompt();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            let command = match parser.parse(input) {
                Ok(inner) => inner,
                Err(parser::ParseErr::IncorrectNArguments(command, actual, expected)) => {
                    eprintln!("Incorrect number of args for command: {}, {}/{}", command, actual, expected);
                    continue;
                },
                Err(parser::ParseErr::UnknownCommand(command)) => {
                    eprintln!("Unknown command: {}", command);
                    continue;
                },
                Err(parser::ParseErr::EmptyLine) => continue,
            };


            match command {
                command::Command::Run { test_name, file_name } => {
                    println!("Running {} on file: {}", test_name, file_name);
                    self.host.run_test_from_file(&file_name, &test_name);
                },
                command::Command::Quit => {
                    return Ok(())
                },
                _ => {}

            }


            if input == "quit" {
                return Ok(())
            }
        }
    }
}
