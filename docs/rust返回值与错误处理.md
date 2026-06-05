# rust 返回值与错误处理

## 1. Rust 的错误处理概述

### 1.1 传统 OOP 语言的异常机制（以 Java 为例）

在 Java 中，方法可以“抛出”异常。调用者要么捕获（`try-catch`），要么继续向上抛出。

```java
// Java 示例
public String readFile(String path) throws IOException {
    // 可能抛出 IOException
    return Files.readString(Path.of(path));
}

public void process() {
    try {
        String content = readFile("data.txt");
        System.out.println(content);
    } catch (IOException e) {
        System.err.println("读取失败：" + e.getMessage());
    }
}
```

**问题：**

- 异常控制流不透明：一个方法可能悄悄抛出几十种异常，但签名只声明了一部分（如 `throws`）。
- 性能开销：异常展开（unwinding）成本较高，尤其在不该用异常控制的普通逻辑中（如“用户未找到”）。
- 容易被忽略：不强制处理，程序员可能忘记 `catch` 或记录。

### 1.2 Rust 的错误处理哲学

Rust 没有异常机制。它遵循以下设计原则：

1. **错误也是值** —— 用普通枚举类型（`Option` 和 `Result`）表示可能缺失或失败的结果。
2. **显式优于隐式** —— 函数的返回值类型必须明确表达“可能出错”或“可能无值”，调用者必须处理这些可能性。
3. **区分可恢复错误与不可恢复错误**
   - 可恢复：如文件不存在、数字解析失败，用 `Result` 处理。
   - 不可恢复：如数组越界、违反断言、程序进入非法状态，用 `panic!` 崩溃并清理。
4. **低开销且显式** —— `Option` 和 `Result` 是普通枚举，很多场景会被编译器优化得非常紧凑（例如利用 niche optimization），但具体布局和成本取决于类型；它们的核心优势是把错误路径显式放进类型系统。

### 1.3 Rust 的错误处理方式一览

| 机制                    | 用途                                                                  |
| ----------------------- | --------------------------------------------------------------------- |
| `panic!`              | 不可恢复错误，程序直接退出（或通过 `catch_unwind` 捕获）。          |
| `Option<T>`           | 值可能缺失（有或无），无错误细节。例如：从哈希表取值。                |
| `Result<T, E>`        | 操作可能失败（成功值 T 或错误值 E），可携带错误信息。                 |
| `match`               | 模式匹配，用于解构 `Option` 或 `Result` 并分支处理。              |
| `?` 操作符            | 简化 `Result` 或 `Option` 的传递，失败时提前返回。                |
| `unwrap` / `expect` | 快速取出成功值，若失败则 `panic!`（用于原型或确信不会出错的场景）。 |

本教程重点讲解 `match`、`Option`、`Result` 和 `panic!`，`?` 操作符会在进阶内容中学习。

---

## 2. match —— 强大的模式匹配工具

`match` 是 Rust 的控制流结构，用于将一个值与多个模式逐一比较，并执行第一个匹配的模式对应的代码。它类似其他语言的 `switch`，但强大得多。

### 2.1 基本语法结构

```rust
match 值 {
    模式1 => 表达式1,
    模式2 => 表达式2,
    ...
    _ => 默认表达式,   // 下划线匹配所有情况
}
```

**要点：**

- 所有可能情况必须被覆盖（编译器会检查完整性）。
- 每个 `=>` 右边可以是一个代码块（用 `{}` 包裹）。
- 默认分支用 `_` 表示，相当于 `default`。
- `match` 本身是一个表达式，可以返回值。

### 2.2 返回值与示例

`match` 的每个分支必须返回相同类型的值（除非分支是 `{}` 块且块最后有返回值，或使用 `break` / `return` 等跳转）。

**示例1：匹配数字**

```rust
let x = 3;
let description = match x {
    1 => "one",
    2 => "two",
    3 => "three",
    _ => "other",
};
println!("{}", description); // 输出: three
```

