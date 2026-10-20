// Direct farm-land sales (Form 4797). An installment sale of a parcel is
// figured by the Form 6252 Coverage block in installment.rs (T3-6252).

use crate::FederalResult;
use crate::federal_rules::valid_date;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::{FormLine, TaxForm};
use alloc::{format, string::String};
use libertas_macros::*;

/// Direct sale of an accepted farm land parcel
/// Opening basis, acquisition and recapture history come from the selected parcel once.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmLandSale {
    /// Accepted sale identity; assigned on normal Save
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    /// Accepted parcel from this farm
    #[libertas_number(min = 1)]
    pub basis_property: i64,
    /// Sale label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Actual date sold
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub sold: u32,
    /// Full consideration, including buyer note and assumed liabilities
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub proceeds: i64,
    /// Actual selling expenses
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub selling: i64,
    /// Elect section 1062 tax-payment installments with qualifying use, buyer and recorded covenant
    pub deferral: bool,
    /// Actual transaction character
    pub situation: FarmLandSituation,
    /// How consideration was received
    pub payment: crate::SalePayment,
}

/// Actual farm land transaction; special source owners retain unresolved mandatory gain
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum FarmLandSituation {
    /// Ordinary whole-parcel direct sale with purchased or estate fair-market-value basis
    Direct,
    /// Only part of this parcel was sold; allocation history is required
    PartialParcel,
    /// Related-party disposition; actual loss and recapture rules are required
    RelatedParty,
    /// Exchange or involuntary conversion
    ExchangeOrConversion,
    /// Gift, carryover, special-use valuation, conservation-easement or income-in-respect-of-decedent basis
    SpecialBasis,
    /// Other unresolved ownership or transaction facts
    Other,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedFarmLandSale {
    pub cost_sharing_recapture: i64,
    pub label: String,
    pub acquisition: FarmLandAcquisition,
    pub acquired: u32,
    pub sold: u32,
    pub proceeds: i64,
    pub selling: i64,
    pub basis: i64,
    pub conservation: i64,
    pub amt: FarmLandAmt,
    pub deferral: bool,
    pub eligible: bool,
    pub payment: crate::SalePayment,
}

/// Land acquisition
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FarmLandAcquisition {
    /// Purchased
    Purchased,
    /// Inherited with estate fair-market-value basis
    /// Use the reconciled section 1014 basis from estate records, including later basis adjustments. The initial basis must be solely date-of-death or elected alternate-valuation fair market value. Special-use valuation, carryover basis and income in respect of a decedent require other source calculations.
    Inherited,
}

/// AMT land-sale source records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FarmLandAmt {
    /// Same as for regular tax
    Same,
    /// Different retained AMT records
    Different {
        /// Retained AMT parcel basis
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        basis: i64,
        /// Prior conservation deductions allowed for AMT
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        conservation: i64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LandGain {
    pub gain: i64,
    pub percent: i64,
    pub recapture: i64,
    pub ordinary: i64,
    pub section1231: i64,
    // Business-asset unrecaptured section 1250 component; zero for land.
    pub unrecaptured: i64,
}
impl LandGain {
    pub(crate) fn zero() -> Self {
        Self { gain: 0, percent: 0, recapture: 0, ordinary: 0, section1231: 0, unrecaptured: 0 }
    }
}
impl ResolvedFarmLandSale {
    pub(crate) fn valid(&self) -> bool {
        self.eligible
            && !self.label.is_empty()
            && valid_date(self.acquired)
            && valid_date(self.sold)
            && self.acquired <= self.sold
            && (crate::year_constants::TAX_YEAR_START_DATE
                ..=crate::year_constants::TAX_YEAR_END_DATE)
                .contains(&self.sold)
            && match self.amt {
                FarmLandAmt::Same => true,
                FarmLandAmt::Different { basis, conservation } => {
                    [basis, conservation].iter().all(|v| (0..=100000000000).contains(v))
                }
            }
            && [self.proceeds, self.selling, self.basis, self.conservation]
                .iter()
                .all(|v| (0..=100000000000).contains(v))
            && self.payment.valid(self.proceeds)
    }
    // The year-of-sale Form 6252 of an installment sale at a gain of land
    // held more than a year: section 1252 recapture is not taxed ahead of
    // the payments (it is not section 453(i) income) but taken first from
    // each year's installment income (line 25).
    pub(crate) fn installment_for(
        &self,
        amt: bool,
    ) -> Option<crate::kernel::installment::Schedule> {
        let (basis, _) = self.records(amt);
        let long = self.acquisition == FarmLandAcquisition::Inherited
            || crate::dates::held_more_than_one_year(self.acquired, self.sold);
        if !long {
            return None;
        }
        let (recapture, cost_sharing) = self.ordinary_pools_for(amt);
        self.payment.form(
            dollars(self.proceeds),
            dollars(basis),
            0,
            dollars(self.selling),
            0,
            recapture + cost_sharing,
        )
    }
    fn records(&self, amt: bool) -> (i64, i64) {
        match (&self.amt, amt) {
            (FarmLandAmt::Different { basis, conservation }, true) => (*basis, *conservation),
            _ => (self.basis, self.conservation),
        }
    }
    fn ordinary_pools_for(&self, amt: bool) -> (i64, i64) {
        let (basis, conservation) = self.records(amt);
        let gain = positive(dollars(self.proceeds) - dollars(self.selling) - dollars(basis));
        let section1252 =
            phase::cap(gain, [fraction(dollars(conservation), self.recapture_percent(), 100)]);
        (
            section1252,
            phase::cap(positive(gain - section1252), [dollars(self.cost_sharing_recapture)]),
        )
    }
    // Form 4797 line 29: full recapture through five years, then
    // 80/60/40/20%; no recapture during the tenth or later holding year.
    fn recapture_percent(&self) -> i64 {
        if self.sold <= crate::dates::add_years(self.acquired, 5) {
            100
        } else if self.sold <= crate::dates::add_years(self.acquired, 6) {
            80
        } else if self.sold <= crate::dates::add_years(self.acquired, 7) {
            60
        } else if self.sold <= crate::dates::add_years(self.acquired, 8) {
            40
        } else if self.sold <= crate::dates::add_years(self.acquired, 9) {
            20
        } else {
            0
        }
    }
    pub(crate) fn calculate(&self) -> LandGain {
        self.calculate_for(false)
    }
    pub(crate) fn calculate_for(&self, amt: bool) -> LandGain {
        let (basis, _) = self.records(amt);
        let gain = dollars(self.proceeds) - dollars(self.selling) - dollars(basis);
        if self.acquisition == FarmLandAcquisition::Purchased
            && !crate::dates::held_more_than_one_year(self.acquired, self.sold)
        {
            return LandGain {
                gain,
                percent: 0,
                recapture: 0,
                ordinary: gain,
                section1231: 0,
                unrecaptured: 0,
            };
        }
        let percent = self.recapture_percent();
        let (recapture, cost_sharing) = self.ordinary_pools_for(amt);
        if let Some(form) = self.installment_for(amt) {
            // 2026's part: line 25 is the recapture taken first from line 24.
            return LandGain {
                gain: form.income,
                percent,
                recapture: phase::cap(form.ordinary, [recapture]),
                ordinary: form.ordinary,
                section1231: form.gain,
                unrecaptured: 0,
            };
        }
        let ordinary = recapture + cost_sharing;
        LandGain {
            gain,
            percent,
            recapture,
            ordinary,
            section1231: gain - ordinary,
            unrecaptured: 0,
        }
    }
    pub(crate) fn report(
        &self,
        id: i64,
        index: usize,
        form: crate::farm::FarmForm,
        r: &mut FederalResult,
    ) {
        let input = form.input("land_sales");
        if self.deferral {
            crate::farm_deferral::report(self, r);
        }
        if self.payment.installment() {
            let short = self.acquisition == FarmLandAcquisition::Purchased
                && !crate::dates::held_more_than_one_year(self.acquired, self.sold);
            if short {
                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::Farming),
                    "Farm land sale",
                    "ISSUE_FARM_LAND_INSTALLMENT_SHORT_HOLDING_PERIOD",
                    "An installment sale of farm land held one year or less is not calculated (Form 6252 line 26 to Form 4797 line 11).",
                );
            } else if self.deferral {
                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::Farming),
                    "Farm land sale",
                    "ISSUE_FARM_LAND_INSTALLMENT_WITH_DEFERRAL",
                    "An installment sale of farm land with the Form 1062 tax-payment election is not calculated (Form 6252 and Form 1062 line 15).",
                );
            } else if let Some((key, _message)) = self.payment.refusal(self.proceeds) {
                #[cfg(test)]
                let message = _message;

                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::Farming),
                    "Farm land sale",
                    key,
                    message,
                );
            }
        }
        if let Some(form) = self.installment_for(false) {
            crate::installment::report_sale(
                r,
                &format!("farm_{id}_land_{index}_form6252"),
                &input,
                &form,
                0,
            );
        }
        let v = self.calculate();
        if self.installment_for(false).is_some() {
            let (pool1252, pool1255) = self.ordinary_pools_for(false);
            emit_lines(
                r,
                &mut (),
                [
                    (
                        "conservation_recapture_remaining",
                        positive(pool1252 - v.recapture),
                        "EXPLAIN_FARM_LAND_INSTALLMENT_RECAPTURE_HISTORY",
                    ),
                    (
                        "cost_sharing_recapture_remaining",
                        positive(pool1255 - positive(v.ordinary - v.recapture)),
                        "EXPLAIN_FARM_LAND_INSTALLMENT_RECAPTURE_HISTORY",
                    ),
                ],
                |(name, amount, key), _, _| {
                    Some(calculation_row(
                        None,
                        &format!("farm_{id}_land_{index}_{name}"),
                        LineValue::MoneyDollars(amount),
                        &[&input],
                        "https://www.irs.gov/instructions/i6252",
                        crate::Message::new(key, alloc::vec![]),
                    ))
                },
            );
        }
        // Form 4797: the section 1231 part is line 2, or line 4 once Form 6252
        // line 26 reports it; a parcel held one year or less is ordinary gain
        // or loss on line 11.
        let short = self.acquisition == FarmLandAcquisition::Purchased
            && !crate::dates::held_more_than_one_year(self.acquired, self.sold);
        emit_lines(
            r,
            &mut (),
            [
                ("gain", v.gain, msg!("EXPLAIN_FARM_N_LAND_N_N_GAIN"), None),
                (
                    "conservation_recapture",
                    v.recapture,
                    msg!("EXPLAIN_FARM_N_LAND_N_N_CONSERVATION_RECAPTURE"),
                    None,
                ),
                (
                    "cost_sharing_recapture",
                    if self.acquisition == FarmLandAcquisition::Inherited
                        || crate::dates::held_more_than_one_year(self.acquired, self.sold)
                    {
                        positive(v.ordinary - v.recapture)
                    } else {
                        0
                    },
                    msg!("EXPLAIN_FARM_LAND_1255_RECAPTURE"),
                    None,
                ),
                (
                    "ordinary",
                    v.ordinary,
                    msg!("EXPLAIN_FARM_N_LAND_N_N_ORDINARY"),
                    short.then_some("11"),
                ),
                (
                    "section1231",
                    v.section1231,
                    msg!("EXPLAIN_FARM_N_LAND_N_N_SECTION1231"),
                    (!short).then_some(if self.installment_for(false).is_some() {
                        "4"
                    } else {
                        "2"
                    }),
                ),
            ],
            |(name, amount, explanation, form_line), _, _| {
                Some(calculation_row(
                    form_line.map(|line| FormLine::new(TaxForm::Form4797, line)),
                    &format!("farm_{id}_land_{index}_{name}"),
                    LineValue::MoneyCents(amount * 100),
                    &[&input],
                    "https://www.irs.gov/instructions/i4797",
                    explanation,
                ))
            },
        );
    }
}

