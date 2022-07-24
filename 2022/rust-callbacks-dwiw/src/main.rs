// SPDX-License-Identifier: MIT

pub struct MyStruct<'a> {
    callbacks: Vec<Box<dyn Fn(usize) -> usize + 'a>>,
}

impl<'a> MyStruct<'a> {
    pub fn new() -> MyStruct<'a> {
        MyStruct { callbacks: Vec::new() }
    }

    pub fn add_callback(&mut self, func: impl Fn(usize) -> usize + 'a) {
        self.callbacks.push(Box::new(func));
    }

    pub fn trigger(&self, val: usize) {
        for (i, cb) in self.callbacks.iter().enumerate() {
            println!("{}: {}", i, cb(val));
        }
    }
}


fn cb1(val: usize) -> usize { val + 1 }

fn main() {
    let borrowed_content = String::from("Foo");
    let moved_content_2 = String::from("Foo");
    let cb3 = move |n| n + moved_content_2.len();
    let cb4 = |n| n + borrowed_content.len();

    // "borrowed_content" must outlive "obj". It is sufficient to declare
    // "obj" second, but I've created an explicit scope here for clarity.
    {
        let mut obj = MyStruct::new();
        let moved_content = String::from("Foo");
        let cb2 = |n| n + 4;

        // Functions (reusable)
        obj.add_callback(cb1);
        obj.add_callback(cb1);
        // Lambdas
        obj.add_callback(|n| n + 3);
        obj.add_callback(cb2);
        obj.add_callback(cb2);
        // Closure owning its variables
        obj.add_callback(move |n| n + moved_content.len());
        obj.add_callback(&cb3);
        obj.add_callback(&cb3);
        // Closure borrowing its variables
        obj.add_callback(|n| n + borrowed_content.len());
        obj.add_callback(&cb4);
        obj.add_callback(&cb4);

        obj.trigger(6);
    }
}
