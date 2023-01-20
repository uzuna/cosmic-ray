import pytest

from cosmic_ray import RayBox, Ray


def test_raybox():
    # const bytes
    text = b"test_text"
    # on heap bytearray
    text2 = bytearray(text)
    rb = RayBox()
    ray = Ray(0)
    rb.attack(text2, ray)
    assert text2 == b"uest_text"
    rb.attack_rand(text2)
    assert text2 != b"uest_text"
    rb.restore(text2)
    assert text2 == text
