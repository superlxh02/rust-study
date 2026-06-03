/*
mpsc通道基础语法
rust提供了mpsc通道，用于线程之间的通信。
mpsc -> 多生产者单消费者通道
*/

//两个子线程生产数据传递给主线程，主线程阻塞接收
pub fn demo(){
    println!("...............mpsc通道基础语法示例开始...................");
    // 创建mpsc通道,返回值是(发送者, 接收者)元组
    let (tx, rx) = std::sync::mpsc::channel();
    // 发送者支持克隆操作，多个生产者
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    let thread1 = std::thread::spawn(move || {
        tx1.send(10).unwrap();
    });
    let thread2 = std::thread::spawn(move || {
        tx2.send(100).unwrap();
    });
    let received1 = rx.recv().unwrap();
    let received2 = rx.recv().unwrap();
    println!("received1: {}", received1);
    println!("received2: {}", received2);
    thread1.join().unwrap();
    thread2.join().unwrap();
    println!("...............mpsc通道基础语法示例结束...................");
}