/*
 * 返回值处理的方式
 Rust没有异常机制，Rust提供了两种方式，用来处理返回值：
  1. Option  -> 接近null指针的概念，表示返回值的有无
  2. Result  -> 表示值是否成功或失败
 */

// Option返回的示例，如果i大于0，返回Some(i)，否则返回None
fn opt_demo(i: i32) -> Option<i32> {
    if i > 0 {
        Some(i)
    } else {
        None

    }
}

// Result返回的示例，如果i大于0，返回Ok(i)，否则返回Err("i is negative")
fn result_demo(i: i32) -> Result<i32, String> {
    if i > 0 {
        Ok(i)
    } else {
        Err(String::from("i is negative"))
    }
}

pub fn demo(){
    println!("...............返回值处理示例开始.................");
    //1.处理option返回值,通常使用match表达式处理
    let i = opt_demo(-2);
    match i {
        Some(i) => println!("i is {}", i),
        None => println!("i is negative"),
    }
    //如果你确定i大于0，你可以直接unwrap(),unwarp表示如果i是Some(i)，则返回i，否则panic，程序终止
    let i = opt_demo(10).unwrap();
    println!("i is {}", i);

    //2.处理result返回值,通常使用match表达式处理
    let i = result_demo(0);
    match i {
        Ok(i) => println!("i is {}", i),
        Err(e) => println!("i is negative, error: {}", e),
    }
    //如果你确定i大于0，你可以直接unwrap(),unwarp表示如果i是Ok(i)，则返回i，否则panic，程序终止
    let i = result_demo(10).unwrap();
    println!("i is {}", i);
    println!("...............返回值处理示例结束.................");
}