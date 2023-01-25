# core-strategy

This crate contains all the Channel strategies for HOPRd.

- promiscuous strategy
- passive strategy
- random strategy (cover traffic) - NOT yet fully implemented in this crate!

## Passive Strategy

This strategy does nothing, it does not suggest to open nor close any channels.

## Promiscuous Strategy

This strategy opens or closes automatically channels based the following rules:

- if node quality is below or equal to a threshold `Q` and we have a channel opened to it, the strategy will close it
- if the stake on a channel has dropped below `Smin` (minimum channel stake threshold), strategy will close it.
  - if node quality is above `Q` and no channel is opened yet (or has been closed bc stake was low), it will try to open channel to it (with initial stake `S`).
    However, the channel is opened only if the following is both true:
  - the total node balance does not drop below `Bmin`
  - the number of channels opened by this strategy does not exceed `Nmax`

The default parameters are:

- `Q` = 0.5
- `S` = 0.1 mHOPR
- `Smin` = 0.01 mHOPR
- `Bmin` = 0.1 mHOPR
- `Nmax` = `k * sqrt(total number of all nodes in the network)`, constant `k` must be greater or equal 1.
- `k` = 1

Also, the candidates for opening (quality > `Q`), are sorted by best quality first.
So that means if some nodes cannot have channel opened to them, because we hit `Bmin` or `Nmax`,
the better quality ones were taking precedence.

The sorting algorithm is intentionally unstable, so that the nodes which have the same quality get random order.
The constant `k` can be also set to a value > 1, which will make the strategy to open more channels for smaller networks,
but it would keep the same asymptotic properties.

## Random Strategy

Currently not yet implemented.
