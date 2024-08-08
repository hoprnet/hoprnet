import pytest
import subprocess


@pytest.mark.xfail(reason="sometimes the log output after tests can contain errors")
def test_logs_should_not_contain_any_errors():
    cmd = r"find /tmp/ -type f -iregex '.*hopr-smoke-test.*.log' -exec grep ERROR {} \;"
    command_array = cmd.split()
    result = subprocess.run(command_array, stdout=subprocess.PIPE)
    error_lines = result.stdout.decode().split("\n")

    assert error_lines == []
