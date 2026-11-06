// Form 4797 direct dispositions: a business asset or
// actual-expense vehicle sold, scrapped or taken out of business use, and a
// whole rental property sold. The Form 4797 Coverage block is in section1231.rs.
// https://www.irs.gov/instructions/i4797
// https://www.irs.gov/publications/p544
// https://www.irs.gov/publications/p946 (depreciation in the year of disposition)
// https://www.irs.gov/publications/p925 (section 469(g) full disposition)
pub use crate::disposition_model::{
    AssetDisposal, Buyer, DisposalKind, Livestock, LivestockPurpose, PartialDisposition,
    RentalSale, SoldLivestock,
};
pub(crate) use crate::disposition_model::{Convention, disposal_half_months};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::push_line;
use crate::{BusinessAsset, FederalDraft, FederalIncome, FederalRental, FederalResult};
use crate::{FormLine, TaxForm};
use alloc::{format, vec::Vec};

// One business asset's Form 4797 amounts, whole dollars. `realized` is the
// business share of the gross sales price (line 22); `basis` the cost, reduced
// by nothing a Form 4797 line 24 item takes into account, plus the expense of
// sale (line 23); `depreciation` line 24; `personal` is the personal-use part's
// gain or loss (a loss is not deductible) from `personal_proceeds` less
// `personal_basis`, its Form 8949 row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AssetGain {
    pub realized: i64,
    pub basis: i64,
    pub depreciation: i64,
    // The gain or loss recognized: for a condemnation, the gain not postponed
    // (section 1033); for a sale to a related buyer, zero when it is a loss.
    pub gain: i64,
    pub long: bool,
    pub scrapped: bool,
    // Livestock for draft, breeding, dairy or sport: its lines are 7 and 18.
    pub livestock: bool,
    // Section 1250 property (a building, a 15-year land improvement, a 20-year
    // farm building): line 28 recapture, not line 27.
    pub section1250: bool,
    pub recapture: i64,
    // Line 27b: the section 1245 recapture (section 1250 property has it only
    // for a section 179 deduction).
    pub recapture_1245: i64,
    pub ordinary: i64,
    pub section1231: i64,
    pub unrecaptured: i64,
    pub personal: i64,
    pub personal_proceeds: i64,
    pub personal_basis: i64,
    // Form 4797 line 28a: the additional depreciation of section 1250 property
    // (zero for a straight-line building without a section 50(c) reduction).
    pub additional: i64,
    // The year-of-sale Form 6252 of an installment sale at a gain; the
    // section 1231 and unrecaptured amounts above are then 2026's part.
    pub installment: Option<crate::kernel::installment::Schedule>,
    // Section 1033: the realized gain a replacement postpones.
    pub postponed: i64,
    // Section 267: a loss on a sale to a relative or a controlled buyer,
    // which is not deductible.
    pub disallowed: i64,
    // Section 1239: the whole gain on a sale to a controlled buyer is
    // ordinary (Part II line 11).
    pub section1239: bool,
}
impl AssetGain {
    // The gain or loss recognized in 2026.
    pub(crate) fn recognized(&self) -> i64 {
        match self.installment {
            Some(_) => self.ordinary + self.section1231,
            None => self.gain,
        }
    }
    // Whether the gain or loss is reported in Part II (line 11 or line 18):
    // an abandonment, property held too short a time, or a sale to a buyer
    // section 1239 treats the gain of as ordinary.
    pub(crate) fn part_two(&self) -> bool {
        self.scrapped || !self.long || self.section1239
    }
}

// A partial disposition of a building: the part's own cost, the depreciation
// allowed on it, the gain or loss, and the unrecaptured section 1250 share.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PartialGain {
    pub cost: i64,
    pub proceeds: i64,
    pub depreciation: i64,
    pub gain: i64,
    pub long: bool,
    pub unrecaptured: i64,
}
impl PartialGain {
    // The Form 4797 character: line 2 (section 1231) when held more than a
    // year, line 11 (ordinary) otherwise; a gain on the part carries its
    // unrecaptured section 1250 share, the smaller of the gain and the
    // depreciation allowed on the part.
    pub(crate) fn land_gain(&self) -> crate::farm_land::LandGain {
        crate::farm_land::LandGain {
            gain: self.gain,
            percent: 0,
            recapture: 0,
            ordinary: if self.long { 0 } else { self.gain },
            section1231: if self.long { self.gain } else { 0 },
            unrecaptured: self.unrecaptured,
        }
    }
}

// One annual Form4797 observation boundary for every actual property source.
// Keep source cents until here; gain is then derived from these rounded native
// columns so the reported cost, recovery, adjusted basis and gain always foot.
struct PropertyColumns {
    realized: i64,
    net: i64,
    cost: i64,
    recovery: i64,
}
impl PropertyColumns {
    fn from_cents(realized: i64, net: i64, cost: i64, recovery: i64) -> Self {
        Self {
            realized: dollars(realized),
            net: dollars(net),
            cost: dollars(cost),
            recovery: dollars(recovery),
        }
    }
    fn gain(&self) -> i64 {
        self.net - (self.cost - self.recovery)
    }
    fn expense(&self) -> i64 {
        self.realized - self.net
    }
}
// Shared recognition consumes the annual native property columns.
// This stage does no additional rounding.
fn recognize_disposal(
    realized_gain: i64,
    net_realized: i64,
    replacement: Option<i64>,
    buyer: Buyer,
) -> (i64, i64, i64, bool) {
    let (mut gain, postponed) = match replacement {
        Some(spent) if realized_gain > 0 => {
            let recognized = phase::cap(realized_gain, [phase::floor(net_realized - spent, 0)]);
            (recognized, realized_gain - recognized)
        }
        _ => (realized_gain, 0),
    };
    let disallowed = if buyer != Buyer::Unrelated && gain < 0 {
        let loss = -gain;
        gain = phase::floor(gain, 0);
        loss
    } else {
        0
    };
    (gain, postponed, disallowed, buyer == Buyer::ControlledEntity && gain > 0)
}
struct InstallmentProperty {
    realized: i64,
    cost: i64,
    recovery: i64,
    sale_expense: i64,
    gain: i64,
    long: bool,
    disallowed: i64,
    section1239: bool,
}
type PropertyCharacter = (i64, i64, i64, i64);
fn settle_installment(
    payment: Option<crate::SalePayment>,
    source: InstallmentProperty,
    original: PropertyCharacter,
) -> (Option<crate::kernel::installment::Schedule>, PropertyCharacter) {
    let (recapture, ordinary, section1231, unrecaptured) = original;
    let recaptured = if source.long {
        recapture
    } else {
        phase::floor(phase::cap(source.gain, [source.recovery]), 0)
    };
    let installment = payment
        .filter(|_| source.gain > 0 && source.disallowed == 0 && !source.section1239)
        .and_then(|payment| {
            payment.form(
                source.realized,
                source.cost,
                source.recovery,
                source.sale_expense,
                recaptured,
                0,
            )
        });
    let amounts = match &installment {
        Some(form) if source.long => {
            (recapture, ordinary, form.gain, phase::cap(unrecaptured, [form.gain]))
        }
        Some(form) => (recaptured, recaptured + form.gain, 0, 0),
        None => (recapture, ordinary, section1231, unrecaptured),
    };
    (installment, amounts)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PartialOperands {
    pub full_year: i64,
    pub cost: i64,
    pub original_part_cost: i64,
    pub original_total_cost: i64,
}

struct AssetSaleRecovery {
    depreciation: crate::kernel::ratio::exact::Exact,
    straight_line: Option<crate::kernel::ratio::exact::Exact>,
}
fn rental_asset_sale(
    asset: &BusinessAsset,
    index: usize,
    mid_quarter: bool,
    amt: bool,
    source: Option<&crate::nonmacrs::RentalRecoverySource<'_>>,
) -> Option<AssetGain> {
    let recovery = if let Some(source) = source.filter(|_| asset.nonmacrs()) {
        let actual = source.asset(index, asset);
        let recovery = actual.nonmacrs_sale_recovery(mid_quarter, amt)?;
        Some(AssetSaleRecovery {
            depreciation: source.project(&recovery.depreciation),
            straight_line: recovery.straight_line.map(|amount| source.project(&amount)),
        })
    } else {
        if let Some(source) = source {
            return asset.sale_in_context(mid_quarter, amt, source.context(asset));
        }
        None
    };
    asset.sale_with_recovery(mid_quarter, amt, None, recovery.as_ref())
}

impl BusinessAsset {
    // A sale, condemnation or scrapping that produces Form 4797 amounts; None
    // while the asset is kept, including a change to personal use.
    // Form4797 line24 and the section1250 comparison are lifetime operands.
    // Project each completed operand once, before the existing sale stages.
    fn nonmacrs_sale_recovery(&self, mid_quarter: bool, amt: bool) -> Option<AssetSaleRecovery> {
        self.nonmacrs_route()?;
        let asset = if amt { self.amt_projection_or_self() } else { self.clone() };
        let prior = match asset.history {
            crate::EquipmentHistory::Continuing { prior_depreciation, .. } => prior_depreciation,
            crate::EquipmentHistory::New { section179, .. } => section179.elected(),
        };
        let bonus = asset.bonus_amount();
        let current = asset.nonmacrs_recovery(bonus).ok()?.exact_amount.unwrap_or_else(|| {
            crate::kernel::ratio::exact::whole(asset.deduction(mid_quarter) - bonus)
        });
        let allowed = crate::kernel::ratio::exact::whole(prior)
            + crate::kernel::ratio::exact::whole(bonus)
            + current;
        // Admitted non-MACRS real property uses actual straight-line history.
        // Keep its original fractional allowance until the Form4797 boundary;
        // credit basis reductions remain additional depreciation at that owner.
        let straight_line = self.section1250().then(|| allowed.clone());
        let depreciation = allowed
            + crate::kernel::ratio::exact::whole(self.investment_basis_reduction())
            - crate::kernel::ratio::exact::whole(self.investment_basis_increase())
            + crate::kernel::ratio::exact::whole(self.entered_credit_reduction());
        Some(AssetSaleRecovery { depreciation, straight_line })
    }
    pub(crate) fn sale_in_context(
        &self,
        mid_quarter: bool,
        amt: bool,
        context: crate::macrs_source::AdsInvocation<'_>,
    ) -> Option<AssetGain> {
        let selected = if amt { self.amt_projection_or_self() } else { self.clone() };
        if selected.macrs_source_issue_in_context(context).is_some() {
            return None;
        }
        if selected.source_ads_recovery_in_context(context).ok().flatten().is_none() {
            return self.sale_for(mid_quarter, amt);
        }
        let prior = match selected.history {
            crate::EquipmentHistory::Continuing { prior_depreciation, .. } => prior_depreciation,
            crate::EquipmentHistory::New { section179, .. } => section179.elected(),
        };
        let current = selected
            .source_bonus_amount_in_context(context)
            .checked_add(selected.source_macrs_amount_in_context(mid_quarter, context)?)?;
        let allowed = prior.checked_add(current)?;
        let recovery = AssetSaleRecovery {
            depreciation: crate::kernel::ratio::exact::whole(
                allowed
                    .checked_add(selected.investment_basis_reduction())?
                    .checked_sub(selected.investment_basis_increase())?
                    .checked_add(selected.entered_credit_reduction())?,
            ),
            straight_line: selected
                .source_straight_line_through_sale(context)
                .map(crate::kernel::ratio::exact::whole),
        };
        self.sale_with_recovery(mid_quarter, amt, None, Some(&recovery))
    }
    pub(crate) fn sale_for(&self, mid_quarter: bool, amt: bool) -> Option<AssetGain> {
        self.sale_with_purchase(mid_quarter, amt, None)
    }
    pub(crate) fn sale_with_purchase(
        &self,
        mid_quarter: bool,
        amt: bool,
        purchase: Option<crate::cooperative_basis::AssetAmounts>,
    ) -> Option<AssetGain> {
        self.sale_with_recovery(mid_quarter, amt, purchase, None)
    }
    fn sale_with_recovery(
        &self,
        mid_quarter: bool,
        amt: bool,
        purchase: Option<crate::cooperative_basis::AssetAmounts>,
        recovery: Option<&AssetSaleRecovery>,
    ) -> Option<AssetGain> {
        let AssetDisposal::Disposed { date, how } = &self.disposal else {
            return None;
        };
        let (price, expenses, scrapped, livestock, buyer, replacement) = match how {
            DisposalKind::Sold { proceeds, selling_expenses, livestock, buyer, .. } => {
                (*proceeds, *selling_expenses, false, livestock.classification(), *buyer, None)
            }
            DisposalKind::Condemned { award, expenses, replacement_cost } => {
                (*award, *expenses, false, Livestock::No, Buyer::Unrelated, Some(*replacement_cost))
            }
            DisposalKind::Scrapped => (0, 0, true, Livestock::No, Buyer::Unrelated, None),
            DisposalKind::PersonalUse => return None,
        };
        let total = price - expenses;
        let asset = if amt { self.amt_projection_or_self() } else { self.clone() };
        let prior = match asset.history {
            crate::EquipmentHistory::Continuing { prior_depreciation, .. } => prior_depreciation,
            // Placed in service and disposed of in 2026: only the section 179
            // election is a deduction (no MACRS or bonus, Reg. 1.168(d)-1).
            crate::EquipmentHistory::New { section179, .. } => section179.elected(),
        };
        // Line 22 is the gross sales price and line 23 the cost plus the
        // expense of sale; the gain is the same either way.
        // Section 50(c): the basis reduction of an investment credit counts as
        // depreciation allowed (Form 4797 line 24), less the basis a recapture
        // restores at the disposition (section 50(c)(2)), so line 23 is the
        // cost before it. A clean-vehicle or other entered credit's reduction
        // is likewise line 24 depreciation, not a lower line 23 (i4797 line 22).
        let reduction = purchase.map_or_else(
            || self.investment_basis_reduction(),
            |amounts| amounts.investment_reduction,
        );
        let net_reduction = reduction - self.investment_basis_increase();
        let credit = purchase.map_or_else(
            || self.entered_credit_reduction(),
            |amounts| amounts.entered_credit_reduction,
        );
        let owned_recovery =
            if recovery.is_none() { self.nonmacrs_sale_recovery(mid_quarter, amt) } else { None };
        let recovery = recovery.or(owned_recovery.as_ref());
        let mut columns = PropertyColumns::from_cents(
            self.business_share_of(price),
            self.business_share_of(total),
            purchase.map_or_else(
                || self.business_basis() + reduction + credit,
                |amounts| amounts.sale_cost_cents,
            ),
            prior
                + purchase.map_or_else(
                    || asset.deduction(mid_quarter),
                    |amounts| amounts.bonus + amounts.macrs,
                )
                + net_reduction
                + credit,
        );
        if let Some(recovery) = recovery {
            columns.recovery = crate::kernel::ratio::exact::dollars(&recovery.depreciation)?;
        }
        let PropertyColumns { realized, net: net_realized, cost, recovery: depreciation } = columns;
        let sale_expense = columns.expense();
        let basis = cost + sale_expense;
        let realized_gain = columns.gain();
        // Existing equipment keeps its original annual Form4797 line rounding.
        let (gain, postponed, disallowed, section1239) = recognize_disposal(
            realized_gain,
            net_realized,
            replacement.map(|spent| dollars(self.business_share_of(spent))),
            buyer,
        );
        let personal_proceeds = dollars(total) - net_realized;
        let personal_basis = purchase.and_then(|amounts| amounts.personal_basis_cents).map_or_else(
            || dollars(self.cost) - dollars(self.business_basis() + reduction + credit),
            dollars,
        );
        let personal = personal_proceeds - personal_basis;
        let long = livestock.held_long_enough(self.acquired, *date);
        let section1250 = self.section1250();
        let mut additional = 0;
        let mut recapture_1245 = 0;
        let (recapture, ordinary, section1231, unrecaptured) = if disallowed > 0 {
            (0, 0, 0, 0)
        } else if scrapped || !long || section1239 {
            // Line 11 (line 18 for livestock): an abandonment, property held
            // too short a time and a gain section 1239 makes ordinary.
            (0, gain, 0, 0)
        } else if gain <= 0 {
            (0, 0, gain, 0)
        } else if self.class == crate::EquipmentClass::ThirtyNine
            && purchase.and_then(|amounts| amounts.straight_line_through_sale).is_none()
            && recovery.and_then(|value| value.straight_line.as_ref()).is_none()
            && self.first_year_179() == 0
        {
            // Straight-line real property placed in service after 1986 has no
            // additional depreciation (line 28) unless a section 50(c)
            // reduction counts as depreciation: the depreciation allowed over
            // straight line on the cost before the reduction (i4797 line 26a)
            // is ordinary, the rest of the gain up to the depreciation is
            // unrecaptured section 1250 gain.
            if self.investment_credit.is_some() {
                additional = crate::kernel::ratio::positive(
                    depreciation - dollars(self.straight_line_on_cost(*date)),
                );
            }
            let amounts = crate::kernel::recapture::section1250(gain, depreciation, 0, additional);
            (
                amounts.ordinary_dollars,
                amounts.ordinary_dollars,
                amounts.remaining1231_dollars,
                amounts.unrecaptured1250_dollars,
            )
        } else if section1250 {
            // A 15-year land improvement or 20-year farm building is section
            // 1250 property (Pub 225): a section 179 deduction is recaptured
            // as section 1245 gain, the depreciation allowed over straight
            // line on the rest of the cost is additional depreciation, and the
            // straight-line part of the depreciation is unrecaptured gain.
            let section179 = phase::cap(dollars(self.first_year_179()), [depreciation]);
            let straight_line = purchase
                .and_then(|amounts| amounts.straight_line_through_sale)
                .map(dollars)
                .or_else(|| {
                    recovery
                        .and_then(|value| value.straight_line.as_ref())
                        .and_then(crate::kernel::ratio::exact::dollars)
                })
                .unwrap_or_else(|| {
                    dollars(self.straight_line_through(
                        self.business_share_of(self.cost) - self.first_year_179(),
                        *date,
                    ))
                });
            additional = crate::kernel::ratio::positive(depreciation - section179 - straight_line);
            let amounts =
                crate::kernel::recapture::section1250(gain, depreciation, section179, additional);
            recapture_1245 = amounts.section1245_dollars;
            (
                amounts.ordinary_dollars,
                amounts.ordinary_dollars,
                amounts.remaining1231_dollars,
                amounts.unrecaptured1250_dollars,
            )
        } else {
            // Section 1245: line 27b is the smaller of the gain and line 24.
            let amounts = crate::kernel::recapture::section1245(gain, depreciation);
            recapture_1245 = amounts.section1245_dollars;
            (amounts.ordinary_dollars, amounts.ordinary_dollars, amounts.remaining1231_dollars, 0)
        };
        // Form 6252: the recapture stays in 2026 (line 12); the rest of the
        // gain follows the payments, its unrecaptured section 1250 part
        // first. The method applies to the business part; the personal part's
        // gain is counted in full (the election out, section 453(d)). Property
        // held one year or less recaptures the depreciation it can and its
        // balance is ordinary (Part II line 11, "From Form 6252").
        let payment = match how {
            DisposalKind::Sold { payment, .. } => {
                Some(payment.scaled(|amount| self.business_share_of(amount)))
            }
            _ => None,
        };
        let (installment, (recapture, ordinary, section1231, unrecaptured)) = settle_installment(
            payment,
            InstallmentProperty {
                realized,
                cost,
                recovery: depreciation,
                sale_expense,
                gain,
                long,
                disallowed,
                section1239,
            },
            (recapture, ordinary, section1231, unrecaptured),
        );
        Some(AssetGain {
            realized,
            basis,
            depreciation,
            gain,
            long,
            scrapped,
            livestock: livestock != Livestock::No,
            section1250,
            recapture,
            recapture_1245,
            ordinary,
            section1231,
            unrecaptured,
            personal,
            personal_proceeds,
            personal_basis,
            additional,
            installment,
            postponed,
            disallowed,
            section1239,
        })
    }
    // The partial disposition of a 39-year building (Reg. 1.168(i)-8(d)): the
    // part's own cost leaves the building with the depreciation allowed on it
    // through the month of the disposition (the part's share of the
    // building's, Reg. 1.168(i)-8(f)(3)), and the gain or loss is section
    // 1231 when the building has been held more than a year, else ordinary.
    pub(crate) fn partial_for(&self) -> Option<PartialGain> {
        self.partial_with_purchase(None)
    }
    pub(crate) fn partial_with_purchase(
        &self,
        purchase: Option<PartialOperands>,
    ) -> Option<PartialGain> {
        let part = self.partial.as_ref()?;
        let crate::EquipmentHistory::Continuing { prior_depreciation, .. } = self.history else {
            return None;
        };
        if self.class != crate::EquipmentClass::ThirtyNine || self.disposal_date().is_some() {
            return None;
        }
        // The building's depreciation through the month of the disposition:
        // its mid-month share of the full year.
        let allowed = prior_depreciation
            + crate::kernel::ratio::fraction(
                purchase.map_or_else(
                    || self.whole_year_macrs_before_partial(),
                    |amounts| amounts.full_year,
                ),
                disposal_half_months(part.date, Convention::MidMonth),
                24,
            );
        let (part_cost, total_cost) = purchase.map_or((part.cost_of_part, self.cost), |amounts| {
            (amounts.original_part_cost, amounts.original_total_cost)
        });
        let depreciation = dollars(crate::kernel::ratio::fraction(allowed, part_cost, total_cost));
        let cost = dollars(
            purchase
                .map_or_else(|| self.business_share_of(part.cost_of_part), |amounts| amounts.cost),
        );
        let proceeds = dollars(self.business_share_of(part.proceeds));
        let gain = proceeds - (cost - depreciation);
        let long = crate::dates::held_more_than_one_year(self.placed, part.date);
        Some(PartialGain {
            cost,
            proceeds,
            depreciation,
            gain,
            long,
            unrecaptured: if long && gain > 0 {
                crate::kernel::recapture::section1250(gain, depreciation, 0, 0)
                    .unrecaptured1250_dollars
            } else {
                0
            },
        })
    }
    // The disposal facts this App calculates; None when supported, else the
    // situation and form line refused by name.
    pub(crate) fn disposal_refusal_in_context(
        &self,
        context: crate::macrs_source::AdsInvocation<'_>,
    ) -> Option<(&'static str, &'static str)> {
        let AssetDisposal::Disposed { .. } = &self.disposal else {
            return None;
        };
        if let Some(key) = self.macrs_source_issue_in_context(context) {
            return Some((
                key,
                crate::fixture_english!(
                    "The property's actual recovery source is needed to calculate disposal basis and recapture; omitted current depreciation does not establish its allowable history."
                ),
            ));
        }
        self.disposal_terms_refusal()
    }
    fn disposal_terms_refusal(&self) -> Option<(&'static str, &'static str)> {
        let AssetDisposal::Disposed { date, how } = &self.disposal else {
            return None;
        };
        if *date <= self.placed {
            return Some((
                "ISSUE_ASSET_DISPOSAL_DATE_BEFORE_SERVICE",
                crate::fixture_english!(
                    "The date sold or stopped using must be after the date placed in service."
                ),
            ));
        }
        match how {
            DisposalKind::Sold { proceeds, payment, .. } => payment.refusal(*proceeds),
            _ => None,
        }
    }
}

// The part of the gain on equipment rented out outside a business that
// section 1245 makes ordinary: the depreciation allowed, or all of the gain on
// a sale to a controlled buyer (section 1239). Whole dollars.
pub(crate) fn equipment_ordinary(sale: &AssetGain) -> i64 {
    if sale.gain <= 0 {
        0
    } else if sale.section1239 {
        sale.gain
    } else {
        phase::floor(phase::cap(sale.gain, [sale.depreciation]), 0)
    }
}

// Business-asset dispositions of one business, summed (whole dollars): each
// sale, condemnation or scrapping, and the partial disposition of a building.
pub(crate) fn business_assets(
    assets: &[BusinessAsset],
    mid_quarter: bool,
    amt: bool,
) -> Option<crate::farm_land::LandGain> {
    business_assets_source(assets, mid_quarter, amt, None)
}
fn business_assets_source(
    assets: &[BusinessAsset],
    mid_quarter: bool,
    amt: bool,
    source: Option<&crate::nonmacrs::RentalRecoverySource<'_>>,
) -> Option<crate::farm_land::LandGain> {
    sum_business_asset_components(business_asset_components(assets, mid_quarter, amt, source))
}
fn sum_business_asset_components(
    components: impl IntoIterator<Item = (FarmRentalPropertyRow, crate::farm_land::LandGain)>,
) -> Option<crate::farm_land::LandGain> {
    let mut total: Option<crate::farm_land::LandGain> = None;
    for (_, value) in components {
        let t = total.get_or_insert(crate::farm_land::LandGain::zero());
        t.gain += value.gain;
        t.recapture += value.recapture;
        t.ordinary += value.ordinary;
        t.section1231 += value.section1231;
        t.unrecaptured += value.unrecaptured;
    }
    total
}

// The same recognized asset components feed the native total and the
// Form4835 row allocation. Each component retains its actual native row identity.
fn business_asset_components<'a>(
    assets: &'a [BusinessAsset],
    mid_quarter: bool,
    amt: bool,
    source: Option<&'a crate::nonmacrs::RentalRecoverySource<'a>>,
) -> impl Iterator<Item = (FarmRentalPropertyRow, crate::farm_land::LandGain)> + 'a {
    assets.iter().enumerate().flat_map(move |(index, asset)| {
        asset_components(
            index,
            rental_asset_sale(asset, index, mid_quarter, amt, source),
            asset.partial_for(),
        )
    })
}
fn asset_components(
    index: usize,
    sale: Option<AssetGain>,
    partial: Option<PartialGain>,
) -> impl Iterator<Item = (FarmRentalPropertyRow, crate::farm_land::LandGain)> {
    sale.map(|value| {
        (
            FarmRentalPropertyRow::Asset(index),
            crate::farm_land::LandGain {
                gain: value.recognized(),
                percent: 0,
                recapture: value.recapture,
                ordinary: value.ordinary,
                section1231: value.section1231,
                unrecaptured: value.unrecaptured,
            },
        )
    })
    .into_iter()
    .chain(partial.map(|part| (FarmRentalPropertyRow::Partial(index), part.land_gain())))
}

