use core::panic;
use std::{env, process};
use env::{Args};

#[derive(Debug)]
enum MaxConnections {
    Infinite,
    Max(u32)
}

#[derive(Debug)]
struct Arguments {
    target: String,
    port: u16,
    timeout: u32,
    max_connections: MaxConnections
}

fn main() {
    let arguments: Arguments = parse_arguments(env::args());

    println!("{:?}", arguments);
}

fn parse_arguments(mut arguments: Args) -> Arguments{
    
    let mut arg_struct = Arguments {
        target: "".to_owned(),
        port: 80,
        timeout: 30,
        max_connections: MaxConnections::Infinite
    };
    
    //Skip over the filename
    arguments.next();

    loop {
        let mut option: Option<&str> = None;
        let value: String;

        let arg: String;

        match arguments.next() {
            Some(argument) => {
                arg = argument;
                if arg.starts_with("--") {

                    if arg == "--help" {
                        show_help();
                        std::process::exit(0);
                    }

                    option = Some(arg.strip_prefix("--").unwrap());
                    value = arguments.next().expect(format!("Missing value for opton {}", option.as_ref().unwrap()).as_str());
                } else if arg.starts_with("-") {
                    let flag = arg.strip_prefix("-").unwrap();
                    match flag {
                        "h" => option = Some("target"),
                        "p" => option = Some("port"),
                        "t" => option = Some("timeout"),
                        "m" => option = Some("max"),
                        _ => panic!(format!("Unknown option {}", flag).as_str())
                    }
                    value = arguments.next().expect(format!("Missing value for opton {}", option.as_ref().unwrap()).as_str());
                } else {
                    option = None;
                    value = arg;
                }
            },
            None => break
        }
        let option = option;

        match option {
            Some(option) => {
                match option {
                    "target" => arg_struct.target = value,
                    "host" => arg_struct.target = value,
                    "port" => arg_struct.port = value.parse::<u16>().unwrap(),
                    "timeout" => arg_struct.timeout = value.parse::<u32>().unwrap(),
                    "max" => arg_struct.max_connections = if value == "ininite" { MaxConnections::Infinite} else { MaxConnections::Max(value.parse::<u32>().unwrap())},
                    _ => {
                        panic!(format!("Unknown option {}", option).as_str());
                    }
                }
            },
            None => {
                if arg_struct.target == "" {
                    arg_struct.target = value;
                } else {
                    panic!(format!("Unexpected nameless argument {}", value).as_str())
                }
            }
        }

    }

    if arg_struct.target == "" {
        show_usage();
        std::process::exit(1);
    }

    arg_struct

}

fn show_usage() {
    println!("Usage: sloris [--port PORT] [--timeout TIMEOUT] [--max MAX] TARGET");
}

fn show_help() {
    println!("HELP")
}
