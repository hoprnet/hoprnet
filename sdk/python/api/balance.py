from decimal import Decimal, InvalidOperation
from typing import Optional

WEI_TO_READABLE = Decimal("1e18")


class Balance:
    def __init__(self, value: str):
        if not isinstance(value, str):
            raise TypeError(f"Balance value must be a string, got {type(value).__name__} ({value})")

        self._value: str = value

        if self.unit and self.unit.split()[0] == "wei":
            converted_value = Decimal(self.value) / WEI_TO_READABLE
            self._value = f"{converted_value} {self.unit.split(maxsplit=1)[1]}"

        self.balance_format_check()

    def balance_format_check(self):
        if len(self._value.split()) > 3 or self.unit is None:
            raise TypeError(f"Invalid balance format: {self._value}")

        try:
            _ = self.value
        except TypeError:
            raise TypeError(f"Invalid balance value: {self._value}")

        return True

    @property
    def as_str(self) -> str:
        return self._value

    @property
    def value(self) -> Decimal:
        try:
            return Decimal(self._value.split()[0])
        except InvalidOperation:
            raise TypeError(f"Invalid balance value: {self._value}")

    @property
    def unit(self) -> Optional[str]:
        return self._value.split(maxsplit=1)[1] if " " in self._value else None

    @classmethod
    def zero(cls, unit: str):
        return cls(f"0 {unit}")

    @classmethod
    def from_float(cls, value: Optional[float], unit: str):
        if value is None:
            value = 0.0

        return cls(f"{value} {unit}")

    def __eq__(self, other):
        if not isinstance(other, Balance):
            return False

        return self.value == other.value and self.unit == other.unit

    def __lt__(self, other):
        if not isinstance(other, Balance):
            raise TypeError("Comparison must be with another Balance object")

        if self.unit != other.unit:
            raise ValueError(f"Cannot compare balances with different units: {self.unit} and {other.unit}")

        return self.value < other.value

    def __le__(self, other):
        if not isinstance(other, Balance):
            raise TypeError("Comparison must be with another Balance object")

        if self.unit != other.unit:
            raise ValueError(f"Cannot compare balances with different units: {self.unit} and {other.unit}")

        return self.value <= other.value

    def __add__(self, other):
        if self.unit != other.unit:
            raise TypeError(f"Cannot add balances with different units: {self.unit} and {other.unit}")

        return Balance(f"{self.value + other.value} {self.unit}")

    def __sub__(self, other):
        if self.unit != other.unit:
            raise TypeError(f"Cannot subtract balances with different units: {self.unit} and {other.unit}")

        return Balance(f"{self.value - other.value} {self.unit}")

    def __truediv__(self, other):
        if isinstance(other, Balance):
            if self.unit != other.unit:
                raise TypeError(f"Cannot divide balances with different units: {self.unit} and {other.unit}")
            return self.value / other.value
        else:
            return Balance(f"{self.value / Decimal(other)} {self.unit}")

    def __rtruediv__(self, other):
        if isinstance(other, Balance):
            raise TypeError("Cannot divide two Balance objects directly")
        else:
            return Balance(f"{Decimal(other) / self.value} {self.unit}")

    def __mul__(self, other):
        if isinstance(other, Balance):
            raise TypeError("Cannot multiply two Balance objects directly")
        else:
            return Balance(f"{self.value * Decimal(other)} {self.unit}")

    def __rmul__(self, other):
        if isinstance(other, Balance):
            raise TypeError("Cannot multiply two Balance objects directly")
        else:
            return Balance(f"{self.value * Decimal(other)} {self.unit}")

    def __pow__(self, power):
        if not isinstance(power, (int, float, Decimal)):
            raise TypeError("Power must be an integer or float")
        return Balance(f"{self.value ** Decimal(power)} {self.unit}")

    def __round__(self, ndigits: int = 0):
        if not isinstance(ndigits, int):
            raise TypeError("ndigits must be an integer")
        return Balance(f"{round(self.value, ndigits)} {self.unit}")

    def __repr__(self):
        key_pair_string: str = ", ".join([f"{key}={value}" for key, value in vars(self).items()])
        return f"{self.__class__.__name__}({key_pair_string})"
