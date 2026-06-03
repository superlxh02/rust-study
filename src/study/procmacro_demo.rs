use procmarco_demo::{make_fn, HelloDerive, log_call};
    trait Hello {
    fn hello(&self);
}

#[derive(HelloDerive)]
struct User {
    name: String,
}

#[log_call]
fn test() {
    println!("inside test");
}

make_fn!(generated, "hello from generated fn");

pub fn demo() {
    println!(".............过程宏示例开始.............");
    let user = User {
        name: String::from("lxh"),
    };
    user.hello();
    test();
    generated();
    println!(".............过程宏示例结束.............");
}