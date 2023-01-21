# Type hint for Python
# https://pyo3.rs/v0.18.0/python_typing_hints

class Ray:
    """
    Information about which bit at which position to invert
    """

    def __init__(self, offset: int) -> None:
        """
        new Ray
        :param offset: affect address
        """
        pass
    @staticmethod
    def with_pattern(offset: int, pattern: int) -> Ray:
        """
        new with pattern

        :param offset: affect address
        :param pattern: bit pattern includes: [0,1,2,4,8,16,32,64,128]
        """
        pass

class RayBox:
    """
    バッファの操作記録を保持する構造体
    """

    def __init__(self) -> None:
        """new RayBox"""
        pass
    def attack(self, buf: bytearray, ray: Ray) -> None:
        """
        attack buffer by ray

        :param buf: target buffer
        :param ray: attack offset and pattern
        """
        pass
    def attack_offset(self, buf: bytearray, offset: int) -> None:
        """
        attack buffer by ray

        :param buf: target buffer
        :param offset: attack offset
        """
        pass
    def attack_random(self, buf: bytearray) -> None:
        """
        attack buffer by random ray

        :param buf: target buffer
        """
        pass
    def attack_randam_with_pattern(self, buf: bytearray) -> None:
        """
        attack buffer by random offset and pattern ray

        :param buf: target buffer
        """
        pass
    def restore(self, buf: bytearray) -> None:
        """
        restore buffer

        :param buf: target buffer
        """
        pass
