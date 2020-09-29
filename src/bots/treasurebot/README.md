# Treasure Bot

Treasure bot is an instance of the [HOPR chat bot](https://github.com/hoprnet/hopr-chatbot) which asks players to find treasure by assembling a 'map' which has been spread across four different 'islands' (bootstrap servers).

## Story

Captain Horatio Hopper's galleon was sunk by pirates. They didn't trust each other with the location of the treasure, so they split the secret across four maps. Delighted with this solution, they drank so much rum in celebration that now they can't remember how to read their maps! Can you find the four map pieces and uncover the treasure?

## Game Logic

When participants first communicate with Treasure bot, it messages back explaining the story and revealing the supposed location of the treasure in the form of a bootstrap address.

This is visualized as an ASCII map of a group of islands, one of which has an X on it.

Participants must alter the configuration of their .env file to connect to the new bootstrap server and message Treasure bot again. They repeat this four times, receiving a new bootstrap server and an updated map.

The final bootstrap server information is the same as the server where participants started the game. Talking to this instance of Treasure bot a second time reveals the clue that X marks the spot.

Participants must tweet the coordinates of the intersection of a large X drawn using the four smaller Xs on their map. Treasure bot checks that the tweet has the correct information and sends the participant a link to claim their prize in DAI.

Because the bots on each different server cannot communicate, the map, coordinates and final treasure location are all actually generated from a hash of the user's address.
