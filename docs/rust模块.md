# rust 模块

## 1. 模块基础概念

### 1.1 什么是模块

模块（module）是 Rust 中组织代码的基本单元。一个模块可以包含函数、结构体、枚举、常量、trait 甚至其他模块。模块形成树状结构，根模块是 crate 根（通常是 `main.rs` 或 `lib.rs`）。

### 1.2 定义模块 – `mod` 关键字

**功能**：声明一个模块，并可以内联模块内容或从外部文件加载。

**语法**：
```rust
// 内联模块
mod my_module {
    // 模块内容
}

// 模块可以嵌套
mod parent {
    mod child {
        // ...
    }
}
```

- 模块名使用 `snake_case` 命名规范。
- 模块内容默认私有（不可从外部访问），需要使用 `pub` 暴露。

### 1.3 将模块拆分到文件

当模块内容很长时，可以将模块放在单独的文件中。

- **传统方式（Rust 2015/2018 兼容）**：
  - 创建 `my_module.rs`，内容直接写模块内的代码。
  - 在 `main.rs` 或 `lib.rs` 中写 `mod my_module;`（注意分号，没有花括号），会自动加载同名的 `.rs` 文件。
- **现代方式（Rust 2018+ 推荐）**：
  - 在 `main.rs` 同目录下创建 `my_module.rs`，内容和上面一样。
  - 如果模块内部还有子模块，可以创建 `my_module/` 目录，并在其中放置 `mod.rs` 或使用同名文件。但 Rust 2018 引入了新的路径规则：父模块文件 `parent.rs`，子模块放在 `parent/child.rs` 中。

**示例**：
```rust
// main.rs
mod my_module;  // 寻找 my_module.rs 或 my_module/mod.rs

fn main() {
    my_module::hello();
}
```

```rust
// my_module.rs
pub fn hello() {
    println!("Hello from module!");
}
```

---

## 2. 模块引用与路径

### 2.1 绝对路径与相对路径

Rust 使用类似文件系统的路径来引用模块中的项。路径的分隔符是双冒号 `::`。

- **绝对路径**：从 crate 根开始，使用 `crate::`。
- **相对路径**：从当前模块开始，可以使用 `self::`（当前模块）或 `super::`（父模块）。

### 2.2 路径语法 – `::`

`::` 是路径分隔符，用于在不同层级之间导航。

**示例**：
```rust
mod a {
    pub mod b {
        pub fn f() {}
    }
}

fn main() {
    // 绝对路径
    crate::a::b::f();
    // 相对路径
    a::b::f();
}
```

- `crate::` 表示当前 crate 的根。
- `super::` 表示父模块。
- `self::` 表示当前模块（通常可省略）。

### 2.3 使用 `use` 引入路径

`use` 可以将路径绑定为一个短名称，避免每次书写完整路径。

**基本用法**：
```rust
use crate::a::b::f;
// 或者
use a::b::f;

fn main() {
    f(); // 直接调用，无需 a::b::f
}
```

**引入多个项**：
```rust
use std::io::{self, Read, Write}; // self 表示模块本身
use std::collections::{HashMap, HashSet};
```

**重命名**：
```rust
use std::fmt::Result as FmtResult;
```

**重新导出（`pub use`）**：将引入的项作为当前模块的公共接口暴露出去。
```rust
mod my_module {
    pub use std::collections::HashMap; // 外部可以通过 my_module::HashMap 访问
}
```

---

## 3. 访问控制（可见性）

Rust 的所有项（函数、结构体、字段、模块等）默认是**私有**的，只有同一模块或子模块可以访问。使用 `pub` 关键字可以公开。

### 3.1 `pub` – 公开可见性

**功能**：使一个项对外部模块可见。

```rust
mod outer {
    pub fn public_fn() {}      // 外部可见
    fn private_fn() {}         // 仅 outer 模块内可见

    pub mod inner {
        pub fn inner_public() {}
        fn inner_private() {}
    }
}

fn main() {
    outer::public_fn();      // 可以
    // outer::private_fn();  // 错误：私有
    outer::inner::inner_public(); // 可以（因为 inner 是 pub，且 inner_public 也是 pub）
}
```

### 3.2 结构体字段的可见性

结构体字段默认也是私有。即使结构体本身是 `pub`，字段仍需要单独 `pub` 才能从外部访问。

```rust
pub struct User {
    pub name: String,   // 公开字段
    age: u8,            // 私有字段
}

impl User {
    pub fn new(name: String, age: u8) -> Self {
        User { name, age }
    }
    pub fn age(&self) -> u8 { self.age } // 提供 getter 访问私有字段
}
```

### 3.3 枚举变体的可见性

枚举如果标记为 `pub`，其所有变体自动公开。

```rust
pub enum Color {
    Red,
    Green,
    Blue,
}
// 外部可以访问 Color::Red 等
```

