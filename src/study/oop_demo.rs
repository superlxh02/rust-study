// rust 的面向对象写法示例：
// Rust 没有 class 关键字，但可以用 struct 聚合数据，用 impl 定义方法，
// 再配合 trait 的默认实现和 trait object 实现封装、代码复用与多态。

mod bank_account {
    // 结构体本身公开，表示外部可以使用 Account 这个类型。
    // 字段不加 pub，所以 owner 和 balance 只能在当前模块内部访问，
    // 外部代码必须通过方法维护余额不为负等业务规则。
    pub struct Account {
        owner: String,
        balance: i32,
    }

    impl Account {
        // 构造函数统一初始化内部状态，避免外部绕过校验随意构造。
        pub fn new(owner: String) -> Self {
            Self { owner, balance: 0 }
        }

        // 存款方法只接受正数金额，非法输入被忽略。
        pub fn deposit(&mut self, amount: i32) {
            if amount > 0 {
                self.balance += amount;
            }
        }

        // 只读查询余额，不暴露字段本身。
        pub fn balance(&self) -> i32 {
            self.balance
        }

        // 只读查询账户所有者，同样通过方法暴露。
        pub fn owner(&self) -> &str {
            &self.owner
        }

        // 私有辅助方法，外部无法直接调用。
        fn can_withdraw(&self, amount: i32) -> bool {
            amount > 0 && self.balance >= amount
        }

        // 取款时复用私有判断逻辑，保证余额不会被扣成负数。
        pub fn withdraw(&mut self, amount: i32) -> bool {
            if self.can_withdraw(amount) {
                self.balance -= amount;
                true
            } else {
                false
            }
        }
    }
}

// 组合优于继承示例：Rust 不要求用“父类 -> 子类”表达关系，
// 而是把能力拆成可组合的小部件，让不同实体自由选择需要的组成部分。
struct Engine {
    horsepower: u32,
}

impl Engine {
    fn start(&self) {
        println!("engine start, horsepower = {}", self.horsepower);
    }
}

struct Battery {
    percent: u8,
}

impl Battery {
    fn status(&self) {
        println!("battery status = {}%", self.percent);
    }
}

struct Gps {
    position: String,
}

impl Gps {
    fn locate(&self) {
        println!("current position = {}", self.position);
    }
}

struct Car {
    name: String,
    engine: Engine,
    gps: Gps,
}

impl Car {
    fn drive(&self) {
        println!("car {} is ready", self.name);
        self.engine.start();
        self.gps.locate();
    }
}

struct DeliveryRobot {
    id: u32,
    battery: Battery,
    gps: Gps,
}

impl DeliveryRobot {
    fn deliver(&self) {
        println!("delivery robot {} is working", self.id);
        self.battery.status();
        self.gps.locate();
    }
}

// 对象安全 trait：方法不使用泛型参数，不返回裸 Self，因此可以变成 dyn Shape。
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

// 静态分发：编译期为每个具体类型生成专门代码。
fn print_area_static<T: Shape>(shape: &T) {
    println!("静态分发: {} area = {:.2}", shape.name(), shape.area());
}

// 动态分发：运行时通过 trait object 的虚表调用具体实现。
fn print_area_dynamic(shape: &dyn Shape) {
    println!("动态分发: {} area = {:.2}", shape.name(), shape.area());
}

fn create_shape(kind: &str) -> Box<dyn Shape> {
    match kind {
        "circle" => Box::new(Circle { radius: 1.0 }),
        "rect" => Box::new(Rectangle {
            width: 2.0,
            height: 3.0,
        }),
        _ => panic!("unknown shape"),
    }
}

pub fn demo() {
    println!("...............面向对象写法示例开始.................");

    let mut acc = bank_account::Account::new(String::from("Alice"));
    acc.deposit(100);
    println!("账户所有者: {}, 余额: {}", acc.owner(), acc.balance());
    println!("取款 30 是否成功: {}", acc.withdraw(30));
    println!("取款后余额: {}", acc.balance());

    let car = Car {
        name: String::from("city-car"),
        engine: Engine { horsepower: 160 },
        gps: Gps {
            position: String::from("garage"),
        },
    };
    car.drive();

    let robot = DeliveryRobot {
        id: 7,
        battery: Battery { percent: 87 },
        gps: Gps {
            position: String::from("warehouse"),
        },
    };
    robot.deliver();

    let circle = Circle { radius: 2.0 };
    let rect = Rectangle {
        width: 3.0,
        height: 4.0,
    };
    print_area_static(&circle);
    print_area_static(&rect);

    let borrowed_shapes: Vec<&dyn Shape> = vec![&circle, &rect];
    for shape in borrowed_shapes {
        print_area_dynamic(shape);
    }

    let owned_shapes: Vec<Box<dyn Shape>> = vec![create_shape("circle"), create_shape("rect")];
    for shape in owned_shapes {
        println!("Box<dyn Shape>: {} area = {:.2}", shape.name(), shape.area());
    }

    println!("...............面向对象写法示例结束.................");
}
