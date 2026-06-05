# 动手写一个简单 Async Runtime

> 通过实现一个极简的异步运行时，理解 Rust 异步机制的核心原理：`Future`、`Waker`、任务调度和唤醒。

## 1. 为什么需要自己实现 Runtime？

在之前的文章中，我们提到 Rust 标准库只提供了异步的**接口**（`Future` trait）和**唤醒机制**（`Waker`），但并没有提供**调度执行引擎**。`async fn` 返回的 `Future` 是惰性的，必须由运行时不断调用 `poll` 才能推进。

理解运行时内部如何工作，能帮助我们：

- 真正掌握 `Future` / `Waker` / `Context` 的协作方式；
- 理解 `tokio` 等工业级运行时的设计基础；
- 学会如何将任意异步任务适配到自定义调度器。

本文将带领你实现一个**单线程、任务队列、支持主动让出**的极简运行时。代码完全基于 `std`，不依赖任何第三方库。

## 2. 整体设计思路

我们的运行时需要实现以下功能：

1. **任务抽象**：用 `Task` 结构体包装一个 `Future`，并提供 `poll` 驱动能力。
2. **调度器**：用 `Executor` 管理一个就绪任务队列，不断从队列中取出任务执行（`poll`）。
3. **唤醒机制**：`Future` 在 `poll` 过程中通过 `Context` 使用或保存 `Waker`，当等待的事件就绪后通过 `Waker` 将任务重新放回队列。
4. **运行时入口**：提供 `spawn` 提交任务，`block_on` 阻塞运行直到根任务完成。

我们的设计采用**单线程**模型：

- 一个主线程作为工作线程，循环从队列取任务 `poll`。
- 当队列为空但还有未完成的任务时，工作线程进入 `park` 状态，等待 `Waker` 唤醒。
- `Waker` 唤醒时会把任务重新加入队列，并 `unpark` 工作线程。

任务间的切换是**协作式**的：任务在 `poll` 中返回 `Pending`（例如主动让出，或等待某个事件）后，调度器才会切换到其他就绪任务。

## 3. 核心组件与实现详解

### 3.1 Executor —— 调度器

```rust
struct Executor {
    queue: VecDeque<Arc<Task>>,
    task_count: usize,
    work_thread: Thread,
}
```

`Executor` 持有：

- `queue`：就绪任务队列（双端队列）。
- `task_count`：当前未完成的任务总数（包括队列中的和正在执行的）。
- `work_thread`：工作线程的句柄。当有新任务入队时，用于唤醒工作线程。

**为什么需要 `task_count`？**
`run()` 主循环需要知道何时退出：当没有任何未完成任务时，`block_on` 应该返回。

### 3.2 Task —— 任务抽象

```rust
struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
    is_queued: AtomicBool,
    executor: Weak<Mutex<Executor>>,
}
```

- `future`：被 `Mutex` 保护的 `Future` 对象。使用 `Option` 是因为 `poll_once` 时会将 `Future` 临时取出，`poll` 之后如果未完成则再放回。类型擦除为 `dyn Future<Output = ()> + Send + 'static`，意味着这个运行时只支持返回 `()` 的异步任务（为简化实现，`block_on` 另有处理）。
- `is_queued`：原子布尔值，标记任务是否已经在队列中，避免重复入队。
- `executor`：指向持有它的 `Executor` 的弱引用，用于在 `schedule` 时将自己放回队列。

**为什么需要 `is_queued`？**
`Waker` 可能会被多次调用（例如多个事件源同时触发），但一个任务同时只能存在于队列中一次。该标记防止重复入队造成资源浪费和混乱。

**为什么 `executor` 是弱引用？**
防止循环引用：`Task` 被 `Arc` 持有，同时 `Task` 内部又持有 `Executor` 的强引用会导致 `Executor` 永远无法释放。弱引用允许在 `Executor` 已销毁时，`schedule` 操作静默失败。

### 3.3 Task 的核心方法

#### 3.3.1 `schedule` —— 将任务放入执行队列

