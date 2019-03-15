use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, Error};
use std::os::unix::io::FromRawFd;

fn handle_conn(mut stream: TcpStream) ->Result<(), Error> {
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

pub fn get_listeners_by_addrs(addrs: &Vec<String>) -> Vec<TcpListener> {
    let mut listeners = Vec::new();
    for addr in addrs.iter() {
        let listener = TcpListener::bind(addr).expect("coudn't bind");
        listeners.push(listener);
        println!("Listen {} ok!", addr);
    }

    listeners
}

pub fn get_listeners() -> Vec<TcpListener> {
    let mut listeners = Vec::new();
    for fd in 6..7 {
        let listener;
        unsafe {
            listener = TcpListener::from_raw_fd(fd);
        }
        listeners.push(listener);
    }

    listeners
}

fn listen_by(listener: TcpListener) {
    println!("Thread {:?} started! {:?}",
        thread::current().id(), listener.local_addr().unwrap());
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            let r = handle_conn(stream);
            match r {
                Ok(_v) => {}
                Err(_e) => {}
            }
        });
    }
}

fn start_listening(listeners: &mut Vec<TcpListener>) {
    let mut handles = Vec::new();
    loop {
        let listener = listeners.pop();
        match listener {
            Some(v) => {
                let handle = thread::spawn(move || { listen_by(v); });
                handles.push(handle);
            }
            None => break,
        }
    }

    loop {
        let handle = handles.pop();
        match handle {
            Some(v) => v.join().unwrap(),
            None =>break,
        }
    }
}

pub fn tcp_start_alone(addrs: &String) {
    let addrs = addrs.replace(" ", ";");
    let addrs = addrs.replace(",", ";");
    let addrs = addrs.replace("\t", ";");
    let addrs = addrs.split(";");
    let mut addrs_v: Vec<String> = Vec::new();

    for s in addrs {
        if s.len() > 0 {
            addrs_v.push(s.to_string());
        }
    }

    let mut listeners = get_listeners_by_addrs(&addrs_v);
    start_listening(&mut listeners);
}

pub fn tcp_start_daemon() {
    let mut listeners = get_listeners();
    start_listening(&mut listeners);
}
