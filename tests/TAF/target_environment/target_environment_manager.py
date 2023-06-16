from .target_environment import TargetEnvironment
from ..topologies.topology import Topology
from .anvil_env_manager import AnvilEnvironmentManager


class TargetEnvironmentManager(object):
    """
    Used to manage the deployment of a Network Topology to a certain Target Environment.
    """

    def __init__(self) -> None:
        pass

    def deploy_topology(self, topology: Topology, environment: TargetEnvironment) -> None:
        """
        Deploy the given topology in a specific given environment.
        """
        match environment:
            case TargetEnvironment.ANVIL:
                anvil_manager = AnvilEnvironmentManager()
                anvil_manager.setup_local_nodes(topology.number_of_nodes)
                anvil_manager.op
            case TargetEnvironment.DOCKER_COMPOSE:
                pass
            case TargetEnvironment.KUBERNETES:
                pass