pub(crate) fn purchase_asset_components_with_claims(
    draft: &FederalDraft,
    activity: i64,
    assets: &[BusinessAsset],
    mid_quarter: bool,
    alternative: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,

    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> Result<Vec<(FarmRentalPropertyRow, crate::farm_land::LandGain)>, crate::Section179PreparedIssue>
{
    Ok(purchase_asset_packet_with_claims(
        draft,
        activity,
        assets,
        mid_quarter,
        alternative,
        source_claims,
        macrs,
    )?
    .0)
}

type PurchaseAssetPacket =
    (Vec<(FarmRentalPropertyRow, crate::farm_land::LandGain)>, Vec<Option<AssetGain>>);

fn purchase_asset_packet_with_claims(
    draft: &FederalDraft,
    activity: i64,
    assets: &[BusinessAsset],
    mid_quarter: bool,
    alternative: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,

    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> Result<PurchaseAssetPacket, crate::Section179PreparedIssue> {
    let mut components = Vec::new();
    let mut reports = Vec::new();
    for (index, asset) in assets.iter().enumerate() {
        let sale = crate::cooperative_basis::sale_with_claims(
            draft,
            activity,
            asset,
            mid_quarter,
            alternative,
            source_claims,
            macrs,
        )?;
        let partial = crate::cooperative_basis::partial_with_claims(
            draft,
            activity,
            asset,
            mid_quarter,
            alternative,
            source_claims,
            macrs,
        )?;
        reports.push(sale);
        components.extend(asset_components(index, sale, partial));
    }
    Ok((components, reports))
}

pub(crate) fn purchase_asset_gross_with_claims(
    draft: &FederalDraft,
    activity: i64,
    assets: &[BusinessAsset],
    mid_quarter: bool,
    alternative: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,

    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> Result<i64, crate::Section179PreparedIssue> {
    crate::kernel::ratio::checked_sum(
        purchase_asset_components_with_claims(
            draft,
            activity,
            assets,
            mid_quarter,
            alternative,
            source_claims,
            macrs,
        )?
        .iter()
        .map(|(_, component)| phase::floor(component.gain, 0)),
    )
    .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))
}

pub(crate) fn property_asset_sale(
    d: &FederalDraft,
    id: i64,
    asset: &BusinessAsset,
    index: usize,
    mid_quarter: bool,
    amt: bool,
) -> Option<AssetGain> {
    rental_asset_sale(
        asset,
        index,
        mid_quarter,
        amt,
        crate::nonmacrs::rental_recovery_source(d, id, true).as_ref(),
    )
}
fn business_asset_gross_with_source(
    assets: &[BusinessAsset],
    mid_quarter: bool,
    amt: bool,
    source: Option<&crate::nonmacrs::RentalRecoverySource<'_>>,
) -> i64 {
    assets
        .iter()
        .enumerate()
        .map(|(index, asset)| {
            rental_asset_sale(asset, index, mid_quarter, amt, source)
                .map_or(0, |sale| phase::floor(sale.recognized(), 0))
                + asset.partial_for().map_or(0, |part| phase::floor(part.gain, 0))
        })
        .sum()
}

// One native recognized positive component, shared by physical and filer views.
pub(crate) fn business_asset_gross_components(
    assets: &[BusinessAsset],
    mid_quarter: bool,
    amt: bool,
) -> impl Iterator<Item = i64> + '_ {
    assets.iter().flat_map(move |asset| {
        asset
            .sale_for(mid_quarter, amt)
            .into_iter()
            .map(|sale| phase::floor(sale.recognized(), 0))
            .chain(asset.partial_for().map(|part| phase::floor(part.gain, 0)))
    })
}

// Pub. 225, chapter 15, Percentage From Farming, includes machinery gains;
// Pub. 505 includes draft, breeding, dairy and sporting livestock gains.
// Reuse the native recognized gain, including recapture and current installment
// income. Land has a separate source and does not enter this numerator.
// Physical Business sources receive their canonical community gain share once;
// FarmRental assets already carry their divided source amounts.
#[cfg(test)]
pub(crate) fn farming_property_gross(
    d: &FederalDraft,
) -> Result<Result<i64, &'static str>, crate::Section179PreparedIssue> {
    let source_claims = crate::related_forms::effective_claimants(d);
    farming_property_gross_with_claims(d, &source_claims)
}

pub(crate) fn farming_property_gross_with_claims(
    d: &FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Result<i64, &'static str>, crate::Section179PreparedIssue> {
    let mid_quarter = crate::depreciation::mid_quarter(d);
    let mut gross = 0_i64;
    for entry in &crate::community::property_entries(d) {
        let (assets, farm, share) = match &entry.income {
            FederalIncome::Business { data } => (
                &data.assets,
                matches!(data.activity, crate::BusinessActivity::Farming { .. }),
                d.setup.as_ref().and_then(|setup| {
                    crate::community::share(
                        setup,
                        entry.owner.owned(),
                        entry.community,
                        &entry.income,
                    )
                }),
            ),
            FederalIncome::FarmRental { data } => (&data.assets, true, None),
            _ => continue,
        };
        for asset in assets {
            if farm
                && asset.partial.is_some()
                && crate::cooperative_basis::partial_with_claims(
                    d,
                    entry.id,
                    asset,
                    mid_quarter,
                    false,
                    source_claims,
                    None,
                )?
                .is_some_and(|part| part.gain > 0)
            {
                return Ok(Err("ISSUE_FARMER_GROSS_SOURCE_UNAVAILABLE"));
            }
            let Some(sale) = crate::cooperative_basis::sale_with_claims(
                d,
                entry.id,
                asset,
                mid_quarter,
                false,
                source_claims,
                None,
            )?
            else {
                continue;
            };
            if !sale.livestock {
                if !farm {
                    continue;
                }
                // The native owner establishes section 1245 machinery separately
                // from section 1250 real property. A positive real-property gain
                // needs its actual farming character; its recovery class alone
                // cannot establish that the gain is excluded from this numerator.
                if sale.section1250 && sale.recognized() > 0 {
                    return Ok(Err("ISSUE_FARMER_GROSS_SOURCE_UNAVAILABLE"));
                }
            }
            let Some(next) =
                phase::floor(sale.recognized(), 0).checked_mul(100).and_then(|amount| {
                    gross.checked_add(share.map_or(amount, |share| share.gain(amount)))
                })
            else {
                return Ok(Err("ISSUE_FARMER_GROSS_AMOUNT_UNAVAILABLE"));
            };
            gross = next;
        }
    }
    Ok(Ok(gross))
}