**示例2：匹配枚举（即将用到的 `Option` 和 `Result`）**

```rust
let some_value = Some(42);
match some_value {
    Some(value) => println!("Got {}", value),
    None => println!("Got nothing"),
}
```

**示例3：带块的分支**

```rust
let num = 10;
let result = match num {
    0 => {
        println!("zero");
        0
    }
    n if n > 0 => {
        println!("positive");
        n * 2
    }
    _ => {
        println!("negative");
        -1
    }
};
// 输出: positive
// result = 20
```

---

## 3. Option `<T>` —— 表示值可能存在或缺失

### 3.1 定义和内在含义

`Option<T>` 是 Rust 标准库内置枚举，定义如下：

```rust
enum Option<T> {
    Some(T),  // 存在一个类型为 T 的值
    None,     // 没有值
}
```

它解决了一个常见问题：**如何安全地表示“可能没有值”**？其他语言常用 `null` 或 `nullptr`，但 Tony Hoare 称 `null` 引用是他的“十亿美元错误”。Rust 彻底抛弃了 `null`，用 `Option` 强制你处理缺失情况。

### 3.2 使用场景

- 从哈希表 `get` 一个键（可能不存在）。
- 查找字符串中是否包含某个子串的位置（可能没有）。
- 链表或树结构的后继节点（可能为空）。
- 将 `T` 转换为 `Option<T>` 表示该值可能不存在。

### 3.3 示例

**创建 `Option`：**

```rust
let some_number = Some(5);      // 类型推断为 Option<i32>
let some_string = Some("hello".to_string());
let absent_number: Option<i32> = None;  // 必须标注类型，因为 None 无法推断
```

**安全地取出值（配合 `match`）：**

```rust
fn divide(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator == 0.0 {
        None
    } else {
        Some(numerator / denominator)
    }
}

fn main() {
    let result = divide(10.0, 2.0);
    match result {
        Some(value) => println!("结果: {}", value),
        None => println!("除数不能为零"),
    }
}
```

**常用方法：**

- `unwrap()`：取出 `Some` 的值，若为 `None` 则 `panic!`。
- `expect(msg)`：类似 `unwrap`，但可自定义 `panic` 消息。
- `unwrap_or(default)`：取出值或返回默认值。
- `is_some()` / `is_none()`：判断。

```rust
let x: Option<i32> = None;
let y = x.unwrap_or(100);   // y = 100
```

### 3.4 与 `match` 的密切关系

几乎每次使用 `Option` 都会涉及 `match`（或 `if let` / `?`），因为编译器强制你处理 `None` 分支。

---

## 4. Result<T, E> —— 表示操作可能成功或失败

### 4.1 定义和内在含义

`Result<T, E>` 是另一个内置枚举，用于可能失败的操作，并携带错误信息：

```rust
enum Result<T, E> {
    Ok(T),   // 成功，包含值 T
    Err(E),  // 失败，包含错误 E
}
```

与 `Option` 的区别：`Option` 只关心“有没有”，`Result` 关心“为什么会失败”。`E` 可以是任何类型，通常用标准库的 `String`、自定义错误枚举或 `std::io::Error` 等。

### 4.2 使用场景

- 文件打开、读取、写入。
- 网络请求。
- 字符串解析为数字（`"42".parse::<i32>()` 返回 `Result<i32, ParseIntError>`）。
- 数据库查询。

### 4.3 示例

**读取文件并返回 `Result`：**

```rust
use std::fs::File;
use std::io::Read;

fn read_username_from_file(path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(path);  // 返回 Result<File, std::io::Error>
    let mut file = match file {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    let mut username = String::new();
    match file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
}
```

这样写很冗长，稍后会介绍 `?` 简化。但这里完美展示了 `match` 处理 `Result` 的方式。

**解析数字：**

