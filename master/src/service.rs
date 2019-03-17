use std::thread;
use std::process;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::FromRawFd;
use std::io::{Read, Error};
use std::fs::File;

extern crate getopts;
use std::{env};
use getopts::Options;

extern crate threadpool;
use threadpool::ThreadPool;

fn get_listeners_by_addrs(addrs: &Vec<String>) -> Vec<TcpListener> {
    let mut listeners = Vec::new();
    for addr in addrs.iter() {
        let listener = TcpListener::bind(addr).expect("coudn't bind");
        listeners.push(listener);
        info!("Listen {} ok!", addr);
    }

    listeners
}

fn get_listeners() -> Vec<TcpListener> {
    let mut listeners = Vec::new();
    for fd in 6..7 {
        unsafe {
            let listener = TcpListener::from_raw_fd(fd);
            listener.set_nonblocking(false).expect("Cannot set non-blocking");
            listeners.push(listener);
        }
    }

    listeners
}

fn waiting_loop(pool: ThreadPool, listener: TcpListener, f: fn(TcpStream)) {
    info!("Thread {:?} started! {:?}",
        thread::current().id(), listener.local_addr().unwrap());

    for stream in listener.incoming() {
        info!("accept one {:?}", stream);
        let stream = stream.unwrap();
        //thread::spawn(move || { f(stream); });
        pool.execute(move || { f(stream); });
    }
}

fn start_listening(listeners: &mut Vec<TcpListener>, f: fn(TcpStream)) {

    let mut handles = Vec::new();
    let nthreads;
    unsafe {
        nthreads = NTHREADS;
    }
    let pool = ThreadPool::new(nthreads);

    loop {
        let listener = listeners.pop();
        match listener {
            Some(v) => {
                let p = pool.clone();
                let handle = thread::spawn(move || { waiting_loop(p, v, f); });
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

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

static mut CALLED: bool = false;
static mut NTHREADS: usize = 128;

fn server_init() -> Option<String> {
    unsafe {
        if CALLED {
            return None;
        }
        CALLED = true;
    }

	let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();

    opts.optopt("f", "", "set configure file name", "configure");
    opts.optopt("l", "", "set log file name", "logfile");
    opts.optopt("n", "", "service names in daemon mode", "service_names");
    opts.optopt("t", "", "service type in daemon mode", "service_type");
    opts.optopt("s", "", "listening addrs in alone mode", "addrs");
    opts.optopt("C", "", "max threads created", "nthreads");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => { m }
        Err(e) => { panic!(e.to_string()); }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        process::exit(0);
    }

    /*
    let confile;
    match matches.opt_str("f") {
        Some(x) => { confile = x; }
        None => { print_usage(&program, opts); return; }
    }
    */

    let logfile;
    match matches.opt_str("l") {
        None    => {}
        Some(x) => {
            logfile = x;
        	log4rs::init_file(logfile, Default::default()).unwrap();
        }
    }

    match matches.opt_str("C") {
        None    => {}
        Some(x) => {
            let n: isize = x.parse().unwrap();
            if n > 0 {
                unsafe { NTHREADS = n as usize; }
            }
        }
    }

    info!("daemon started!");
    return matches.opt_str("s");
}

//////////////////////////////////////////////////////////////////////////////

pub fn tcp_start_alone(addrs: &String, f: fn(TcpStream)) {
    server_init();

    info!("starting server...");
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

    info!("disconnect from master");
    process::exit(0);
}

pub fn tcp_start_daemon(f: fn(TcpStream)) {
    server_init();

    let mut listeners = get_listeners();
    if listeners.len() == 0 {
        info!("no listeners available!");
        process::exit(0);
    }

    let handle = thread::spawn(|| { monitor_master(); });
    start_listening(&mut listeners, f);
    handle.join().unwrap();
}

pub fn tcp_start(f: fn(TcpStream)) {
    let addrs = server_init();
    match addrs {
        Some(v) => { tcp_start_alone(&v, f); }
        None    => { tcp_start_daemon(f); }
    }
}