// Section 175 uses agricultural production gross, unlike section 6654's
// broader farmer test. Draft, breeding and dairy livestock gains qualify;
// sporting livestock, machinery and land gains do not (Reg. 1.175-5(a)(2)).
// Read recognized gain from the canonical sale owner, not proceeds or net1231.
pub(crate) fn conservation_livestock_gross_with_claims(
    d: &FederalDraft,
    alternative: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<alloc::collections::BTreeMap<i64, i64>, crate::IssueContext> {
    physical_conservation_livestock_gross_with_claims(d, alternative, source_claims).map(|values| {
        values
            .into_iter()
            .map(|(id, amount)| {
                let business = d.income.iter().any(|entry| {
                    entry.id == id && matches!(entry.income, FederalIncome::Business { .. })
                });
                (
                    id,
                    if business {
                        crate::community::SourceAmount::property(d, id, amount).filer
                    } else {
                        amount
                    },
                )
            })
            .collect()
    })
}

pub(crate) fn physical_conservation_livestock_gross_with_claims(
    d: &FederalDraft,
    alternative: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<alloc::collections::BTreeMap<i64, i64>, crate::IssueContext> {
    let mid_quarter = crate::depreciation::mid_quarter(d);
    let macrs = crate::macrs_source::MacrsElectionView::new(d);
    let mut totals = alloc::collections::BTreeMap::new();
    for entry in crate::community::property_entries(d) {
        let assets = match &entry.income {
            FederalIncome::Business { data }
                if matches!(data.activity, crate::BusinessActivity::Farming { .. }) =>
            {
                &data.assets
            }
            FederalIncome::FarmRental { data } => &data.assets,
            _ => continue,
        };
        let mut gross = 0_i64;
        for (index, asset) in assets.iter().enumerate() {
            let crate::AssetDisposal::Disposed { how, .. } = &asset.disposal else {
                continue;
            };
            let livestock = match how {
                DisposalKind::Sold { livestock: SoldLivestock::No, .. }
                | DisposalKind::Sold {
                    livestock:
                        SoldLivestock::CattleOrHorse { purpose: Some(LivestockPurpose::Sport) }
                        | SoldLivestock::Other { purpose: Some(LivestockPurpose::Sport) },
                    ..
                }
                | DisposalKind::Scrapped
                | DisposalKind::PersonalUse => continue,
                DisposalKind::Sold { livestock, .. } => Some(*livestock),
                DisposalKind::Condemned { .. } => None,
            };
            let issue = || {
                crate::interview::income_issue_context(&entry)
                    .item("assets", index)
                    .corrective(crate::TaxSection::Farming)
            };
            let Some(sale) = crate::cooperative_basis::sale_with_claims(
                d,
                entry.id,
                asset,
                mid_quarter,
                alternative,
                source_claims,
                Some(&macrs),
            )
            .map_err(|_| issue())?
            else {
                return Err(issue());
            };
            let gain = sale.recognized();
            if gain <= 0 {
                continue;
            }
            // These transactions need their own production-gross source fitting.
            if asset.partial.is_some()
                || asset.nonmacrs()
                || !matches!(
                    how,
                    DisposalKind::Sold {
                        payment: crate::SalePayment::InFull,
                        buyer: Buyer::Unrelated,
                        ..
                    }
                )
            {
                return Err(issue());
            }
            let purpose = livestock.ok_or_else(issue)?.purpose().require(issue)?;
            if purpose.is_none_or(|purpose| purpose == LivestockPurpose::Sport) {
                continue;
            }
            let amount = crate::extended_rules::annual_dollars_to_working_cents(gain)
                .map_err(|_| issue())?;
            gross = gross.checked_add(amount).ok_or_else(issue)?;
        }
        totals.insert(entry.id, gross);
    }
    Ok(totals)
}

// Positive recognized components only. Risk/PAL restrictions remove losses;
// callers subtract only the property net their post-limit owners included.
pub(crate) fn rental_property_gross(d: &FederalDraft, amt: bool) -> i64 {
    let mid_quarter = crate::depreciation::mid_quarter(d);
    let property_entries = crate::community::property_entries(d);
    property_entries
        .iter()
        .map(|entry| {
            let recovery_source = crate::nonmacrs::rental_recovery_source(d, entry.id, true);
            let source = recovery_source.as_ref();
            match &entry.income {
                FederalIncome::Rental { data } if rental_sale_refusal(data, source).is_none() => {
                    rental_disposition(data, amt).ok().flatten().map_or(0, |sale| sale.gross)
                        + rental_partials(data, amt, source)
                            .ok()
                            .unwrap_or_default()
                            .iter()
                            .map(|(_, part)| phase::floor(part.gain, 0))
                            .sum::<i64>()
                        + business_asset_gross_with_source(&data.assets, mid_quarter, amt, source)
                }
                FederalIncome::NonbusinessProperty {
                    kind: crate::NonbusinessPropertyKind::PersonalRental { assets, .. },
                    ..
                } => assets
                    .iter()
                    .filter_map(|asset| asset.sale_for(mid_quarter, amt))
                    .map(|sale| equipment_ordinary(&sale))
                    .sum(),
                _ => 0,
            }
        })
        .sum()
}

// The synthetic source id of the dispositions of a Form 2106 vehicle: never an
// income entry's, like the earlier-year installment and exchange sources.
pub(crate) fn employee_source_id(form: usize) -> i64 {
    -2001 - form as i64
}

// Every direct business-property source by its business: farm-land sales
// and business-asset dispositions (never rentals, which are passive), and the
// sales of the actual-expense vehicles of each Form 2106 (Pub 463: Form 4797,
// no self-employment tax and no qualified business income).
#[cfg(test)]
pub(crate) fn direct_sources(
    draft: &FederalDraft,
    amt: bool,
) -> Vec<(i64, crate::farm_land::LandGain)> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    direct_sources_with_claims(draft, amt, &source_claims)
}

pub(crate) fn direct_sources_with_claims(
    draft: &FederalDraft,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Vec<(i64, crate::farm_land::LandGain)> {
    direct_source_scope_with_claims(draft, amt, false, None, source_claims, None, None).0
}

// Same native property source, with its actual admission errors preserved.
// This accessor supplies the Form4835 risk stage, not a second gain engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FarmRentalPropertyRow {
    Land(usize),
    Asset(usize),
    Partial(usize),
    RentalBuilding,
    RentalLand,
    RentalShort(usize),
    RentalPartial(usize),
    RentalCasualty(usize),
    RentalCombined,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct FarmRentalPropertyRows {
    pub partial_gains: Vec<(usize, i64)>,
    pub character_rows: Vec<(FarmRentalPropertyRow, usize, i64)>,
    pub prior_section1231_allowed: i64,
}

pub(crate) struct FarmRentalPropertySource {
    pub value: Option<crate::farm_land::LandGain>,
    components: Vec<(FarmRentalPropertyRow, crate::farm_land::LandGain)>,
}

impl FarmRentalPropertySource {
    // Unchanged native rows stay in components; character_rows contains
    // only actual risk/PAL overrides, including a fully suspended zero.
    pub(crate) fn rental_casualty_income(
        &self,
        rows: &FarmRentalPropertyRows,
    ) -> Result<i64, crate::activity_risk::ProjectionIssue> {
        crate::kernel::ratio::checked_sum(
            self.components
                .iter()
                .filter(|(row, _)| matches!(row, FarmRentalPropertyRow::RentalCasualty(_)))
                .map(|(row, value)| {
                    rows.character_rows
                        .iter()
                        .find(|(source, class, _)| source == row && *class == 0)
                        .map_or(value.ordinary, |(_, _, amount)| *amount)
                }),
        )
        .ok_or(crate::activity_risk::ProjectionIssue::Arithmetic(
            crate::kernel::loss_limits::LossLimitError::Overflow,
        ))
    }

    // Use the existing rental row allocator over all actual negative native
    // components of each character, including the separately authored prior
    // section1231 loss. Positive rows never absorb a suspended loss.
    pub(crate) fn allowed_rows(
        &self,
        allowed: &crate::farm_land::LandGain,
        prior: i64,
    ) -> Result<FarmRentalPropertyRows, crate::activity_risk::ProjectionIssue> {
        let overflow = || {
            crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Overflow,
            )
        };
        let mut rows: Vec<_> = self
            .components
            .iter()
            .filter_map(|(row, value)| match row {
                FarmRentalPropertyRow::Partial(index) => Some((*index, value.gain)),
                _ => None,
            })
            .collect();
        let mut prior_section1231_allowed = prior;
        let mut character_rows = Vec::new();
        for class in 0..2 {
            let amount = |v: &crate::farm_land::LandGain| {
                if class == 0 { v.ordinary } else { v.section1231 }
            };
            let prior = if class == 1 { prior } else { 0 };
            let raw = crate::kernel::ratio::checked_sum(
                self.components
                    .iter()
                    .map(|(_, value)| amount(value))
                    .chain(core::iter::once(-prior)),
            )
            .ok_or_else(overflow)?;
            let cut = amount(allowed).checked_sub(raw).ok_or_else(overflow)?;
            let mut weights = self
                .components
                .iter()
                .map(|(_, value)| phase::checked_loss(amount(value)).ok_or_else(overflow))
                .collect::<Result<Vec<_>, _>>()?;
            weights.push(prior);
            let total =
                crate::kernel::ratio::checked_sum(weights.iter().copied()).ok_or_else(overflow)?;
            if !(0..=total).contains(&cut) {
                return Err(crate::activity_risk::ProjectionIssue::Arithmetic(
                    crate::kernel::loss_limits::LossLimitError::Footing,
                ));
            }
            let allocated = crate::kernel::ratio::allocate_remaining(cut, &weights);
            if class == 1 {
                prior_section1231_allowed -= allocated[self.components.len()];
            }
            for ((row, value), share) in self.components.iter().zip(allocated) {
                if let FarmRentalPropertyRow::Partial(index) = row {
                    let row = rows
                        .iter_mut()
                        .find(|(row, _)| row == index)
                        .expect("actual partial component");
                    row.1 = row.1.checked_add(share).ok_or_else(overflow)?;
                } else if share != 0 {
                    character_rows.push((
                        *row,
                        class,
                        amount(value).checked_add(share).ok_or_else(overflow)?,
                    ));
                }
            }
        }
        Ok(FarmRentalPropertyRows {
            partial_gains: rows,
            character_rows,
            prior_section1231_allowed,
        })
    }
}

pub(crate) fn farm_rental_property_source_with_claims(
    d: &FederalDraft,
    id: i64,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,

    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> Result<FarmRentalPropertySource, crate::activity_risk::SourceIssue> {
    let issue = |key| {
        crate::activity_risk::SourceIssue::new(
            id,
            crate::activity_risk::ProjectionIssue::Source(key),
        )
    };
    let entries = crate::community::property_entries(d);
    let entry = entries
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| issue("DIRECT_ACTIVITY_SOURCE_UNAVAILABLE"))?;
    let FederalIncome::FarmRental { data } = &entry.income else {
        return Err(issue("DIRECT_ACTIVITY_SOURCE_UNAVAILABLE"));
    };
    let recovery_source = crate::nonmacrs::rental_recovery_source_with_macrs(d, id, true, macrs);
    for (index, asset) in data.assets.iter().enumerate() {
        let asset = recovery_source.as_ref().map_or(asset, |source| source.asset(index, asset));
        if let Some(key) = recovery_source.as_ref().and_then(|source| source.history_issue(asset)) {
            return Err(issue(key));
        }
        if asset.nonmacrs() && (!asset.valid() || asset.nonmacrs_issue().is_some()) {
            return Err(issue(asset.nonmacrs_issue().unwrap_or("ASSET_NONMACRS_ORIGINAL_HISTORY")));
        }
        if !asset.disposal.valid() {
            return Err(issue(
                "ISSUE_ENTER_A_RETURN_YEAR_DATE_SOLD_OR_STOPPED_USING_THE_SALE_PRICE_AND_SELLING_A990EE59",
            ));
        }
        if let Some((key, _)) = asset.disposal_refusal_in_context(
            recovery_source
                .as_ref()
                .map_or_else(crate::macrs_source::AdsInvocation::default, |source| {
                    source.context(asset)
                }),
        ) {
            return Err(issue(key));
        }
    }
    let (land, error) = crate::farm_land::source_components_with_claims(
        d,
        id,
        &data.farm,
        amt,
        true,
        source_claims,
    );
    if let Some(key) = error {
        return Err(issue(key));
    }
    let mut components: Vec<_> = land
        .into_iter()
        .map(|(index, value)| (FarmRentalPropertyRow::Land(index), value))
        .collect();
    components.extend(
        purchase_asset_components_with_claims(
            d,
            id,
            &data.assets,
            crate::depreciation::mid_quarter(d),
            amt,
            source_claims,
            macrs,
        )
        .map_err(|source| match source {
            crate::Section179PreparedIssue::AmountOverflow(_)
            | crate::Section179PreparedIssue::AggregateAmountOverflow => {
                crate::activity_risk::SourceIssue::new(
                    id,
                    crate::activity_risk::ProjectionIssue::Arithmetic(
                        crate::kernel::loss_limits::LossLimitError::Overflow,
                    ),
                )
            }
            crate::Section179PreparedIssue::SourceBoundaryUnavailable(_, key) => issue(key),
            _ => issue("ISSUE_COOP_PROPERTY_HISTORY_NEEDED"),
        })?,
    );
    let sum = |field: fn(&crate::farm_land::LandGain) -> i64| {
        crate::kernel::ratio::checked_sum(components.iter().map(|(_, value)| field(value)))
            .ok_or_else(|| {
                crate::activity_risk::SourceIssue::new(
                    id,
                    crate::activity_risk::ProjectionIssue::Arithmetic(
                        crate::kernel::loss_limits::LossLimitError::Overflow,
                    ),
                )
            })
    };
    let value = if components.is_empty() {
        None
    } else {
        Some(crate::farm_land::LandGain {
            gain: sum(|v| v.gain)?,
            percent: 0,
            recapture: sum(|v| v.recapture)?,
            ordinary: sum(|v| v.ordinary)?,
            section1231: sum(|v| v.section1231)?,
            unrecaptured: sum(|v| v.unrecaptured)?,
        })
    };
    Ok(FarmRentalPropertySource { value, components })
}

// Before-risk C/F preallocation keeps direct_sources. Final consumers use
// the same invocation's allowed current ScheduleC or Form4835 property rows.

pub(crate) fn physical_allowed_direct_source_packet_with_claims(
    d: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<DirectSourcePacket, crate::activity_risk::SourceIssue> {
    if let Some(issue) = &pool.source_issue {
        return Err(issue.clone());
    }
    let cached = pool.direct_business_property.keys().copied().collect();
    let (mut sources, issue, parts, partial) = direct_source_scope_with_claims(
        d,
        amt,
        false,
        None,
        source_claims,
        pool.macrs.as_deref(),
        Some(&cached),
    );
    let native_issue = issue.map(|(id, key)| {
        let mut issue = crate::activity_risk::SourceIssue::new(
            id,
            crate::activity_risk::ProjectionIssue::Source(key),
        );
        if let Some(adjustments) = &d.adjustments {
            issue.context = adjustments
                .employee_expenses
                .iter()
                .enumerate()
                .find(|(position, _)| employee_source_id(*position) == id)
                .map(|(position, claim)| alloc::boxed::Box::new(claim.context(position)));
        }
        issue
    });
    for (id, value) in &mut sources {
        if d.income.iter().any(|entry| {
            entry.id == *id && matches!(entry.income, FederalIncome::FarmRental { .. })
        }) {
            *value = *pool.farm_rental_property.get(id).ok_or_else(|| {
                crate::activity_risk::SourceIssue::new(
                    *id,
                    crate::activity_risk::ProjectionIssue::Source(
                        "DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE",
                    ),
                )
            })?;
        }
    }
    for (&id, value) in &pool.farm_rental_property {
        if !sources.iter().any(|(source, _)| *source == id) {
            sources.push((id, *value));
        }
    }
    Ok((sources, parts, partial, native_issue))
}

// Financial consumers share the attribution owner's same-invocation projection.
pub(crate) fn allowed_direct_sources_with_claims(
    d: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Vec<(i64, crate::community::PropertyCents)>, crate::activity_risk::SourceIssue> {
    Ok(crate::community::source_projection(d, pool, amt, source_claims)?.cent_direct())
}

// Accepted identities distinguish simultaneous asset sales and partial dispositions.
// Amounts retain their owner's annual-Dollar boundary, before return-wide1231
// netting/lookback. Recapture and unrecaptured amounts are subsets, not additions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DirectBusinessPropertyId {
    Asset(i64),
    Partial(i64),
    Land { sale: i64, parcel: i64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DirectBusinessPropertyPart {
    pub id: DirectBusinessPropertyId,
    pub dollars: crate::farm_land::LandGain,
    pub report: Option<AssetGain>,
}

pub(crate) type DirectBusinessPropertyParts =
    alloc::collections::BTreeMap<i64, Result<Vec<DirectBusinessPropertyPart>, &'static str>>;

pub(crate) fn checked_direct_business_property_sources_with_claims(
    draft: &FederalDraft,
    amt: bool,
    sources: Option<&alloc::collections::BTreeSet<i64>>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,

    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> Result<(DirectSourceValues, DirectBusinessPropertyParts), DirectSourceIssue> {
    let (values, issue, parts, _) =
        direct_source_scope_with_claims(draft, amt, true, sources, source_claims, macrs, None);
    issue.map_or(Ok((values, parts)), Err)
}

type DirectSourceValues = Vec<(i64, crate::farm_land::LandGain)>;
type DirectSourcePacket = (
    DirectSourceValues,
    DirectBusinessPropertyParts,
    Vec<(i64, i64)>,
    Option<crate::activity_risk::SourceIssue>,
);
type DirectSourceIssue = (i64, &'static str);

fn direct_source_scope_with_claims(
    draft: &FederalDraft,
    amt: bool,
    business_only: bool,
    source: Option<&alloc::collections::BTreeSet<i64>>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,

    macrs: Option<&crate::macrs_source::MacrsElectionView>,
    cached: Option<&alloc::collections::BTreeSet<i64>>,
) -> (DirectSourceValues, Option<DirectSourceIssue>, DirectBusinessPropertyParts, Vec<(i64, i64)>) {
    let fallback = macrs.is_none().then(|| crate::macrs_source::MacrsElectionView::new(draft));
    let source_view = macrs.or(fallback.as_ref()).unwrap();
    let property_entries = crate::community::property_entries(draft);
    let land = crate::farm_land::source_scopes_with_claims(draft, amt, source_claims);
    let mid_quarter = crate::depreciation::mid_quarter_with_macrs(draft, Some(source_view));
    let mut result = Vec::new();
    let mut issue = None;
    let mut property_parts = alloc::collections::BTreeMap::new();
    let mut partial_gains = Vec::new();
    for entry in property_entries.iter().filter(|entry| {
        source.is_none_or(|ids| ids.contains(&entry.id))
            && cached.is_none_or(|ids| !ids.contains(&entry.id))
    }) {
        // A farm rental's property sales are direct property income like a
        // farm's: the gain stays out of its passive pool (Reg. 1.469-2T(c)(2)
        // would make it passive income, so this can only raise tax).
        if business_only
            && !matches!(&entry.income,
            FederalIncome::Business { data } if matches!(data.activity,
                crate::BusinessActivity::Ordinary { .. } | crate::BusinessActivity::Farming { .. }))
        {
            continue;
        }
        let assets = match &entry.income {
            FederalIncome::Business { data } => &data.assets,
            FederalIncome::FarmRental { data } => &data.assets,
            _ => continue,
        };
        if business_only {
            for asset in assets {
                if !asset.disposal.valid() {
                    issue.get_or_insert((entry.id,
                        "ISSUE_ENTER_A_RETURN_YEAR_DATE_SOLD_OR_STOPPED_USING_THE_SALE_PRICE_AND_SELLING_A990EE59"));
                } else if let Some((key, _)) = asset.disposal_refusal_in_context(
                    source_view.context(entry.owner.owned(), Some(entry.id), asset),
                ) {
                    issue.get_or_insert((entry.id, key));
                }
            }
        }
        let land_source = land.iter().find(|(id, _, _)| *id == entry.id);
        if let Some((_, _, Some(key))) = land_source {
            issue.get_or_insert((entry.id, *key));
        }
        let own_land = land_source.and_then(|(_, value, _)| *value);
        let recovery_source = crate::nonmacrs::rental_recovery_source(draft, entry.id, true);
        for (index, asset) in assets.iter().enumerate() {
            let actual =
                recovery_source.as_ref().map_or(asset, |source| source.asset(index, asset));
            if actual.nonmacrs() {
                if !actual.valid() {
                    issue.get_or_insert((entry.id, "ASSET_NONMACRS_ORIGINAL_HISTORY"));
                } else if let Some(key) = actual.nonmacrs_issue() {
                    issue.get_or_insert((entry.id, key));
                }
            }
        }
        let (components, reports) = match purchase_asset_packet_with_claims(
            draft,
            entry.id,
            assets,
            mid_quarter,
            amt,
            source_claims,
            macrs,
        ) {
            Ok(components) => components,
            Err(source) => {
                issue.get_or_insert((
                    entry.id,
                    match source {
                        crate::Section179PreparedIssue::SourceBoundaryUnavailable(_, key) => key,
                        _ => "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
                    },
                ));
                continue;
            }
        };
        // Anonymous native partial amounts suffice for additive report totals;
        // accepted identity remains separately required by ordered consumers.
        if matches!(entry.income, FederalIncome::Business { .. }) {
            partial_gains.extend(components.iter().filter_map(|(row, value)| {
                matches!(row, FarmRentalPropertyRow::Partial(_)).then_some((entry.id, value.gain))
            }));
        }
        let mut parts = Vec::new();
        let mut parts_issue = None;
        if matches!(entry.income, FederalIncome::Business { .. }) {
            if let FederalIncome::Business { data } = &entry.income
                && let crate::BusinessActivity::Farming { data: farm } = &data.activity
            {
                let (land_parts, source_issue) = crate::farm_land::source_components_with_claims(
                    draft,
                    entry.id,
                    farm,
                    amt,
                    true,
                    source_claims,
                );
                parts_issue = source_issue;
                for (index, dollars) in land_parts {
                    let sale = &farm.land_sales[index];
                    if sale.id <= 0 || sale.basis_property <= 0 {
                        parts_issue.get_or_insert("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED");
                    } else {
                        parts.push(DirectBusinessPropertyPart {
                            id: DirectBusinessPropertyId::Land {
                                sale: sale.id,
                                parcel: sale.basis_property,
                            },
                            dollars,
                            report: None,
                        });
                    }
                }
            }
            for (row, dollars) in &components {
                let (index, partial) = match row {
                    FarmRentalPropertyRow::Asset(index) => (*index, false),
                    FarmRentalPropertyRow::Partial(index) => (*index, true),
                    _ => unreachable!(),
                };
                let id = assets[index].id;
                if id <= 0 {
                    parts_issue.get_or_insert("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED");
                } else {
                    parts.push(DirectBusinessPropertyPart {
                        id: if partial {
                            DirectBusinessPropertyId::Partial(id)
                        } else {
                            DirectBusinessPropertyId::Asset(id)
                        },
                        dollars: *dollars,
                        report: (!partial).then_some(reports[index]).flatten(),
                    });
                }
            }
        }
        let mut own_assets = sum_business_asset_components(components);
        if let FederalIncome::Business { data } = &entry.income {
            match crate::cohorts::property_total(data) {
                Ok(Some(value)) => {
                    // These cost records do not yet carry accepted property IDs.
                    // Their native total remains valid; an ordered loss consumer
                    // cannot infer a durable component identity from its position.
                    parts_issue.get_or_insert("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED");
                    let own = own_assets.get_or_insert(crate::farm_land::LandGain::zero());
                    match checked_property_sum(*own, value) {
                        Some(value) => *own = value,
                        None => {
                            issue.get_or_insert((entry.id, "AMORTIZATION_TOTAL_UNREPRESENTABLE"));
                            continue;
                        }
                    }
                }
                Ok(None) => (),
                Err(key) => {
                    issue.get_or_insert((entry.id, key));
                }
            }
        }
        if matches!(entry.income, FederalIncome::Business { .. }) {
            let mut identities = alloc::collections::BTreeSet::new();
            if parts.iter().any(|part| !identities.insert(part.id)) {
                parts_issue.get_or_insert("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED");
            }
            property_parts.insert(entry.id, parts_issue.map_or(Ok(parts), Err));
        }
        let value = match (own_land, own_assets) {
            (None, None) => continue,
            (Some(v), None) | (None, Some(v)) => v,
            (Some(a), Some(b)) => match checked_property_sum(a, b) {
                Some(value) => value,
                None => {
                    issue.get_or_insert((entry.id, "AMORTIZATION_TOTAL_UNREPRESENTABLE"));
                    continue;
                }
            },
        };
        result.push((entry.id, value));
    }
    if let Some(adjustments) =
        draft.adjustments.as_ref().filter(|_| !business_only && source.is_none())
    {
        for (position, claim) in adjustments.employee_expenses.iter().enumerate() {
            let mut assets = claim.assets();
            if let Some((_, key)) = claim.equipment_issue(mid_quarter, Some(source_view)) {
                issue.get_or_insert((employee_source_id(position), key));
            } else {
                assets.extend(claim.equipment_assets.iter().cloned());
            }
            let components = assets.iter().enumerate().flat_map(|(index, asset)| {
                asset_components(
                    index,
                    asset.sale_in_context(
                        mid_quarter,
                        amt,
                        source_view.context(claim.owner, None, asset),
                    ),
                    asset.partial_for(),
                )
            });
            if let Some(value) = sum_business_asset_components(components) {
                result.push((employee_source_id(position), value));
            }
        }
    }
    (result, issue, property_parts, partial_gains)
}

// One shed or replaced-in-part building row held one year or less, sold in a
// whole-property sale: its allocated price, cost and allowed depreciation
// (Part II line 11, whole dollars).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShortRow {
    pub index: usize,
    pub price: i64,
    pub basis: i64,
    pub depreciation: i64,
    pub gain: i64,
}

// One sold or condemned rental property's Form 4797 amounts, whole dollars:
// the buildings and improvements held more than a year in Part III (lines
// 20-24), the land in Part I, the rows held a year or less in Part II.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RentalGain {
    pub realized: i64,
    pub basis: i64,
    pub depreciation: i64,
    pub building: i64,
    pub land_realized: i64,
    pub land_basis: i64,
    pub land: i64,
    // Recognized in 2026: after an installment sale, the part in its
    // payments (Form 6252 line 26), unrecaptured section 1250 gain first.
    pub unrecaptured: i64,
    pub recognized: i64,
    // Positive recognized property components before losses (Form1116).
    pub gross: i64,
    // The building's section 1250 additional depreciation recaptured as
    // ordinary income: the section 50(c) reductions of rehabilitation credits
    // count as depreciation (Form 4797 line 28a).
    pub additional: i64,
    pub ordinary: i64,
    pub installment: Option<crate::kernel::installment::Schedule>,
    // The rows held one year or less and the ordinary Part II line 11 amount
    // they and a controlled buyer's building gain make (included in
    // `recognized`).
    pub short_rows: alloc::vec::Vec<ShortRow>,
    // Actual recognized components, captured where installment, condemnation
    // and related-party recognition are resolved, before activity limits.
    pub components: Vec<(FarmRentalPropertyRow, crate::farm_land::LandGain)>,
    pub line11: i64,
    // The gain a condemnation's replacement property postpones.
    pub postponed: i64,
    // The personal-use part of a sale (section 280A home use): its proceeds,
    // cost and gain or loss (a loss is not deductible).
    pub personal_proceeds: i64,
    pub personal_basis: i64,
    pub personal: i64,
    // Losses section 267 disallows on a sale to a related buyer.
    pub disallowed: i64,
    // Whether the building gain went to Part II line 11 under section 1239.
    pub section1239: bool,
    // The building (Part III) and land (line 2) results the return takes, after
    // the replacement property of a condemnation and the losses section 267
    // disallows.
    pub building_recognized: i64,
    pub land_recognized: i64,
}
// Depreciation allowed or allowable on a building from its service year
// through the date of sale (the 2026 amount by the mid-month convention).
pub(crate) fn accumulated(
    b: &crate::RentalBuilding,
    amt: bool,
    property_type: &crate::RentalPropertyType,
    sold: u32,
) -> Result<i64, (&'static str, &'static str)> {
    let mut total = 0;
    for year in (b.in_service / 10000).max(1987)..2026 {
        total += crate::extended_rules::depreciation_for_year(
            b,
            amt,
            (property_type, &crate::RentalLocation::UnitedStates),
            year,
            None,
            crate::kernel::source_fact::SourceFact::NotReported,
        )?;
    }
    Ok(total
        + crate::extended_rules::depreciation_for_year(
            b,
            amt,
            (property_type, &crate::RentalLocation::UnitedStates),
            2026,
            Some(sold),
            crate::kernel::source_fact::SourceFact::NotReported,
        )?)
}

// The depreciation allowed on the part of a building replaced or removed,
// cents: the part's share of the building's depreciation through the month of
// the disposition (Reg. 1.168(i)-8(f)(3)), the share of cost.
pub(crate) fn partial_depreciation(
    b: &crate::RentalBuilding,
    amt: bool,
    property_type: &crate::RentalPropertyType,
) -> Result<i64, (&'static str, &'static str)> {
    let Some(part) = &b.partial else {
        return Ok(0);
    };
    let whole = crate::RentalBuilding { partial: None, ..b.clone() };
    let allowed = accumulated(&whole, amt, property_type, part.date)?;
    Ok(fraction(allowed, part.cost_of_part, b.cost - b.land))
}

// The partial dispositions of a rental's building rows: the gain or loss of
// each part (Form 4797 line 2 when the row has been held more than a year,
// else line 11), with its row index.
// Remaining whole-building records do not establish a separately removed
// mixed-use component's business basis, depreciation or personal sale gain.
fn mixed_component_history_issue(v: &FederalRental) -> Option<(&'static str, &'static str)> {
    (matches!(v.sale, RentalSale::Sold { personal_share_percent, .. } if personal_share_percent != 0)
        && v.buildings.iter().any(|building| building.partial.is_some()))
    .then_some((
        "RENTAL_MIXED_COMPONENT_HISTORY_UNAVAILABLE",
        crate::fixture_english!("A separately disposed component of a mixed-use rental needs its own business basis, regular/AMT depreciation and personal-sale allocation. Whole-building sale records do not establish these component amounts."),
    ))
}

pub(crate) fn rental_partials(
    v: &FederalRental,
    amt: bool,
    source: Option<&crate::nonmacrs::RentalRecoverySource<'_>>,
) -> Result<alloc::vec::Vec<(usize, PartialGain)>, (&'static str, &'static str)> {
    if let Some(issue) = mixed_component_history_issue(v) {
        return Err(issue);
    }
    if v.location.foreign() && v.buildings.iter().any(|building| building.partial.is_some()) {
        crate::rental_foreign::source_admission(v, source)?;
    }
    let mut result = alloc::vec::Vec::new();
    for (index, b) in v.buildings.iter().enumerate() {
        let Some(part) = &b.partial else {
            continue;
        };
        let depreciation = dollars(partial_depreciation(b, amt, &v.property_type)?);
        let cost = dollars(part.cost_of_part);
        let proceeds = dollars(part.proceeds);
        let gain = proceeds - (cost - depreciation);
        let long = crate::dates::held_more_than_one_year(b.in_service, part.date);
        result.push((
            index,
            PartialGain {
                cost,
                proceeds,
                depreciation,
                gain,
                long,
                unrecaptured: if long && gain > 0 {
                    crate::kernel::recapture::section1250(gain, depreciation, 0, 0)
                        .unrecaptured1250_dollars
                } else {
                    0
                },
            },
        ));
    }
    Ok(result)
}

// A rental's whole-property disposition by sale or condemnation (the
// buildings and improvements in Part III, the rows held a year or less in
// Part II, the land in Part I). The sale divides the price, the expenses and
// the cost between the rental and the personal-use part (Pub 544, "Property
// Used Partly for Business or Rental"); the depreciation allowed or allowable
// is the rental part's alone.
pub(crate) fn rental_sale(
    v: &FederalRental,
    amt: bool,
) -> Result<Option<RentalGain>, (&'static str, &'static str)> {
    if let Some(issue) = mixed_component_history_issue(v) {
        return Err(issue);
    }
    let (date, price, land_price, selling_expenses, payment, buyer, personal, replacement) =
        match &v.sale {
            RentalSale::Sold {
                date,
                price,
                land_price,
                selling_expenses,
                payment,
                buyer,
                personal_share_percent,
            } => (
                *date,
                *price,
                *land_price,
                *selling_expenses,
                Some(payment),
                *buyer,
                i64::from(*personal_share_percent),
                None,
            ),
            RentalSale::Condemned { date, award, land_award, expenses, replacement_cost } => (
                *date,
                *award,
                *land_award,
                *expenses,
                None,
                Buyer::Unrelated,
                0,
                Some(*replacement_cost),
            ),
            RentalSale::No | RentalSale::Exchanged(_) => return Ok(None),
        };
    // The rental part of an amount (cents).
    let rental = |amount: i64| fraction(amount, 100 - personal, 100);
    let (price_r, land_price_r, expenses_r) =
        (rental(price), rental(land_price), rental(selling_expenses));
    let building_price = price_r - land_price_r;
    // Selling expenses follow the price allocation.
    let building_expenses =
        if price_r > 0 { fraction(expenses_r, building_price, price_r) } else { 0 };
    let land_expenses = expenses_r - building_expenses;
    // Each building row: its cost less land and less any part disposed of
    // earlier in 2026, and its depreciation allowed.
    struct Row {
        index: usize,
        weight: i64,
        depreciation: i64,
        additional: i64,
        short: bool,
        business_basis: i64,
    }
    let mut rows = alloc::vec::Vec::new();
    let mut land_basis = 0;
    let mut rental_building_basis = 0;
    // Section 50(c): a rehabilitation credit's basis reduction counts as
    // depreciation, less the basis a recapture restores (section 50(c)(2));
    // the straight line on the cost before it is not additional depreciation.
    for (index, b) in v.buildings.iter().enumerate() {
        let part = b.partial.as_ref().map_or(0, |p| p.cost_of_part);
        land_basis += b.land;
        let allowed = accumulated(b, amt, &v.property_type, date)?
            - partial_depreciation(b, amt, &v.property_type)?;
        let (business_basis, depreciation, mut additional) = if personal != 0 && b.cost > b.land {
            b.sale_history.as_ref()
                .and_then(|history| history.amounts(b.cost - b.land - part, amt, b.rehabilitation_reduction() > 0))
                .ok_or((
                    "RENTAL_BUILDING_SALE_HISTORY_UNAVAILABLE",
                    crate::fixture_english!("The sold rental's mixed-use building needs its actual business basis and regular/AMT business depreciation through the sale date."),
                ))?
        } else {
            (
                rental(b.cost - b.land - part),
                allowed + b.rehabilitation_net_reduction(Some(date)),
                0,
            )
        };
        rental_building_basis += business_basis;
        if personal == 0 && b.rehabilitation_reduction() > 0 {
            let unreduced =
                crate::RentalBuilding { rehabilitation: None, credit_reduction: 0, ..b.clone() };
            additional = allowed + b.rehabilitation_net_reduction(Some(date))
                - (accumulated(&unreduced, amt, &v.property_type, date)?
                    - partial_depreciation(&unreduced, amt, &v.property_type)?);
        }
        rows.push(Row {
            index,
            business_basis,
            weight: b.cost - b.land - part,
            depreciation,
            additional,
            short: !crate::dates::held_more_than_one_year(b.in_service, date),
        });
    }
    // The building part of the price is divided among the rows by cost only
    // when a row is held one year or less (a documented assumption); otherwise
    // the rows are one combined Part III property.
    let total_weight: i64 = rows.iter().map(|row| row.weight).sum();
    let split = total_weight > 0 && rows.iter().any(|row| row.short);
    let mut short_rows = alloc::vec::Vec::new();
    let (mut long_price, mut long_expenses) = (building_price, building_expenses);
    let (mut basis_cents, mut depreciation_cents, mut additional_cents) = (0, 0, 0);
    for row in &rows {
        if split && row.short {
            let share = fraction(building_price, row.weight, total_weight);
            let expense = fraction(building_expenses, row.weight, total_weight);
            long_price -= share;
            long_expenses -= expense;
            let basis = dollars(row.business_basis);
            let depreciation = dollars(row.depreciation);
            short_rows.push(ShortRow {
                index: row.index,
                price: dollars(share),
                basis,
                depreciation,
                gain: dollars(share - expense) - (basis - depreciation),
            });
        } else {
            basis_cents += row.business_basis;
            depreciation_cents += row.depreciation;
            additional_cents += row.additional;
        }
    }
    // Line 22 is the building's gross sales price and line 23 its cost plus
    // its share of the expense of sale; the gain is the same either way.
    let net_realized = dollars(long_price - long_expenses);
    let realized = dollars(long_price);
    let basis = dollars(basis_cents);
    let depreciation = dollars(depreciation_cents);
    let building = net_realized - (basis - depreciation);
    let additional = crate::kernel::ratio::positive(dollars(additional_cents));
    let land_realized = dollars(land_price_r - land_expenses);
    let land_basis = dollars(rental(land_basis));
    let land = land_realized - land_basis;
    // The personal-use part: a gain is a capital gain, a loss is not deductible.
    let land_total: i64 = v.buildings.iter().map(|b| b.land).sum();
    let all_cost = total_weight + land_total;
    let rental_cost = rental_building_basis + rental(land_total);
    let personal_proceeds = dollars(price - selling_expenses) - dollars(price_r - expenses_r);
    let personal_basis = dollars(all_cost - rental_cost);
    let personal_gain = if personal > 0 { personal_proceeds - personal_basis } else { 0 };
    // Section 1033: the gain a condemnation recognizes is limited to the
    // amount realized above the replacement property's cost, divided among the
    // gains in proportion to their size; the recapture is limited with it.
    let short_total: i64 = short_rows.iter().map(|row| row.gain).sum();
    let mut parts = [building, land, short_total];
    let mut postponed = 0;
    if let Some(spent) = replacement {
        let gain: i64 = parts.iter().sum();
        let positive: i64 = parts.iter().map(|part| phase::floor(*part, 0)).sum();
        if gain > 0 {
            let recognized = phase::cap(
                gain,
                [phase::floor(dollars(price_r - expenses_r) - dollars(rental(spent)), 0)],
            );
            postponed = gain - recognized;
            let mut left = recognized;
            let mut shares = [0i64; 3];
            for (share, part) in shares.iter_mut().zip(&parts) {
                if *part > 0 {
                    *share = fraction(recognized, *part, positive);
                    left -= *share;
                }
            }
            // The rounding remainder joins the largest gain.
            if let Some(largest) = (0..3).max_by_key(|i| parts[*i]) {
                shares[largest] += left;
            }
            for (part, share) in parts.iter_mut().zip(shares) {
                if *part > 0 {
                    *part = share;
                }
            }
        }
    }
    let [mut building_rec, mut land_rec, _] = parts;
    let mut disallowed = 0;
    let mut short_rec = alloc::vec::Vec::new();
    // Section 267(a)(1): a loss on a sale to a related buyer is not
    // deductible, on each property separately.
    let related = buyer != Buyer::Unrelated;
    if replacement.is_some() && postponed > 0 {
        // Each short row's recognized share follows its realized share.
        for row in &short_rows {
            short_rec.push(if short_total > 0 && row.gain > 0 {
                fraction(parts[2], row.gain, short_total.max(1))
            } else {
                row.gain
            });
        }
    } else {
        short_rec.extend(short_rows.iter().map(|row| row.gain));
    }
    if related {
        for amount in short_rec.iter_mut().chain([&mut building_rec, &mut land_rec]) {
            if *amount < 0 {
                disallowed += -*amount;
                *amount = phase::floor(*amount, 0);
            }
        }
    }
    let short_sum: i64 = short_rec.iter().sum();
    // Section 1239: a controlled entity's purchase of depreciable property
    // makes the gain ordinary income (Part II line 11).
    let section1239 = buyer == Buyer::ControlledEntity && building_rec > 0;
    let controlled_building = if section1239 { building_rec } else { 0 };
    // Unrecaptured Section 1250 Gain Worksheet line 1: the smaller of
    // Form 4797 line 24 and line 26 for the building.
    let (ordinary, unrecaptured) = if section1239 {
        (0, 0)
    } else {
        let amounts = crate::kernel::recapture::section1250(
            phase::floor(building_rec, 0),
            depreciation,
            0,
            additional,
        );
        (amounts.ordinary_dollars, amounts.unrecaptured1250_dollars)
    };
    // One Form 6252 for the part that gains (pro rata payments give the same
    // result as one form for each when both gain); a part sold at a loss
    // cannot use the installment method and is reported in full (Pub 537,
    // "Single Sale of Several Assets"), with the proceeds and payments
    // divided in proportion to the net fair market values (the price shares).
    let mut installment = None;
    let mut reported_in_full = 0;
    if let Some(payment) = payment {
        let total_price = price_r.max(1);
        let form_for =
            |part_price: i64, cost: i64, depreciation: i64, expenses: i64, recapture: i64| {
                // Payments and a loan the buyer assumes follow the part's price.
                payment.scaled(|amount| fraction(rental(amount), part_price, total_price)).form(
                    dollars(part_price),
                    cost,
                    depreciation,
                    dollars(expenses),
                    recapture,
                    0,
                )
            };
        if building > 0 && land >= 0 {
            installment = form_for(
                long_price + land_price_r,
                basis + land_basis,
                depreciation,
                long_expenses + land_expenses,
                ordinary,
            );
        } else if building > 0 {
            installment = form_for(long_price, basis, depreciation, long_expenses, ordinary);
            reported_in_full = land_rec;
        } else if land > 0 {
            installment = form_for(land_price_r, land_basis, 0, land_expenses, 0);
            reported_in_full = building_rec;
        }
        if installment.is_none() {
            reported_in_full = 0;
        }
    }
    let line11 = short_sum + controlled_building;
    let recognized = match &installment {
        Some(form) => form.gain + reported_in_full + short_sum,
        None => building_rec + land_rec + short_sum,
    };
    let mut components = Vec::new();
    let mut component = |row, ordinary, section1231, unrecaptured| {
        components.push((
            row,
            crate::farm_land::LandGain {
                gain: ordinary + section1231,
                percent: 0,
                recapture: ordinary,
                ordinary,
                section1231,
                unrecaptured,
            },
        ));
    };
    if let Some(form) = &installment {
        // The existing Form6252 combines only gaining parts. Its recognized
        // amount is one positive component; a losing part remains separate.
        component(
            FarmRentalPropertyRow::RentalCombined,
            ordinary,
            form.gain - ordinary,
            phase::cap(unrecaptured, [form.gain]),
        );
        if building <= 0 {
            component(FarmRentalPropertyRow::RentalBuilding, 0, building_rec, 0);
        }
        if land < 0 {
            component(FarmRentalPropertyRow::RentalLand, 0, land_rec, 0);
        }
    } else {
        component(
            FarmRentalPropertyRow::RentalBuilding,
            ordinary + controlled_building,
            building_rec - ordinary - controlled_building,
            unrecaptured,
        );
        component(FarmRentalPropertyRow::RentalLand, 0, land_rec, 0);
    }
    for (row, amount) in short_rows.iter().zip(&short_rec) {
        component(FarmRentalPropertyRow::RentalShort(row.index), *amount, 0, 0);
    }
    Ok(Some(RentalGain {
        realized,
        basis: basis + (realized - net_realized),
        depreciation,
        building,
        land_realized,
        land_basis,
        land,
        unrecaptured: installment.map_or(unrecaptured, |f| phase::cap(unrecaptured, [f.gain])),
        recognized,
        gross: match &installment {
            Some(form) => {
                phase::floor(form.gain, 0)
                    + phase::floor(reported_in_full, 0)
                    + short_rec.iter().map(|amount| phase::floor(*amount, 0)).sum::<i64>()
            }
            None => {
                phase::floor(building_rec, 0)
                    + phase::floor(land_rec, 0)
                    + short_rec.iter().map(|amount| phase::floor(*amount, 0)).sum::<i64>()
            }
        },
        additional,
        ordinary,
        installment,
        short_rows,
        components,
        line11,
        postponed,
        personal_proceeds,
        personal_basis,
        personal: personal_gain,
        disallowed,
        section1239,
        building_recognized: building_rec,
        land_recognized: land_rec,
    }))
}

// A rental's disposition for the Form 4797 netting: its sale or
// condemnation, or its like-kind exchange (Form 8824) as a sale when it does
// not qualify.
pub(crate) fn rental_disposition(
    v: &FederalRental,
    amt: bool,
) -> Result<Option<RentalGain>, (&'static str, &'static str)> {
    match &v.sale {
        RentalSale::No => Ok(None),
        RentalSale::Sold { .. } | RentalSale::Condemned { .. } => rental_sale(v, amt),
        RentalSale::Exchanged(exchange) => exchange.gain(v, amt).map(Some),
    }
}

// The rental sale facts this App calculates; None when supported.
pub(crate) fn rental_sale_refusal(
    v: &FederalRental,
    source: Option<&crate::nonmacrs::RentalRecoverySource<'_>>,
) -> Option<(&'static str, &'static str)> {
    if v.location.foreign() && !matches!(v.sale, crate::RentalSale::No) {
        return crate::rental_foreign::source_admission(v, source).err();
    }
    if !matches!(v.personal_use, crate::RentalPersonalUse::No)
        && v.at_risk
            .as_ref()
            .is_some_and(|risk| risk.prior_sale(false) > 0 || risk.prior_sale(true) > 0)
    {
        return Some((
            "ISSUE_RENTAL_HOME_AT_RISK_SALE_CARRY",
            crate::fixture_english!(
                "A prior at-risk property-sale loss on a rental with personal use needs the separate home and former-activity allocation, which is not calculated."
            ),
        ));
    }
    let date = v.sale.date()?;
    if v.buildings.iter().any(|b| {
        matches!(
            b.acquired,
            crate::RentalAcquisition::Converted { .. } | crate::RentalAcquisition::Gift
        )
    }) {
        return Some((
            "ISSUE_RENTAL_SALE_DUAL_BASIS",
            crate::fixture_english!(
                "Selling a rental converted from personal use or received as a gift needs the separate gain and loss bases, which are not calculated."
            ),
        ));
    }
    if let RentalSale::Exchanged(exchange) = &v.sale
        && !exchange.valid()
    {
        return exchange.refusal(v);
    }
    if !v.sale.valid() {
        return Some((
            "ISSUE_RENTAL_SALE_DATE_OR_LAND_PRICE_INVALID",
            crate::fixture_english!(
                "Enter the date you sold the rental in 2026 and a land price no larger than the sale price."
            ),
        ));
    }
    if v.buildings.iter().any(|b| b.in_service >= date) {
        return Some((
            "ISSUE_RENTAL_SALE_BEFORE_BUILDING_SERVICE",
            crate::fixture_english!(
                "The date sold or exchanged must be after the date each building and improvement was first available to rent."
            ),
        ));
    }
    // A part replaced or removed is a disposition before the whole property
    // leaves; an exchange continues the buildings and carries no part out.
    if v.buildings.iter().any(|b| {
        b.partial
            .as_ref()
            .is_some_and(|part| matches!(v.sale, RentalSale::Exchanged(_)) || part.date >= date)
    }) {
        return Some((
            "ISSUE_RENTAL_PARTIAL_WITH_DISPOSITION",
            crate::fixture_english!(
                "A part of a building replaced or removed must come before the date the whole property was sold or condemned, and a rental exchanged for other property cannot also have a part disposed of (Form 4797 line 1b and Form 8824)."
            ),
        ));
    }
    if v.buildings.iter().any(|b| b.rehabilitation.is_some()) {
        let installment =
            matches!(&v.sale, RentalSale::Sold { payment, .. } if payment.installment());
        let replaced = matches!(&v.sale, RentalSale::Condemned { replacement_cost, .. } if *replacement_cost > 0);
        if matches!(v.sale, RentalSale::Exchanged(_)) || installment || replaced {
            return Some((
                "ISSUE_RENTAL_SALE_WITH_REHABILITATION_CREDIT",
                crate::fixture_english!(
                    "A rental with a rehabilitation credit that is exchanged for other property, replaced after a condemnation or sold by installment is not calculated: the credit's recapture and basis reduction (Form 4255, Form 4797 line 28a) are not carried through Form 8824 or Form 6252."
                ),
            ));
        }
    }
    if let RentalSale::Sold { personal_share_percent: 0, .. } = &v.sale
        && !matches!(v.personal_use, crate::RentalPersonalUse::No)
    {
        return Some((
            "ISSUE_RENTAL_SALE_PERSONAL_SHARE_NEEDED",
            crate::fixture_english!(
                "Enter the personal-use share of a property you or your family also used (a percent of the years you owned it)."
            ),
        ));
    }
    // The exchange of a property also used personally needs the home-use split
    // of Form 8824 that this App does not carry through.
    if matches!(v.sale, RentalSale::Exchanged(_))
        && !matches!(v.personal_use, crate::RentalPersonalUse::No)
    {
        return Some((
            "ISSUE_RENTAL_EXCHANGE_WITH_PERSONAL_USE",
            crate::fixture_english!(
                "The like-kind exchange of a rental property you or your family also used is not calculated (Form 8824 with the home-use rules of Publication 523)."
            ),
        ));
    }
    if let RentalSale::Exchanged(exchange) = &v.sale
        && let Some(refusal) = exchange.refusal(v)
    {
        return Some(refusal);
    }
    if let RentalSale::Sold { price, payment, .. } = &v.sale
        && payment.installment()
        && let Some(refusal) = payment.refusal(*price)
    {
        return Some(refusal);
    }
    None
}

// Sold rentals by their income entry: the whole gain is section 1231
// (Part III building gain after no additional depreciation, and the land),
// the rows held a year or less and a controlled buyer's building gain are
// ordinary (line 11). A kept rental with a part replaced or removed has the
// part's gain or loss; separately disposed equipment joins the same source.
pub(crate) fn rental_sources(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    amt: bool,
    home_use: Option<&crate::extended_rules::HomeUseContext>,
) -> Result<Vec<(i64, crate::farm_land::LandGain)>, crate::activity_risk::SourceIssue> {
    let draft = form_context.draft;
    let property_entries = crate::community::property_entries(draft);
    let mid_quarter =
        crate::depreciation::mid_quarter_with_macrs(draft, Some(form_context.macrs().as_ref()));
    let mut result = Vec::new();
    for entry in &property_entries {
        if let Some(value) =
            rental_property_source(form_context, entry, mid_quarter, amt, home_use)?
        {
            result.push((entry.id, value));
        }
    }
    Ok(result)
}

pub(crate) fn rental_source(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    amt: bool,
    home_use: Option<&crate::extended_rules::HomeUseContext>,
) -> Result<Option<crate::farm_land::LandGain>, crate::activity_risk::SourceIssue> {
    let draft = form_context.draft;
    let entry = draft.income.iter().find(|entry| entry.id == id).ok_or_else(|| {
        crate::activity_risk::SourceIssue::new(
            id,
            crate::activity_risk::ProjectionIssue::Source("DIRECT_ACTIVITY_SOURCE_UNAVAILABLE"),
        )
    })?;
    let property = crate::community::property_entry(draft, entry);
    rental_property_source(
        form_context,
        &property,
        crate::depreciation::mid_quarter_with_macrs(draft, Some(form_context.macrs().as_ref())),
        amt,
        home_use,
    )
}

// Native components of the same rental activity, including separately
// disposed equipment. Reuse asset recognition and its regular/AMT basis.
fn rental_current_property(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    v: &FederalRental,
    mid_quarter: bool,
    amt: bool,
    home_use: Option<&crate::extended_rules::HomeUseContext>,
) -> Result<FarmRentalPropertySource, crate::activity_risk::ProjectionIssue> {
    let recovery_source = crate::nonmacrs::rental_recovery_source_with_macrs(
        form_context.draft,
        id,
        true,
        Some(form_context.macrs().as_ref()),
    );
    let source = recovery_source.as_ref();
    let issue = |(key, _)| crate::activity_risk::ProjectionIssue::Source(key);
    let mut components = Vec::new();
    if let Some(whole) = rental_disposition(v, amt).map_err(issue)? {
        if whole.components.is_empty() {
            // Form8824 already owns this recognized source amount.
            components.push((
                FarmRentalPropertyRow::RentalCombined,
                crate::farm_land::LandGain {
                    gain: whole.recognized,
                    percent: 0,
                    recapture: whole.ordinary,
                    ordinary: whole.ordinary + whole.line11,
                    section1231: whole.recognized - whole.ordinary - whole.line11,
                    unrecaptured: whole.unrecaptured,
                },
            ));
        } else {
            components.extend(whole.components);
        }
    }
    components.extend(
        rental_partials(v, amt, source)
            .map_err(issue)?
            .into_iter()
            .map(|(index, part)| (FarmRentalPropertyRow::RentalPartial(index), part.land_gain())),
    );
    for (index, asset) in v.assets.iter().enumerate() {
        let asset = source.map_or(asset, |source| source.asset(index, asset));
        if let Some(key) = source.and_then(|source| source.history_issue(asset)) {
            return Err(crate::activity_risk::ProjectionIssue::Source(key));
        }
        if asset.nonmacrs() && (!asset.valid() || asset.nonmacrs_issue().is_some()) {
            return Err(crate::activity_risk::ProjectionIssue::Source(
                asset.nonmacrs_issue().unwrap_or("ASSET_NONMACRS_ORIGINAL_HISTORY"),
            ));
        }
        if !asset.disposal.valid() {
            return Err(crate::activity_risk::ProjectionIssue::Source(
                "ISSUE_ENTER_A_RETURN_YEAR_DATE_SOLD_OR_STOPPED_USING_THE_SALE_PRICE_AND_SELLING_A990EE59",
            ));
        }
        if let Some((key, _)) = asset.disposal_refusal_in_context(
            source.map_or_else(crate::macrs_source::AdsInvocation::default, |source| {
                source.context(asset)
            }),
        ) {
            return Err(crate::activity_risk::ProjectionIssue::Source(key));
        }
    }
    components.extend(business_asset_components(&v.assets, mid_quarter, amt, source));
    let casualty_losses: Vec<(usize, i64)> = if crate::extended_rules::used_as_home(v)
        && v.buildings.iter().any(|building| building.casualty.is_some())
    {
        crate::extended_rules::rental_home_casualty_losses(form_context, id, v, amt, home_use)
            .map_err(issue)?
            .into_iter()
            .map(|row| (row.building, row.dollars))
            .collect()
    } else {
        crate::casualty::rental::property_losses(v, amt)
            .map_err(crate::activity_risk::ProjectionIssue::Source)?
            .into_iter()
            .map(|(index, loss)| (index, dollars(loss)))
            .collect()
    };
    components.extend(casualty_losses.into_iter().map(|(index, loss)| {
        let ordinary = -loss;
        (
            FarmRentalPropertyRow::RentalCasualty(index),
            crate::farm_land::LandGain {
                gain: ordinary,
                percent: 0,
                recapture: 0,
                ordinary,
                section1231: 0,
                unrecaptured: 0,
            },
        )
    }));
    let sum = |field: fn(&crate::farm_land::LandGain) -> i64| {
        crate::kernel::ratio::checked_sum(components.iter().map(|(_, value)| field(value))).ok_or(
            crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Overflow,
            ),
        )
    };
    let value = if components.is_empty() {
        None
    } else {
        Some(crate::farm_land::LandGain {
            gain: sum(|v| v.gain)?,
            percent: 0,
            recapture: sum(|v| v.recapture)?,
            ordinary: sum(|v| v.ordinary)?,
            section1231: sum(|v| v.section1231)?,
            unrecaptured: sum(|v| v.unrecaptured)?,
        })
    };
    Ok(FarmRentalPropertySource { value, components })
}

// The one rental property Form6198 character settlement. Both native source
// recognition and its QBI history projection consume these same exact cents.
fn rental_property_risk_cents(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    data: &FederalRental,
    value: &crate::farm_land::LandGain,
    prior: i64,
    amt: bool,
    home_use: Option<&crate::extended_rules::HomeUseContext>,
) -> Result<(i64, [i64; 2], [i64; 2]), crate::activity_risk::SourceIssue> {
    let source_issue = |issue| crate::activity_risk::SourceIssue::new(id, issue);
    let arithmetic = || {
        source_issue(crate::activity_risk::ProjectionIssue::Arithmetic(
            crate::kernel::loss_limits::LossLimitError::Overflow,
        ))
    };
    let raw =
        value.gain.checked_mul(100).and_then(|v| v.checked_sub(prior)).ok_or_else(arithmetic)?;
    let allowed =
        crate::extended_rules::rental_source_risk(form_context, id, data, amt, raw, home_use)
            .map_err(source_issue)?
            .ok_or_else(|| source_issue(crate::activity_risk::ProjectionIssue::InvalidMethod))?;
    // Settle the already-owned property suspension by native character. A
    // positive ordinary row never absorbs an unrelated section1231 loss cut.
    let nets = [
        (0usize, value.ordinary.checked_mul(100).ok_or_else(arithmetic)?),
        (
            1,
            value
                .section1231
                .checked_mul(100)
                .and_then(|v| v.checked_sub(prior))
                .ok_or_else(arithmetic)?,
        ),
    ];
    let held = allowed.sale.checked_sub(raw).ok_or_else(arithmetic)?;
    let held = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(held)
        .map_err(|_| arithmetic())?;
    let crate::kernel::source_fact::SourceFact::Reported(classes) =
        crate::kernel::loss_limits::passive_characters(
            crate::kernel::source_fact::SourceFact::Reported(
                crate::kernel::loss_limits::CharacterInput { nets: &nets, suspended: held },
            ),
        )
        .map_err(|error| source_issue(crate::activity_risk::ProjectionIssue::Arithmetic(error)))?
    else {
        return Err(source_issue(crate::activity_risk::ProjectionIssue::InvalidMethod));
    };
    let held = [classes[0].held(), classes[1].held()];
    let cents = [
        nets[0].1.checked_add(held[0]).ok_or_else(arithmetic)?,
        nets[1].1.checked_add(held[1]).ok_or_else(arithmetic)?,
    ];
    Ok((allowed.sale, cents, held))
}

fn rental_property_source(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    entry: &crate::FederalIncomeEntry,
    mid_quarter: bool,
    amt: bool,
    home_use: Option<&crate::extended_rules::HomeUseContext>,
) -> Result<Option<crate::farm_land::LandGain>, crate::activity_risk::SourceIssue> {
    let source_issue = |issue| crate::activity_risk::SourceIssue::new(entry.id, issue);
    // Equipment rented out outside a business: the depreciation
    // recaptured is ordinary investment income (no section 1231 gain).
    if let FederalIncome::NonbusinessProperty {
        kind: crate::NonbusinessPropertyKind::PersonalRental { assets, .. },
        ..
    } = &entry.income
    {
        let ordinary: i64 = assets
            .iter()
            .filter_map(|asset| {
                asset.sale_in_context(
                    mid_quarter,
                    amt,
                    form_context.macrs().context(entry.owner.owned(), Some(entry.id), asset),
                )
            })
            .map(|sale| equipment_ordinary(&sale))
            .sum();
        if ordinary != 0 {
            return Ok(Some(crate::farm_land::LandGain {
                gain: ordinary,
                percent: 0,
                recapture: ordinary,
                ordinary,
                section1231: 0,
                unrecaptured: 0,
            }));
        }
        return Ok(None);
    }
    let FederalIncome::Rental { data } = &entry.income else {
        return Ok(None);
    };
    let recovery_source = crate::nonmacrs::rental_recovery_source_with_macrs(
        form_context.draft,
        entry.id,
        true,
        Some(form_context.macrs().as_ref()),
    );
    if let Some((key, _)) = rental_sale_refusal(data, recovery_source.as_ref()) {
        return Err(source_issue(crate::activity_risk::ProjectionIssue::Source(key)));
    }
    let current = rental_current_property(form_context, entry.id, data, mid_quarter, amt, home_use)
        .map_err(source_issue)?;
    let prior = data
        .at_risk
        .as_ref()
        .ok_or_else(|| {
            source_issue(crate::activity_risk::ProjectionIssue::Source(
                "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
            ))
        })?
        .prior_sale(amt);
    if current.value.is_none() && prior == 0 {
        return Ok(None);
    }
    let mut value = current.value.unwrap_or_else(crate::farm_land::LandGain::zero);
    let (allowed_sale, cents, _) =
        rental_property_risk_cents(form_context, entry.id, data, &value, prior, amt, home_use)?;
    // Keep the actual signed prior cents through settlement and project once
    // against this same native joint, preserving ordinary-first rounding.
    let gain = dollars(allowed_sale);
    let [ordinary, section1231] = crate::kernel::ratio::project_rounded_pair(gain, cents, 0)
        .ok_or_else(|| {
            source_issue(crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Footing,
            ))
        })?;
    value.gain = gain;
    value.ordinary = ordinary;
    value.recapture = ordinary;
    value.section1231 = section1231;
    // A pure exchange with no recognized amount remains known absent. Keep
    // actual equipment/partial components even when their amounts offset.
    if matches!(data.sale, RentalSale::Exchanged(_))
        && prior == 0
        && value.gain == 0
        && current.components.iter().all(|(row, value)| {
            matches!(row, FarmRentalPropertyRow::RentalCombined)
                && value.ordinary == 0
                && value.section1231 == 0
        })
    {
        return Ok(None);
    }
    Ok(Some(value))
}

// Form 6198 sees every property disposition of the activity before Form
// 8582, including a replaced building component while the rental is kept.
pub(crate) fn rental_sale_amount(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    v: &FederalRental,
    mid_quarter: bool,
    amt: bool,
    home_use: Option<&crate::extended_rules::HomeUseContext>,
) -> Result<i64, crate::activity_risk::ProjectionIssue> {
    let current = rental_current_property(form_context, id, v, mid_quarter, amt, home_use)?
        .value
        .map_or(0, |v| v.gain);
    let prior = v
        .at_risk
        .as_ref()
        .ok_or(crate::activity_risk::ProjectionIssue::Source(
            "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
        ))?
        .prior_sale(amt);
    current.checked_mul(100).and_then(|current| current.checked_sub(prior)).ok_or(
        crate::activity_risk::ProjectionIssue::Arithmetic(
            crate::kernel::loss_limits::LossLimitError::Overflow,
        ),
    )
}

// Property character source for the rental's passive-pool entry, including
// kept-property equipment dispositions. Absence stays distinct from errors.
pub(crate) fn kept_rental_partials(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    amt: bool,
    home_use: Option<&crate::extended_rules::HomeUseContext>,
) -> Result<Option<crate::farm_land::LandGain>, crate::activity_risk::SourceIssue> {
    let d = form_context.draft;
    let property_entries = crate::community::property_entries(d);
    let Some(v) = property_entries.iter().find_map(|entry| match &entry.income {
        FederalIncome::Rental { data } if entry.id == id => Some(data),
        _ => None,
    }) else {
        return Ok(None);
    };
    let current = rental_current_property(
        form_context,
        id,
        v,
        crate::depreciation::mid_quarter(d),
        amt,
        home_use,
    )
    .map_err(|issue| crate::activity_risk::SourceIssue::new(id, issue))?;
    // Whole-only property retains its established PropertyDisposition packet.
    // Separate current equipment/partials and prior property losses require
    // the existing ordinary/section1231 character settlement instead.
    if !current.components.iter().any(|(row, _)| {
        matches!(
            row,
            FarmRentalPropertyRow::Asset(_)
                | FarmRentalPropertyRow::Partial(_)
                | FarmRentalPropertyRow::RentalPartial(_)
                | FarmRentalPropertyRow::RentalCasualty(_)
        )
    }) && v.at_risk.as_ref().is_some_and(|risk| risk.prior_sale(amt) == 0)
    {
        return Ok(None);
    }
    rental_source(form_context, id, amt, home_use)
}

// Consumers after section 469 must use the same allowed character amounts
// as the shared passive pool, never reconstruct them from the raw sale.
pub(crate) fn allowed_rental_sources(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
) -> Result<Vec<(i64, crate::farm_land::LandGain)>, crate::activity_risk::SourceIssue> {
    let mut sources = rental_sources(form_context, amt, Some(&pool.home_use))?;
    for (id, value) in &mut sources {
        if let Some((_, cuts)) = pool.rental_partial_cuts.iter().find(|(source, _)| source == id) {
            value.ordinary += cuts[0];
            value.recapture += cuts[0];
            value.section1231 += cuts[1];
            value.gain += cuts[0] + cuts[1];
        }
    }
    Ok(sources)
}

// Source cents for the prior property-loss component at the two existing
// limits. Keep the native current-first prior report row as the dollar carrier;
// the vintage engine receives actual cents, never that rounded report row.
pub(crate) fn rental_prior_property_release(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    pool: &crate::extended_rules::RentalResult,
    id: i64,
    v: &FederalRental,
) -> Result<(i64, i64, i64), crate::activity_risk::SourceIssue> {
    use crate::kernel::ratio;
    let fail = || {
        crate::activity_risk::SourceIssue::new(
            id,
            crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Footing,
            ),
        )
    };
    let Some((current, rows)) = rental_allowed_property_rows(form_context, pool, id, v, false)?
    else {
        return Ok((0, 0, 0));
    };
    let prior = v.at_risk.as_ref().ok_or_else(fail)?.prior_sale(false);
    if prior == 0 {
        return Ok((0, 0, 0));
    }
    let value = current.value.unwrap_or_else(crate::farm_land::LandGain::zero);
    let (_, _, risk_held) = rental_property_risk_cents(
        form_context,
        id,
        v,
        &value,
        prior,
        false,
        Some(&pool.home_use),
    )?;
    let mut weights = current
        .components
        .iter()
        .map(|(_, component)| {
            crate::kernel::phase::checked_loss(component.section1231)
                .and_then(|loss| loss.checked_mul(100))
                .ok_or_else(fail)
        })
        .collect::<Result<Vec<_>, _>>()?;
    weights.push(prior);
    let risk_cuts = ratio::allocate_remaining(risk_held[1], &weights);
    for (weight, cut) in weights.iter_mut().zip(risk_cuts) {
        *weight -= cut;
    }
    let risk_prior = *weights.last().ok_or_else(fail)?;
    let passive_cut = pool
        .rental_partial_cuts
        .iter()
        .find(|(source, _)| *source == id)
        .map_or(0, |(_, cuts)| cuts[1]);
    let capacity = ratio::checked_sum(weights.iter().copied()).ok_or_else(fail)?;
    // PAL has an annual-Dollar boundary. Its rounded suspension cannot hold
    // more than the actual negative source cents it represents.
    let passive_cut =
        crate::kernel::phase::cap(passive_cut.checked_mul(100).ok_or_else(fail)?, [capacity]);
    if passive_cut < 0 {
        return Err(fail());
    }
    let passive_cuts = ratio::allocate_remaining(passive_cut, &weights);
    let final_prior = risk_prior - passive_cuts.last().copied().ok_or_else(fail)?;
    Ok((rows.prior_section1231_allowed, risk_prior, final_prior))
}

// The source carries risk-limited property amounts; the pool adds its PAL
// cuts. Allocate both limits to negative component rows within the same
// Form 4797 part. A retained at-risk property loss is a separate prior item.
// Set final native character rows from the bound source allocation. Raw
// recognition, depreciation and basis rows remain the disposition owner's.
fn set_farm_rental_property_row(r: &mut FederalResult, key: &str, amount: i64) {
    // Pure owner calls can use an empty scratch report. As for ordinary
    // rentals below, only an existing disposition row is a report target;
    // financial source availability and totals come from the bound cache.
    if let Some(row) = r.lines.iter_mut().find(|row| row.id == key) {
        row.amount = amount * 100;
    }
}

// The common invocation returns each current Schedule C property row to its
// original Form4797 location; proceeds, recovery and basis remain native facts.
pub(crate) fn report_direct_business_property_limits(
    d: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    r: &mut FederalResult,
) {
    for (id, parts) in &pool.direct_business_property {
        let Some(assets) = d.income.iter().find_map(|entry| match &entry.income {
            FederalIncome::Business { data } if entry.id == *id => Some(&data.assets),
            _ => None,
        }) else {
            continue;
        };
        for part in parts {
            let (asset, partial) = match part.original.id {
                DirectBusinessPropertyId::Asset(id) => (id, false),
                DirectBusinessPropertyId::Partial(id) => (id, true),
                DirectBusinessPropertyId::Land { .. } => continue,
            };
            let Some(index) = assets.iter().position(|source| source.id == asset) else { continue };
            let value = part.reported();
            if partial {
                set_farm_rental_property_row(
                    r,
                    &format!("business_{id}_asset_{index}_partial_gain"),
                    value.gain,
                );
            } else {
                for (suffix, original, allowed) in [
                    ("ordinary", part.original.dollars.ordinary, value.ordinary),
                    ("section1231", part.original.dollars.section1231, value.section1231),
                ] {
                    if original < 0 {
                        set_farm_rental_property_row(
                            r,
                            &format!("business_{id}_asset_{index}_disposal_{suffix}"),
                            allowed,
                        );
                    }
                }
            }
        }
    }
}

// The bound rental calculation is captured in a scratch owner report. Apply
// its cached Form4835 rows to the final report after that snapshot is adopted,
// where the native business-asset and farm-land rows actually live.
pub(crate) fn report_farm_rental_property_limits(
    pool: &crate::extended_rules::RentalResult,
    r: &mut FederalResult,
) {
    for (id, rows) in &pool.farm_rental_property_rows {
        for (index, gain) in &rows.partial_gains {
            set_farm_rental_property_row(
                r,
                &format!("business_{id}_asset_{index}_partial_gain"),
                *gain,
            );
        }
        for (source, class, amount) in &rows.character_rows {
            let suffix = if *class == 0 { "ordinary" } else { "section1231" };
            let prefix = match source {
                FarmRentalPropertyRow::Land(index) => format!("farm_{id}_land_{index}"),
                FarmRentalPropertyRow::Asset(index) => {
                    format!("business_{id}_asset_{index}_disposal")
                }
                FarmRentalPropertyRow::Partial(_) => {
                    unreachable!("partial rows have one native gain")
                }
                _ => unreachable!("farm source has no ordinary-rental rows"),
            };
            set_farm_rental_property_row(r, &format!("{prefix}_{suffix}"), *amount);
        }
        if rows.prior_section1231_allowed > 0 {
            row(
                r,
                "2",
                &format!("rental_{id}_sale_at_risk_adjustment"),
                -rows.prior_section1231_allowed,
                "income.farm_rental.at_risk",
                msg!("EXPLAIN_RENTAL_AT_RISK"),
            );
        }
    }
}

// One direct-rental row settlement for both detailed Form4797 rows and
// partial-disposition totals. Risk, PAL and native rounding stay shared.
pub(crate) fn rental_allowed_property_rows(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    pool: &crate::extended_rules::RentalResult,
    id: i64,
    v: &FederalRental,
    amt: bool,
) -> Result<
    Option<(FarmRentalPropertySource, FarmRentalPropertyRows)>,
    crate::activity_risk::SourceIssue,
> {
    let d = form_context.draft;
    let source = rental_current_property(
        form_context,
        id,
        v,
        crate::depreciation::mid_quarter(d),
        amt,
        Some(&pool.home_use),
    )
    .map_err(|issue| crate::activity_risk::SourceIssue::new(id, issue))?;
    let Some(mut allowed) = rental_source(form_context, id, amt, Some(&pool.home_use))? else {
        return Ok(None);
    };
    if let Some((_, cuts)) = pool.rental_partial_cuts.iter().find(|(source, _)| *source == id) {
        allowed.ordinary += cuts[0];
        allowed.section1231 += cuts[1];
        allowed.gain += cuts[0] + cuts[1];
    }
    let prior_cents = v
        .at_risk
        .as_ref()
        .ok_or_else(|| {
            crate::activity_risk::SourceIssue::new(
                id,
                crate::activity_risk::ProjectionIssue::Source(
                    "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
                ),
            )
        })?
        .prior_sale(amt);
    let arithmetic = || {
        crate::activity_risk::SourceIssue::new(
            id,
            crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Overflow,
            ),
        )
    };
    let current_cents =
        source.value.map_or(0, |v| v.section1231).checked_mul(100).ok_or_else(arithmetic)?;
    let raw_cents = current_cents.checked_sub(prior_cents).ok_or_else(arithmetic)?;
    // Preserve the native current-component dollar boundary. The prior
    // row is its residual against the actual pre-limit source joint;
    // a half-dollar boundary must not become a fictitious suspension.
    let [_, prior_row] = crate::kernel::ratio::project_rounded_pair(
        dollars(raw_cents),
        [current_cents, -prior_cents],
        0,
    )
    .ok_or_else(|| {
        crate::activity_risk::SourceIssue::new(
            id,
            crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Footing,
            ),
        )
    })?;
    let prior = prior_row.checked_neg().ok_or_else(arithmetic)?;
    let rows = source
        .allowed_rows(&allowed, prior)
        .map_err(|issue| crate::activity_risk::SourceIssue::new(id, issue))?;
    Ok::<_, crate::activity_risk::SourceIssue>(Some((source, rows)))
}

