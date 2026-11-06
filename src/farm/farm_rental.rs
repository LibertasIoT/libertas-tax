// Coverage: Form 4835 — Supported (2026-10-09; 2026 drafts f4835 of 5/28/26 and i4835 of
//   9/21/26, Schedule E 2026 draft Part V, i8582 (2025), Pub 225 (2025)).
//   Lines: Part I 1 (crop and livestock shares converted to money, with the net of a redeemed CCC
//   crop's sale), 2a/2b (Form 1099-PATR routed by the document's chosen farm), 3a/3b (program
//   payments, Form 1099-G box 7 routed the same way, CCC market gain), 4a/4b/4c (CCC loans under
//   the section 77 election and the forfeited principal), 5a/5b/5d (crop insurance and disaster
//   payments), 6 (other income and CCC surplus), 7; Part II 8 (car and truck), 9–29 and 30a–30f
//   (the Farm page's expense lines relabeled to Form 4835 numbers, one row per line), 12
//   (depreciation, section 179 never), 19b (vehicle loan interest), 10 (section 175 conservation
//   from the one pool shared with Schedule F farms), 31, 32 (net, with prior-year unallowed losses
//   as "PAL"; blank for a loss, which is line 34c after Form 8582, F7), 34c (allowed loss after Form 8582), the suspended loss; Schedule E Part V line 40
//   (schedule_e_farm_rental_total) and line 42 (schedule_e_farming_fishing_gross); Schedule 1 line
//   5 through the passive pool (extended_rules.rs rentals_for: one PassiveEntry per rental with
//   rental_allowance = active participation, Form 8582 line 1 or 2); Form 8995 through the rental
//   QBI answer; the farmer estimated-tax test (underpayment.rs two_thirds_return_year) and the farming
//   NOL test (farm.rs check_nol). No self-employment tax, earned income or optional method
//   (section 1402(a)(1)). Land sales and asset sales reuse the Farm page records; their gain is
//   passive income only with actual exclusive-rental use, tenant and development records,
//   zero prior losses, no professional/divided treatment, and current post-risk operating
//   loss plus positive property gain netting to zero or less in both REG and AMT.
//   Other property gains remain outside PAL with a property-specific source notice.
//   The former whole-rental special switch is removed. Actual named source limits
//   preserve known rental receipts; detailed farm_specials records own their income,
//   election and mandatory-basis boundaries. Remaining independent risk policy:
//   an answered No without method facts leaves an operating loss out (line34b),
//   while supplied methods use the source's actual Form6198 owner. Section179
//   is zeroed: the property is depreciated). A real estate professional keeps the rental passive
//   (WARN_FARM_RENTAL_PASSIVE_ASSUMED). Refused by name: Form 1116 with a farm rental (line 3e),
//   section 175 with cooperative distributions on the same rental, and the shared records'
//   existing refusals. The rental's property is itemized on Form 4562 like a business's
//   (federal_rules.rs form_4562_asset_lines and form_4562_totals: lines 14a, 17, 19a-19j and 22).
//   Evidence: farm_rental.rs, farm_page.rs (FarmRental, farm_rental_record), extended_rules.rs,
//   farm.rs (FarmForm, sources), farm_ccc.rs, farm_insurance.rs, farm_conservation.rs,
//   debt.rs (crop_forfeiture); src/tests/farm/farm_rental.rs.
use crate::farm::FarmForm;
pub use crate::farm_rental_source::{
    FarmRentalPropertyUse, FarmRentalTenantConnection, FarmRentalValueEnhancement,
};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::ratio::{dollars, positive};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::push_line;
use crate::{
    BusinessAsset, BusinessMileage, FederalBusinessExpense, FederalDraft, FederalFarmRental,
    FederalIncome, FederalIncomeEntry, FederalResult, FormLine, Owner, TaxForm,
};
use alloc::format;

impl FarmRentalPropertyUse {
    fn admitted(value: &Option<Self>) -> bool {
        use crate::kernel::source_fact::SourceFact;
        let use_fact = value.as_ref().map_or(SourceFact::Unavailable, SourceFact::Reported);
        let SourceFact::Reported(Self::OnlyThisRental { tenant_connection, value_enhancement }) =
            use_fact
        else {
            return false;
        };
        let tenant =
            tenant_connection.as_ref().map_or(SourceFact::Unavailable, SourceFact::Reported);
        let enhancement =
            value_enhancement.as_ref().map_or(SourceFact::Unavailable, SourceFact::Reported);
        matches!(tenant, SourceFact::Reported(FarmRentalTenantConnection::NoOwnershipOrServices))
            && matches!(
                enhancement,
                SourceFact::Reported(FarmRentalValueEnhancement::NoParticipation)
            )
    }
}

