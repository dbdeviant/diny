#[derive(Debug, Copy, Clone, PartialEq, diny::AsyncSerialization)]
pub struct EmptyStructUnitary;
#[derive(Debug, Copy, Clone, PartialEq, diny::AsyncSerialization)]
pub struct EmptyStructNewType();
#[derive(Debug, Copy, Clone, PartialEq, diny::AsyncSerialization)]
pub struct EmptyStructAnonType{}

#[derive(Debug, Copy, Clone, PartialEq, diny::AsyncSerialization)]
pub struct EmptyStruct {
    pub unitary: EmptyStructUnitary,
    pub new_type: EmptyStructNewType,
    pub anon_type: EmptyStructAnonType,
}

impl EmptyStruct {
    #[allow(unused)]
    pub fn new() -> Self {
        EmptyStruct{
            unitary: EmptyStructUnitary,
            new_type: EmptyStructNewType(),
            anon_type: EmptyStructAnonType {  },
        }    
    }
}