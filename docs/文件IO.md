# rust文件I/O与文件系统操作

## 1. 文件I/O

文件I/O主要包括文件的打开、关闭、读取、写入等操作。Rust通过 `std::fs::File`和 `std::io`模块提供这些功能。

### 1.1 打开文件

#### 1.1.1 功能：打开现有文件 – `File::open`

**功能**：以只读模式打开一个已存在的文件。如果文件不存在，返回错误。

**接口签名**：

```rust
pub fn open<P: AsRef<Path>>(path: P) -> Result<File>
```

- **参数**：`path` – 文件路径（可以是字符串、Path或PathBuf）。
- **返回值**：`Result<File>` – 成功返回 `File`对象，失败返回 `std::io::Error`。

**简单调用示例**：

```rust
use std::fs::File;
let file = File::open("hello.txt").expect("文件打开失败");
```

#### 1.1.2 功能：创建或覆盖文件 – `File::create`

**功能**：以只写模式打开文件。如果文件不存在则创建；如果存在则**截断（覆盖）**原有内容。

**接口签名**：

```rust
pub fn create<P: AsRef<Path>>(path: P) -> Result<File>
```

- **参数**：`path` – 文件路径。
- **返回值**：`Result<File>`。

**简单调用示例**：

```rust
let file = File::create("output.txt").expect("创建文件失败");
```

#### 1.1.3 功能：自定义选项打开文件 – `OpenOptions`

**功能**：通过 `OpenOptions`可以精细控制打开方式（读/写/追加/创建/截断等）。

##### 1.1.3.1 `OpenOptions::new`

**功能**：创建一个空的 `OpenOptions`结构体。

**接口签名**：

```rust
pub fn new() -> Self
```

- **返回值**：`OpenOptions`实例。

##### 1.1.3.2 `OpenOptions::read` / `write` / `append` / `create` / `truncate`

**功能**：设置打开选项（链式调用）。

**接口签名**（以 `read`为例）：

```rust
pub fn read(&mut self, read: bool) -> &mut Self
```

- **参数**：`read` – `true`表示启用读权限。
- **返回值**：`&mut Self`，支持链式调用。

##### 1.1.3.3 `OpenOptions::open`

**功能**：根据配置打开文件。

**接口签名**：

```rust
pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<File>
```

- **参数**：`path` – 文件路径。
- **返回值**：`Result<File>`。

**简单调用示例**：

```rust
use std::fs::OpenOptions;
let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .append(true)
    .open("log.txt")
    .expect("打开失败");
```

### 1.2 读取文件

#### 1.2.1 功能：读取全部内容为字符串 – `std::fs::read_to_string`

**功能**：将整个文件的内容读入一个 `String`中（便捷函数，不需要手动打开文件）。

**接口签名**：

```rust
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String>
```

- **参数**：`path` – 文件路径。
- **返回值**：`Result<String>` – 文件内容。

**简单调用示例**：

```rust
let content = std::fs::read_to_string("data.txt").unwrap();
println!("{}", content);
```

#### 1.2.2 功能：读取全部内容为字节数组 – `std::fs::read`

**功能**：将整个文件读入 `Vec<u8>`中。

**接口签名**：

```rust
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>>
```

- **参数**：`path` – 文件路径。
- **返回值**：`Result<Vec<u8>>`。

**简单调用示例**：

```rust
let bytes = std::fs::read("image.png").unwrap();
```

#### 1.2.3 功能：从 `File`中读取 – `std::io::Read` trait

`File`实现了 `Read` trait，可以使用 `read`方法逐步读取。

##### 1.2.3.1 `Read::read`

**功能**：从文件中读取数据到缓冲区，返回读取的字节数。

**接口签名**：

```rust
fn read(&mut self, buf: &mut [u8]) -> Result<usize>
```

- **参数**：`buf` – 可变字节切片，用于存放读取的数据。
- **返回值**：`Result<usize>` – 实际读取的字节数，0表示EOF。

