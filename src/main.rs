fn main() -> Result<(), Box<dyn std::error::Error>> {
    do_test()
}

pub fn do_test() -> Result<(), Box<dyn std::error::Error>> {
    let text = format!("hello {}", "world"); 
    foo(&text.as_str());
    Ok(())
}

fn foo(input: &str) {
    bar(input);
}

fn bar(input: &str) {
    println!("{}", input);
    panic!("test");
}