```rust
fn schedule(self: &Arc<Self>) {
    if self.is_queued.swap(true, Ordering::SeqCst) {
        return;
    }
    let Some(executor) = self.executor.upgrade() else { return };
    let mut executor = executor.lock().unwrap();
    executor.queue.push_back(self.clone());
    executor.work_thread.unpark();
}
```

逻辑：

1. 通过 `swap(true)` 原子地把 `is_queued` 设为 `true`，如果原值为 `true`，说明任务已在队列，直接返回。
2. 升级弱引用拿到 `Executor`（如果 `Executor` 已释放则放弃）。
3. 加锁后将任务克隆一份放入队列尾部。
4. 唤醒工作线程（`unpark`）。如果工作线程正因队列空而 `park`，它会立即醒来处理新任务。

#### 3.3.2 `poll_once` —— 单次驱动任务

```rust
fn poll_once(self: &Arc<Self>) {
    let Some(mut future) = self.future.lock().unwrap().take() else {
        return;
    };
    let waker = Waker::from(self.clone());
    let mut context = Context::from_waker(&waker);
    match future.as_mut().poll(&mut context) {
        Poll::Ready(()) => {
            if let Some(executor) = self.executor.upgrade() {
                let mut executor = executor.lock().unwrap();
                debug_assert!(executor.task_count > 0);
                executor.task_count -= 1;
            }
        }
        Poll::Pending => {
            *self.future.lock().unwrap() = Some(future);
        }
    }
}
```

详细步骤：

1. **取出 Future**：从 `self.future` 中 `take()` 取出 `Option`，得到一个 `Some(future)`。之后 `self.future` 变为 `None`。
2. **构造 Waker**：因为 `Task` 实现了 `Wake` trait，`Waker::from(self.clone())` 可以将 `Arc<Task>` 转为 `Waker`。这个 `Waker` 的 `wake()` 会调用 `Task::wake`，进而执行 `schedule`。
3. **构造 Context**：`Context` 包含 `Waker` 引用。
4. **调用 `poll`**：传入 `Pin<&mut dyn Future>` 和 `&mut Context`。
   - 如果返回 `Ready`，说明任务已完成：减少 `task_count`。注意此时 `self.future` 已经是 `None`，不再持有 Future。
   - 如果返回 `Pending`：将之前取出的 `future` 重新放回 `self.future`，以便下次 `poll_once` 再次取出。
5. **为什么 `poll` 后需要把 `future` 放回？**
   `Task` 必须持有 Future，因为任务可能被多次 `poll`（在 Pending 后等待唤醒）。如果不放回，下一次 `poll_once` 会因 `future` 为 `None` 而直接返回，任务将永远无法完成。

### 3.4 实现 `Wake` trait —— 让 Task 自身成为 Waker

```rust
impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.schedule();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.schedule();
    }
}
```

通过实现 `Wake`，我们可以用 `Waker::from(task_ref)` 轻松创建一个唤醒器。当调用 `waker.wake()` 时，实际上就是调用 `Task::wake`，从而将任务重新放回执行队列。

**这是适配的关键**：任何需要唤醒任务的地方，只要持有 `Task` 的 `Arc`，就能构造 `Waker` 并传给 `Future::poll`。

### 3.5 Runtime —— 面向用户的接口

```rust
struct Runtime {
    executor: Arc<Mutex<Executor>>,
}
```

`Runtime` 持有一个 `Executor`（用 `Arc` 和 `Mutex` 包装），提供三个方法。

#### 3.5.1 `new` —— 创建运行时

```rust
fn new() -> Arc<Self> {
    Arc::new(Self {
        executor: Arc::new(Mutex::new(Executor {
            queue: VecDeque::new(),
            task_count: 0,
            work_thread: thread::current(),
        })),
    })
}
```

注意 `work_thread` 被设置为**当前线程**（即创建 `Runtime` 的线程）。运行时的主循环会在同一个线程上执行。

#### 3.5.2 `spawn` —— 提交任务

```rust
fn spawn<F>(&self, future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    let task = Arc::new(Task {
        future: Mutex::new(Some(Box::pin(future))),
        executor: Arc::downgrade(&self.executor),
        is_queued: AtomicBool::new(true),
    });
    let mut executor = self.executor.lock().unwrap();
    executor.task_count += 1;
    executor.queue.push_back(task);
    executor.work_thread.unpark();
}
```

