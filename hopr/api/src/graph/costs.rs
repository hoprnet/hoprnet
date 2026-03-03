use super::traits::{
    CostFn, EdgeLinkObservable, EdgeNetworkObservableRead, EdgeObservableRead, EdgeProtocolObservable,
};

/// A boxed cost function accepting `(current_cost, edge_weight, path_index) -> new_cost`.
pub type BasicCostFn<C, W> = Box<dyn Fn(C, &W, usize) -> C>;

/// Build a HOPR cost function for immediate graph traversals.
///
/// Represents a backwards compatible cost function for the heartbeat protocol in v3.
pub struct SimpleHoprCostFn<C, W> {
    initial: C,
    min: Option<C>,
    cost_fn: BasicCostFn<C, W>,
}

impl<C, W> CostFn for SimpleHoprCostFn<C, W>
where
    C: Clone + PartialOrd + Send + Sync + 'static,
    W: EdgeObservableRead + Send + 'static,
{
    type Cost = C;
    type Weight = W;

    fn initial_cost(&self) -> Self::Cost {
        self.initial.clone()
    }

    fn min_cost(&self) -> Option<Self::Cost> {
        self.min.clone()
    }

    fn into_cost_fn(self) -> Box<dyn Fn(Self::Cost, &Self::Weight, usize) -> Self::Cost> {
        self.cost_fn
    }
}

impl<W> SimpleHoprCostFn<f64, W>
where
    W: EdgeObservableRead + Send + 'static,
{
    pub fn new(length: std::num::NonZeroUsize) -> Self {
        let length = length.get();
        Self {
            initial: 1.0,
            min: Some(0.0),
            cost_fn: Box::new(move |initial_cost: f64, observation: &W, path_index: usize| {
                match path_index {
                    0 => {
                        // the first edge should always go to an already connected and measured peer,
                        // otherwise use a negative cost that should remove the edge from consideration
                        if observation.immediate_qos().is_some_and(|o| o.is_connected())
                            && observation.intermediate_qos().is_some_and(|o| o.capacity().is_some())
                        {
                            return initial_cost;
                        }

                        -initial_cost
                    }
                    v if v == (length - 1) => {
                        // the last edge should always go from an already connected and measured peer,
                        // otherwise use a negative cost that should remove the edge from consideration
                        if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                            return initial_cost;
                        }

                        -initial_cost
                    }
                    _ => initial_cost,
                }
            }),
        }
    }
}

/// Build a forward HOPR cost function for full graph traversals.
pub struct HoprForwardCostFn<C, W> {
    initial: C,
    min: Option<C>,
    cost_fn: BasicCostFn<C, W>,
}

impl<C, W> CostFn for HoprForwardCostFn<C, W>
where
    C: Clone + PartialOrd + Send + Sync + 'static,
    W: EdgeObservableRead + Send + 'static,
{
    type Cost = C;
    type Weight = W;

    fn initial_cost(&self) -> Self::Cost {
        self.initial.clone()
    }

    fn min_cost(&self) -> Option<Self::Cost> {
        self.min.clone()
    }

    fn into_cost_fn(self) -> Box<dyn Fn(Self::Cost, &Self::Weight, usize) -> Self::Cost> {
        self.cost_fn
    }
}

impl<W> HoprForwardCostFn<f64, W>
where
    W: EdgeObservableRead + Send + 'static,
{
    pub fn new(length: std::num::NonZeroUsize) -> Self {
        let length = length.get();
        Self {
            initial: 1.0,
            min: Some(0.0),
            cost_fn: Box::new(move |initial_cost: f64, observation: &W, path_index: usize| {
                match path_index {
                    0 => {
                        // the first edge should always go to an already connected and measured peer,
                        // otherwise use a negative cost that should remove the edge from consideration
                        if let Some(immediate_observation) = observation.immediate_qos()
                            && immediate_observation.is_connected()
                            && let Some(intermediate_observation) = observation.intermediate_qos()
                            && intermediate_observation.capacity().is_some()
                        {
                            // loopbacks through a single peer are forbidden, therefore the first edge
                            // may consider the preexisting measurements over an immediate observation
                            return initial_cost * immediate_observation.score().max(intermediate_observation.score());
                        }

                        -initial_cost
                    }
                    v if v == (length - 1) => {
                        // The last edge (relay -> dest) may lack immediate QoS in me's graph
                        // because me doesn't directly observe relay-to-dest connectivity.
                        // Accept capacity (on-chain channel) OR connectivity + score.
                        if let Some(intermediate_observation) = observation.intermediate_qos()
                            && intermediate_observation.capacity().is_some()
                        {
                            let score = intermediate_observation.score();
                            return if score > 0.0 {
                                initial_cost * score
                            } else {
                                initial_cost
                            };
                        }

                        initial_cost
                    }
                    _ => {
                        // intermediary edges only need to have capacity and score
                        if let Some(intermediate_observation) = observation.intermediate_qos()
                            && intermediate_observation.capacity().is_some()
                        {
                            return initial_cost * intermediate_observation.score();
                        }

                        -initial_cost
                    }
                }
            }),
        }
    }
}

