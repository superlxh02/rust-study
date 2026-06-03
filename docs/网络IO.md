# rust 网络I/O

## 1. TCP（面向连接）

TCP提供可靠的、面向连接的字节流传输。Rust通过 `TcpStream`（客户端/服务端连接）和 `TcpListener`（服务端监听）实现。

### 1.1 TCP 客户端 API

#### 1.1.1 连接服务器 – `TcpStream::connect`

**功能**：创建一个TCP流并连接到远程服务器。

**接口签名**：

```rust
pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<TcpStream>
```

- **参数**：`addr` – 服务器地址，如 `"127.0.0.1:8080"`或 `("localhost", 8080)`。
- **返回值**：`Result<TcpStream>` – 成功返回已连接的流对象。

**简单调用示例**：

```rust
use std::net::TcpStream;
let stream = TcpStream::connect("127.0.0.1:8080").expect("连接失败");
```

#### 1.1.2 从流中读取数据 – `Read` trait

`TcpStream`实现了 `std::io::Read` trait，可以使用 `read`、`read_to_string`等方法。

##### 1.1.2.1 `Read::read`

**功能**：从流中读取数据到缓冲区，返回读取的字节数（0表示对方关闭连接）。

**接口签名**：

```rust
fn read(&mut self, buf: &mut [u8]) -> Result<usize>
```

**简单调用示例**：

```rust
use std::io::Read;
let mut buf = [0; 1024];
let n = stream.read(&mut buf).unwrap();
```

#### 1.1.3 向流中写入数据 – `Write` trait

`TcpStream`实现了 `std::io::Write` trait，可以使用 `write`、`write_all`、`flush`等方法。

##### 1.1.3.1 `Write::write_all`

**功能**：尝试写入整个缓冲区。

**简单调用示例**：

```rust
use std::io::Write;
stream.write_all(b"Hello, server!").unwrap();
```

#### 1.1.4 关闭连接 – `shutdown` 或自动关闭

`TcpStream`在离开作用域时会自动关闭（`Drop`）。也可以手动调用 `shutdown`部分关闭。

##### 1.1.4.1 `TcpStream::shutdown`

**功能**：关闭连接的读端、写端或两端。

**接口签名**：

```rust
pub fn shutdown(&self, how: Shutdown) -> Result<()>
```

- **参数**：`how` – `Shutdown::Read`，`Shutdown::Write`或 `Shutdown::Both`。
- **返回值**：`Result<()>`。

**简单调用示例**：

```rust
use std::net::Shutdown;
stream.shutdown(Shutdown::Write).unwrap();
```

### 1.2 TCP 服务器端 API

#### 1.2.1 绑定监听地址 – `TcpListener::bind`

**功能**：创建 `TcpListener`并绑定到本地地址，开始监听。

**接口签名**：

```rust
pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<TcpListener>
```

- **参数**：`addr` – 本地地址，如 `"127.0.0.1:8080"`。
- **返回值**：`Result<TcpListener>`。

**简单调用示例**：

```rust
use std::net::TcpListener;
let listener = TcpListener::bind("127.0.0.1:8080").expect("绑定失败");
```

#### 1.2.2 接受客户端连接 – `TcpListener::accept`

**功能**：阻塞等待一个客户端的连接请求，返回 `(TcpStream, SocketAddr)`。

**接口签名**：

```rust
pub fn accept(&self) -> Result<(TcpStream, SocketAddr)>
```

- **返回值**：`Result<(TcpStream, SocketAddr)>` – 流和对方地址。

**简单调用示例**：

```rust
let (stream, addr) = listener.accept().unwrap();
println("客户端{}已连接", addr);
```

#### 1.2.3 获取监听器本地地址 – `TcpListener::local_addr`

**功能**：返回监听器的本地套接字地址。

**接口签名**：

```rust
pub fn local_addr(&self) -> Result<SocketAddr>
```

**简单调用示例**：

