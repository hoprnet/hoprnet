from ...target_environment.target_environment import TargetEnvironment
from ...target_environment.anvil_env_manager import LocalTestEnvironmentManager
import pytest
from typing import List

"""
Send Messages Test Plan Implementation.
"""


@pytest.fixture
def environment_setup():
    env = TargetEnvironment.LOCAL
    match env:
        case TargetEnvironment.LOCAL:
            local_env = LocalTestEnvironmentManager()
            local_env.setup_local_nodes(5)
        case TargetEnvironment.DOCKER_COMPOSE:
            pass
        case TargetEnvironment.KUBERNETES:
            pass


class Input:
    def __init__(
        self, environment: TargetEnvironment, sender: int, receiver: int, message: str, path: List[int], hops: int
    ) -> None:
        self.environment = environment
        self.sender = sender
        self.receiver = receiver
        self.message = message
        self.path = path
        self.hops = hops


class Output:
    def __init__(self, receivedMessage: str, visitationPath: List[List[int]], relayNode: int) -> None:
        self.receivedMessage = receivedMessage
        self.visitationPath = visitationPath
        self.relayNode = relayNode


@pytest.mark.parametrize(
    "input, output",
    [
        (
            Input(sender=1, receiver=2, message="Hello from future", path=[3], hops=None),
            Output(
                receivedMessage="217,145,72,101,108,108,111,32,102,114,111,109,32,102,117",
                visitationPath=[[1, 3], [3, 2]],
                relayNode=3,
            ),
        )
    ],
)
def test_case1(input: Input, output: Output):
    """ """