pub(crate) fn report_rental_partial_limits(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    pool: &crate::extended_rules::RentalResult,
    r: &mut FederalResult,
) {
    let d = form_context.draft;
    let entries = crate::community::property_entries(d);
    for entry in &entries {
        let FederalIncome::Rental { data: v } = &entry.income else {
            continue;
        };
        let id = entry.id;
        let projected = rental_allowed_property_rows(form_context, pool, id, v, false);
        let (source, rows) = match projected {
            Ok(Some(value)) => value,
            Ok(None) => continue,
            Err(issue) => {
                issue.report(r);
                continue;
            }
        };
        for (source_row, value) in &source.components {
            if let FarmRentalPropertyRow::RentalCasualty(index) = source_row {
                row(
                    r,
                    "14",
                    &format!("rental_{id}_casualty_{index}_ordinary"),
                    value.ordinary,
                    "income.rental.buildings.casualty",
                    msg!("EXPLAIN_RENTAL_CASUALTY_ACTIVITY"),
                );
            }
        }
        for (index, amount) in &rows.partial_gains {
            set_farm_rental_property_row(
                r,
                &format!("rental_{id}_asset_{index}_partial_gain"),
                *amount,
            );
        }
        for (row_id, class, amount) in &rows.character_rows {
            let suffix = if *class == 0 { "ordinary" } else { "section1231" };
            let key = match row_id {
                FarmRentalPropertyRow::Asset(index) => {
                    format!("rental_{id}_asset_{index}_disposal_{suffix}")
                }
                FarmRentalPropertyRow::RentalPartial(index) => {
                    format!("rental_{id}_building_{index}_partial_gain")
                }
                FarmRentalPropertyRow::RentalCasualty(index) => {
                    format!("rental_{id}_casualty_{index}_ordinary")
                }
                FarmRentalPropertyRow::RentalBuilding => format!("rental_{id}_sale_building"),
                FarmRentalPropertyRow::RentalLand => format!("rental_{id}_sale_land"),
                FarmRentalPropertyRow::RentalShort(index) => {
                    format!("rental_{id}_sale_row_{index}_ordinary")
                }
                FarmRentalPropertyRow::RentalCombined => {
                    // The Form8824 owner supplies one resolved recognized
                    // source, not a guessed building/land allocation.
                    let raw = source
                        .components
                        .iter()
                        .find(|(row, _)| row == row_id)
                        .expect("native component identity")
                        .1;
                    let raw = if *class == 0 { raw.ordinary } else { raw.section1231 };
                    row(
                        r,
                        if *class == 0 { "11" } else { "2" },
                        &format!("rental_{id}_sale_{suffix}_activity_adjustment"),
                        *amount - raw,
                        "income.rental.at_risk",
                        msg!("EXPLAIN_RENTAL_AT_RISK"),
                    );
                    continue;
                }
                _ => unreachable!("ordinary rental has no farm-land component"),
            };
            set_farm_rental_property_row(r, &key, *amount);
        }
        if rows.prior_section1231_allowed > 0 {
            row(
                r,
                "2",
                &format!("rental_{id}_sale_at_risk_adjustment"),
                -rows.prior_section1231_allowed,
                "income.rental.at_risk",
                msg!("EXPLAIN_RENTAL_AT_RISK"),
            );
        }
    }
}

