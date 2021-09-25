use core::task::Context;
use futures::{AsyncRead, AsyncBufRead, AsyncWrite};    
use diny::{backend, buffer::buffer_state::BufferState};
use crate::Formatter as ThisFormat;

type Error = <ThisFormat as backend::Format>::Error;
type Data = bool;
const BUF_SIZE: usize = 1;

const TRUE:  u8 = 1;
const FALSE: u8 = 0;

#[inline(always)]
fn to_le_bytes(v: Data) -> [u8; BUF_SIZE] {
    match v {
        true  => [TRUE],
        false => [FALSE],
    }
}

#[inline(always)]
fn from_le_bytes(bytes: [u8; BUF_SIZE]) -> futures::io::Result<Data> {
    match bytes[0] {
        TRUE  => Ok(true),
        FALSE => Ok(false),
        _ => Err(<ThisFormat as backend::Format>::invalid_data_err()),
    }
}

numeric_encode_decode_def!();
serialize_all_def!    (ThisFormat, Data, Encoder);
deserialize_exact_def!(ThisFormat, Data, Decoder);