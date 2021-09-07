use core::{marker::PhantomData, task::{Context, Poll}};
use futures::{AsyncRead, AsyncBufRead};
use crate::backend::{Decode, DecodeStatus, Encode, Format, FormatDecode, FormatEncode};


pub enum Encoder<F, Dta>
where
    F: FormatEncode,
{
    Init(PhantomData<*const F>, PhantomData<*const Dta>),
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

    fn start_encode<W>(format: &Self::Format, writer: &mut W, _data: &Self::Data, cx: &mut Context<'_>) -> Result<Option<Self>, <<Self as Encode>::Format as Format>::Error>
    where
        W: futures::AsyncWrite + Unpin,
     {
        F::EncodeUnit::start_encode(format, writer, &(), cx)
        .map(|ok| ok.map(Self::V0))
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, _data: &Self::Data, cx: &mut Context<'_>) -> Poll<Result<(), <<Self as Encode>::Format as Format>::Error>>
    where
        W: futures::AsyncWrite + Unpin,
    {
        let res = match self {
            Self::Init(_, _) => {
                F::EncodeUnit::start_encode(format, writer, &(), cx)
                .map(|o| match o {
                    None    => Self::Fini,
                    Some(s) => Self::V0(s),
                })
            },
            Self::V0(enc) => {
                futures::ready!(enc.poll_encode(format, writer, &(), cx))
                .map(|_| Self::Fini)
            }
            Self::Fini => {
                debug_assert!(false);
                Err(F::invalid_input_err())
            }
        };

        match res {
            Ok(enc) => {
                *self = enc;
                match self {
                    Self::Fini => Poll::Ready(Ok(())),
                    _          => Poll::Pending,
                }
            },
            Err(e) => {
                *self = Self::Fini;
                Poll::Ready(Err(e))
            }
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

    fn start_decode<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> Result<DecodeStatus<Self::Data, Self>, <F as Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        F::DecodeUnit::start_decode(format, reader, cx)
        .map(|ok| ok.bimap(|_| <Self::Data as NewUnitStruct>::new_unit_struct(), Self::V0))
    }

    fn poll_decode<R>(&mut self, format: &F, reader: &mut R, cx: &mut Context<'_>) -> ::core::task::Poll<Result<Self::Data, <F as Format>::Error>>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        let res = match self {
            Self::Init(_) => {
                F::DecodeUnit::start_decode(format, reader, cx)
                .map(|s| match s {
                    DecodeStatus::Ready  (_) => Self::Fini,
                    DecodeStatus::Pending(p) => Self::V0(p),
                })
            },
            Self::V0(dec) => {
                futures::ready!(dec.poll_decode(format, reader, cx))
                .map(|_| Self::Fini)
            }
            Self::Fini => {
                Err(F::invalid_input_err())
            }
        };

        match res {
            Ok(dec) => {
                *self = dec;
                match self {
                    Self::Fini => Poll::Ready(Ok(<Self::Data as NewUnitStruct>::new_unit_struct())),
                    _  => Poll::Pending,
                }
            },
            Err(e) => {
                *self = Self::Fini;
                Poll::Ready(Err(e))
            }
        }
    }
}

/// Indicates that the derived serialization is for a unitary struct.
/// This is utilized because we can't rely on the Default trait being
/// defined for the struct
pub trait NewUnitStruct {
    fn new_unit_struct() -> Self;
}