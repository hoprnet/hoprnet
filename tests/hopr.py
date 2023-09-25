import json
import logging
from hoprd_sdk import Configuration, ApiClient
from hoprd_sdk.models import MessagesBody, ChannelsBody
from hoprd_sdk.rest import ApiException
from hoprd_sdk.api import NodeApi, MessagesApi, AccountApi, ChannelsApi, PeersApi
from urllib3.exceptions import MaxRetryError


def getlogger():
    logger = logging.getLogger("tests")
    logger.setLevel(logging.INFO)

    return logger


log = getlogger()


class HoprdAPI:
    """
    HOPRd API helper to handle exceptions and logging.
    """

    def __init__(self, url: str, token: str):
        self.configuration = Configuration()
        self.configuration.host = f"{url}/api/v3"
        self.configuration.api_key["x-auth-token"] = token
        print(self.configuration.host)

    async def balances(self, type: str or list[str] = "all"):
        """
        Returns the balance of the node.
        :param: type: str =  "all" | "hopr" | "native" | "safe_native" | "safe_hopr"
        :return: balances: dict | int
        """
        all_types = ["hopr", "native", "safe_native", "safe_hopr"]
        if type == "all":
            type = all_types
        elif isinstance(type, str):
            type = [type]

        for t in type:
            if t not in all_types:
                log.error(
                    f"Type `{type}` not supported. Use `all`, `hopr`, `native`, "
                    + "`safeNative` or `safeHopr`"
                )
                return None

        log.debug("Getting own balance")

        try:
            with ApiClient(self.configuration) as client:
                account_api = AccountApi(client)
                thread = account_api.account_get_balances(async_req=True)
                response = thread.get()
        except ApiException as e:
            body = json.loads(e.body.decode())
            log.error(f"ApiException calling AccountApi->account_get_balances: {body}")
            return None
        except OSError:
            log.error("OSError calling AccountApi->account_get_balances")
            return None
        except MaxRetryError:
            log.error("MaxRetryError calling AccountApi->account_get_balances")
            return None

        return_dict = {}

        for t in type:
            return_dict[t] = int(getattr(response, t))

        return return_dict if len(return_dict) > 1 else return_dict[type[0]]

    async def open_channel(self, peer_address: str, amount: str):
        """
        Opens a channel with the given peer_address and amount.
        :param: peer_address: str
        :param: amount: str
        :return: bool
        """
        log.debug(f"Opening channel to '{peer_address}'")

        body = ChannelsBody(peer_address, amount)
        try:
            with ApiClient(self.configuration) as client:
                channels_api = ChannelsApi(client)
                thread = channels_api.channels_open_channel(body=body, async_req=True)
                response = thread.get()
                log.debug(
                    "Response after trying to open a channel to "
                    + f"{peer_address} {response}"
                )
        except ApiException as e:
            body = json.loads(e.body.decode())
            if body["status"] == "CHANNEL_ALREADY_OPEN":
                log.debug("Channel already opened")
            else:
                log.error(
                    f"ApiException calling ChannelsApi->channels_open_channel: {body}"
                )
        except OSError:
            log.error("OSError calling ChannelsApi->channels_open_channel")
            return False
        except MaxRetryError:
            log.error("MaxRetryError calling ChannelsApi->channels_open_channel")
            return False

        return True

    async def close_channel(self, channel_id: str):
        """
        Closes a given channel.
        :param: channel_id: str
        :return: bool
        """
        log.debug(f"Closing channel with id {channel_id}")

        try:
            with ApiClient(self.configuration) as client:
                channels_api = ChannelsApi(client)
                thread = channels_api.channels_close_channel(channel_id, async_req=True)
                thread.get()
        except ApiException as e:
            body = json.loads(e.body.decode())
            log.error(
                f"ApiException calling ChannelsApi->channels_close_channel: {body}"
            )
            return False
        except OSError:
            log.error("OSError calling ChannelsApi->channels_close_channel")
            return False
        except MaxRetryError:
            log.error("MaxRetryError calling ChannelsApi->channels_close_channel")
            return False

        return True

    async def incoming_channels(self, only_id: bool = False):
        """
        Returns all open incoming channels.
        :return: channels: list
        """
        log.debug("Getting open incoming channels")

        try:
            with ApiClient(self.configuration) as client:
                channels_api = ChannelsApi(client)
                thread = channels_api.channels_get_channels(
                    full_topology="false", including_closed="false", async_req=True
                )
                response = thread.get()
        except ApiException as e:
            body = json.loads(e.body.decode())
            log.error(
                f"ApiException calling ChannelsApi->channels_get_channels: {body}"
            )
            return []
        except OSError:
            log.error("OSError calling ChannelsApi->channels_get_channels")
            return []
        except MaxRetryError:
            log.error("MaxRetryError calling ChannelsApi->channels_get_channels")
            return []

        if not hasattr(response, "incoming"):
            log.warning("Response does not contain `incoming`")
            return []

        if len(response.incoming) == 0:
            log.info("No incoming channels")
            return []

        if only_id:
            return [channel.id for channel in response.incoming]
        else:
            return response.incoming

    async def outgoing_channels(self, only_id: bool = False):
        """
        Returns all open outgoing channels.
        :return: channels: list
        """
        log.debug("Getting open outgoing channels")

        try:
            with ApiClient(self.configuration) as client:
                channels_api = ChannelsApi(client)
                thread = channels_api.channels_get_channels(
                    full_topology="false", including_closed="false", async_req=True
                )
                response = thread.get()
        except ApiException as e:
            body = json.loads(e.body.decode())
            log.error(
                f"ApiException calling ChannelsApi->channels_get_channels: {body}"
            )
            return []
        except OSError as e:
            body = json.loads(e.body.decode())
            log.error(f"OSError calling ChannelsApi->channels_get_channels: {body}")
            return []
        except MaxRetryError:
            log.error("MaxRetryError calling ChannelsApi->channels_get_channels")
            return []

        if not hasattr(response, "outgoing"):
            log.warning("Response does not contain `outgoing`")
            return []

        if len(response.outgoing) == 0:
            log.info("No outgoing channels")
            return []

        if only_id:
            return [channel.id for channel in response.outgoing]
        else:
            return response.outgoing

    async def all_channels(self, include_closed: bool):
        """
        Returns all channels.
        :param: include_closed: bool
        :return: channels: list
        """
        log.debug(f"Getting all channels (include_closed={include_closed})")

        try:
            with ApiClient(self.configuration) as client:
                channels_api = ChannelsApi(client)
                thread = channels_api.channels_get_channels(
                    full_topology="true",
                    including_closed=include_closed,
                    async_req=True,
                )
                response = thread.get()
        except ApiException as e:
            body = json.loads(e.body.decode())
            log.error(
                f"ApiException calling ChannelsApi->channels_get_channels: {body}"
            )
            return []
        except OSError:
            log.error("OSError calling ChannelsApi->channels_get_channels")
            return []
        except MaxRetryError:
            log.error("MaxRetryError calling ChannelsApi->channels_get_channels")
            return []
        else:
            return response

    async def get_unique_nodeAddress_peerId_aggbalance_links(self):
        """
        Returns a dict containing all unique source_peerId-source_address links.
        """
        log.debug("Getting channel topology")

        try:
            with ApiClient(self.configuration) as client:
                channels_api = ChannelsApi(client)
                thread = channels_api.channels_get_channels(
                    full_topology="true", including_closed="false", async_req=True
                )
                response = thread.get()
        except ApiException as e:
            body = json.loads(e.body.decode())
            log.error(
                f"ApiException calling ChannelsApi->channels_get_channels: {body}"
            )
            return None
        except OSError:
            log.error("OSError calling ChannelsApi->channels_get_channels")
            return None
        except MaxRetryError:
            log.error("MaxRetryError calling ChannelsApi->channels_get_channels")
            return None

        if not hasattr(response, "all"):
            log.error("Response does not contain `all`")
            return None

        peerid_address_aggbalance_links = {}
        for item in response.all:
            if not hasattr(item, "source_peer_id") or not hasattr(
                item, "source_address"
            ):
                log.error(
                    "Response does not contain `source_peerid` or `source_address`"
                )
                continue

            if not hasattr(item, "status"):
                log.error("Response does not contain `status`")
                continue

            source_peer_id = item.source_peer_id
            source_address = item.source_address
            balance = int(item.balance) / 1e18

            if item.status != "Open":
                # Other Statuses: "Waiting for commitment", "Closed", "Pending to close"
                # Ensures that nodes must have at least 1 open channel in to receive ct
                continue

            if source_peer_id not in peerid_address_aggbalance_links:
                peerid_address_aggbalance_links[source_peer_id] = {
                    "source_node_address": source_address,
                    "channels_balance": balance,
                }

            else:
                peerid_address_aggbalance_links[source_peer_id][
                    "channels_balance"
                ] += balance

        return peerid_address_aggbalance_links

    async def ping(self, peer_id: str, metric: str = "latency"):
        """
        Pings the given peer_id and returns the measure.
        :param: peer_id: str
        :param: metric: str = "latency"
        :return: measure: int
        """
        log.debug(f"Pinging peer {peer_id}")

        try:
            with ApiClient(self.configuration) as client:
                peers_api = PeersApi(client)
                thread = peers_api.peers_ping_peer(peer_id, async_req=True)
                response = thread.get()
        except ApiException as e:
            body = json.loads(e.body.decode())
            log.error(f"ApiException calling PeersApi->peers_ping_peer: {body}")
            return 0
        except OSError:
            log.error("OSError calling PeersApi->peers_ping_peer")
            return 0
        except MaxRetryError:
            log.error("MaxRetryError calling PeersApi->peers_ping_peer")
            return 0

        if not hasattr(response, metric):
            log.error(f"No `{metric}` measure from peer {peer_id}")
            return 0

        measure = int(getattr(response, metric))

        log.debug(f"Measured {measure:3d}({metric}) from peer {peer_id}")
        return measure

    async def peers(
        self,
        params: list or str = "peer_id",
        status: str = "connected",
    ):
        """
        Returns a list of peers.
        :param: param: list or str = "peer_id"
        :param: status: str = "connected"
        :param: quality: int = 0..1
        :return: peers: list
        """
        log.debug("Getting peers")

        params = [params] if isinstance(params, str) else params

        try:
            with ApiClient(self.configuration) as client:
                node_api = NodeApi(client)
                thread = node_api.node_get_peers(async_req=True)
                response = thread.get()
        except ApiException as e:
            body = json.loads(e.body.decode())
            log.error(f"ApiException calling NodeApi->node_get_peers: {body}")
            return []
        except OSError:
            log.error("OSError calling NodeApi->node_get_peers")
            return []
        except MaxRetryError:
            log.error("MaxRetryError calling NodeApi->node_get_peers")
            return []

        if not hasattr(response, status):
            log.error(f"No `{status}` returned from the API")
            return []

        if len(getattr(response, status)) == 0:
            log.info(f"No peer with status `{status}`")
            return []

        for param in params:
            if not hasattr(getattr(response, status)[0], param):
                log.error(f"No param `{param}` found for peers")
                return []

        output_list = []
        for peer in getattr(response, status):
            output_list.append({param: getattr(peer, param) for param in params})

        return output_list

    async def get_address(self, address: str):
        """
        Returns the address of the node.
        :param: address: str = "hopr" | "native"
        :return: address: str
        """
        log.debug("Getting address")

        try:
            with ApiClient(self.configuration) as client:
                account_api = AccountApi(client)
                thread = account_api.account_get_address(async_req=True)
                response = thread.get()
        except ApiException as e:
            body = json.loads(e.body.decode())
            log.error(f"ApiException calling AccountApi->account_get_address: {body}")
            return None
        except OSError:
            log.error("OSError calling AccountApi->account_get_address")
            return None
        except MaxRetryError:
            log.error("MaxRetryError calling AccountApi->account_get_address")
            return None

        if not hasattr(response, address):
            log.error(f"No {address} returned from the API")
            return None

        return getattr(response, address)

    async def send_message(
        self, destination: str, message: str, hops: list[str], tag: int = 0x0320
    ) -> bool:
        """
        Sends a message to the given destination.
        :param: destination: str
        :param: message: str
        :param: hops: list[str]
        :param: tag: int = 0x0320
        :return: bool
        """
        log.debug("Sending message")

        body = MessagesBody(tag, message, destination, path=hops)
        try:
            with ApiClient(self.configuration) as client:
                message_api = MessagesApi(client)
                thread = message_api.messages_send_message(body=body, async_req=True)
                thread.get()
        except ApiException as e:
            body = json.loads(e.body.decode())
            log.error(f"ApiException calling MessageApi->messages_send_message: {body}")
            return False
        except OSError:
            log.error("OSError calling MessageApi->messages_send_message:")
            return False
        except MaxRetryError:
            log.error("MaxRetryError calling MessageApi->messages_send_message")
            return False

        return True