const SOURCE: &str = "https://www.irs.gov/instructions/i4797";

fn row(
    r: &mut FederalResult,
    line: &str,
    id: &str,
    amount: i64,
    input: &str,
    explanation: crate::Message,
) {
    push_line(
        r,
        Some(FormLine::new(TaxForm::Form4797, line)),
        id,
        amount * 100,
        &[input],
        SOURCE,
        explanation,
    );
}

// Form 4797 Part IV, section 179 recapture (column (a)) of an asset still in
// use at 50% or less or kept for personal use: lines 35 to 37.
fn report_section179_recapture(
    r: &mut FederalResult,
    prefix: &str,
    asset: &BusinessAsset,
    input_root: &str,
    source: Option<&crate::nonmacrs::RentalRecoverySource<'_>>,
) {
    let Some(recapture) = asset.section179_recapture() else {
        return;
    };
    emit_lines(
        r,
        &mut (),
        [
            (
                "35(a)",
                "section179_prior",
                recapture.allowed,
                msg!("EXPLAIN_ASSET_SECTION179_PRIOR"),
            ),
            (
                "36(a)",
                "section179_recomputed",
                recapture.recomputed,
                msg!("EXPLAIN_ASSET_SECTION179_RECOMPUTED"),
            ),
            (
                "37(a)",
                "section179_recapture",
                recapture.excess(),
                msg!("EXPLAIN_ASSET_SECTION179_RECAPTURE"),
            ),
        ],
        |(line, suffix, amount, explanation), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form4797, line)),
                &format!("{prefix}_{suffix}"),
                LineValue::MoneyCents(if asset.nonmacrs() {
                    source
                        .and_then(|source| {
                            crate::kernel::ratio::exact::rounded(
                                &source.project(&crate::kernel::ratio::exact::whole(amount)),
                            )
                        })
                        .unwrap_or(amount)
                } else {
                    amount
                }),
                &[input_root],
                SOURCE,
                explanation,
            ))
        },
    );
}