// Aggregate parcels by their real business, never by a synthetic K-1 source.
#[cfg(test)]
pub(crate) fn sources(draft: &crate::FederalDraft, amt: bool) -> alloc::vec::Vec<(i64, LandGain)> {
    source_scopes(draft, amt)
        .into_iter()
        .filter_map(|(id, value, _)| value.map(|value| (id, value)))
        .collect()
}

// Keep each source's legacy totals and missing-source status together; an
// unrelated rental must not hide the status of a direct Schedule C/F source.
pub(crate) fn source_scopes(
    draft: &crate::FederalDraft,
    amt: bool,
) -> alloc::vec::Vec<(i64, Option<LandGain>, Option<&'static str>)> {
    // A farm rental's land sales are reported under its own identity.
    crate::community::property_entries(draft)
        .into_iter()
        .filter_map(|entry| {
            let farm = match &entry.income {
                crate::FederalIncome::Business { data } => match &data.activity {
                    crate::BusinessActivity::Farming { data } => data,
                    _ => return None,
                },
                crate::FederalIncome::FarmRental { data } => &data.farm,
                _ => return None,
            };
            let (value, issue) = source_view(draft, entry.id, farm, amt, true);
            Some((entry.id, value, issue))
        })
        .collect()
}

// The same actual-parcel owner serves the return's property view and Form8958's
// whole-source column. Conservation and cooperative basis adjustments stay here.
pub(crate) fn source(
    draft: &crate::FederalDraft,
    id: i64,
    farm: &crate::FarmIncome,
    amt: bool,
) -> Option<LandGain> {
    source_view(draft, id, farm, amt, false).0
}
fn source_view(
    draft: &crate::FederalDraft,
    id: i64,
    farm: &crate::FarmIncome,
    amt: bool,
    allocated: bool,
) -> (Option<LandGain>, Option<&'static str>) {
    let (components, issue) = source_components(draft, id, farm, amt, allocated);
    if farm.land_sales.is_empty() {
        return (None, issue);
    }
    let mut total = LandGain::zero();
    for (_, value) in components {
        total.gain += value.gain;
        total.recapture += value.recapture;
        total.ordinary += value.ordinary;
        total.section1231 += value.section1231;
    }
    (Some(total), issue)
}

