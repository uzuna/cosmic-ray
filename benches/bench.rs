use criterion::{criterion_group, criterion_main, Criterion};

use cosmic_ray::{Ray, RayBox, RayBoxVec};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("attack 20 times and restore", |b| {
        let mut buf = vec![0_u8; 1024];
        b.iter(|| {
            let mut rb = RayBox::default();
            for i in 0..20 {
                rb.attack(&mut buf, Ray::new(i)).unwrap();
            }
            rb.restore_all(&mut buf).unwrap();
        })
    });

    c.bench_function("attack 20 times and restore use Vec", |b| {
        b.iter(|| {
            let mut rb = RayBoxVec::new(vec![0_u8; 1024]);
            for i in 0..20 {
                rb.attack(Ray::new(i)).unwrap();
            }
            rb.restore_all();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
