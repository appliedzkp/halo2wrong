use super::main_gate::MainGateConfig;
use super::{integer::IntegerConfig, range::RangeConfig};
use crate::circuit::{AssignedCondition, AssignedInteger};
use crate::rns::Integer;
use halo2::arithmetic::FieldExt;

#[derive(Default, Clone, Debug)]
pub struct Point<N: FieldExt> {
    x: Integer<N>,
    y: Integer<N>,
    is_identity: bool,
}

/* Shared structure of curve affine points */

#[derive(Clone, Debug)]
pub struct AssignedPoint<N: FieldExt> {
    x: AssignedInteger<N>,
    y: AssignedInteger<N>,
    // indicate whether the poinit is the identity point of curve or not
    z: AssignedCondition<N>,
}

impl<F: FieldExt> AssignedPoint<F> {
    pub fn new(x: AssignedInteger<F>, y: AssignedInteger<F>, z: AssignedCondition<F>) -> AssignedPoint<F> {
        AssignedPoint { x, y, z }
    }

    pub fn is_identity(&self) -> AssignedCondition<F> {
        self.z.clone()
    }
}

mod base_field_ecc;
mod general_ecc;

#[derive(Clone, Debug)]
pub struct EccConfig {
    range_config: RangeConfig,
    main_gate_config: MainGateConfig,
}

impl EccConfig {
    fn integer_chip_config(&self) -> IntegerConfig {
        IntegerConfig::new(self.range_config.clone(), self.main_gate_config.clone())
    }
}
