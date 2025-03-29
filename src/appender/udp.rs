//! Appender that sends log messages over UDP.
//!
//! UdpAppender is a appender that sends log messages over UDP.
//! It is useful when you want to send log messages to a log server.
//!
//! ```rust
//! use ftlog::appender::udp::UdpAppender;
//! let appender = UdpAppender::builder().target("127.0.0.1:8080".parse().unwrap()).build();
//! ```

use std::io::Write;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Arc;
use typed_builder::TypedBuilder;

/// Appender log messages by UDP.
pub struct UdpAppender {
    /// UDP socket
    socket: Arc<UdpSocket>,
    /// Target address
    target: SocketAddr,
}

#[derive(TypedBuilder)]
#[builder(build_method(vis = "", name = __build_udp_appender), builder_method(vis = "pub"))]
pub struct UdpAppenderBuilder {
    target: SocketAddr,
    #[builder(default = "127.0.0.1:0".parse().unwrap())]
    bind_addr: SocketAddr,
}

#[allow(dead_code, non_camel_case_types, missing_docs)]
#[automatically_derived]
impl<__bind_addr: typed_builder::Optional<SocketAddr>>
    UdpAppenderBuilderBuilder<((SocketAddr,), __bind_addr)>
{
    pub fn build(self) -> UdpAppender {
        let builder = self.__build_udp_appender();
        // Create a UDP socket and bind to the local address
        let socket = UdpSocket::bind(&builder.bind_addr).expect("failed to bind to UDP socket");
        UdpAppender {
            socket: Arc::new(socket),
            target: builder.target,
        }
    }
}

impl UdpAppender {
    /// UdpAppender builder
    /// you can configure the remote server address and bind local address
    /// ```rust
    /// use ftlog::appender::udp::UdpAppender;
    /// let appender = UdpAppender::builder()
    ///     .target("127.0.0.1:8080".parse().unwrap())
    ///     .bind_addr("127.0.0.1:0".parse().unwrap())
    ///     .build();
    /// ```

    pub fn builder() -> UdpAppenderBuilderBuilder {
        UdpAppenderBuilder::builder()
    }

    pub fn new<T: ToSocketAddrs>(target: T) -> Self {
        Self::builder()
            .target(target.to_socket_addrs().unwrap().next().unwrap())
            .build()
    }
}

impl Write for UdpAppender {
    /// Write log record to the UDP socket
    fn write(&mut self, record: &[u8]) -> std::io::Result<usize> {
        if record.is_empty() {
            return Ok(0);
        }
        // send the record to the target
        self.socket.send_to(record, &self.target)?;
        Ok(record.len())
    }

    /// Flush always return Ok, because UDP is connectionless
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Write;
    use std::net::UdpSocket;
    use std::sync::mpsc::channel;
    use std::sync::{Arc, Mutex};
    use std::thread;
    #[test]
    fn test_udp_appender() {
        let mut appender = UdpAppender::new("127.0.0.1:8080");
        let flag = Arc::new(Mutex::new(false));
        let flag_clone = flag.clone();
        let (tx, rx) = channel();
        // create a thread to receive the record
        let handler = thread::spawn(move || {
            let socket = UdpSocket::bind("127.0.0.1:8080").unwrap();
            // socket recv
            let _ = tx.send(());
            let mut buf = [0; 1024];
            let (size, _) = socket.recv_from(&mut buf).unwrap();
            let mut f = flag_clone.lock().unwrap();
            // check the record
            *f = &buf[..size] == b"hello";
        });
        rx.recv().unwrap();
        // write record
        appender.write("hello".as_bytes()).unwrap();
        handler.join().unwrap();
        assert!(*flag.clone().lock().unwrap());
    }
}
