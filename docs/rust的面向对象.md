# rust 的 OOP 写法

## 1. 封装（Encapsulation）

### 1.1 什么是封装

封装是将数据（属性）和操作数据的方法（行为）捆绑在一起形成一个独立的单元，并对外隐藏内部实现细节，仅暴露必要的接口。

### 1.2 封装的本质

两件事：

- **数据聚合**：把相关的数据字段捆绑成一个复合类型（如 C 语言的结构体）。
- **访问控制**：决定外部代码能看到什么、能修改什么，从而保护内部状态的不变性。

### 1.3 Rust 实现封装的方法

Rust 通过 **结构体（struct）** 实现数据聚合，通过 **模块系统（mod）** 和 **可见性（pub）** 实现访问控制。

**技术点**：

- **结构体（struct）**：将零个或多个不同类型的字段组合成一个新的类型。这是数据封装的最基本形式。
- **可见性规则**：
  - 默认情况下，结构体的字段是 **私有的**（private），只能在定义它的模块及其子模块内访问。
  - 使用 `pub` 关键字可以公开字段或方法。
  - 结构体本身可以选择公开（`pub struct`）或不公开。
  - 还可以使用 `pub(crate)`、`pub(super)`、`pub(in path)` 实现更细粒度的可见性。
- **方法（impl 块）**：为结构体定义关联函数和实例方法，这些方法构成了对外操作的接口。

> 与传统 OOP 语言不同：Rust 没有 `class` 关键字，但 `struct + impl` 的组合完全等价于类。

### 1.4 代码实现

```rust
// 定义一个模块，模拟一个“类”
mod bank_account {
    // 公开结构体，但字段私有 —— 这是数据聚合
    pub struct Account {
        owner: String,   // 私有字段
        balance: i32,    // 私有字段
    }

    impl Account {
        // 公开的构造函数（通常称为 new）
        pub fn new(owner: String) -> Self {
            Account {
                owner,
                balance: 0,
            }
        }

        // 公开的方法：存款
        pub fn deposit(&mut self, amount: i32) {
            if amount > 0 {
                self.balance += amount;
            }
        }

        // 公开的方法：查询余额（只读）
        pub fn balance(&self) -> i32 {
            self.balance
        }

        // 私有方法：内部计算逻辑，外部不可见
        fn can_withdraw(&self, amount: i32) -> bool {
            self.balance >= amount
        }

        // 公开方法：取款（内部调用私有方法）
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

fn main() {
    let mut acc = bank_account::Account::new(String::from("Alice"));
    acc.deposit(100);
    println!("余额: {}", acc.balance());   // 通过方法访问
    acc.withdraw(30);
    println!("取款后余额: {}", acc.balance());

    // 以下代码无法编译，因为字段是私有的，无法直接访问：
    // println!("{}", acc.balance);        // 错误：字段 balance 私有
    // println!("{}", acc.owner);         // 错误：字段 owner 私有
    // acc.can_withdraw(10);               // 错误：私有方法
}
```

### 1.5 代码分析

- **数据聚合（struct）**：`Account` 结构体将 `owner` 和 `balance` 捆绑在一起，形成一个自包含的单元。
- **访问控制（可见性）**：
  - `pub struct Account` 使得其他模块可以创建该类型的变量、将其作为参数等，但内部的字段依然是私有的。
  - 没有 `pub` 的字段（`owner`、`balance`）外部无法直接读写，只能通过公开的方法（`deposit`、`balance`、`withdraw`）间接操作。
  - 私有方法 `can_withdraw` 是内部辅助逻辑，可以随时修改实现而不影响外部。
- **方法接口**：`new`、`deposit`、`balance`、`withdraw` 构成了这个“类”的公共接口。使用者不需要知道内部用 `balance` 这个字段存储，将来可以改为 `HashMap` 或其他数据结构，只要保持方法签名不变即可。
- **不变性保护**：`deposit` 方法只允许正数存款，`withdraw` 保证余额不会为负，这些业务规则被封装在方法内部，外部无法绕过。

> **结论**：Rust 通过 **struct（数据聚合） + 可见性控制（pub/私有）** 实现了完全且安全的封装，比许多 OOP 语言更严格（字段默认私有）。

---

## 2. 组合优于继承（Composition over Inheritance）

### 2.1 传统继承想解决什么问题

在很多面向对象语言中，继承通常用来表达两件事：

- **代码复用**：子类复用父类字段和方法。
- **类型关系**：子类被看作父类的一种，例如 `Dog is an Animal`。

这种模型在简单场景下很直观，但大型系统里容易出现几个问题：

- 父类和子类耦合过深，父类改动可能影响一串子类。
- 一个实体经常有多种能力，很难用单一继承树描述。
- 为了复用一点行为，不得不建立并不自然的父子关系。

### 2.2 Rust 的选择：不用类继承

Rust 没有类，也没有“结构体继承结构体”的机制。一个 `struct` 不能继承另一个 `struct` 的字段或方法。

这不是缺陷，而是 Rust 有意选择的设计方向：**组合优于继承**。

Rust 更鼓励你把系统拆成小的实体、小的能力、小的组件，然后按真实关系组合起来：

