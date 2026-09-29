use std::fmt::Display;

pub struct Worker<T> {
    value: T,
}

impl<T: Display> Worker<T> {
    pub fn run(&self, prefix: &str) -> String {
        format!("{}:{}", prefix, self.value)
    }

    pub fn write(&self) {
        println!("{}", self.value);
    }
}
