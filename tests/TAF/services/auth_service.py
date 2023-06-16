from ..test_data.connection_data import ConnectionData


class AuthenticationService:
    def __init__(self) -> None:
        pass

    @staticmethod
    def get_auth_token() -> str:
        """
        Return hardcoded API token for now, would be interesting to use Tokens wrapper to create
        a new token for each test, or for all the test session, but depends on the test environment
        setup of the TAS project.
        """
        return ConnectionData.AUTH_TOKEN
