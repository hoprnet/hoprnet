from dataclasses import dataclass
from typing import Any, List, Union

from api_lib.objects.request import APIfield, RequestData


@dataclass
class OpenChannelBody(RequestData):
    amount: str = APIfield("amount")
    destination: str = APIfield("destination")


@dataclass
class FundChannelBody(RequestData):
    amount: str = APIfield("amount")


@dataclass
class GetChannelsBody(RequestData):
    full_topology: bool = APIfield("fullTopology", False)
    including_closed: bool = APIfield("includingClosed", False)


@dataclass
class GetPeersBody(RequestData):
    quality: float = APIfield("quality")


@dataclass
class CreateSessionBody(RequestData):
    capabilities: List[Any] = APIfield("capabilities")
    destination: str = APIfield("destination")
    listen_host: str = APIfield("listenHost")
    forward_path: Union[str, dict] = APIfield("forwardPath")
    return_path: Union[str, dict] = APIfield("returnPath")
    target: Union[str, dict] = APIfield("target")
    response_buffer: str = APIfield("responseBuffer")


@dataclass
class SessionCapabilitiesBody(RequestData):
    retransmission: bool = APIfield("Retransmission", False)
    segmentation: bool = APIfield("Segmentation", False)
    retransmission_ack_only: bool = APIfield("RetransmissionAckOnly", False)
    no_delay: bool = APIfield("NoDelay", False)
    no_rate_control: bool = APIfield("NoRateControl", False)


@dataclass
class SessionPathBodyRelayers(RequestData):
    relayers: List[str] = APIfield("IntermediatePath")


@dataclass
class SessionPathBodyHops(RequestData):
    hops: int = APIfield("Hops")


@dataclass
class SessionTargetBody(RequestData):
    service: int = APIfield("Service")


@dataclass
class WithdrawBody(RequestData):
    address: str = APIfield("address")
    amount: str = APIfield("amount")
