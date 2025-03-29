use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;

use ftlog::{appender::udp::UdpAppender, info};
use std::time::Duration;

fn main() {
    let target_addr = "127.0.0.1:8080";
    // Create a UDP appender that sends log records to UDP socket at target address
    let udp_appender = UdpAppender::builder()
        .target(target_addr.parse().unwrap())
        .build();
    // Use channel to send exit signal to the child thread
    let (tx, rx) = mpsc::channel();
    let handler = thread::spawn(move || {
        // Create a UDP socket to receive log records
        let socket = UdpSocket::bind(target_addr).unwrap();
        let mut buf = [0; 1024];
        // Set read timeout. Receiving will time out when the log sending is complete.
        socket
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        loop {
            // Check if exit signal is received
            if let Ok(_) = rx.try_recv() {
                println!("Child thread received exit signal.");
                break;
            }
            match socket.recv_from(&mut buf) {
                Ok((size, _)) => match std::str::from_utf8(&buf[..size]) {
                    Ok(s) => print!("Received: {:}", s),
                    Err(e) => println!("Error utf-8 str: {}", e),
                },
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    println!("Read timed out.");
                }
                Err(e) => {
                    eprintln!("Error receiving data: {}", e);
                    break;
                }
            }
        }
    });
    ftlog::builder().root(udp_appender).try_init().unwrap();
    info!("Hello, world!");
    for i in 0..120 {
        info!("running {}!", i);
        std::thread::sleep(Duration::from_millis(10));
    }
    tx.send(1).unwrap();
    handler.join().unwrap();
}
