import pytest


@pytest.fixture
def error_fixture():
    assert 0


def test_ok():
    print("ok")
