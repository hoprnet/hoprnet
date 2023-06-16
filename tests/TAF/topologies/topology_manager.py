from .topology import Topology


class TopologyManager(object):
    """
    Generate a Network Topology with given parameters.
    """

    def __init__(self, topology: Topology) -> None:
        self.topology = topology

    def generate_topology(self) -> None:
        """
        Generate YAML topology file
        """
        pass
