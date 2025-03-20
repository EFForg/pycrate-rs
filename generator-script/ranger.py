import unittest


class Single:
    def __init__(self, n: int) -> None:
        self.n = n

    def contains(self, n: int) -> bool:
        return self.n == n

    def to_rust(self) -> str:
        return f'{self.n}'

    def remove(self, n: int) -> list['Single']:
        assert n == self.n
        return []


def range_or_single(low: int, high: int) -> 'Range | Single':
    assert low <= high
    if low == high:
        return Single(low)
    return Range(low, high)


class Range:
    def __init__(self, low: int, high: int) -> None:
        assert low < high
        self.low = low
        self.high = high

    def contains(self, n: int) -> bool:
        return n >= self.low and n <= self.high

    def to_rust(self) -> str:
        return f'{self.low}..={self.high}'

    def remove(self, n: int) -> list['Range | Single']:
        if n == self.low:
            self.low = self.low + 1
        elif n == self.high:
            self.high = self.high - 1
        else:
            left = range_or_single(self.low, n - 1)
            right = range_or_single(n + 1, self.high)
            return [left, right]
        return [range_or_single(self.low, self.high)]


class Ranger:
    def __init__(self, n_bits: int) -> None:
        self.n_bits = n_bits
        self.ranges: list[Range | Single]
        if self.n_bits != 0:
            self.ranges = [Range(0, (2 ** n_bits) - 1)]
        else:
            self.ranges = []

    def to_rust(self) -> str:
        return ' | '.join([range.to_rust() for range in self.ranges])

    def remove(self, n: int) -> None:
        for i in range(len(self.ranges)):
            if self.ranges[i].contains(n):
                results = self.ranges[i].remove(n)
                self.ranges[i:i+1] = results
                return


class TestRanger(unittest.TestCase):
    def test_range(self):
        r = Ranger(3)
        assert r.to_rust() == '0..=7'
        r.remove(0)
        assert r.to_rust() == '1..=7'
        r.remove(7)
        assert r.to_rust() == '1..=6'
        r.remove(3)
        assert r.to_rust() == '1..=2 | 4..=6'
        r.remove(2)
        assert r.to_rust() == '1 | 4..=6'
        r.remove(5)
        assert r.to_rust() == '1 | 4 | 6'
        r.remove(1)
        r.remove(4)
        r.remove(6)
        assert r.ranges == []

    def test_zero_bits(self):
        r = Ranger(0)
        assert r.ranges == []


if __name__ == "__main__":
    unittest.main()
