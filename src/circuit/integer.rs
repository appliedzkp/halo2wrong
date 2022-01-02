use super::main_gate::MainGate;
use super::{AssignedCondition, AssignedInteger, AssignedValue, UnassignedInteger};
use crate::circuit::main_gate::{MainGateConfig, MainGateInstructions};
use crate::circuit::range::{RangeChip, RangeConfig};
use crate::circuit::AssignedLimb;
use crate::rns::{Common, Integer, Rns};
use crate::{NUMBER_OF_LIMBS, NUMBER_OF_LOOKUP_LIMBS};
use halo2::arithmetic::FieldExt;
use halo2::circuit::Region;
use halo2::plonk::Error;

mod add;
mod assert_in_field;
mod assert_not_zero;
mod assert_zero;
mod assign;
mod div;
mod invert;
mod mul;
mod neg;
mod reduce;
mod square;
mod sub;

pub enum Range {
    Remainder,
    Operand,
    MulQuotient,
}

#[derive(Clone, Debug)]
pub struct IntegerConfig {
    range_config: RangeConfig,
    main_gate_config: MainGateConfig,
}

impl IntegerConfig {
    pub fn new(range_config: RangeConfig, main_gate_config: MainGateConfig) -> Self {
        Self {
            range_config,
            main_gate_config,
        }
    }
}

pub struct IntegerChip<Wrong: FieldExt, Native: FieldExt> {
    config: IntegerConfig,
    pub rns: Rns<Wrong, Native>,
}

impl<W: FieldExt, N: FieldExt> IntegerChip<W, N> {
    pub(crate) fn new_assigned_integer(&self, limbs: Vec<AssignedLimb<N>>, native_value: AssignedValue<N>) -> AssignedInteger<N> {
        AssignedInteger::new(limbs, native_value, self.rns.bit_len_limb)
    }
}