// One business asset's refusals and Form 4797 rows. `prefix` names the asset's
// rows (`business_<id>_asset_<index>` or an employee vehicle's) and `context`
// is the record a correction opens.
#[allow(clippy::too_many_arguments)]
pub(crate) fn report_business_asset(
    r: &mut FederalResult,
    #[cfg(test)] section: &str,
    context: &crate::IssueContext,
    prefix: &str,
    index: usize,
    assets: &[BusinessAsset],
    mileage: usize,
    sale: Option<AssetGain>,
    partial: Option<PartialGain>,
    input_root: &str,
) {
    report_business_asset_source(
        r,
        #[cfg(test)]
        section,
        context,
        prefix,
        index,
        assets,
        mileage,
        sale,
        partial,
        input_root,
        None,
    );
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn report_business_asset_source(
    r: &mut FederalResult,
    #[cfg(test)] section: &str,
    context: &crate::IssueContext,
    prefix: &str,
    index: usize,
    assets: &[BusinessAsset],
    mileage: usize,
    sale: Option<AssetGain>,
    partial: Option<PartialGain>,
    input_root: &str,
    source: Option<&crate::nonmacrs::RentalRecoverySource<'_>>,
) {
    let asset = &assets[index];
    let actual = source.map_or(asset, |source| source.asset(index, asset));
    if actual.nonmacrs() {
        let issue = if !actual.valid() {
            Some("ASSET_NONMACRS_ORIGINAL_HISTORY")
        } else {
            actual.nonmacrs_issue()
        };
        if let Some(key) = issue {
            crate::issue_localized!(
                r,
                context,
                section,
                key,
                "Retain the original property schedule and actual source records."
            );
            return;
        }
    }
    report_section179_recapture(
        r,
        prefix,
        source.map_or(asset, |source| source.asset(index, asset)),
        input_root,
        source,
    );
    let name = || BusinessAsset::display_arg(assets, index, mileage);
    if let Some(part) = partial {
        report_partial(r, context, prefix, input_root, name(), &part, asset.cost);
    }
    if asset.disposal == AssetDisposal::No {
        return;
    }
    if !asset.disposal.valid() {
        crate::issue_localized!(
            r,
            context,
            section,
            "ISSUE_ENTER_A_RETURN_YEAR_DATE_SOLD_OR_STOPPED_USING_THE_SALE_PRICE_AND_SELLING_A990EE59",
            "Enter a 2026 date sold or stopped using, the sale price and selling expenses within the supported limits, and installment amounts no larger than the sale price (Form 6252 lines 5, 6 and 21).",
        );
        return;
    }
    let refusal = if sale.is_some() {
        asset.disposal_terms_refusal()
    } else {
        asset.disposal_refusal_in_context(
            source.map_or_else(crate::macrs_source::AdsInvocation::default, |source| {
                source.context(asset)
            }),
        )
    };
    if let Some((key, _message)) = refusal {
        #[cfg(test)]
        let message = _message;

        crate::issue_localized!(r, context, section, key, message,);
        return;
    }
    let prefix = format!("{prefix}_disposal");
    let disposal_input = format!("{input_root}.disposal");
    let input = disposal_input.as_str();
    let Some(v) = sale else {
        let vehicle = asset.is_listed_vehicle();
        r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)] format!(
                "{}: stopped business use in 2026. Depreciation stops at the date of the change under the convention; no gain or loss is recognized, and the property keeps its adjusted basis.{}",
                BusinessAsset::display_text(assets, index, mileage),
                if vehicle {
                    " If this vehicle is used 50% or less for business in a later year of its recovery period, excess depreciation must be recaptured on that year's return (Form 4797 Part IV)."
                } else {
                    ""
                }
            ),
            crate::Message::new(
                if vehicle {
                    "NOTE_ASSET_STOPPED_BUSINESS_USE_VEHICLE"
                } else {
                    "NOTE_ASSET_STOPPED_BUSINESS_USE"
                },
                alloc::vec![name()],
            ),
        ));
        return;
    };
    if v.disallowed > 0 {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("NOTE_SALE_TO_RELATIVE_LOSS_DISALLOWED", alloc::vec![name()]),
            context.clone(),
        );
        return;
    }
    if v.personal < 0 {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)]
            format!(
                "{}: the personal-use part of the sale is a ${} loss, which is not deductible.",
                BusinessAsset::display_text(assets, index, mileage),
                -v.personal
            ),
            crate::Message::new(
                "NOTE_ASSET_PERSONAL_SALE_LOSS",
                alloc::vec![
                    name(),
                    crate::MessageArg::Currency {
                        currency_code: "USD".into(),
                        scale: 0,
                        minor_units: -v.personal,
                    },
                ],
            ),
        ));
    }
    // The personal part of an installment sale is counted in full (section
    // 453(d) election out for that part): its gain is the Form 8949 row.
    if v.installment.is_some() && v.personal > 0 {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("NOTE_ASSET_PERSONAL_PART_IN_FULL", alloc::vec![name()]),
            context.clone(),
        );
    }
    let replaced = matches!(
        asset.disposal,
        AssetDisposal::Disposed {
            how: DisposalKind::Condemned { replacement_cost, .. },
            ..
        } if replacement_cost > 0
    );
    if replaced && v.recapture > 0 {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("NOTE_CONDEMNATION_RECAPTURE_LIMITED", alloc::vec![name()]),
            context.clone(),
        );
    }
    // Livestock for draft, breeding, dairy or sport has its own lines: 18 for
    // the ordinary gain or loss and 7 for the section 1231 gain or loss.
    if v.part_two() {
        if let Some(form) = &v.installment {
            row(
                r,
                if v.livestock { "18" } else { "11" },
                &format!("{prefix}_ordinary"),
                v.recapture,
                input,
                msg!("EXPLAIN_N_ORDINARY_3"),
            );
            crate::installment_return::report_sale(
                r,
                &format!("{prefix}_form6252"),
                input,
                form,
                0,
                true,
            );
            row(
                r,
                if v.livestock { "18" } else { "11" },
                &format!("{prefix}_installment_ordinary"),
                form.gain,
                input,
                msg!("EXPLAIN_N_INSTALLMENT_ORDINARY"),
            );
            return;
        }
        row(
            r,
            if v.livestock { "18" } else { "11" },
            &format!("{prefix}_ordinary"),
            v.gain,
            input,
            if v.scrapped {
                msg!("EXPLAIN_N_ORDINARY_2")
            } else if v.section1239 {
                msg!("EXPLAIN_N_ORDINARY_RELATED")
            } else {
                msg!("EXPLAIN_N_ORDINARY_3")
            },
        );
        return;
    }
    if v.gain < 0 || (v.gain == 0 && v.postponed == 0) {
        row(
            r,
            if v.livestock { "7" } else { "2" },
            &format!("{prefix}_section1231"),
            v.gain,
            input,
            msg!("EXPLAIN_N_SECTION1231"),
        );
        return;
    }
    for (line, suffix, amount, explanation) in [
        ("22", "sale_price", v.realized, msg!("EXPLAIN_ASSET_SALE_N_6")),
        ("23", "cost", v.basis, msg!("EXPLAIN_ASSET_SALE_N_7")),
        ("24", "depreciation", v.depreciation, msg!("EXPLAIN_ASSET_SALE_N_8")),
        ("25", "adjusted_basis", v.basis - v.depreciation, msg!("EXPLAIN_ASSET_SALE_N_4")),
        ("26", "gain", v.gain + v.postponed, msg!("EXPLAIN_ASSET_SALE_N_5")),
    ] {
        row(r, line, &format!("{prefix}_{suffix}"), amount, input, explanation);
    }
    if v.postponed > 0 {
        push_line(
            r,
            None,
            &format!("{prefix}_postponed"),
            v.postponed * 100,
            &[input],
            "https://www.irs.gov/publications/p544",
            msg!("EXPLAIN_ASSET_DISPOSAL_POSTPONED"),
        );
    }
    if v.section1250 {
        if v.additional > 0 {
            row(
                r,
                "28a",
                &format!("{prefix}_additional_depreciation"),
                v.additional,
                input,
                msg!("EXPLAIN_ASSET_ADDITIONAL_DEPRECIATION"),
            );
        }
        if v.recapture_1245 > 0 {
            row(
                r,
                "27b",
                &format!("{prefix}_1245_recapture"),
                v.recapture_1245,
                input,
                msg!("EXPLAIN_N_N_RECAPTURE_2"),
            );
        }
        row(
            r,
            "28g",
            &format!("{prefix}_1250_recapture"),
            v.recapture - v.recapture_1245,
            input,
            msg!("EXPLAIN_N_N_RECAPTURE"),
        );
    } else {
        row(
            r,
            "27b",
            &format!("{prefix}_1245_recapture"),
            v.recapture_1245,
            input,
            msg!("EXPLAIN_N_N_RECAPTURE_2"),
        );
    }
    if let Some(form) = &v.installment {
        report_installment(r, &prefix, input, form, v.unrecaptured);
    }
}