fn sold(disposal: &crate::AssetDisposal) -> bool {
    matches!(disposal, crate::AssetDisposal::Disposed { how: crate::DisposalKind::Sold { .. }, .. })
}
fn choices(applicable: bool) -> alloc::vec::Vec<i32> {
    if applicable { alloc::vec![0, 1] } else { alloc::vec::Vec::new() }
}
// Rebuild applicability from the owning activity and actual sold property.
// Posted choice lists are never evidence. A new applicable question returns
// the complete unaccepted editor through the ordinary Save correction path.
pub(crate) fn refresh_property_questions(page: &mut crate::IncomeRecord) -> bool {
    let crate::IncomeRecord::FarmRental(page) = page else {
        return false;
    };
    let mut changed = false;
    for asset in &mut page.assets {
        let next = choices(sold(&asset.disposal));
        changed |= asset.farm_rental_use.is_none()
            && !next.is_empty()
            && next != asset.allowed_farm_rental_use;
        asset.allowed_farm_rental_use = next;
        if !sold(&asset.disposal) {
            asset.farm_rental_use = None;
        }
    }
    for sale in &mut page.more.land_sales {
        let applicable = page
            .more
            .land_basis
            .iter()
            .any(|parcel| parcel.id > 0 && parcel.id == sale.basis_property);
        let next = choices(applicable);
        changed |= sale.farm_rental_use.is_none()
            && !next.is_empty()
            && next != sale.allowed_farm_rental_use;
        sale.allowed_farm_rental_use = next;
        if !applicable {
            sale.farm_rental_use = None;
        }
    }
    changed
}

pub(crate) fn retain_inapplicable_property_facts(
    next: &mut FederalIncomeEntry,
    previous: &[FederalIncomeEntry],
) {
    let Some(previous) = previous.iter().find(|entry| entry.id > 0 && entry.id == next.id) else {
        return;
    };
    let (FederalIncome::FarmRental { data: next }, FederalIncome::FarmRental { data: previous }) =
        (&mut next.income, &previous.income)
    else {
        return;
    };
    for asset in &mut next.assets {
        if !sold(&asset.disposal)
            && asset.farm_rental_use.is_none()
            && let Some(old) = previous.assets.iter().find(|old| old.id > 0 && old.id == asset.id)
        {
            asset.farm_rental_use = old.farm_rental_use.clone();
        }
    }
    for sale in &mut next.farm.land_sales {
        if !next
            .farm
            .land_basis
            .iter()
            .any(|parcel| parcel.id > 0 && parcel.id == sale.basis_property)
            && sale.farm_rental_use.is_none()
            && let Some(old) =
                previous.farm.land_sales.iter().find(|old| old.id > 0 && old.id == sale.id)
        {
            sale.farm_rental_use = old.farm_rental_use.clone();
        }
    }
}

// Reg.1.469-2(c)(2)(iii),(f)(5),(f)(6): use and tenant/development
// records establish this property's class; its form name cannot do so.
fn property_use_complete(v: &FederalFarmRental) -> bool {
    v.farm.land_sales.iter().all(|sale| {
        FarmRentalPropertyUse::admitted(&sale.farm_rental_use)
            && sale.situation.direct()
            && !sale.deferral
            && matches!(sale.payment, crate::SalePayment::InFull)
    }) && v.assets.iter().all(|asset| {
        asset.partial.is_none()
            && (matches!(asset.disposal, crate::AssetDisposal::No)
                || (FarmRentalPropertyUse::admitted(&asset.farm_rental_use)
                    && matches!(
                        asset.disposal,
                        crate::AssetDisposal::Disposed {
                            how: crate::DisposalKind::Sold {
                                payment: crate::SalePayment::InFull,
                                buyer: crate::Buyer::Unrelated,
                                ..
                            },
                            ..
                        }
                    )))
    })
}

#[allow(clippy::too_many_arguments)] // forwards the native activity and selected-system owner context
pub(crate) fn passive_property_gain(
    context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    v: &FederalFarmRental,
    owner: usize,
    mid_quarter: bool,
    projection: &RiskSource,
    amt: bool,
) -> Result<bool, crate::activity_risk::SourceIssue> {
    let d = context.draft;
    let Some(entry) = d.income.iter().find(|entry| entry.id == id) else {
        return Ok(false);
    };
    let clean = !d.any_professional()
        && !v.history.any()
        && v.at_risk.as_ref().is_some_and(|risk| {
            risk.prior(false) == 0
                && risk.prior(true) == 0
                && risk.prior_sale(false) == 0
                && risk.prior_sale(true) == 0
        })
        && v.qbi_aggregation_group.is_none()
        && d.setup
            .as_ref()
            .and_then(|setup| {
                crate::community::share(setup, entry.owner.owned(), entry.community, &entry.income)
            })
            .is_none()
        && property_use_complete(v);
    if !clean {
        return Ok(false);
    }
    let admitted = |source: &RiskSource| -> Result<bool, crate::activity_risk::SourceIssue> {
        let gain = source.property.as_ref().map_or(0, |value| value.gain);
        let gain = crate::extended_rules::annual_dollars_to_working_cents(gain)
            .map_err(|error| crate::activity_risk::SourceIssue::new(id, error.into()))?;
        let net = source.operating.checked_add(gain).ok_or_else(|| {
            crate::activity_risk::SourceIssue::new(
                id,
                crate::kernel::loss_limits::LossLimitError::Overflow.into(),
            )
        })?;
        // Reg.1.469-2T(f)(3): nondepreciable rental property recharacterizes
        // only positive net income. These are native post-risk, pre-PAL inputs.
        Ok(source.operating < 0 && gain > 0 && net <= 0)
    };
    if !admitted(projection)? {
        return Ok(false);
    }
    let other = risk_source_with_claims(context, id, v, owner, mid_quarter, !amt, None)?;
    admitted(&other)
}