```rust
fn parse_number(s: &str) -> Result<i32, std::num::ParseIntError> {
    s.parse()
}

fn main() {
    let result = parse_number("42a");
    match result {
        Ok(num) => println!("数字是 {}", num),
        Err(e) => println!("解析错误: {}", e),
    }
}
```

**常用方法：**

- `unwrap()`：成功返回 `T`，失败 `panic!`。
- `expect(msg)`：类似。
- `unwrap_or(default)`：失败返回默认值（注意此时丢失错误信息）。
- `is_ok()` / `is_err()`：判断。
- `ok()`：将 `Result<T,E>` 转为 `Option<T>`（丢弃错误）。
- `err()`：将 `Result<T,E>` 转为 `Option<E>`（丢弃成功值）。

---

## 5. Option 和 Result 的表达语义区别

很多初学者会混淆两者，事实上它们表达的是**完全不同层次的概念**。下面从四个维度对比。

| 维度                       | Option `<T>`                                             | Result<T, E>                                             |
| -------------------------- | ---------------------------------------------------------- | -------------------------------------------------------- |
| **核心语义**         | **值可能存在也可能不存在**。强调“有无”。           | **操作可能成功也可能失败**。强调“成败”并附原因。 |
| **错误信息**         | 无。`None` 只表示缺失，不解释为什么。                    | 有。`Err(e)` 携带错误类型 `E`，可包含详细信息。      |
| **典型场景**         | - 从容器中查找元素 `<br>`- 可选配置项 `<br>`- 链表末尾 | - 文件 I/O `<br>`- 网络请求 `<br>`- 解析字符串       |
| **合并错误处理**     | 不适用，因为没有错误细节。                                 | 可链式传递错误，例如使用 `?` 将 `Err` 向上传播。     |
| **是否包含失败原因** | 否                                                         | 是                                                       |

### 5.1 实例对比：查找 vs 解析

**使用 `Option`（查找）**

```rust
let map = std::collections::HashMap::from([("key", 42)]);
let value = map.get("key");      // Option<&i32>，可能为 None
match value {
    Some(v) => println!("找到: {}", v),
    None    => println!("没有这个键"),
}
```

如果键不存在，你不需要知道为什么（比如是用户输入错误还是内部逻辑问题），只需要知道“没有”。这就是 `Option` 的合适场景。

**使用 `Result`（解析）**

```rust
let input = "42a";
let number: Result<i32, _> = input.parse();   // Result<i32, ParseIntError>
match number {
    Ok(n) => println!("解析成功: {}", n),
    Err(e) => println!("解析失败，原因: {}", e),
}
```

解析可能因为格式不对而失败，你需要知道**失败的具体原因**（例如包含了非数字字符），以便向用户报告或重试。这就是 `Result` 的场景。

### 5.2 相互转换

你可以根据需要在两者间转换：

- `Option::ok_or(E)` 将 `Option<T>` 转为 `Result<T, E>`（`None` 变为 `Err(E)`）。
- `Result::ok()` 将 `Result<T,E>` 转为 `Option<T>`（丢弃错误）。
- `Result::err()` 转为 `Option<E>`（丢弃成功值）。

```rust
let opt = Some(42);
let res = opt.ok_or("缺失");   // Result<i32, &str> -> Ok(42)

let res2: Result<i32, &str> = Err("error");
let opt2 = res2.ok();          // Option<i32> -> None
```

### 5.3 何时使用哪一个？

- 如果你只关心“有没有”，不关心“为什么没有” → 用 `Option`。
- 如果操作可能因为多种外部原因失败，并且你需要传达失败原因 → 用 `Result`。
- 编写库函数时，推荐使用 `Result` 并提供有意义的错误类型，这样调用者可以做出恰当处理。
- 内部辅助函数或普通查找，`Option` 足够。

---

## 6. panic! —— 不可恢复错误

### 6.1 定义与行为

`panic!` 是一个宏，当程序遇到无法处理的错误时，它会：

1. 打印错误信息（以及可选的自定义消息）。
2. 展开（unwind）栈或直接中止（取决于编译配置）。
3. 退出当前线程（如果是主线程则程序退出）。

