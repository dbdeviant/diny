use core::{marker::PhantomData, task::Context};
use crate::backend::{self, Decode, Encode, FormatDecode, FormatEncode};
use crate::io;


pub enum Encoder<F, Dta>
where
    F: FormatEncode,
{
    Init(PhantomData<* const F>, PhantomData<* const Dta>),
    V0(F::EncodeUnit),
    Fini,
}

impl<F, Dta> Encoder<F, Dta>
where
    F: FormatEncode,
{
    pub fn init() -> Self {
        Self::Init(PhantomData, PhantomData)
    }
}

impl<F, Dta> Encode for Encoder<F, Dta>
where
    F: FormatEncode,
{
    type Format = F;
    type Data = Dta;

    fn init(_data: &Self::Data) -> Self {
        Self::Init(PhantomData, PhantomData)
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, _data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
     {
        F::EncodeUnit::start_encode(format, writer, &(), cx)
        .map_pending(Self::V0)
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        match self {
            Self::Init(_, _) => encode_chain!(*self, Self::start_encode(format, writer, data, cx)),
            Self::V0(enc)    => encode_poll_fini!(*self, enc.poll_encode(format, writer, &(), cx)),
            Self::Fini       => backend::PollEncodeStatus::Error(F::invalid_input_err()),
        }
    }
}

pub enum Decoder<F, Dta>
where
    F: FormatDecode,
{
    Init(PhantomData::<*const Dta>),
    V0(F::DecodeUnit),
    Fini,
}

impl<F, Dta> Decode for Decoder<F, Dta>
where
    F: FormatDecode,
    Dta: NewUnitStruct,
{
    type Data = Dta;
    type Format = F;

    fn init() -> Self {
        Self::Init(PhantomData::<*const Dta>)
    }

    fn start_decode<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, <F as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        F::DecodeUnit::start_decode(format, reader, cx)
        .and_then(
            |()| backend::StartDecodeStatus::Fini(<Self::Data as NewUnitStruct>::new_unit_struct()),
            Self::V0
        )
    }

    fn poll_decode<R>(&mut self, format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<Self::Data, <F as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        match self {
            Self::Init(_) => decode_chain!(*self, Self, Self::start_decode(format, reader, cx)),
            Self::V0(dec) => decode_poll_fini!(*self, Self, dec.poll_decode(format, reader, cx), |_| <Self::Data as NewUnitStruct>::new_unit_struct()),
            Self::Fini    => backend::PollDecodeStatus::Error(F::invalid_input_err()),
        }
    }
}

/// Indicates that the derived serialization is for a unitary struct.
/// This is utilized because we can't rely on the Default trait being
/// defined for the struct
pub trait NewUnitStruct {
    fn new_unit_struct() -> Self;
}