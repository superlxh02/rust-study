
//定义一个泛型结构体
struct Data<T> {
    value: T,

}
//定义一个泛型trait
trait MyTrait<T>
{
    fn get(&self) -> &T;
}

//实现泛型trait
impl<T> MyTrait<T> for Data<T> {
    fn get(&self) ->&T  {
        &self.value
    }
}

//定义一个泛型函数
fn function<T>() {
    println!("the T type is {:?}", std::any::type_name::<T>())
}

pub fn demo() {
    println!("...............泛型示例开始...............");
    let d = Data::<String> { value: String::from("hello") };
    println!("d.get : result is {}",d.get());
    function::<i32>();
    function::<Data<i32>>();
    println!("...............泛型示例结束...............");
}