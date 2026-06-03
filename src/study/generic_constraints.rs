//定义一个trait
trait MyTrait{
    fn foo(&self);
    fn bar(&self);
}


//定义一个泛型函数,使用泛型约束，要求T实现MyTrait trait。
fn func<T:MyTrait>(t:&T){
    t.foo();
    t.bar();
}

struct MyStruct{}
impl MyTrait for MyStruct{
    fn foo(&self){
        println!("foo");
    }
    fn bar(&self){
        println!("bar");
    }
}

pub fn demo(){
    println!("...............泛型约束示例开始.................");
    func(&MyStruct{});
   // func(&String::from("hello"));  //编译报错，因为String没有实现MyTrait
    println!("...............泛型约束示例结束.................");
}