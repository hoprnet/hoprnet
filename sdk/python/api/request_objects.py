from decimal import Decimal, ROUND_UP


class ApiRequestObject:
    def __init__(self, *args, **kwargs):
        if not hasattr(self, "keys"):
            self.keys = {}

        if args:
            kwargs.update(args[0])

        kwargs = {k: v for k, v in kwargs.items() if not k.startswith("__")}
        kwargs.pop("self", None)

        if set(kwargs.keys()) != set(self.keys.keys()):
            raise ValueError(f"Keys mismatch: {set(kwargs.keys())} != {set(self.keys.keys())}")

        for key, value in kwargs.items():
            setattr(self, key, value)

        self.post_init()

    @property
    def as_dict(self) -> dict:
        return {value: getattr(self, key) for key, value in self.keys.items()}

    @property
    def as_header_string(self) -> str:
        attrs_as_dict = {value: getattr(self, key) for key, value in self.keys.items()}
        return "&".join([f"{k}={v}" for k, v in attrs_as_dict.items()])

    def post_init(self):
        pass


class OpenChannelBody(ApiRequestObject):
    keys = {
        "amount": "amount",
        "destination": "destination",
    }

    def __init__(self, amount: Decimal, destination: str):
        super().__init__(vars())

    def post_init(self):
        self.amount = f"{self.amount.quantize(Decimal('1.0000000000'), rounding=ROUND_UP)} wxHOPR"


class CloseChannelsBody(ApiRequestObject):
    keys = {"direction": "direction", "status": "status"}

    def __init__(self, direction: str, status: str):
        super().__init__(vars())


class FundChannelBody(ApiRequestObject):
    keys = {"amount": "amount"}

    def __init__(self, amount: Decimal):
        super().__init__(vars())

    def post_init(self):
        self.amount = f"{self.amount.quantize(Decimal('1.0000000000'), rounding=ROUND_UP)} wxHOPR"


class GetChannelsBody(ApiRequestObject):
    keys = {
        "full_topology": "fullTopology",
        "including_closed": "includingClosed",
    }

    def __init__(self, full_topology: str, including_closed: str):
        super().__init__(vars())


class GetPeersBody(ApiRequestObject):
    keys = {"quality": "quality"}

    def __init__(self, quality: float):
        super().__init__(vars())


class CreateSessionBody(ApiRequestObject):
    keys = {
        "capabilities": "capabilities",
        "destination": "destination",
        "listen_host": "listenHost",
        "forward_path": "forwardPath",
        "return_path": "returnPath",
        "target": "target",
        "response_buffer": "responseBuffer",
    }

    def __init__(
        self,
        capabilities: list,
        destination: str,
        listen_host: str,
        forward_path: str,
        return_path: str,
        target: str,
        response_buffer: str,
    ):
        super().__init__(vars())


class SessionCapabilitiesBody(ApiRequestObject):
    keys = {
        "retransmission": "Retransmission",
        "segmentation": "Segmentation",
        "retransmission_ack_only": "RetransmissionAckOnly",
        "no_delay": "NoDelay",
    }

    def __init__(
        self,
        retransmission: bool = False,
        segmentation: bool = False,
        retransmission_ack_only: bool = False,
        no_delay: bool = False,
    ):
        super().__init__(vars())

    @property
    def as_array(self) -> list:
        return [self.keys[var] for var in vars(self) if var in self.keys and vars(self)[var]]


class SessionPathBodyRelayers(ApiRequestObject):
    keys = {
        "relayers": "IntermediatePath",
    }

    def __init__(self, relayers: list[str]):
        super().__init__(vars())


class SessionPathBodyHops(ApiRequestObject):
    keys = {
        "hops": "Hops",
    }

    def __init__(self, hops: int = 0):
        super().__init__(vars())

    def post_init(self):
        self.hops = int(self.hops)


class SessionTargetBody(ApiRequestObject):
    keys = {
        "service": "Service",
    }

    def __init__(self, service: int = 0):
        super().__init__(vars())


class SendMessageBody(ApiRequestObject):
    keys = {"body": "body", "hops": "hops", "path": "path", "destination": "destination", "tag": "tag"}

    def __init__(self, body: str, hops: int, path: list[str], destination: str, tag: int):
        super().__init__(vars())


class WithdrawBody(ApiRequestObject):
    keys = {"address": "address", "amount": "amount"}

    def __init__(self, address: str, amount: str):
        super().__init__(vars())
