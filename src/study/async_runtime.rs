use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

struct Executor {
    queue: VecDeque<Arc<Task>>,
    task_count: usize,
    work_thread: Thread,
}

//任务结构体，runtime调度的单位
struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>, //类型擦除的future
    is_queued: AtomicBool,                                                     //判断是否在队列里
    executor: Weak<Mutex<Executor>>,                                           //执行器的弱引用
}

impl Task {
    //逻辑就是把自己放入队列
    fn schedule(self: &Arc<Self>) {
        // 如果之前就是 true，说明任务已经在队列里，不需要重复入队。
        if self.is_queued.swap(true, Ordering::SeqCst) {
            return;
        }

        //如果对象还存活，拿到executor
        let Some(executor) = self.executor.upgrade() else {
            return;
        };

        //加锁
        let mut executor = executor.lock().unwrap();

        //把自己放入队列
        executor.queue.push_back(self.clone());

        //唤醒线程
        executor.work_thread.unpark();
    }

    fn poll_once(self: &Arc<Self>) {
        // 从 Task 中取出 Future, self.future变为none了
        let Some(mut future) = self.future.lock().unwrap().take() else {
            return;
        };

        //基于自己构建一个waker
        let waker = Waker::from(self.clone());

        //基于waker构建一个context
        let mut context = Context::from_waker(&waker);

        //poll推进一次，如果完成，task数量-1，如果没有完成，放回task。
        match future.as_mut().poll(&mut context) {
            Poll::Ready(()) => {
                if let Some(executor) = self.executor.upgrade() {
                    let mut executor = executor.lock().unwrap();
                    debug_assert!(executor.task_count > 0);
                    executor.task_count -= 1;
                }
            }
            Poll::Pending => {
                //再把future塞到 self.future里
                *self.future.lock().unwrap() = Some(future);
            }
        }
    }
}

//为task自己实现wake trait ，让他自己能够变成waker
impl Wake for Task {
    //唤醒逻辑就是 就是再次把自己放入队列里等待被runtime poll
    fn wake(self: Arc<Self>) {
        self.schedule();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.schedule();
    }
}

//runtime持有一个executor
struct Runtime {
    executor: Arc<Mutex<Executor>>,
}

impl Runtime {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            executor: Arc::new(Mutex::new(Executor {
                queue: VecDeque::new(),
                task_count: 0,
                work_thread: thread::current(),
            })),
        })
    }

    //异步提交task
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        //基于传入的async 构造task
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(future))),
            executor: Arc::downgrade(&self.executor),
            is_queued: AtomicBool::new(true),
        });

        //拿到executor
        let mut executor = self.executor.lock().unwrap();

        executor.task_count += 1;

        //加入队列
        executor.queue.push_back(task);

        //唤醒线程
        executor.work_thread.unpark();
    }

    //阻塞运行的主逻辑-死循环，不断从队列拿出task进行poll
    fn run(&self) {
        loop {
            //从队列拿出task
            let task = {
                let mut executor = self.executor.lock().unwrap();

                if executor.task_count == 0 {
                    return;
                }

                let task = executor.queue.pop_front();

                if let Some(task) = &task {
                    // task 已经从队列里拿出来了，所以标记为“不在队列中”。
                    task.is_queued.store(false, Ordering::SeqCst);
                }

                task
            };

            match task {
                Some(task) => {
                    task.poll_once();
                }
                None => {
                    // 队列为空，但还有任务没完成。
                    // 说明有任务 Pending，runtime 需要等待某个 waker 唤醒它。
                    thread::park();
                }
            }
        }
    }

    //阻塞完成
    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        //创建空的结果
        let result = Arc::new(Mutex::new(None));
        let result_clone = result.clone();

        //异步提交，把结果放到result_clone
        self.spawn(async move {
            let value = future.await;
            *result_clone.lock().unwrap() = Some(value);
        });

        //阻塞运行
        self.run();

        //返回结果
        result
            .lock()
            .unwrap()
            .take()
            .expect("block_on did not produce an output")
    }
}

//构造一个pending的结果，让task主动挂起
struct YieldNow {
    is_yielded: bool, //是否已经主动挂起
}

impl Future for YieldNow {
    type Output = ();

    //poll的逻辑，第一次被 poll 时主动让出执行权并且调用waker让runtime下次还可以poll自己，第二次被 poll 时完成。
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_yielded {
            Poll::Ready(())
        } else {
            self.is_yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

//构造一个yieldnow结构体
fn yield_now() -> YieldNow {
    YieldNow { is_yielded: false }
}

//异步task
async fn sub_task(n: i32) {
    println!("task{} start", n);
    yield_now().await;
    println!("task{} done", n);
}

async fn main_task(runtime: Arc<Runtime>) {
    println!("main_task start");

    sub_task(1).await;

    runtime.spawn(sub_task(3));
    runtime.spawn(sub_task(4));

    sub_task(2).await;

    println!("main_task done");
}

pub fn demo() {
    println!("..............开始自定义异步运行时演示......................");
    let runtime = Runtime::new();
    runtime.block_on(main_task(runtime.clone()));
    println!("..............自定义异步运行时演示结束......................");
}
