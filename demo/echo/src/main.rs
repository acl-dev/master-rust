use std::io::{Read, Write, Error};
use std::net::TcpStream;
extern crate master;

fn handle_echo(mut stream: TcpStream) ->Result<(), Error> {
    println!("incoming from {}", stream.peer_addr()?);
    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            return Ok(());
        }
        stream.write(&buf[..bytes_read])?;
    }
}

fn handle_conn(stream: TcpStream) {
    let r = handle_echo(stream);
    match r {
        Ok(_v) => {}
        Err(_e) => {}
    }
}

fn main() {
    let addrs = "127.0.0.1:8188, 127.0.0.1:8288; 127.0.0.1:8388;".to_string();
    println!("start listen {}", addrs);
    master::service::tcp_start_alone(&addrs, handle_conn);
}
