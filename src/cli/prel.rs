pub fn start_prel() {
    let mut runtime = super::runtime::Runtime::new();

    {
        let mut str = String::new();
        str += "输入\"terl help\"获取帮助\n";
        str += "进入PREL环境\n";
        str += "Ctrl+C以退出\n";
        print!("{}", str)
    }
    loop {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                // print!("{}", input);
                if input == "?\n" {
                    println!("{:?}", runtime);
                    break;
                }
                runtime.input(&input, "114514.tl").run();
            }
            Err(_) => panic!("Err at read_line"),
        }
    }
}