pub(crate) fn property_boundary_notes(
    d: &FederalDraft,
    id: i64,
    v: &FederalFarmRental,
    r: &mut FederalResult,
) {
    let Some(entry) = d.income.iter().find(|entry| entry.id == id) else {
        return;
    };
    let context = crate::interview::income_issue_context(entry);
    for (index, sale) in v.farm.land_sales.iter().enumerate() {
        let context = context.item("land_sales", index).named(&sale.label);
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("WARN_FARM_RENTAL_PROPERTY_PAL_SOURCE", alloc::vec![]),
            context,
        );
    }
    for (index, asset) in v.assets.iter().enumerate().filter(|(_, asset)| sold(&asset.disposal)) {
        let context = context.item("assets", index).named(&asset.description);
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("WARN_FARM_RENTAL_PROPERTY_PAL_SOURCE", alloc::vec![]),
            context,
        );
    }
}

impl FederalFarmRental {
    pub(crate) fn valid(&self) -> bool {
        self.farm.valid()
            && matches!(self.farm.accounting, crate::FarmAccounting::Cash { .. })
            && self.farm.crp.is_empty()
            && self.farm.supplies.is_none()
            && crate::federal_rules::expenses_valid(&self.expenses)
            && self.expenses.iter().all(|e| EXPENSE_LINES[e.category.index()].is_some())
            && self.vehicles.len() <= 100
            && self.vehicles.iter().all(|vehicle| vehicle.valid())
            && self.assets.len() <= 100
            && self.assets.iter().all(BusinessAsset::valid)
            && self.payroll.valid()
            && crate::activity_risk::farm_rental_source_issue(&self.at_risk, &self.history)
                .is_none()
            && self.qbi_aggregation_group.as_ref().is_none_or(|group| !group.is_empty())
    }
    // Form 4835 line 7 in cents: the rental's own gross income plus the Form
    // 1099-G box 7 payments and taxable Form 1099-PATR distributions routed
    // to it.

    pub(crate) fn gross_with_claims(
        &self,
        d: &FederalDraft,
        owner: usize,
        id: i64,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
    ) -> Result<i64, crate::Section179PreparedIssue> {
        self.farm
            .gross()
            .checked_add(crate::federal_rules::farm_routed_income_with_claims(
                d,
                owner,
                id,
                source_claims,
            )?)
            .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))
    }
    // Form 4835 line 12 in cents: depreciation, regular or alternative minimum
    // tax. Section 179 is never taken (it is zeroed before the rules run).
    pub(crate) fn depreciation(&self, mid_quarter: bool, amt: bool) -> i64 {
        crate::nonmacrs::asset_depreciation_amounts(&self.assets, mid_quarter, amt)
            .expense_cents(0)
            .expect("non-MACRS financial availability checked before legacy consumers")
    }
    // Form 4835 line 31 in cents.

    pub(crate) fn expenses_total_with_claims(
        &self,
        d: &FederalDraft,
        id: i64,
        mid_quarter: bool,
        amt: bool,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
    ) -> i64 {
        self.expenses_with_recovery_with_claims(
            d,
            id,
            amt,
            self.depreciation(mid_quarter, amt),
            BusinessMileage::total(&self.vehicles),
            source_claims,
        )
    }

    fn expenses_with_recovery_with_claims(
        &self,
        d: &FederalDraft,
        id: i64,
        amt: bool,
        depreciation: i64,
        mileage: i64,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
    ) -> i64 {
        crate::farm_conservation::rental_deduction_with_claims(
            d,
            id,
            &self.farm,
            amt,
            source_claims,
        ) + self.farm.ccc.as_ref().map_or(0, crate::FarmCcc::deficiency)
            + self.farm.specials.fertilizer_deduction()
            + self.farm.specials.production_deduction(true, amt)
            + FederalBusinessExpense::total(&self.expenses)
            + mileage
            + self
                .assets
                .iter()
                .map(|asset| asset.vehicle_operating() + asset.vehicle_loan_interest())
                .sum::<i64>()
            + depreciation
    }

    pub(crate) fn return_depreciation_with_claims(
        &self,
        d: &FederalDraft,
        id: i64,
        mid_quarter: bool,
        amt: bool,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
        macrs: Option<&crate::macrs_source::MacrsElectionView>,
    ) -> Result<i64, crate::Section179PreparedIssue> {
        let whole = d.income.iter().find_map(|entry| match &entry.income {
            FederalIncome::FarmRental { data } if entry.id == id => Some(data),
            _ => None,
        });
        if let Some(whole) = whole {
            let amount = crate::cooperative_basis::depreciation_with_claims(
                d,
                id,
                &whole.assets,
                mid_quarter,
                amt,
                source_claims,
                macrs,
            )?;
            if let Some(amount) = crate::community::farm_rental_amount(d, id, |_| amount) {
                return Ok(amount);
            }
        }
        crate::cooperative_basis::depreciation_with_claims(
            d,
            id,
            &self.assets,
            mid_quarter,
            amt,
            source_claims,
            macrs,
        )
    }
    pub(crate) fn return_mileage(&self, d: &FederalDraft, id: i64) -> i64 {
        crate::community::farm_rental_amount(d, id, |whole| BusinessMileage::total(&whole.vehicles))
            .unwrap_or_else(|| BusinessMileage::total(&self.vehicles))
    }

    #[allow(clippy::too_many_arguments)] // forwards the existing return source context
    pub(crate) fn net_on_return_with_claims(
        &self,
        d: &FederalDraft,
        owner: usize,
        id: i64,
        mid_quarter: bool,
        amt: bool,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
        macrs: Option<&crate::macrs_source::MacrsElectionView>,
    ) -> Result<i64, crate::Section179PreparedIssue> {
        let routed = crate::federal_rules::farm_routed_income_with_refunds_with_claims(
            d,
            owner,
            id,
            amt,
            source_claims,
        )?;
        Ok(self.farm.gross() + crate::community::income_amount(d, id, routed)
            - self.expenses_with_recovery_with_claims(
                d,
                id,
                amt,
                self.return_depreciation_with_claims(
                    d,
                    id,
                    mid_quarter,
                    amt,
                    source_claims,
                    macrs,
                )?,
                self.return_mileage(d, id),
                source_claims,
            ))
    }
    // Line 7 less line 31 in cents, before prior-year unallowed losses.
    pub(crate) fn net(
        &self,
        d: &FederalDraft,
        owner: usize,
        id: i64,
        mid_quarter: bool,
        amt: bool,
    ) -> Result<i64, crate::Section179PreparedIssue> {
        let source_claims = crate::related_forms::effective_claimants(d);
        self.net_with_claims(d, owner, id, mid_quarter, amt, &source_claims)
    }

    pub(crate) fn net_with_claims(
        &self,
        d: &FederalDraft,
        owner: usize,
        id: i64,
        mid_quarter: bool,
        amt: bool,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
    ) -> Result<i64, crate::Section179PreparedIssue> {
        self.net_in_context(d, owner, id, mid_quarter, amt, source_claims, None)
    }
    #[allow(clippy::too_many_arguments)] // forwards the existing return source context
    pub(crate) fn net_in_context(
        &self,
        d: &FederalDraft,
        owner: usize,
        id: i64,
        mid_quarter: bool,
        amt: bool,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
        macrs: Option<&crate::macrs_source::MacrsElectionView>,
    ) -> Result<i64, crate::Section179PreparedIssue> {
        let fallback = macrs.is_none().then(|| crate::macrs_source::MacrsElectionView::new(d));
        // This source net precedes community allocation. Keep the whole native
        // schedule and its annual category rounding, with the actual ADS context.
        let depreciation = crate::nonmacrs::asset_depreciation_amounts_for_view(
            if owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
            Some(id),
            &self.assets,
            mid_quarter,
            amt,
            macrs.or(fallback.as_ref()).unwrap(),
        )
        .expense_cents(0)
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))?;
        Ok(self.gross_with_claims(d, owner, id, source_claims)?
            - self.expenses_with_recovery_with_claims(
                d,
                id,
                amt,
                depreciation,
                BusinessMileage::total(&self.vehicles),
                source_claims,
            ))
    }
}

