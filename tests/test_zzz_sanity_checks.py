import subprocess

import pytest

from sdk.python.localcluster.constants import SUITE_NAME


@pytest.mark.xfail(reason="sometimes the log output after tests can contain errors")
def test_logs_should_not_contain_any_errors():
    cmd = r"find /tmp/ -type f -iregex '.*FOLDER.*.log' -exec grep ERROR {} \;"
    cmd = cmd.replace("FOLDER", SUITE_NAME)

    command_array = cmd.split()
    result = subprocess.run(command_array, stdout=subprocess.PIPE)
    error_lines = result.stdout.decode().split("\n")

    assert error_lines == []