**简单调用示例**：

```rust
use std::io::Read;
let mut file = File::open("data.bin").unwrap();
let mut buffer = [0; 1024];
let n = file.read(&mut buffer).unwrap();
println!("读取了 {} 字节", n);
```

##### 1.2.3.2 `Read::read_to_end`

**功能**：读取所有剩余内容直到EOF，追加到 `Vec<u8>`中。

**接口签名**：

```rust
fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize>
```

- **参数**：`buf` – 存放数据的 `Vec`。
- **返回值**：`Result<usize>` – 读取的总字节数。

**简单调用示例**：

```rust
let mut file = File::open("data.txt").unwrap();
let mut contents = Vec::new();
file.read_to_end(&mut contents).unwrap();
```

##### 1.2.3.3 `Read::read_to_string`

**功能**：读取所有剩余内容，追加到 `String`中。

**接口签名**：

```rust
fn read_to_string(&mut self, buf: &mut String) -> Result<usize>
```

- **参数**：`buf` – 存放字符串的 `String`。
- **返回值**：`Result<usize>`。

**简单调用示例**：

```rust
let mut file = File::open("hello.txt").unwrap();
let mut content = String::new();
file.read_to_string(&mut content).unwrap();
```

### 1.3 写入文件

#### 1.3.1 功能：写入全部内容 – `std::fs::write`

**功能**：将数据（字节切片或字符串）一次性写入文件。如果文件不存在则创建；存在则覆盖。

**接口签名**：

```rust
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()>
```

- **参数**：
  - `path` – 文件路径。
  - `contents` – 要写入的数据（`&[u8]`或 `&str`）。
- **返回值**：`Result<()>`。

**简单调用示例**：

```rust
std::fs::write("output.txt", "Hello, Rust!").unwrap();
```

#### 1.3.2 功能：向 `File`写入 – `std::io::Write` trait

`File`实现了 `Write` trait。

##### 1.3.2.1 `Write::write`

**功能**：将缓冲区中的数据写入文件，返回写入的字节数。

**接口签名**：

```rust
fn write(&mut self, buf: &[u8]) -> Result<usize>
```

- **参数**：`buf` – 要写入的数据切片。
- **返回值**：`Result<usize>` – 实际写入的字节数。

**简单调用示例**：

```rust
use std::io::Write;
let mut file = File::create("out.txt").unwrap();
let bytes = file.write(b"Hello").unwrap();
```

##### 1.3.2.2 `Write::write_all`

**功能**：尝试写入整个缓冲区，直到所有数据都写完或出错。

**接口签名**：

```rust
fn write_all(&mut self, buf: &[u8]) -> Result<()>
```

- **参数**：`buf` – 要写入的数据。
- **返回值**：`Result<()>`。

**简单调用示例**：

```rust
file.write_all(b"Entire content").unwrap();
```

##### 1.3.2.3 `Write::flush`

**功能**：刷新缓冲区，确保所有数据都写入底层系统。

**接口签名**：

```rust
fn flush(&mut self) -> Result<()>
```

- **返回值**：`Result<()>`。

**简单调用示例**：

```rust
file.flush().unwrap();
```

### 1.4 关闭文件

Rust中 `File`在离开作用域时会自动关闭（`Drop` trait），无需显式调用 `close`。如果需要提前关闭，可以调用 `std::mem::drop(file)`。

### 1.5 文件I/O综合示例：复制文件

下面是一个完整的示例，演示打开源文件、读取内容、创建目标文件并写入。

```rust
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

fn copy_file(src: &str, dst: &str) -> std::io::Result<()> {
    // 打开源文件（只读）
    let mut src_file = File::open(src)?;
    // 创建目标文件（只写，覆盖）
    let mut dst_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dst)?;
  
    // 分配缓冲区
    let mut buffer = [0; 4096];
    loop {
        let n = src_file.read(&mut buffer)?;
        if n == 0 { break; } // EOF
        dst_file.write_all(&buffer[..n])?;
    }
    Ok(())
}

pub fn demo_io() {
    match copy_file("source.txt", "destination.txt") {
        Ok(_) => println!("文件复制成功"),
        Err(e) => eprintln!("复制失败: {}", e),
    }
}
```