- 将传入的 `Future` 用 `Box::pin` 固定在堆上，再放入 `Task::future` 的 `Mutex` 中。
- 初始化 `is_queued` 为 `true` 表示创建后立刻入队（避免重复入队的检查在此无影响，因为初次入队 `is_queued` 状态应与入队同步。实际代码中入队前设为 `true` 是安全的，因为还没有任何 `schedule` 调用过。）
- 增加 `task_count`，将任务加入队列，唤醒工作线程。

#### 3.5.3 `run` —— 调度循环（核心）

```rust
fn run(&self) {
    loop {
        let task = {
            let mut executor = self.executor.lock().unwrap();
            if executor.task_count == 0 {
                return;
            }
            let task = executor.queue.pop_front();
            if let Some(task) = &task {
                task.is_queued.store(false, Ordering::SeqCst);
            }
            task
        };
        match task {
            Some(task) => task.poll_once(),
            None => {
                thread::park();
            }
        }
    }
}
```

逻辑：

1. 加锁获取 `Executor`。
2. 如果 `task_count == 0`，没有未完成任务，直接退出循环（`block_on` 即将结束）。
3. 从队列头部弹出一个任务。如果弹出成功，将该任务的 `is_queued` 设为 `false`，表示它已不在队列中。
4. 释放锁（作用域结束自动释放），然后处理任务：
   - 如果有任务，调用 `poll_once` 驱动它。
   - 如果没有任务（队列空但 `task_count > 0`），说明所有未完成任务都处于 `Pending` 状态，没有就绪任务可执行。工作线程调用 `thread::park()` 进入休眠，等待 `Waker` 唤醒。
5. 循环继续。

**为什么 `task_count > 0` 时队列也可能为空？**
当一个任务返回 `Pending` 时，它没有被放回队列（`poll_once` 中只有 `Ready` 会减少计数，`Pending` 只是把 future 放回 task，但不会重新入队）。此时任务不在队列中，但 `task_count` 仍将其计数在内。只有当某个事件触发 `Waker` 调用 `schedule` 后，任务才会重新入队。

**`park/unpark` 的协作**：

- 工作线程在无就绪任务时 `park`，释放 CPU。
- 当任务被唤醒（`schedule` 被调用）时，`unpark` 会唤醒工作线程，重新进入循环处理新入队的任务。

#### 3.5.4 `block_on` —— 阻塞运行根任务

```rust
fn block_on<F>(&self, future: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();
    self.spawn(async move {
        let value = future.await;
        *result_clone.lock().unwrap() = Some(value);
    });
    self.run();
    result.lock().unwrap().take().expect("block_on did not produce an output")
}
```

`block_on` 用于执行一个根 `Future`，并阻塞当前线程直到其完成，返回结果。实现技巧：

1. 创建一个 `Mutex<Option<Output>>` 用于存放结果。
2. 将一个包装异步任务 `spawn` 出去：该任务 `await` 传入的 `future`，将结果存入 `result_clone`。
3. 调用 `run()` 开始调度循环。当所有任务（包括这个包装任务）完成时，`task_count` 变为 `0`，`run()` 返回。
4. 从 `result` 中取出结果返回。

**为什么不直接用 `spawn` 然后 `run`？**
`run` 需要知道何时退出。如果直接 `spawn(future)`，任务完成后 `task_count` 归零，`run` 正常退出。但这里我们需要拿到 `Future` 的输出，所以用了一个额外的外层任务来捕获结果。

### 3.6 主动让出 —— `YieldNow`

为了让任务能够主动让出 CPU（模拟 I/O 等待），我们实现了一个特殊的 `Future`：`YieldNow`。

```rust
struct YieldNow {
    is_yielded: bool,
}

impl Future for YieldNow {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_yielded {
            Poll::Ready(())
        } else {
            self.is_yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn yield_now() -> YieldNow {
    YieldNow { is_yielded: false }
}
```

**行为**：

- 第一次 `poll`：`is_yielded` 为 `false`，将其设为 `true`，然后调用 `cx.waker().wake_by_ref()` **立即唤醒自己**，返回 `Pending`。
- 第二次 `poll`：`is_yielded` 为 `true`，返回 `Ready`。

