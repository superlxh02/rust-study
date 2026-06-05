
# rust线程安全底层原理：Send与Sync

## 1. 线程安全问题的本质

在多线程环境下，数据竞争（data race）是主要的安全隐患：多个线程同时访问同一内存位置，至少有一个是写操作，且没有同步机制。C/C++ 中的数据竞争属于未定义行为。Rust 在 safe Rust 中通过所有权系统和**类型系统**在编译期杜绝数据竞争；如果使用 `unsafe`，则需要程序员自行维护这些安全不变量。

### 1.1 Rust的线程安全模型核心

- **所有权转移**：通过 `move`闭包将数据所有权转移到新线程，避免共享。
- **借用规则**：要么多个不可变借用（`&T`），要么一个可变借用（`&mut T`），但不能同时存在。
- **`Send`和 `Sync` trait**：标记哪些类型可以安全地在线程间传递或共享。

## 2. `Send` trait

### 2.1 概念

`Send`是一个**标记trait**（无方法），表示**类型的值可以安全地从一个线程转移到另一个线程**。换句话说，实现了 `Send`的类型的所有权可以跨线程边界传递。

### 2.2 自动推导规则

- 绝大多数Rust类型都实现了 `Send`，包括所有基本类型（整数、浮点、bool、char）、`String`、`Vec`、`Box`等。
- 一个类型**仅当它所有字段都实现了 `Send`**时，才会自动实现 `Send`。
- 裸指针（`*const T` / `*mut T`）**不实现 `Send`**，因为它们没有所有权语义。
- `Rc<T>`（引用计数指针）**不实现 `Send`**，因为它的引用计数不是原子操作，跨线程会导致计数错误。
- `Arc<T>`实现了 `Send`（要求 `T: Send + Sync`，见下文）。这里的线程安全首先指引用计数本身安全；内部数据是否能被并发访问仍由 `T` 的 `Send`/`Sync` 决定。

### 2.3 手动实现 `Send`

`Send`是一个 `unsafe trait`，因为手动标记意味着你承诺该类型在线程间传递是安全的。通常不需要手动实现，除非自定义类型包含非 `Send`字段但你能证明它跨线程转移不会造成未定义行为。

```rust
// 示例：自定义类型包含裸指针。裸指针默认不是 Send，
// 如果手动实现 Send，必须由类型作者保证跨线程转移是安全的。
struct MyData {
    ptr: *mut u8, // 裸指针不实现Send
}

// 不安全的手动标记，需内部保证线程安全
unsafe impl Send for MyData {}
```

### 2.4 核心原理

Rust编译器在编译时检查：当你将数据所有权通过 `thread::spawn`转移到新线程时，要求闭包捕获的所有数据都必须是 `Send`的。否则编译错误。这防止了诸如 `Rc`被移动到另一个线程导致引用计数损坏。

```rust
use std::rc::Rc;
let rc = Rc::new(5);
std::thread::spawn(move || {
    println!("{}", rc); // 编译错误：`Rc<i32>` 不能在线程间安全传递
});
```

## 3. `Sync` trait

### 3.1 概念

`Sync`也是一个标记trait，表示**类型的值可以安全地在多个线程之间共享**（即通过不可变引用 `&T`被多个线程同时访问）。`Sync`是 `Send`的补充：`T`是 `Sync`当且仅当 `&T`是 `Send`。

### 3.2 自动推导规则

- 基本类型都是 `Sync`的。
- 一个类型**仅当它所有字段都是 `Sync`**时，才会自动实现 `Sync`。
- `Cell<T>`和 `RefCell<T>`**不实现 `Sync`**（因为它们的内部可变性不是线程安全的，没有使用原子操作）。
- `Mutex<T>`和 `RwLock<T>`实现了 `Sync`（要求 `T: Send`），因为它们内部使用原子操作实现同步。
- `Rc<T>`**不实现 `Sync`**（引用计数非原子）。
- `Arc<T>`实现了 `Sync`（要求 `T: Sync + Send`）。

### 3.3 手动实现 `Sync`

与 `Send`类似，`Sync`也是 `unsafe trait`，通常不需要手动实现。

### 3.4 核心原理

当你在多线程中通过不可变引用 `&T`访问数据时，编译器要求 `T: Sync`。例如，`Mutex`的 `lock`方法返回的 `MutexGuard`虽然内部有可变引用，但锁机制保证了线程安全。

```rust
use std::cell::RefCell;
let refcell = RefCell::new(5);
// 错误：`RefCell<i32>` 不能在线程间共享
std::thread::spawn(|| {
    let _ = &refcell;
});
```

## 4. `Send`和 `Sync`的关系与常见类型

| 类型          | Send | Sync | 原因                                |
| ------------- | ---- | ---- | ----------------------------------- |
| `i32`, `bool` | ✅    | ✅    | 基本类型，无共享状态                |
| `String`      | ✅    | ✅    | 堆数据，所有权唯一                  |
| `*mut T`      | ❌    | ❌    | 裸指针，无安全保证                  |
| `Rc<T>`       | ❌    | ❌    | 非原子引用计数                      |
| `Arc<T>`      | ✅    | ✅    | 引用计数是原子的；跨线程共享要求 `T: Send + Sync` |
| `RefCell<T>`  | ✅    | ❌    | 内部可变性非线程安全                |
| `Mutex<T>`    | ✅    | ✅    | 使用锁保证线程安全                  |
| `RwLock<T>`   | ✅    | ✅    | 同上                                |
| `Cell<T>`     | ✅    | ❌    | 内部可变性非线程安全                |