## 2. 文件系统操作

文件系统操作包括创建/删除目录、遍历目录、获取文件元数据（路径、权限、大小等）。

### 2.1 目录操作

#### 2.1.1 功能：创建目录 – `std::fs::create_dir`

**功能**：创建一个空目录。如果父目录不存在，则返回错误。

**接口签名**：

```rust
pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<()>
```

- **参数**：`path` – 要创建的目录路径。
- **返回值**：`Result<()>`。

**简单调用示例**：

```rust
std::fs::create_dir("my_folder").unwrap();
```

#### 2.1.2 功能：递归创建目录 – `std::fs::create_dir_all`

**功能**：创建目录及其所有不存在的父目录。

**接口签名**：

```rust
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()>
```

- **参数**：`path` – 目录路径。
- **返回值**：`Result<()>`。

**简单调用示例**：

```rust
std::fs::create_dir_all("a/b/c/d").unwrap();
```

#### 2.1.3 功能：删除空目录 – `std::fs::remove_dir`

**功能**：删除一个空目录。如果目录非空，返回错误。

**接口签名**：

```rust
pub fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()>
```

- **参数**：`path` – 目录路径。
- **返回值**：`Result<()>`。

**简单调用示例**：

```rust
std::fs::remove_dir("empty_folder").unwrap();
```

#### 2.1.4 功能：递归删除目录及其内容 – `std::fs::remove_dir_all`

**功能**：删除目录以及内部所有文件和子目录。

**接口签名**：

```rust
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<()>
```

- **参数**：`path` – 目录路径。
- **返回值**：`Result<()>`。

**简单调用示例**：

```rust
std::fs::remove_dir_all("non_empty_folder").unwrap();
```

### 2.2 路径操作

#### 2.2.1 功能：获取当前工作目录 – `std::env::current_dir`

**功能**：返回当前进程的工作目录。

**接口签名**：

```rust
pub fn current_dir() -> Result<PathBuf>
```

- **返回值**：`Result<PathBuf>` – 当前目录的绝对路径。

**简单调用示例**：

```rust
let cur_dir = std::env::current_dir().unwrap();
println!("{}", cur_dir.display());
```

#### 2.2.2 功能：获取文件的绝对路径 – `std::fs::canonicalize`

**功能**：将相对路径解析为绝对路径，并解析所有符号链接。

**接口签名**：

```rust
pub fn canonicalize<P: AsRef<Path>>(path: P) -> Result<PathBuf>
```

- **参数**：`path` – 原始路径。
- **返回值**：`Result<PathBuf>` – 规范化后的绝对路径。

**简单调用示例**：

```rust
let abs_path = std::fs::canonicalize("data/file.txt").unwrap();
```

### 2.3 文件元数据

#### 2.3.1 功能：获取文件元数据 – `std::fs::metadata`

**功能**：获取文件或目录的信息（大小、权限、修改时间等）。

**接口签名**：

```rust
pub fn metadata<P: AsRef<Path>>(path: P) -> Result<Metadata>
```

- **参数**：`path` – 文件或目录路径。
- **返回值**：`Result<Metadata>`。

**简单调用示例**：

```rust
let metadata = std::fs::metadata("file.txt").unwrap();
println!("文件大小: {} bytes", metadata.len());
```

#### 2.3.2 `Metadata`常用方法

##### 2.3.2.1 `metadata.is_file`

**功能**：判断是否为普通文件。

**接口签名**：

```rust
pub fn is_file(&self) -> bool
```

- **返回值**：`bool`。

##### 2.3.2.2 `metadata.is_dir`

**功能**：判断是否为目录。

**接口签名**：

```rust
pub fn is_dir(&self) -> bool
```

