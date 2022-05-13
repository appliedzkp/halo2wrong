use std::rc::Rc;

use super::{IntegerChip, IntegerInstructions, Range};
use crate::{rns::Integer, AssignedInteger, WrongExt};
use halo2::arithmetic::FieldExt;
use halo2::plonk::Error;
use maingate::{
    halo2, Assigned, AssignedCondition, CombinationOptionCommon, MainGateInstructions, RegionCtx,
    Term,
};

impl<W: WrongExt, N: FieldExt, const NUMBER_OF_LIMBS: usize, const BIT_LEN_LIMB: usize>
    IntegerChip<W, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB>
{
    /// Inverts an [`AssignedInteger`].
    ///
    /// Returns the inverse of `a` and an [`AssignedCondition`] indicating if
    /// the inversion was successful.
    /// The input [`AssignedInteger`] must be reduced. This function is intended
    /// to be called through [`IntegerChip::invert`].
    pub(super) fn _invert(
        &self,
        ctx: &mut RegionCtx<'_, '_, N>,
        a: &AssignedInteger<W, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB>,
    ) -> Result<
        (
            AssignedInteger<W, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB>,
            AssignedCondition<N>,
        ),
        Error,
    > {
        let main_gate = self.main_gate();

        let one = N::one();
        let integer_one = Integer::from_big(1u32.into(), Rc::clone(&self.rns));

        let a_int = a.integer();

        let inv_or_one = match a_int.as_ref() {
            Some(a) => match a.invert() {
                Some(a) => Some(a),
                None => Some(integer_one),
            },
            None => None,
        };

        // TODO: For range constraints, we have these options:
        // 1. extend mul to support prenormalized value.
        // 2. call normalize here.
        // 3. add wrong field range check on inv.
        let inv_or_one = self.range_assign_integer(ctx, inv_or_one.into(), Range::Remainder)?;
        let a_mul_inv = &self.mul(ctx, a, &inv_or_one)?;

        // We believe the mul result is strictly less than wrong modulus, so we add
        // strict constraints here. The limbs[1..NUMBER_OF_LIMBS] of a_mul_inv
        // should be 0.
        self.assert_strict_bit(ctx, a_mul_inv)?;

        // If a_mul_inv is 0 (i.e. not 1), then inv_or_one must be 1.
        // inv_or_one = 1 <-> inv_or_one[0] = 1 /\ inv_or_one.natvie = 1.
        // Here we short x.limbs[i] as x[i].
        // 1. (a_mul_inv[0] - 1) * (inv_or_one.native - 1) = 0
        // 2. (a_mul_inv[0] - 1) * (inv_or_one[0] - 1) = 0
        main_gate.one_or_one(ctx, &a_mul_inv.limb(0), &inv_or_one.native())?;
        main_gate.one_or_one(ctx, &a_mul_inv.limb(0), &inv_or_one.limb(0))?;

        // Align with main_gain.invert(), cond = 1 - a_mul_inv
        let cond = a_mul_inv.limb(0).value().map(|a_mul_inv| one - a_mul_inv);
        let cond = main_gate.combine(
            ctx,
            &[
                Term::Assigned(a_mul_inv.limb(0), one),
                Term::Unassigned(cond, one),
                Term::Zero,
                Term::Zero,
                Term::Zero,
            ],
            -one,
            CombinationOptionCommon::OneLinerMul.into(),
        )?[1];

        Ok((inv_or_one, cond.into()))
    }

    /// Inverts an [`AssignedInteger`]. This integer must be non-zero.
    ///
    /// The input integers must be reduced. This function is intended
    /// to be called through [`IntegerChip::invert_incomplete`].
    pub(crate) fn _invert_incomplete(
        &self,
        ctx: &mut RegionCtx<'_, '_, N>,
        a: &AssignedInteger<W, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB>,
    ) -> Result<AssignedInteger<W, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB>, Error> {
        let a_int = a.integer();
        let inv = match a_int.as_ref() {
            Some(a) => match a.invert() {
                Some(a) => Some(a),
                None => {
                    // any number will fail it if a is zero
                    // no assertion here for now since we might want to fail in tests
                    Some(Integer::from_big(1u32.into(), Rc::clone(&self.rns)))
                }
            },
            None => None,
        };

        let inv = self.range_assign_integer(ctx, inv.into(), Range::Remainder)?;
        // let must_be_one = &self.mul(ctx, &a, &inv, offset)?;
        // self.assert_strict_one(ctx, must_be_one, offset)?;
        self._mul_into_one(ctx, a, &inv)?;

        Ok(inv)
    }
}
