// Coverage: current ordinary Schedule C uses the shared Form6198 owner after
// selected section179 and home-office deductions. Current domestic passive
// operating/property sources then enter the common Form8582 pool and return
// to original ScheduleC/SE or Form4797/1231/QBI consumers; REG/AMT stay independent.
// Current native property classes retain original row IDs and positive1245/1250
// subsets. Prior losses, whole-business releases,465 recapture, divided sources
// and binding179 retain their named source boundaries. IRS Forms6198/8582
// instructions and Reg.1.469-1T(d)(3): https://www.irs.gov/instructions/i8582
use crate::kernel::phase;
use crate::{BusinessActivity, FederalBusiness, RentalAtRisk};
use alloc::vec::Vec;

impl FederalBusiness {
    pub(crate) fn all_at_risk(&self) -> bool {
        matches!(self.at_risk, RentalAtRisk::All)
    }

    // The accepted no-special-treatment answer excludes prior losses and
    // unlisted elections. Current passive operating income uses the common
    // section 469 pool; farms and foreign sources retain their own boundaries.
    pub(crate) fn current_passive_supported(
        &self,
        material: crate::participation::Participation,
    ) -> bool {
        material == crate::participation::Participation::No
            && matches!(self.activity, BusinessActivity::Ordinary { .. })
            && matches!(self.source, crate::BusinessSource::UnitedStates)
            && !self.special_treatment
    }

    pub(crate) fn current_risk_supported(
        &self,
        material: crate::participation::Participation,
    ) -> bool {
        self.all_at_risk()
            || (matches!(self.activity, BusinessActivity::Ordinary { .. })
                && material != crate::participation::Participation::Unavailable
                && matches!(&self.at_risk, RentalAtRisk::Limited { record: Some(_), .. }))
    }
}

