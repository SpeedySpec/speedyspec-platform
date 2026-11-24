#[speedyspec_port_macros::port(key = "T\\P")]
pub trait P {
    fn get<T>(&self, v: T) -> i64;
}
fn main() {}