**效果**：
任务在执行 `yield_now().await` 时，会**主动让出一次**，让调度器有机会运行其他就绪任务，然后在下一次循环中继续执行。这个机制模拟了异步操作中常见的“短暂让出”模式。

这里要注意 `wake_by_ref` 与 `is_queued` 的时序：

`poll_once` 中：

1. 从 `future` 中 `take()` 出来。
2. 构造 `Waker`。
3. 调用 `future.poll()`。
4. 在 `YieldNow::poll` 中，调用了 `wake_by_ref()` -> `Task::wake_by_ref` -> `schedule`。
   - 任务从队列取出后，`run` 已经执行了 `task.is_queued.store(false)`。
   - 因此进入 `poll_once` 时，`is_queued` 是 `false`。
   - `schedule` 中的 `is_queued.swap(true)` 返回 `false`，随后会把任务重新放入队列。
5. `poll` 返回 `Pending`。
6. `poll_once` 将 `future` 放回 `self.future`。
7. 退出 `poll_once`，返回到 `run` 循环。
8. 循环继续，由于队列中已经有一个任务（就是我们刚刚放回去的那个），它会立即被取出并再次 `poll_once`。此时 `YieldNow` 的 `is_yielded` 已经是 `true`，于是返回 `Ready`。

所以 `yield_now().await` 实际上会导致任务被 `poll` 两次，中间会执行一次其他任务（如果有的话）。这正是我们期望的“让出一次执行机会”的行为。

## 4. 示例代码运行流程详解

我们将通过 `demo()` 函数演示运行时如何工作。代码中包含一个 `main_task`，它内部依次执行子任务，并动态产生新任务。

### 4.1 示例任务定义

```rust
async fn sub_task(n: i32) {
    println!("task{} start", n);
    yield_now().await;
    println!("task{} done", n);
}

async fn main_task(runtime: Arc<Runtime>) {
    println!("main_task start");
    sub_task(1).await;
    runtime.spawn(sub_task(3));
    runtime.spawn(sub_task(4));
    sub_task(2).await;
    println!("main_task done");
}
```

`sub_task` 打印 start，然后 `yield_now().await` 主动让出一次，再打印 done。

`main_task` 执行顺序：

1. 打印 `main_task start`。
2. 执行 `sub_task(1).await`：完成子任务 1（它会在第一次 poll 时让出一次，第二次 poll 完成）。
3. 分别 `spawn` 子任务 3 和 4（不等待，只是提交到运行时）。
4. 执行 `sub_task(2).await`。
5. 打印 `main_task done`。

### 4.2 执行流程模拟

我们手动推演一下 `runtime.block_on(main_task(runtime.clone()))` 的执行过程。

**初始状态**：

- 通过 `block_on` 创建根任务（包装了 `main_task` 的匿名任务），入队。`task_count = 1`。
- 调用 `run()` 进入调度循环。

**循环迭代 1**：

- 从队列取出根任务，`is_queued` 设 `false`。
- `poll_once` 根任务：

  - 根任务内部执行 `main_task` 的开始。
  - 调用 `sub_task(1).await`：
    - 第一次 `poll` `sub_task(1)`：打印 `task1 start`，遇到 `yield_now().await`，调用 `yield_now` 的 `poll` → 返回 `Pending` 并唤醒自己（任务被重新入队，但注意此时根任务还没有完成，还在 `poll` 中）。
    - `sub_task(1)` 返回 `Pending`，根任务暂停，根任务的 `poll` 也返回 `Pending`。
  - 根任务返回 `Pending` 之前，会将 `sub_task(1)` 的 future 状态保存（在根任务状态机中）。
  - `poll_once` 完成：由于 `Pending`，根任务的 future 被重新放回 `Task` 中（没有减少计数）。
- 此时队列中有什么？`sub_task(1)` 是 `main_task` 中的一个 `.await`，它拿到的 `Waker` 来自根任务的 `poll` 上下文。因此，`yield_now` 调用 `cx.waker().wake_by_ref()` 时，唤醒的是根任务所在的 `Task`，而不是单独的 `sub_task(1)`。

  因此，`yield_now` 后，根任务重新放入了队列（`is_queued` 当时是 `false`，所以入队成功）。队列现在有根任务。

