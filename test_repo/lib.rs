/// Greets the world.
pub fn greet_world() {
    println!("Hello, world!");
}

struct User {
    name: String,
}

impl User {
    fn new(name: String) -> Self {
        Self { name }
    }
}
