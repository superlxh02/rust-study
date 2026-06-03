/*
Box和Rc智能指针的基本使用
 */


use std::rc::Rc;

struct Data{
   data:String
}

impl Drop for Data {
   fn drop(&mut self) {
      println!("drop data:{}",self.data);
   }
}

pub fn demo(){
   println!("...............智能指针示例开始.................");
   let data = Box::new(Data{data:"hello world".to_string()});
   println!("data:{}",data.data);
   let str = std::rc::Rc::new("hello world".to_string());
   println!("rc count is {}",Rc::strong_count(&str));
   let str1 = str.clone();
   {
      let str2 = str.clone();
      let str3 = str.clone();
      println!("rc count is {}",Rc::strong_count(&str));
   }
   println!("rc count is {}",Rc::strong_count(&str));
   println!("...............智能指针示例结束.................");
}
