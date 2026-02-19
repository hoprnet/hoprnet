use super::traits::{
    CostFn, EdgeLinkObservable, EdgeNetworkObservableRead, EdgeObservableRead, EdgeProtocolObservable,
};

/// Build a HOPR cost function for immediate graph traversals.
///
/// Represents a backwards compatible cost function for the heartbeat protocol in v3.
#[allow(clippy::type_complexity)]
pub struct SimpleHoprCostFn<C, W> {
    initial: C,
    min: Option<C>,
    cost_fn: Box<dyn Fn(C, &W, usize) -> C>,
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
    pub fn new(length: usize) -> Self {
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
                    v if v == length => {
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

/// Build a HOPR cost function for full graph traversals.
#[allow(clippy::type_complexity)]
pub struct HoprCostFn<C, W> {
    initial: C,
    min: Option<C>,
    cost_fn: Box<dyn Fn(C, &W, usize) -> C>,
}

impl<C, W> CostFn for HoprCostFn<C, W>
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

impl<W> HoprCostFn<f64, W>
where
    W: EdgeObservableRead + Send + 'static,
{
    pub fn new(length: usize) -> Self {
        Self {
            initial: 1.0,
            min: Some(0.0),
            cost_fn: Box::new(move |initial_cost: f64, observation: &W, path_index: usize| {
                match path_index {
                    0 => {
                        // the first edge should always go to an already connected and measured peer,
                        // otherwise use a negative cost that should remove the edge from consideration
                        if observation.immediate_qos().is_some_and(|o| o.is_connected())
                            && let Some(intermediate_observation) = observation.intermediate_qos()
                            && intermediate_observation.capacity().is_some()
                        {
                            return initial_cost * intermediate_observation.score();
                        }

                        -initial_cost
                    }
                    v if v == length => {
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
#[allow(clippy::type_complexity)]
pub struct LoopbackPathCostFn<C, W> {
    initial: C,
    min: Option<C>,
    cost_fn: Box<dyn Fn(C, &W, usize) -> C>,
}

impl<W> Default for LoopbackPathCostFn<f64, W>
where
    W: EdgeObservableRead + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C, W> CostFn for LoopbackPathCostFn<C, W>
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

impl<W> LoopbackPathCostFn<f64, W>
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
                        if observation.immediate_qos().is_some_and(|o| o.is_connected())
                            && let Some(intermediate_observation) = observation.intermediate_qos()
                            && intermediate_observation.capacity().is_some()
                        {
                            return initial_cost * intermediate_observation.score();
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