**循环迭代 2**：

- 取出根任务，再次 `poll_once`。
- 根任务继续执行：再次 `poll` `sub_task(1)`，这次 `sub_task(1)` 的 `yield_now` 第二次 `poll` 返回 `Ready`，`sub_task(1)` 完成。接着打印 `task1 done`。
- 根任务继续：`spawn` 任务 3 和 4（两个新任务入队，`task_count` 增加 2，变为 3）。
- 接着执行 `sub_task(2).await`：类似步骤，`sub_task(2)` 第一次 `poll` 会因 `yield_now` 而让出并唤醒根任务。
- 根任务再次返回 `Pending`，但这次 `sub_task(2)` 未完成，根任务的状态机保存其位置。
- 根任务重新入队。

同时，任务 3 和 4 也已入队。

**后续迭代**：

- 调度器会从队列中取出根任务、任务 3、任务 4 等，轮流驱动它们。
- 每个任务执行到 `yield_now` 时会让出，使其他任务有机会运行。
- 最终所有子任务完成，当 `main_task` 完成所有步骤后打印 `main_task done`，根任务返回 `Ready`，根任务被销毁，`task_count` 减到 0。
- `run()` 检测到 `task_count == 0` 退出，`block_on` 返回。

具体顺序取决于运行时队列的 FIFO 顺序和 `yield_now` 的时机，但总体展示了协作式多任务切换。

## 5. 标准库与运行时的适配契约

Rust 标准库只定义了异步的**接口**（`Future`、`Poll`、`Context`、`Waker`），而**不提供任何调度实现**。任何运行时（无论简单还是复杂）都必须遵循以下契约来适配标准库。

### 5.1 标准库提供了什么？

| 组件           | 作用                                                                | 运行时需要做什么                                               |
| -------------- | ------------------------------------------------------------------- | -------------------------------------------------------------- |
| `Future` trait | 定义异步计算单元，有一个 `poll` 方法。                              | 运行时不需要修改 `Future`，只需调用其 `poll` 方法。            |
| `Poll<T>` 枚举 | `poll` 的返回类型：`Ready(T)` 表示完成，`Pending` 表示未完成。      | 根据返回值决定任务是否继续或等待。                             |
| `Context<'a>`  | 封装了 `Waker`，在调用 `poll` 时传递给 Future。                     | 构造 `Context`，并在其中提供合适的 `Waker`。                   |
| `Waker`        | 唤醒任务的句柄，调用 `wake()` 会通知运行时该任务可以再次被 `poll`。 | 提供自定义的 `Waker`，其唤醒逻辑必须能将任务重新放入调度队列。 |
| `Wake` trait   | 简化 `Waker` 定义的辅助 trait。                                     | 实现 `Wake`，然后在需要时通过 `Waker::from` 获得 `Waker`。     |

### 5.2 运行时需要实现什么？

无论运行时的内部结构如何（单线程、多线程、工作窃取等），它都必须完成以下适配工作：

#### 5.2.1 定义任务容器（Task）

运行时需要一种方式将 `Future` 包装起来，以便能够：

- 存储 `Future`（通常需要 `Pin` 固定）。
- 提供重新调度的方法（即 `wake` 时能够将任务放回队列）。

**标准库未规定任务的具体形式**，你可以用结构体、枚举、trait 对象等。但任务通常包含：

- 一个 `Future`（可能使用 `Box::pin` 或 `pin!`）。
- 一个指向调度队列的引用（以便 `wake` 时入队）。
- 可选的状态标志（如是否已在队列中，避免重复入队）。

#### 5.2.2 实现自定义 `Waker`

`Waker` 必须能够触发任务的重新调度。标准库提供了两种方式：

- **推荐方式**：为任务类型实现 `std::task::Wake` trait，然后使用 `Waker::from(arc_task)` 构造。`Wake::wake` 中调用任务的调度函数（如 `schedule`）。
- **手动方式**：构造 `RawWaker` 和 `RawWakerVTable`，定义 `clone`、`wake`、`wake_by_ref`、`drop` 行为。

