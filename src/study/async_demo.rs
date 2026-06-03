use std::task::Poll;

// 简单异步任务
async fn simple_async_task(){
    println!("async task");
}

// 简单异步任务示例
async fn simple_demo() ->String{
    println!("async simple demo start");
    simple_async_task().await;
    println!("async simple demo end");
    "OK".to_string()
}


fn demo1(){
    //使用pin包裹simple_demo的返回值，simple_demo的返回值是future类型
    let mut fut = std::pin::pin!(simple_demo());
    //创建一个空的waker
    let waker = std::task::Waker::noop();
    //基于waker构造一个context
    let mut context = std::task::Context::from_waker(&waker);
    //调用poll推进异步任务-此时异步任务会开始执行
    let res =  fut.poll(&mut context);
    //match匹配表达式
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
async fn multiple_demo() ->String{
    println!("async multiple demo start");
    simple_async_task().await;
    simple_async_task().await;
    simple_async_task().await;
    println!("async multiple demo end");
    "OK".to_string()
}
fn demo2(){
    let  mut fut = std::pin::pin!(multiple_demo());
    let waker = std::task::Waker::noop();
    let mut context = std::task::Context::from_waker(&waker);
    loop{
        let res =  fut.as_mut().poll(&mut context);
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

pub fn demo(){
    println!("..............async 示例代码开始.....................");
    demo1();
    demo2();
    println!("..............async 示例代码结束.....................");
}