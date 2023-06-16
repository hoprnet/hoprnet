from typing import List
from websocket import create_connection
from websocket import WebSocketTimeoutException
from ..api_object_model.root_api_model import RootApiModel


class Message(object):
    def __init__(self) -> None:
        self.message: str
        self.timestamp: int


class WebsocketClientService2(RootApiModel):
    """
    Websocket client wrapper service with the purpose to manage one websocket client connection.
    """

    def __init__(self, nodeIndex, path, timeout=30) -> None:
        self.receivedMessages: List[Message] = []


class WebsocketClientService(RootApiModel):
    """
    Websocket client wrapper service with the purpose to manage one websocket client connection.
    """

    def __init__(self, nodeIndex, path, timeout=30) -> None:
        self.create_client_connection(nodeIndex, path, timeout)

    def create_client_connection(self, nodeIndex, path, timeout=30) -> None:
        """
        Open a new websocket client connection to the given URL
        """
        wsUrl = self.get_ws_url(nodeIndex, path)
        print("wsUrl = {}".format(wsUrl))
        self.webSocket = create_connection(wsUrl, timeout)

    def receive_message(self):
        """
        Blocking operation. Wait for a message and return it to the caller when received.
        """
        message = self.webSocket.recv()
        return message

    def close_client_connection(self):
        """
        Closing the current websocket client connection in a safe way.
        """
        try:
            self.webSocket.close()
        except WebSocketTimeoutException:
            pass
