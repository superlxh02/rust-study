/*
----------rust并发编程实践-生产者消费者模型实现----------------------
 */

//队列数据结构
struct Queue<T> {
    data: std::sync::Mutex<std::collections::VecDeque<T>>, //使用互斥锁包裹，基于VecDeque实现
    condvar: std::sync::Condvar,                           //条件变量
    stopped: std::sync::atomic::AtomicBool,                //协作停止的标志位
}
impl<T> Queue<T> {
    //初始化队列数据结构
    fn new() -> std::sync::Arc<Queue<T>> {
        std::sync::Arc::new(Queue::<T> {
            data: std::sync::Mutex::new(std::collections::VecDeque::new()),
            condvar: std::sync::Condvar::new(),
            stopped: std::sync::atomic::AtomicBool::new(false),
        })
    }
    //停止队列生产消费操作
    fn stop(&self) {
        //设置协作停止标志位为true
        self.stopped
            .store(true, std::sync::atomic::Ordering::Release);
        //通知所有等待的生产者和消费者线程
        self.condvar.notify_all();
    }
    //生产者线程操作：将数据压入队列
    fn push(&self, data: T) {
        let mut guard = self.data.lock().unwrap();
        if self.stopped.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        guard.push_back(data);
        self.condvar.notify_one();
    }

    fn pop(&self) -> Option<T> {
        let mut guard = self.data.lock().unwrap();
        //等待队列非空或协作停止标志位为true
        guard = self
            .condvar
            .wait_while(guard, |g| {
                g.is_empty() && !self.stopped.load(std::sync::atomic::Ordering::Acquire)
            })
            .unwrap();
        //如果队列为空且协作停止标志位为true，返回None
        if guard.is_empty() && self.stopped.load(std::sync::atomic::Ordering::Acquire) {
            return None;
        }
        //从队列头部弹出数据返回
        guard.pop_front()
    }
}

pub fn demo() {
    println!("...............生产者消费者模型示例开始...................");
    let queue = Queue::<i32>::new();
    let produce_queue = queue.clone();
    let produce = std::thread::spawn(move || {
        let mut count = 0;
        while count < 11 {
            produce_queue.push(count);
            count += 1;
        }
        produce_queue.stop();
    });
    let consume_queue1 = queue.clone();
    let consume1 = std::thread::spawn(move || {
        while let Some(data) = consume_queue1.pop() {
            println!("consumer1 consume data: {}", data);
        }
    });
    let consume_queue2 = queue.clone();
    let consume2 = std::thread::spawn(move || {
        while let Some(data) = consume_queue2.pop() {
            println!("consumer2 consume data: {}", data);
        }
    });

    produce.join().unwrap();
    consume1.join().unwrap();
    consume2.join().unwrap();
    println!("...............生产者消费者模型示例结束...................");
}
