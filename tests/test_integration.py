import json
import os
import subprocess

from conftest import DEFAULT_API_TOKEN


def test_hoprd_protocol_integration_tests(setup_7_nodes):
    with open("/tmp/hopr-smoke-test-anvil.cfg") as f:
        data = json.load(f)

    anvil_private_key = data["private_keys"][0]

    env_vars = os.environ.copy()
    env_vars.update(
        {
            "HOPRD_API_TOKEN": f"{DEFAULT_API_TOKEN}",
            "PRIVATE_KEY": f"{anvil_private_key}",
        }
    )

    nodes_api_as_str = " ".join(list(map(lambda x: f"\"localhost:{x['api_port']}\"", setup_7_nodes.values())))

    log_file_path = f"/tmp/hopr-smoke-{__name__}.log"
    subprocess.run(
        f"./tests/integration-test.sh {nodes_api_as_str} 2>&1 | tee {log_file_path}",
        shell=True,
        capture_output=True,
        env=env_vars,
        # timeout=2000,
        check=True,
    )
