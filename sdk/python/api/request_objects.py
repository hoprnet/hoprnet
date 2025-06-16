from dataclasses import dataclass, field, fields
from typing import Any, List, Union


def api_field(api_key: str, **kwargs):
    metadata = kwargs.pop("metadata", {})
    metadata["api_key"] = api_key
    return field(metadata=metadata, **kwargs)


class ApiRequestObject:
    @property
    def as_dict(self) -> dict:
        result = {}
        for f in fields(self):
            api_key = f.metadata.get("api_key", f.name)
            result[api_key] = getattr(self, f.name)
        return result

    @property
    def as_header_string(self) -> str:
        return "&".join([f"{k}={v}" for k, v in self.as_dict.items()])


@dataclass
class OpenChannelBody(ApiRequestObject):
    amount: str = api_field("amount")
    destination: str = api_field("destination")


@dataclass
class FundChannelBody(ApiRequestObject):
    amount: str = api_field("amount")


@dataclass
class GetChannelsBody(ApiRequestObject):
    full_topology: str = api_field("fullTopology")
    including_closed: str = api_field("includingClosed")


@dataclass
class GetPeersBody(ApiRequestObject):
    quality: float = api_field("quality")


@dataclass
class CreateSessionBody(ApiRequestObject):
    capabilities: List[Any] = api_field("capabilities")
    destination: str = api_field("destination")
    listen_host: str = api_field("listenHost")
    forward_path: Union[str, dict] = api_field("forwardPath")
    return_path: Union[str, dict] = api_field("returnPath")
    target: Union[str, dict] = api_field("target")
    response_buffer: str = api_field("responseBuffer")


@dataclass
class SessionCapabilitiesBody(ApiRequestObject):
    retransmission: bool = api_field("Retransmission")
    segmentation: bool = api_field("Segmentation")
    retransmission_ack_only = api_field("RetransmissionAckOnly")
    no_delay: bool = api_field("NoDelay")

    @property
    def as_array(self) -> list:
        return [f.metadata["api_key"] for f in fields(self) if getattr(self, f.name)]


@dataclass
class SessionPathBodyRelayers(ApiRequestObject):
    relayers: List[str] = api_field("IntermediatePath")


@dataclass
class SessionPathBodyHops(ApiRequestObject):
    hops: int = api_field("Hops")

    def post_init(self):
        self.hops = int(self.hops)


@dataclass
class SessionTargetBody(ApiRequestObject):
    service: int = api_field("Service")


@dataclass
class WithdrawBody(ApiRequestObject):
    address: str = api_field("address")
    amount: str = api_field("amount")
