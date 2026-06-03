/// 定义一个对象安全的 trait
trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> &str;
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    fn name(&self) -> &str {
        "Circle"
    }
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
    fn name(&self) -> &str {
        "Rectangle"
    }
}

// 静态分发：编译时确定具体类型（单态化）
fn print_area_static<T: Shape>(shape: &T) {
    println!("{} area = {}", shape.name(), shape.area());
}

// 动态分发：通过 trait 对象在运行时决定
fn print_area_dynamic(shape: &dyn Shape) {
    println!("{} area = {}", shape.name(), shape.area());
}

// 拥有所有权的 trait 对象（放在堆上）
fn create_shape(kind: &str) -> Box<dyn Shape> {
    match kind {
        "circle" => Box::new(Circle { radius: 1.0 }),
        "rect" => Box::new(Rectangle { width: 2.0, height: 3.0 }),
        _ => panic!("unknown shape"),
    }
}

fn main() {

}

pub fn demo() {
    println!("...............trait对象示例开始...................");
    // 静态分发示例
    let circle = Circle { radius: 2.0 };
    let rect = Rectangle { width: 3.0, height: 4.0 };
    print_area_static(&circle);
    print_area_static(&rect);

    // 动态分发：&dyn Shape（借用）
    let shapes: Vec<&dyn Shape> = vec![&circle, &rect];
    for s in shapes {
        print_area_dynamic(s);
    }

    // 动态分发：Box<dyn Shape>（拥有所有权，堆分配）
    let shape1 = create_shape("circle");
    let shape2 = create_shape("rect");
    let shape_list: Vec<Box<dyn Shape>> = vec![shape1, shape2];
    for s in shape_list {
        println!("{} area = {}", s.name(), s.area());
    }
    println!("...............trait对象示例结束...................");
}