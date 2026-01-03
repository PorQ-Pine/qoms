use qoms_coms::QOMS_SOCKET_PATH;
use qoms_coms::SendToQoms;
use std::env;
use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use qoms_coms::Splash::*;

fn help() {
    eprintln!("Usage: <command> [args...]");
    eprintln!("Commands:");
    eprintln!("  send <request_type>  - Send a request enum to the data provider.");
    std::process::exit(1);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        help();
    }

    let command = &args[1];

    match command.as_str() {
        "send" => {
            if args.len() < 3 {
                help();
            }
            let request_type = &args[2];
            let request = match request_type.as_str() {
                "poweroff" => SendToQoms::RequestSplash(PowerOff),
                "reboot" => SendToQoms::RequestSplash(Reboot),
                "sleep" => SendToQoms::RequestSplash(Sleep),
                "relogin" => SendToQoms::RequestReLogin,
                _ => {
                    eprintln!("Unknown request type: {}", request_type);
                    std::process::exit(1);
                }
            };

            if let Ok(data) = postcard::to_allocvec(&request) {
                let mut stream = UnixStream::connect(QOMS_SOCKET_PATH)?;
                stream.write_all(&data)?;
                // eprintln!("Sent request: {:?}", request);
            } else {
                eprintln!("Failed to postcard");
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            help();
        }
    }

    Ok(())
}
