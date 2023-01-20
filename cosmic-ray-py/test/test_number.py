from cosmic_ray_py import RayBox, Ray


def test_raybox():
    # const bytes
    text = b"test_text"
    # on heap bytearray
    text2 = bytearray(text)
    rb = RayBox()
    ray = Ray(0)
    rb.attack(text2, ray)
    # t=01110100
    # u=01110101
    assert text2 == b"uest_text"
    rb.attack_random(text2)
    assert text2 != b"uest_text"
    rb.restore(text2)
    assert text2 == text


def test_raybox_pattern():
    # const bytes
    text = b"test_text"
    # on heap bytearray
    text2 = bytearray(text)
    rb = RayBox()

    # patternを指定して破壊
    # 4=00000100
    ray = Ray.with_pattern(0, 4)
    rb.attack(text2, ray)
    # t=01110100
    # p=01110000
    assert text2 == b"pest_text"
