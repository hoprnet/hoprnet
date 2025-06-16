from typing import Any


def convert_unit(value: Any):
    """
    Convert input value to appropriate type with preference for Balance.

    Conversion priority:
    1. None -> None
    2. Numeric string -> float -> int (if whole number)
    3. Fallback -> original value
    Args:
        value: Input value of any type

    Returns:
        Converted value or original value if conversion fails
    """
    if value is None:
        return None

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
