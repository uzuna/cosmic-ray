import pytest

from cosmic_ray import Number, ByteBox

def test_number():
    n = Number(21)
    n.add(45)
    assert n.get() == 66

def test_bytebox():
    b = ByteBox(b'\x7f\x45\x4c\x46\x01\x01\x01\x00')
    print(b.elements)
    print(b.raw())
    print(b.reference())

