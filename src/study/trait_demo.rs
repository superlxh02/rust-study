// 定义一个trait，用于描述动物的基本行为
 trait BaseBehavior {
    fn eat(&self);
    fn drink(&self);
}


// 定义一个狗结构体，实现BaseBehavior trait
struct Dog{
    name: String,
}
impl BaseBehavior for Dog {
    fn eat(&self) {
        println!("dog {} eat", self.name);
    }
    fn drink(&self) {
        println!("dog {} drink", self.name);
    }
}

// 定义一个人结构体，实现BaseBehavior trait
struct Person{
    name: String,
}
impl BaseBehavior for Person {
    fn eat(&self) {
        println!("person {} eat", self.name);
    }
    fn drink(&self) {
        println!("person {} drink", self.name);
    }
}

pub fn demo() {
    println!("...............trait示例开始...................");
    let dog = Dog { name: String::from("wxh") };
    dog.eat();
    dog.drink();
    let person = Person { name: String::from("zhangsan") };
    person.eat();
    person.drink();
    println!("...............trait示例结束...................");
}