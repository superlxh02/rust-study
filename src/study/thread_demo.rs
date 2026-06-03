use std::cell::RefCell;

pub fn demo() {
    println!("...............线程示例开始.................");
    //Rust的锁是一个泛型，需要包裹锁所保护对象的类型，如果没有直接传空的
    //rust想要在多个线程之间共享一个变量，需要使用Arc来包裹锁，或者使用static
    // 定义一个Mutex,并初始化为()元组，空类型的锁
    let mutex = std::sync::Arc::new(std::sync::Mutex::<()>::new(()));
    let mutttt = std::sync::Mutex::<i32>::new(1);


    let mutex1 = mutex.clone();
    //创建一个线程
    let t1 = std::thread::spawn(move ||{
        let mut lock = mutex1.lock().unwrap();


        println!("hello world from thread{:?}",std::thread::current().id());
    });

    let mutex2 = mutex.clone();
    //创建另外一个线程
    let t2 = std::thread::spawn(move|| {
        let mut lock = mutex2.lock().unwrap();
        println!("hello world from thread{:?}",std::thread::current().id());
    });
    //等待并回收两个线程
    t1.join().unwrap();
    t2.join().unwrap();
    println!("...............线程示例结束.................");
}
