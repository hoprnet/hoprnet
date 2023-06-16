import json
from requests import Response
from websocket import WebSocketTimeoutException

from .root_api_model import RootApiModel
from ..test_data.urls import Urls
from ..services.rest_api_service import RestApiService
from ..services.ws_api_service import WebsocketClientService


class Messages(RootApiModel):
    """
    Messages object wrapper with all useful methods to interact with a certain node addresses,
    balances, and withdrawals.
    """

    def __init__(self) -> None:
        super().__init__()

    def send_message(self, nodeIndex: int, message: str, recipient: str, path, hops: int) -> int:
        """
        Send a message to another peer using a given path (list of node addresses that should relay our
        message through network).
        If no path is given, HOPR will attempt to find a path.
        :param nodeIndex The index of the node
        :return: The status code of the send message operation
        """
        url = self.get_rest_url(nodeIndex, Urls.MESSAGES_SEND)
        body = {"body": message, "recipient": recipient}
        if path is not None:
            body["path"] = path
        if hops is not None:
            body["hops"] = hops
        print("url={}".format(url))
        print("body={}".format(json.dumps(body)))

        restService = RestApiService(self.get_auth_token())
        response: Response = restService.post_request(url, body)
        print("Response: {} status: {}".format(response.json(), response.status_code))
        if response.status_code != 200 or response.status_code != 202:
            self.handle_http_error(response)
        return response.status_code

    def check_node_does_not_get_message(
        self, nodeIndex: int, webSocket: WebsocketClientService, timeout: int = 5
    ) -> None:
        """
        Checking that a certain node does not get the message in a specified timeout.
        """
        try:
            webSocket.receive_message()
            assert "Node {} received the message when it should not have".format(nodeIndex)
        except WebSocketTimeoutException:
            # Timeout hit, as expected, just close the WS client connection
            webSocket.close_client_connection()
