use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream, UdpSocket};
use std::thread;
use std::time::Duration;

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let addr = stream.peer_addr()?;
    println!("[TCP 服务器] 新连接: {}", addr);

    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer)? {
            0 => {
                println!("[TCP 服务器] {} 已断开", addr);
                break;
            }
            n => {
                let msg = String::from_utf8_lossy(&buffer[..n]);
                println!("[TCP 服务器] 收到: {}", msg);
                stream.write_all(&buffer[..n])?;
            }
        }
    }

    Ok(())
}

fn demo_tcp() -> io::Result<()> {
    // 绑定 127.0.0.1:0 让操作系统分配空闲端口，避免固定端口被占用。
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let server_addr = listener.local_addr()?;

    let server = thread::spawn(move || -> io::Result<()> {
        let (stream, _) = listener.accept()?;
        handle_client(stream)
    });

    let mut stream = TcpStream::connect(server_addr)?;
    let messages = ["Hello", "Rust", "TCP"];
    let mut buffer = [0; 512];

    for msg in messages {
        stream.write_all(msg.as_bytes())?;
        let n = stream.read(&mut buffer)?;
        let echo = String::from_utf8_lossy(&buffer[..n]);
        println!("[TCP 客户端] 发送: {}, 收到回声: {}", msg, echo);
    }

    stream.shutdown(Shutdown::Write)?;
    server.join().expect("TCP 服务器线程 panic")?;
    Ok(())
}

fn demo_udp() -> io::Result<()> {
    let socket_a = UdpSocket::bind("127.0.0.1:0")?;
    let socket_b = UdpSocket::bind("127.0.0.1:0")?;
    let addr_a = socket_a.local_addr()?;
    let addr_b = socket_b.local_addr()?;

    socket_a.set_read_timeout(Some(Duration::from_secs(1)))?;
    socket_b.set_read_timeout(Some(Duration::from_secs(1)))?;

    let endpoint_b = thread::spawn(move || -> io::Result<()> {
        let mut buf = [0; 1024];
        let (len, src) = socket_b.recv_from(&mut buf)?;
        let msg = String::from_utf8_lossy(&buf[..len]);
        println!("[UDP {}] 收到来自 {} 的消息: {}", addr_b, src, msg);
        socket_b.send_to(format!("Echo: {}", msg).as_bytes(), src)?;
        Ok(())
    });

    socket_a.send_to(b"Hello UDP", addr_b)?;
    let mut buf = [0; 1024];
    let (len, src) = socket_a.recv_from(&mut buf)?;
    let msg = String::from_utf8_lossy(&buf[..len]);
    println!("[UDP {}] 收到来自 {} 的消息: {}", addr_a, src, msg);

    endpoint_b.join().expect("UDP 端点线程 panic")?;
    Ok(())
}

pub fn demo() {
    println!("...............网络 IO 示例开始.................");

    if let Err(err) = demo_tcp() {
        eprintln!("TCP 演示失败: {}", err);
    }
    if let Err(err) = demo_udp() {
        eprintln!("UDP 演示失败: {}", err);
    }

    println!("...............网络 IO 示例结束.................");
}
