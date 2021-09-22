macro_rules! encode_chain {
    ($lhs: expr, $enc: expr) => {
        match $enc {
            backend::StartEncodeStatus::Fini => {
                $lhs = Self::Fini;
                backend::PollEncodeStatus::Fini
            }
            backend::StartEncodeStatus::Pending(enc) => {
                $lhs = enc;
                backend::PollEncodeStatus::Pending
            }
            backend::StartEncodeStatus::Error(e) => {
                $lhs = Self::Fini;
                backend::PollEncodeStatus::Error(e)
            }
        }
    };
}

macro_rules! encode_poll_chain {
    ($lhs: expr, $enc: expr, $chain: expr) => {
        match $enc {
            backend::PollEncodeStatus::Fini => encode_chain!($lhs, $chain),
            backend::PollEncodeStatus::Pending => {
                backend::PollEncodeStatus::Pending
            }
            backend::PollEncodeStatus::Error(e) => {
                $lhs = Self::Fini;
                backend::PollEncodeStatus::Error(e)
            }
        }
    };
}

macro_rules! encode_poll_fini {
    ($lhs: expr, $enc: expr) => {
        match $enc {
            backend::PollEncodeStatus::Fini => {
                $lhs = Self::Fini;
                backend::PollEncodeStatus::Fini
            }
            backend::PollEncodeStatus::Pending => {
                backend::PollEncodeStatus::Pending
            }
            backend::PollEncodeStatus::Error(e) => {
                $lhs = Self::Fini;
                backend::PollEncodeStatus::Error(e)
            }
        }
    };
}


macro_rules! decode_chain {
    ($lhs: expr, $rhs: ident, $dec: expr) => {
        match $dec {
            backend::StartDecodeStatus::Fini(d) => {
                $lhs = $rhs::Fini;
                backend::PollDecodeStatus::Fini(d)
            }
            backend::StartDecodeStatus::Pending(enc) => {
                $lhs = enc;
                backend::PollDecodeStatus::Pending
            }
            backend::StartDecodeStatus::Error(e) => {
                $lhs = $rhs::Fini;
                backend::PollDecodeStatus::Error(e)
            }
        }
    };
}

macro_rules! decode_poll_chain {
    ($lhs: expr, $rhs: ident, $enc: expr, $cont: expr) => {
        match $enc {
            #[allow(clippy::redundant_closure_call)]
            backend::PollDecodeStatus::Fini(d) => {
                decode_chain!($lhs, $rhs, ($cont)(d))
            }
            backend::PollDecodeStatus::Pending => {
                backend::PollDecodeStatus::Pending
            }
            backend::PollDecodeStatus::Error(e) => {
                $lhs = $rhs::Fini;
                backend::PollDecodeStatus::Error(e)
            }
        }
    };
}

macro_rules! decode_poll_fini {
    ($lhs: expr, $rhs: ident, $dec: expr, $fin: expr) => {
        match $dec {
            backend::PollDecodeStatus::Fini(d) => {
                $lhs = $rhs::Fini;
                #[allow(clippy::redundant_closure_call)]
                backend::PollDecodeStatus::Fini(($fin)(d))
            }
            backend::PollDecodeStatus::Pending => {
                backend::PollDecodeStatus::Pending
            }
            backend::PollDecodeStatus::Error(e) => {
                $lhs = $rhs::Fini;
                backend::PollDecodeStatus::Error(e)
            }
        }
    };
}
