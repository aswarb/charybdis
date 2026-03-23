use std::io::Read;
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;


fn handle_client(stream: UnixStream) {
    let reader = BufReader::new(&stream);
    for line in reader.lines() {
        match line {
            Ok(l) => {
                println!("received: {}", l);
            }
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = UnixListener::bind("/tmp/charybdis-io")?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(err) => {
                break;
            }
        }
    }
    Ok(())
}
