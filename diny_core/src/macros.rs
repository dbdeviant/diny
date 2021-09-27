#[macro_export]
/// Starts [encoding](crate::backend::Encode) by calling the `$enc` expression, and converts the
/// result to a [PollEncodeStatus](crate::backend::PollEncodeStatus)
macro_rules! encode_chain {
    ($lhs: expr, $enc: expr) => {
        match $enc {
            $crate::backend::StartEncodeStatus::Fini => {
                $lhs = Self::Fini;
                $crate::backend::PollEncodeStatus::Fini
            }
            $crate::backend::StartEncodeStatus::Pending(enc) => {
                $lhs = enc;
                $crate::backend::PollEncodeStatus::Pending
            }
            $crate::backend::StartEncodeStatus::Error(e) => {
                $lhs = Self::Fini;
                $crate::backend::PollEncodeStatus::Error(e)
            }
        }
    };
}

/// Continues [encoding](crate::backend::Encode) by first polling the `$enc` expression, and then
/// calling the `$chain` expression, and finally converts the
/// result to a [PollEncodeStatus](crate::backend::PollEncodeStatus)
#[macro_export]
macro_rules! encode_poll_chain {
    ($lhs: expr, $enc: expr, $chain: expr) => {
        match $enc {
            $crate::backend::PollEncodeStatus::Fini => $crate::encode_chain!($lhs, $chain),
            $crate::backend::PollEncodeStatus::Pending => {
                $crate::backend::PollEncodeStatus::Pending
            }
            $crate::backend::PollEncodeStatus::Error(e) => {
                $lhs = Self::Fini;
                $crate::backend::PollEncodeStatus::Error(e)
            }
        }
    };
}

/// Finalizes [encoding](crate::backend::Encode) by polling the `$enc` expression, and converts
/// the result to a [PollEncodeStatus](crate::backend::PollEncodeStatus)
#[macro_export]
macro_rules! encode_poll_fini {
    ($lhs: expr, $enc: expr) => {
        match $enc {
            $crate::backend::PollEncodeStatus::Fini => {
                $lhs = Self::Fini;
                $crate::backend::PollEncodeStatus::Fini
            }
            $crate::backend::PollEncodeStatus::Pending => {
                $crate::backend::PollEncodeStatus::Pending
            }
            $crate::backend::PollEncodeStatus::Error(e) => {
                $lhs = Self::Fini;
                $crate::backend::PollEncodeStatus::Error(e)
            }
        }
    };
}


/// Starts [decoding](crate::backend::Decode) by calling the `$dec` expression, and converts the
/// result to a [PollDecodeStatus](crate::backend::PollDecodeStatus)
#[macro_export]
macro_rules! decode_chain {
    ($lhs: expr, $rhs: ident, $dec: expr) => {
        match $dec {
            $crate::backend::StartDecodeStatus::Fini(d) => {
                $lhs = $rhs::Fini;
                $crate::backend::PollDecodeStatus::Fini(d)
            }
            $crate::backend::StartDecodeStatus::Pending(enc) => {
                $lhs = enc;
                $crate::backend::PollDecodeStatus::Pending
            }
            $crate::backend::StartDecodeStatus::Error(e) => {
                $lhs = $rhs::Fini;
                $crate::backend::PollDecodeStatus::Error(e)
            }
        }
    };
}

/// Continues [decoding](crate::backend::Decode) by first polling the `$dec` expression, and then
/// calling the `$chain` closure with any resulting data, and finally converts the
/// result to a [PollDecodeStatus](crate::backend::PollDecodeStatus)
#[macro_export]
macro_rules! decode_poll_chain {
    ($lhs: expr, $rhs: ident, $dec: expr, $chain: expr) => {
        match $dec {
            #[allow(clippy::redundant_closure_call)]
            $crate::backend::PollDecodeStatus::Fini(d) => {
                $crate::decode_chain!($lhs, $rhs, ($chain)(d))
            }
            $crate::backend::PollDecodeStatus::Pending => {
                $crate::backend::PollDecodeStatus::Pending
            }
            $crate::backend::PollDecodeStatus::Error(e) => {
                $lhs = $rhs::Fini;
                $crate::backend::PollDecodeStatus::Error(e)
            }
        }
    };
}

/// Finalizes [decoding](crate::backend::Decode) by polling the `$dec` expression, converts
/// the final data by calling the `$fini` closure, and finally converts
/// the result to a [PollDecodeStatus](crate::backend::PollDecodeStatus)
#[macro_export]
macro_rules! decode_poll_fini {
    ($lhs: expr, $rhs: ident, $dec: expr, $fin: expr) => {
        match $dec {
            $crate::backend::PollDecodeStatus::Fini(d) => {
                $lhs = $rhs::Fini;
                #[allow(clippy::redundant_closure_call)]
                $crate::backend::PollDecodeStatus::Fini(($fin)(d))
            }
            $crate::backend::PollDecodeStatus::Pending => {
                $crate::backend::PollDecodeStatus::Pending
            }
            $crate::backend::PollDecodeStatus::Error(e) => {
                $lhs = $rhs::Fini;
                $crate::backend::PollDecodeStatus::Error(e)
            }
        }
    };
}
