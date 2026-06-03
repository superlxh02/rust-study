use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

fn demo_file_io(base_dir: &Path) -> io::Result<()> {
    let input_path = base_dir.join("input.txt");
    let copy_path = base_dir.join("copy.txt");

    // 写入完整内容。write 会创建文件；如果文件已存在，会覆盖旧内容。
    fs::write(&input_path, "10\n20\n30\n")?;

    // OpenOptions 可以组合 read/write/append/create/truncate 等打开方式。
    let mut append_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&input_path)?;
    append_file.write_all(b"40\n")?;
    append_file.flush()?;

    // read_to_string 一次性读取 UTF-8 文本。
    let content = fs::read_to_string(&input_path)?;
    println!("读取到的文本:\n{}", content.trim_end());

    // File 实现了 Read/Write trait，因此可以手动读取到 String。
    let mut file = fs::File::open(&input_path)?;
    let mut manual_content = String::new();
    file.read_to_string(&mut manual_content)?;
    println!("手动读取字节数: {}", manual_content.len());

    // copy 示例：读源文件、写目标文件。
    fs::copy(&input_path, &copy_path)?;
    let metadata = fs::metadata(&copy_path)?;
    println!("复制文件大小: {} bytes", metadata.len());

    Ok(())
}

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
                println!("文件: {}", path.display());
                println!("  大小: {} bytes", metadata.len());
                println!("  是否只读: {}", metadata.permissions().readonly());
            } else if metadata.is_dir() {
                let (sub_count, sub_size) = walk_dir(&path)?;
                file_count += sub_count;
                total_size += sub_size;
            }
        }
    }

    Ok((file_count, total_size))
}

fn demo_fs(base_dir: &Path) -> io::Result<()> {
    let nested_dir = base_dir.join("nested").join("child");

    // create_dir_all 会递归创建缺失目录；目录已存在时也不会失败。
    fs::create_dir_all(&nested_dir)?;
    fs::write(nested_dir.join("note.txt"), "hello file system")?;

    let abs = fs::canonicalize(base_dir)?;
    println!("临时演示目录绝对路径: {}", abs.display());

    let (files, size) = walk_dir(base_dir)?;
    println!("目录统计: {} 个文件, {} 字节", files, size);

    Ok(())
}

fn demo_dir() -> PathBuf {
    std::env::temp_dir().join("rust_study_file_io_demo")
}

pub fn demo() {
    println!("...............文件 IO 示例开始.................");

    let base_dir = demo_dir();
    if base_dir.exists() {
        // 演示目录只用于本示例，先清理旧数据可以保证每次输出稳定。
        fs::remove_dir_all(&base_dir).expect("清理旧演示目录失败");
    }
    fs::create_dir_all(&base_dir).expect("创建演示目录失败");

    if let Err(err) = demo_file_io(&base_dir) {
        eprintln!("文件读写演示失败: {}", err);
    }
    if let Err(err) = demo_fs(&base_dir) {
        eprintln!("文件系统演示失败: {}", err);
    }

    if let Err(err) = fs::remove_dir_all(&base_dir) {
        eprintln!("清理演示目录失败: {}", err);
    }

    println!("...............文件 IO 示例结束.................");
}
