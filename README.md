# Cosmic Ray

ソフトエラーによって永続化データが破壊(1bit反転)するのを再現する

## Usage

- 任意のファイルの特定のビットを反転させる(アドレス指定、ランダム)

```sh
# attack
cosmic-ray testdata/dummy.json attack
[2023-01-11T08:38:16Z INFO  cosmic_ray] backup original file "testdata/dummy.orig"
[2023-01-11T08:38:16Z INFO  cosmic_ray] write 14 is 0b100001 from 0b100000

# restore
cosmic-ray testdata/dummy.json restore
[2023-01-11T08:38:21Z INFO  cosmic_ray] restore file "testdata/dummy.json"
```
