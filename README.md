# Rust Study

这是一个 Rust 基础学习项目，包含教程文档和配套示例代码。

项目目标是用一组可运行的小 demo，把 Rust 的基础语法、类型系统、内存管理、并发、IO、宏和 async 编程串起来。文档适合按 mdBook 的章节顺序阅读，代码适合在 `src/study` 中按模块对照运行。

## 内容结构

- `docs/`：教程文档，作为 mdBook 的源目录。
- `src/study/`：教程配套代码，每个主题模块只对外暴露 `demo()` 函数。
- `src/main.rs`：统一调用入口，可以按需要注释或打开某个 demo。
- `procmarco_demo/`：过程宏示例 crate。
- `book.toml`：mdBook 配置。
- `.github/workflows/pages.yml`：GitHub Pages 自动部署配置。

## 教程章节

当前 mdBook 目录覆盖：

1. 基础语法
2. Cargo 构建系统
3. 泛型
4. Rust 内存管理：所有权、智能指针、内部可变性、生命周期
5. 返回值和错误处理
6. 结构体
7. trait 特征与泛型约束
8. Rust 模块
9. 闭包
10. 容器：`String`、`Vec`、`HashMap`、`VecDeque`
11. Rust 的面向对象写法：封装、组合优于继承、多态
12. 宏
13. 并发编程：线程、线程安全、并发扩展、原子操作
14. 文件 IO
15. 网络 IO
16. async 编程：机制、`futures` 体验、自定义 runtime

## 运行示例代码

检查项目是否能编译：

```bash
cargo check
```

运行当前在 `main.rs` 中打开的所有 demo：

```bash
cargo run
```

部分示例说明：

- `base_syntax::demo()` 是猜数字交互程序，会等待标准输入，默认在 `main.rs` 中注释。
- `async_runtime::demo()` 是自定义 runtime 扩展示例，默认注释，可按需单独打开。
- 网络 IO 示例会绑定本地 `127.0.0.1` 临时端口，如果运行环境限制本地 socket，可能需要在本机正常终端中运行。

## 阅读文档

本项目使用 mdBook 生成在线教程。

[Rust 在线教程文档](https://superlxh02.github.io/rust-study/)

本地构建：

```bash
mdbook build
```

本地预览：

```bash
mdbook serve --open
```

如果 `mdbook` 不在 PATH 中，可以使用：

```bash
~/.cargo/bin/mdbook serve --open
```
