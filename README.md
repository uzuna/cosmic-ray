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
attack 20 times and restore ref mut                                                                            
                        time:   [366.45 ns 366.83 ns 367.26 ns]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  1 (1.00%) high mild
  5 (5.00%) high severe

attack 20 times and restore move vec                                                                            
                        time:   [413.34 ns 414.97 ns 417.34 ns]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
```
