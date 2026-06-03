use std::cell::{Cell, RefCell};

// Cell<T> 适合 Copy 类型：读取时复制值，修改时整体替换值，
// 因此不需要拿到内部值的引用，也不会出现引用失效问题。
struct Counter {
    count: Cell<i32>,
}

impl Counter {
    fn new() -> Self {
        Self {
            count: Cell::new(0),
        }
    }

    fn increment(&self) {
        let current = self.count.get();
        self.count.set(current + 1);
    }

    fn value(&self) -> i32 {
        self.count.get()
    }
}

// RefCell<T> 适合任意类型：编译期允许通过 &self 修改内部状态，
// 运行期再检查“多个不可变借用”或“一个可变借用”的规则。
struct DataCache {
    computed: RefCell<Vec<i32>>,
    raw_data: Vec<i32>,
}

impl DataCache {
    fn new(data: Vec<i32>) -> Self {
        Self {
            computed: RefCell::new(Vec::new()),
            raw_data: data,
        }
    }

    fn get_computed(&self) -> Vec<i32> {
        let mut cache = self.computed.borrow_mut();

        if cache.is_empty() {
            for &val in &self.raw_data {
                cache.push(val * 2);
            }
        }

        // 显式释放可变借用，再获取不可变借用并克隆结果返回。
        drop(cache);
        self.computed.borrow().clone()
    }
}

fn cell_demo() {
    let counter = Counter::new();
    counter.increment();
    counter.increment();
    println!("Cell 计数器结果: {}", counter.value());
}

fn refcell_cache_demo() {
    let cache = DataCache::new(vec![1, 2, 3]);
    println!("第一次计算缓存: {:?}", cache.get_computed());
    println!("第二次直接复用缓存: {:?}", cache.get_computed());
}

fn refcell_borrow_rule_demo() {
    let data = RefCell::new(42);

    let b1 = data.borrow();
    let b2 = data.borrow();
    println!("多个不可变借用可以共存: {} {}", b1, b2);

    // 文档中直接 borrow_mut 会 panic。这里用 try_borrow_mut 演示同一规则，
    // 让 demo 可以稳定运行并继续执行后续示例。
    if data.try_borrow_mut().is_err() {
        println!("已经存在不可变借用，无法获取可变借用");
    }

    drop(b1);
    drop(b2);

    let mut b3 = data.borrow_mut();
    *b3 += 1;
    println!("不可变借用释放后，可变借用成功: {}", b3);
}

pub fn demo() {
    println!("...............内部可变性示例开始.................");
    cell_demo();
    refcell_cache_demo();
    refcell_borrow_rule_demo();
    println!("...............内部可变性示例结束.................");
}
