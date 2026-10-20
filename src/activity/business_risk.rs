// Coverage: current material-participating ordinary Schedule C uses the shared
// Form6198 owner after selected section179 and home-office deductions. Prior
// losses retain their original-form prerequisite; property and special activity
// character remain with their existing owners. No new loss-limit algorithm.
use crate::{BusinessActivity, FederalBusiness, RentalAtRisk};

impl FederalBusiness {
    pub(crate) fn all_at_risk(&self) -> bool {
        matches!(self.at_risk, RentalAtRisk::All)
    }

    pub(crate) fn current_risk_supported(&self) -> bool {
        self.all_at_risk()
            || (matches!(self.activity, BusinessActivity::Ordinary { .. })
                && self.material_participation
                && matches!(&self.at_risk, RentalAtRisk::Limited { record: Some(_), .. }))
    }
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
    alternative: bool,
) -> Result<FinalOrdinarySource, crate::activity_risk::ProjectionIssue> {
    use crate::activity_risk::ProjectionIssue;
    let RentalAtRisk::Limited { record: Some(record), .. } = &business.at_risk else {
        return Err(ProjectionIssue::Source("ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED"));
    };
    let risk = &business.at_risk;
    if [false, true].into_iter().any(|amt| risk.prior(amt) != 0 || risk.prior_sale(amt) != 0) {
        return Err(ProjectionIssue::Source("BUSINESS_RISK_ORIGINAL_FORM_HISTORY_REQUIRED"));
    }
    if business.sale.is_some()
        || business
            .assets
            .iter()
            .any(|asset| asset.disposal != crate::AssetDisposal::No || asset.partial.is_some())
    {
        return Err(ProjectionIssue::Source("BUSINESS_RISK_PROPERTY_SOURCE_REQUIRED"));
    }
    let projection = crate::activity_risk::project_record(
        &[(crate::activity_risk::Part1Line::Operating, operating)],
        record,
        &crate::RiskEffectiveCategory::OtherTradeBusiness,
        alternative,
    )?;
    if projection.recapture.clone()? != 0 {
        return Err(ProjectionIssue::Source("BUSINESS_RISK_RECAPTURE_CHARACTER_REQUIRED"));
    }
    let profit_cents = projection.components.allowed[0];
    let adjustment_cents = profit_cents.checked_sub(operating).ok_or(
        crate::activity_risk::ProjectionIssue::Arithmetic(
            crate::kernel::loss_limits::LossLimitError::Overflow,
        ),
    )?;
    Ok(FinalOrdinarySource { profit_cents, adjustment_cents, projection })
}
