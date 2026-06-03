use std::collections::{HashMap, VecDeque};

// String 是 Rust 标准库提供的可增长 UTF-8 字符串。
// 它拥有字符串内容的所有权，适合需要拼接、修改、从函数返回的场景。
fn string_demo() {
    println!("String 使用.............");

    // 1. 创建 String：常见方式有 String::new、String::from、to_string。
    let mut title = String::new();
    title.push_str("Rust");
    title.push(' ');
    title.push_str("containers");
    println!("title = {}", title);

    let hello = String::from("hello");
    let world = "world".to_string();
    let message = format!("{}, {}!", hello, world);
    println!("message = {}", message);

    // 2. Rust 字符串是 UTF-8，按字节切片时必须落在字符边界上。
    let chinese = String::from("你好 Rust");
    println!("chars:");
    for ch in chinese.chars() {
        println!("  {}", ch);
    }

    // 3. parse 可以把字符串解析成其他类型，返回 Result。
    let number = "42".parse::<i32>().unwrap();
    println!("parse result = {}", number);
}

// Vec<T> 是连续内存上的动态数组。
// 它适合按顺序存储一组同类型元素，并且需要高效随机访问。
fn vec_demo() {
    println!("Vec 使用.............");

    // 1. 创建 Vec：可以用 Vec::new，也可以用 vec! 宏。
    let mut numbers = Vec::<i32>::new();
    numbers.push(10);
    numbers.push(20);
    numbers.push(30);

    let more_numbers = vec![1, 2, 3];
    println!("numbers = {:?}", numbers);
    println!("more_numbers = {:?}", more_numbers);

    // 2. 读取元素：索引越界会 panic，get 越界会返回 None。
    println!("numbers[0] = {}", numbers[0]);
    match numbers.get(10) {
        Some(value) => println!("numbers[10] = {}", value),
        None => println!("numbers[10] 不存在"),
    }

    // 3. 修改和遍历元素。
    for value in &mut numbers {
        *value += 1;
    }
    for value in &numbers {
        println!("vec item = {}", value);
    }

    // 4. 删除元素：remove 会移动后续元素，pop 从尾部删除更高效。
    let removed = numbers.remove(1);
    println!("remove result = {}, numbers = {:?}", removed, numbers);
    println!("pop result = {:?}", numbers.pop());
}

// HashMap<K, V> 是哈希表，适合通过 key 快速查找 value。
// 使用 HashMap 时，key 必须实现 Eq 和 Hash。
fn hashmap_demo() {
    println!("HashMap 使用.............");

    let mut scores = HashMap::<String, i32>::new();

    // 1. insert 插入键值对。如果 key 已存在，会覆盖旧值并返回旧值。
    scores.insert(String::from("Alice"), 95);
    scores.insert(String::from("Bob"), 82);
    let old = scores.insert(String::from("Bob"), 88);
    println!("Bob old score = {:?}", old);

    // 2. get 返回 Option<&V>，因为 key 可能不存在。
    match scores.get("Alice") {
        Some(score) => println!("Alice score = {}", score),
        None => println!("Alice 不存在"),
    }

    // 3. entry 适合“没有就插入，有就修改”的场景。
    scores.entry(String::from("Carol")).or_insert(90);
    *scores.entry(String::from("Alice")).or_insert(0) += 1;

    // 4. 遍历 HashMap 的顺序不稳定，不要依赖输出顺序。
    for (name, score) in &scores {
        println!("{} => {}", name, score);
    }

    scores.remove("Bob");
    println!("remove Bob 后: {:?}", scores);
}

// VecDeque<T> 是双端队列，适合从头部和尾部都高效插入/删除。
// 它内部是环形缓冲区，不保证像 Vec 一样整体连续。
fn vecdeque_demo() {
    println!("VecDeque 使用.............");

    let mut queue = VecDeque::<String>::new();

    queue.push_back(String::from("task-1"));
    queue.push_back(String::from("task-2"));
    queue.push_front(String::from("urgent-task"));
    println!("queue = {:?}", queue);

    while let Some(task) = queue.pop_front() {
        println!("handle {}", task);
    }

    queue.push_back(String::from("tail"));
    queue.push_front(String::from("head"));
    println!("front = {:?}, back = {:?}", queue.front(), queue.back());
}

// 综合示例：用多个容器构建一个简单任务看板。
fn task_board_demo() {
    println!("综合示例：任务看板.............");

    let raw_tasks = String::from("learn rust,write demo,read book");
    let mut waiting = VecDeque::<String>::new();
    let mut finished = Vec::<String>::new();
    let mut owners = HashMap::<String, String>::new();

    // String::split 返回迭代器，逐个切出任务名。
    for task in raw_tasks.split(',') {
        waiting.push_back(task.trim().to_string());
    }

    owners.insert(String::from("learn rust"), String::from("Alice"));
    owners.insert(String::from("write demo"), String::from("Bob"));
    owners.insert(String::from("read book"), String::from("Carol"));

    while let Some(task) = waiting.pop_front() {
        let owner = owners
            .get(&task)
            .map(String::as_str)
            .unwrap_or("unknown");
        println!("{} is handling {}", owner, task);
        finished.push(task);
    }

    println!("finished tasks = {:?}", finished);
}

pub fn demo() {
    println!("...............容器示例开始.................");
    string_demo();
    vec_demo();
    hashmap_demo();
    vecdeque_demo();
    task_board_demo();
    println!("...............容器示例结束.................");
}
