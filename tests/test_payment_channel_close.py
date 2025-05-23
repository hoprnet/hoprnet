import asyncio
import logging

import pytest
from sdk.python.api.channelstatus import ChannelStatus
from sdk.python.localcluster.constants import (
    OPEN_CHANNEL_FUNDING_VALUE_HOPR,
    TICKET_PRICE_PER_HOP,
)
from sdk.python.localcluster.node import Node

from .conftest import barebone_nodes, attacking_nodes
from .utils import (
    PARAMETERIZED_SAMPLE_SIZE,
    create_channel,
    send_and_receive_packets_with_pop,
    shuffled,
    check_channel_status,
)

# Define the conversion rate between HOPR and native tokens
HOPR_TO_NATIVE_CONVERSION_RATE = 1000000000000
NATIVE_TO_HOPR_CONVERSION_RATE = 1 / HOPR_TO_NATIVE_CONVERSION_RATE

class TestNoProoFOfRelayWithSwarm:
    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "packets_count", [1, 100]
    )
    @pytest.mark.parametrize(
        "src,dest", [tuple(shuffled(barebone_nodes())[:2]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
    )
    async def test_opening_and_closing_payment_channel_does_not_drain_funds(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):

        packet_count = 1

        balance_before_open = await swarm7[src].api.balances()
        logging.info(f"Balance before open: {balance_before_open}")

        async with create_channel(swarm7[src], swarm7[dest], funding=TICKET_PRICE_PER_HOP * packet_count) as channel:

            balance_after_open = await swarm7[src].api.balances()

            logging.info(f"Balance after open: {balance_after_open}")

        
        balance_after_close = await swarm7[src].api.balances()
        logging.info(f"Balance after close: {balance_after_close}")

        assert balance_before_open.safe_hopr == balance_after_close.safe_hopr + balance_after_close.hopr

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "src,dest", [tuple(attacking_nodes())]
    )
    async def test_send_until_value_exceeds_redemption_cost_then_close(
        self, src: str, dest: str, swarm3: dict[str, Node]
    ):
        """
        Opens a channel, sends messages until unredeemed ticket value on dest
        is barely less than estimated redemption cost, then closes the channel.
        This test assumes conversion of gas to HOPR token as a constant. Winning
        probability is fixed to 1.0 and ticket price is fixed to 100. 
        """

        message_count = 100
        estimated_redemption_cost_wei = 3017220877431800 # estimated cost of ticket redemption in native
        messages_sent_count = 0
        channel_closed_due_to_cost = False
        

        funding = message_count * TICKET_PRICE_PER_HOP
        channel = await swarm3[src].api.open_channel(swarm3[dest].peer_id, str(int(funding)))
        assert channel is not None
        await asyncio.wait_for(check_channel_status(swarm3[src], swarm3[dest], status=ChannelStatus.Open), 10.0)
        for i in range(message_count):
            message_content = f"message_series_{i}"
                
            await send_and_receive_packets_with_pop(
                [message_content], src=swarm3[src], dest=swarm3[src], path=[swarm3[dest].peer_id]
            )

            messages_sent_count += 1
            dest_ticket_stats = await swarm3[dest].api.get_tickets_statistics()
            current_unredeemed_value_on_dest = int(dest_ticket_stats.unredeemed_value or 0) * HOPR_TO_NATIVE_CONVERSION_RATE

            logging.info(
                f"Message {messages_sent_count} sent. "
                f"Dest unredeemed value: {current_unredeemed_value_on_dest} wei. "
                f"Target redemption cost: {estimated_redemption_cost_wei} wei."
            )

            if current_unredeemed_value_on_dest + HOPR_TO_NATIVE_CONVERSION_RATE >= estimated_redemption_cost_wei:
                logging.info(
                    f"Unredeemed value ({current_unredeemed_value_on_dest}) "
                    f"met/exceeded redemption cost ({estimated_redemption_cost_wei}). Closing channel."
                )

                close_response = await swarm3[src].api.close_channel(channel.id)
                assert close_response is not None, "Failed to initiate channel closing."

                channel_closed_due_to_cost = True
                break 
            else:
                if i == message_count - 1:
                    logging.warning(
                        f"Max messages ({message_count}) sent, but unredeemed value "
                        f"({current_unredeemed_value_on_dest}) did not reach redemption cost "
                        f"({estimated_redemption_cost_wei}). Channel will be closed by context manager."
                        )
                    close_response = await swarm3[src].api.close_channel(channel.id)
                    assert close_response is not None, "Failed to initiate channel closing."

                

        assert channel_closed_due_to_cost, f"Channel was not closed due to cost. "
       
        logging.info(f"Test completed. Messages sent: {messages_sent_count}. Channel closed due to cost: {channel_closed_due_to_cost}")