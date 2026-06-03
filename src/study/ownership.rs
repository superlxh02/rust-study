/*
  -----------------3条基本原则------------------
   1.唯一归属：每个值在任意时刻有且仅有一个所有者（Owner）。
   2.自动释放：当所有者离开作用域时，该值被自动丢弃（Drop，释放内存）。
   3.移动语义（Move）：赋值或传参时，所有权默认会被转移（原变量失效，避免双重释放）。

  ----------------2种特殊情况（无需转移所有权）------------------
   1.克隆（Clone）：若需深拷贝，显式调用 .clone()，此时新旧变量各自拥有独立的内存。
   2.借用（Borrow）：通过引用 & 使用数据而不获取所有权。

  ------------------------借用的规则------------------
    不可变借用 &T：允许同时存在多个，但只能读不能改。
    可变借用 &mut T：同一时刻只能存在一个，且可读写。
    互斥原则：可变借用与不可变借用不能同时存在（防止数据竞争）。

   tips:如果学过C++的，那么rust的规则正好和C++的规则是相反的。
        C++等号赋值和形参传递在不加引用默认的前提下，都是复制语义；而rust默认是移动语义，rust需要显式调用clone()方法来复制数据。
 */


// 1. 唯一归属 & 2. 自动释放
fn rule1_and_2_ownership_and_drop() {
    println!("--- 规则1&2：唯一归属与自动释放 ---");

    // 每个值只有一个所有者
    let s1 = String::from("hello"); // s1 是 "hello" 的所有者

    // 当 s1 离开作用域时，Rust 会自动调用 drop 函数释放内存
    // 无需手动 free/delete，也不会忘记释放
    println!("s1 = {}", s1);
} // <- s1 在此离开作用域，内存自动释放


// 3. 移动语义
fn rule3_move() {
    println!("\n--- 规则3：移动语义 ---");

    let s1 = String::from("rust"); // s1 是所有者
    let s2 = s1;                   // 所有权从 s1 转移给 s2 (Move)

    // println!("s1 = {}", s1); //  编译错误！s1 已经失效，不能再使用
    println!("s2 = {}", s2);    //  只有 s2 有效

    // 【补充观察：基本类型是拷贝语义】
    // 像整型、浮点型等实现了 Copy trait 的类型，赋值时是按位拷贝，不会发生移动
    let x = 5;
    let y = x; // x 被拷贝给 y
    println!("x = {}, y = {}", x, y); //x 和 y 都有效，因为它们是栈上的简单值
}


// 特殊情况1：克隆
fn special_case1_clone() {
    println!("\n--- 特殊情况1：克隆 ---");

    let s1 = String::from("clone_me");
    let s2 = s1.clone(); // 显式深拷贝，s1 和 s2 拥有各自独立的堆内存

    println!("s1 = {}, s2 = {}", s1, s2); // 两者都有效，互不影响
}


// 借用规则：不可变借用 &T
fn borrow_immutable() {
    println!("\n--- 借用规则：不可变借用 ---");

    let s = String::from("hello");

    // 可以同时存在多个不可变借用
    let r1 = &s;
    let r2 = &s;

    // r1 和 r2 只是借用了 s 的读取权，s 仍然是所有者
    println!("s = {}, r1 = {}, r2 = {}", s, r1, r2); //  全部有效
}


// 借用规则：可变借用 &mut T
fn borrow_mutable() {
    println!("\n--- 借用规则：可变借用 ---");

    let mut s = String::from("hello");

    // 同一时刻只能存在一个可变借用
    let r1 = &mut s;
    r1.push_str("_mut");

    // let r2 = &mut s; //  编译错误！s 已经被可变借用给 r1，不能再次可变借用
    // println!("r1 = {}, r2 = {}", r1, r2);

    println!("r1 = {}", r1); //  只有一个可变借用，合法

    // 【关键观察：借用生命周期结束】
    // r1 在上一行最后一次使用后，其生命周期就结束了，s 可以再次被借用
    let r2 = &mut s; //  合法！因为 r1 已经不再使用
    r2.push_str("_again");
    println!("r2 = {}", r2);
}


// 借用规则：互斥原则
fn borrow_exclusion() {
    println!("\n--- 借用规则：互斥原则（不可变与可变不能共存） ---");

    let mut s = String::from("rust");

    let r1 = &s;      // 不可变借用
    let r2 = &s;      // 不可变借用

    // let r3 = &mut s; //  编译错误！当存在不可变借用时，不能有可变借用
    // 原因：如果 r3 修改了 s，那 r1 和 r2 指向的值就变了，这是严重的数据竞争

    println!("r1 = {}, r2 = {}", r1, r2); // 不可变借用的最后一次使用在这里

    // r1 和 r2 的生命周期在上面那行结束后，就可以重新可变借用了
    let r3 = &mut s; //  合法
    r3.push_str("_safe");
    println!("r3 = {}", r3);
}


// 在函数中传递所有权与借用
fn practice_ownership_in_function() {
    println!("\n--- 在函数中传递所有权与借用 ---");

    let s1 = String::from("world");

    // 1. 传参也会发生 Move（转移所有权）
    takes_ownership(s1);
    // println!("{}", s1); //  s1 的所有权已经移入函数，此处失效

    // 2. 传引用（借用）不会转移所有权
    let mut s2 = String::from("hello");
    borrows_immutable(&s2); // 传不可变引用
    println!("s2 is still valid: {}", s2); // s2 仍然有效

    borrows_mutable(&mut s2); // 传可变引用
    println!("s2 after mutation: {}", s2); //  s2 被函数内部修改了

    // 3. 函数返回所有权
    let s3 = gives_ownership(); // 函数将创建的值的所有权返回给 s3
    println!("s3 = {}", s3);
}

// 辅助函数
fn takes_ownership(some_string: String) {
    println!("I own this now: {}", some_string);
} // <- some_string 离开作用域，内存释放

fn borrows_immutable(some_string: &String) {
    println!("I just borrowed immutably: {}", some_string);
} // <- some_string 离开作用域，但因为它是引用，不会释放原始值

fn borrows_mutable(some_string: &mut String) {
    some_string.push_str(" rust!");
    println!("I borrowed mutably and changed it: {}", some_string);
}

fn gives_ownership() -> String {
    let some_string = String::from("I am new");
    some_string // 直接返回，所有权转移给调用者
}


pub fn demo() {
    println!("...........所有权示例开始...........");
    rule1_and_2_ownership_and_drop();
    rule3_move();
    special_case1_clone();
    borrow_immutable();
    borrow_mutable();
    borrow_exclusion();
    practice_ownership_in_function();
    println!("...........所有权示例结束...........");
}


