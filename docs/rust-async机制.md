# Rust Async 机制入门

## 1. async 基础语法和使用-留下一个语法结构的印象

我们需要先建立起最直观的认识。你可以把 `async` 和 `.await` 理解为两个简单的工具：

- **`async`**：给函数或代码块贴上一个“异步”标签，让它有能力在未来“暂停”和“恢复”。
- **`.await`**：在异步函数内部，用来“等”另一个异步任务完成，并拿到它的结果。

下面我们通过代码来看它们的具体用法。

### 1.1 `async` 函数和 `async` 代码块

```rust
// 用 async fn 定义一个异步函数
async fn say_hello() -> String {
    "你好，世界".to_string()
}

// 用 async {} 定义一个异步代码块
let block = async {
    println!("这是一个异步代码块");
};
```

**关键点**：当你调用 `say_hello()` 或者执行 `async {}` 时，**函数体或代码块里面的代码并不会立即执行**。它们只是返回一个东西，这个东西在 Rust 里叫做 **`Future`**（可以暂时理解为“一张未来会兑现的票”）。

```rust
let future = say_hello();  // 此时 "你好，世界" 还没有被创建，什么也没打印
```


> 💡 这个 `Future` 对象是 **惰性** 的——它不会自己动起来，必须有东西去“推动”它，里面的代码才会真正执行。
> （不理解“惰性”和“推动”不要紧，第二部分会详细解释背后的机制，你目前只需要记住这个现象即可。）

### 1.2 `.await` —— 等待 Future 完成

如果你正在一个 `async` 函数或者 `async` 代码块里面，想要拿到 `Future` 最终产出的值，就可以在它后面写上 `.await`：

```rust
async fn get_message() -> String {
    "Hello".to_string()
}

async fn print_message() {
    let msg = get_message().await;  // 等待 get_message() 这个 Future 完成，拿到里面的 String
    println!("{}", msg);
}
```

`.await` 的作用就是：**等待这个 Future 完成，然后把结果取出来**。你可以把它想象成从一张票里“兑现”出真正的礼物。

**注意**：`.await` 只能在 `async` 函数或 `async` 代码块内部使用。在普通函数里写 `.await` 会导致编译错误。

### 1.3 总结

1. 用 `async fn` 或 `async {}` 定义异步任务，它们返回一个 **`Future`**。
2. `Future` 是惰性的，调用后不会立即执行里面的代码，需要被“驱动”才会运行。
3. 在 `async` 里面用 `.await` 来等待一个 `Future` 完成，并拿到它的结果。

> 下面的内容会深入解释 "async到底是什么" “future是什么” “为什么 Future 是惰性的”“它到底怎么被驱动的”“.await是什么“ ”.await背后做了什么”等等。如果你暂时觉得这些细节有点绕，可以先回到上面的例子，把基本用法和三个要点记住，再往后看。

---

## 2. async 机制解读

### 2.1 从 “async” 这个名字说起

在 Rust 中，你会看到一个关键字：`async`。它的中文译名是**异步**。

既然叫异步，那什么是**异步**呢？不妨先从它的反面——“同步”——开始理解。

想象两种通讯方式：

- **同步**：你给朋友打电话。拨通之后，你就把电话贴在耳边，一直等到对方说完、挂断，你才放下手机做下一件事。在这段时间里，你**只能等待**，什么都做不了。
- **异步**：你给朋友发一条微信。消息发出后，你就把手机放下，去泡咖啡、整理房间。等到朋友回复，手机“叮”的一声通知你，你再拿起手机处理回复。在等待回复的时间里，你**并没有卡住**，而是去做了其他事情。

程序世界里的同步和异步，和上面的场景很接近：

- **同步调用**：一个函数被调用后，调用者会一直等着这个函数全部执行完、返回结果，然后才能继续往下走。等待期间，调用者什么也干不了。
- **异步调用**：一个函数被调用后，它可能并不会立刻完成，而是先返回一个“未来才会完成的值”。调用者拿到这个“未来值”后，可以先去干别的事情，等这个值真正准备好再回来处理。

### 2.2 光靠 rust 语言的关键字和标准库的接口无法直接实现异步

假设你用 async 写了一个异步函数：

