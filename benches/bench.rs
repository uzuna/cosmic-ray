use criterion::{criterion_group, criterion_main, Criterion};

use cosmic_ray::{Ray, RayBox, RayBoxVec};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("attack 20 times and restore ref mut", |b| {
        let mut buf = vec![0_u8; 1024];
        b.iter(|| {
            let mut rb = RayBox::default();
            for i in 0..20 {
                rb.attack(&mut buf, Ray::new(i)).unwrap();
            }
            rb.restore_all(&mut buf).unwrap();
        })
    });

    c.bench_function("attack 20 times and restore move vec", |b| {
        let mut store = Some(vec![0_u8; 1024]);
        b.iter(|| {
            let mut v = None;
            std::mem::swap(&mut store, &mut v);
            let mut rb = RayBoxVec::new(v.unwrap());
            for i in 0..20 {
                rb.attack(Ray::new(i)).unwrap();
            }
            rb.restore_all();
            let mut x = Some(RayBoxVec::into_inner(rb));
            std::mem::swap(&mut store, &mut x);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
