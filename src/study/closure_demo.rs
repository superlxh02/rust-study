pub fn demo(){
    println!("...............闭包示例开始...............");
    callback_func_1(10, |str:String|{
        println!("str:{}",str);
        return Some(str);
    });
    callback_func_2(||{
        println!("hello world");
    });
    callback_func_2(||{
        println!("hello world");
    });
    println!("...............闭包示例结束...............");
}


//回调函数示例1，固定fn参数
fn callback_func_1(i:i32,callback:fn(String)->Option<String>){
    let res = callback(i.to_string());
    if let Some(str) = res{
        println!("res:{}",str);
    }
}

fn callback_func_2<F>(f:F)
where F: Fn()
{
    f();
}