```rust
async fn do_something() {
    println!("doing something");
}

fn main() {
    do_something(); // 你以为会执行？
}
```

编译、运行，你会发现什么也没有发生，连打印都没有。就算你把异步任务用 `.await` 串起来：

```rust
async fn run() {
    do_something().await;
}

fn main() {
    run(); // 依然不会执行任何实际代码
}
```

`run()` 同样只是返回了一个代表“未来会完成的计算”的变量，并不会真正执行内部的 `println!`。

为什么会这样？因为 Rust 标准库只定义了异步任务的**接口**（`Future` trait）以及唤醒机制所需的基本类型，但它本身**没有提供调度和执行异步任务的引擎**。你可以这样理解：标准库给了你汽车的设计图纸和零件，但并没有给你发动机。没有发动机，车是开不走的。

这里的“发动机”，就是**运行时（runtime）**。

要理解异步运行时，可以先回想一个更熟悉的概念——操作系统的资源**调度**。

- 操作系统中有很多线程都想使用 CPU，但 CPU 核心有限。操作系统的调度器会按照一定的策略（比如轮流分配时间片、先来先服务等）决定**下一个让哪个线程运行**。
- 在 Rust 的异步世界中，我们有大量的异步任务。每一个异步任务都可以看作是一个实现了 `Future` 接口的对象。这些任务需要在某个线程上被推进执行。**运行时的作用就类似于操作系统的调度器**，只不过它调度的不是线程，而是这些异步任务（也就是 `Future`）。

具体来说，一个典型的异步运行时至少承担以下几项职责：

- **驱动任务执行**：不断调用异步任务（`Future`）的 `poll` 方法，让任务向前推进。
- **管理任务队列**：维护一个或多个任务集合，决定下一次调用哪个任务的 `poll`（例如，一个任务返回 `Pending` 后，把它放到等待队列，先去 `poll` 其他就绪任务）。
- **响应唤醒通知**：当一个任务被外部事件（如 I/O 完成、定时器到期）通过 `Waker` 唤醒时，运行时将该任务重新标记为“可运行”，并在后续的循环中再次 `poll` 它。

> 这里你只需要建立一个总体印象：**异步运行时就是一个不断挑选任务、调用其 `poll` 方法并处理返回结果的调度循环**。一个异步任务就可以看作是一个 `Future`。`poll`方法就可以看做执行任务。

因此，Rust 的 `async` 本质上是一套**面向运行时的基础设施**。它规定了异步任务该如何表示、如何被推进、如何被唤醒，但把“什么时候推进、推进谁”这些调度决策完全交给了外部运行时（如 `tokio`、`async-std`）。这就是为什么我们常说 Rust 是“零成本抽象”的异步模型——语言本身不强制任何调度策略，开发者可以根据场景选择最合适的运行时。

### 2.3 `async` 的本质：无栈协程

那么，既然光靠 rust 语言的关键字和标准库的接口无法直接实现异步，那 `async` 关键字是什么意思？本身到底做了什么？它的本质是：**无栈协程**。

我们先从“协程”这个概念讲起。

#### 2.3.1 协程是什么？

**协程定义**：一种可以暂停和恢复执行的程序组件。它能够在某一个点保存当前的执行状态，主动让出控制权，之后再从暂停的地方继续执行。现代编程语言，程序组件就是函数，所以可以狭义的理解成可暂停和恢复的函数。

**一个生活类比**：你正在看书，突然需要去接一个电话。你拿一张书签夹在当前页，合上书，去接电话。电话结束后，你重新打开书，从书签的位置继续往下读。这张书签，就是协程保存的“状态”；合上书，就是“让出控制权”；回来继续读，就是“恢复执行”。

**简单总结**：协程就是一段能够**主动暂停、稍后继续**的代码。

协程通常有两种实现方式：

- **有栈协程**：每个协程拥有自己独立的调用栈（像线程那样），切换时需要保存和恢复整个栈，开销较大，但灵活性高（比如 Go 语言的 goroutine）。
- **无栈协程**：协程不拥有独立的栈，而是通过编译器将代码重写成一个**状态机**，状态保存在一个结构体中。切换时只需要改变一下状态标记，几乎没有内存拷贝，开销极小。