// Preserve each actual recognized parcel component for the same source's
// loss-row reporting. Resolution and tax character stay in this owner.
pub(crate) fn source_components(
    draft: &crate::FederalDraft,
    id: i64,
    farm: &crate::FarmIncome,
    amt: bool,
    allocated: bool,
) -> (alloc::vec::Vec<(usize, LandGain)>, Option<&'static str>) {
    let mut components = alloc::vec::Vec::new();
    let mut issue = None;
    for (index, sale) in farm.land_sales.iter().enumerate() {
        let Some(sale) = sale.resolve_view(draft, id, farm, allocated) else {
            issue.get_or_insert(sale.source_key());
            continue;
        };
        components.push((index, sale.calculate_for(amt)));
    }
    (components, issue)
}

pub(crate) fn qbi(
    draft: &crate::FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: Option<&crate::section1231::Result1231>,
    id: i64,
) -> Result<i64, crate::section1231::SourceError> {
    let Some((_, own)) = crate::asset_sale::allowed_direct_sources(draft, pool, false)?
        .into_iter()
        .find(|(source, _)| *source == id)
    else {
        return Ok(0);
    };
    if own.section1231 == 0 {
        return Ok(own.ordinary);
    }
    let character = character.ok_or((
        "ISSUE_FARM_LAND_RESOLVE_BUSINESS_PROPERTY_SECTION_1231_CHARACTER_BEFORE_7F94E201",
        crate::fixture_english!(
            "Resolve business-property section 1231 character before business QBI."
        ),
    ))?;
    use crate::section1231::PropertyCharacter;
    let ordinary = match crate::section1231::activity_character(draft, pool, character, id, false)?
    {
        PropertyCharacter::Capital => 0,
        PropertyCharacter::Ordinary => own.section1231,
        PropertyCharacter::Recaptured(amount) => amount,
    };
    Ok(own.ordinary + ordinary)
}