### 3.4 `pub(crate)` – crate 内可见

**功能**：限制可见性仅当前 crate 内，对外部 crate 不可见。

```rust
pub(crate) fn internal_helper() {} // 整个 crate 内可访问
```

### 3.5 `pub(super)` – 父模块可见

```rust
mod parent {
    pub(super) fn visible_in_parent() {} // 仅父模块内可见
}
```

### 3.6 `pub(in path)` – 指定路径内可见

```rust
pub(in crate::some::path) fn restricted() {} // 仅在 crate::some::path 模块内可见
```

---

## 4. crate 根与执行入口

### 4.1 什么是 crate

crate 是 Rust 的编译单元。每个 crate 对应一个库或可执行文件。crate 有一个根模块，所有其他模块都挂在根下。

### 4.2 可执行 crate（binary crate）

- 根文件默认为 `main.rs`（也可以通过在 `Cargo.toml` 中指定）。
- 根模块中包含 `main` 函数，作为程序入口。
- 使用 `mod` 声明的其他模块，编译器会寻找对应文件。

### 4.3 库 crate（library crate）

- 根文件默认为 `lib.rs`。
- 没有 `main` 函数，而是对外导出 API。
- 可以被其他 crate 依赖。

### 4.4 `crate::` 根路径

在任何模块中，`crate::` 都指向当前 crate 的根模块（即 `main.rs` 或 `lib.rs` 的顶层）。

```rust
// 在任意模块中
crate::some_function();   // 调用根模块中的函数
```

### 4.5 外部 crate 的引用

使用 `extern crate`（Rust 2018 后通常不需要显式写，因为 `use` 会自动引入）。但为了兼容或特殊需求，仍可使用。

```rust
// 在 Cargo.toml 中依赖后，直接 use
use rand::Rng;
use std::collections::HashMap;  // 标准库
```

---

## 5. 模块文件组织示例

### 5.1 项目结构

假设一个项目如下：
```
my_project/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── lib.rs
    ├── graphics.rs
    ├── math/
    │   ├── mod.rs
    │   └── vector.rs
    └── utils/
        ├── mod.rs
        └── logging.rs
```

### 5.2 文件内容示例

**src/main.rs**（可执行 crate 根）：
```rust
mod graphics;       // 加载 graphics.rs
mod math;           // 加载 math/mod.rs
mod utils;          // 加载 utils/mod.rs

fn main() {
    graphics::render();
    math::add(1, 2);
    utils::logging::info("Hello");
}
```

**src/graphics.rs**：
```rust
pub fn render() {
    println!("Rendering...");
}
```

**src/math/mod.rs**：
```rust
mod vector;         // 加载 math/vector.rs
pub use vector::Vector;  // 重新导出

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**src/math/vector.rs**：
```rust
pub struct Vector {
    pub x: f64,
    pub y: f64,
}
```

**src/utils/mod.rs**：
```rust
pub mod logging;    // 加载 utils/logging.rs
```

**src/utils/logging.rs**：
```rust
pub fn info(msg: &str) {
    println!("[INFO] {}", msg);
}
```

### 5.3 路径引用示例

```rust
// 在 main.rs 中
crate::graphics::render();
crate::math::Vector { x: 1.0, y: 2.0 };
crate::utils::logging::info("start");

// 在 math/mod.rs 中
use crate::utils::logging;   // 绝对路径
super::graphics::render();   // 相对路径（父模块）
```

---

## 6. 完整示例：模块化计算器

下面是一个使用模块组织的简单计算器，展示模块定义、可见性、路径和 `use`。

### 6.1 项目结构

```
calculator/
├── Cargo.toml
└── src/
    ├── main.rs
    └── operations.rs
```

### 6.2 `src/operations.rs`

```rust
// 子模块（内联）
mod internal {
    pub fn add(a: i32, b: i32) -> i32 { a + b }
    pub fn sub(a: i32, b: i32) -> i32 { a - b }
    fn helper() { /* private */ }
}

pub use internal::{add, sub};  // 重新导出，对外公开 add 和 sub

pub fn mul(a: i32, b: i32) -> i32 {
    a * b
}

pub fn div(a: i32, b: i32) -> Option<i32> {
    if b == 0 { None } else { Some(a / b) }
}
```

### 6.3 `src/main.rs`

```rust
mod operations;  // 加载 operations.rs

use operations::{add, sub, mul, div};  // 引入这些函数

fn main() {
    let x = 10;
    let y = 5;
    println!("{} + {} = {}", x, y, add(x, y));
    println!("{} - {} = {}", x, y, sub(x, y));
    println!("{} * {} = {}", x, y, mul(x, y));
    match div(x, y) {
        Some(v) => println!("{} / {} = {}", x, y, v),
        None => println!("除数不能为零"),
    }
}
```
