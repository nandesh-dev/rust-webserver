mod request;

use request::Request;
use std::io::{Error, ErrorKind, Result};
use std::net::{TcpListener, TcpStream};

const ADDRESS: &str = "127.0.0.1:3000";

fn handle_stream(stream: TcpStream) -> Result<Request> {
    let request = Request::from_stream(&stream)?;
    Ok(request)
}

fn main() -> Result<()> {
    match TcpListener::bind(ADDRESS) {
        Ok(listener) => {
            println!("Listening on {}", ADDRESS);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        dbg!(handle_stream(stream));
                    }
                    Err(error) => {
                        println!("Connection failed {}", error)
                    }
                }
            }
        }
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Error creating tcp listener {}", e),
            ))
        }
    };

    Ok(())
}
