extern crate master;

fn main() {
    let addrs = "127.0.0.1:8188, 127.0.0.1:8288; 127.0.0.1:8388;".to_string();
    println!("start listen {}", addrs);
    master::service::tcp_start_alone(&addrs);
}
