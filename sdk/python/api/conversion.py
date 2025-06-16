from typing import Any

from .balance import Balance


def convert_unit(value: Any):
    """
    Convert input value to appropriate type with preference for Balance.

    Conversion priority:
    1. None -> None
    2. Balance -> unchanged Balance
    3. String with units -> Balance
    4. Numeric string -> float -> int (if whole number)
    5. Fallback -> original value
    Args:
        value: Input value of any type

    Returns:
        Converted value or original value if conversion fails
    """
    if value is None:
        return None

    if isinstance(value, Balance):
        return value

    try:
        value = Balance(value)
    except TypeError:
        pass
    else:
        return value

    try:
        value = float(value)
    except ValueError:
        pass
    except TypeError:
        pass

    try:
        integer = int(value)
        if integer == value:
            value = integer

    except ValueError:
        pass
    except TypeError:
        pass

    return value
