pub use futures::io::AsyncBufRead;
pub use futures::io::AsyncWrite;
pub use futures::io::AsyncRead;
pub use futures::io::BufReader;
pub use futures::io::Error;
pub use futures::io::ErrorKind;
pub use futures::io::Result;

/// Helper module for common error functionality
pub mod error {
    use crate::io;

    /// Helper method to instantiate an InvalidInput error
    #[inline(always)]
    pub fn invalid_input() -> io::Error {
        io::ErrorKind::InvalidInput.into()
    }

    /// Helper method to instantiate an InvalidData error
    #[inline(always)]
    pub fn invalid_data() -> io::Error {
        io::ErrorKind::InvalidData.into()
    }
    
    /// Helper method to instantiate an UnexpectedEof error
    #[inline(always)]
    pub fn unexpected_eof() -> io::Error {
        io::ErrorKind::UnexpectedEof.into()
    }
    
     /// Helper method to instantiate an WriteZero error
     #[inline(always)]
    pub fn write_zero() -> io::Error {
        io::ErrorKind::WriteZero.into()
    }    
}