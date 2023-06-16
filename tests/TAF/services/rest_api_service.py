import requests
from requests import Response

from ..api_object_model.root_api_model import RootApiModel

HTTP_STATUS_CODE_OK = 200
HTTP_STATUS_CODE_SEND_MESSAGE_OK = 202


class RestApiService(RootApiModel):
    """
    Wrapper utility class over Python requests library used to handle REST API calls
    """

    def __init__(self, authToken) -> None:
        self.authToken = authToken
        self.headers = {"X-Auth-Token": self.authToken, "Connection": "close"}

    def set_headers(self, headers) -> None:
        """ """
        self.headers = headers

    def get_request(self, nodeIndex: int, path: str) -> object:
        """
        Making a REST GET request and returning the 'Response' object to the caller.
        :return: Request data
        """
        url = self.get_rest_url(nodeIndex, path)
        response: Response = requests.get(url, headers=self.headers)
        if response.status_code >= 200 and response.status_code < 300:
            # Tickets statistics fetched successfully.
            data = response.json()
            return data
        else:
            self.handle_http_error(response)

    def post_request(self, url, payload) -> Response:
        """
        Making a REST POST request and returning the 'Response' object to the caller.
        """
        print(f"payload: {payload}")
        response = requests.post(url, json=payload, headers=self.headers)
        return response

    def put_request(self, url, payload):
        """ """
        response = requests.put(url, data=payload, headers=self.headers)
        return response