// A partial disposition's rows: the gain or loss on line 2 (held more than a
// year) or line 11, and the part's depreciation allowed (no form line). The
// return-wide totals are lines 1b and 1c (`partial_totals`).
fn report_partial(
    r: &mut FederalResult,
    context: &crate::IssueContext,
    prefix: &str,
    input: &str,
    name: crate::MessageArg,
    part: &PartialGain,
    cost: i64,
) {
    row(
        r,
        if part.long { "2" } else { "11" },
        &format!("{prefix}_partial_gain"),
        part.gain,
        input,
        msg!("EXPLAIN_N_PARTIAL_GAIN"),
    );
    push_line(
        r,
        None,
        &format!("{prefix}_partial_depreciation"),
        part.depreciation * 100,
        &[input],
        "https://www.irs.gov/publications/p544",
        msg!("EXPLAIN_N_PARTIAL_DEPRECIATION"),
    );
    let _ = cost;
    crate::federal_rules::note_once_scoped(
        r,
        crate::Message::new(
            "NOTE_PARTIAL_DISPOSITION_REDUCE_COST",
            alloc::vec![
                name,
                crate::MessageArg::Currency {
                    currency_code: "USD".into(),
                    scale: 0,
                    minor_units: part.cost,
                },
            ],
        ),
        context.clone(),
    );
}

// The gains and losses of every partial disposition of a MACRS building on the
// return, whole dollars: Form 4797 lines 1b (gain) and 1c (loss).