Rust 的 `async` 采用的就是**无栈协程**。也就是说，编译器会把 `async` 函数或代码块转换成一个状态机，通过这个状态机来实现“暂停——恢复”的能力。

为了理解这是怎么做到的，我们需要先弄清楚“状态机”本身是什么。

#### 2.3.2 状态机是什么？

状态机（State Machine）是一个很朴素的模型：一个东西可以处于若干不同的**状态**，在某个事件发生时，会从一个状态**转换**到另一个状态。

日常生活中的红绿灯就是一个简单的状态机：

- 状态：红灯、绿灯、黄灯。
- 转换规则：红灯持续一段时间后变绿灯，绿灯变黄灯，黄灯变红灯。
- 任意时刻，红绿灯一定只处于某一个确定的状态。

在程序里，状态机通常被实现为一个枚举（表示所有可能的状态），以及一套逻辑，根据当前状态和输入决定下一个状态。

##### 2.3.2.1 一个简单的状态机代码示例

为了更具体地理解状态机在代码里长什么样，我们来看一个用 Rust 枚举实现的简单例子。假设我们要描述一个灯的开关状态转换：灯可以处于“开”或“关”两种状态，每次按下按钮，状态就在两者之间切换。

```rust
// 定义状态
enum LightState {
    On,
    Off,
}

// 状态机结构体，保存当前状态
struct Light {
    state: LightState,
}

impl Light {
    fn new() -> Self {
        Light {
            state: LightState::Off,
        }
    }

    // 按下按钮，状态转换
    fn press_button(&mut self) {
        match self.state {
            LightState::Off => {
                println!("灯亮了");
                self.state = LightState::On;
            }
            LightState::On => {
                println!("灯灭了");
                self.state = LightState::Off;
            }
        }
    }
}
```

在这个例子中：

- `LightState` 枚举就是状态机的“所有可能状态”。
- `Light` 结构体保存了当前状态，并在 `press_button` 方法中根据当前状态决定下一状态，完成转换。

**总结一下**，一个状态机至少包含两部分：

1. **当前状态**：记录现在处于哪个阶段；
2. **转移逻辑**：根据当前状态和某个触发动作，切换到下一个状态。

async 状态机也是一样的道理，只不过它的状态是“代码执行到了哪个 `.await`”，转移逻辑则是“当子 Future 就绪后，继续执行下一段代码；如果子 Future 还未就绪，就暂停等待”。

#### 2.3.3 Rust 中 `async` 的状态机

> 这里你只需要建立一个总体印象：**一个异步任务就可以看作是一个 `Future`。`poll`方法就可以看做执行任务的接口，后面讲future会具体介绍**

异步函数的状态机原理与普通状态机类似，但更复杂一些。

普通状态机通常包含两部分：

1. 当前状态；
2. 根据当前状态和某个动作，切换到下一个状态的逻辑。

而 async 状态机也可以这样理解：

1. 当前状态：记录这个 `async fn` 执行到了哪里，比如还没开始、正在等待 `step_one`、正在等待 `step_two`、已经结束；
2. 转移逻辑：每次外部调用 `poll` 时，状态机根据当前状态继续执行。如果遇到 `.await`，就 `poll` 对应的子 Future；如果子 Future 返回 `Ready`，就进入下一个状态；如果子 Future 返回 `Pending`，就保存当前状态并返回 `Pending`。

**注意**：下面的代码不是编译器真实生成的代码，也不是可以直接编译的 Rust 代码，而是为了帮助理解 async 状态机执行过程的**伪代码**。真实的编译器生成的结构要复杂得多，而且因为有 `Pin` 的存在，不能简单地像普通结构体那样直接改写字段。

以这样一个异步函数为例：

```rust
async fn my_task() {
    step_one().await;
    step_two().await;
}
```

编译器会将它转换成类似下面的状态机枚举（伪代码）：

```rust
enum MyTaskStateMachine {
    Start,
    WaitingStepOne {
        step_one_future: StepOneFuture,
    },
    WaitingStepTwo {
        step_two_future: StepTwoFuture,
    },
    Done,
}
```

