from .root_api_model import RootApiModel


class Aliases(RootApiModel):
    """
    Aliases object wrapper with all useful methods to interact with a certain node addresses, balances, and withdrawals.
    """

    def __init__(self) -> None:
        super().__init__()

    def get_all_aliases(self) -> None:
        """
        Get all aliases you set previously and thier corresponding peer IDs.
        :return Returns List of Aliases and corresponding peerIds.
        """
        pass

    def assign_alias(self, alias) -> bool:
        """
        Instead of using HOPR address, we can assign HOPR address to a specific name called alias.
        Give an address a more memorable alias and use it instead of Hopr address.
        Aliases are kept locally and are not saved or shared on the network.
        :param alias Alias that we previously assigned to some PeerId.
        """
        pass

    def get_peer_id(self, alias) -> str:
        """
        Get the PeerId (Hopr address) that have this alias assigned to it.
        :param alias Alias that we previously assigned to some PeerId.
        :return HOPR address for the provided alias.
        """
        pass

    def delete_alias(self, alias) -> bool:
        """
        Unassign an alias from a PeerId. You can always assign back alias to that PeerId using /aliases endpoint.
        :param alias Alias that we want to remove.
        :return True if the alias was properly removed, False otherwise.
        """
        pass

    def other_utility_method_around_these_APIs(self) -> None:
        """ """
        pass
