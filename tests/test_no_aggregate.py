import asyncio
import logging

import pytest

from sdk.python.localcluster.constants import (
    TICKET_PRICE_PER_HOP,
)
from sdk.python.localcluster.node import Node

from .conftest import attacking_nodes
from .utils import (
    TICKET_AGGREGATION_THRESHOLD,
    PARAMETERIZED_SAMPLE_SIZE,
    check_all_tickets_redeemed,
    check_unredeemed_tickets_value,
    create_channel,
    send_and_receive_packets_with_pop,
)

class TestRedeemingWithSwarm:
    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "nodes_config", 
        [
                {
                "local1": {},
                "local2": {},
                },
                {
                "local1": {
                    "HOPR_NODE_DONT_AGGREGATE" : "12D3KooWLWoHJjaS1z9cXn19DE9gPrSbYHkf7CHMbUtLUqbZKDby"
                },
                "local2": {},
                }
        ],                 
    )
    @pytest.mark.parametrize(
        "src,dest", [tuple(attacking_nodes())]
    )
    async def test_hoprd_should_aggregate_and_redeem_tickets_in_channel_on_api_request(
        self, src: str, dest: str, swarm3: dict[str, Node], nodes_config: dict[str, dict], config_to_yaml: str
    ):
        message_count = 100

        async with create_channel(swarm3[src], swarm3[dest], funding=message_count * TICKET_PRICE_PER_HOP) as channel:
            packets = [
                f"Channel agg and redeem on 1-hop: {src} - {dest} - {src} #{i:08d}" for i in range(message_count)
            ]
            await send_and_receive_packets_with_pop(
                packets, src=swarm3[src], dest=swarm3[src], path=[swarm3[dest].peer_id]
            )

            await asyncio.wait_for(
                check_unredeemed_tickets_value(swarm3[dest], message_count * TICKET_PRICE_PER_HOP), 30.0
            )

            ticket_statistics = await swarm3[dest].api.get_tickets_statistics()
            assert ticket_statistics.unredeemed_value == message_count * TICKET_PRICE_PER_HOP

            ret_code = await asyncio.wait_for(swarm3[dest].api.channels_aggregate_tickets(channel.id), 20.0)
            logging.info(f"Aggregate tickets ret_code: {ret_code}")

            ticket_statistics = await swarm3[dest].api.get_tickets_statistics()
            assert ticket_statistics.unredeemed_value == message_count * TICKET_PRICE_PER_HOP

            balance_before_redeem = await swarm3[dest].api.balances()
            logging.info(f"Balance before redeem: {balance_before_redeem}")

            assert await swarm3[dest].api.channel_redeem_tickets(channel.id)

            await asyncio.wait_for(check_all_tickets_redeemed(swarm3[dest]), 120.0)

            ticket_statistics = await swarm3[dest].api.get_tickets_statistics()
            assert ticket_statistics.unredeemed_value == 0

            balance_after_redeem = await swarm3[dest].api.balances()
            logging.info(f"Balance after redeem: {balance_after_redeem}")

            native_difference = balance_before_redeem.native - balance_after_redeem.native
            logging.info(f"Native difference: {native_difference}")

            # if src node wont aggregate tickets for dest node then the redeem
            # operation should consume more gas, as each ticket is redeemed separately
            if(nodes_config["local1"].get("HOPR_NODE_DONT_AGGREGATE") == swarm3[dest].peer_id):
                assert native_difference > 29183308124100 # value of aggregated tickets
            else:
                assert native_difference > 0