// Project one native ordinary/1231 category, then its actual negative source
// rows. Independent half-Dollar row rounding must not change the category.
// Known positive source/subset columns retain their original annual values.
pub(crate) fn project_direct_business_property_rows(
    parts: &mut [crate::business_risk::PropertySource],
) -> Result<(), crate::activity_risk::ProjectionIssue> {
    use crate::activity_risk::ProjectionIssue;
    use crate::kernel::{loss_limits::LossLimitError, ratio};
    let overflow = || ProjectionIssue::Arithmetic(LossLimitError::Overflow);
    let footing = || ProjectionIssue::Arithmetic(LossLimitError::Footing);
    // Actual property IDs, never presentation order, own residual ties.
    parts.sort_by_key(|part| part.original.id);
    for class in 0..2 {
        for part in parts.iter() {
            let (current, allowed) = (part.current_cents[class], part.allowed_cents[class]);
            if (current >= 0 && allowed != current)
                || (current < 0 && !(current..=0).contains(&allowed))
            {
                return Err(footing());
            }
        }
        let net = ratio::checked_sum(parts.iter().map(|part| part.allowed_cents[class]))
            .ok_or_else(overflow)?;
        let positive = ratio::checked_sum(parts.iter().map(|part| {
            ratio::positive(if class == 0 {
                part.original.dollars.ordinary
            } else {
                part.original.dollars.section1231
            })
        }))
        .ok_or_else(overflow)?;
        let loss = positive.checked_sub(ratio::dollars(net)).ok_or_else(overflow)?;
        if loss < 0 {
            return Err(footing());
        }
        let weights = parts
            .iter()
            .map(|part| phase::checked_loss(part.allowed_cents[class]).ok_or_else(overflow))
            .collect::<Result<Vec<_>, _>>()?;
        ratio::checked_sum(weights.iter().copied()).ok_or_else(overflow)?;
        let rows = ratio::allocate_remaining(loss, &weights);
        if ratio::checked_sum(rows.iter().copied()) != Some(loss) {
            return Err(footing());
        }
        for (part, allocated) in parts.iter_mut().zip(rows) {
            let original = if class == 0 {
                part.original.dollars.ordinary
            } else {
                part.original.dollars.section1231
            };
            if allocated < 0 || allocated > phase::checked_loss(original).ok_or_else(overflow)? {
                return Err(footing());
            }
            let value = if original < 0 { -allocated } else { original };
            if class == 0 {
                part.reported_dollars.ordinary = value;
            } else {
                part.reported_dollars.section1231 = value;
            }
        }
    }
    for part in parts {
        part.reported_dollars.gain = part
            .reported_dollars
            .ordinary
            .checked_add(part.reported_dollars.section1231)
            .ok_or_else(overflow)?;
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FinalOrdinarySource {
    pub profit_cents: i64,
    pub adjustment_cents: i64,
    pub projection: crate::activity_risk::ActivityProjection,
}

// The source producer has certified the current operating cents. Call only
// after its actual selected179 and home-office stages; annual property Dollars
// never become fabricated operating cents.
pub(crate) fn project(
    business: &FederalBusiness,
    operating: i64,
    mut property: alloc::vec::Vec<PropertySource>,
    alternative: bool,
) -> Result<
    (FinalOrdinarySource, alloc::vec::Vec<PropertySource>),
    crate::activity_risk::ProjectionIssue,
> {
    use crate::activity_risk::ProjectionIssue;
    let RentalAtRisk::Limited { record: Some(record), .. } = &business.at_risk else {
        return Err(ProjectionIssue::Source("ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED"));
    };
    let risk = &business.at_risk;
    if [false, true].into_iter().any(|amt| risk.prior(amt) != 0 || risk.prior_sale(amt) != 0) {
        return Err(ProjectionIssue::Source("BUSINESS_RISK_ORIGINAL_FORM_HISTORY_REQUIRED"));
    }
    if business.sale.is_some() {
        return Err(ProjectionIssue::Source("BUSINESS_RISK_PROPERTY_SOURCE_REQUIRED"));
    }
    property.sort_by_key(|part| part.original.id);
    let mut components = alloc::vec![(crate::activity_risk::Part1Line::Operating, operating)];
    for part in &property {
        components.extend(
            part.current_cents.map(|value| (crate::activity_risk::Part1Line::Property, value)),
        );
    }
    let projection = crate::activity_risk::project_record(
        &components,
        record,
        &crate::RiskEffectiveCategory::OtherTradeBusiness,
        alternative,
    )?;
    if projection.components.allowed.len() != components.len()
        || components.len() != 1 + 2 * property.len()
    {
        return Err(ProjectionIssue::Arithmetic(
            crate::kernel::loss_limits::LossLimitError::Footing,
        ));
    }
    for (part, allowed) in
        property.iter_mut().zip(projection.components.allowed[1..].chunks_exact(2))
    {
        part.risk_allowed_cents = [allowed[0], allowed[1]];
        part.allowed_cents = part.risk_allowed_cents;
    }
    if projection.recapture.clone()? != 0 {
        return Err(ProjectionIssue::Source("BUSINESS_RISK_RECAPTURE_CHARACTER_REQUIRED"));
    }
    let profit_cents = projection.components.allowed[0];
    let adjustment_cents = profit_cents.checked_sub(operating).ok_or(
        crate::activity_risk::ProjectionIssue::Arithmetic(
            crate::kernel::loss_limits::LossLimitError::Overflow,
        ),
    )?;
    Ok((FinalOrdinarySource { profit_cents, adjustment_cents, projection }, property))
}

// Physical ordinary source, after its actual selected deductions and at-risk
// limit. A missing PAL result is a provisional operand, never a final zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PassiveOrdinarySource {
    // Native Chapter1 expense correction, separately from risk/PAL holds.
    pub operating_correction_cents: i64,
    pub before_pal_cents: i64,
    pub allowed_cents: Option<i64>,
}

// Each current native Form4797 row keeps its accepted identity and character.
// Annual recognized Dollars are lifted once to work alongside operating Cents;
// they are never re-recognized from underlying basis or proceeds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PropertySource {
    pub original: crate::asset_sale::DirectBusinessPropertyPart,
    // Native character order throughout: ordinary, then section1231.
    pub current_cents: [i64; 2],
    pub risk_allowed_cents: [i64; 2],
    pub allowed_cents: [i64; 2],
    pub(crate) reported_dollars: crate::farm_land::LandGain,
}
impl PropertySource {
    pub(crate) fn new(
        original: crate::asset_sale::DirectBusinessPropertyPart,
    ) -> Result<Self, crate::activity_risk::ProjectionIssue> {
        if original.dollars.ordinary.checked_add(original.dollars.section1231)
            != Some(original.dollars.gain)
            || !(0..=crate::kernel::ratio::positive(original.dollars.ordinary))
                .contains(&original.dollars.recapture)
            || !(0..=crate::kernel::ratio::positive(original.dollars.section1231))
                .contains(&original.dollars.unrecaptured)
        {
            return Err(crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Footing,
            ));
        }
        let current_cents = [
            crate::extended_rules::annual_dollars_to_working_cents(original.dollars.ordinary)?,
            crate::extended_rules::annual_dollars_to_working_cents(original.dollars.section1231)?,
        ];
        Ok(Self {
            original,
            current_cents,
            risk_allowed_cents: current_cents,
            allowed_cents: current_cents,
            reported_dollars: original.dollars,
        })
    }

    // Supplied by the native property owner after the final applicable stage;
    // all reports and consumers read the same class-total projection.
    pub(crate) fn reported(&self) -> crate::farm_land::LandGain {
        self.reported_dollars
    }
}
