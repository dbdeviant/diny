#[derive(Debug, Copy, Clone, PartialEq, diny::AsyncSerialization)]
pub struct MyRec {
    pub bool_0: bool,
    pub u8_1: u8,
    pub opt_u16_2: Option<u16>,
    pub opt_u32_3: Option<Option<u32>>,
}

impl MyRec {
    #[allow(unused)]
    pub fn new() -> Self {
        MyRec{
            bool_0: true,                      
            u8_1: 2u8.pow(7),                  
            opt_u16_2: Some(2u16.pow(11)),     
            opt_u32_3: Some(Some(2u32.pow(25)))
        }    
    }
}