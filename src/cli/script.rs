pub fn start_script(src: &str, args: Vec<String>) {
    if false {
        println!("{args:?}");
    }
    let mut runtime = super::runtime::Runtime::new();
    runtime.input(src).run();
}