- `Start`：还没开始执行。
- `WaitingStepOne`：已经创建了 `step_one()` 返回的子 Future，正在等待它完成。
- `WaitingStepTwo`：已经创建了 `step_two()` 返回的子 Future，正在等待它完成。
- `Done`：整个 async 函数执行完毕。

当我们调用 `my_task()` 时，返回的就是这个状态机的一个实例，初始状态为 `Start`。之后每次对这个实例调用 `poll` 方法，就会根据当前状态决定执行哪一段代码。

**执行过程大致是这样的：**

**第一次 `poll`：**

1. 当前状态是 `Start`；
2. 调用 `step_one()` 得到子 Future，把状态切换到 `WaitingStepOne`，并将子 Future 保存在这个状态变体中；
3. 接着立即 `poll` 这个子 Future；
4. 如果子 Future 返回 `Ready`，就继续执行，进入 `step_two()` 的等待阶段（切换到 `WaitingStepTwo`）；
5. 如果子 Future 返回 `Pending`，那么整个 `my_task` 状态机也会返回 `Pending`，控制权交还给运行时。

**第二次 `poll`（以及之后可能的更多次）：**

1. **不会从函数开头重新执行**，而是直接从 `WaitingStepOne` 状态恢复；
2. 再次 `poll` 之前保存的那个 `step_one_future`；
3. 如果它这次返回 `Ready`，就继续创建 `step_two()` 的子 Future，进入 `WaitingStepTwo` 状态，并立即 `poll` 它；
4. 如果 `step_two_future` 也立刻 `Ready`，状态变为 `Done`，整体返回 `Ready`；
5. 如果任何一次子 Future 返回 `Pending`，当前状态机就保存好状态并返回 `Pending`，等待下一次被唤醒后继续。

这个过程中，状态机在每次 `poll` 时，根据当前状态决定执行哪一段代码，并且可能多次返回 `Pending`，暂停在某个 `.await` 点。下次再被 `poll` 时，直接从上次暂停的状态继续执行，而不是从头开始。这就是无栈协程通过状态机实现“暂停—恢复”的核心原理。

#### 2.3.4 协程与异步的关系

那么，协程和“异步”有什么关系？——毕竟关键字名就叫 `async`。

**答：协程可以用来实现异步，是异步的实现方式之一**

 **进一步理解异步的本质**

- **异步是一种代码运作的模式，本身并无绑定任何底层实现机制**
- **异步的实现方式不只有协程一种**
- **异步也可以基于线程或者进程**

异步编程的核心诉求是：**当一个任务需要等待（比如等待网络数据）时，不阻塞当前线程，而是让出执行权，让线程可以先去处理其他任务**。

协程的“暂停——恢复”能力，恰好完美契合了这个需求：
在 `.await` 处，如果子任务未就绪，当前协程就可以**暂停**，把执行权交还给运行时；等到子任务就绪后，运行时再**恢复**该协程，从上次暂停的地方继续执行。整个过程中，线程没有被阻塞，而是持续在各协程之间切换推进。

于是，Rust 选择将 `async` 函数编译成状态机形式的无栈协程，让它天然具备“可暂停、可恢复”的特性，再通过 `Future` trait 暴露给运行时。这样，开发者写异步代码时，看到的只是顺序的 `.await`，而背后运行时则高效地在各种任务间切换。正因为这种机制直接服务于“不阻塞等待”的异步范式，所以关键字取名为 `async`。

### 2.4 Future trait —— 异步世界的统一接口

```rust
// 用 async fn 定义一个异步函数
async fn say_hello() -> String {
    "你好，世界".to_string()
}

let fut = say_hello();
```

这段代码里 被 `async`修饰的 `say_hello`的函数，他的返回值 `fut`你觉得会是什么类型？

fut是实现了 `future trait`的 `future`对象类型,**你可能会疑惑明明函数定义的时候返回是String，这个疑问会在下面做一个解答**。

**`future` 你可以理解为未来量，或者是未来将要完成的异步任务。一个future也可以看做是一个异步任务**

#### 2.4.1 `Future` trait 的定义

在 Rust 标准库中，`Future` trait 的核心定义如下（简化版）：

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

