// Foreign real property uses required ADS under IRC168(g)(1)(A).
// Sources: Pub.527 (property used predominantly outside the United States),
// Pub.946 and IRC168(g)(2)(C), 168(g)(3)(A).
use crate::validation::ValidationIssue;
use crate::{FederalRental, RentalLocation};

pub(crate) fn location_issue(location: &RentalLocation) -> Option<ValidationIssue> {
    let RentalLocation::Foreign { country } = location else { return None };
    if country.len() != 2
        || !country.bytes().all(|byte| byte.is_ascii_uppercase())
        || country == "US"
    {
        Some(
            ValidationIssue::new("RENTAL_FOREIGN_COUNTRY_INVALID")
                .at_field("country")
                .in_field("Foreign"),
        )
    } else {
        None
    }
}

pub(crate) fn source_admission(
    source: &FederalRental,
    recovery: Option<&crate::nonmacrs::RentalRecoverySource<'_>>,
) -> Result<(), (&'static str, &'static str)> {
    if !source.location.foreign() {
        return Ok(());
    }
    if matches!(&source.location, RentalLocation::Foreign { country } if ["AQ", "CQ", "GQ", "RQ", "VQ"].contains(&country.as_str()))
    {
        return Err((
            "RENTAL_FOREIGN_POSSESSION_UNAVAILABLE",
            "This property's U.S.-possession tax treatment needs its actual territorial income and recovery records.",
        ));
    }
    if source.assets.iter().enumerate().any(|(index, asset)| {
        recovery.is_none_or(|recovery| {
            let asset = recovery.asset(index, asset);
            let context = recovery.context(asset);
            asset.qualified_improvement(context) != Ok(true)
                || asset.macrs_source_issue_in_context(context).is_some()
        })
    }) {
        return Err((
            "RENTAL_FOREIGN_ASSET_ADS_UNAVAILABLE",
            "Supply the actual ADS class lives and original no-bonus recovery of the incidental assets.",
        ));
    }
    if source.buildings.iter().any(|building| building.in_service < 19870101)
        || !matches!(source.sale, crate::RentalSale::No)
        || source
            .buildings
            .iter()
            .any(|building| building.partial.is_some() || building.casualty.is_some())
    {
        return Err((
            "RENTAL_FOREIGN_RECOVERY_HISTORY_UNAVAILABLE",
            "Supply the foreign property's actual change-of-use and disposition recovery history.",
        ));
    }
    Ok(())
}

use crate::kernel::{ratio::exact::Exact, source_fact::SourceFact};

// One rental invocation's source operands. Annual PAL dollars are kept as
// dollars; neither gross receipts nor expense classes come from its net.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RentalForeignSource {
    pub source: i64,
    pub owner: crate::Owner,
    pub alternative: bool,
    pub country: alloc::string::String,
    pub gross_rents_cents: i64,
    pub direct_expenses_cents: SourceFact<Exact>,
    pub interest_cents: SourceFact<Exact>,
    pub depreciation_cents: SourceFact<i64>,
    pub current_before_risk_cents: SourceFact<Exact>,
    pub prior_at_risk_allowed_cents: SourceFact<i64>,
    pub operating: SourceFact<crate::extended_rules::RentalOperatingSource>,
    pub post_pal: SourceFact<crate::extended_rules::PostPalSourceAmounts>,
    pub pal_current_net_dollars: SourceFact<i64>,
    pub pal_prior_allowed_dollars: SourceFact<i64>,
    pub filed_ordinary_dollars: SourceFact<i64>,
    pub boundary: Option<&'static str>,
}
impl RentalForeignSource {
    pub(crate) fn new(
        source: i64,
        owner: crate::Owner,
        alternative: bool,
        rental: &FederalRental,
        expenses: &[crate::extended_model::RentalExpenseAmount],
    ) -> Option<Self> {
        let RentalLocation::Foreign { country } = &rental.location else { return None };
        let mut facts = Self {
            source,
            owner,
            alternative,
            country: country.clone(),
            gross_rents_cents: rental.rent,
            direct_expenses_cents: SourceFact::Unavailable,
            interest_cents: SourceFact::Unavailable,
            depreciation_cents: SourceFact::Unavailable,
            current_before_risk_cents: SourceFact::Unavailable,
            prior_at_risk_allowed_cents: SourceFact::Unavailable,
            operating: SourceFact::Unavailable,
            post_pal: SourceFact::Unavailable,
            pal_current_net_dollars: SourceFact::Unavailable,
            pal_prior_allowed_dollars: SourceFact::Unavailable,
            filed_ordinary_dollars: SourceFact::Unavailable,
            boundary: None,
        };
        facts.expenses(rental, expenses);
        Some(facts)
    }
    pub(crate) fn expenses(
        &mut self,
        rental: &FederalRental,
        expenses: &[crate::extended_model::RentalExpenseAmount],
    ) {
        // Personal-use limits and prior home-use costs do not establish the
        // current foreign expense classes. The rental owner still computes
        // its known income independently of this credit attribution.
        if !matches!(rental.personal_use, crate::RentalPersonalUse::No)
            || !rental.method_changes.is_empty()
            || rental.home_operating_carryover != 0
            || crate::rental_home_carry::total(rental, self.alternative) != Some(0)
        {
            return;
        }
        let (interest_rows, direct_rows): (alloc::vec::Vec<_>, alloc::vec::Vec<_>) =
            expenses.iter().cloned().partition(|row| {
                matches!(
                    row.category,
                    crate::RentalExpenseCategory::MortgageInterest
                        | crate::RentalExpenseCategory::VehicleLoanInterest
                        | crate::RentalExpenseCategory::OtherInterest
                )
            });
        let direct = crate::RentalExpense::total_amounts(&direct_rows);
        let interest = crate::RentalExpense::total_amounts(&interest_rows);
        // Preserve the native expense owner's grouped cent rounding. A split
        // that cannot reproduce its actual total has no established residual.
        if &direct + &interest != crate::RentalExpense::total_amounts(expenses) {
            return;
        }
        self.direct_expenses_cents = SourceFact::Reported(direct);
        self.interest_cents = SourceFact::Reported(interest);
    }
    pub(crate) fn settle_pal(&mut self, current: i64, prior: i64, allowed_loss: i64) {
        let Some(losses) =
            crate::kernel::phase::checked_loss(current).and_then(|loss| loss.checked_add(prior))
        else {
            return;
        };
        let Some(prior_allowed) = crate::section1231::determined_share(allowed_loss, losses, prior)
        else {
            return;
        };
        self.pal_current_net_dollars =
            SourceFact::Reported(current.max(0) - (allowed_loss - prior_allowed));
        self.pal_prior_allowed_dollars = SourceFact::Reported(prior_allowed);
    }
}
