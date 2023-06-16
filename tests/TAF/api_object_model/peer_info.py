from .root_api_model import RootApiModel

from ..test_data.urls import Urls


class PeerInfo(RootApiModel):
    """
    PeerInfo object wrapper with all useful methods to get information about a certain peer/node.
    """

    def __init__(self) -> None:
        super().__init__()

    def get_peer_info(self, peerId) -> str:
        """
        Get information about this peer.
        :param peerId 16Uiu2HAmVfV4GKQhdECMqYmUMGLy84RjTJQxTWDcmUX5847roBar
        """
        "http://{baseHostname}{peerInfo}".format(baseHostname=self.get_base_hostname(), peerInfo=Urls.PEER_INFO)
        pass

    def other_utility_method_around_these_APIs(self) -> None:
        """ """
        pass