**适配关键**：`wake` 被调用时，运行时必须能够将对应的任务放回就绪队列，并（必要时）唤醒调度线程。

#### 5.2.3 提供调度循环（Executor）

调度循环负责：

- 从就绪队列中取出任务。
- 为每个任务构造 `Context`（内含正确的 `Waker`）。
- 调用任务的 `poll` 方法。
- 根据 `poll` 的返回值：
  - `Ready`：丢弃任务（或将其标记为完成，减少活跃计数）。
  - `Pending`：将任务保存，但**不自动放回队列**。任务只在被 `Waker` 唤醒时才重新入队。

**常见模式**：调度循环会维护一个“就绪队列”（存放可运行的任务）和一个“活跃计数”（记录尚未完成的任务）。当队列为空但计数 > 0 时，调度线程进入休眠（如 `thread::park()` 或条件变量等待），直到某个 `Waker` 唤醒它。

#### 5.2.4 提供任务提交接口（spawn）

运行时需要提供类似 `spawn` 的函数，允许用户将任意 `Future` 包装成任务并加入调度队列。

### 5.3 适配的核心逻辑（伪代码）

```rust
// 1. 定义任务
struct MyTask {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send>>>>,
    scheduler: Weak<MyScheduler>,   // 用于重新入队
    in_queue: AtomicBool,
}

// 2. 实现 Wake -> 唤醒时调度
impl Wake for MyTask {
    fn wake(self: Arc<Self>) {
        if !self.in_queue.swap(true, Ordering::SeqCst) {
            if let Some(sched) = self.scheduler.upgrade() {
                sched.push_back(self);
            }
        }
    }
}

// 3. 调度循环
fn run(&self) {
    loop {
        let task = self.queue.pop_front();
        let task = match task {
            Some(t) => { t.in_queue.store(false); t }
            None => { park(); continue; }
        };
        let waker = Waker::from(task.clone());
        let mut cx = Context::from_waker(&waker);
        let mut future = task.future.lock().unwrap().take().unwrap();
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(()) => { /* 完成，减少计数 */ }
            Poll::Pending => { *task.future.lock().unwrap() = Some(future); }
        }
    }
}
```

### 5.4 适配的本质

**标准库与运行时的适配点在于 `Waker` 如何与任务队列联动**。

- 标准库只要求：`poll` 时需要传入一个 `Context`，其中包含一个有效的 `Waker`；`Waker` 被调用时，运行时应能重新调度对应的任务。
- 至于如何实现任务存储、队列管理、并发控制、I/O 事件源集成，标准库不做任何假设，完全由运行时决定。

因此，**任何运行时只需保证**：

1. 它能为每个任务提供一个 `Waker`，该 `Waker` 的 `wake` 方法能够将这个任务放回运行时的调度队列。
2. 它有一个调度循环，不断从队列中取出任务，调用 `poll`，并根据 `Pending`/`Ready` 做出相应处理。

只要满足这两点，标准库提供的 `Future`、`async`/`await` 语法就能无缝适配。这就是 Rust 异步“零成本抽象”和“与运行时解耦”的根本原因。



## 6. 完整代码

