// form_6781 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_section_1256(
    _cx: &mut Collect<'_>,
    _src: &Source<'_, '_>,
    t: &mut Totals,
    _r: &mut FederalResult,
    data: &crate::ContractsSource,
) -> Result<(), CollectFailure> {
    t.withholding_1099 += data.form.withholding;
    Ok(())
}