这个 trait 只有一个核心方法：`poll`。第一次看到这个签名，你可能会觉得里面的 `Pin`、`Context`、`Poll` 很陌生。别着急，我们一步一步拆开来看。

#### 2.4.2 `type Output` —— 最终结果类型

`type Output;` 是一个关联类型，它表示这个 Future 最终完成时会产出的值的类型。例如：

```rust
async fn foo() -> i32 { 10 }
```

`foo()` 返回的 Future 的 `Output` 就是 `i32`。如果函数没有显式返回值，那就是 `()`。

#### 2.4.3 `poll` —— 推进执行的唯一入口

`poll` 方法的意思是：**尝试推进这个 Future 的执行**。你可以把它理解成异步任务的“执行按钮”，每按一次，任务就往前走一步，直到它彻底完成或者被某个等待点卡住。之前提到过future对象是惰性的——它不会自行启动，必须由异步运行时（或者手写驱动循环）通过 poll 方法不断推进，才能真正执行内部的代码。

**`poll` 就是执行 `async fn` 的函数**。当运行时调用一个 Future 的 `poll` 时，编译器生成的状态机就开始运转。

#### 2.4.4 `Poll` 枚举——完成还是等待？

`poll` 的返回值是 `Poll<Self::Output>`。`Poll` 是一个枚举：

```rust
pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

- `Poll::Ready(T)`：任务已经完成，`T` 是最终结果。
- `Poll::Pending`：任务还没有完成，现在需要等待某些条件（比如网络数据未到达），暂时无法继续。
  如果 `poll` 返回 `Pending`，运行时就知道这个任务目前必须暂停，应该转而去调度其他任务。等将来条件满足，再重新 `poll` 这个任务。

**这里就解答了之前的疑惑：future调用poll以后，如果返回值是Poll::Ready(T),T就是定义async函数的时候的返回值，可以通过 match匹配Poll::Ready(T)，获取到返回值**

#### 2.4.5 `Context` 与 `Waker` —— 唤醒的纽带

`poll` 的第二个参数是 `cx: &mut Context<'_>`。`Context` 目前主要用来提供 `Waker`。`Waker` 是一个可以被用来“唤醒任务”的句柄。

它们的作用大致如下：

- 当一个 Future 在执行过程中发现自己暂时无法继续（例如数据没到），它就会返回 `Poll::Pending`，并利用 `cx.waker()` 拿到 `Waker`，把它注册到某个事件源上。
- 当事件就绪后，事件源调用 `Waker` 上的 `wake()` 方法，通知运行时：“之前等待的那个任务现在可以继续了！”
- 运行时收到通知后，会在合适的时机重新 `poll` 这个 Future。

> 注意：`Waker` 并不是直接恢复 Future 执行的，它只是给运行时发一个“可以重试了”的信号。真正负责再次 `poll` 的仍然是运行时。

#### 2.4.6 `Pin<&mut Self>` —— 为什么 poll 不能是普通的 `&mut self`

现在来看 `poll` 签名中最让人困惑的部分：`self: Pin<&mut Self>`。

要理解为什么需要 `Pin`，得从异步状态机的内部结构说起。

前面我们说过，`async fn` 会被编译成一个状态机。状态机在不同的 `.await` 点之间可能需要保存局部变量。比如：

```rust
async fn example() {
    let a = 10;
    let b = &a;          // b 引用了 a
    some_future().await; // 这里可能暂停
    println!("{}", b);   // 恢复后继续使用 b
}
```

在这个函数里，变量 `b` 是一个指向 `a` 的引用。编译器在生成状态机时，必须同时保存 `a` 和 `b`。这意味着状态机结构体内部可能出现**自引用**：一个字段是另一个字段的引用。

例如，编译器生成的状态机可能在内存中长这样：

```text
状态机结构体 {
    a: 10,
    b: &a,   // 指向同一结构体内的字段 a
}
```

如果这个状态机结构体被**移动**到另一个内存地址，`a` 的位置变了，但 `b` 还指向原来的地址，那么 `b` 就成了悬垂引用，会导致未定义行为。

常规的 Rust 引用（`&`）并不保证被引用对象的内存地址不变，因此这种自引用结构在可以自由移动的情况下是危险的。而 `Future` 可能会在多个 `poll` 调用之间被传递、放入队列等，如果它的内存地址发生变化，内部的自引用就会失效。