- **返回值**：`bool`。

##### 2.3.2.3 `metadata.len`

**功能**：返回文件大小（字节数）。

**接口签名**：

```rust
pub fn len(&self) -> u64
```

- **返回值**：`u64`。

##### 2.3.2.4 `metadata.permissions`

**功能**：返回文件的权限信息。

**接口签名**：

```rust
pub fn permissions(&self) -> Permissions
```

- **返回值**：`Permissions`。

**简单调用示例**：

```rust
if metadata.is_file() {
    println!("大小: {} bytes", metadata.len());
    println!("权限: {:?}", metadata.permissions());
}
```

### 2.4 修改权限

#### 2.4.1 功能：设置文件权限 – `std::fs::set_permissions`

**功能**：修改文件或目录的权限。

**接口签名**：

```rust
pub fn set_permissions<P: AsRef<Path>>(path: P, perm: Permissions) -> Result<()>
```

- **参数**：
  - `path` – 路径。
  - `perm` – 新的权限。
- **返回值**：`Result<()>`。

**简单调用示例**：

```rust
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt; // Unix平台
let mut perms = fs::metadata("script.sh").unwrap().permissions();
perms.set_mode(0o755); // 设置 rwxr-xr-x
fs::set_permissions("script.sh", perms).unwrap();
```

> 注意：Windows平台权限模型不同，可以使用 `PermissionsExt` trait或其他平台特定方法。

### 2.5 读取目录内容

#### 2.5.1 功能：读取目录条目 – `std::fs::read_dir`

**功能**：返回目录中所有条目的迭代器。

**接口签名**：

```rust
pub fn read_dir<P: AsRef<Path>>(path: P) -> Result<ReadDir>
```

- **参数**：`path` – 目录路径。
- **返回值**：`Result<ReadDir>`，可以迭代出 `DirEntry`。

**简单调用示例**：

```rust
for entry in std::fs::read_dir(".").unwrap() {
    let entry = entry.unwrap();
    let file_name = entry.file_name();
    println!("{}", file_name.to_string_lossy());
}
```

#### 2.5.2 `DirEntry`常用方法

##### 2.5.2.1 `DirEntry::path`

**功能**：获取该条目的完整路径。

**接口签名**：

```rust
pub fn path(&self) -> PathBuf
```

- **返回值**：`PathBuf`。

##### 2.5.2.2 `DirEntry::file_name`

**功能**：获取文件名（不含路径）。

**接口签名**：

```rust
pub fn file_name(&self) -> OsString
```

- **返回值**：`OsString`。

##### 2.5.2.3 `DirEntry::metadata`

**功能**：获取该条目的元数据。

**接口签名**：

```rust
pub fn metadata(&self) -> Result<Metadata>
```

- **返回值**：`Result<Metadata>`。

### 2.6 文件系统操作综合示例：目录树遍历与统计

下面是一个完整示例，递归遍历目录，统计文件数量、总大小，并输出文件路径及权限。

```rust
use std::fs::{self, DirEntry, Metadata};
use std::io;
use std::path::Path;

fn walk_dir(dir: &Path) -> io::Result<(usize, u64)> {
    let mut file_count = 0;
    let mut total_size = 0;

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                file_count += 1;
                total_size += metadata.len();
                // 输出文件信息
                println!("文件: {}", path.display());
                println!("  大小: {} bytes", metadata.len());
                println!("  权限: {:?}", metadata.permissions());
            } else if metadata.is_dir() {
                let (sub_count, sub_size) = walk_dir(&path)?;
                file_count += sub_count;
                total_size += sub_size;
            }
        }
    }
    Ok((file_count, total_size))
}

pub fn demo_fs() {
    let start_dir = Path::new(".");
    match walk_dir(start_dir) {
        Ok((files, size)) => {
            println!("\n总计: {} 个文件, {} 字节", files, size);
        }
        Err(e) => eprintln!("遍历出错: {}", e),
    }
}
```