impl FarmLandSale {
    pub(crate) fn valid(&self) -> bool {
        self.id >= 0
            && self.id <= 100_000_000_000
            && self.basis_property > 0
            && self.basis_property <= 100_000_000_000
            && !self.label.trim().is_empty()
            && valid_date(self.sold)
            && (crate::year_constants::TAX_YEAR_START_DATE
                ..=crate::year_constants::TAX_YEAR_END_DATE)
                .contains(&self.sold)
            && [self.proceeds, self.selling].into_iter().all(|v| (0..=100_000_000_000).contains(&v))
            && self.payment.valid(self.proceeds)
    }
    pub(crate) fn resolve(
        &self,
        draft: &crate::FederalDraft,
        id: i64,
        farm: &crate::FarmIncome,
    ) -> Option<ResolvedFarmLandSale> {
        self.resolve_view(draft, id, farm, false)
    }
    // Monetary parcel inputs already use the return's property fraction;
    // project externally linked cooperative adjustments by that same fraction.
    pub(crate) fn resolve_for_return(
        &self,
        draft: &crate::FederalDraft,
        id: i64,
        farm: &crate::FarmIncome,
    ) -> Option<ResolvedFarmLandSale> {
        self.resolve_view(draft, id, farm, true)
    }
    fn resolve_view(
        &self,
        draft: &crate::FederalDraft,
        id: i64,
        farm: &crate::FarmIncome,
        allocated: bool,
    ) -> Option<ResolvedFarmLandSale> {
        let parcel = farm.land_basis.iter().find(|p| p.id > 0 && p.id == self.basis_property)?;
        if !self.valid()
            || self.situation != FarmLandSituation::Direct
            || parcel.acquired > self.sold
        {
            return None;
        }
        let mut history = parcel.prior_cost_sharing.clone();
        for source in farm.specials.cost_sharing.iter().filter(|source| source.asset == parcel.id) {
            if source.received > self.sold {
                return None;
            }
            let amounts = source.current_amounts()?;
            if amounts.valuation_missing {
                return None;
            }
            history.push(crate::Farm126History {
                received: source.received,
                excluded: amounts.excludable,
                statement: source.description.clone(),
            });
        }
        let cost_sharing_recapture = history
            .iter()
            .map(|h| {
                let elapsed =
                    (10..20).find(|years| self.sold <= crate::dates::add_years(h.received, *years));
                let percent = elapsed.map_or(0, |years| 100 - (years as i64 - 10) * 10);
                fraction(h.excluded, percent, 100)
            })
            .sum();
        let regular175 = crate::farm_conservation::parcel_adjustment(
            draft, id, farm, parcel.id, self.sold, false,
        )?;
        let amt175 = crate::farm_conservation::parcel_adjustment(
            draft, id, farm, parcel.id, self.sold, true,
        )?;
        let (mut cooperative_basis, mut cooperative_proceeds) =
            crate::cooperative::land_adjustments(draft, id, parcel.id);
        if allocated {
            cooperative_basis = crate::community::property_amount(draft, id, cooperative_basis);
            cooperative_proceeds =
                crate::community::property_amount(draft, id, cooperative_proceeds);
        }
        Some(ResolvedFarmLandSale {
            label: self.label.clone(),
            acquisition: parcel.acquisition,
            acquired: parcel.acquired,
            sold: self.sold,
            proceeds: self.proceeds + cooperative_proceeds,
            selling: self.selling,
            basis: farm.parcel_basis(parcel.id, false)? - cooperative_basis + regular175.0,
            conservation: parcel.prior_conservation + regular175.1,
            amt: FarmLandAmt::Different {
                basis: farm.parcel_basis(parcel.id, true)? - cooperative_basis + amt175.0,
                conservation: parcel.prior_amt_conservation + amt175.1,
            },
            deferral: self.deferral,
            eligible: true,
            payment: self.payment.clone(),
            cost_sharing_recapture,
        })
        .filter(ResolvedFarmLandSale::valid)
    }
    fn source_key(&self) -> &'static str {
        if self.situation == FarmLandSituation::Direct {
            "ISSUE_FARM_LAND_LINKED_BASIS_NEEDED"
        } else {
            "ISSUE_FARM_LAND_TRANSACTION_OWNER_NEEDED"
        }
    }
    pub(crate) fn report(
        &self,
        draft: &crate::FederalDraft,
        farm: &crate::FarmIncome,
        id: i64,
        index: usize,
        form: crate::farm::FarmForm,
        r: &mut FederalResult,
    ) {
        if let Some(sale) = self.resolve_for_return(draft, id, farm) {
            sale.report(id, index, form, r);
        } else {
            crate::require_localized!(
                r,
                false,
                &crate::IssueContext::Section(crate::TaxSection::Farming),
                "Farm parcel disposition source",
                self.source_key(),
                "This sale needs its canonical parcel basis, recapture history and actual transaction owner. Partial, related-party, exchange, special-basis or unresolved transfers cannot be omitted as an unclaimed benefit.",
            );
        }
    }
}

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_record!(FarmLandSale {
    amounts: [proceeds, selling, situation, payment],
    gains: [],
    facts: [id, basis_property, label, sold, deferral],
});

crate::community::divide_enum!(FarmLandSituation {
    records: [],
    units: [Direct, PartialParcel, RelatedParty, ExchangeOrConversion, SpecialBasis, Other],
    tuples: [],
});

crate::community::divide_enum!(FarmLandAcquisition {
    records: [],
    units: [Purchased, Inherited],
    tuples: [],
});
