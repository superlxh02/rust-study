# 使用 futures 体验 Rust async 异步编程

前面的 async 文档已经讲过：`async fn` 会返回 `Future`，`.await` 会等待一个 `Future` 完成。但初学者最容易卡住的地方是：

> 我写了 `async`，为什么代码没有自己跑起来？

答案是：Rust 语言本身只提供 `Future`、`async/.await` 这些基础机制，不自带完整运行时。你需要一个运行时去不断 `poll` 这些 Future，把它们推进到完成。

本文用第三方库 `futures` 体验最小化的 async 编程方式。

## 1. 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
futures = "0.3"
```

`futures` 不是大型业务运行时，但它提供了很多 async 基础工具：

- `executor::block_on`：把一个 Future 跑到完成。
- `executor::LocalPool`：单线程任务池，可以提交多个 Future。
- `join!`：同时等待多个 Future。
- `channel::oneshot` / `channel::mpsc`：异步通道，体验事件唤醒。

## 2. 有运行时以后，async 代码怎么写

先看一个最简单的异步函数：

```rust
async fn load_user_name(user_id: u32) -> String {
    println!("开始读取用户 {}", user_id);
    format!("user-{}", user_id)
}
```

调用它时：

```rust
let fut = load_user_name(1);
```

这行代码只是创建了一个 `Future`，函数体还没有真正执行。要让它执行，需要运行时：

```rust
use futures::executor::block_on;

let name = block_on(load_user_name(1));
println!("{}", name);
```

`block_on` 可以理解为一个最小运行时入口。它会不断推动这个 Future，直到 Future 返回最终结果。

## 3. `.await`：在 async 函数里等待另一个任务

```rust
async fn build_profile(user_id: u32) -> String {
    let name = load_user_name(user_id).await;
    format!("profile({})", name)
}
```

这段代码看起来像同步代码：先读取用户名，再构造 profile。

但 `.await` 的特别之处在于：如果被等待的 Future 暂时没完成，当前任务可以暂停，把执行权还给运行时。运行时就有机会去推进其他任务。

## 4. `join!`：同时等待多个 Future

如果你这样写：

```rust
let a = delayed_task("task-A", 120).await;
let b = delayed_task("task-B", 80).await;
```

这是顺序等待：先等 A，再等 B。

如果两个任务互不依赖，可以用 `join!`：

```rust
use futures::join;

let (a, b) = join!(
    delayed_task("task-A", 120),
    delayed_task("task-B", 80),
);
```

`join!` 会同时等待两个 Future。只要某个 Future 暂时没有准备好，运行时可以去 poll 另一个 Future。

注意：`join!` 不是创建操作系统线程。它表达的是“这几个异步任务可以一起推进”。真正是否多线程执行，取决于你使用的运行时。

## 5. 异步通道：体验唤醒

`futures::channel::oneshot` 的接收端实现了 `Future`：

```rust
use futures::channel::oneshot;
use std::thread;
use std::time::Duration;

async fn delayed_task(name: &'static str, millis: u64) -> String {
    let (tx, rx) = oneshot::channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(millis));
        let _ = tx.send(format!("{} finished", name));
    });

    rx.await.expect("发送端提前关闭")
}
```

这段代码的过程是：

1. 创建一个 `oneshot` 通道。
2. 标准线程睡一会儿，然后发送结果。
3. async 任务在 `rx.await` 处暂停。
4. 发送端发送结果后，接收端 Future 被唤醒。
5. 运行时再次 poll 它，拿到结果并继续执行。

这就是异步运行时和 `Waker` 的实际使用感：任务不是一直占着线程等，而是暂时挂起，等事件来了再继续。

## 6. LocalPool：把多个任务提交给运行时

`block_on` 一次主要驱动一个顶层 Future。如果想体验“运行时管理多个任务”，可以使用 `LocalPool`：

```rust
use futures::executor::LocalPool;
use futures::task::LocalSpawnExt;

let mut pool = LocalPool::new();
let spawner = pool.spawner();

spawner.spawn_local(async {
    println!("任务1");
}).unwrap();

spawner.spawn_local(async {
    println!("任务2");
}).unwrap();

pool.run();
```

这里有两个角色：

- `spawner`：负责把 Future 放进任务池。
- `pool.run()`：负责不断 poll 池子里的任务，直到任务都完成。

## 7. 完整示例代码

对应代码在：

```text
src/study/futures_async_demo.rs
```

入口函数是：

```rust
pub fn demo()
```

并且已经在 `src/study/mod.rs` 中导出：

```rust
pub mod futures_async_demo;
```

可以在 `main.rs` 中调用：

```rust
study::futures_async_demo::demo();
```

## 8. 初学者要记住的模型

先记住这四句话就够了：

1. `async fn` 返回的是 `Future`，不是直接返回最终值。
2. `Future` 默认是惰性的，需要运行时推动。
3. `.await` 只能写在 async 上下文中，用来等待另一个 Future。
4. 等待期间任务可以暂停，运行时可以去推进其他任务。

如果把同步代码想象成“一个人排队逐个办事”，那么 async 更像“把等待中的事情登记好，谁准备好了就继续处理谁”。运行时就是负责登记、唤醒、调度的那个人。
