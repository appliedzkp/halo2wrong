use crate::{
    ecc::base_field_ecc::{BaseFieldEccChip, Point},
    Witness,
};
use halo2curves::{
    group::{ff::PrimeField, Curve, Group},
    CurveAffine,
};

macro_rules! div_ceil {
    ($a:expr, $b:expr) => {
        (($a - 1) / $b) + 1
    };
}
#[derive(Clone, Debug)]
pub(crate) struct Buckets<C: CurveAffine, const NUMBER_OF_LIMBS: usize, const BIT_LEN_LIMB: usize> {
    points: Vec<Point<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>>,
    correction_point: Point<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>,
}
impl<C: CurveAffine, const NUMBER_OF_LIMBS: usize, const BIT_LEN_LIMB: usize>
    Buckets<C, NUMBER_OF_LIMBS, BIT_LEN_LIMB>
{
    pub(crate) fn points(&self) -> Vec<Point<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>> {
        self.points.clone()
    }
    pub(crate) fn correction_point(
        &self,
    ) -> &Point<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB> {
        &self.correction_point
    }
    pub(crate) fn size(&self) -> usize {
        self.points.len()
    }
    pub(crate) fn window(&self) -> usize {
        (self.size() as f64).log2() as usize
    }
}
impl<
        'a,
        C: CurveAffine,
        const NUMBER_OF_LIMBS: usize,
        const BIT_LEN_LIMB: usize,
        const NUMBER_OF_SUBLIMBS: usize,
    > BaseFieldEccChip<'a, C, NUMBER_OF_LIMBS, BIT_LEN_LIMB, NUMBER_OF_SUBLIMBS>
{
    // randomized bucket values must be known in synthesis time
    fn initial_buckets(&mut self, window: usize) -> Buckets<C, NUMBER_OF_LIMBS, BIT_LEN_LIMB> {
        assert!(window > 1);
        let size = 1 << window;
        let mut acc: C::CurveExt = self.aux_generator.into();
        // initial_bucket = R_0, 2 * R_1 , 4 * R_2, ...
        let initial_buckets: Vec<C> = (0..size)
            .map(|_| {
                let ret = acc;
                acc = acc.double();
                ret.to_affine()
            })
            .collect::<Vec<C>>();
        // register constant initial values as initial bucket values
        // note that if they are already assigned it just returns assigned values
        let buckets: Vec<
            Point<
                <C as CurveAffine>::Base,
                <C as CurveAffine>::ScalarExt,
                NUMBER_OF_LIMBS,
                BIT_LEN_LIMB,
            >,
        > = initial_buckets
            .iter()
            .map(|point| self.register_constant(*point))
            .collect();
        let number_of_windows = div_ceil!(C::Scalar::NUM_BITS as usize, window);
        // find sum of random bucket values
        // R0 + R1 + R2 + ...
        let bucket_sum = initial_buckets
            .iter()
            .skip(1)
            .rev()
            .fold(
                (C::Curve::identity(), C::Curve::identity()),
                |(sum, acc), bucket| {
                    let sum = sum + bucket;
                    (sum, acc + sum)
                },
            )
            .1;

        // apply double add to find correction point
        let correction_point = (0..number_of_windows)
            .fold(C::CurveExt::identity(), |acc, _| {
                let acc = (0..window).fold(acc, |acc, _| acc.double());
                acc + bucket_sum
            })
            .to_affine();

        // register negated correction point
        let correction_point = self.register_constant(-correction_point);
        Buckets {
            points: buckets,
            correction_point,
        }
    }
    pub(crate) fn msm_bucket(
        &mut self,
        points: &[Point<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>],
        scalars: &[Witness<C::Scalar>],
        window_size: usize,
    ) -> Point<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB> {
        let number_of_points = points.len();
        assert!(number_of_points > 0);
        assert_eq!(number_of_points, scalars.len());
        let bucket_size = 1 << window_size;
        let number_of_windows = div_ceil!(C::Scalar::NUM_BITS as usize, window_size);
        // warm up buckets

        let initial_buckets = self.initial_buckets(window_size);
        // decompose scalars into binary radix representation
        let scalars: Vec<Vec<Vec<Witness<C::Scalar>>>> = scalars
            .iter()
            .map(|scalar| {
                let bits = self
                    .operations
                    .to_bits(scalar, C::Scalar::NUM_BITS as usize);
                bits.chunks(window_size)
                    .rev()
                    .map(|chunk| chunk.to_vec())
                    .collect()
            })
            .collect();
        let distorted_result = (0..number_of_windows)
            .fold(None, |acc, j| {
                let acc = acc.map(
                    |acc: Point<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>| {
                        (0..window_size).fold(acc, |acc, _| self.double(&acc))
                    },
                );
                // each round start with same set of randomized buckets
                let mut buckets = initial_buckets.points();
                for (scalar, point) in scalars.iter().zip(points.iter()) {
                    // * slice the scalar
                    // * select a bucket
                    // * add point to the selected bucket
                    // * update buckets
                    let bucket = self.select_multi(&scalar[j], &buckets);
                    let bucket_updated = self.add(&bucket, point);
                    self.update_table(&scalar[j], &bucket_updated, &mut buckets);
                }
                // accumulate buckets
                buckets.reverse();
                let (inner_acc, sum) = (buckets[0].clone(), buckets[0].clone());
                let round_sum = buckets
                    .iter()
                    // skip init values
                    .skip(1)
                    // skip null bucket
                    .take(bucket_size - 2)
                    .fold((sum, inner_acc), |(sum, inner_acc), bucket| {
                        // sum = B_0 + B_1 + B_2 + ...
                        let sum = self.add(&sum, bucket);
                        // inner_acc = 0*B_0 + 1*B_1 + 2*B_2 + ...
                        let inner_acc = self.add(&inner_acc, &sum);
                        (sum, inner_acc)
                    })
                    .1;
                Some(match acc {
                    Some(acc) => self.add(&acc, &round_sum),
                    None => round_sum,
                })
            })
            .unwrap();
        self.add(&distorted_result, initial_buckets.correction_point())
    }
}
