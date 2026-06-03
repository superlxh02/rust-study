/*
声明宏语法：
 */

macro_rules! macro_demo {
    //无参数匹配
    () => {
        println!("hello world!");
    };

    //有参数匹配-字面量
    ($name:literal)=>{
        println!("iteral match:{}",$name);
    };

    //有参数匹配-任意表达式(示例传入函数)
    ($name:expr)=>{
        println!("有参数匹配-任意表达式(示例传入函数)");
        $name;
    };

    //重复匹配-任意数量表达式
    ($($name:expr),*)=>{
        println!("重复匹配-任意数量表达式:");
        $(
            $name;
        )*
    };
}

fn func1(){
    println!("func1 called");
}

fn func2(){
    println!("func2 called");
}

fn func3(){
    println!("func3 called");
}

fn func4(a:i32,b:i32){
    println!("func4 called:{},{}",a,b);
}

pub fn demo(){
    println!("..............声明宏示例开始..................");
    macro_demo!();
    macro_demo!("this is a literal");
    macro_demo!(func1());
    macro_demo!(func2(),func3(),func4(10,20));

    println!("..............声明宏示例结束..................");

}