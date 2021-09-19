#![feature(generic_associated_types)]

#[macro_use]
extern crate criterion;
extern crate futures;

mod common;

use criterion::Criterion;
use common::*;


fn width(c: &mut Criterion) {
    let width_0  = [Large::default();  0];
    let width_1  = [Large::default();  1];
    let width_2  = [Large::default();  2];
    let width_3  = [Large::default();  3];
    let width_4  = [Large::default();  4];
    let width_5  = [Large::default();  5];
    let width_6  = [Large::default();  6];
    let width_7  = [Large::default();  7];
    let width_8  = [Large::default();  8];
    let width_9  = [Large::default();  9];
    let width_10 = [Large::default(); 10];

    let name = "array_ser_width";
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

    let name = "array_de_width";
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
    let depth_0  = [Large::default(); 0];
    let depth_1  = [Large::default()];
    let depth_2  = [[Large::default()]];
    let depth_3  = [[Large::default()]];
    let depth_4  = [[[Large::default()]]];
    let depth_5  = [[[[Large::default()]]]];
    let depth_6  = [[[[[Large::default()]]]]];
    let depth_7  = [[[[[[Large::default()]]]]]];
    let depth_8  = [[[[[[[Large::default()]]]]]]];
    let depth_9  = [[[[[[[[Large::default()]]]]]]]];
    let depth_10 = [[[[[[[[[Large::default()]]]]]]]]];

    let name = "array_ser_depth";
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

    let name = "array_de_depth";
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
    let origin  = [Large::default(); 0];
    let breadth = [[[[[[[[[[Large::default()]]]]]]]]]; 10];

    let name = "array_ser_breadth";
    let mut group = c.benchmark_group(name);
    group.bench_with_input(criterion::BenchmarkId::new(name, "00"), &origin , ser_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "10"), &breadth, ser_bench);
    group.finish();

    let name = "array_de_breadth";
    let mut group = c.benchmark_group(name);
    group.bench_with_input(criterion::BenchmarkId::new(name, "00"), &origin , de_bench);
    group.bench_with_input(criterion::BenchmarkId::new(name, "10"), &breadth, de_bench);
    group.finish();
}

criterion_group!(array, width, depth, breadth);
criterion_main!(array);