```rust
let addr = listener.local_addr().unwrap();
```

#### 1.2.4 迭代接入连接 – `TcpListener::incoming`

**功能**：返回一个迭代器，每次迭代接受一个新连接，返回 `Result<TcpStream>`。

**接口签名**：

```rust
pub fn incoming(&self) -> Incoming<'_>
```

- **返回值**：`Incoming`迭代器，每个元素是 `Result<TcpStream>`。

**简单调用示例**：

```rust
for stream in listener.incoming() {
    let mut stream = stream.unwrap();
    // 处理每个连接
}
```

### 1.3 TCP 客户端与服务端操作逻辑分析

在实际使用TCP时，客户端和服务端遵循典型的“请求-响应”模式。下面分别说明操作流程。

#### 1.3.1 客户端操作逻辑

1. **创建连接**：调用 `TcpStream::connect`，传入服务器IP和端口。该函数会阻塞直到与服务器建立TCP三次握手成功或超时。
2. **发送请求**：通过 `write`或 `write_all`向流中写入数据。由于TCP是字节流，可能需要多次写入才能发送完整消息（应用层需自行处理消息边界）。
3. **接收响应**：通过 `read`循环读取数据，直到满足预期长度或收到EOF（对方关闭写端）。
4. **关闭连接**：显式调用 `shutdown`或让 `stream`离开作用域自动关闭，发送FIN包。

#### 1.3.2 服务端操作逻辑

1. **创建监听器**：调用 `TcpListener::bind`绑定到一个本地端口，内核开始监听。
2. **循环接受连接**：在循环中调用 `accept`或使用 `incoming`迭代器，每个新连接产生一个新的 `TcpStream`。
3. **处理每个连接**：通常为每个客户端生成一个新线程（或使用异步任务）来处理，避免阻塞其他客户端。处理步骤：
   - 读取客户端请求数据。
   - 业务处理。
   - 发送响应数据。
4. **关闭连接**：处理完毕后，流离开作用域自动关闭，或主动调用 `shutdown`。

### 1.4 TCP 完整示例

下面实现一个简单的**回声服务器**（Echo Server）：客户端发送一行文本，服务器原样返回。

#### 1.4.1 服务端代码（`tcp_server.rs`）

```rust
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

// 处理单个客户端连接
fn handle_client(mut stream: TcpStream) {
    let addr = stream.peer_addr().unwrap();
    println!("[服务器] 新连接: {}", addr);
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                // 客户端关闭了连接
                println!("[服务器] {} 已断开", addr);
                break;
            }
            Ok(n) => {
                // 将接收到的数据原样写回
                let msg = &buffer[..n];
                println!("[服务器] 收到 {} 字节，发送回去", n);
                if let Err(e) = stream.write_all(msg) {
                    eprintln!("[服务器] 写入错误: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("[服务器] 读取错误: {}", e);
                break;
            }
        }
    }
}

pub fn run_server() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("绑定失败");
    println!("[服务器] 监听在 127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // 为每个客户端创建一个新线程
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("[服务器] 接受连接失败: {}", e);
            }
        }
    }
}
```

#### 1.4.2 客户端代码（`tcp_client.rs`）

```rust
use std::net::TcpStream;
use std::io::{Write, Read};
use std::thread::sleep;
use std::time::Duration;

pub fn run_client() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").expect("连接服务器失败");
    println!("[客户端] 已连接到服务器");

    // 发送几条消息
    let messages = vec!["Hello", "Rust", "TCP", "再见"];
    let mut buffer = [0; 512];

    for msg in messages {
        // 发送消息
        stream.write_all(msg.as_bytes()).unwrap();
        println!("[客户端] 发送: {}", msg);

        // 接收回声
        let n = stream.read(&mut buffer).unwrap();
        let echo = String::from_utf8_lossy(&buffer[..n]);
        println!("[客户端] 收到回声: {}", echo);

        sleep(Duration::from_secs(1));
    }

    // 关闭写端，通知服务器不会再发送数据
    stream.shutdown(std::net::Shutdown::Write).unwrap();
    println!("[客户端] 连接关闭");
}
```