pub(crate) fn partial_totals_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<[i64; 2], crate::activity_risk::SourceIssue> {
    let property_entries = crate::community::property_entries(draft);
    let mut totals = [0i64; 2];
    let mut add = |gain: i64| {
        if gain > 0 {
            totals[0] += gain;
        } else {
            totals[1] += -gain;
        }
    };
    for entry in &property_entries {
        match &entry.income {
            FederalIncome::Business { data } => {
                if data.assets.iter().all(|asset| asset.partial.is_none()) {
                    continue;
                }
                let projection =
                    crate::community::source_projection(draft, pool, false, source_claims)?;
                if !draft.setup.as_ref().is_some_and(crate::community::applies) {
                    for (_, gain) in
                        projection.native_partial_gains.iter().filter(|(id, _)| *id == entry.id)
                    {
                        add(*gain);
                    }
                    continue;
                }
                let components = projection.components.get(&entry.id).ok_or_else(|| {
                    crate::activity_risk::SourceIssue::new(
                        entry.id,
                        crate::activity_risk::ProjectionIssue::Source(
                            "DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED",
                        ),
                    )
                })?;
                let components = components.as_ref().map_err(|key| {
                    crate::activity_risk::SourceIssue::new(
                        entry.id,
                        crate::activity_risk::ProjectionIssue::Source(key),
                    )
                })?;
                for component in components {
                    if matches!(component.component, Some(DirectBusinessPropertyId::Partial(_))) {
                        add(component.filer.gain);
                    }
                }
            }
            FederalIncome::FarmRental { data }
                if data.assets.iter().any(|asset| asset.partial_for().is_some()) =>
            {
                let rows = pool.farm_rental_property_rows.get(&entry.id).ok_or_else(|| {
                    crate::activity_risk::SourceIssue::new(
                        entry.id,
                        crate::activity_risk::ProjectionIssue::Source(
                            "DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE",
                        ),
                    )
                })?;
                for (_, gain) in &rows.partial_gains {
                    add(crate::extended_rules::annual_dollars_to_working_cents(*gain).map_err(
                        |error| crate::activity_risk::SourceIssue::new(entry.id, error.into()),
                    )?);
                }
            }
            FederalIncome::Rental { data }
                if rental_sale_refusal(
                    data,
                    crate::nonmacrs::rental_recovery_source(draft, entry.id, true).as_ref(),
                )
                .is_none()
                    && (data.buildings.iter().any(|building| building.partial.is_some())
                        || data.assets.iter().any(|asset| asset.partial.is_some())) =>
            {
                if let Some((source, rows)) = rental_allowed_property_rows(
                    crate::related_forms::RelatedFormContext::original(draft, source_claims),
                    pool,
                    entry.id,
                    data,
                    false,
                )? {
                    for (row, value) in &source.components {
                        match row {
                            FarmRentalPropertyRow::RentalPartial(_) => {
                                let gain = rows
                                    .character_rows
                                    .iter()
                                    .find(|(key, _, _)| key == row)
                                    .map_or(value.gain, |(_, _, amount)| *amount);
                                add(crate::extended_rules::annual_dollars_to_working_cents(gain)
                                    .map_err(|error| {
                                    crate::activity_risk::SourceIssue::new(entry.id, error.into())
                                })?);
                            }
                            FarmRentalPropertyRow::Partial(index) => {
                                let (_, gain) = rows
                                    .partial_gains
                                    .iter()
                                    .find(|(key, _)| key == index)
                                    .expect("actual asset partial row");
                                add(crate::extended_rules::annual_dollars_to_working_cents(*gain)
                                    .map_err(|error| {
                                        crate::activity_risk::SourceIssue::new(
                                            entry.id,
                                            error.into(),
                                        )
                                    })?);
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(totals)
}

// An installment sale's Form 6252 and its Form 4797 line 4 amount; Part
// III line 34 stays empty.
fn report_installment(
    r: &mut FederalResult,
    prefix: &str,
    input: &str,
    form: &crate::kernel::installment::Schedule,
    unrecaptured: i64,
) {
    crate::installment_return::report_sale(
        r,
        &format!("{prefix}_form6252"),
        input,
        form,
        unrecaptured,
        true,
    );
    row(
        r,
        "4",
        &format!("{prefix}_installment_gain"),
        form.gain,
        input,
        msg!("EXPLAIN_N_INSTALLMENT_GAIN"),
    );
}

// One sold or condemned rental's refusals and Form 4797 rows.
pub(crate) fn report_rental(
    d: &FederalDraft,
    r: &mut FederalResult,
    id: i64,
    label: &str,
    v: &FederalRental,
) {
    #[cfg(test)]
    let section = format!("Income: {label}");
    let context = crate::IssueContext::income_id(id);
    let recovery_source = crate::nonmacrs::rental_recovery_source(d, id, true);
    let source = recovery_source.as_ref();
    if let Some((key, _message)) = rental_sale_refusal(v, source) {
        #[cfg(test)]
        let message = _message;

        crate::issue_localized!(r, &context, &section, key, message,);
        return;
    }
    let mid_quarter = crate::depreciation::mid_quarter(d);
    for index in 0..v.assets.len() {
        report_business_asset_source(
            r,
            #[cfg(test)]
            &section,
            &context,
            &format!("rental_{id}_asset_{index}"),
            index,
            &v.assets,
            0,
            rental_asset_sale(&v.assets[index], index, mid_quarter, false, source),
            v.assets[index].partial_for(),
            "income.rental.assets",
            source,
        );
    }
    let name = crate::MessageArg::Text(label.into());
    // A part of a building replaced or removed, whether or not the property
    // was sold afterwards.
    match rental_partials(v, false, source) {
        Ok(parts) => {
            for (index, part) in &parts {
                report_partial(
                    r,
                    &context,
                    &format!("rental_{id}_building_{index}"),
                    "income.rental",
                    name.clone(),
                    part,
                    0,
                );
            }
        }
        Err((key, _message)) => {
            #[cfg(test)]
            let message = _message;

            crate::issue_localized!(r, &context, &section, key, message,);
            return;
        }
    }
    let value = match rental_disposition(v, false) {
        Ok(Some(value)) => value,
        Ok(None) => return,
        Err((key, _message)) => {
            #[cfg(test)]
            let message = _message;

            crate::issue_localized!(r, &context, &section, key, message,);
            return;
        }
    };
    let prefix = format!("rental_{id}_sale");
    let input = "income.rental.sale";
    // A like-kind exchange reports its Form 8824 and, when it qualifies,
    // only the recognized gain; one that fails continues as a sale.
    if let RentalSale::Exchanged(exchange) = &v.sale {
        exchange.report(r, id, label, v);
        if !matches!(exchange.outcome(), crate::exchange::Outcome::Sale { .. }) {
            return;
        }
    }
    if value.disallowed > 0 {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("NOTE_SALE_TO_RELATIVE_LOSS_DISALLOWED", alloc::vec![name.clone()]),
            context.clone(),
        );
    }
    if value.personal < 0 || (value.personal == 0 && value.personal_basis > value.personal_proceeds)
    {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new(
                "NOTE_RENTAL_SALE_PERSONAL_LOSS",
                alloc::vec![
                    name.clone(),
                    crate::MessageArg::Currency {
                        currency_code: "USD".into(),
                        scale: 0,
                        minor_units: value.personal_basis - value.personal_proceeds,
                    },
                ],
            ),
            context.clone(),
        );
    }
    if !value.short_rows.is_empty() {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new(
                "NOTE_RENTAL_SALE_IMPROVEMENT_ALLOCATION",
                alloc::vec![name.clone()],
            ),
            context.clone(),
        );
    }
    if matches!(&v.sale, RentalSale::Condemned { replacement_cost, .. } if *replacement_cost > 0)
        && value.depreciation > 0
    {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("NOTE_CONDEMNATION_RECAPTURE_LIMITED", alloc::vec![name.clone()]),
            context.clone(),
        );
    }
    for short in &value.short_rows {
        let Some((_, recognized)) = value
            .components
            .iter()
            .find(|(row, _)| *row == FarmRentalPropertyRow::RentalShort(short.index))
        else {
            crate::activity_risk::ProjectionIssue::Source(
                "DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE",
            )
            .report(id, r);
            return;
        };
        row(
            r,
            "11",
            &format!("{prefix}_row_{}_ordinary", short.index),
            recognized.ordinary,
            input,
            msg!("EXPLAIN_RENTAL_SALE_ROW_ORDINARY"),
        );
    }
    if value.postponed > 0 {
        push_line(
            r,
            None,
            &format!("{prefix}_postponed"),
            value.postponed * 100,
            &[input],
            "https://www.irs.gov/publications/p544",
            msg!("EXPLAIN_RENTAL_SALE_POSTPONED"),
        );
    }
    // A failed exchange continues as a sale of the realized parts.
    let exchanged = matches!(v.sale, RentalSale::Exchanged(_));
    let building_recognized = if exchanged { value.building } else { value.building_recognized };
    let land_recognized = if exchanged { value.land } else { value.land_recognized };
    if let Some(form) = &value.installment {
        report_installment(r, &prefix, input, form, value.unrecaptured);
        // A part sold at a loss is reported in full on line 2.
        if value.building <= 0 {
            row(
                r,
                "2",
                &format!("{prefix}_building"),
                building_recognized,
                input,
                msg!("EXPLAIN_N_BUILDING"),
            );
        }
        if value.land < 0 {
            row(r, "2", &format!("{prefix}_land"), land_recognized, input, msg!("EXPLAIN_N_LAND"));
        }
        return;
    }
    if value.building <= 0 {
        row(
            r,
            "2",
            &format!("{prefix}_building"),
            building_recognized,
            input,
            msg!("EXPLAIN_N_BUILDING"),
        );
    }
    if value.section1239 {
        row(
            r,
            "11",
            &format!("{prefix}_building_ordinary"),
            value.building,
            input,
            msg!("EXPLAIN_N_ORDINARY_RELATED"),
        );
    }
    if value.building > 0 && !value.section1239 {
        for (line, suffix, amount, explanation) in [
            ("22", "sale_price", value.realized, msg!("EXPLAIN_ASSET_SALE_N")),
            ("23", "cost", value.basis, msg!("EXPLAIN_ASSET_SALE_N_2")),
            ("24", "depreciation", value.depreciation, msg!("EXPLAIN_ASSET_SALE_N_3")),
            (
                "25",
                "adjusted_basis",
                value.basis - value.depreciation,
                msg!("EXPLAIN_ASSET_SALE_N_4"),
            ),
            ("26", "gain", value.building, msg!("EXPLAIN_ASSET_SALE_N_5")),
        ] {
            row(r, line, &format!("{prefix}_building_{suffix}"), amount, input, explanation);
        }
    }
    if value.building > 0 && value.additional > 0 && !value.section1239 {
        row(
            r,
            "28a",
            &format!("{prefix}_building_additional_depreciation"),
            value.additional,
            input,
            msg!("EXPLAIN_ASSET_ADDITIONAL_DEPRECIATION"),
        );
        row(
            r,
            "28g",
            &format!("{prefix}_building_1250_recapture"),
            value.ordinary,
            input,
            msg!("EXPLAIN_N_N_RECAPTURE"),
        );
    }
    row(r, "2", &format!("{prefix}_land"), land_recognized, input, msg!("EXPLAIN_N_LAND"));
    push_line(
        r,
        None,
        &format!("{prefix}_unrecaptured_1250"),
        value.unrecaptured * 100,
        &[input],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_N_UNRECAPTURED_N"),
    );
}

// Reg. 1.1411-4(g)(6) and (g)(7)(i)(B): the gain on disposing of self-rented
// property leaves NII, and so does the gain of a real estate professional who
// meets the 500-hour test in the year of the disposition (i8960, Safe Harbor
// for Real Estate Professionals: the gain or loss is deemed derived from
// property used in the ordinary course of a trade or business, Form 8960 line
// 5b). Reuse the source owner's section 1231 tracing, including lookback
// recapture.
pub(crate) struct SelfRentalNiit {
    pub excluded: [i64; 2],
    pub current_components_known_zero: bool,
}

pub(crate) fn self_rental_niit_with_claims(
    d: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: &crate::section1231::Result1231,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<SelfRentalNiit, crate::section1231::SourceError> {
    let mut excluded = [0, 0]; // ordinary, capital
    let mut current_components_known_zero = true;
    for (id, gain) in allowed_rental_sources(
        crate::related_forms::RelatedFormContext::original(d, source_claims),
        pool,
        false,
    )? {
        if gain.gain <= 0 || !d.income.iter().any(|e| e.id == id && excludes_gain(d, e)) {
            continue;
        }
        let ordinary = match crate::section1231::activity_character_with_claims(
            d,
            pool,
            character,
            id,
            false,
            source_claims,
        )? {
            crate::section1231::PropertyCharacter::Capital => 0,
            crate::section1231::PropertyCharacter::Ordinary => gain.section1231,
            crate::section1231::PropertyCharacter::Recaptured(amount) => amount,
        };
        let source = [gain.ordinary + ordinary, gain.section1231 - ordinary];
        current_components_known_zero &= source == [0; 2];
        excluded[0] += source[0];
        excluded[1] += source[1];
    }
    Ok(SelfRentalNiit { excluded, current_components_known_zero })
}

// Whether the disposition gain of a rental entry is outside net investment
// income: the property is self-rented, or its owner is a real estate
// professional who meets the 500-hour test this year.
fn excludes_gain(d: &FederalDraft, entry: &crate::FederalIncomeEntry) -> bool {
    let FederalIncome::Rental { data } = &entry.income else {
        return false;
    };
    data.property_type == crate::RentalPropertyType::SelfRental
        || (data.participation.professional()
            && data.property_type.business_participation().is_none()
            && d.professional(crate::community::professional_owner(
                d.setup.as_ref(),
                entry.owner.owned(),
            ))
            .is_some_and(|p| p.over_500_hours))
}

// Actual amortized property input, without manufacturing a depreciable equipment
// record. The cost owner establishes basis and recovery; this owner applies
// disposal recognition, buyer, holding period and the existing recapture engine.
pub(crate) struct AmortizedProperty<'a> {
    pub acquired: u32,
    pub end: &'a crate::AmortizationEnd,
    pub cost_cents: i64,
    pub recovery_cents: i64,
    // Section1254 applies to drilling/mining preference deductions; all other
    // admitted property here uses section1245. Eligibility stays with the source.
    pub section1254: bool,
}

pub(crate) struct AmortizedSale {
    pub value: crate::farm_land::LandGain,
    pub realized: i64,
    pub recognized_gain: i64,
    pub cost: i64,
    pub recovery: i64,
    pub postponed: i64,
    pub disallowed: i64,
    pub part_two: bool,
    pub installment: Option<crate::kernel::installment::Schedule>,
    pub section1254: bool,
}

pub(crate) fn amortized_property_sale(
    source: AmortizedProperty<'_>,
) -> Result<Option<AmortizedSale>, &'static str> {
    let end = source.end;
    if end.date / 10000 != crate::year_constants::TAX_YEAR {
        return Ok(None);
    }
    let (price, expenses, buyer, replacement, scrapped) = match &end.how {
        DisposalKind::Sold {
            proceeds,
            selling_expenses,
            buyer,
            livestock: SoldLivestock::No,
            ..
        } => (*proceeds, *selling_expenses, *buyer, None, false),
        DisposalKind::Condemned { award, expenses, replacement_cost } => {
            (*award, *expenses, Buyer::Unrelated, Some(*replacement_cost), false)
        }
        DisposalKind::Scrapped => (0, 0, Buyer::Unrelated, None, true),
        DisposalKind::PersonalUse => return Ok(None),
        _ => return Err("ISSUE_AMORTIZATION_DISPOSITION_SOURCE_NEEDED"),
    };
    if !end.how.valid()
        || source.cost_cents < 0
        || !(0..=source.cost_cents).contains(&source.recovery_cents)
    {
        return Err("ISSUE_AMORTIZATION_DISPOSITION_SOURCE_NEEDED");
    }
    let columns = PropertyColumns::from_cents(
        price,
        price - expenses,
        source.cost_cents,
        source.recovery_cents,
    );
    let PropertyColumns { realized, net, cost, recovery } = columns;
    let (gain, postponed, disallowed, section1239) =
        recognize_disposal(columns.gain(), net, replacement.map(dollars), buyer);
    // Arithmetic safety: reported Dollars are projected back into native Cents.
    if [realized, cost + columns.expense(), recovery, gain, postponed, disallowed]
        .iter()
        .any(|amount| amount.checked_mul(100).is_none())
    {
        return Err("ISSUE_AMORTIZATION_DISPOSITION_SOURCE_NEEDED");
    }
    let long = Livestock::No.held_long_enough(source.acquired, end.date);
    let part_two = scrapped || !long || section1239;
    // IRC1254(a)(1): smaller of gain and deducted exploration/development
    // costs; section1245 uses its registered shared class owner.
    let (recapture, ordinary, section1231) = if part_two {
        (0, gain, 0)
    } else if gain <= 0 {
        (0, 0, gain)
    } else if source.section1254 {
        let recapture = phase::cap(gain, [recovery]);
        (recapture, recapture, gain - recapture)
    } else {
        let v = crate::kernel::recapture::section1245(gain, recovery);
        (v.ordinary_dollars, v.ordinary_dollars, v.remaining1231_dollars)
    };
    let payment = match &end.how {
        DisposalKind::Sold { payment, .. } => {
            if payment.refusal(price).is_some() {
                return Err("ISSUE_AMORTIZATION_DISPOSITION_SOURCE_NEEDED");
            }
            Some(payment.clone())
        }
        _ => None,
    };
    let (installment, (recapture, ordinary, section1231, _)) = settle_installment(
        payment,
        InstallmentProperty {
            realized,
            cost,
            recovery,
            sale_expense: realized - net,
            gain,
            long,
            disallowed,
            section1239,
        },
        (recapture, ordinary, section1231, 0),
    );
    Ok(Some(AmortizedSale {
        value: crate::farm_land::LandGain {
            gain: ordinary + section1231,
            percent: 0,
            recapture,
            ordinary,
            section1231,
            unrecaptured: 0,
        },
        realized,
        recognized_gain: gain,
        cost: cost + realized - net,
        recovery,
        postponed,
        disallowed,
        part_two,
        installment,
        section1254: source.section1254,
    }))
}

pub(crate) fn report_amortized_sale(
    result: &mut FederalResult,
    prefix: &str,
    sale: &AmortizedSale,
    input: &str,
) {
    // 2026 draft Form4797 (October1):27b is1245,30b is1254.
    // https://www.irs.gov/pub/irs-dft/f4797--dft.pdf
    if sale.part_two {
        row(
            result,
            "11",
            &format!("{prefix}_ordinary"),
            sale.value.ordinary,
            input,
            msg!("EXPLAIN_N_ORDINARY_3"),
        );
    } else if sale.recognized_gain <= 0 {
        row(
            result,
            "2",
            &format!("{prefix}_section1231"),
            sale.value.section1231,
            input,
            msg!("EXPLAIN_N_SECTION1231"),
        );
    } else {
        for (line, suffix, amount, explanation) in [
            ("22", "realized", sale.realized, msg!("EXPLAIN_ASSET_SALE_N_6")),
            ("23", "cost", sale.cost, msg!("EXPLAIN_ASSET_SALE_N_7")),
            ("24", "recovery", sale.recovery, msg!("EXPLAIN_ASSET_SALE_N_8")),
            ("25", "adjusted_basis", sale.cost - sale.recovery, msg!("EXPLAIN_ASSET_SALE_N_4")),
            ("26", "gain", sale.recognized_gain + sale.postponed, msg!("EXPLAIN_ASSET_SALE_N_5")),
            (
                if sale.section1254 { "30b" } else { "27b" },
                "ordinary",
                sale.value.ordinary,
                msg!("EXPLAIN_N_ORDINARY_3"),
            ),
        ] {
            row(result, line, &format!("{prefix}_{suffix}"), amount, input, explanation);
        }
    }
    for (suffix, amount) in [("postponed", sale.postponed), ("disallowed", sale.disallowed)] {
        crate::line_emission::line(
            result,
            &format!("{prefix}_{suffix}"),
            amount,
            &[input],
            "https://www.irs.gov/instructions/i4797",
            msg!("EXPLAIN_ASSET_DISPOSAL_POSTPONED"),
        );
    }
    if let Some(form) = &sale.installment {
        crate::installment_return::report_sale(
            result,
            &format!("{prefix}_form6252"),
            input,
            form,
            0,
            true,
        );
    }
}

// The shared association kernel owns ID uniqueness even for a local property
// view. Parent/owner validation uses SourceAssociations in the normal Save path.
impl crate::kernel::associations::AssociationSource for BusinessAsset {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> crate::kernel::associations::SourceKind {
        crate::kernel::associations::SourceKind::Equipment
    }
    fn owner(&self) -> crate::kernel::source_fact::SourceFact<crate::Owner> {
        // A standalone asset does not own its containing activity's person fact.
        crate::kernel::source_fact::SourceFact::Unavailable
    }
    fn parent(
        &self,
        _: crate::kernel::associations::Relation,
    ) -> crate::kernel::source_fact::SourceFact<i64> {
        crate::kernel::source_fact::SourceFact::NotReported
    }
}
pub(crate) fn research_property(
    assets: &[BusinessAsset],
    id: i64,
) -> Result<&BusinessAsset, &'static str> {
    match crate::kernel::associations::AssociationView::new(assets).source(id) {
        crate::kernel::source_fact::SourceFact::Reported(asset) => Ok(asset),
        _ => Err("RESEARCH_TRANSFER_PROPERTY_NEEDED"),
    }
}

pub(crate) fn checked_property_sum(
    a: crate::farm_land::LandGain,
    b: crate::farm_land::LandGain,
) -> Option<crate::farm_land::LandGain> {
    let result = crate::farm_land::LandGain {
        gain: a.gain.checked_add(b.gain)?,
        percent: a.percent,
        recapture: a.recapture.checked_add(b.recapture)?,
        ordinary: a.ordinary.checked_add(b.ordinary)?,
        section1231: a.section1231.checked_add(b.section1231)?,
        unrecaptured: a.unrecaptured.checked_add(b.unrecaptured)?,
    };
    // Native reports project Dollars back into Cents; both stages must fit.
    [result.gain, result.recapture, result.ordinary, result.section1231, result.unrecaptured]
        .into_iter()
        .try_for_each(|amount| amount.checked_mul(100).map(|_| ()))?;
    Some(result)
}
