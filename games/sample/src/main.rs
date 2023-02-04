#![allow(dead_code)]

mod context;
mod helpers;
mod items;

fn main() {
    println!("Hello, world!");
    helper__Nuts!();
    helper__can_use!("Slingshot");
    helper___is_child_item!("Slingshot");
    println!("Slingshot = {:?}", items::Items::Slingshot);
}
