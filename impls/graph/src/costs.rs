use hopr_api::graph::{
    CostFn, EdgeLinkObservable,
    traits::{EdgeNetworkObservableRead, EdgeObservableRead, EdgeProtocolObservable},
};

use crate::Observations;

/// Build a HOPR cost function for immediate graph traversals.
#[allow(clippy::type_complexity)]
pub struct SimpleHoprCostFn {
    cost_fn: Box<
        dyn Fn(
            <SimpleHoprCostFn as CostFn>::Cost,
            &<SimpleHoprCostFn as CostFn>::Weight,
            usize,
        ) -> <SimpleHoprCostFn as CostFn>::Cost,
    >,
}

impl SimpleHoprCostFn {
    pub fn new(length: usize) -> Self {
        Self {
            cost_fn: Box::new(
                move |initial_cost: f64, observation: &crate::Observations, path_index: usize| {
                    match path_index {
                        0 => {
                            // the first edge should always go to an already connected and measured peer,
                            // otherwise use a negative cost that should remove the edge from consideration
                            if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                                // TODO(20260217: extend once 1-hop probing verifiably works)
                                // && o.average_latency().is_some_and(|latency| latency)
                                if observation.intermediate_qos().is_some_and(|o| o.capacity().is_some()) {
                                    return initial_cost;
                                }
                            }

                            -initial_cost
                        }
                        v if v == length => {
                            // the last edge should always go from an already connected and measured peer,
                            // otherwise use a negative cost that should remove the edge from consideration
                            if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                                // TODO(20260217: extend once 1-hop probing verifiably works)
                                // if observation.intermediate_qos().is_some_and(|o| o.capacity().o.score()()) {
                                return initial_cost;
                                // }
                            }

                            -initial_cost
                        }
                        _ => initial_cost,
                    }
                },
            ),
        }
    }
}

impl CostFn for SimpleHoprCostFn {
    type Cost = f64;
    type Weight = Observations;

    fn initial_cost(&self) -> Self::Cost {
        1.0
    }

    fn min_cost(&self) -> Option<Self::Cost> {
        Some(0.0)
    }

    #[warn(clippy::type_complexity)]
    fn into_cost_fn(self) -> Box<dyn Fn(Self::Cost, &Self::Weight, usize) -> Self::Cost> {
        self.cost_fn
    }
}

/// Build a HOPR cost function for full graph traversals.
#[allow(clippy::type_complexity)]
pub struct HoprCostFn {
    cost_fn: Box<
        dyn Fn(<HoprCostFn as CostFn>::Cost, &<HoprCostFn as CostFn>::Weight, usize) -> <HoprCostFn as CostFn>::Cost,
    >,
}

impl HoprCostFn {
    pub fn new(length: usize) -> Self {
        Self {
            cost_fn: Box::new(
                move |initial_cost: f64, observation: &crate::Observations, path_index: usize| {
                    match path_index {
                        0 => {
                            // the first edge should always go to an already connected and measured peer,
                            // otherwise use a negative cost that should remove the edge from consideration
                            if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                                if let Some(intermediate_observation) = observation.intermediate_qos() {
                                    if intermediate_observation.capacity().is_some() {
                                        return initial_cost * intermediate_observation.score();
                                    }
                                }
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
                        _ => initial_cost,
                    }
                },
            ),
        }
    }
}

impl CostFn for HoprCostFn {
    type Cost = f64;
    type Weight = Observations;

    fn initial_cost(&self) -> Self::Cost {
        1.0
    }

    fn min_cost(&self) -> Option<Self::Cost> {
        Some(0.0)
    }

    #[warn(clippy::type_complexity)]
    fn into_cost_fn(self) -> Box<dyn Fn(Self::Cost, &Self::Weight, usize) -> Self::Cost> {
        self.cost_fn
    }
}

/// Used for finding simple paths without the final loopback in a loopback call.
#[allow(clippy::type_complexity)]
pub struct LoopbackPathCostFn {
    cost_fn: Box<
        dyn Fn(
            <LoopbackPathCostFn as CostFn>::Cost,
            &<LoopbackPathCostFn as CostFn>::Weight,
            usize,
        ) -> <LoopbackPathCostFn as CostFn>::Cost,
    >,
}

impl Default for LoopbackPathCostFn {
    fn default() -> Self {
        Self::new()
    }
}

impl LoopbackPathCostFn {
    pub fn new() -> Self {
        Self {
            cost_fn: Box::new(
                move |initial_cost: f64, observation: &crate::Observations, path_index: usize| {
                    match path_index {
                        0 => {
                            // the first edge should always go to an already connected and measured peer,
                            // otherwise use a negative cost that should remove the edge from consideration
                            if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                                // TODO(20260217: extend once 1-hop probing verifiably works)
                                // && o.average_latency().is_some_and(|latency| latency)
                                if observation.intermediate_qos().is_some_and(|o| o.capacity().is_some()) {
                                    return initial_cost;
                                }
                            }

                            -initial_cost
                        }
                        _ => {
                            // the last peer is the one before a hop back to ourselves, so it's capacity must exist
                            if observation.intermediate_qos().is_some_and(|o| o.capacity().is_some()) {
                                return initial_cost;
                            }

                            -initial_cost
                        }
                    }
                },
            ),
        }
    }
}

impl CostFn for LoopbackPathCostFn {
    type Cost = f64;
    type Weight = Observations;

    fn initial_cost(&self) -> Self::Cost {
        1.0
    }

    fn min_cost(&self) -> Option<Self::Cost> {
        Some(0.0)
    }

    #[warn(clippy::type_complexity)]
    fn into_cost_fn(self) -> Box<dyn Fn(Self::Cost, &Self::Weight, usize) -> Self::Cost> {
        self.cost_fn
    }
}
