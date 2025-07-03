
import logging

import pytest

from sdk.python.localcluster.constants import (
    TICKET_PRICE_PER_HOP,
)
from sdk.python.localcluster.node import Node

from .conftest import session_attack_nodes
from .utils import (
    create_channel,
    send_and_receive_packets_with_pop,
)

class TestNoProoFOfRelayWithSwarm:
    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "route",
        [session_attack_nodes()]
    )
    @pytest.mark.parametrize(
        "nodes_config", 
        [
                {
                "local1": {},
                "local2": {},
                "local3": {}
                },
                {
                "local1": {},
                "local2": {},
                "local3": {
                    "HOPR_DISABLE_ACK_TO_PEERS": "12D3KooWLWoHJjaS1z9cXn19DE9gPrSbYHkf7CHMbUtLUqbZKDby",
                }
                }
        ],                 
    )
    async def test_dont_send_ack(self, route, swarm3: dict[str, Node], nodes_config: dict[str, dict], config_to_yaml: str):
        """
        Test dest node not sending ack to previous relay node based on its peer id
        """

        packet_count = 100
        packets = [f"Lower ticket win probability check: #{i:08d}" for i in range(packet_count)]

        src = route[0]
        relay = route[1]
        dest = route[-1]

        logging.info(f"src: {swarm3[src].peer_id}, relay: {swarm3[relay].peer_id}, dest: {swarm3[dest].peer_id}")

        async with create_channel(
                swarm3[src], swarm3[relay], funding=2 * packet_count * TICKET_PRICE_PER_HOP
            ) as channel:
                await send_and_receive_packets_with_pop(
                    packets, src=swarm3[src], dest=swarm3[dest], path=[swarm3[relay].peer_id]
                )

       
        metrics = await swarm3[relay].api.metrics()
        metrics = "\n".join(line for line in metrics.splitlines() if not line.startswith("#"))
        metrics_dict = dict(line.split(maxsplit=1) for line in metrics.splitlines())
        # logging.info(f"Current node local2 metrics: {metrics}")

        ack_count_local2 = float(metrics_dict.get("hopr_received_ack_count{valid=\"true\"}", 0))
        logging.info(f"ack_count_local2: {ack_count_local2}")

        ticket_statistics = await swarm3[relay].api.get_tickets_statistics()
        logging.info(f"ticket_statistics: {ticket_statistics}")