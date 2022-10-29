fn main() {
    // unsafe read & write
    let mut a: usize = 1;
    unsafe {
        let ptr_a = &mut a as *mut usize;
        std::ptr::write(ptr_a, 2);
    }
    println!("a is {a}");

    use terl::cli::program_begin;
    program_begin();
}