为了解决这个问题，Rust 引入了 `Pin`。`Pin` 是一个标记类型，它的核心语义是：**对于被 `Pin` 包裹且没有实现 `Unpin` 的值，安全代码不能把它从当前内存位置移走**。当 `poll` 要求 `Pin<&mut Self>` 时，它等于承诺：在 Future 被固定之后，它的内存地址不会再被安全代码随意改变。这样编译器就可以安全地处理可能跨 `.await` 保存引用的状态机。

简单来说：

- **自引用结构**：异步状态机可能包含指向自身的引用。
- **移动问题**：如果状态机被移动，那些自引用会变成悬垂指针。
- **Pin 的保证**：对 `!Unpin` 类型来说，`Pin` 禁止安全代码移动其被指向的值，确保内部引用不会失效。
- **Unpin**：大多数普通类型都是 `Unpin` 的，`Pin` 对它们不施加额外的移动限制；而编译器生成的 async 状态机通常是 `!Unpin` 的，所以必须通过 `Pin` 来访问。

在实际使用中，我们通常用以下方式固定一个 Future：

- **栈上固定**：使用 `std::pin::pin!` 宏，例如：
  ```rust
  let mut fut = std::pin::pin!(some_async_fn());
  ```
- **堆上固定**：使用 `Box::pin`，例如：
  ```rust
  let mut fut = Box::pin(some_async_fn());
  ```

然后通过 `as_mut()` 获取 `Pin<&mut Future>`，再调用 `poll`：

```rust
fut.as_mut().poll(&mut cx);
```

初学阶段不必深究 `Pin` 的全部细节，只需要记住：**因为 async 状态机可能自引用，所以 poll 要求 Future 不能被移动，要用 Pin 固定住。使用 `pin!` 或 `Box::pin` 把 Future 固定好，然后通过 `as_mut()` 调用 poll 即可。**

### 2.5 `.await` 的深入理解

前面我们讲了 `async fn` 会返回 Future，也讲了 Future 需要被 `poll` 推进。接下来我们结合已经学过的机制，更深入地理解 `.await`。

#### 2.5.1 `.await` 的背后：poll 的调用链

当你写：

```rust
let value = child_future.await;
```

这里要特别注意一个容易误解的地方：**`.await` 本身并不会“凭空”调用 `poll`，它是在父 Future 被 `poll` 的过程中才起作用的。**

整个调用链是这样的：

1. **运行时（或你手写的驱动循环）** 调用最外层 Future（即当前这个 `async fn` 返回的那个 Future）的 `poll` 方法。
2. 在这个最外层 Future 的 `poll` 方法内部，编译器生成的状态机代码会执行到 `.await` 的位置。此时，编译器生成的代码会去**调用子 Future（即 `child_future`）的 `poll` 方法**。
3. 根据子 Future 的 `poll` 返回值决定下一步：
   - 如果返回 `Poll::Ready(value)`，说明子 Future 已经完成，`.await` 表达式的结果就是这个 `value`，父 Future 继续执行后面的代码。
   - 如果返回 `Poll::Pending`，说明子 Future 还没完成，于是父 Future 也会暂停，并向上返回 `Poll::Pending`，最终把控制权交还给运行时。

可以总结为：

> **运行时 poll 父 Future → 父 Future 在 `.await` 点 poll 子 Future → 子 Future Ready 则父 Future 继续，子 Future Pending 则父 Future Pending**

也就是说，`.await` 是编译器在父 Future 的状态机中插入的一段逻辑，它负责在父 Future 被驱动时去推进子 Future，但它自己不会独立地、在运行时外部去调用 `poll`。

#### 2.5.2 `.await` 不一定会挂起

很多初学者会以为：只要写了 `.await`，任务就一定会挂起。这是不对的。

`.await` 是一个**可能挂起点**，不是一定挂起点。例如：

```rust
async fn simple() {
    println!("simple");
}

async fn run() {
    println!("run start");
    simple().await;
    println!("run end");
}
```

`simple()` 里面没有任何真正需要等待的东西，所以它第一次被 `poll` 时就会直接完成。因此 `simple().await` 会立刻拿到结果，`run` 会继续执行，输出：