- 需要复用数据：把一个结构体作为另一个结构体的字段。
- 需要复用行为：把行为写在组件自己的 `impl` 中。
- 需要统一接口：使用 `trait` 描述能力。
- 需要运行时多态：使用 `dyn Trait`。

这种方式不要求所有类型都塞进一棵继承树里。一个实体可以自由拥有多个组件，也可以实现多个 trait，从而表达更灵活的关系。

### 2.3 组合的核心思想

组合表达的是 **has-a** 关系，而不是传统继承里的 **is-a** 关系。

例如：

- 汽车有发动机和 GPS。
- 配送机器人有电池和 GPS。
- 汽车和机器人都能定位，但它们不是彼此的父类或子类。

在 Rust 中，这种关系可以直接写成字段组合。

### 2.4 代码实现（用组合表达实体关系）

```rust
// 发动机是一个独立组件。
struct Engine {
    horsepower: u32,
}

impl Engine {
    fn start(&self) {
        println!("engine start, horsepower = {}", self.horsepower);
    }
}

// 电池也是一个独立组件。
struct Battery {
    percent: u8,
}

impl Battery {
    fn status(&self) {
        println!("battery status = {}%", self.percent);
    }
}

// GPS 是可复用组件。
// 汽车可以有 GPS，机器人也可以有 GPS。
struct Gps {
    position: String,
}

impl Gps {
    fn locate(&self) {
        println!("current position = {}", self.position);
    }
}

// Car 不是继承 Vehicle，而是组合需要的组件。
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

// DeliveryRobot 和 Car 没有继承关系，
// 但它可以复用同一个 Gps 组件，再组合自己的 Battery。
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

fn main() {
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
}
```

### 2.5 代码分析

- `Engine`、`Battery`、`Gps` 都是独立组件，各自管理自己的数据和行为。
- `Car` 组合了 `Engine` 和 `Gps`，因此它能启动发动机，也能定位。
- `DeliveryRobot` 组合了 `Battery` 和 `Gps`，因此它能查看电量，也能定位。
- `Car` 和 `DeliveryRobot` 复用了 `Gps`，但不需要建立共同父类。
- 如果以后需要新增 `Drone`，只要组合 `Battery`、`Gps` 或其他组件即可，不需要修改继承层级。

> **结论**：Rust 不使用类继承来组织对象关系。Rust 更倾向于把状态和能力拆成可组合的实体，通过 `struct` 字段组合和 `trait` 接口约束来表达关系。这种模型更显式、更灵活，也更符合 Rust 对所有权和类型边界的要求。

---

## 3. 多态（Polymorphism）

### 3.1 什么是多态

多态允许同一段代码操作不同的具体类型，而无需在编译时知道它们的具体类型。最常见的多态是 **子类型多态**（例如，基类指针指向派生类对象）。

### 3.2 多态的本质

**统一接口，不同实现**。核心是“延迟绑定”——在运行时决定调用哪个具体实现。

### 3.3 Rust 实现动态多态的方法

Rust 使用 **trait 对象（trait object）** 实现动态多态。
Trait 对象是一个胖指针：指向数据本身的指针 + 指向虚表（vtable）的指针。

**两种常用的 trait 对象形式**：

- `Box<dyn Trait>`：拥有所有权的 trait 对象，存放在堆上。
- `&dyn Trait`：借用（引用）的 trait 对象，不拥有所有权。

**技术要点**：

- 只能对 **对象安全（object-safe）** 的 trait 创建 trait 对象。要求 trait 的方法不能有泛型参数，且返回类型不能是 `Self`（除非该方法接收 `self` 类型为 `Box<Self>` 等特殊情况）。
- Trait 对象必须通过某种指针（`&`、`Box`、`Rc` 等）包装。
- 动态分发有少量运行时开销（一次间接跳转），但相比静态分发（单态化）可以减小二进制体积，并允许异构集合。

### 3.4 代码实现（动态多态 + 静态分发的对比）

```rust
// 定义一个对象安全的 trait
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
}
```

### 3.5 代码分析

#### 3.5.1 静态分发（`print_area_static`）

- 函数泛型参数 `T: Shape` 会在编译时对每个具体类型生成一个单独的函数副本（单态化）。
- 没有运行时开销，但会导致二进制体积增大（不过通常可接受）。
- 优点是编译器可以进行内联等优化。

#### 3.5.2 动态分发（`&dyn Shape` 或 `Box<dyn Shape>`）

- 函数接收 `&dyn Shape` 或 `Box<dyn Shape>`，是一个胖指针：数据指针 + 虚表指针。
- 调用 `shape.area()` 时，实际通过虚表找到具体类型的 `area` 函数，有少量运行时开销（一次间接跳转）。
- 可以将不同类型的 trait 对象放入同一个 `Vec` 中，实现真正的“异质集合”。
- `Box<dyn Trait>` 拥有数据的所有权，当其离开作用域时会自动释放堆上的数据。

#### 3.5.3 对象安全条件

- `Shape` 的 `area(&self)` 和 `name(&self)` 返回 `f64` 和 `&str`，没有使用 `Self` 作为返回值（除了 `self` 本身），因此对象安全。
- 如果 trait 中有 `fn new() -> Self` 这样的方法，就无法创建 trait 对象。
