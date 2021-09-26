use core::task::Context;
use diny::{backend, buffer::buffer_state::BufferState, io};
use crate::Formatter as ThisFormat;

type Error = <ThisFormat as backend::Format>::Error;
type Data = char;
const BUF_SIZE: usize = 4;

#[inline(always)]
fn to_le_bytes(v: Data) -> [u8; BUF_SIZE] {
    (v as u32).to_le_bytes()
}

#[inline(always)]
fn from_le_bytes(bytes: [u8; BUF_SIZE]) -> io::Result<Data> {
    let u = u32::from_le_bytes(bytes);
    match core::char::from_u32(u) {
        None    => Err(<ThisFormat as backend::Format>::invalid_data_err()),
        Some(c) => Ok(c),
    }
}

numeric_encode_decode_def!();
serialize_all_def!    (ThisFormat, Data, Encoder);
deserialize_exact_def!(ThisFormat, Data, Decoder);