from enum import Enum, unique


@unique
class HTTPMethod(Enum):
    GET = "get"
    POST = "post"
    DELETE = "delete"
