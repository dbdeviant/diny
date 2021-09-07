#[derive(Debug, PartialEq, diny::AsyncSerialization)]
pub enum MyEnum {
    Bool0(bool),
    U81(u8),
}