/// Build a HOPR cost function for full graph traversals in the return direction.
///
/// Used when the planner (`me`) constructs the return path `dest -> relay -> me`.
/// The first edge (`dest -> relay`) has relaxed requirements compared to [`HoprForwardCostFn`]
/// because the planner lacks intermediate QoS (probe) data for that edge.
///
/// Only payment channel capacity is required for the first edge. If probe-based QoS with a
/// positive score is available, that score is used to scale the edge cost; otherwise the
/// initial cost is effectively passed through without score-based scaling.
pub struct HoprReturnCostFn<C, W> {
    initial: C,
    min: Option<C>,
    cost_fn: BasicCostFn<C, W>,
}

impl<C, W> CostFn for HoprReturnCostFn<C, W>
where
    C: Clone + PartialOrd + Send + Sync + 'static,
    W: EdgeObservableRead + Send + 'static,
{
    type Cost = C;
    type Weight = W;

    fn initial_cost(&self) -> Self::Cost {
        self.initial.clone()
    }

    fn min_cost(&self) -> Option<Self::Cost> {
        self.min.clone()
    }

    fn into_cost_fn(self) -> Box<dyn Fn(Self::Cost, &Self::Weight, usize) -> Self::Cost> {
        self.cost_fn
    }
}

impl<W> HoprReturnCostFn<f64, W>
where
    W: EdgeObservableRead + Send + 'static,
{
    pub fn new(length: std::num::NonZeroUsize) -> Self {
        let length = length.get();
        Self {
            initial: 1.0,
            min: Some(0.0),
            cost_fn: Box::new(move |initial_cost: f64, observation: &W, path_index: usize| {
                match path_index {
                    0 => {
                        // The first edge of the return path (dest -> relay) requires
                        // payment channel capacity.
                        // When probes exist, scale by score; otherwise pass through
                        // the cost as baseline trust (capacity-only from on-chain).
                        if let Some(intermediate_observation) = observation.intermediate_qos()
                            && intermediate_observation.capacity().is_some()
                        {
                            let score = intermediate_observation.score();
                            return if score > 0.0 {
                                initial_cost * score
                            } else {
                                initial_cost
                            };
                        }

                        -initial_cost
                    }
                    v if v == (length - 1) => {
                        // the last edge should always go from an already connected and measured peer,
                        // otherwise use a negative cost that should remove the edge from consideration
                        if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                            let score = observation.score();
                            if score > 0.0 {
                                return initial_cost * score;
                            }
                        }

                        -initial_cost
                    }
                    _ => {
                        // intermediary edges only need to have capacity and score
                        if let Some(intermediate_observation) = observation.intermediate_qos()
                            && intermediate_observation.capacity().is_some()
                        {
                            return initial_cost * intermediate_observation.score();
                        }

                        -initial_cost
                    }
                }
            }),
        }
    }
}

/// Used for finding simple paths without the final loopback in a loopback call.
pub struct ForwardPathCostFn<C, W> {
    initial: C,
    min: Option<C>,
    cost_fn: BasicCostFn<C, W>,
}

impl<W> Default for ForwardPathCostFn<f64, W>
where
    W: EdgeObservableRead + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C, W> CostFn for ForwardPathCostFn<C, W>
where
    C: Clone + PartialOrd + Send + Sync + 'static,
    W: EdgeObservableRead + Send + 'static,
{
    type Cost = C;
    type Weight = W;

    fn initial_cost(&self) -> Self::Cost {
        self.initial.clone()
    }

    fn min_cost(&self) -> Option<Self::Cost> {
        self.min.clone()
    }

    fn into_cost_fn(self) -> Box<dyn Fn(Self::Cost, &Self::Weight, usize) -> Self::Cost> {
        self.cost_fn
    }
}

impl<W> ForwardPathCostFn<f64, W>
where
    W: EdgeObservableRead + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            initial: 1.0,
            min: Some(0.0),
            cost_fn: Box::new(move |initial_cost: f64, observation: &W, path_index: usize| {
                match path_index {
                    0 => {
                        // the first edge should always go to an already connected and measured peer,
                        // otherwise use a negative cost that should remove the edge from consideration
                        if let Some(immediate_observation) = observation.immediate_qos()
                            && immediate_observation.is_connected()
                            && let Some(intermediate_observation) = observation.intermediate_qos()
                            && intermediate_observation.capacity().is_some()
                        {
                            // loopbacks through a single peer are forbidden, therefore the first edge
                            // may consider the preexisting measurements over an immediate observation
                            return initial_cost * immediate_observation.score().max(intermediate_observation.score());
                        }

                        -initial_cost
                    }
                    _ => {
                        // intermediary edges only need to have capacity and score.
                        // When capacity exists but no probes have run yet (score 0), pass through
                        // initial_cost to allow the first probe to discover this path.
                        if let Some(intermediate_observation) = observation.intermediate_qos()
                            && intermediate_observation.capacity().is_some()
                        {
                            let score = intermediate_observation.score();
                            return if score > 0.0 {
                                initial_cost * score
                            } else {
                                initial_cost
                            };
                        }

                        -initial_cost
                    }
                }
            }),
        }
    }
}
