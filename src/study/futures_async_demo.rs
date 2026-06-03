use futures::channel::{mpsc, oneshot};
use futures::executor::{block_on, LocalPool};
use futures::stream::StreamExt;
use futures::task::LocalSpawnExt;
use futures::join;
use std::thread;
use std::time::{Duration, Instant};

// 普通 async fn：调用它不会立刻执行函数体，而是返回一个 Future。
// 只有运行时 poll 这个 Future 时，函数体才会真正往前走。
async fn load_user_name(user_id: u32) -> String {
    println!("开始读取用户 {}", user_id);
    format!("user-{}", user_id)
}

// 在 async 函数里可以用 .await 等待另一个 Future 完成。
// 写起来像同步代码，但它在等待时可以把执行权还给运行时。
async fn build_profile(user_id: u32) -> String {
    let name = load_user_name(user_id).await;
    format!("profile({})", name)
}

// 用标准线程模拟一个“稍后才完成”的外部事件。
// futures::channel::oneshot 的 Receiver 实现了 Future：
// 发送端 send 之后，接收端 Future 会被唤醒，运行时就会继续 poll 它。
async fn delayed_task(name: &'static str, millis: u64) -> String {
    let (tx, rx) = oneshot::channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(millis));
        let _ = tx.send(format!("{} finished after {}ms", name, millis));
    });

    rx.await.expect("发送端提前关闭")
}

async fn block_on_demo() {
    let profile = build_profile(1).await;
    println!("block_on 得到结果: {}", profile);
}

async fn join_demo() {
    let start = Instant::now();

    // join! 会同时等待多个 Future。
    // 这里两个任务都先进入 Pending，随后由各自的标准线程唤醒；
    // 总耗时接近较慢的那个任务，而不是两个耗时相加。
    let (left, right) = join!(delayed_task("task-A", 120), delayed_task("task-B", 80));

    println!("join! 结果1: {}", left);
    println!("join! 结果2: {}", right);
    println!("join! 总耗时约: {:?}", start.elapsed());
}

async fn mpsc_demo() {
    let (mut tx, mut rx) = mpsc::channel::<String>(4);

    thread::spawn(move || {
        for index in 1..=3 {
            // futures 的 mpsc Sender::try_send 是非阻塞发送。
            // 真实项目里如果发送端也在 async 环境中，通常会使用 send(...).await。
            tx.try_send(format!("message-{}", index)).unwrap();
            thread::sleep(Duration::from_millis(30));
        }
    });

    while let Some(message) = rx.next().await {
        println!("mpsc 收到: {}", message);
    }
}

fn local_pool_demo() {
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    // LocalPool 是 futures 提供的单线程任务池。
    // spawn_local 把 Future 放进池子，run 会不断 poll 池中的任务直到全部完成。
    spawner
        .spawn_local(async {
            println!("LocalPool 任务1开始");
            let result = delayed_task("local-task-1", 50).await;
            println!("LocalPool 任务1结果: {}", result);
        })
        .unwrap();

    spawner
        .spawn_local(async {
            println!("LocalPool 任务2开始");
            let result = delayed_task("local-task-2", 20).await;
            println!("LocalPool 任务2结果: {}", result);
        })
        .unwrap();

    pool.run();
}

pub fn demo() {
    println!("...............futures 异步编程示例开始.................");

    // block_on 是最小化的运行时入口：把一个 Future 驱动到完成。
    block_on(block_on_demo());

    // 在一个异步主任务中体验 .await、join! 和异步通道。
    block_on(async {
        join_demo().await;
        mpsc_demo().await;
    });

    // 用 LocalPool 体验“把多个任务提交给运行时调度”。
    local_pool_demo();

    println!("...............futures 异步编程示例结束.................");
}
