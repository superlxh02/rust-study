# trait 特征

trait 是 Rust 中表达“某个类型具备某种能力”的核心机制。

本章分两部分：

1. trait 的基础语法：定义 trait、实现 trait、调用 trait 方法。
2. trait 用于泛型约束：用 trait bound 约束泛型参数，让泛型代码既通用又安全。

如果你熟悉其他语言，可以暂时把 trait 理解成“接口”，但 Rust 的 trait 还承担了泛型约束、静态分发、动态分发等更多职责。
