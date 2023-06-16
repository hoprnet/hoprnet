from strenum import StrEnum


class ConnectionData(StrEnum):
    # BASE_HOSTNAME = "172.17.0.2",
    BASE_HOSTNAME = "localhost"
    BASE_PORT = "1330"
    SERVER_URL = ("api/v2",)
    AUTH_TOKEN = "%th1s-IS-a-S3CR3T-ap1-PUSHING-b1ts-TO-you%"
