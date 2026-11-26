#[speedyspec_port_macros::port(edge)]
pub trait P {
    fn get(&self) -> i64;
}
fn main() {}
