use super::IntegerChip;
use crate::rns::MaybeReduced;
use crate::{AssignedInteger, WrongExt};
use halo2::arithmetic::FieldExt;
use halo2::plonk::Error;
use maingate::{
    halo2, CombinationOptionCommon, MainGateInstructions, RangeInstructions, RegionCtx, Term,
};

impl<W: WrongExt, N: FieldExt, const NUMBER_OF_LIMBS: usize, const BIT_LEN_LIMB: usize>
    IntegerChip<W, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB>
{
    fn assert_zero_v0_range_tune(&self) -> usize {
        // TODO
        BIT_LEN_LIMB
    }

    fn assert_zero_v1_range_tune(&self) -> usize {
        // TODO
        BIT_LEN_LIMB
    }

    fn assert_zero_quotient_range_tune(&self) -> usize {
        // TODO
        BIT_LEN_LIMB
    }

    /// Asserts an [`AssignedInteger`] is zero.
    ///
    /// The input [`AssignedInteger`] must be reduced. This function is intended
    /// to be called through [`IntegerChip::assert_zero`].
    pub(super) fn _assert_zero(
        &self,
        ctx: &mut RegionCtx<'_, '_, N>,
        a: &AssignedInteger<W, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB>,
    ) -> Result<(), Error> {
        let main_gate = self.main_gate();
        let (zero, one) = (N::zero(), N::one());

        let a_int = a.integer();
        let reduction_witness: MaybeReduced<W, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB> =
            a_int.as_ref().map(|a_int| a_int.reduce()).into();
        let quotient = reduction_witness.short();
        let (t_0, t_1, t_2, t_3) = reduction_witness.intermediate_values();
        let (_, _, v_0, v_1) = reduction_witness.residues();

        // apply ranges

        let range_chip = self.range_chip();
        let quotient = range_chip.range_value(
            ctx,
            &quotient.into(),
            self.assert_zero_quotient_range_tune(),
        )?;
        let v_0 = range_chip.range_value(ctx, &v_0.into(), self.assert_zero_v0_range_tune())?;
        let v_1 = range_chip.range_value(ctx, &v_1.into(), self.assert_zero_v1_range_tune())?;

        // | A   | B | C   | D |
        // | --- | - | --- | - |
        // | a_0 | q | t_0 | - |
        // | a_1 | q | t_1 | - |
        // | a_2 | q | t_2 | - |
        // | a_3 | q | t_3 | - |

        let t_0 = main_gate.combine(
            ctx,
            &[
                Term::Assigned(a.limb(0), one),
                Term::Assigned(quotient, self.rns.negative_wrong_modulus_decomposed[0]),
                Term::Unassigned(t_0, -one),
                Term::Zero,
                Term::Zero,
            ],
            zero,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?[2];

        let t_1 = main_gate.combine(
            ctx,
            &[
                Term::Assigned(a.limb(1), one),
                Term::Assigned(quotient, self.rns.negative_wrong_modulus_decomposed[1]),
                Term::Unassigned(t_1, -one),
                Term::Zero,
                Term::Zero,
            ],
            zero,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?[2];

        let t_2 = main_gate.combine(
            ctx,
            &[
                Term::Assigned(a.limb(2), one),
                Term::Assigned(quotient, self.rns.negative_wrong_modulus_decomposed[2]),
                Term::Unassigned(t_2, -one),
                Term::Zero,
                Term::Zero,
            ],
            zero,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?[2];

        let t_3 = main_gate.combine(
            ctx,
            &[
                Term::Assigned(a.limb(3), one),
                Term::Assigned(quotient, self.rns.negative_wrong_modulus_decomposed[3]),
                Term::Unassigned(t_3, -one),
                Term::Zero,
                Term::Zero,
            ],
            zero,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?[2];

        // u_0 = t_0 + t_1 * R
        // u_0 = v_0 * R^2
        // 0 = t_0 + t_1 * R - v_0 * R^2

        // | A   | B   | C   | D     |
        // | --- | --- | --- | ----- |
        // | t_0 | t_1 | v_0 | -     |

        let left_shifter_r = self.rns.left_shifter_r;
        let left_shifter_2r = self.rns.left_shifter_2r;

        main_gate.combine(
            ctx,
            &[
                Term::Assigned(t_0, one),
                Term::Assigned(t_1, left_shifter_r),
                Term::Assigned(v_0, -left_shifter_2r),
                Term::Zero,
                Term::Zero,
            ],
            zero,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        // u_1 = t_2 + t_3 * R
        // v_1 * 2R = u_1 + v_0
        // 0 = t_2 + t_3 * R + v_0 - v_1 * 2R

        // | A   | B   | C   | D     |
        // | --- | --- | --- | ----- |
        // | t_2 | t_3 | v_0 | v_1   |

        main_gate.combine(
            ctx,
            &[
                Term::Assigned(t_2, one),
                Term::Assigned(t_3, left_shifter_r),
                Term::Assigned(v_0, one),
                Term::Assigned(v_1, -left_shifter_2r),
                Term::Zero,
            ],
            zero,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        // native red

        main_gate.combine(
            ctx,
            &[
                Term::Assigned(a.native(), -one),
                Term::Zero,
                Term::Assigned(quotient, self.rns.wrong_modulus_in_native_modulus),
                Term::Zero,
                Term::Zero,
            ],
            zero,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(())
    }
}
