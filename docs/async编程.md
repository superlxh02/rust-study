# async 编程

Rust 的 async 编程由三层内容组成：

1. 语言层机制：`async fn`、`Future`、`.await`、`Waker` 和状态机。
2. 三方库体验：在已有运行时或执行器的前提下，如何写日常 async 代码。
3. 自定义 runtime：亲手实现一个最小运行时，加深对调度、唤醒和 `poll` 的理解。

初学时可以先掌握“怎么用”：在 async 函数里写 `.await`，用运行时驱动 Future。等能写出代码后，再回头理解 Future 为什么是惰性的、运行时为什么必须存在。
