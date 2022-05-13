use super::{IntegerChip, IntegerInstructions, Range};
use crate::{AssignedInteger, WrongExt};
use halo2::arithmetic::FieldExt;
use halo2::plonk::Error;
use maingate::{halo2, CombinationOptionCommon, MainGateInstructions, RegionCtx, Term};

impl<W: WrongExt, N: FieldExt, const NUMBER_OF_LIMBS: usize, const BIT_LEN_LIMB: usize>
    IntegerChip<W, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB>
{
    /// TODO not sure about this
    /// Cheks the integer value is an element of the Wrong Field (Zp)
    ///
    /// This function must be called on reduced integers
    /// TODO explain further
    pub(super) fn _assert_in_field(
        &self,
        ctx: &mut RegionCtx<'_, '_, N>,
        input: &AssignedInteger<W, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB>,
    ) -> Result<(), Error> {
        // element is: a = [a0, a1, a2, a3]
        // wrong field modulus: p = [p0, p1, p2, p3]
        // result: p - a =  c = [c0, c1, c2, c3]

        // Constraints:
        // 0 = -c_0 + p_0 - a_0 + b_0 * R
        // 0 = -c_1 + p_1 - a_1 + b_1 * R - b_0
        // 0 = -c_2 + p_2 - a_2 + b_2 * R - b_1
        // 0 = -c_3 + p_3 - a_3           - b_2

        // Witness layout:
        // | A   | B   | C   | D     |
        // | --- | --- | --- | ----- |
        // | c_0 | a_0 | b_0 | -     |
        // | c_1 | a_1 | b_1 | b_0   |
        // | c_2 | a_2 | b_2 | b_1   |
        // | c_3 | a_3 | -   | b_2   |

        let main_gate = self.main_gate();

        // to make a=p case not passing compare with p-1
        let modulus_minus_one = &self.rns.wrong_modulus_minus_one.clone();

        let integer = input.integer();
        // result containts borrows must be bits and subtraction result must be in range
        let comparision_result = integer.as_ref().map(|integer| integer.compare_to_modulus());

        let result = comparision_result.as_ref().map(|r| r.result.clone());
        let result = &self.range_assign_integer(ctx, result.into(), Range::Remainder)?;

        // assert borrow values are bits
        let borrow = comparision_result.as_ref().map(|r| r.borrow);
        let b_0 = borrow.map(|borrow| if borrow[0] { N::one() } else { N::zero() });
        let b_1 = borrow.map(|borrow| if borrow[1] { N::one() } else { N::zero() });
        let b_2 = borrow.map(|borrow| if borrow[2] { N::one() } else { N::zero() });
        let b_0 = main_gate.assign_bit(ctx, &b_0.into())?.into();
        let b_1 = main_gate.assign_bit(ctx, &b_1.into())?.into();
        let b_2 = main_gate.assign_bit(ctx, &b_2.into())?.into();

        let left_shifter = self.rns.left_shifter_r;
        let one = N::one();

        // | A   | B   | C   | D     |
        // | c_0 | a_0 | b_0 | -     |

        // 0 = -c_0 + p_0 - a_0 + b_0 * R
        main_gate.combine(
            ctx,
            &[
                Term::Assigned(result.limb(0), -one),
                Term::Assigned(input.limb(0), -one),
                Term::Assigned(b_0, left_shifter),
                Term::Zero,
                Term::Zero,
            ],
            modulus_minus_one[0],
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        // | A   | B   | C   | D     |
        // | --- | --- | --- | ----- |
        // | c_1 | a_1 | b_1 | b_0   |

        // 0 = -c_1 + p_1 - a_1 + b_1 * R - b_0
        main_gate.combine(
            ctx,
            &[
                Term::Assigned(result.limb(1), -one),
                Term::Assigned(input.limb(1), -one),
                Term::Assigned(b_1, left_shifter),
                Term::Assigned(b_0, -one),
                Term::Zero,
            ],
            modulus_minus_one[1],
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        // | A   | B   | C   | D     |
        // | --- | --- | --- | ----- |
        // | c_2 | a_2 | b_2 | b_1   |

        // 0 = -c_2 + p_2 - a_2 + b_2 * R - b_1
        main_gate.combine(
            ctx,
            &[
                Term::Assigned(result.limb(2), -one),
                Term::Assigned(input.limb(2), -one),
                Term::Assigned(b_2, left_shifter),
                Term::Assigned(b_1, -one),
                Term::Zero,
            ],
            modulus_minus_one[2],
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        // | A   | B   | C   | D     |
        // | --- | --- | --- | ----- |
        // | c_3 | a_3 | -   | b_2   |

        // 0 = -c_3 + p_3 - a_3 - b_2

        main_gate.combine(
            ctx,
            &[
                Term::Assigned(result.limb(3), -one),
                Term::Assigned(input.limb(3), -one),
                Term::Zero,
                Term::Assigned(b_2, -one),
                Term::Zero,
            ],
            modulus_minus_one[3],
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(())
    }
}
