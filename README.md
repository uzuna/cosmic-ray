# Cosmic Ray

ソフトエラーによって永続化データが破壊(1bit反転)するのを再現する

## Usage

### binary

```sh
# attack
cosmic-ray testdata/dummy.json attack
[2023-01-11T08:38:16Z INFO  cosmic_ray] backup original file "testdata/dummy.orig"
[2023-01-11T08:38:16Z INFO  cosmic_ray] write 14 is 0b100001 from 0b100000

# restore
cosmic-ray testdata/dummy.json restore
[2023-01-11T08:38:21Z INFO  cosmic_ray] restore file "testdata/dummy.json"
```

### library

```rust
use cosmic_ray::{RayBoxVec, Ray};

let buf = vec![0_u8; 12];
let reference = buf.clone();

let mut raybox = RayBoxVec::new(buf);
raybox.attack(Ray::new(0)).unwrap();
assert_ne!(&*raybox, &reference);
raybox.restore();
assert_eq!(&*raybox, &reference);
```


## Performance

```txt
attack 20 times and restore                                                                            
                        time:   [348.56 ns 349.19 ns 349.90 ns]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  6 (6.00%) high mild
  4 (4.00%) high severe

attack 20 times and restore use Vec                                                                             
                        time:   [647.38 ns 648.14 ns 648.90 ns]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  4 (4.00%) high severe
```
