import random
from cosmic_ray_py import RayBox, Ray


testdata = b"text_text_text_text_text"


def attack_restore(data: bytearray, reference: bytes):
    rb = RayBox()
    for i in range(20):
        rb.attack(data, Ray(i))
    rb.restore(data)
    assert reference == data


def attack_restore_offset(data: bytearray, reference: bytes):
    rb = RayBox()
    for i in range(20):
        rb.attack_offset(data, i)
    rb.restore(data)
    assert reference == data


def attack_restore_random(data: bytearray, reference: bytes):
    rb = RayBox()
    for i in range(20):
        rb.attack_random(data)
    rb.restore(data)
    assert reference == data


def attack_restore_random_python(data: bytearray, reference: bytes):
    rb = RayBox()

    for i in range(20):
        rb.attack(data, Ray(random.randrange(len(reference))))
    rb.restore(data)
    assert reference == data


def test_attack_restore(benchmark):
    """値を指定して反転"""
    benchmark(attack_restore, bytearray(testdata), testdata)


def test_attack_restore_offset(benchmark):
    """Offsetだけを指定して反転"""
    benchmark(attack_restore_offset, bytearray(testdata), testdata)


def test_attack_restore_random(benchmark):
    """Rustの中で値を生成して反転"""
    benchmark(attack_restore_random, bytearray(testdata), testdata)


def test_attack_restore_random_python(benchmark):
    """Pythonでランダムに値を生成して反転"""
    benchmark(attack_restore_random_python, bytearray(testdata), testdata)