```
run start
simple
run end
```

不会出现 `Pending`。只有当被 await 的子 Future 返回 `Poll::Pending` 时，当前 Future 才会暂停。

#### 2.5.3 多个 `.await` 默认是顺序执行

例如：

```rust
async fn main_task() {
    task1().await;
    task2().await;
    task3().await;
}
```

这段代码默认是顺序执行的：先等待 `task1` 完成，再等待 `task2`，最后等待 `task3`。如果 `task1` 还没有完成，那么 `task2` 根本不会开始执行。

这和很多人想象中的“异步自动并发”不一样。`.await` 的意思是**等待当前这个 Future 完成，再继续往下执行**。

如果你想让多个任务并发执行，需要使用运行时提供的 `spawn`，或者使用类似 `join` 的工具。例如在 tokio 中：

```rust
let h1 = tokio::spawn(task1());
let h2 = tokio::spawn(task2());

h1.await.unwrap();
h2.await.unwrap();
```

这里 `task1` 和 `task2` 才是作为两个独立任务交给运行时调度。所以要区分：

- `task1().await`：当前任务等待 `task1` 完成。
- `spawn(task1())`：创建一个独立任务，让运行时单独调度。
- `join`：同时推进多个 Future，等待它们都完成。

#### 2.5.4 父 Future 和子 Future 的嵌套关系

假设有嵌套的异步函数：

```rust
async fn leaf() {
    println!("leaf");
}

async fn child() {
    leaf().await;
}

async fn parent() {
    child().await;
}
```

运行时通常只需要管理 `parent()` 返回的那个最外层 Future。当运行时 poll `parent` 时，`parent` 内部执行到 `child().await`，这时 `parent` 这个状态机会去 poll `child`。而 `child` 内部又会去 poll `leaf`。

调用链可以理解为：

```
runtime poll parent
    parent poll child
        child poll leaf
```

对于普通 `.await` 嵌套，运行时不需要直接拿到每一个子 Future。它只要 poll 最外层 Future，子 Future 会由父 Future 的状态机继续向下 poll。这也是 Rust async 状态机非常关键的地方——**任务树是由状态机层层嵌套驱动，而不是由运行时逐个管理所有子任务**。

### 2.6 示例代码及其讲解

下面，我们用示例代码亲手体验上面所讲的一切。我们完全脱离第三方运行时，用最原始的手动 `poll` 方式驱动 `Future`，看清它内部的执行顺序。

#### 2.6.1 完整示例代码

```rust
use std::task::Poll;

// 简单异步任务
async fn simple_async_task() {
    println!("    simple async task");
}

// 简单异步任务示例
async fn simple_demo() -> String {
    println!("async simple demo start");
    simple_async_task().await;
    println!("async simple demo end");
    "OK".to_string()
}

fn demo1() {
    // 使用 pin 包裹 simple_demo 的返回值，simple_demo 的返回值是 Future 类型
    let mut fut = std::pin::pin!(simple_demo());
    // 创建一个空的 waker
    let waker = std::task::Waker::noop();
    // 基于 waker 构造一个 context
    let mut context = std::task::Context::from_waker(&waker);
    // 调用 poll 推进异步任务——此时异步任务会开始执行
    let res = fut.poll(&mut context);
    // match 匹配表达式
    match res {
        Poll::Pending => {
            println!("simple demo future pending");
        }
        Poll::Ready(res) => {
            println!("simple demo future ready: {:?}", res);
        }
    }
}

// 多个异步任务示例
async fn multiple_demo() -> String {
    println!("async multiple demo start");
    simple_async_task().await;
    simple_async_task().await;
    simple_async_task().await;
    println!("async multiple demo end");
    "OK".to_string()
}

fn demo2() {
    let mut fut = std::pin::pin!(multiple_demo());
    let waker = std::task::Waker::noop();
    let mut context = std::task::Context::from_waker(&waker);
    loop {
        let res = fut.as_mut().poll(&mut context);
        match res {
            Poll::Pending => {
                println!("multiple demo future pending");
            }
            Poll::Ready(res) => {
                println!("multiple demo future ready: {:?}", res);
                break;
            }
        }
    }
}

pub fn demo() {
    println!("..............async 示例代码开始.....................");
    demo1();
    demo2();
    println!("..............async 示例代码结束.....................");
}
```

