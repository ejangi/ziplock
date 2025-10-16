fn main() {
    println!("Hello from ZipLock test build!");
    println!("Testing GNU toolchain compilation...");

    // Test some basic functionality
    let test_string = "Testing GNU toolchain";
    println!("String test: {}", test_string);

    // Test vector allocation
    let mut test_vec = Vec::new();
    for i in 1..=5 {
        test_vec.push(i);
    }
    println!("Vector test: {:?}", test_vec);

    // Test file system access
    match std::env::current_dir() {
        Ok(path) => println!("Current directory: {}", path.display()),
        Err(e) => println!("Error getting current directory: {}", e),
    }

    println!("Test build successful!");
}
