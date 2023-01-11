# Cosmic Ray

ソフトエラーによって永続化データが破壊(1bit反転)するのを再現する

## Usage

- 任意のファイルの特定のビットを反転させる(アドレス指定、ランダム、)

```sh
# pos
cosr --pos <integer> --pattern <u8> <filepath>
```
