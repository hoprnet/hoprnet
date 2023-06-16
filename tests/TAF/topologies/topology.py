from typing import List


class Topology:
    """
    Network Topology Configuration model used in the generation of a network topology by NetworkTopology class.
    """

    def __init__(self, number_of_nodes: int, channels: List[int]) -> None:
        """
        :number_of_nodes: The number of nodes to generate for the topology.
        :channels: The list of channels to open [[nodeIndex1, nodeIndex2], [nodeIndex3, nodeIndex4]]
        """
        self.number_of_nodes = number_of_nodes
        self.channels = channels
