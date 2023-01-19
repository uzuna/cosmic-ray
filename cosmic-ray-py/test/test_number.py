import pytest

from cosmic_ray import Number, ByteBox, affect

def test_number():
    n = Number(21)
    n.add(45)
    assert n.get() == 66

def test_bytebox():
    # const bytes
    text = b"test_text"
    b = ByteBox(text)
    print(b.elements)
    print(b.raw())
    print(b.reference())

    # on heap bytearray
    text2 = bytearray(text)
    affect(text2)
    print(text2)
    print(str(text2, 'utf-8'))
