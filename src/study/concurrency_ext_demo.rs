use std::cell::RefCell;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

thread_local! {
    static TLS_COUNT: RefCell<u32> = RefCell::new(0);
}

fn rwlock_demo() {
    let data = Arc::new(RwLock::new(0));
    let mut handles = Vec::new();

    for index in 0..3 {
        let data = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            let guard = data.read().unwrap();
            println!("读线程 {} 看到值: {}", index, *guard);
        }));
    }

    {
        let data = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            let mut guard = data.write().unwrap();
            *guard += 10;
            println!("写线程把值加 10");
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("RwLock 最终值: {}", *data.read().unwrap());
}

fn try_lock_demo() {
    let lock = Mutex::new(0);
    let _guard = lock.lock().unwrap();

    // 文档中的重复 lock 会死锁；这里使用 try_lock 展示“锁被占用”这一状态。
    if lock.try_lock().is_err() {
        println!("try_lock 发现锁被占用，不会阻塞当前线程");
    }
}

fn timeout_try_lock_demo() {
    let lock = Mutex::new(0);
    let start = Instant::now();

    loop {
        if let Ok(mut guard) = lock.try_lock() {
            *guard += 1;
            println!("超时重试示例成功加一: {}", *guard);
            break;
        }
        if start.elapsed() > Duration::from_secs(1) {
            println!("超时放弃");
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn builder_demo() {
    let handle = thread::Builder::new()
        .name("my-thread".to_string())
        .stack_size(3 * 1024 * 1024)
        .spawn(|| {
            println!("线程名称: {:?}", thread::current().name());
            42
        })
        .unwrap();

    println!("线程返回值: {}", handle.join().unwrap());
}

fn scoped_thread_demo() {
    let mut data = vec![1, 2, 3, 4];

    thread::scope(|scope| {
        let (left, right) = data.split_at_mut(2);

        scope.spawn(move || {
            for item in left.iter_mut() {
                *item *= 2;
            }
        });

        scope.spawn(move || {
            for item in right.iter_mut() {
                *item *= 2;
            }
        });
    });

    println!("作用域线程修改后的数据: {:?}", data);
}

fn increment_tls() {
    TLS_COUNT.with(|count| {
        *count.borrow_mut() += 1;
    });
}

fn show_tls() {
    TLS_COUNT.with(|count| {
        println!("线程 {:?} 的 TLS 计数: {}", thread::current().id(), *count.borrow());
    });
}

fn thread_local_demo() {
    let t1 = thread::spawn(|| {
        increment_tls();
        increment_tls();
        show_tls();
    });
    let t2 = thread::spawn(|| {
        show_tls();
        increment_tls();
        show_tls();
    });

    t1.join().unwrap();
    t2.join().unwrap();
}

pub fn demo() {
    println!("...............并发编程扩展示例开始.................");
    rwlock_demo();
    try_lock_demo();
    timeout_try_lock_demo();
    builder_demo();
    scoped_thread_demo();
    thread_local_demo();
    println!("...............并发编程扩展示例结束.................");
}
