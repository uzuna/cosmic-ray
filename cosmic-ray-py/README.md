# Cosmic Ray Python

ソフトエラーによって永続化データが破壊(1bit反転)するのを再現する
Rust製のライブラリCosmicRayのバインディング

## Usage

```py

def test_raybox():
    # const bytes
    text = b"test_text"

    # 書き換えたいバイト列をbytearrayにする
    # Pythonのメモリ状をRustで書き換えるため
    text2 = bytearray(text)

    # 書き換え記録を保持してくれるRayBox構造体を経由で操作を行う
    rb = RayBox()

    # アドレス0番目を書き換え
    ray = Ray(0)
    rb.attack(text2, ray)
    assert text2 == b"uest_text"

    # ランダムに書き換え
    rb.attack_rand(text2)
    assert text2 != b"uest_text"

    # データを元に戻す
    rb.restore(text2)
    assert text2 == text