pub trait IntegerInstructions<N: FieldExt> {
    fn assign_integer(&self, region: &mut Region<'_, N>, integer: Option<Integer<N>>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;
    fn range_assign_integer(
        &self,
        region: &mut Region<'_, N>,
        integer: UnassignedInteger<N>,
        range: Range,
        offset: &mut usize,
    ) -> Result<AssignedInteger<N>, Error>;

    fn add(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;
    fn add_constant(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &Integer<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;
    fn mul2(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;
    fn mul3(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;
    fn sub(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;
    fn neg(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;
    fn mul(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;
    fn mul_constant(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &Integer<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;
    fn mul_into_one(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error>;

    fn square(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;

    fn div(
        &self,
        region: &mut Region<'_, N>,
        a: &AssignedInteger<N>,
        b: &AssignedInteger<N>,
        offset: &mut usize,
    ) -> Result<(AssignedInteger<N>, AssignedCondition<N>), Error>;
    fn div_incomplete(
        &self,
        region: &mut Region<'_, N>,
        a: &AssignedInteger<N>,
        b: &AssignedInteger<N>,
        offset: &mut usize,
    ) -> Result<AssignedInteger<N>, Error>;

    fn invert(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<(AssignedInteger<N>, AssignedCondition<N>), Error>;
    fn invert_incomplete(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;

    fn reduce(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error>;

    fn assert_equal(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error>;
    fn assert_strict_equal(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error>;
    fn assert_not_equal(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error>;
    fn assert_not_zero(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error>;
    fn assert_zero(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error>;

    fn assert_strict_one(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error>;
    fn assert_strict_bit(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error>;

    fn assert_in_field(&self, region: &mut Region<'_, N>, input: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error>;

    fn cond_select(
        &self,
        region: &mut Region<'_, N>,
        a: &AssignedInteger<N>,
        b: &AssignedInteger<N>,
        cond: &AssignedCondition<N>,
        offset: &mut usize,
    ) -> Result<AssignedInteger<N>, Error>;

    fn cond_select_or_assign(
        &self,
        region: &mut Region<'_, N>,
        a: &AssignedInteger<N>,
        b: &Integer<N>,
        cond: &AssignedCondition<N>,
        offset: &mut usize,
    ) -> Result<AssignedInteger<N>, Error>;
}

impl<W: FieldExt, N: FieldExt> IntegerInstructions<N> for IntegerChip<W, N> {
    fn add(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        let (a, b) = (
            &self.reduce_if_limb_values_exceeds_unreduced(region, a, offset)?,
            &self.reduce_if_limb_values_exceeds_unreduced(region, b, offset)?,
        );
        self._add(region, a, b, offset)
    }

    fn add_constant(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &Integer<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        let a = &self.reduce_if_limb_values_exceeds_unreduced(region, a, offset)?;
        self._add_constant(region, a, b, offset)
    }

    fn mul2(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        self._mul2(region, a, offset)
    }

    fn mul3(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        self._mul3(region, a, offset)
    }

    fn sub(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        let (a, b) = (
            &self.reduce_if_limb_values_exceeds_unreduced(region, a, offset)?,
            &self.reduce_if_limb_values_exceeds_unreduced(region, b, offset)?,
        );
        self._sub(region, a, b, offset)
    }

    fn neg(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        let a = &self.reduce_if_limb_values_exceeds_unreduced(region, a, offset)?;
        self._neg(region, a, offset)
    }

    fn mul(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        let (a, b) = (
            &self.reduce_if_limb_values_exceeds_reduced(region, a, offset)?,
            &self.reduce_if_limb_values_exceeds_reduced(region, b, offset)?,
        );
        let (a, b) = (
            &self.reduce_if_max_operand_value_exceeds(region, a, offset)?,
            &self.reduce_if_max_operand_value_exceeds(region, b, offset)?,
        );
        self._mul(region, a, b, offset)
    }

    fn mul_constant(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &Integer<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        let a = &self.reduce_if_limb_values_exceeds_reduced(region, a, offset)?;
        let a = &self.reduce_if_max_operand_value_exceeds(region, a, offset)?;
        self._mul_constant(region, a, b, offset)
    }

    fn mul_into_one(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error> {
        let (a, b) = (
            &self.reduce_if_limb_values_exceeds_reduced(region, a, offset)?,
            &self.reduce_if_limb_values_exceeds_reduced(region, b, offset)?,
        );
        let (a, b) = (
            &self.reduce_if_max_operand_value_exceeds(region, a, offset)?,
            &self.reduce_if_max_operand_value_exceeds(region, b, offset)?,
        );
        self._mul_into_one(region, a, b, offset)
    }

    fn square(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        let a = &self.reduce_if_limb_values_exceeds_reduced(region, a, offset)?;
        let a = &self.reduce_if_max_operand_value_exceeds(region, a, offset)?;
        self._square(region, a, offset)
    }

    fn div(
        &self,
        region: &mut Region<'_, N>,
        a: &AssignedInteger<N>,
        b: &AssignedInteger<N>,
        offset: &mut usize,
    ) -> Result<(AssignedInteger<N>, AssignedCondition<N>), Error> {
        let (a, b) = (
            &self.reduce_if_limb_values_exceeds_reduced(region, a, offset)?,
            &self.reduce_if_limb_values_exceeds_reduced(region, b, offset)?,
        );
        let (a, b) = (
            &self.reduce_if_max_operand_value_exceeds(region, a, offset)?,
            &self.reduce_if_max_operand_value_exceeds(region, b, offset)?,
        );
        self._div(region, a, b, offset)
    }

    fn div_incomplete(
        &self,
        region: &mut Region<'_, N>,
        a: &AssignedInteger<N>,
        b: &AssignedInteger<N>,
        offset: &mut usize,
    ) -> Result<AssignedInteger<N>, Error> {
        let (a, b) = (
            &self.reduce_if_limb_values_exceeds_reduced(region, a, offset)?,
            &self.reduce_if_limb_values_exceeds_reduced(region, b, offset)?,
        );
        let (a, b) = (
            &self.reduce_if_max_operand_value_exceeds(region, a, offset)?,
            &self.reduce_if_max_operand_value_exceeds(region, b, offset)?,
        );
        self._div_incomplete(region, a, b, offset)
    }

    fn invert(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<(AssignedInteger<N>, AssignedCondition<N>), Error> {
        let a = &self.reduce_if_limb_values_exceeds_reduced(region, a, offset)?;
        let a = &self.reduce_if_max_operand_value_exceeds(region, a, offset)?;
        self._invert(region, a, offset)
    }

    fn invert_incomplete(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        let a = &self.reduce_if_limb_values_exceeds_reduced(region, a, offset)?;
        let a = &self.reduce_if_max_operand_value_exceeds(region, a, offset)?;
        self._invert_incomplete(region, a, offset)
    }

    fn reduce(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        self._reduce(region, a, offset)
    }

    fn range_assign_integer(
        &self,
        region: &mut Region<'_, N>,
        integer: UnassignedInteger<N>,
        range: Range,
        offset: &mut usize,
    ) -> Result<AssignedInteger<N>, Error> {
        self._range_assign_integer(region, integer, range, offset)
    }

    fn assign_integer(&self, region: &mut Region<'_, N>, integer: Option<Integer<N>>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
        self._assign_integer(region, integer, offset, true)
    }

    fn assert_equal(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error> {
        let c = &self.sub(region, a, b, offset)?;
        self.assert_zero(region, c, offset)?;
        Ok(())
    }

    fn assert_strict_equal(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error> {
        let main_gate = self.main_gate();
        for idx in 0..NUMBER_OF_LIMBS {
            main_gate.assert_equal(region, a.limb(idx), b.limb(idx), offset)?;
        }
        Ok(())
    }

    fn assert_not_equal(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, b: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error> {
        let c = &self.sub(region, a, b, offset)?;
        self.assert_not_zero(region, c, offset)?;
        Ok(())
    }

    fn assert_not_zero(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error> {
        let a = &self.reduce_if_limb_values_exceeds_reduced(region, a, offset)?;
        let a = &self.reduce_if_max_operand_value_exceeds(region, a, offset)?;
        self._assert_not_zero(region, a, offset)?;
        Ok(())
    }

    fn assert_zero(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error> {
        let a = &self.reduce_if_max_operand_value_exceeds(region, a, offset)?;
        self._assert_zero(region, a, offset)
    }

    fn assert_strict_one(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error> {
        let main_gate = self.main_gate();
        for i in 1..NUMBER_OF_LIMBS {
            main_gate.assert_zero(region, a.limb(i), offset)?;
        }
        main_gate.assert_one(region, a.limb(0), offset)
    }

    fn assert_strict_bit(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error> {
        let main_gate = self.main_gate();
        for i in 1..NUMBER_OF_LIMBS {
            main_gate.assert_zero(region, a.limb(i), offset)?;
        }
        main_gate.assert_bit(region, a.limb(0), offset)
    }

    fn cond_select(
        &self,
        region: &mut Region<'_, N>,
        a: &AssignedInteger<N>,
        b: &AssignedInteger<N>,
        cond: &AssignedCondition<N>,
        offset: &mut usize,
    ) -> Result<AssignedInteger<N>, Error> {
        let main_gate = self.main_gate();

        let mut limbs: Vec<AssignedLimb<N>> = Vec::with_capacity(NUMBER_OF_LIMBS);
        for i in 0..NUMBER_OF_LIMBS {
            let res = main_gate.cond_select(region, &a.limb(i), &b.limb(i), cond, offset)?;

            let max_val = if a.limbs[i].max_val > b.limbs[i].max_val {
                a.limbs[i].max_val.clone()
            } else {
                b.limbs[i].max_val.clone()
            };

            limbs.push(res.to_limb(max_val));
        }

        let native_value = main_gate.cond_select(region, &a.native(), &b.native(), cond, offset)?;

        Ok(self.new_assigned_integer(limbs, native_value))
    }

    fn cond_select_or_assign(
        &self,
        region: &mut Region<'_, N>,
        a: &AssignedInteger<N>,
        b: &Integer<N>,
        cond: &AssignedCondition<N>,
        offset: &mut usize,
    ) -> Result<AssignedInteger<N>, Error> {
        let main_gate = self.main_gate();

        let mut limbs: Vec<AssignedLimb<N>> = Vec::with_capacity(NUMBER_OF_LIMBS);
        for i in 0..NUMBER_OF_LIMBS {
            let b_limb = b.limb(i);

            let res = main_gate.cond_select_or_assign(region, a.limb(i), b_limb.fe(), cond, offset)?;

            // here we assume given constant is always in field
            let max_val = a.limb(i).max_val();
            limbs.push(res.to_limb(max_val));
        }

        let native_value = main_gate.cond_select_or_assign(region, a.native(), b.native(), cond, offset)?;

        Ok(self.new_assigned_integer(limbs, native_value))
    }

    fn assert_in_field(&self, region: &mut Region<'_, N>, a: &AssignedInteger<N>, offset: &mut usize) -> Result<(), Error> {
        let a = &self.reduce_if_limb_values_exceeds_reduced(region, a, offset)?;
        let a = &self.reduce_if_max_operand_value_exceeds(region, a, offset)?;
        self._assert_in_field(region, a, offset)
    }
}

impl<W: FieldExt, N: FieldExt> IntegerChip<W, N> {
    pub fn new(config: IntegerConfig, rns: Rns<W, N>) -> Self {
        IntegerChip { config, rns }
    }

    fn range_chip(&self) -> RangeChip<N> {
        let bit_len_lookup = self.rns.bit_len_limb / NUMBER_OF_LOOKUP_LIMBS;
        RangeChip::<N>::new(self.config.range_config.clone(), bit_len_lookup)
    }

    fn main_gate(&self) -> MainGate<N> {
        let main_gate_config = self.config.main_gate_config.clone();
        MainGate::<N>::new(main_gate_config)
    }
}

#[cfg(test)]
mod tests {
    use super::{IntegerChip, IntegerConfig, IntegerInstructions, Range};
    use crate::circuit::main_gate::{MainGate, MainGateColumn, MainGateConfig, MainGateInstructions};
    use crate::circuit::range::{RangeChip, RangeConfig, RangeInstructions};
    use crate::circuit::{AssignedCondition, AssignedInteger, UnassignedValue};
    use crate::rns::{fe_to_big, Common, Integer, Limb, Rns};
    use crate::NUMBER_OF_LOOKUP_LIMBS;
    use halo2::arithmetic::FieldExt;
    use halo2::circuit::{Layouter, Region, SimpleFloorPlanner};
    use halo2::dev::MockProver;
    use halo2::plonk::{Circuit, ConstraintSystem, Error};

    use halo2::pasta::Fp as Wrong;
    use halo2::pasta::Fq as Native;

    impl<W: FieldExt, N: FieldExt> IntegerChip<W, N> {
        fn assign_integer_no_check(&self, region: &mut Region<'_, N>, integer: Option<Integer<N>>, offset: &mut usize) -> Result<AssignedInteger<N>, Error> {
            self._assign_integer(region, integer, offset, false)
        }
    }
    const BIT_LEN_LIMB: usize = 68;

    fn rns<W: FieldExt, N: FieldExt>() -> Rns<W, N> {
        Rns::<W, N>::construct(BIT_LEN_LIMB)
    }

    fn setup<W: FieldExt, N: FieldExt>() -> (Rns<W, N>, u32) {
        let rns = rns();
        #[cfg(not(feature = "no_lookup"))]
        let k: u32 = (rns.bit_len_lookup + 1) as u32;
        #[cfg(feature = "no_lookup")]
        let k: u32 = 14;
        (rns, k)
    }

    #[derive(Clone, Debug)]
    struct TestCircuitConfig {
        range_config: RangeConfig,
        main_gate_config: MainGateConfig,
    }

    impl TestCircuitConfig {
        fn new<W: FieldExt, N: FieldExt>(meta: &mut ConstraintSystem<N>) -> Self {
            let main_gate_config = MainGate::<N>::configure(meta);

            let overflow_bit_lengths = rns::<W, N>().overflow_lengths();
            let range_config = RangeChip::<N>::configure(meta, &main_gate_config, overflow_bit_lengths);

            TestCircuitConfig {
                range_config,
                main_gate_config,
            }
        }

        fn integer_chip_config(&self) -> IntegerConfig {
            IntegerConfig {
                range_config: self.range_config.clone(),
                main_gate_config: self.main_gate_config.clone(),
            }
        }

        fn config_range<N: FieldExt>(&self, layouter: &mut impl Layouter<N>) -> Result<(), Error> {
            let bit_len_lookup = BIT_LEN_LIMB / NUMBER_OF_LOOKUP_LIMBS;
            let range_chip = RangeChip::<N>::new(self.range_config.clone(), bit_len_lookup);
            #[cfg(not(feature = "no_lookup"))]
            range_chip.load_limb_range_table(layouter)?;
            #[cfg(not(feature = "no_lookup"))]
            range_chip.load_overflow_range_tables(layouter)?;

            Ok(())
        }
    }

    #[derive(Default, Clone, Debug)]
    struct TestCircuitRange<W: FieldExt, N: FieldExt> {
        rns: Rns<W, N>,
    }

    impl<W: FieldExt, N: FieldExt> Circuit<N> for TestCircuitRange<W, N> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<N>) -> Self::Config {
            TestCircuitConfig::new::<W, N>(meta)
        }

        fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<N>) -> Result<(), Error> {
            let integer_chip = IntegerChip::<W, N>::new(config.integer_chip_config(), self.rns.clone());
            let rns = self.rns.clone();

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;

                    let a = rns.new_from_big(rns.max_remainder.clone());
                    integer_chip.range_assign_integer(&mut region, Some(a).into(), Range::Remainder, offset)?;

                    // should fail
                    // let a = rns.new_from_big(rns.max_remainder.clone() + 1usize);
                    // integer_chip.range_assign_integer(&mut region, Some(a).into(), Range::Remainder, offset)?;

                    let a = rns.new_from_big(rns.max_operand.clone());
                    integer_chip.range_assign_integer(&mut region, Some(a).into(), Range::Operand, offset)?;

                    // should fail
                    // let a = rns.new_from_big(rns.max_operand.clone() + 1usize);
                    // integer_chip.range_assign_integer(&mut region, Some(a).into(), Range::Operand, offset)?;

                    Ok(())
                },
            )?;

            config.config_range(&mut layouter)?;

            Ok(())
        }
    }

    #[test]
    fn test_xxx() {
        let (rns, k) = setup();
        let circuit = TestCircuitRange::<Wrong, Native> { rns };
        let prover = match MockProver::run(k, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }

    #[derive(Default, Clone, Debug)]
    struct TestCircuitReduction<W: FieldExt, N: FieldExt> {
        rns: Rns<W, N>,
    }

    impl<W: FieldExt, N: FieldExt> Circuit<N> for TestCircuitReduction<W, N> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<N>) -> Self::Config {
            TestCircuitConfig::new::<W, N>(meta)
        }

        fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<N>) -> Result<(), Error> {
            let integer_chip = IntegerChip::<W, N>::new(config.integer_chip_config(), self.rns.clone());
            let rns = self.rns.clone();

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;

                    let overflows = rns.rand_with_limb_bit_size(rns.bit_len_limb + 5);
                    let reduced = rns.reduce(&overflows).result;

                    let overflows = &integer_chip.assign_integer_no_check(&mut region, Some(overflows), offset)?;
                    let reduced_0 = &integer_chip.range_assign_integer(&mut region, Some(reduced).into(), Range::Remainder, offset)?;
                    let reduced_1 = &integer_chip.reduce(&mut region, overflows, offset)?;
                    assert_eq!(reduced_1.max_val(), rns.max_remainder);

                    integer_chip.assert_equal(&mut region, reduced_0, reduced_1, offset)?;
                    integer_chip.assert_strict_equal(&mut region, reduced_0, reduced_1, offset)?;

                    Ok(())
                },
            )?;

            config.config_range(&mut layouter)?;

            Ok(())
        }
    }

    #[test]
    fn test_reduction_circuit() {
        let (rns, k) = setup();
        let circuit = TestCircuitReduction::<Wrong, Native> { rns };
        let prover = match MockProver::run(k, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }

    #[derive(Default, Clone, Debug)]
    struct TestCircuitEquality<W: FieldExt, N: FieldExt> {
        rns: Rns<W, N>,
    }

    impl<W: FieldExt, N: FieldExt> Circuit<N> for TestCircuitEquality<W, N> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<N>) -> Self::Config {
            TestCircuitConfig::new::<W, N>(meta)
        }

        fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<N>) -> Result<(), Error> {
            let integer_chip = IntegerChip::<W, N>::new(config.integer_chip_config(), self.rns.clone());
            let rns = self.rns.clone();

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;

                    let a = rns.rand_in_operand_range();
                    let b = rns.rand_in_operand_range();
                    let a = &integer_chip.range_assign_integer(&mut region, Some(a).into(), Range::Operand, offset)?.clone();
                    let b = &integer_chip.range_assign_integer(&mut region, Some(b).into(), Range::Operand, offset)?.clone();
                    integer_chip.assert_not_equal(&mut region, a, b, offset)?;
                    integer_chip.assert_equal(&mut region, a, a, offset)?;
                    integer_chip.assert_not_zero(&mut region, a, offset)?;

                    Ok(())
                },
            )?;

            config.config_range(&mut layouter)?;

            Ok(())
        }
    }

    #[test]
    fn test_equality_circuit() {
        let (rns, k) = setup();
        let circuit = TestCircuitReduction::<Wrong, Native> { rns };
        let prover = match MockProver::run(k, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }

    #[derive(Default, Clone, Debug)]
    struct TestCircuitMultiplication<W: FieldExt, N: FieldExt> {
        rns: Rns<W, N>,
    }

    impl<W: FieldExt, N: FieldExt> Circuit<N> for TestCircuitMultiplication<W, N> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<N>) -> Self::Config {
            TestCircuitConfig::new::<W, N>(meta)
        }

        fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<N>) -> Result<(), Error> {
            let integer_chip = IntegerChip::<W, N>::new(config.integer_chip_config(), self.rns.clone());
            let rns = self.rns.clone();

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;

                    let a = rns.rand_in_operand_range();
                    let b = rns.rand_in_operand_range();
                    let c = (a.value() * b.value()) % &rns.wrong_modulus;
                    let c = rns.new_from_big(c);

                    let a = &integer_chip.range_assign_integer(&mut region, Some(a).into(), Range::Operand, offset)?.clone();
                    let b = &integer_chip.range_assign_integer(&mut region, Some(b).into(), Range::Operand, offset)?.clone();
                    let c_0 = &integer_chip.range_assign_integer(&mut region, Some(c).into(), Range::Remainder, offset)?;
                    let c_1 = &integer_chip.mul(&mut region, a, b, offset)?;
                    assert_eq!(c_1.max_val(), rns.max_remainder);

                    integer_chip.assert_equal(&mut region, c_0, c_1, offset)?;
                    integer_chip.assert_strict_equal(&mut region, c_0, c_1, offset)?;

                    let a = rns.rand_in_unreduced_range();
                    let b = rns.rand_in_unreduced_range();
                    let c = (a.value() * b.value()) % &rns.wrong_modulus;
                    let c = rns.new_from_big(c);

                    let a = &integer_chip.assign_integer_no_check(&mut region, Some(a).into(), offset)?;
                    let b = &integer_chip.assign_integer_no_check(&mut region, Some(b).into(), offset)?;
                    let c_0 = &integer_chip.range_assign_integer(&mut region, Some(c).into(), Range::Remainder, offset)?;
                    let c_1 = &integer_chip.mul(&mut region, a, b, offset)?;
                    assert_eq!(c_1.max_val(), rns.max_remainder);

                    integer_chip.assert_equal(&mut region, c_0, c_1, offset)?;
                    integer_chip.assert_strict_equal(&mut region, c_0, c_1, offset)?;

                    let a = rns.rand_in_unreduced_range();
                    let b = rns.rand_in_field();
                    let c = (a.value() * b.value()) % &rns.wrong_modulus;
                    let c = rns.new_from_big(c);

                    let a = &integer_chip.assign_integer_no_check(&mut region, Some(a).into(), offset)?;
                    let c_0 = &integer_chip.range_assign_integer(&mut region, Some(c).into(), Range::Remainder, offset)?;
                    let c_1 = &integer_chip.mul_constant(&mut region, a, &b, offset)?;
                    assert_eq!(c_1.max_val(), rns.max_remainder);

                    integer_chip.assert_equal(&mut region, c_0, c_1, offset)?;
                    integer_chip.assert_strict_equal(&mut region, c_0, c_1, offset)?;

                    let a = W::rand();
                    let inv = a.invert().unwrap();

                    // will fail
                    // let inv = W::rand();

                    let a = fe_to_big(a);
                    let inv = fe_to_big(inv);
                    let a = rns.new_from_big(a);
                    let inv = rns.new_from_big(inv);

                    let a = &integer_chip.range_assign_integer(&mut region, Some(a).into(), Range::Remainder, offset)?;
                    let inv = &integer_chip.range_assign_integer(&mut region, Some(inv).into(), Range::Remainder, offset)?;
                    integer_chip.mul_into_one(&mut region, a, inv, offset)?;

                    Ok(())
                },
            )?;

            config.config_range(&mut layouter)?;

            Ok(())
        }
    }

    #[test]
    fn test_multiplication_circuit() {
        let (rns, k) = setup();
        let circuit = TestCircuitMultiplication::<Wrong, Native> { rns };
        let prover = match MockProver::run(k, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }

    #[derive(Default, Clone, Debug)]
    struct TestCircuitSquaring<W: FieldExt, N: FieldExt> {
        rns: Rns<W, N>,
    }

    impl<W: FieldExt, N: FieldExt> Circuit<N> for TestCircuitSquaring<W, N> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<N>) -> Self::Config {
            TestCircuitConfig::new::<W, N>(meta)
        }

        fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<N>) -> Result<(), Error> {
            let integer_chip = IntegerChip::<W, N>::new(config.integer_chip_config(), self.rns.clone());
            let rns = self.rns.clone();

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;

                    let a = rns.rand_in_operand_range();
                    let c = (a.value() * a.value()) % &rns.wrong_modulus;
                    let c = rns.new_from_big(c);

                    let a = &integer_chip.range_assign_integer(&mut region, Some(a).into(), Range::Operand, offset)?;
                    let c_0 = &integer_chip.range_assign_integer(&mut region, Some(c).into(), Range::Remainder, offset)?;
                    let c_1 = &integer_chip.square(&mut region, a, offset)?;
                    assert_eq!(c_1.max_val(), rns.max_remainder);

                    integer_chip.assert_equal(&mut region, c_0, c_1, offset)?;
                    integer_chip.assert_strict_equal(&mut region, c_0, c_1, offset)?;

                    let a = rns.rand_in_unreduced_range();
                    let c = (a.value() * a.value()) % &rns.wrong_modulus;
                    let c = rns.new_from_big(c);

                    let a = &integer_chip.assign_integer_no_check(&mut region, Some(a).into(), offset)?;
                    let c_0 = &integer_chip.range_assign_integer(&mut region, Some(c).into(), Range::Remainder, offset)?;
                    let c_1 = &integer_chip.square(&mut region, a, offset)?;
                    assert_eq!(c_1.max_val(), rns.max_remainder);

                    integer_chip.assert_equal(&mut region, c_0, c_1, offset)?;
                    integer_chip.assert_strict_equal(&mut region, c_0, c_1, offset)?;

                    Ok(())
                },
            )?;

            config.config_range(&mut layouter)?;

            Ok(())
        }
    }

    #[test]
    fn test_squaring_circuit() {
        let (rns, k) = setup();
        let circuit = TestCircuitSquaring::<Wrong, Native> { rns };
        let prover = match MockProver::run(k, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }

    #[derive(Default, Clone, Debug)]
    struct TestCircuitInField<W: FieldExt, N: FieldExt> {
        rns: Rns<W, N>,
    }

    impl<W: FieldExt, N: FieldExt> Circuit<N> for TestCircuitInField<W, N> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<N>) -> Self::Config {
            TestCircuitConfig::new::<W, N>(meta)
        }

        fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<N>) -> Result<(), Error> {
            let integer_chip = IntegerChip::<W, N>::new(config.integer_chip_config(), self.rns.clone());
            let rns = self.rns.clone();

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;
                    let a = rns.rand_in_field();
                    let a = &integer_chip.range_assign_integer(&mut region, Some(a.clone()).into(), Range::Remainder, offset)?;
                    integer_chip.assert_in_field(&mut region, a, offset)?;

                    // must fail
                    // let a = rns.new_from_big(rns.wrong_modulus.clone());
                    // let a = &integer_chip.range_assign_integer(&mut region, Some(a.clone()).into(), Range::Remainder, offset)?;
                    // integer_chip.assert_in_field(&mut region, a, offset)?;

                    Ok(())
                },
            )?;

            config.config_range(&mut layouter)?;

            Ok(())
        }
    }

    #[test]
    fn test_assert_in_field_circuit() {
        let (rns, k) = setup();
        let circuit = TestCircuitInField::<Wrong, Native> { rns };
        let prover = match MockProver::run(k, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }

    #[derive(Default, Clone, Debug)]
    struct TestCircuitNonDeterministic<W: FieldExt, N: FieldExt> {
        rns: Rns<W, N>,
    }

    impl<W: FieldExt, N: FieldExt> Circuit<N> for TestCircuitNonDeterministic<W, N> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<N>) -> Self::Config {
            TestCircuitConfig::new::<W, N>(meta)
        }

        fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<N>) -> Result<(), Error> {
            let integer_chip = IntegerChip::<W, N>::new(config.integer_chip_config(), self.rns.clone());
            let main_gate = MainGate::<N>::new(config.main_gate_config.clone());
            let rns = self.rns.clone();

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;
                    let a = rns.rand_in_remainder_range();
                    let inv = rns.invert(&a).unwrap();

                    // 1 / a
                    let a = &integer_chip.range_assign_integer(&mut region, Some(a.clone()).into(), Range::Remainder, offset)?;
                    let inv_0 = &integer_chip.range_assign_integer(&mut region, Some(inv.clone()).into(), Range::Remainder, offset)?;
                    let (inv_1, cond) = integer_chip.invert(&mut region, a, offset)?;
                    integer_chip.assert_equal(&mut region, inv_0, &inv_1, offset)?;
                    main_gate.assert_zero(&mut region, cond, offset)?;

                    // 1 / 0
                    let zero = integer_chip.assign_integer(&mut region, Some(rns.zero()), offset)?;
                    let (must_be_one, cond) = integer_chip.invert(&mut region, &zero, offset)?;
                    integer_chip.assert_strict_one(&mut region, &must_be_one, offset)?;
                    main_gate.assert_one(&mut region, cond, offset)?;

                    // 1 / p
                    let wrong_modulus = rns.new_from_limbs(rns.wrong_modulus_decomposed.clone());
                    let modulus = integer_chip.assign_integer(&mut region, Some(wrong_modulus), offset)?;
                    let (must_be_one, cond) = integer_chip.invert(&mut region, &modulus, offset)?;
                    integer_chip.assert_strict_one(&mut region, &must_be_one, offset)?;
                    main_gate.assert_one(&mut region, cond, offset)?;

                    // 1 / a
                    let inv_1 = integer_chip.invert_incomplete(&mut region, a, offset)?;
                    integer_chip.assert_equal(&mut region, inv_0, &inv_1, offset)?;

                    // must be failing
                    // integer_chip.invert_incomplete(&mut region, &zero, offset)?;

                    // a / b
                    let a = rns.rand_in_remainder_range();
                    let b = rns.rand_in_remainder_range();
                    let c = rns.div(&a, &b).unwrap();
                    let a = &integer_chip.range_assign_integer(&mut region, Some(a.clone()).into(), Range::Remainder, offset)?;
                    let b = &integer_chip.range_assign_integer(&mut region, Some(b.clone()).into(), Range::Remainder, offset)?;
                    let c_0 = &integer_chip.range_assign_integer(&mut region, Some(c.clone()).into(), Range::Remainder, offset)?;
                    let (c_1, cond) = integer_chip.div(&mut region, a, b, offset)?;
                    integer_chip.assert_equal(&mut region, c_0, &c_1, offset)?;
                    main_gate.assert_zero(&mut region, cond, offset)?;

                    // 0 / b
                    let (c_1, cond) = integer_chip.div(&mut region, &zero, b, offset)?;
                    integer_chip.assert_zero(&mut region, &c_1, offset)?;
                    main_gate.assert_zero(&mut region, cond, offset)?;

                    // p / b
                    let (c_1, cond) = integer_chip.div(&mut region, &modulus, b, offset)?;
                    integer_chip.assert_zero(&mut region, &c_1, offset)?;
                    main_gate.assert_zero(&mut region, cond, offset)?;

                    // a / 0
                    let (must_be_self, cond) = integer_chip.div(&mut region, a, &zero, offset)?;
                    integer_chip.assert_equal(&mut region, &must_be_self, a, offset)?;
                    main_gate.assert_one(&mut region, cond, offset)?;

                    // a / p
                    let (must_be_self, cond) = integer_chip.div(&mut region, a, &modulus, offset)?;
                    integer_chip.assert_equal(&mut region, &must_be_self, a, offset)?;
                    main_gate.assert_one(&mut region, cond, offset)?;

                    // a / b
                    let c_1 = integer_chip.div_incomplete(&mut region, a, b, offset)?;
                    integer_chip.assert_equal(&mut region, c_0, &c_1, offset)?;

                    // must be failing
                    // integer_chip.div_incomplete(&mut region, a, &zero, offset)?;

                    Ok(())
                },
            )?;

            config.config_range(&mut layouter)?;

            Ok(())
        }
    }

    #[test]
    fn test_non_deterministic_circuit() {
        let (rns, k) = setup();
        let circuit = TestCircuitNonDeterministic::<Wrong, Native> { rns };

        let prover = match MockProver::run(k, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }

    #[derive(Default, Clone, Debug)]
    struct TestCircuitAssertNotZero<W: FieldExt, N: FieldExt> {
        integer_a: Option<Integer<N>>,
        rns: Rns<W, N>,
    }

    impl<W: FieldExt, N: FieldExt> Circuit<N> for TestCircuitAssertNotZero<W, N> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<N>) -> Self::Config {
            TestCircuitConfig::new::<W, N>(meta)
        }

        fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<N>) -> Result<(), Error> {
            let integer_chip = IntegerChip::<W, N>::new(config.integer_chip_config(), self.rns.clone());

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;
                    let integer_a_0 = &integer_chip.assign_integer(&mut region, self.integer_a.clone(), offset)?.clone();
                    integer_chip.assert_not_zero(&mut region, integer_a_0, offset)?;

                    Ok(())
                },
            )?;

