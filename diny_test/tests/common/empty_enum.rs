#[derive(Debug, PartialEq, diny::AsyncSerialization)]
pub enum EmptyEnum {
    Unitary,
    NewType(),
    AnonType{},
}