```rust
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

struct Executor {
    queue: VecDeque<Arc<Task>>,
    task_count: usize,
    work_thread: Thread,
}

//任务结构体，runtime调度的单位
struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>, //类型擦除的future
    is_queued: AtomicBool,                                                     //判断是否在队列里
    executor: Weak<Mutex<Executor>>,                                           //执行器的弱引用
}

impl Task {
    //逻辑就是把自己放入队列
    fn schedule(self: &Arc<Self>) {
        // 如果之前就是 true，说明任务已经在队列里，不需要重复入队。
        if self.is_queued.swap(true, Ordering::SeqCst) {
            return;
        }

        //如果对象还存活，拿到executor
        let Some(executor) = self.executor.upgrade() else {
            return;
        };

        //加锁
        let mut executor = executor.lock().unwrap();

        //把自己放入队列
        executor.queue.push_back(self.clone());

        //唤醒线程
        executor.work_thread.unpark();
    }

    fn poll_once(self: &Arc<Self>) {
        // 从 Task 中取出 Future, self.future变为none了
        let Some(mut future) = self.future.lock().unwrap().take() else {
            return;
        };

        //基于自己构建一个waker
        let waker = Waker::from(self.clone());

        //基于waker构建一个context
        let mut context = Context::from_waker(&waker);

        //poll推进一次，如果完成，task数量-1，如果没有完成，放回task。
        match future.as_mut().poll(&mut context) {
            Poll::Ready(()) => {
                if let Some(executor) = self.executor.upgrade() {
                    let mut executor = executor.lock().unwrap();
                    debug_assert!(executor.task_count > 0);
                    executor.task_count -= 1;
                }
            }
            Poll::Pending => {
                //再把future塞到 self.future里
                *self.future.lock().unwrap() = Some(future);
            }
        }
    }
}

//为task自己实现wake trait ，让他自己能够变成waker
impl Wake for Task {
    //唤醒逻辑就是 就是再次把自己放入队列里等待被runtime poll
    fn wake(self: Arc<Self>) {
        self.schedule();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.schedule();
    }
}

//runtime持有一个executor
struct Runtime {
    executor: Arc<Mutex<Executor>>,
}

impl Runtime {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            executor: Arc::new(Mutex::new(Executor {
                queue: VecDeque::new(),
                task_count: 0,
                work_thread: thread::current(),
            })),
        })
    }

    //异步提交task
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        //基于传入的async 构造task
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(future))),
            executor: Arc::downgrade(&self.executor),
            is_queued: AtomicBool::new(true),
        });

        //拿到executor
        let mut executor = self.executor.lock().unwrap();

        executor.task_count += 1;

        //加入队列
        executor.queue.push_back(task);

        //唤醒线程
        executor.work_thread.unpark();
    }

    //阻塞运行的主逻辑-死循环，不断从队列拿出task进行poll
    fn run(&self) {
        loop {
            //从队列拿出task
            let task = {
                let mut executor = self.executor.lock().unwrap();

                if executor.task_count == 0 {
                    return;
                }

                let task = executor.queue.pop_front();

                if let Some(task) = &task {
                    // task 已经从队列里拿出来了，所以标记为“不在队列中”。
                    task.is_queued.store(false, Ordering::SeqCst);
                }

                task
            };

            match task {
                Some(task) => {
                    task.poll_once();
                }
                None => {
                    // 队列为空，但还有任务没完成。
                    // 说明有任务 Pending，runtime 需要等待某个 waker 唤醒它。
                    thread::park();
                }
            }
        }
    }

    //阻塞完成
    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        //创建空的结果
        let result = Arc::new(Mutex::new(None));
        let result_clone = result.clone();

        //异步提交，把结果放到result_clone
        self.spawn(async move {
            let value = future.await;
            *result_clone.lock().unwrap() = Some(value);
        });

        //阻塞运行
        self.run();

        //返回结果
        result
            .lock()
            .unwrap()
            .take()
            .expect("block_on did not produce an output")
    }
}

//构造一个pending的结果，让task主动挂起
struct YieldNow {
    is_yielded: bool, //是否已经主动挂起
}

impl Future for YieldNow {
    type Output = ();

    //poll的逻辑，第一次被 poll 时主动让出执行权并且调用waker让runtime下次还可以poll自己，第二次被 poll 时完成。
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_yielded {
            Poll::Ready(())
        } else {
            self.is_yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

//构造一个yieldnow结构体
fn yield_now() -> YieldNow {
    YieldNow { is_yielded: false }
}

//异步task
async fn sub_task(n: i32) {
    println!("task{} start", n);
    yield_now().await;
    println!("task{} done", n);
}

async fn main_task(runtime: Arc<Runtime>) {
    println!("main_task start");

    sub_task(1).await;

    runtime.spawn(sub_task(3));
    runtime.spawn(sub_task(4));

    sub_task(2).await;

    println!("main_task done");
}

pub fn demo() {
    println!("..............开始自定义异步运行时演示......................");
    let runtime = Runtime::new();
    runtime.block_on(main_task(runtime.clone()));
    println!("..............自定义异步运行时演示结束......................");
}


```
