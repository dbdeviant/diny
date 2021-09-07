
macro_rules! serialize_all_def {
    ($format: ty, $data: ty, $encoder: ty) => {
        pub(crate) type SerializeAll<'w, W> = diny::buffer::BufferEncoder<'w, $format, W, $data, $encoder>;

        pub(crate) fn serialize<'w, W>(format: &'w $format, writer: &'w mut W, data: &$data) -> SerializeAll<'w, W>
        where
            W: ::futures::io::AsyncWrite + Unpin,
        {
            SerializeAll::new(format, writer, <$encoder as ::diny::backend::Encode>::init(data))
        }
    };
}

macro_rules! deserialize_exact_def {
    ($format: ty, $data: ty, $decoder: ty) => {
        pub(crate) type DeserializeExact<'r, R> = diny::backend::DeserializeExact<'r, $format, R, $data, $decoder>;

        pub(crate) fn deserialize<'r, R>(format: &'r $format, reader: &'r mut R) -> DeserializeExact<'r, R>
        where
            R: ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + Unpin,
        {
            DeserializeExact::new(format, reader, <$decoder as ::diny::backend::Decode>::init())
        }
   };
}