#### 2.6.2 代码讲解

我们一边看代码，一边解释它运行时的行为。

##### 2.6.2.1 最简单的异步任务 —— `simple_async_task`

```rust
async fn simple_async_task() {
    println!("    simple async task");
}
```

- 这是一个用 `async` 声明的函数，编译器会为它生成状态机。
- 函数内部没有任何 `.await`，所以它没有暂停点。一旦被 `poll`，就会一口气执行完，返回 `Poll::Ready(())`。
- 只调用 `simple_async_task()` 不会执行任何代码，仅仅返回一个 Future。

##### 2.6.2.2 包含一次 `.await` 的任务 —— `simple_demo`

```rust
async fn simple_demo() -> String {
    println!("async simple demo start");
    simple_async_task().await;
    println!("async simple demo end");
    "OK".to_string()
}
```

- 函数内有一个 `.await`，因此编译器生成的状态机至少包含两个阶段。
- 因为 `simple_async_task` 没有阻塞点，所以 `simple_demo` 被 `poll` 时会一次性走完所有流程：
  1. 打印 `async simple demo start`
  2. 执行到 `.await`，推动 `simple_async_task`，它立即完成
  3. 打印 `async simple demo end`
  4. 返回 `"OK"`
- 最终 `poll` 返回 `Poll::Ready("OK".to_string())`。

##### 2.6.2.3 `demo1` —— 手动驱动一个 Future

```rust
let mut fut = std::pin::pin!(simple_demo());
```

- `simple_demo()` 返回一个 Future。`std::pin::pin!` 宏把它“钉”在栈上，保证内存地址不变。这是为了满足 `poll` 的 `Pin` 要求，初学者可先当作固定写法。

```rust
let waker = std::task::Waker::noop();
let mut context = std::task::Context::from_waker(&waker);
```

- 创建一个空的 `Waker`，它不执行任何实际唤醒。用它构造 `Context`，对于这个没有真正等待点的示例完全够用。

```rust
let res = fut.poll(&mut context);
```

- 我们亲手调用了 `poll` 此时 `simple_demo` 的状态机开始运转，顺序走完全部流程，最终返回 `Ready`。

```rust
match res {
    Poll::Pending => { println!("simple demo future pending"); }
    Poll::Ready(res) => { println!("simple demo future ready: {:?}", res); }
}
```

- 因为任务不阻塞，`res` 必定是 `Ready`，所以输出：`simple demo future ready: "OK"`

##### 2.6.2.4 多个 `.await` 的任务 —— `multiple_demo`

```rust
async fn multiple_demo() -> String {
    println!("async multiple demo start");
    simple_async_task().await;
    simple_async_task().await;
    simple_async_task().await;
    println!("async multiple demo end");
    "OK".to_string()
}
```

- 虽然有三个 `.await`，但因为每个 `simple_async_task` 都没有等待点，所以一次 `poll` 就会连续打印三行，最后返回 `Ready`。这说明 `.await` 只是“可能挂起点”，不一定会真的挂起。

##### 2.6.2.5 `demo2` —— 循环 `poll` 模拟运行时

```rust
let mut fut = std::pin::pin!(multiple_demo());
let waker = std::task::Waker::noop();
let mut context = std::task::Context::from_waker(&waker);
loop {
    let res = fut.as_mut().poll(&mut context);
    match res {
        Poll::Pending => {
            println!("multiple demo future pending");
        }
        Poll::Ready(res) => {
            println!("multiple demo future ready: {:?}", res);
            break;
        }
    }
}
```

- 这里用一个 `loop` 反复调用 `poll`，模拟了一个极简的“异步运行时调度循环”。
- 如果某次 `poll` 返回 `Pending`，我们打印一行 `pending` 并继续循环（真实运行时会切换到其他任务）。
- 如果返回 `Ready`，打印结果并退出循环。
- 因为 `multiple_demo` 没有真正的阻塞，第一次 `poll` 就直接完成，不会输出 `Pending`。