            config.config_range(&mut layouter)?;

            Ok(())
        }
    }

    #[derive(Default, Clone, Debug)]
    struct TestCircuitAddition<W: FieldExt, N: FieldExt> {
        rns: Rns<W, N>,
    }

    impl<W: FieldExt, N: FieldExt> Circuit<N> for TestCircuitAddition<W, N> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<N>) -> Self::Config {
            TestCircuitConfig::new::<W, N>(meta)
        }

        fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<N>) -> Result<(), Error> {
            let integer_chip = IntegerChip::<W, N>::new(config.integer_chip_config(), self.rns.clone());
            let rns = self.rns.clone();

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;

                    {
                        // addition in remainder range
                        let a = rns.rand_in_remainder_range();
                        let b = rns.rand_in_remainder_range();

                        let c = a.value() + b.value();
                        let c = rns.new_from_big(c);
                        let c_in_field = (a.value() + b.value()) % &self.rns.wrong_modulus;
                        let c_in_field = rns.new_from_big(c_in_field);

                        let a = integer_chip.range_assign_integer(&mut region, Some(a.clone()).into(), Range::Remainder, offset)?;
                        let b = integer_chip.range_assign_integer(&mut region, Some(b.clone()).into(), Range::Remainder, offset)?;

                        let c_0 = &integer_chip.add(&mut region, &a, &b, offset)?;
                        let c_1 = integer_chip.assign_integer_no_check(&mut region, Some(c).into(), offset)?;
                        assert_eq!(a.max_val() + b.max_val(), c_0.max_val());
                        integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;

                        // reduce and enfoce strict equality
                        let c_0 = integer_chip.reduce(&mut region, c_0, offset)?;
                        let c_1 = integer_chip.range_assign_integer(&mut region, Some(c_in_field).into(), Range::Remainder, offset)?;
                        integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;
                        integer_chip.assert_strict_equal(&mut region, &c_0, &c_1, offset)?;
                    }

                    {
                        // constant addition in remainder range
                        let a = rns.rand_in_remainder_range();
                        let b = rns.rand_in_field();

                        let c = a.value() + b.value();
                        let c = rns.new_from_big(c);
                        let c_in_field = (a.value() + b.value()) % &self.rns.wrong_modulus;
                        let c_in_field = rns.new_from_big(c_in_field);

                        let a = integer_chip.range_assign_integer(&mut region, Some(a.clone()).into(), Range::Remainder, offset)?;

                        let c_0 = &integer_chip.add_constant(&mut region, &a, &b, offset)?;
                        let c_1 = integer_chip.assign_integer_no_check(&mut region, Some(c).into(), offset)?;
                        assert_eq!(a.max_val() + b.value(), c_0.max_val());
                        integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;

                        // reduce and enfoce strict equality
                        let c_0 = integer_chip.reduce(&mut region, c_0, offset)?;
                        let c_1 = integer_chip.range_assign_integer(&mut region, Some(c_in_field).into(), Range::Remainder, offset)?;
                        integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;
                        integer_chip.assert_strict_equal(&mut region, &c_0, &c_1, offset)?;
                    }

                    {
                        // go beyond unreduced range
                        let a = rns.rand_in_remainder_range();
                        let mut a = integer_chip.assign_integer(&mut region, Some(a.clone()).into(), offset)?;

                        for _ in 0..10 {
                            let c = (a.integer().unwrap().value() * 2usize) % &self.rns.wrong_modulus;
                            let c = rns.new_from_big(c);
                            a = integer_chip.add(&mut region, &a, &a, offset)?;
                            let c_1 = integer_chip.range_assign_integer(&mut region, Some(c).into(), Range::Remainder, offset)?;
                            let c_0 = integer_chip.reduce(&mut region, &a, offset)?;
                            integer_chip.assert_equal(&mut region, &a, &c_1, offset)?;
                            integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;
                            integer_chip.assert_strict_equal(&mut region, &c_0, &c_1, offset)?;
                        }
                    }

                    {
                        // addition in unreduced range
                        for _ in 0..10 {
                            let a = rns.rand_in_unreduced_range();
                            let b = rns.rand_in_unreduced_range();
                            let c = (a.value() + b.value()) % rns.wrong_modulus.clone();
                            let c = rns.new_from_big(c);

                            let a = integer_chip.assign_integer_no_check(&mut region, Some(a.clone()).into(), offset)?;
                            let b = integer_chip.assign_integer_no_check(&mut region, Some(b.clone()).into(), offset)?;
                            let c_0 = &integer_chip.add(&mut region, &a, &b, offset)?;
                            let c_1 = integer_chip.range_assign_integer(&mut region, Some(c).into(), Range::Remainder, offset)?;
                            assert_eq!(a.max_val() + b.max_val(), c_0.max_val());
                            integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;

                            // reduce and enfoce strict equality
                            let c_0 = integer_chip.reduce(&mut region, c_0, offset)?;
                            integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;
                            integer_chip.assert_strict_equal(&mut region, &c_0, &c_1, offset)?;
                        }
                    }

                    {
                        // subtraction in remainder range
                        let a = rns.rand_in_remainder_range();
                        let b = rns.rand_in_remainder_range();

                        let a_norm = (a.value() % rns.wrong_modulus.clone()) + rns.wrong_modulus.clone();
                        let b_norm = b.value() % rns.wrong_modulus.clone();
                        let c = a_norm - b_norm;
                        let c = rns.new_from_big(c);

                        let a = integer_chip.range_assign_integer(&mut region, Some(a.clone()).into(), Range::Remainder, offset)?;
                        let b = integer_chip.range_assign_integer(&mut region, Some(b.clone()).into(), Range::Remainder, offset)?;

                        let c_0 = &integer_chip.sub(&mut region, &a, &b, offset)?;
                        let c_1 = integer_chip.range_assign_integer(&mut region, Some(c.clone()).into(), Range::Remainder, offset)?;
                        assert_eq!(a.max_val() + rns.make_aux(b.max_vals()).value(), c_0.max_val());
                        integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;
                    }

                    {
                        // subtraction in unreduced range
                        let a = rns.rand_in_unreduced_range();
                        let b = rns.rand_in_unreduced_range();

                        let a_norm = (a.value() % rns.wrong_modulus.clone()) + rns.wrong_modulus.clone();
                        let b_norm = b.value() % rns.wrong_modulus.clone();
                        let c = a_norm - b_norm;
                        let c = rns.new_from_big(c);

                        let a = integer_chip.assign_integer_no_check(&mut region, Some(a.clone()).into(), offset)?;
                        let b = integer_chip.assign_integer_no_check(&mut region, Some(b.clone()).into(), offset)?;

                        let c_0 = &integer_chip.sub(&mut region, &a, &b, offset)?;
                        let c_1 = integer_chip.range_assign_integer(&mut region, Some(c.clone()).into(), Range::Remainder, offset)?;
                        assert_eq!(a.max_val() + rns.make_aux(b.max_vals()).value(), c_0.max_val());
                        integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;
                    }

                    {
                        // go beyond unreduced range
                        let a = rns.rand_in_remainder_range();
                        let mut a = integer_chip.assign_integer(&mut region, Some(a.clone()).into(), offset)?;

                        for _ in 0..10 {
                            let b = rns.rand_in_unreduced_range();

                            let a_norm = (a.integer().unwrap().value() % rns.wrong_modulus.clone()) + rns.wrong_modulus.clone();
                            let b_norm = b.value() % rns.wrong_modulus.clone();
                            let c = a_norm - b_norm;
                            let c = rns.new_from_big(c);

                            let b = integer_chip.assign_integer_no_check(&mut region, Some(b.clone()).into(), offset)?;

                            let c_0 = &integer_chip.sub(&mut region, &a, &b, offset)?;
                            let c_1 = integer_chip.range_assign_integer(&mut region, Some(c.clone()).into(), Range::Remainder, offset)?;
                            integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;
                            a = c_0.clone();
                        }
                    }

                    {
                        // negation in unreduced range
                        let a = rns.rand_in_unreduced_range();
                        let a_norm = a.value() % rns.wrong_modulus.clone();
                        let c = rns.wrong_modulus.clone() - a_norm;
                        let c = rns.new_from_big(c);

                        let a = integer_chip.assign_integer_no_check(&mut region, Some(a.clone()).into(), offset)?;

                        let c_0 = &integer_chip.neg(&mut region, &a, offset)?;
                        let c_1 = integer_chip.range_assign_integer(&mut region, Some(c.clone()).into(), Range::Remainder, offset)?;
                        assert_eq!(rns.make_aux(a.max_vals()).value(), c_0.max_val());
                        integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;
                    }

                    {
                        // mul2 in unreduced range
                        let a = rns.rand_in_unreduced_range();
                        let c = (a.value() * 2usize) % rns.wrong_modulus.clone();
                        let c = rns.new_from_big(c);

                        let a = integer_chip.assign_integer_no_check(&mut region, Some(a.clone()).into(), offset)?;

                        let c_0 = &integer_chip.mul2(&mut region, &a, offset)?;
                        let c_1 = integer_chip.range_assign_integer(&mut region, Some(c.clone()).into(), Range::Remainder, offset)?;
                        assert_eq!(a.max_val() * 2usize, c_0.max_val());
                        integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;
                    }

                    {
                        // mul3 in unreduced range
                        let a = rns.rand_in_unreduced_range();
                        let c = (a.value() * 3usize) % rns.wrong_modulus.clone();
                        let c = rns.new_from_big(c);

                        let a = integer_chip.assign_integer_no_check(&mut region, Some(a.clone()).into(), offset)?;
                        let c_0 = &integer_chip.mul3(&mut region, &a, offset)?;
                        let c_1 = integer_chip.range_assign_integer(&mut region, Some(c.clone()).into(), Range::Remainder, offset)?;
                        assert_eq!(a.max_val() * 3usize, c_0.max_val());
                        integer_chip.assert_equal(&mut region, &c_0, &c_1, offset)?;
                    }

                    Ok(())
                },
            )?;

            config.config_range(&mut layouter)?;

            Ok(())
        }
    }

    #[test]
    fn test_addition() {
        let (rns, k) = setup();
        let circuit = TestCircuitAddition::<Wrong, Native> { rns };
        let prover = match MockProver::run(k, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }

    #[derive(Default, Clone, Debug)]
    struct TestCircuitConditionals<W: FieldExt, N: FieldExt> {
        rns: Rns<W, N>,
    }

    impl<W: FieldExt, N: FieldExt> Circuit<N> for TestCircuitConditionals<W, N> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<N>) -> Self::Config {
            TestCircuitConfig::new::<W, N>(meta)
        }

        fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<N>) -> Result<(), Error> {
            let integer_chip = IntegerChip::<W, N>::new(config.integer_chip_config(), self.rns.clone());
            let main_gate = MainGate::<N>::new(config.main_gate_config.clone());
            let rns = self.rns.clone();

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;

                    // select second operand when condision is zero

                    let a = Some(rns.rand_in_remainder_range()).into();
                    let b = Some(rns.rand_in_remainder_range()).into();
                    let cond = N::zero();
                    let cond = Some(cond).into();

                    let a = integer_chip.range_assign_integer(&mut region, a, Range::Remainder, offset)?;
                    let b = integer_chip.range_assign_integer(&mut region, b, Range::Remainder, offset)?;

                    let cond: AssignedCondition<N> = main_gate.assign_value(&mut region, &cond, MainGateColumn::A, offset)?.into();
                    let selected = integer_chip.cond_select(&mut region, &a, &b, &cond, offset)?;
                    integer_chip.assert_equal(&mut region, &b, &selected, offset)?;
                    integer_chip.assert_strict_equal(&mut region, &b, &selected, offset)?;
                    assert_eq!(b.max_val(), selected.max_val());

                    // select first operand when condision is one

                    let a = Some(rns.rand_in_remainder_range()).into();
                    let b = Some(rns.rand_in_remainder_range()).into();
                    let cond = N::one();
                    let cond = UnassignedValue::new(Some(cond));

                    let a = integer_chip.range_assign_integer(&mut region, a, Range::Remainder, offset)?;
                    let b = integer_chip.range_assign_integer(&mut region, b, Range::Remainder, offset)?;

                    let cond: AssignedCondition<N> = main_gate.assign_value(&mut region, &cond, MainGateColumn::A, offset)?.into();
                    let selected = integer_chip.cond_select(&mut region, &a, &b, &cond, offset)?;
                    integer_chip.assert_equal(&mut region, &a, &selected, offset)?;
                    integer_chip.assert_strict_equal(&mut region, &a, &selected, offset)?;
                    assert_eq!(a.max_val(), selected.max_val());

                    // select constant operand when condision is zero

                    let a = Some(rns.rand_in_remainder_range()).into();
                    let b = rns.rand_in_remainder_range();
                    let cond = N::zero();
                    let cond = UnassignedValue::new(Some(cond));

                    let a = integer_chip.range_assign_integer(&mut region, a, Range::Remainder, offset)?;
                    let cond: AssignedCondition<N> = main_gate.assign_value(&mut region, &cond, MainGateColumn::A, offset)?.into();
                    let selected = integer_chip.cond_select_or_assign(&mut region, &a, &b, &cond, offset)?;
                    let b_assigned = integer_chip.assign_integer(&mut region, Some(b), offset)?;
                    integer_chip.assert_equal(&mut region, &b_assigned, &selected, offset)?;
                    integer_chip.assert_strict_equal(&mut region, &b_assigned, &selected, offset)?;
                    assert_eq!(a.max_val(), selected.max_val());

                    // select non constant operand when condision is zero

                    let a = Some(rns.rand_in_remainder_range()).into();
                    let b = rns.rand_in_remainder_range();
                    let cond = N::one();
                    let cond = UnassignedValue::new(Some(cond));

                    let a = integer_chip.range_assign_integer(&mut region, a, Range::Remainder, offset)?;
                    let cond: AssignedCondition<N> = main_gate.assign_value(&mut region, &cond, MainGateColumn::A, offset)?.into();
                    let selected = integer_chip.cond_select_or_assign(&mut region, &a, &b, &cond, offset)?;
                    integer_chip.assert_equal(&mut region, &a, &selected, offset)?;
                    integer_chip.assert_strict_equal(&mut region, &a, &selected, offset)?;
                    assert_eq!(a.max_val(), selected.max_val());

                    Ok(())
                },
            )?;

            config.config_range(&mut layouter)?;

            Ok(())
        }
    }

    #[test]
    fn test_condition_circuit() {
        let (rns, k) = setup();
        let circuit = TestCircuitConditionals::<Wrong, Native> { rns };

        let prover = match MockProver::run(k, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }
}
