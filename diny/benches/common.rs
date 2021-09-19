#![allow(unused_attributes)] // It really is used...
#![feature(generic_associated_types)]

use criterion::{Bencher, measurement::WallTime};

pub fn ser_bench<T>(b: &mut Bencher<WallTime>, t: &T)
where
    T: diny::AsyncSerialization,
{
    let mut buf = [0u8; 16 * 1024];
    let format = diny_test::format();

    b.iter(|| {
        let mut writer = diny::util::AsyncSliceWriter::from(&mut buf[..]);
        let write = t.serialize(&format, &mut writer);
        let _ = futures::executor::block_on(write);
    });
}

pub fn de_bench<T>(b: &mut Bencher<WallTime>, t: &T)
where
    T: diny::AsyncSerialization,
{
    let mut buf = [0u8; 128];
    let format = diny_test::format();
    let mut writer = diny::util::AsyncSliceWriter::from(&mut buf[..]);
    let write = t.serialize(&format, &mut writer);
    let _ = futures::executor::block_on(write);

    b.iter(|| {
        let mut reader = diny::util::AsyncSliceReader::from(&buf[..]);
        let read = <T as diny::AsyncDeserialize>::deserialize(&format, &mut reader);
        let _ = futures::executor::block_on(read);
    });
}

#[derive(Copy, Clone, diny::AsyncSerialization)]
pub struct Large{ f0: u64, f1: u64, f2: u64, f3: u64, f4: u64, f5: u64, f6: u64, f7: u64, f8: u64, f9: u64, }

impl Default for Large {
    fn default() -> Self {
        Self {
            f0: 1 << (6   ),
            f1: 1 << (6* 2),
            f2: 1 << (6* 3),
            f3: 1 << (6* 4),
            f4: 1 << (6* 5),
            f5: 1 << (6* 6),
            f6: 1 << (6* 7),
            f7: 1 << (6* 8),
            f8: 1 << (6* 9),
            f9: 1 << (6*10),
        }
    }
}