pub(crate) struct RiskSource {
    pub before_risk: i64,
    pub prior_risk: i64,
    pub prior_operating_allowed: Option<i64>,
    pub operating: i64,
    pub property: Option<crate::farm_land::LandGain>,
    pub property_rows: crate::asset_sale::FarmRentalPropertyRows,
    pub property_source: crate::asset_sale::FarmRentalPropertySource,
}

// Form6198 uses the operating source's cents and the property owner's already
// recognized Form4797 dollars. The checked scale change is only the shared
// settlement unit; it does not reconstruct unrounded property recognition.

pub(crate) fn risk_source_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    v: &FederalFarmRental,
    owner: usize,
    mid_quarter: bool,
    amt: bool,
    r: Option<&mut FederalResult>,
) -> Result<RiskSource, crate::activity_risk::SourceIssue> {
    let crate::related_forms::RelatedFormContext { draft: d, claims: source_claims, .. } =
        form_context;
    use crate::activity_risk::{Part1Line, ProjectionIssue, SourceIssue};
    let issue = |error| SourceIssue::new(id, error);
    let arithmetic =
        || issue(ProjectionIssue::Arithmetic(crate::kernel::loss_limits::LossLimitError::Overflow));
    if let Some(source) = crate::activity_risk::farm_rental_source_issue(&v.at_risk, &v.history) {
        return Err(issue(ProjectionIssue::Source(source.key)));
    }
    let risk = v
        .at_risk
        .as_ref()
        .ok_or_else(|| issue(ProjectionIssue::Source("ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED")))?;
    let source = crate::asset_sale::farm_rental_property_source_with_claims(
        d,
        id,
        amt,
        source_claims,
        Some(form_context.macrs().as_ref()),
    )?;
    let mut property = source.value;
    let current_operating = v
        .net_on_return_with_claims(
            d,
            owner,
            id,
            mid_quarter,
            amt,
            source_claims,
            Some(form_context.macrs().as_ref()),
        )
        .map_err(|source| match source {
            crate::Section179PreparedIssue::SourceBoundaryUnavailable(_, key) => {
                issue(ProjectionIssue::Source(key))
            }
            _ => arithmetic(),
        })?;
    let operating = current_operating.checked_sub(risk.prior(amt)).ok_or_else(arithmetic)?;
    let crate::RentalAtRisk::Limited { record: Some(record), .. } = risk else {
        // Answered No without method facts preserves the native profit path;
        // a genuine property/operating net loss still needs the missing method.
        if matches!(risk, crate::RentalAtRisk::Limited { record: None, .. }) {
            let net = crate::kernel::ratio::checked_sum([
                operating,
                property
                    .as_ref()
                    .map_or(0, |value| value.gain)
                    .checked_mul(100)
                    .ok_or_else(arithmetic)?,
                -risk.prior_sale(amt),
            ])
            .ok_or_else(arithmetic)?;
            if net < 0 {
                return Err(issue(ProjectionIssue::Source(
                    "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
                )));
            }
        }
        if risk.prior_sale(amt) != 0 {
            let value = property.get_or_insert(crate::farm_land::LandGain::zero());
            value.section1231 = dollars(
                value
                    .section1231
                    .checked_mul(100)
                    .and_then(|amount| amount.checked_sub(risk.prior_sale(amt)))
                    .ok_or_else(arithmetic)?,
            );
            value.gain = value.ordinary.checked_add(value.section1231).ok_or_else(arithmetic)?;
        }
        return Ok(RiskSource {
            before_risk: operating,
            prior_risk: risk.prior(amt),
            prior_operating_allowed: Some(risk.prior(amt)),
            operating,
            property_rows: source
                .allowed_rows(
                    &property.unwrap_or_else(crate::farm_land::LandGain::zero),
                    dollars(risk.prior_sale(amt)),
                )
                .map_err(issue)?,
            property,
            property_source: source,
        });
    };
    let ordinary = property
        .as_ref()
        .map_or(0, |value| value.ordinary)
        .checked_mul(100)
        .ok_or_else(arithmetic)?;
    let section1231 = property
        .as_ref()
        .map_or(0, |value| value.section1231)
        .checked_mul(100)
        .and_then(|value| value.checked_sub(risk.prior_sale(amt)))
        .ok_or_else(arithmetic)?;
    let projection = crate::activity_risk::project_record(
        &[
            (Part1Line::Operating, operating),
            (Part1Line::OtherDisposition, ordinary),
            (Part1Line::Property, section1231),
        ],
        record,
        &crate::RiskEffectiveCategory::HoldingRealProperty,
        amt,
    )
    .map_err(issue)?;
    let prior_operating_allowed = crate::activity_risk::prior_operating_allowed(
        Some(current_operating),
        risk.prior(amt),
        projection.components.suspended[0],
    )
    .map_err(issue)?;
    let allowed = &projection.components.allowed;
    let before_risk = operating;
    let operating = allowed[0]
        .checked_add(projection.recapture.clone().map_err(issue)?)
        .ok_or_else(arithmetic)?;
    if property.is_some() || risk.prior_sale(amt) != 0 {
        let mut value = property.take().unwrap_or_else(crate::farm_land::LandGain::zero);
        value.ordinary = dollars(allowed[1]);
        value.section1231 = dollars(allowed[2]);
        value.gain = value.ordinary.checked_add(value.section1231).ok_or_else(arithmetic)?;
        // Signed settlement suspends only negative components. Positive native
        // recapture/unrecaptured amounts retain their actual source character.
        property = Some(value);
    }
    let property_rows = source
        .allowed_rows(
            &property.unwrap_or_else(crate::farm_land::LandGain::zero),
            dollars(risk.prior_sale(amt)),
        )
        .map_err(issue)?;
    if let Some(r) = r {
        crate::extended_rules::report_risk_projection(
            id,
            amt,
            "income.farm_rental.at_risk",
            projection,
            r,
        )
        .map_err(issue)?;
    }
    Ok(RiskSource {
        before_risk,
        prior_risk: risk.prior(amt),
        prior_operating_allowed,
        operating,
        property,
        property_rows,
        property_source: source,
    })
}

