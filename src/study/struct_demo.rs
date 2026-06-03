//定义一个结构体MyStruct，包含a、b、c三个字段，a、b为整数类型，c为字符串类型
struct MyStruct {
    a: i32,
    b: i32,
    c:String
}
// impl关键字用于给结构体实现结构体的方法
// Self关键字用于表示当前结构体的类型
// &self关键字用于表示当前结构体实例的引用，类似于this关键字
impl MyStruct {
    fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: String::from("hello")
        }
    }

    fn demo(&self) {
        let res = self.a + self.b;
        println!("res:{} , c: {}", res, self.c);
    }

}

pub fn demo() {
    println!("...............结构体示例开始.................");
    // 创建一个MyStruct实例，使用::调用new方法，返回一个MyStruct实例
    let ms = MyStruct::new();
    ms.demo();
    println!("...............结构体示例结束.................");
}