```rust
fn main() {
    panic!("发生了致命错误！");   // 程序输出错误信息后崩溃
    println!("这行永远不会执行");
}
```

### 6.2 什么时候应该 panic？

Rust 的哲学：**宁可崩溃，也不要继续执行非法状态**。以下情况适合使用 `panic!`：

- 违反契约，例如数组索引越界（Rust 会自动 `panic!`）。
- 不可恢复的错误，比如违反关键业务不变量、程序进入无法继续的非法状态。
- 示例代码或原型中快速失败。
- 当你知道错误不可能发生时，对 `Option` 或 `Result` 调用 `unwrap()` / `expect()`（但生产代码应慎用）。

### 6.3 示例：显式 panic 与自动 panic

```rust
// 显式 panic
fn check_age(age: i32) {
    if age < 0 {
        panic!("年龄不能为负数！");
    }
    println!("年龄有效");
}

// 自动 panic：数组越界
let arr = [1, 2, 3];
println!("{}", arr[5]);   // 编译可通过，运行时会 panic!
```

### 6.4 与 Result 的对比

- `Result` 用于**可预见的**、**调用者应该处理的**错误（如文件不存在）。
- `panic!` 用于**不可恢复的**、**调用者无法合理处理的**错误（如程序逻辑错误）。

例如：如果 `Vec::get` 越界，它返回 `Option`，而不是 `panic!`（因为调用者可能想检查边界）。但直接用索引 `vec[5]` 就会 `panic!`，因为语言假设你确信索引有效。

### 6.5 如何捕获 panic（不推荐用于常规错误处理）

Rust 提供了 `std::panic::catch_unwind`，允许在当前线程捕获 `panic!`，但**这主要用于需要保护不崩溃的场景**（如 C++ 异常边界），不应该用来替代 `Result`。

```rust
use std::panic;

let result = panic::catch_unwind(|| {
    panic!("崩溃了");
});
assert!(result.is_err());
```

通常你不需要这样做，保持“使用 `Result` 处理错误，使用 `panic!` 处理 bug”的原则即可。

---

## 7. 综合示例：从文件读取数字并求和

下面是一个完整的程序，综合运用了 `match`、`Option`、`Result` 和 `panic!`。程序从 `numbers.txt` 读取每行一个整数，计算总和，并处理各种错误。

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};

/// 读取文件并返回每行解析后的 i32 向量，若文件不存在则返回错误。
fn read_numbers_from_file(path: &str) -> Result<Vec<i32>, io::Error> {
    let file = File::open(path)?;   // ? 遇到 Err 则提前返回
    let reader = BufReader::new(file);
    let mut numbers = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;  // 读取行时可能出错
        match line.trim().parse::<i32>() {
            Ok(num) => numbers.push(num),
            Err(e) => {
                eprintln!("警告：第 {} 行解析失败: {} (内容: {})", line_num + 1, e, line);
                // 继续处理其他行，而不是 panic
            }
        }
    }
    Ok(numbers)
}

fn main() {
    let filename = "numbers.txt";
    match read_numbers_from_file(filename) {
        Ok(nums) => {
            let sum: i32 = nums.iter().sum();
            println!("成功读取 {} 个数字，总和为 {}", nums.len(), sum);
        }
        Err(e) => {
            eprintln!("致命错误：无法读取文件 {}: {}", filename, e);
            panic!("程序无法继续运行，因为缺少关键文件");  // 这里 panic! 表示不可恢复
        }
    }
}
```

**解释：**

- 用 `Result<Vec<i32>, io::Error>` 表示可能 I/O 错误。
- 在 `main` 中，若文件打开失败，输出错误并 `panic!`（因为文件名是硬编码的，缺少则表明环境问题）。
- 解析每行时使用 `match`，对解析失败的行只警告，不中断整个程序（可恢复）。
- 使用 `?` 操作符简化 `Result` 的传播