// Form 4835 Part II line of each `BusinessExpenseCategory` the Farm page can
// write, in its `index()` order: (return-line suffix, printed line, explanation).
// The Farm page's expense lines keep their Schedule F labels in the source;
// these are the Form 4835 numbers, two lower for lines 11–31 of Schedule F.
// Other expenses are lines 30a–30f, one row, untagged (which of the six lines
// is a layout choice). Categories the page never writes are None.
type ExpenseLine = (&'static str, Option<&'static str>, &'static str);
const EXPENSE_LINES: [Option<ExpenseLine>; crate::BusinessExpenseCategory::COUNT] = [
    None,
    None,
    None,
    Some((
        "line13_employee_benefit_programs",
        Some("13"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE13_EMPLOYEE_BENEFIT_PROGRAMS",
    )),
    Some(("line18_insurance", Some("18"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE18_INSURANCE")),
    Some((
        "line19a_mortgage_interest",
        Some("19a"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE19A_MORTGAGE_INTEREST",
    )),
    Some((
        "line19c_other_interest",
        Some("19c"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE19C_OTHER_INTEREST",
    )),
    None,
    None,
    Some((
        "line21_pension_and_profit_sharing",
        Some("21"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE21_PENSION_AND_PROFIT_SHARING",
    )),
    Some((
        "line22a_rent_vehicles_equipment",
        Some("22a"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE22A_RENT_VEHICLES_EQUIPMENT",
    )),
    Some(("line22b_rent_other", Some("22b"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE22B_RENT_OTHER")),
    Some(("line23_repairs", Some("23"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE23_REPAIRS")),
    Some(("line26_supplies", Some("26"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE26_SUPPLIES")),
    Some(("line27_taxes", Some("27"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE27_TAXES")),
    None,
    Some(("line30_meals", None, "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE30_MEALS")),
    Some(("line28_utilities", Some("28"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE28_UTILITIES")),
    Some(("line20_labor_hired", Some("20"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE20_LABOR_HIRED")),
    Some(("line30_other", None, "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE30_OTHER")),
    Some(("line9_chemicals", Some("9"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE9_CHEMICALS")),
    Some(("line11_custom_hire", Some("11"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE11_CUSTOM_HIRE")),
    Some(("line14_feed", Some("14"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE14_FEED")),
    Some((
        "line15_fertilizers_and_lime",
        Some("15"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE15_FERTILIZERS_AND_LIME",
    )),
    Some((
        "line16_freight_and_trucking",
        Some("16"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE16_FREIGHT_AND_TRUCKING",
    )),
    Some((
        "line17_gasoline_fuel_and_oil",
        Some("17"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE17_GASOLINE_FUEL_AND_OIL",
    )),
    Some((
        "line24_seeds_and_plants",
        Some("24"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE24_SEEDS_AND_PLANTS",
    )),
    Some((
        "line25_storage_and_warehousing",
        Some("25"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE25_STORAGE_AND_WAREHOUSING",
    )),
    Some((
        "line29_veterinary_breeding_and_medicine",
        Some("29"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE29_VETERINARY_BREEDING_AND_MEDICINE",
    )),
];

// One rental's Form 4835 lines 1–7, Part II and the notes and refusals of its
// shared records. Lines 32 and 34c come from the passive pool (extended_rules.rs).

pub(crate) fn report_with_claims(
    d: &FederalDraft,
    e: &FederalIncomeEntry,
    v: &FederalFarmRental,
    r: &mut FederalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) {
    let id = e.id;
    let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
    let mq = crate::depreciation::mid_quarter(d);
    let farm = &v.farm;
    let death = crate::farm::owner_death_date(d, id);
    let farming = crate::IssueContext::Section(crate::TaxSection::Farming);
    let accepted = d.income.iter().find(|entry| entry.id == id).unwrap_or(e);
    let property = crate::community::property_entry(d, accepted);
    let FederalIncome::FarmRental { data: property } = &property.income else {
        unreachable!("same farm-rental source")
    };
    for (index, sale) in property.farm.land_sales.iter().enumerate() {
        crate::require_localized!(
            r,
            death.is_none_or(|date| sale.sold <= date),
            &farming,
            "Farm land final-return allocation",
            "ISSUE_A_SALE_AFTER_THIS_FARM_OWNER_S_DEATH_DOES_NOT_BELONG_TO_THE_OWNER_S_E146F4F3",
            "A sale after this farm owner's death does not belong to the owner's final individual return. Resolve the estate or beneficiary's ownership and reporting; a surviving spouse's own farm follows that spouse's dates.",
        );
        sale.report_with_claims(
            crate::related_forms::RelatedFormContext::original(d, source_claims),
            &property.farm,
            id,
            index,
            FarmForm::Form4835,
            r,
        );
    }
    if let Some(conservation) = &farm.conservation {
        conservation.report_with_claims(farm.gross(), d, id, FarmForm::Form4835, r, source_claims);
    }
    if let Some(insurance) = crate::farm_insurance_paper::report_value(d, id, source_claims) {
        insurance.report(true, id, FarmForm::Form4835, r);
    }
    if let Some(ccc) = &farm.ccc {
        ccc.report(true, id, FarmForm::Form4835, r);
    }
    farm.specials.report(
        true,
        id,
        FarmForm::Form4835,
        crate::interview::income_issue_context(e),
        r,
    );
    farm.report_parcels_with_claims(d, id, FarmForm::Form4835, r, source_claims);
    farm.report_property_history(&v.assets, crate::interview::income_issue_context(e), r);
    let mut cooperative =
        match crate::federal_rules::farm_cooperative_with_claims(d, owner, id, source_claims) {
            Ok(amounts) => amounts,
            Err(issue) => {
                crate::section179_return::Issue::Prepared(issue).report(r);
                return;
            }
        };
    cooperative.distributions = crate::community::income_amount(d, id, cooperative.distributions);
    cooperative.taxable = crate::community::income_amount(d, id, cooperative.taxable);
    cooperative.nonqualified = crate::community::income_amount(d, id, cooperative.nonqualified);
    cooperative.qualified_payments =
        crate::community::income_amount(d, id, cooperative.qualified_payments);
    crate::farm::require_farm_depreciation(r, &v.assets);
    let program_1099g = crate::community::income_amount(
        d,
        id,
        crate::federal_rules::farm_1099g_program(d, owner, id, source_claims),
    );
    let (market, taxable_market, sale_net, surplus) =
        farm.ccc.as_ref().map_or((0, 0, 0, 0), crate::FarmCcc::cash_parts);
    let receipts = |pick: fn(&crate::FarmReceipt) -> bool| -> i64 {
        match &farm.accounting {
            crate::FarmAccounting::Cash { receipts } => receipts
                .iter()
                .filter(|receipt| pick(receipt))
                .map(|receipt| receipt.amount())
                .sum(),
            crate::FarmAccounting::Accrual { .. } => 0,
        }
    };
    let production = receipts(|r| matches!(r, crate::FarmReceipt::Raised { .. }));
    let program = receipts(|r| matches!(r, crate::FarmReceipt::Program { .. }));
    let other = receipts(|r| matches!(r, crate::FarmReceipt::Other { .. }));
    let grants = crate::community::income_amount(
        d,
        id,
        crate::information_returns::farm_grants_with_claims(d, id, source_claims),
    );
    let routed = program_1099g + cooperative.taxable + grants;
    let gross = farm.gross()
        + routed
        + crate::community::income_amount(
            d,
            id,
            crate::farm_insurance_paper::income_delta(d, id, source_claims),
        );
    let depreciation =
        match v.return_depreciation_with_claims(d, id, mq, false, source_claims, macrs) {
            Ok(amount) => amount,
            Err(issue) => {
                crate::section179_return::Issue::Prepared(issue).report(r);
                return;
            }
        };
    let expenses = v.expenses_with_recovery_with_claims(
        d,
        id,
        false,
        depreciation,
        v.return_mileage(d, id),
        source_claims,
    );
    let path = |tail: &str| FarmForm::Form4835.input(tail);
    emit_lines(
        r,
        &mut (),
        [
            (
                "production",
                production + sale_net + farm.specials.weather_lines(true).iter().sum::<i64>(),
                Some("1"),
                msg!("EXPLAIN_FARM_RENTAL_N_PRODUCTION"),
            ),
            (
                "cooperative_distributions",
                cooperative.distributions,
                Some("2a"),
                msg!("EXPLAIN_FARM_RENTAL_N_COOPERATIVE_DISTRIBUTIONS"),
            ),
            (
                "cooperative_taxable",
                cooperative.taxable,
                Some("2b"),
                msg!("EXPLAIN_FARM_RENTAL_N_COOPERATIVE_TAXABLE"),
            ),
            (
                "program_payments",
                program + program_1099g + market + farm.specials.cost_sharing_gross(),
                Some("3a"),
                msg!("EXPLAIN_FARM_RENTAL_N_PROGRAM_PAYMENTS"),
            ),
            (
                "program_taxable",
                program + program_1099g + taxable_market + farm.specials.cost_sharing_income(),
                Some("3b"),
                msg!("EXPLAIN_FARM_RENTAL_N_PROGRAM_TAXABLE"),
            ),
            (
                "other_income",
                other + grants + surplus + farm.specials.other_income(),
                Some("6"),
                msg!("EXPLAIN_FARM_RENTAL_N_OTHER_INCOME"),
            ),
            ("gross_income", gross, Some("7"), msg!("EXPLAIN_FARM_RENTAL_N_GROSS_INCOME")),
            ("depreciation", depreciation, Some("12"), msg!("EXPLAIN_FARM_RENTAL_N_DEPRECIATION")),
            ("expenses", expenses, Some("31"), msg!("EXPLAIN_FARM_RENTAL_N_EXPENSES")),
        ],
        |(name, value, line, note), _, _| {
            Some(calculation_row(
                line.map(|line| FormLine::new(TaxForm::Form4835, line)),
                &format!("farm_rental_{id}_{name}"),
                LineValue::MoneyCents(value),
                &[FarmForm::Form4835.root()],
                "https://www.irs.gov/instructions/i4835",
                note,
            ))
        },
    );
    // Line 8 holds the standard-mileage and actual-expense vehicles' costs
    // except their business share of vehicle-loan interest, which is the new
    // 2026 line 19b; the total is unchanged.
    if !v.vehicles.is_empty() || v.assets.iter().any(BusinessAsset::is_vehicle) {
        let loan_interest = BusinessMileage::loan_interest(&v.vehicles)
            + v.assets.iter().map(BusinessAsset::vehicle_loan_interest).sum::<i64>();
        push_line(
            r,
            Some(FormLine::new(TaxForm::Form4835, "8")),
            &format!("farm_rental_{id}_car_and_truck"),
            v.return_mileage(d, id) - BusinessMileage::loan_interest(&v.vehicles)
                + v.assets.iter().map(BusinessAsset::vehicle_operating).sum::<i64>(),
            &[&path("vehicles"), &path("assets")],
            "https://www.irs.gov/instructions/i4835",
            msg!("EXPLAIN_FARM_RENTAL_N_CAR_AND_TRUCK"),
        );
        if loan_interest != 0 {
            push_line(
                r,
                Some(FormLine::new(TaxForm::Form4835, "19b")),
                &format!("farm_rental_{id}_vehicle_loan_interest"),
                loan_interest,
                &[&path("vehicles"), &path("assets")],
                "https://www.irs.gov/instructions/i4835",
                msg!("EXPLAIN_FARM_RENTAL_N_VEHICLE_LOAN_INTEREST"),
            );
        }
    }
    // Part II lines from the stored expense categories, one row per printed
    // line; categories that share a line are added together.
    let mut totals = FederalBusinessExpense::category_totals(&v.expenses);
    totals[crate::BusinessExpenseCategory::Fertilizers.index()] +=
        farm.specials.fertilizer_deduction();
    for (category, amount) in crate::farm_production::category_totals(&farm.specials, true) {
        totals[category.index()] += amount;
    }
    let mut rows: alloc::vec::Vec<(&str, Option<&str>, &str, i64)> = alloc::vec::Vec::new();
    for (entry, amount) in EXPENSE_LINES.into_iter().zip(totals) {
        let Some((suffix, line, explanation)) = entry else {
            continue;
        };
        match rows.iter_mut().find(|row| row.0 == suffix) {
            Some(row) => row.3 += amount,
            None => rows.push((suffix, line, explanation, amount)),
        }
    }
    let capital = crate::farm_production::uniform_capital(&farm.specials, true);
    rows.push((
        "line30g_production_capitalized",
        Some("30g"),
        "EXPLAIN_FARM_PRODUCTION_CAPITALIZED",
        -capital,
    ));
    for (suffix, line, explanation, amount) in rows {
        if amount != 0 {
            push_line(
                r,
                line.map(|line| FormLine::new(TaxForm::Form4835, line)),
                &format!("farm_rental_{id}_expense_{suffix}"),
                amount,
                &[&path("expenses")],
                "https://www.irs.gov/instructions/i4835",
                crate::Message::new(explanation, alloc::vec![]),
            );
        }
    }
}

// The rental's Form 8995 activity: the allowed net, its payroll W-2 wages and
// the unadjusted basis of the depreciable property still within its
// depreciable period (land is never depreciable property). `active` follows
// the section 469(h) answer, the section 199A(i) minimum-deduction test.
pub(crate) fn qbi_activity(
    v: &FederalFarmRental,
    income: i64,
    owner: crate::Owner,
    activity: i64,
    macrs: &crate::macrs_source::MacrsElectionView,
) -> Option<crate::business_rules::QbiActivity> {
    let crate::RentalBusiness::Yes { material_participation } = &v.qbi_business else {
        return None;
    };
    let property = v
        .assets
        .iter()
        .filter(|asset| asset.qbi_property_in_context(macrs.context(owner, Some(activity), asset)))
        .map(BusinessAsset::business_basis)
        .sum();
    Some(crate::business_rules::QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income,
        wages: dollars(v.payroll.qbi_wages()),
        property: dollars(property),
        active: *material_participation,
        service: Some(false),
        group: v.qbi_aggregation_group.clone(),
    })
}

// Lines 32 and 34c of Form 4835 and the notes of the pool's result for one
// rental. `net` is the rental's amount after prior-year unallowed losses (the
// Form 8582 entry), `allowed_net` the income or negative allowed loss that
// reaches Schedule E line 40.

pub(crate) fn report_pool_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    r: &mut FederalResult,
    id: i64,
    net: i64,
    allowed_loss: i64,
    suspended: i64,
    real_estate_professional: bool,
) {
    let crate::related_forms::RelatedFormContext { draft: d, .. } = form_context;
    let Some(entry) = d.income.iter().find(|entry| entry.id == id) else {
        return;
    };
    let crate::FederalIncome::FarmRental { .. } = &entry.income else {
        return;
    };
    // Form 4835 line 32 (2026 draft and instructions): "If line 31 is more than
    // line 7, don't enter your loss on line 32"; a loss goes through the at-risk
    // and passive activity loss rules to line 34c. A rental that is income only
    // before its prior-year losses prints "PAL" as zero; its loss is the
    // Form 8582 entry and line 34c.
    let line_32 = positive(net);
    let path = FarmForm::Form4835.root();
    crate::line_emission::form_line(
        r,
        FormLine::new(TaxForm::Form4835, "32"),
        &format!("farm_rental_{id}_net_income"),
        line_32,
        &[path, "rental_magi"],
        "https://www.irs.gov/instructions/i4835",
        msg!("EXPLAIN_FARM_RENTAL_N_NET_INCOME"),
    );
    if net < 0 {
        crate::line_emission::form_line(
            r,
            FormLine::new(TaxForm::Form4835, "34c"),
            &format!("farm_rental_{id}_line34c_allowed_loss"),
            -allowed_loss,
            &[path, "rental_magi"],
            "https://www.irs.gov/publications/p925",
            msg!("EXPLAIN_FARM_RENTAL_N_LINE34C_ALLOWED_LOSS"),
        );
    }
    if suspended > 0 {
        let context = crate::interview::income_issue_context(entry);
        let name = crate::interview::income_display_arg(entry);
        if real_estate_professional {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("WARN_FARM_RENTAL_PASSIVE_ASSUMED", alloc::vec![name.clone()]),
                context.clone(),
            );
        }
    }
}

// The taxable Form 1099-PATR distributions routed to a rental that its box 8
// qualified items do not cover, in whole dollars: not qualified business income.

pub(crate) fn nonqualified_cooperative_with_claims(
    d: &FederalDraft,
    id: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::Section179PreparedIssue> {
    let entry = d
        .income
        .iter()
        .find(|entry| entry.id == id)
        .ok_or(crate::Section179PreparedIssue::UnknownSource(id))?;
    Ok(dollars(
        crate::federal_rules::farm_cooperative_with_claims(
            d,
            usize::from(entry.owner == crate::DocumentOwner::Spouses),
            id,
            source_claims,
        )?
        .nonqualified,
    ))
}
