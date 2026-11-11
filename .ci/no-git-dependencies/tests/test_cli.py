"""Exercise the dependency policy through the command used by CI."""

import datetime
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

CHECKER = Path(__file__).resolve().parents[1] / "src/no_git_dependencies_check/main.py"


class DependencyPolicyTests(unittest.TestCase):
    def run_check(self, *arguments, git=True):
        with tempfile.TemporaryDirectory() as directory:
            manifest = "[workspace]\nmembers = []\n"
            if git:
                manifest += '[patch.crates-io]\ncompanion = { git = "https://example.com/companion", rev = "abc" }\n'
            Path(directory, "Cargo.toml").write_text(manifest)
            return subprocess.run(
                [sys.executable, str(CHECKER), *arguments],
                cwd=directory,
                capture_output=True,
                text=True,
                check=False,
            )

    def test_git_dependencies_fail_without_an_exception(self):
        self.assertNotEqual(self.run_check().returncode, 0)

    def test_an_exception_requires_an_expiry(self):
        self.assertNotEqual(self.run_check("--allow", "companion").returncode, 0)

    def test_an_expired_exception_fails(self):
        self.assertNotEqual(self.run_check("--allow", "companion", "--allow-until", "2000-01-01").returncode, 0)

    def test_a_live_exception_only_allows_the_named_crate(self):
        expiry = (datetime.date.today() + datetime.timedelta(days=14)).isoformat()
        self.assertEqual(self.run_check("--allow", "companion", "--allow-until", expiry).returncode, 0)
        self.assertNotEqual(self.run_check("--allow", "other", "--allow-until", expiry).returncode, 0)

    def test_registry_dependencies_need_no_exception(self):
        self.assertEqual(self.run_check(git=False).returncode, 0)


if __name__ == "__main__":
    unittest.main()
