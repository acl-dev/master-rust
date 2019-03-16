use std::thread;
use std::process;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::FromRawFd;
use std::io::{Read, Error};
use std::fs::File;

fn get_listeners_by_addrs(addrs: &Vec<String>) -> Vec<TcpListener> {
    let mut listeners = Vec::new();
    for addr in addrs.iter() {
        let listener = TcpListener::bind(addr).expect("coudn't bind");
        listeners.push(listener);
        println!("Listen {} ok!", addr);
    }

    listeners
}

fn get_listeners() -> Vec<TcpListener> {
    let mut listeners = Vec::new();
    for fd in 6..7 {
        unsafe {
            let listener = TcpListener::from_raw_fd(fd);
            listeners.push(listener);
        }
    }

    listeners
}

fn listen_by(listener: TcpListener, f: fn(TcpStream)) {
    println!("Thread {:?} started! {:?}",
        thread::current().id(), listener.local_addr().unwrap());
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || { f(stream); });
    }
}

fn start_listening(listeners: &mut Vec<TcpListener>, f: fn(TcpStream)) {
    let mut handles = Vec::new();
    loop {
        let listener = listeners.pop();
        match listener {
            Some(v) => {
                let handle = thread::spawn(move || { listen_by(v, f); });
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

pub fn tcp_start_alone(addrs: &String, f: fn(TcpStream)) {
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
    start_listening(&mut listeners, f);
}

fn wait_master(f: &mut File) -> Result<(), Error> {
    let mut buf = [0; 32];
    let _r = f.read(&mut buf)?;
    Ok(())
}

fn monitor_master() {
    let statfd = 5;

    let mut f;
    unsafe {
        f = File::from_raw_fd(statfd);
    }

    match wait_master(&mut f) {
        Ok(_v) => {}
        Err(_e) => { process::exit(1); }
    }

    println!("disconnect from master");
    process::exit(0);
}

pub fn tcp_start_daemon(f: fn(TcpStream)) {
    let mut listeners = get_listeners();
    if listeners.len() == 0 {
        println!("no listeners available!");
    }

    let handle = thread::spawn(|| { monitor_master(); });

    start_listening(&mut listeners, f);

    handle.join().unwrap();
}
