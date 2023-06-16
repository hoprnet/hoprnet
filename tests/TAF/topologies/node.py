class Node(object):
    """
    Node model.
    """

    def __init__(self, p2p_port: int, api_port: int, private_key: str, peer_id: str) -> None:
        self.p2p_port = p2p_port
        self.api_port = api_port
        self.private_key = private_key
        self.peer_id = peer_id
