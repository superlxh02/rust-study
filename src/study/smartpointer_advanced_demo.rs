use std::cell::RefCell;
use std::rc::Rc;
pub fn demo(){
    println!("...............智能指针进阶示例开始.................");
    // 创建一个 Rc 包裹的 i32
    let my_data = Rc::new(100);
    // 尝试修改数据
    // 编译错误！Rc 不允许直接解引用修改，Rc包裹的数据是不可变引用，无法修改数据
    // *my_data = 200;

    //想要修改数据，需要在数据外面使用RefCell来包裹，refcell通过borrow_mut方法获取可变引用，即可修改数据
       let mut data = Rc::new(RefCell::new(1));
        println!("data: {:?}", data.clone());
        func1(& mut data);
        func2(&mut data);
        println!("data: {:?}", data.clone());
        println!("...............智能指针进阶示例结束.................");
}
fn func1(data: &mut Rc<RefCell<i32>>){
      *data.borrow_mut() = 10;
      println!("func1:{}",*data.borrow());
}
fn func2(data: &mut Rc<RefCell<i32>>){
      *data.borrow_mut() = 20;
    println!("func2:{}",*data.borrow());
}
