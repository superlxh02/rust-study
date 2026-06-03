use std::array;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::thread;

const EMPTY: i32 = i32::MIN;

// 一个完全使用安全 Rust 写成的教学版 SPSC 无锁队列。
//
// 为了做到完全安全，这里不存放泛型 T，也不使用未初始化内存，
// 而是把每个槽位设计成 AtomicI32，并保留 i32::MIN 作为“空槽位”哨兵值。
// 这让示例更适合学习原子操作，但也意味着它不是通用队列。
pub struct SpscQueue<const N: usize> {
    // 每个槽位都是一个原子 i32。EMPTY 表示该槽位当前没有有效数据。
    buffer: [AtomicI32; N],
    // head 指向下一个要读取的位置，只由消费者线程更新。
    head: AtomicUsize,
    // tail 指向下一个要写入的位置，只由生产者线程更新。
    tail: AtomicUsize,
}

impl<const N: usize> SpscQueue<N> {
    pub fn new() -> Self {
        assert!(N > 1, "SPSC 队列容量必须大于 1");

        Self {
            buffer: array::from_fn(|_| AtomicI32::new(EMPTY)),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    fn next(index: usize) -> usize {
        (index + 1) % N
    }

    // 实际可用容量是 N - 1。
    // 空出一个槽位后，head == tail 表示空，next_tail == head 表示满。
    pub fn capacity(&self) -> usize {
        N - 1
    }

    pub fn push(&self, value: i32) -> Result<(), i32> {
        assert_ne!(value, EMPTY, "i32::MIN 被保留为空槽位哨兵值");

        // tail 只由生产者修改，生产者读自己的 tail 用 Relaxed 即可。
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = Self::next(tail);

        // head 由消费者发布。Acquire 保证生产者能看到消费者释放的槽位。
        let head = self.head.load(Ordering::Acquire);
        if next_tail == head {
            return Err(value);
        }

        // 写入数据本身。这个写入会被下面的 tail.store(Release) 发布出去。
        self.buffer[tail].store(value, Ordering::Relaxed);

        // Release 发布新的 tail。
        // 消费者用 Acquire 读取 tail 后，一定能看到上面对槽位的写入。
        self.tail.store(next_tail, Ordering::Release);
        Ok(())
    }

    pub fn pop(&self) -> Option<i32> {
        // head 只由消费者修改，消费者读自己的 head 用 Relaxed 即可。
        let head = self.head.load(Ordering::Relaxed);

        // tail 由生产者发布。Acquire 保证消费者能看到生产者写入的数据。
        let tail = self.tail.load(Ordering::Acquire);
        if head == tail {
            return None;
        }

        let value = self.buffer[head].load(Ordering::Relaxed);
        debug_assert_ne!(value, EMPTY);

        // 把槽位重新标记为空。这个写入会被下面的 head.store(Release) 发布出去。
        self.buffer[head].store(EMPTY, Ordering::Relaxed);

        // Release 发布新的 head，让生产者知道这个槽位可以复用。
        self.head.store(Self::next(head), Ordering::Release);
        Some(value)
    }

    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire) == self.tail.load(Ordering::Acquire)
    }
}

pub fn demo() {
    println!("...............原子操作 SPSC 无锁队列示例开始.................");

    let queue = Arc::new(SpscQueue::<8>::new());
    println!("队列可用容量: {}", queue.capacity());

    let producer_queue = Arc::clone(&queue);
    let producer = thread::spawn(move || {
        for value in 0..20 {
            let mut value = value;
            loop {
                match producer_queue.push(value) {
                    Ok(()) => break,
                    Err(returned) => {
                        // 队列满了，拿回发送失败的值，稍后重试。
                        value = returned;
                        thread::yield_now();
                    }
                }
            }
        }

        // 用 -1 作为结束标记，告诉消费者可以退出。
        let mut end = -1;
        while let Err(returned) = producer_queue.push(end) {
            end = returned;
            thread::yield_now();
        }
    });

    let consumer_queue = Arc::clone(&queue);
    let consumer = thread::spawn(move || {
        let mut count = 0;
        let mut sum = 0;

        loop {
            match consumer_queue.pop() {
                Some(-1) => break,
                Some(value) => {
                    count += 1;
                    sum += value;
                    println!("consumer pop: {}", value);
                }
                None => thread::yield_now(),
            }
        }

        (count, sum)
    });

    producer.join().unwrap();
    let (count, sum) = consumer.join().unwrap();
    println!("消费者共收到 {} 个数据，求和结果 {}", count, sum);
    println!("队列是否为空: {}", queue.is_empty());

    println!("...............原子操作 SPSC 无锁队列示例结束.................");
}
