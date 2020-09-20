use std::net::TcpListener;

const TASKMASTER_ADDR: &str = "127.0.0.1:2121";

fn main() {
    let listener = TcpListener::bind(TASKMASTER_ADDR).unwrap();

    for stream in listener.incoming() {
        let _stream = stream.unwrap();

        println!("Conn established");
    }
}
