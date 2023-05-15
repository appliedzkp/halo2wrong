use crate::{maingate::config::LookupGate, Composable, RegionCtx, Witness};
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    halo2curves::ff::PrimeField,
    plonk::Error,
};
use std::collections::BTreeMap;

impl<F: PrimeField, const W: usize> LookupGate<F, W> {
    pub(crate) fn assign(
        &self,
        ctx: &mut RegionCtx<'_, F>,
        column: usize,
        witness: &Witness<F>,
    ) -> Result<(), Error> {
        let advice = self.advice_columns[column % W];
        let new_cell = ctx.assign_advice(|| "", advice, witness.value())?;
        ctx.copy_chain(witness.id(), new_cell)
    }
    pub fn layout(
        &self,
        ly: &mut impl Layouter<F>,
        cell_map: &BTreeMap<u32, AssignedCell<F, F>>,
        loookups: &BTreeMap<usize, Vec<Witness<F>>>,
    ) -> Result<(), Error> {
        self.layout_table(ly)?;
        self.layout_advice(ly, cell_map, loookups)
    }
    fn layout_advice(
        &self,
        ly: &mut impl Layouter<F>,
        cell_map: &BTreeMap<u32, AssignedCell<F, F>>,
        lookups: &BTreeMap<usize, Vec<Witness<F>>>,
    ) -> Result<(), Error> {
        ly.assign_region(
            || "load advices",
            |region| {
                let ctx = &mut RegionCtx::with_map(region, cell_map.clone());
                for (bit_len, witnesses) in lookups.iter() {
                    let tag = self.bit_len_tag.get(bit_len).unwrap_or_else(|| {
                        panic!("composition table is not set, bit lenght: {bit_len}")
                    });
                    for chunk in witnesses.chunks(W) {
                        ctx.enable(self.selector)?;
                        ctx.assign_fixed(|| "tag", self.tag, F::from(*tag as u64))?;
                        for (i, e) in chunk
                            .iter()
                            .cloned()
                            .chain(std::iter::repeat(Witness::dummy()))
                            .take(W)
                            .enumerate()
                        {
                            self.assign(ctx, i % W, &e)?;
                        }
                        ctx.next();
                    }
                }
                Ok(())
            },
        )
    }
    fn layout_table(&self, ly: &mut impl Layouter<F>) -> Result<(), Error> {
        ly.assign_table(
            || "",
            |mut table| {
                let mut offset = 0;
                table.assign_cell(
                    || "table tag",
                    self.tag_table,
                    offset,
                    || Value::known(F::ZERO),
                )?;
                table.assign_cell(
                    || "table value",
                    self.value_table,
                    offset,
                    || Value::known(F::ZERO),
                )?;
                offset += 1;

                for (bit_len, tag) in self.bit_len_tag.iter() {
                    let tag = F::from(*tag as u64);
                    let table_values: Vec<F> = (0..1 << bit_len).map(|e| F::from(e)).collect();
                    for value in table_values.iter() {
                        table.assign_cell(
                            || "table tag",
                            self.tag_table,
                            offset,
                            || Value::known(tag),
                        )?;
                        table.assign_cell(
                            || "table value",
                            self.value_table,
                            offset,
                            || Value::known(*value),
                        )?;
                        offset += 1;
                    }
                }
                Ok(())
            },
        )
    }
}
