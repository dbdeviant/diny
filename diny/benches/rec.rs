#![feature(generic_associated_types)]

#[macro_use]
extern crate criterion;
extern crate futures;

mod common;

use criterion::Criterion;
use common::*;


#[derive(diny::AsyncSerialization, Default)] pub struct Origin();

#[derive(diny::AsyncSerialization, Default)] pub struct Wide1 { f0: Large, }
#[derive(diny::AsyncSerialization, Default)] pub struct Wide2 { f0: Large, f1: Large, }
#[derive(diny::AsyncSerialization, Default)] pub struct Wide3 { f0: Large, f1: Large, f2: Large, }
#[derive(diny::AsyncSerialization, Default)] pub struct Wide4 { f0: Large, f1: Large, f2: Large, f3: Large, }
#[derive(diny::AsyncSerialization, Default)] pub struct Wide5 { f0: Large, f1: Large, f2: Large, f3: Large, f4: Large, }
#[derive(diny::AsyncSerialization, Default)] pub struct Wide6 { f0: Large, f1: Large, f2: Large, f3: Large, f4: Large, f5: Large, }
#[derive(diny::AsyncSerialization, Default)] pub struct Wide7 { f0: Large, f1: Large, f2: Large, f3: Large, f4: Large, f5: Large, f6: Large, }
#[derive(diny::AsyncSerialization, Default)] pub struct Wide8 { f0: Large, f1: Large, f2: Large, f3: Large, f4: Large, f5: Large, f6: Large, f7: Large, }
#[derive(diny::AsyncSerialization, Default)] pub struct Wide9 { f0: Large, f1: Large, f2: Large, f3: Large, f4: Large, f5: Large, f6: Large, f7: Large, f8: Large, }
#[derive(diny::AsyncSerialization, Default)] pub struct Wide10{ f0: Large, f1: Large, f2: Large, f3: Large, f4: Large, f5: Large, f6: Large, f7: Large, f8: Large, f9: Large, }

#[derive(diny::AsyncSerialization, Default)] pub struct Deep1 { f: Large   }
#[derive(diny::AsyncSerialization, Default)] pub struct Deep2 { f: Deep1 }
#[derive(diny::AsyncSerialization, Default)] pub struct Deep3 { f: Deep2 }
#[derive(diny::AsyncSerialization, Default)] pub struct Deep4 { f: Deep3 }
#[derive(diny::AsyncSerialization, Default)] pub struct Deep5 { f: Deep4 }
#[derive(diny::AsyncSerialization, Default)] pub struct Deep6 { f: Deep5 }
#[derive(diny::AsyncSerialization, Default)] pub struct Deep7 { f: Deep6 }
#[derive(diny::AsyncSerialization, Default)] pub struct Deep8 { f: Deep7 }
#[derive(diny::AsyncSerialization, Default)] pub struct Deep9 { f: Deep8 }
#[derive(diny::AsyncSerialization, Default)] pub struct Deep10{ f: Deep9 }

#[derive(diny::AsyncSerialization, Default)]
pub struct Broad10 {
    f0: Deep10,
    f1: Deep10,
    f2: Deep10,
    f3: Deep10,
    f4: Deep10,
    f5: Deep10,
    f6: Deep10,
    f7: Deep10,
    f8: Deep10,
    f9: Deep10,
}


fn width(c: &mut Criterion) {
    let width_0  = Origin::default();
    let width_1  = Wide1 ::default();
    let width_2  = Wide2 ::default();
    let width_3  = Wide3 ::default();
    let width_4  = Wide4 ::default();
    let width_5  = Wide5 ::default();
    let width_6  = Wide6 ::default();
    let width_7  = Wide7 ::default();
    let width_8  = Wide8 ::default();
    let width_9  = Wide9 ::default();
    let width_10 = Wide10::default();

    let name = "rec_ser_width";
    let mut group = c.benchmark_group(name);
    group.bench_with_input(criterion::BenchmarkId::new(name, "00"), &width_0 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "01"), &width_1 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "02"), &width_2 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "03"), &width_3 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "04"), &width_4 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "05"), &width_5 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "06"), &width_6 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "07"), &width_7 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "08"), &width_8 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "09"), &width_9 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "10"), &width_10, ser_bench);
    group.finish();

    let name = "rec_de_width";
    let mut group = c.benchmark_group(name);
    group.bench_with_input(criterion::BenchmarkId::new(name, "00"), &width_0 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "01"), &width_1 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "02"), &width_2 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "03"), &width_3 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "04"), &width_4 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "05"), &width_5 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "06"), &width_6 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "07"), &width_7 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "08"), &width_8 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "09"), &width_9 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "10"), &width_10, de_bench);
    group.finish();
}


fn depth(c: &mut Criterion) {
    let depth_0  = Origin::default();
    let depth_1  = Deep1 ::default();
    let depth_2  = Deep2 ::default();
    let depth_3  = Deep3 ::default();
    let depth_4  = Deep4 ::default();
    let depth_5  = Deep5 ::default();
    let depth_6  = Deep6 ::default();
    let depth_7  = Deep7 ::default();
    let depth_8  = Deep8 ::default();
    let depth_9  = Deep9 ::default();
    let depth_10 = Deep10::default();

    let name = "rec_ser_depth";
    let mut group = c.benchmark_group(name);
    group.bench_with_input(criterion::BenchmarkId::new(name, "00"), &depth_0 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "01"), &depth_1 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "02"), &depth_2 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "03"), &depth_3 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "04"), &depth_4 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "05"), &depth_5 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "06"), &depth_6 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "07"), &depth_7 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "08"), &depth_8 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "09"), &depth_9 , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "10"), &depth_10, ser_bench);
    group.finish();

    let name = "rec_de_depth";
    let mut group = c.benchmark_group(name);
    group.bench_with_input(criterion::BenchmarkId::new(name, "00"), &depth_0 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "01"), &depth_1 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "02"), &depth_2 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "03"), &depth_3 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "04"), &depth_4 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "05"), &depth_5 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "06"), &depth_6 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "07"), &depth_7 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "08"), &depth_8 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "09"), &depth_9 , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "10"), &depth_10, de_bench);
    group.finish();
}


fn breadth(c: &mut Criterion) {
    let origin  = Origin ::default();
    let breadth = Broad10::default();

    let name = "rec_ser_breadth";
    let mut group = c.benchmark_group(name);
    group.bench_with_input(criterion::BenchmarkId::new(name, "00"), &origin , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "10"), &breadth, ser_bench);
    group.finish();

    let name = "rec_de_breadth";
    let mut group = c.benchmark_group(name);
    group.bench_with_input(criterion::BenchmarkId::new(name, "00"), &origin , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "10"), &breadth, de_bench);
    group.finish();
}

criterion_group!(rec, width, depth, breadth);
criterion_main!(rec);