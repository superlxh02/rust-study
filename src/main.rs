mod study;

fn main() {
    // 基础语法 demo 是猜数字交互程序，会等待标准输入；需要时单独打开这一行运行。
    // study::base_syntax::demo();
    study::struct_demo::demo();
    study::return_handle::demo();
    study::ownership::demo();
    study::collections::demo();
    study::trait_demo::demo();
    study::trait_object::demo();
    study::generic::demo();
    study::generic_constraints::demo();
    study::smartpointer_demo::demo();
    study::smartpointer_advanced_demo::demo();
    study::closure_demo::demo();
    study::lifecycle_demo::demo();
    study::thread_demo::demo();
    study::mpsc_demo::demo();
    study::producer_comsumer::demo();
    study::concurrency_ext_demo::demo();
    study::atomic_demo::demo();
    study::oop_demo::demo();
    study::interior_mutability::demo();
    study::file_io_demo::demo();
    study::network_io_demo::demo();
    study::macro_rules_demo::demo();
    study::procmacro_demo::demo();
    study::async_demo::demo();
    study::futures_async_demo::demo();
    // 以下文档属于本次任务声明的例外或扩展主题，需要时可单独运行。
    // study::async_runtime::demo();
}
