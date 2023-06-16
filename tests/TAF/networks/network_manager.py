from .network import Network
import subprocess


class NetworkManager(object):
    """
    Managing the network where the tests will be run.
    """

    mydir = ""

    def __init__(self) -> None:
        pass

    def enable_on_network(self, network: Network) -> None:
        """
        Enable xxx on a specific network.
        Enable Network Registry.
        Sync nodes in network registry
        """
        self.__enable_network_registry(network)
        self.__register_nodes(network)
        self.__sync_nodes_in_network_registry(network)

    def __enable_network_registry(self, network: Network) -> None:
        """ """
        print("Enabling network registry...")
        command = []
        match network:
            case Network.ANVIL:
                command = [
                    "make",
                    "-C",
                    f"{self.mydir}/..",
                    "enable-network-registry",
                    "network=anvil-localhost",
                    "environment_type=local",
                ]
            case Network.ROTSEE:
                pass
            case Network.MONTE_ROSA:
                pass
        subprocess.run(command)
        print("Enabling network registry finished")

    def __register_nodes(self, network: Network) -> None:
        """
        Register nodes in the NR, emit "Registered" events.
        """
        print("Registering nodes...")
        command = []
        match network:
            case Network.ANVIL:
                command = [
                    "make",
                    "-C",
                    f'{self.mydir}/.."',
                    "register-nodes",
                    "network=anvil-localhost",
                    "environment_type=local",
                    'native_addresses="[${1}]"',
                    'peer_ids="[${2}]"',
                ]
            case Network.ROTSEE:
                pass
            case Network.MONTE_ROSA:
                pass
        subprocess.run(command)
        print("Registering nodes finished")

    def __sync_nodes_in_network_registry(self, network: Network) -> None:
        """
        Sync nodes in the NR, emit "EligibilityUpdated" events.
        """
        print("Sync nodes in network registry...")
        command = []
        match network:
            case Network.ANVIL:
                command = [
                    "make",
                    "-C",
                    f'{self.mydir}/.." sync-eligibility',
                    "network=anvil-localhost",
                    "environment",
                    "_type=local",
                    'peer_ids="[${1}]"',
                ]
            case Network.ROTSEE:
                pass
            case Network.MONTE_ROSA:
                pass
        subprocess.run(command)
        print("Sync nodes in network registry finished")