#### 1.4.3 主函数演示

```rust
// 需要分别运行服务端和客户端，或者用两个线程演示
pub fn demo_tcp() {
    // 实际使用时请分别运行 server 和 client
    // 这里简单演示启动服务端线程和客户端线程（仅用于教学，注意顺序）
    std::thread::spawn(|| {
        run_server();
    });
    std::thread::sleep(Duration::from_millis(100));
    run_client();
}
```

## 2. UDP（无连接）

UDP提供不可靠的、无连接的数据报传输。Rust通过 `UdpSocket`实现，既可以发送也可以接收，不区分客户端/服务器。

### 2.1 绑定本地地址 – `UdpSocket::bind`

**功能**：创建一个UDP套接字并绑定到本地地址。

**接口签名**：

```rust
pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<UdpSocket>
```

- **参数**：`addr` – 本地地址，如 `"0.0.0.0:8080"`。
- **返回值**：`Result<UdpSocket>`。

**简单调用示例**：

```rust
use std::net::UdpSocket;
let socket = UdpSocket::bind("127.0.0.1:8080").expect("绑定失败");
```


### 2.2 发送数据到指定地址 – `UdpSocket::send_to`

**功能**：将数据报发送到指定的远程地址。

**接口签名**：

```rust
pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> Result<usize>
```

- **参数**：
  - `buf` – 要发送的数据。
  - `addr` – 目标地址。
- **返回值**：`Result<usize>` – 实际发送的字节数。

**简单调用示例**：

```rust
socket.send_to(b"Hello UDP", "127.0.0.1:9090").unwrap();
```

### 2.3 接收数据并获取发送方地址 – `UdpSocket::recv_from`

**功能**：阻塞等待接收一个数据报，返回接收到的字节数和发送方地址。

**接口签名**：

```rust
pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)>
```

- **参数**：`buf` – 存放数据的缓冲区。
- **返回值**：`Result<(usize, SocketAddr)>` – (字节数, 发送方地址)。

**简单调用示例**：

```rust
let mut buf = [0; 1024];
let (len, src) = socket.recv_from(&mut buf).unwrap();
println!("收到 {} 字节，来自 {}", len, src);
```

### 2.4 连接模式（可选） – `UdpSocket::connect`

**功能**：将UDP套接字“连接”到一个远程地址，之后可以使用 `send`和 `recv`（不带地址参数）。

**接口签名**：

```rust
pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> Result<()>
```

- **参数**：`addr` – 远程地址。
- **返回值**：`Result<()>`。

**简单调用示例**：

```rust
socket.connect("127.0.0.1:9090").unwrap();
socket.send(b"Data").unwrap();
```

### 2.5 UDP 完整示例：简单聊天程序

下面实现一个UDP回声程序：两个端点都绑定到 `127.0.0.1:0`，让操作系统自动分配空闲端口，然后互相发送消息并回应。这样示例不会因为固定端口被占用而失败。

#### 2.5.1 代码（`udp_echo.rs`）

```rust
use std::io;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

pub fn demo_udp() -> io::Result<()> {
    // 使用 127.0.0.1:0 让操作系统分配空闲端口，避免固定端口被占用。
    let socket_a = UdpSocket::bind("127.0.0.1:0")?;
    let socket_b = UdpSocket::bind("127.0.0.1:0")?;
    let addr_a = socket_a.local_addr()?;
    let addr_b = socket_b.local_addr()?;

    socket_a.set_read_timeout(Some(Duration::from_secs(1)))?;
    socket_b.set_read_timeout(Some(Duration::from_secs(1)))?;

    // 端点 B 收到 A 的消息后，原样加上 Echo 前缀发回去。
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
```
