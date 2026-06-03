use rand::Rng;

pub fn demo() {
    println!("...............基础语法示例开始...............");
    guess_number();
    guess_number();
    println!("...............基础语法示例结束...............");
}
fn guess_number() {
    let secret_number = rand::thread_rng().gen_range(1..=100);
    let mut guess = String::new();
    let mut count = 1;

    while count <= 20 {
        println!("the {} time, please input a number :", count);
        let n = std::io::stdin().read_line(&mut guess).unwrap();
        count += 1;
        let num = guess.trim().parse::<u32>();
        match num {
            Ok(num) => {
                if num == secret_number {
                    println!("you guess right");
                    break;
                } else if num > secret_number {
                    println!("you guess too big");
                } else {
                    println!("you guess too small");
                }
            }
            Err(_) => println!("please input a number"),
        }
        guess.clear();
    }
    if count > 20 {
        println!("you fail");
    }
}