**注意**：`RefCell<T>`和 `Cell<T>`都实现了 `Send`（如果 `T: Send`），但**没有实现 `Sync`**。因此你可以将 `RefCell`的所有权移动到另一个线程（`Send`），但不能跨线程共享 `&RefCell`。

## 5. 内部可变性与线程安全

### 5.1 什么是内部可变性

内部可变性允许你通过不可变引用修改内部值。Rust提供了 `Cell<T>`和 `RefCell<T>`（单线程）以及 `Mutex<T>`和 `RwLock<T>`（多线程）来实现。

### 5.2 `Sync`与内部可变性的关系

- **单线程内部可变性**：`Cell<T>`/`RefCell<T>` **不实现 `Sync`**，因为它们的运行时借用检查不是原子操作；如果绕过类型系统把它们跨线程共享，就可能导致数据竞争。
- **线程安全内部可变性**：`Mutex<T>`和 `RwLock<T>`通过锁机制实现了 `Sync`，因此可以被多个线程同时共享 `&Mutex<T>`。

### 5.3 示例对比

```rust
use std::cell::RefCell;
use std::sync::Mutex;
use std::thread;

// RefCell 不能在线程间共享引用
let refcell = RefCell::new(0);
// thread::spawn(|| {
//     *refcell.borrow_mut() = 1; // 错误：`RefCell` 不是 `Sync`
// });

// Mutex 可以
let mutex = Mutex::new(0);
thread::scope(|s| {
    for _ in 0..4 {
        s.spawn(|| {
            let mut guard = mutex.lock().unwrap();
            *guard += 1;
        });
    }
});
// 编译通过，因为 Mutex 实现了 Sync
```

## 6. 编译器如何检查线程安全

Rust的 `std::thread::spawn`函数签名要求闭包捕获的数据必须满足 `Send + 'static`：

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T> 
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

- **`Send`**：闭包本身必须可安全转移给新线程（因为闭包被移动到新线程）。闭包内捕获的所有变量必须都是 `Send`。
- **`'static`**：闭包不能借用局部变量，因为新线程可能活得比当前作用域长。所以通常使用 `move`转移所有权。

编译器在调用 `spawn`时自动检查这些约束，如果捕获了 `Rc`或普通引用，会给出清晰的错误信息。

## 7. `Send`和 `Sync`的自动推导与手动标记

### 7.1 自动推导原理

Rust编译器为每个类型自动实现 `Send`和 `Sync`，基于其成员的实现。这个过程称为**自动trait**（auto trait）。例如：

```rust
struct MyStruct {
    a: i32,
    b: String,
}
// 编译器自动实现 Send 和 Sync，因为 i32 和 String 都实现了。
```

### 7.2 手动禁用 `Send`或 `Sync`

使用 `PhantomData`（零大小类型）可以“伪造”字段，从而阻止自动推导。

```rust
use std::marker::PhantomData;

struct NotSend {
    _marker: PhantomData<*mut ()>, // 裸指针使编译器不自动实现Send
}
// 现在 NotSend 既不是 Send 也不是 Sync
```

### 7.3 为什么需要手动禁用

某些类型虽然在字段层面满足 `Send/Sync`，但逻辑上不应该（例如，一个包含线程ID的类型，不应该被传递到其他线程）。通过 `PhantomData`可以手动阻止。

## 8. `MutexGuard`与 `Send`/`Sync`的特殊性

`MutexGuard<'a, T>`是一个智能指针，表示持有锁的守卫。它的实现中，`MutexGuard` **不是 `Send`**（因为锁应该在同一线程中释放）。这防止了你将锁守卫移动到另一个线程并解锁，破坏了锁的语义。

```rust
use std::sync::Mutex;
let mutex = Mutex::new(0);
let guard = mutex.lock().unwrap();
// 错误：`MutexGuard` 不能发送到其他线程
std::thread::spawn(move || {
    // 因为 guard 不是 Send
});
```

## 9. 总结：Rust线程安全的底层哲学

- **所有权 + 借用规则**：消除了数据竞争的根本可能性。
- **`Send`和 `Sync`**：编译器可检查的标记，将线程安全责任从程序员转移到类型系统。
- **无数据竞争**：在 safe Rust 中，编译器和类型系统会阻止数据竞争；如果使用 `unsafe`，则需要程序员自己维护这些安全不变量。
- **零成本抽象**：`Send`/`Sync`仅用于编译期检查，运行时无任何开销。

掌握 `Send`和 `Sync`是理解Rust并发编程进阶内容（如异步、自定义数据结构）的基础。当你设计自己的并发类型时，应当仔细考虑这些trait的实现。

**最终建议**：在99%的实践中，你不需要手动实现 `Send`/`Sync`；依赖编译器自动推导即可。只有当实现自定义并发原语或包裹裸指针时才需要深入了解。
