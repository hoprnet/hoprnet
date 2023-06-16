from ..test_data.connection_data import ConnectionData
from ..services.auth_service import AuthenticationService


class RootApiModel:
    """
    Root API object from which all other HOPR API objects will inherit common functionality.
    """

    def __init__(self) -> None:
        pass

    def get_auth_token(self) -> str:
        """
        Return hardcoded API token for now, would be interesting to use Tokens wrapper to create
        a new token for each test, or for all the test session, but depends on the test environment
        setup of the TAS project.
        """
        return AuthenticationService.get_auth_token()

    def get_base_hostname(self) -> str:
        """
        Return the base URL of the API, based on the test environment (or CI/CD) setup
        """
        return ConnectionData.BASE_HOSTNAME

    def get_port(self, nodeIndex) -> str:
        """
        Getting the port of a certain node based on the provided nodeIndex.
        The port can be computed/aquired dinamically here, depending on the test environment setup of the TAS project.
        """
        port = "{basePort}{nodeIndex}".format(basePort=ConnectionData.BASE_PORT, nodeIndex=nodeIndex)
        return port

    def get_server_url(self):
        """
        Getting the server URL to prepend in front of the API url.
        The server URL can be aquired dinamically here depending on the test environment setup of the TAS project.
        """
        return ConnectionData.SERVER_URL

    def get_rest_url(self, nodeIndex, path) -> str:
        """ """
        url = "http://{baseHostname}:{port}/{serverUrl}/{path}".format(
            baseHostname=self.get_base_hostname(),
            port=self.get_port(nodeIndex),
            serverUrl=self.get_server_url(),
            path=path,
        )
        return url

    def get_ws_url(self, nodeIndex: int, path: str) -> str:
        """ """
        ws_url = "ws://{baseHostname}:1330{nodeIndex}/{serverUrl}/{path}/websocket?apiToken={authToken}".format(
            baseHostname=self.get_base_hostname(),
            nodeIndex=nodeIndex,
            serverUrl=self.get_server_url(),
            path=path,
            authToken=self.get_auth_token(),
        )
        return ws_url

    def get_default_headers(self):
        """ """
        headers = {"X-Auth-Token": self.get_auth_token()}
        return headers

    def handle_http_error(self, response) -> None:
        """
        Generic method to handle all HTTP errors.
        """
        if response.status_code == 400:
            message = "Invalid input. One of the parameters passed is in an incorrect format. status: {status}".format(
                status=response.json()["status"]
            )
            print("Error: " + message)
            raise Exception(message)
        elif response.status_code == 401:
            raise Exception(
                "Authentication failed. status: {status} error: {error}".format(
                    status=response.json()["status"], error=response.json()["error"]
                )
            )
        elif response.status_code == 403:
            raise Exception(
                "Authorization failed. status: {status} error: {error}".format(
                    status=response.json()["status"], error=response.json()["error"]
                )
            )
        elif response.status_code == 422:
            raise Exception(
                "Unknown failure. status: {status} error: {error}".format(
                    status=response.json()["status"], error=response.json()["error"]
                )
            )
        elif response.status_code == 500:
            raise Exception(
                "Server error. status: {status} error: {error}".format(
                    status=response.json()["status"], error=response.json()["error"]
                )
            )
