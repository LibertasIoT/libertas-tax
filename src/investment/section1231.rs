pub(crate) use crate::section1231_model::LossYear;
pub use crate::section1231_model::{Section1231History, Section1231LossYear};
// Coverage update QBI G7 direct rental (2026-10-17): allowed ordinary property
// income joins its existing qualified rental activity once; capital is excluded.
// Original property vintages, mixed-use depreciation and unresolved character
// retain named source boundaries. Evidence: src/tests/investment/qbi_direct_rental.rs;
// i8995 and Reg.1.199A-3(b)(2)(ii)(A). Full G7 remains open.
// Coverage update B9-7 rental-property checkpoint: one K-1 source aggregates box 1 and both
// rental property parts after their activity limits. Separate component character preserves
// business/rental QBI and NIIT attribution; unresolved rate-class and EBL allocations remain
// named refusals. Evidence: src/tests/passthrough/k1_rental.rs k1_rental_property_*; Pub925/i8582 2025.
// Native direct/rental property source attribution retains ordinary, long-term and25% class
// dollars by actual owner after the existing activity and character stages. The existing
// determined-capacity lookback rule is shared; ambiguous cross-owner recapture remains unknown.
// Tests: business.rs unit13_property_* (original asset figures, both claim selections, signed
// net-loss sources, reorder/restart and ambiguous lookback). K-1/earlier-sale adapters stay separate.
// Coverage update B9-7 business-property checkpoint (2026-10-12): Both K-1 kinds join one property pool/lookback/rate-class and NIIT owner. Partner ordinary property QBI uses its statement and linear loss limits; undetermined recapture refuses, without a proportional assumption.
//   Compound original business, section179 SALE and disposed rental gains share one
//   lookback with independent QBI/NII/AMT and capital netting; src/tests/passthrough/k1_compound.rs.
//   Opposite operating/property characters of one ordinary partnership use the
//   same gross-loss source proof as an S corporation, including independent AMT;
//   src/tests/passthrough/k1_property_offsets.rs. Two loss characters and broader sources remain open.
//   Loss-only nonpassive K-1 property sources defer current lookback and qualified
//   portions after activity limits; ordinary SALE amounts feed Form461 and Form172
//   as business income. Evidence: src/tests/passthrough/k1_property_ebl.rs; i461/i172/i8995.
//   Evidence: src/tests/passthrough/k1_items.rs and src/tests/investment/interview.rs k1_partner_property_*; i1065sk1/i4797/i8582/i8995 (2025 final), f4797 2026 draft. Full B9-7 acceptance remains open.
// Coverage: Form 4797 — Partial (audited 2026-09-24; T2-35 2026-09-26).
//   Lines: Part I: return-wide section 1231 netting, five-year nonrecaptured-loss lookback (FIFO,
//   expiry), business 1250 component, independent AMT history, for K-1 box 9 and farm-land sales.
//   Part II: farm land held one year or less, section 1244 stock losses (line 11, "Losses on
//   Section 1244 (Small Business) Stock", B7a: its own Investment sales kind, stock_loss.rs), open-account
//   shareholder-loan gains. Part III: only section 1252 farmland recapture. Refused explicitly (unit A6):
//   Part III section 1245/1250 recapture and Part IV 179/280F recapture, through the
//   AssetDisposed situation (M1-P3); T2-35 below computes Part III and refuses Part IV by name.
//   Evidence: section1231.rs, farm_land.rs, scorp_1231.rs, federal_rules.rs, 6735–6775,
//   13410–13425; src/tests/investment/passthrough.rs
//   section1231_matches_form_4797_2025_fifo_example_and_rolls_into_2026,
//   section1231_matches_publication_544_2025_lookback_example,
//   section1231_lookback_consumes_business_1250_before_preferential_gain; src/tests/farm/farm.rs
//   farm_land_one_year_boundary_routes_to_correct_form4797_character; sources i4797, Pub 544.
//   Unit T2-35 (2026-09-26, asset_sale.rs): direct dispositions. A continuing Schedule C asset or
//   actual-expense vehicle (BusinessAsset.disposal: Disposed { date, Sold | Scrapped |
//   PersonalUse }): 2026 depreciation by the disposal convention (half-year 1/2, the last recovery
//   year whole; mid-quarter 1.5/4.5/7.5/10.5 of 12; 39-year mid-month), business share of the amount
//   realized; held one year or less or scrapped: Part II line 11; gain: Part III lines 22-26, 1245
//   line 27b = min(gain, line 24) ordinary, the rest line 34 to Part I; 39-year straight-line: line
//   28g zero, unrecaptured 1250 = min(gain, line 24); loss: Part I line 2; personal part's loss not
//   deductible; PersonalUse: no gain, depreciation stops. A whole rental sold (FederalRental.sale:
//   price, land part, selling expenses by price): building Part III with depreciation allowed or
//   allowable through the month of sale, land Part I line 2, all section 1231, unrecaptured 1250
//   per property; section 469(g) in extended_rules (activity net + prior loss + gain: overall loss
//   nonpassive, overall gain passive income; own losses allowed), NII included. New sources feed
//   netting, lookback, rate classes, QBI (1245 recapture in business QBI), NIIT, investment
//   interest, section 179 income and MAGI through asset_sale::direct_sources/rental_sources.
//   Line numbers follow the 2026 draft (IRS f4797--dft.pdf, Cat. No. 13086I, "Created 10/1/26",
//   PDF modified 2026-10-07, retrieved 2026-10-08): the new line 7 (livestock) shifts Part I
//   combine/nonrecaptured/subtract to 8/9/10; Part II ordinary line 11; Part III is lines 21-34
//   (22 price, 23 cost, 24 depreciation, 25 adjusted basis, 26 gain, 27b 1245, 28g 1250, 34 to line 6).
//   Part IV is lines 35-37. Row ids form4797_lineN use the 2026 numbers.
//   Lines 8, 9 and 10 emitted. Lookback history enters as the BusinessPropertyLosses kind.
//   Unit B11 4797 (2026-10-12): the thirteen disposition refusals are computed (map
//   tax-forms/business-disposition-specials.md). Placed and sold in 2026: no MACRS or bonus, the
//   section 179 stays and is recaptured (Reg. 1.168(d)-1(b)(3)(ii), 1.179-1(e)(3)); an unused 179
//   carryforward of a disposed asset leaves Form 4562 line 10 and restores basis (Reg.
//   1.179-3(f)(1)); a mid-quarter asset's last short year is prorated to the disposal quarter
//   (Pub 946); the personal part's gain is a Form 8949 row; installment sales of property held one
//   year or less (line 11) and of mixed-use property (business part only); Part IV column (a)
//   section 179 recapture (lines 35(a)-37(a)) on property kept at 50% or less or for personal use;
//   a rental held one year or less (line 11, price divided by cost), sold after personal use (a
//   required personal-use share; Form 8949 for the personal part), under the RE-professional NII
//   safe harbor (Form 8960 line 5b), by installment with a loss on one part (Form 6252 for the
//   gaining part) or with suspended losses (section 469(g)(3) ratio); condemnations (section 1033
//   recognized gain, recapture limited to it); sales to a relative (section 267 loss disallowed)
//   or a controlled entity (section 1239 ordinary gain); partial dispositions of a building (Reg.
//   1.168(i)-8; lines 1b and 1c); a Form 2106 vehicle's sale and Part IV column (b); equipment
//   rented out outside a business (recapture on line 13, the rest Form 8949); the whole-business
//   sale record (business_sale.rs); the earlier-year installment QBI. Not calculated: a
//   rental sale with a rehabilitation credit on installment, exchange or replaced condemnation,
//   a partial disposition on an exchanged rental or after the sale; a business without material
//   participation (left out earlier); AssetDisposed narrowed to purchased section 197
//   intangibles and sections 1254/1255; installment sales to related buyers, dealers, contingent
//   price and like-kind exchanges and casualties keep their refusals; sections 1252 (farm land
//   aside)/1254/1255 have no entry. 280F caps are not prorated in the year of
//   disposition. Tests: src/tests/investment/business.rs f1_ to f9_, src/tests/investment/investment.rs f5_ to f10_,
//   src/tests/investment/sale_specials.rs, src/tests/investment/interview.rs *_keeps_the_*. Tests: src/tests/investment/business.rs
//   form_4797_business_asset_sale_matches_hand_computation,
//   asset_held_one_year_or_less_is_ordinary_on_part_ii_line_10,
//   section_1245_recapture_is_limited_to_depreciation, disposal_year_depreciation_follows_the_convention,
//   prior_section_1231_losses_make_part_of_the_asset_gain_ordinary,
//   stopped_business_use_ends_depreciation_without_a_sale,
//   asset_disposals_outside_the_calculation_are_refused_by_name,
//   vehicle_kept_for_personal_use_after_low_business_use_is_refused; src/tests/investment/investment.rs
//   form_4797_rental_sale_matches_hand_computation,
//   rental_sale_overall_loss_is_nonpassive_at_the_469g_boundary,
//   rental_unrecaptured_1250_gain_is_the_smaller_of_gain_and_depreciation,
//   rental_sales_outside_the_calculation_are_refused_by_name.
//   Unit40 (2026-10-12): a kept passive rental's partial building disposition
//   shares its Form 8582 activity after Form 6198. Part IX preserves Schedule E,
//   short-held Part II and section 1231 Part I losses separately; only allowed
//   property amounts enter netting, lookback, NIIT and lines 1b/1c. Component
//   gains remain reportable; professional nonpassive treatment is unchanged.
//   Tests: src/tests/investment/investment.rs unit40_* and recomputed f10_partial_disposition_*;
//   sources i8582 (Parts IV/V and IX), Pub 925, i4797, Reg. 1.168(i)-8.
//   T3-6252 (2026-09-27): an installment sale on the asset, rental or farm-land fold replaces the
//   source's gain by its 2026 part (Form 6252 line 26 to line 4; section 1245 recapture stays on
//   line 13; farm-land section 1252 recapture follows the payments, line 25 to line 15), and
//   earlier-year installment sales of business or rental property are their own sources
//   (installment::earlier_sources, exchange::related_sources) in the netting, rate classes, NIIT, Form 4952 check and rental
//   MAGI. The Form 6252 Coverage block is in installment.rs.
//   Unit B2 (2026-10-09): a rental exchanged for other real property (RentalSale::Exchanged)
//   recognizes Form 8824 line 23 as section 1231 gain on line 5 (row rental_N_exchange_section1231;
//   line 21, the Form 4797 line 17 ordinary part, is always zero for post-1986 straight-line real
//   property) with unrecaptured section 1250 gain = min(line 22, building gain, depreciation); a
//   failed exchange is the rental sale split of the realized gain. An earlier-year related-party
//   exchange of business or rental property is its own source (exchange::related_sources). The
//   Form 8824 Coverage block is in exchange.rs.
//   Unit form-3468 (2026-10-09): the section 50(c) basis reduction of an investment credit counts as
//   depreciation. A business asset or rental improvement with a Form 3468 credit shows line 23 as
//   its cost before the reduction and adds the net reduction (the reduction less the basis a Form
//   4255 recapture restores, section 50(c)(2)) to line 24, so the section 1245 recapture of line
//   27b includes it; a 39-year building or sold rental improvement shows the section 1250
//   additional depreciation on line 28a (depreciation allowed over straight line on the cost
//   before the reduction, i4797 line 26a) as ordinary income, line 28g, with the rest of the gain
//   up to the depreciation unrecaptured section 1250 gain (AssetGain.additional, RentalGain.additional;
//   a rental's ordinary part joins the property ordinary income, an installment sale or exchange
//   of a rental with a rehabilitation credit is refused). Tests: T/investment_credit.rs fixtures
//   C, C2 to C4 and F.
//   Line audit fix F13 (2026-10-10, 4797-D1 to D7): line 9 prints the whole nonrecaptured balance
//   (Result1231.lookback); Part IV prints on lines 35(b), 36(b), 37(b); selling expenses join line 23
//   and line 22 is the gross price; the Part I and II source rows are printed (S corporation box 9
//   and farm-land section 1231 on line 2, installment gains on line 4, short-held farm land and
//   the S corporation loan gain on line 11) so lines 2-7 add to line 8; an entered credit
//   reduction (clean vehicle) joins line 24, not a lower line 23; 15-year land improvements and
//   20-year farm buildings are section 1250 property (line 28a over straight line, 28g, a
//   section 179 deduction on 27b); DisposalKind::Sold.livestock sets the 24- and 12-month holding
//   periods of section 1231(b)(3) and prints livestock on lines 7 and 18. Not printed yet: lines
//   6, 12-14, 16, 19 and Part III columns (4797-G7). Tests: src/tests/investment/business.rs f13_*,
//   src/tests/cross/line_audit_f13.rs.

//   Form8379 source attribution also resolves retained installment/exchange
//   sellers through their native source IDs after the joint property character
//   stage; determined lookback capacities are reused, never prorated. Tests:
//   tests/injured_spouse/unit13_earlier_property_sources.rs.
// The section 1231 gains or losses that a main home's sale carries to Form 4797
// (a separate business or rental space, or a rental exchanged for like-kind
// property), whether each is a business you took part in, keyed by the sale's
// income entry. A sale left out or refused has none.
pub(crate) fn property_sources(
    draft: &crate::FederalDraft,
) -> alloc::vec::Vec<(i64, crate::farm_land::LandGain, bool)> {
    let Some(status) = draft.setup.as_ref().map(|setup| setup.filing_status()) else {
        return alloc::vec::Vec::new();
    };
    draft
        .income
        .iter()
        .filter_map(|entry| {
            let crate::FederalIncome::HomeSale { data } = &entry.income else {
                return None;
            };
            let data = &HomeSale::from(data);
            if !data.ordinary || data.facts.as_ref().is_some_and(|f| f.left_out().is_some()) {
                return None;
            }
            let excluded = data.excluded(status).ok()?;
            let ((gain, unrecaptured), business) = match data.separate_form() {
                Some(form) => (form, matches!(data.separate.as_ref()?.kind, SeparateUse::Business)),
                None => (data.exchange_1231(excluded)?, false),
            };
            (gain != 0).then_some((
                entry.id,
                crate::farm_land::LandGain {
                    gain,
                    percent: 0,
                    recapture: 0,
                    ordinary: 0,
                    section1231: gain,
                    unrecaptured,
                },
                business,
            ))
        })
        .collect()
}

// The investments' section 1231 gains as Form 4797 Part I sources (whole dollars): an
// investment, not a business, so the gain is net investment income. These join the
// earlier-year sources.
pub(crate) fn sources(draft: &FederalDraft) -> Vec<(i64, crate::farm_land::LandGain, bool)> {
    draft
        .income
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::OpportunityFund { data } if data.valid() && !data.left_out() => {
                let gain = crate::kernel::ratio::dollars(data.inclusion()?.section_1231);
                (gain != 0).then_some((
                    entry.id,
                    crate::farm_land::LandGain {
                        gain,
                        percent: 0,
                        recapture: 0,
                        ordinary: 0,
                        section1231: gain,
                        unrecaptured: 0,
                    },
                    false,
                ))
            }
            _ => None,
        })
        .collect()
}

// Earlier-year installment sales and related-party exchanges of business or
// rental property, and opportunity fund inclusions, as section 1231 sources
// (Form 4797 lines 4 and 5), each with whether it is a business you materially
// participated in.
fn earlier_sources(draft: &FederalDraft) -> Vec<(i64, crate::farm_land::LandGain, bool)> {
    let mut sources = crate::installment_return::earlier_sources(draft);
    sources.extend(crate::exchange::related_sources(draft));
    // A section 1231 gain recognized from a qualified opportunity fund investment (special gain
    // code B) is an investment, not a business: net investment income, not a passive pool.
    sources.extend(self::sources(draft));
    // A separate business or rental space sold with a main home, or a rental
    // exchanged for like-kind property after the home exclusion (Pub. 523).
    sources.extend(property_sources(draft));
    sources
}

// https://www.irs.gov/instructions/i8824 (Exchanges limited to real property)
// https://www.ecfr.gov/current/title-26/section-1.1400Z2(a)-1
// Related 2024/2025 exchanges are real property, not livestock (Form 8824
// instructions: section 1031 is limited to real property since 2018). QOF
// inclusions retain their original attributes under 26 CFR 1.1400Z2(a)-1(c)(1);
// code B alone cannot establish the original property's livestock classification.
pub(crate) fn earlier_farmer_gross(d: &FederalDraft) -> Result<i64, &'static str> {
    if self::sources(d).iter().any(|(_, gain, _)| gain.gain > 0) {
        return Err("ISSUE_FARMER_GROSS_SOURCE_UNAVAILABLE");
    }
    crate::installment_return::farmer_livestock_gross(d)
}

// Shared Form 4797 Part I stage. Whole dollars, run independently for regular
// tax and AMT. Callers must first establish section 1231 eligibility, remove
// ordinary depreciation recapture, and apply the relevant loss limitations.
// https://www.irs.gov/publications/p544
// https://www.irs.gov/instructions/i4797
// Business-asset 1250 component: Schedule D worksheet lines 5 through 9.
// Portfolio 1250 sources on lines 10/11 do not belong in this calculation.
use crate::home_sale::{HomeSale, SeparateUse};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
use crate::line_emission::emit as emit_lines;
use crate::{FederalDraft, FederalIncome, FederalResult, Message, MessageArg};
use alloc::{boxed::Box, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// The page's "Business-property losses from 2021–2025" kind (question C17).
// Adding the kind is the answer: its two lists, which may be empty, are the
// confirmed history (an empty pair confirms no eligible loss), while leaving
// the kind out leaves the history missing.
/// Business-property losses from 2021–2025
/// Enter each year's net loss after return-wide netting, from the same taxpayer or unchanged joint-filing couple, counting losses from businesses you no longer run. Leave out losses still suspended by basis, at-risk or passive rules. Leave both lists empty when no 2021–2025 return deducted a net loss from business-property sales (Form 4797 line 7, carried to line 11).
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessPropertyLosses {
    /// Regular-tax net losses from 2021 through 2025
    // Tax structure: section 1231(c) uses the five preceding years, one unique balance per year.
    #[libertas_size(max = 5)]
    /// ----
    /// Loss year
    pub regular: Vec<Section1231LossYear>,
    /// AMT net losses from 2021 through 2025
    /// Use the retained AMT Form 4797 history; never substitute the regular-tax balances.
    // Tax structure: section 1231(c) uses the five preceding years, one unique balance per year.
    #[libertas_size(max = 5)]
    /// ----
    /// Loss year
    pub amt: Vec<Section1231LossYear>,
}

impl BusinessPropertyLosses {
    // The stored lookback record this answer writes.
    pub(crate) fn history(&self) -> Section1231History {
        Section1231History { regular: self.regular.clone(), amt: self.amt.clone() }
    }
    // The answer a stored record reopens as.
    pub(crate) fn from_history(history: &Section1231History) -> Self {
        Self { regular: history.regular.clone(), amt: history.amt.clone() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Item {
    // Eligible recognized net amount after ordinary depreciation recapture.
    // A K-1 activity may aggregate multiple underlying property dispositions.
    pub net: i64,
    // Business-asset 1250 component before taxpayer-level section 1231 netting.
    // May exceed this activity's net when other assets have losses.
    pub unrecaptured: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LossUse {
    pub year: u32,
    pub original: i64,
    pub opening: i64,
    pub recaptured: i64,
    // Includes a balance expiring after the current year's last opportunity
    // to use it. Opening = recaptured + expired + carried.
    pub expired: i64,
    pub carried: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CentResult1231 {
    pub net: i64,
    pub ordinary: i64,
    pub long_term: i64,
    pub unrecaptured: i64,
    pub lookback: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Result1231 {
    // Actual cent netting precedes the reported whole-Dollar form columns.
    pub source_cents: Option<CentResult1231>,
    pub year: u32,
    pub net: i64,
    // Negative for a current net loss; positive for lookback recapture.
    pub ordinary: i64,
    pub long_term: i64,
    // Business-asset component only, before Schedule D's other loss offsets.
    pub unrecaptured: i64,
    pub lookback: i64,
    pub history: Vec<LossUse>,
    // Older losses still eligible next year. Current net losses are finalized
    // separately after their deductibility has been established by the return.
    pub retained: Vec<LossYear>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Error1231 {
    InvalidHistory,
    InvalidAmount,
    Overflow,
}

pub(crate) fn calculate(
    year: u32,
    items: &[Item],
    losses: &[LossYear],
) -> Result<Result1231, Error1231> {
    let first = year.checked_sub(5).ok_or(Error1231::InvalidHistory)?;
    if first == 0
        || losses.iter().any(|v| {
            v.year == 0
                || v.year >= year
                || v.original <= 0
                || v.remaining < 0
                || v.remaining > v.original
        })
    {
        return Err(Error1231::InvalidHistory);
    }
    let mut losses = losses.to_vec();
    losses.sort_by_key(|v| v.year);
    if losses.windows(2).any(|v| v[0].year == v[1].year) {
        return Err(Error1231::InvalidHistory);
    }
    let mut net = 0i128;
    let mut unrecaptured = 0i64;
    for item in items {
        if item.unrecaptured < 0 {
            return Err(Error1231::InvalidAmount);
        }
        net = net.checked_add(i128::from(item.net)).ok_or(Error1231::Overflow)?;
        unrecaptured = unrecaptured.checked_add(item.unrecaptured).ok_or(Error1231::Overflow)?;
    }
    let net = i64::try_from(net).map_err(|_| Error1231::Overflow)?;
    // A net loss must be representable as a positive historical balance too.
    if net == i64::MIN {
        return Err(Error1231::Overflow);
    }
    let mut lookback = 0i64;
    for v in losses.iter().filter(|v| v.year >= first) {
        lookback = lookback.checked_add(v.remaining).ok_or(Error1231::Overflow)?;
    }
    let recaptured = phase::cap(phase::floor(net, 0), [lookback]);
    let mut unused_gain = recaptured;
    let mut history = Vec::with_capacity(losses.len());
    let mut retained = Vec::new();
    for v in losses {
        let used = if v.year >= first { phase::cap(unused_gain, [v.remaining]) } else { 0 };
        unused_gain -= used;
        let settlement = crate::kernel::carry::settle_values::<crate::kernel::carry::Cents>(
            v.remaining,
            used,
            v.year <= first,
        )
        .map_err(|_| Error1231::InvalidAmount)?;
        let carried = settlement.closing.get();
        history.push(LossUse {
            year: v.year,
            original: v.original,
            opening: v.remaining,
            recaptured: used,
            expired: settlement.expired.get(),
            carried,
        });
        if carried > 0 {
            retained.push(LossYear { remaining: carried, ..v });
        }
    }
    Ok(Result1231 {
        source_cents: None,
        year,
        net,
        ordinary: if net < 0 { net } else { recaptured },
        long_term: phase::floor(net, 0) - recaptured,
        // Worksheet line 7 first caps the business 1250 pool by net 1231
        // gain, then line 8 lookback reduces that pool before other LTCG.
        unrecaptured: phase::floor(
            phase::cap(unrecaptured, [phase::floor(net, 0)]) - recaptured,
            0,
        ),
        lookback,
        history,
        retained,
    })
}

impl Result1231 {
    pub(crate) fn in_cents(&self) -> Result<CentResult1231, SourceError> {
        if let Some(value) = self.source_cents {
            return Ok(value);
        }
        let lift = crate::extended_rules::annual_dollars_to_working_cents;
        let failure = |_| {
            SourceError::from((
                "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
                "The property working amount exceeds its native range.",
            ))
        };
        Ok(CentResult1231 {
            net: lift(self.net).map_err(failure)?,
            ordinary: lift(self.ordinary).map_err(failure)?,
            long_term: lift(self.long_term).map_err(failure)?,
            unrecaptured: lift(self.unrecaptured).map_err(failure)?,
            lookback: lift(self.lookback).map_err(failure)?,
        })
    }

    fn from_cents(value: Self) -> Self {
        Self {
            year: value.year,
            net: dollars(value.net),
            ordinary: dollars(value.ordinary),
            long_term: dollars(value.long_term),
            unrecaptured: dollars(value.unrecaptured),
            lookback: dollars(value.lookback),
            history: value
                .history
                .iter()
                .map(|row| LossUse {
                    year: row.year,
                    original: dollars(row.original),
                    opening: dollars(row.opening),
                    recaptured: dollars(row.recaptured),
                    expired: dollars(row.expired),
                    carried: dollars(row.carried),
                })
                .collect(),
            retained: value
                .retained
                .iter()
                .map(|row| LossYear {
                    year: row.year,
                    original: dollars(row.original),
                    remaining: dollars(row.remaining),
                })
                .collect(),
            source_cents: Some(CentResult1231 {
                net: value.net,
                ordinary: value.ordinary,
                long_term: value.long_term,
                unrecaptured: value.unrecaptured,
                lookback: value.lookback,
            }),
        }
    }

    // Do not manufacture a new lookback balance merely because Part I has a
    // loss. The return must establish the amount taken into account in taxable
    // income under section 1231(a)(4). This stage does not decide later limits.
    pub(crate) fn next_history(
        &self,
        deductible_current_loss: i64,
    ) -> Result<Vec<LossYear>, Error1231> {
        let loss = phase::floor(self.net.checked_neg().ok_or(Error1231::Overflow)?, 0);
        if deductible_current_loss < 0 || deductible_current_loss > loss {
            return Err(Error1231::InvalidAmount);
        }
        let mut next = self.retained.clone();
        if deductible_current_loss > 0 {
            next.push(LossYear {
                year: self.year,
                original: deductible_current_loss,
                remaining: deductible_current_loss,
            });
        }
        Ok(next)
    }
}

// Rental-allowance MAGI is figured without passive activity items (Pub. 925).
// The pool already adds raw nonpassive ordinary/property net. Replace the
// property share with its effect after nonpassive section 1231 character and
// portfolio capital-loss coordination. This projection never depends on the
// allowance being computed and uses regular facts even for the AMT allowance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceError {
    source_owned: bool,
    pub context: Box<crate::IssueContext>,
    pub detail: (&'static str, &'static str),
}

impl SourceError {
    fn income(id: i64, detail: (&'static str, &'static str)) -> Self {
        Self { source_owned: false, context: Box::new(crate::IssueContext::income_id(id)), detail }
    }
    fn required(id: i64, detail: (&'static str, &'static str)) -> Self {
        Self { source_owned: true, ..Self::income(id, detail) }
    }
    pub(crate) fn is_required_source(&self) -> bool {
        self.source_owned
    }
    pub(crate) fn report_at(
        &self,
        r: &mut FederalResult,
        context: &crate::IssueContext,
        #[cfg(test)] section: &str,
    ) {
        let context = if self.source_owned
            || matches!(self.context.source(), crate::IssueContext::Income { .. })
        {
            &self.context
        } else {
            context
        };
        if !self.source_owned
            || !r.issues.iter().any(|issue| {
                issue.context.identity() == context.identity() && issue.display.key == self.detail.0
            })
        {
            crate::issue_localized!(r, context, section, self.detail.0, self.detail.1);
        }
        if self.source_owned {
            crate::federal_rules::withhold_required_source_result(r);
        }
    }
    fn with_manual(mut self) -> Self {
        self.context = Box::new((*self.context).manual());
        self
    }
}

impl From<(&'static str, &'static str)> for SourceError {
    fn from(detail: (&'static str, &'static str)) -> Self {
        Self {
            source_owned: false,
            context: Box::new(crate::IssueContext::Section1231.manual()),
            detail,
        }
    }
}

impl From<crate::activity_risk::SourceIssue> for SourceError {
    fn from(error: crate::activity_risk::SourceIssue) -> Self {
        let mut result = Self::required(error.source, (
            error.issue.key(),
            crate::fixture_english!("Resolve the actual rental source facts or representable loss amounts before completing property calculations."),
        )).with_manual();
        if let Some(context) = error.context {
            result.context = context;
        }
        result
    }
}

#[cfg(test)]
pub(crate) fn rental_magi_adjustment(
    draft: &FederalDraft,
    portfolio: crate::capital::CapitalAmounts,
    prior_capital: [i64; 2],
    nol_modification: bool,
) -> Result<i64, SourceError> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    rental_magi_adjustment_with_claims(
        draft,
        portfolio,
        prior_capital,
        nol_modification,
        &source_claims,
        None,
    )
}

pub(crate) fn rental_magi_adjustment_with_claims(
    draft: &FederalDraft,
    portfolio: crate::capital::CapitalAmounts,
    prior_capital: [i64; 2],
    nol_modification: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
    selected: Option<&crate::Section179PreparedAllocation>,
) -> Result<i64, SourceError> {
    let lift = crate::extended_rules::annual_dollars_to_working_cents;
    let overflow = |_| {
        SourceError::from((
            "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
            "The property working amount exceeds its native range.",
        ))
    };
    let participation =
        selected.map_or_else(|| draft.direct_participation_view(), |source| source.participation());
    let mut items = Vec::new();
    for entry in &crate::community::property_entries(draft) {
        if let FederalIncome::Scorp { data } = &entry.income
            && crate::ScorpK1::from(data).section1231.as_ref().is_some_and(|v| v.has_items())
        {
            let data = &crate::ScorpK1::from(data);
            match data.nonpassive() {
                crate::participation::Participation::No => continue,
                crate::participation::Participation::Unavailable => {
                    return Err(SourceError::income(
                        entry.id,
                        (
                            "ISSUE_SECTION1231_RESOLVE_PARTICIPATION_BEFORE_CALCULATING_NONPASSIVE_PROPERTY_39FB4535",
                            crate::fixture_english!(
                                "Resolve participation before calculating nonpassive property income for rental MAGI."
                            ),
                        ),
                    ));
                }
                crate::participation::Participation::Yes => {}
            }
            let activity = data
                .pool_activity_result()
                .map_err(|detail| SourceError::income(entry.id, detail))?;
            let character = activity
                .characters
                .ok_or((
                    "ISSUE_SECTION1231_NONPASSIVE_PROPERTY_CHARACTER_IS_UNAVAILABLE_FOR_922C25EC",
                    crate::fixture_english!(
                        "Nonpassive property character is unavailable for rental MAGI."
                    ),
                ))
                .map_err(|detail| SourceError::income(entry.id, detail))?;
            items.push(Item {
                net: lift(dollars(character.nonpassive[1])).map_err(overflow)?,
                unrecaptured: 0,
            });
        }
    }
    let pool_net = items.iter().map(|item| item.net).sum::<i64>();
    let mut direct_ordinary = 0;
    let mut direct_net = 0;
    for entry in &crate::community::property_entries(draft) {
        if let FederalIncome::Business { data } = &entry.income {
            if data.current_passive_supported(participation.get(entry.id)) {
                continue;
            }
            if let Some(parts) =
                selected.and_then(|selected| selected.direct_property.get(&entry.id))
            {
                for part in parts {
                    let value =
                        crate::community::PropertyCents::limited(part).map_err(|error| {
                            crate::activity_risk::SourceIssue::new(entry.id, error.into())
                        })?;
                    direct_ordinary += value.ordinary;
                    direct_net += value.section1231;
                    items.push(Item { net: value.section1231, unrecaptured: value.unrecaptured });
                }
                continue;
            }
        }
        if let FederalIncome::FarmRental { data } = &entry.income {
            let source = crate::farm_rental::risk_source_with_claims(
                crate::related_forms::RelatedFormContext::original(draft, source_claims),
                entry.id,
                data,
                usize::from(entry.owner == crate::DocumentOwner::Spouses),
                crate::depreciation::mid_quarter(draft),
                false,
                None,
            )?;
            if crate::farm_rental::passive_property_gain(
                crate::related_forms::RelatedFormContext::original(draft, source_claims),
                entry.id,
                data,
                usize::from(entry.owner == crate::DocumentOwner::Spouses),
                crate::depreciation::mid_quarter(draft),
                &source,
                false,
            )? {
                continue;
            }
            if let Some(gain) = source.property {
                let gain =
                    crate::community::PropertyCents::for_source(draft, entry.id, gain, false)?;
                direct_ordinary += gain.ordinary;
                direct_net += gain.section1231;
                items.push(Item { net: gain.section1231, unrecaptured: gain.unrecaptured });
            }
            continue;
        }
        if let Some(data) = farm_income(&entry.income) {
            for sale in &data.land_sales {
                if !sale.valid() || !sale.situation.direct() {
                    return Err(SourceError::income(entry.id, (
                        "ISSUE_SECTION1231_RESOLVE_FARM_LAND_SALE_ELIGIBILITY_BEFORE_RENTAL_10544F0A",
                        crate::fixture_english!(
                            "Resolve farm-land sale eligibility before rental MAGI."
                        ),
                    )).with_manual());
                }
                let Some(sale) =
                    sale.resolve_for_return_with_claims(draft, entry.id, data, source_claims)
                else {
                    return Err(SourceError::income(entry.id, ("ISSUE_FARM_LAND_LINKED_BASIS_NEEDED", crate::fixture_english!("Resolve the linked farm parcel basis and exclusion history before rental MAGI."))).with_manual());
                };
                let gain = crate::community::PropertyCents::for_source(
                    draft,
                    entry.id,
                    sale.calculate(),
                    matches!(entry.income, FederalIncome::Business { .. }),
                )?;
                direct_ordinary += gain.ordinary;
                direct_net += gain.section1231;
                items.push(Item { net: gain.section1231, unrecaptured: 0 });
            }
        }
        if let FederalIncome::Business { data } = &entry.income {
            let gain = crate::cohorts::property_total(data).map_err(|key| {
                SourceError::income(
                    entry.id,
                    (
                        key,
                        crate::fixture_english!(
                            "Resolve original amortization disposition sources."
                        ),
                    ),
                )
            })?;
            if let Some(gain) = gain {
                let gain =
                    crate::community::PropertyCents::for_source(draft, entry.id, gain, true)?;
                direct_ordinary += gain.ordinary;
                direct_net += gain.section1231;
                items.push(Item { net: gain.section1231, unrecaptured: 0 });
            }
        }
        // Business-asset dispositions are nonpassive property income of
        // an operating business, like the direct land sales above.
        if let Some(assets) = direct_assets(&entry.income) {
            for (_, gain) in crate::asset_sale::purchase_asset_components_with_claims(
                draft,
                entry.id,
                assets,
                crate::depreciation::mid_quarter(draft),
                false,
                source_claims,
                None,
            )
            .map_err(|issue| property_source_error(entry.id, issue))?
            {
                let gain = crate::community::PropertyCents::for_source(
                    draft,
                    entry.id,
                    gain,
                    matches!(entry.income, FederalIncome::Business { .. }),
                )?;
                direct_ordinary += gain.ordinary;
                direct_net += gain.section1231;
                items.push(Item { net: gain.section1231, unrecaptured: gain.unrecaptured });
            }
        }
    }
    // Earlier-year installment gains of a business you materially
    // participated in are nonpassive; a rental's are passive and excluded.
    // A fund inclusion's section 1231 gain (code B) is an investment: not a
    // business, but nonpassive, so it is in the line 7 income too.
    let nonpassive = earlier_sources(draft)
        .into_iter()
        .filter(|(_, _, business)| *business)
        .chain(self::sources(draft));
    for (id, gain, _) in nonpassive {
        let gain = crate::community::PropertyCents::for_source(draft, id, gain, false)?;
        direct_net += gain.section1231;
        items.push(Item { net: gain.section1231, unrecaptured: gain.unrecaptured });
    }
    let net = pool_net + direct_net;
    // K-1 raw property income is added by the pool. Direct land-sale
    // amounts are absent from Schedule F and must be added here instead.
    if net <= 0 {
        return Ok(dollars(direct_ordinary + direct_net));
    }
    let history = draft.screening.as_ref().and_then(|s| s.section1231.as_ref())
        .filter(|history| history.valid())
        .ok_or(("ISSUE_SECTION1231_COMPLETE_REGULAR_SECTION_1231_LOOKBACK_HISTORY_400E96BC", crate::fixture_english!("Complete regular section 1231 lookback history before computing rental MAGI from nonpassive property gains.")))?;
    let losses = history
        .losses(false)
        .into_iter()
        .map(|row| {
            Ok(LossYear {
                year: row.year,
                original: lift(row.original).map_err(overflow)?,
                remaining: lift(row.remaining).map_err(overflow)?,
            })
        })
        .collect::<Result<Vec<_>, SourceError>>()?;
    let character = calculate(2026, &items, &losses).map_err(|_| {
        (
            "ISSUE_SECTION1231_THE_NONPASSIVE_PROPERTY_PROJECTION_CANNOT_BE_59B193F5",
            crate::fixture_english!(
                "The nonpassive property projection cannot be reconciled for rental MAGI."
            ),
        )
    })?;
    let separate = draft
        .setup
        .as_ref()
        .ok_or((
            "ISSUE_SECTION1231_COMPLETE_RETURN_SETUP_BEFORE_RENTAL_MAGI_35900792",
            crate::fixture_english!("Complete return setup before rental MAGI."),
        ))?
        .filing_status()
        == crate::FederalStatus::Separate;
    let before = portfolio.calculate(prior_capital, separate, nol_modification).income;
    let after = crate::capital::CapitalAmounts {
        long: portfolio.long + dollars(character.long_term),
        ..portfolio
    }
    .calculate(prior_capital, separate, nol_modification)
    .income;
    Ok(dollars(direct_ordinary + character.ordinary - pool_net) + after - before)
}

fn property_source_error(id: i64, issue: crate::Section179PreparedIssue) -> SourceError {
    let key = match issue {
        crate::Section179PreparedIssue::SourceBoundaryUnavailable(_, key) => key,
        _ => "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
    };
    SourceError::required(
        id,
        (
            key,
            crate::fixture_english!(
                "Resolve the actual property recovery and disposition records."
            ),
        ),
    )
}

// The farm records of a Schedule F farm or a Form 4835 farm rental, and the
// assets either may sell: the direct property sources.
fn farm_income(income: &FederalIncome) -> Option<&crate::FarmIncome> {
    match income {
        FederalIncome::Business { data } => match &data.activity {
            crate::BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        FederalIncome::FarmRental { data } => Some(&data.farm),
        _ => None,
    }
}
fn direct_assets(income: &FederalIncome) -> Option<&[crate::BusinessAsset]> {
    match income {
        FederalIncome::Business { data } => Some(&data.assets),
        FederalIncome::FarmRental { data } => Some(&data.assets),
        _ => None,
    }
}

// Form 1116 lines 3d/3e: retain positive source gains before return-wide
// netting. K-1 amounts come from the existing basis/risk/passive owner, never
// from raw losses that were suspended. This projection does not alter 4797.
#[cfg(test)]
pub(crate) fn gross_property(
    d: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
) -> Result<i64, crate::Section179PreparedIssue> {
    let source_claims = crate::related_forms::effective_claimants(d);
    gross_property_with_claims(d, pool, amt, &source_claims)
}

pub(crate) fn gross_property_with_claims(
    d: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::Section179PreparedIssue> {
    Ok(dollars(
        gross_property_sources_with_claims(d, pool, amt, false, source_claims)?.into_iter().sum(),
    ))
}

// Farmer gross uses the same admitted components, with the canonical gain
// fraction for physical Business sources. Other sources already are divided.
// Retain cents across source projection, then use the existing annual dollars.
#[cfg(test)]
pub(crate) fn farmer_gross_property(
    d: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
) -> Result<Option<i64>, crate::Section179PreparedIssue> {
    let source_claims = crate::related_forms::effective_claimants(d);
    farmer_gross_property_with_claims(d, pool, &source_claims)
}

pub(crate) fn farmer_gross_property_with_claims(
    d: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Option<i64>, crate::Section179PreparedIssue> {
    Ok(crate::kernel::ratio::checked_sum(gross_property_sources_with_claims(
        d,
        pool,
        false,
        true,
        source_claims,
    )?)
    .map(dollars))
}

// One positive-property pass supplies both projections. A Business record stays
// physical in community::property_entries; rental records already have their
// source fractions, so only the former carries an additional projection here.

fn gross_property_sources_with_claims(
    d: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
    allocated: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Vec<i64>, crate::Section179PreparedIssue> {
    let mid_quarter = crate::depreciation::mid_quarter(d);
    let mut sources = alloc::vec![(None, crate::asset_sale::rental_property_gross(d, amt))];
    for entry in &crate::community::property_entries(d) {
        let source = matches!(entry.income, FederalIncome::Business { .. }).then_some(entry.id);
        if let Some(assets) = direct_assets(&entry.income) {
            if allocated {
                sources.extend(
                    crate::asset_sale::purchase_asset_components_with_claims(
                        d,
                        entry.id,
                        assets,
                        mid_quarter,
                        amt,
                        source_claims,
                        pool.macrs.as_deref(),
                    )?
                    .into_iter()
                    .map(|(_, component)| (source, phase::floor(component.gain, 0))),
                );
            } else {
                sources.push((
                    source,
                    crate::asset_sale::purchase_asset_gross_with_claims(
                        d,
                        entry.id,
                        assets,
                        mid_quarter,
                        amt,
                        source_claims,
                        pool.macrs.as_deref(),
                    )?,
                ));
            }
            if let FederalIncome::Business { data } = &entry.income {
                sources.push((source, crate::cohorts::property_gross(data).unwrap_or(0)));
            }
        }
        if let Some(farm) = farm_income(&entry.income) {
            sources.extend(
                farm.land_sales
                    .iter()
                    .filter_map(|sale| {
                        if allocated {
                            sale.resolve_for_return_with_claims(d, entry.id, farm, source_claims)
                        } else {
                            sale.resolve_with_claims(d, entry.id, farm, source_claims)
                        }
                    })
                    .map(|sale| (source, phase::floor(sale.calculate_for(amt).gain, 0))),
            );
        }
    }
    if let Some(adjustments) = &d.adjustments {
        for claim in &adjustments.employee_expenses {
            sources.extend(
                crate::asset_sale::business_asset_gross_components(
                    &claim.assets(),
                    mid_quarter,
                    amt,
                )
                .map(|amount| (None, amount)),
            );
        }
    }
    for value in &pool.k1_section1231 {
        sources.push((None, phase::floor(value.business().net, 0)));
        sources.extend(value.rentals.iter().map(|rental| (None, phase::floor(rental.net, 0))));
    }
    sources.extend(
        earlier_sources(d).into_iter().map(|(_, gain, _)| (None, phase::floor(gain.gain, 0))),
    );
    let mut by_source = alloc::collections::BTreeMap::<i64, i64>::new();
    for (id, amount) in &sources {
        if let Some(id) = id {
            let total = by_source.entry(*id).or_default();
            *total = total
                .checked_add(*amount)
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(*id))?;
        }
    }
    sources.retain(|(id, _)| id.is_none());
    sources.extend(by_source.into_iter().map(|(id, amount)| (Some(id), amount)));
    sources
        .into_iter()
        .map(|(id, amount)| {
            let cents = crate::extended_rules::annual_dollars_to_working_cents(amount)
                .map_err(|_| crate::Section179PreparedIssue::AmountOverflow(id.unwrap_or(0)))?;
            Ok(if allocated {
                id.map_or(cents, |id| crate::community::SourceAmount::property(d, id, cents).filer)
            } else {
                cents
            })
        })
        .collect()
}

// Intermediate Form 4797 only. The caller must not add these amounts to the
// return until QBI, NIIT, Form 461 and loss-history finalization are connected.
#[cfg(test)]
pub(crate) fn worksheet(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
) -> Result<Option<Result1231>, SourceError> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    worksheet_with_claims(draft, pool, amt, &source_claims)
}

pub(crate) fn worksheet_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Option<Result1231>, SourceError> {
    worksheet_for_with_claims(draft, pool, amt, false, source_claims)
}

// Form 1062 refigures gain character, not AGI, deductions or activity limits.

pub(crate) fn without_elected_land_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Option<Result1231>, SourceError> {
    worksheet_for_with_claims(draft, pool, false, true, source_claims)
}

fn k1_property(income: &FederalIncome, amt: bool) -> Option<crate::k1_items::BusinessProperty> {
    match income {
        FederalIncome::Scorp { data } => {
            let data = &crate::ScorpK1::from(data);
            if amt {
                data.amt.as_ref()?.source(data).ok()?.business_property().ok()?
            } else {
                data.business_property().ok()?
            }
        }
        FederalIncome::Partnership { data } => {
            if amt {
                data.alternative.as_ref()?.business_property(&data.amt_items()).ok()?
            } else {
                data.regular.business_property(&data.items()).ok()?
            }
        }
        _ => None,
    }
}

// Business, box 2 and box 3 are separately traced inside one K-1 source.
// Round each entered amount once before combining sources.
fn k1_property_parts(income: &FederalIncome, amt: bool) -> [Option<(i64, i64)>; 3] {
    let rentals = match income {
        FederalIncome::Partnership { data } => {
            if amt {
                data.alternative.as_ref().map(|v| v.rentals.clone())
            } else {
                Some(data.regular.rentals.clone())
            }
        }
        FederalIncome::Scorp { data } => {
            let data = crate::ScorpK1::from(data);
            if amt { data.amt.map(|v| v.rentals) } else { Some(data.rentals) }
        }
        _ => None,
    };
    let mut parts = [None; 3];
    parts[0] = k1_property(income, amt).map(|p| (dollars(p.net()), dollars(p.unrecaptured)));
    if let Some(rentals) = rentals {
        for (i, p) in rentals.properties().into_iter().enumerate() {
            parts[i + 1] = p.has_items().then_some((dollars(p.net), dollars(p.unrecaptured)));
        }
    }
    parts
}

fn k1_property_totals(income: &FederalIncome, amt: bool) -> Option<(i64, i64)> {
    let parts = k1_property_parts(income, amt);
    parts
        .iter()
        .any(Option::is_some)
        .then(|| parts.into_iter().flatten().fold((0, 0), |a, p| (a.0 + p.0, a.1 + p.1)))
}

fn worksheet_for_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
    omit_elected_land: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Option<Result1231>, SourceError> {
    let fallback = pool.macrs.is_none().then(|| crate::macrs_source::MacrsElectionView::new(draft));
    let macrs = pool.macrs.as_deref().or(fallback.as_ref()).unwrap();
    let mid_quarter = crate::depreciation::mid_quarter_with_macrs(draft, Some(macrs));
    let history = draft.screening.as_ref().and_then(|s| s.section1231.as_ref());
    let rental_sources = crate::asset_sale::rental_sources(
        crate::related_forms::RelatedFormContext::original(draft, source_claims),
        false,
        Some(&pool.home_use),
    )?;
    let has_items = pool.farm_rental_property.values().any(|value| value.section1231 != 0)
        || crate::community::property_entries(draft).iter().any(|entry| match &entry.income {
        FederalIncome::Scorp { .. } | FederalIncome::Partnership { .. } => {
            k1_property_totals(&entry.income, false).is_some()
                || k1_property_totals(&entry.income, true).is_some()
        }
        FederalIncome::FarmRental { data } => {
            data.farm.land_sales.iter().filter_map(|sale| sale.resolve_for_return_with_claims(draft, entry.id, &data.farm, source_claims)).any(|sale| {
                sale.calculate().section1231 != 0 || sale.calculate_for(true).section1231 != 0
            }) || data.assets.iter().any(|asset| {
                asset
                    .sale_in_context(mid_quarter, false, macrs.context(entry.owner.owned(), Some(entry.id), asset))
                    .is_some_and(|v| v.section1231 != 0)
                    || asset
                        .sale_in_context(mid_quarter, true, macrs.context(entry.owner.owned(), Some(entry.id), asset))
                        .is_some_and(|v| v.section1231 != 0)
                    || asset
                        .partial_for()
                        .is_some_and(|part| part.land_gain().section1231 != 0)
            })
        }
        FederalIncome::Business { data } => {
            crate::cohorts::property_total(data).ok().flatten().is_some_and(|v| v.section1231 != 0) || matches!(
                &data.activity,
                crate::BusinessActivity::Farming { data } if data.land_sales.iter().filter_map(|sale| sale.resolve_for_return_with_claims(draft, entry.id, data, source_claims)).any(|sale|
                    sale.calculate().section1231 != 0 || sale.calculate_for(true).section1231 != 0)
            ) || data.assets.iter().any(|asset| {
                asset
                    .sale_in_context(mid_quarter, false, macrs.context(entry.owner.owned(), Some(entry.id), asset))
                    .is_some_and(|v| v.section1231 != 0)
                    || asset
                        .sale_in_context(mid_quarter, true, macrs.context(entry.owner.owned(), Some(entry.id), asset))
                        .is_some_and(|v| v.section1231 != 0)
                    || asset
                        .partial_for()
                        .is_some_and(|part| part.land_gain().section1231 != 0)
            })
        }
        FederalIncome::Rental { .. } => !rental_sources.is_empty(),
        _ => false,
    });
    let earlier = earlier_sources(draft);
    if history.is_none() && !has_items && earlier.is_empty() {
        return Ok(None);
    }
    let history = history.ok_or(
        ("ISSUE_SECTION1231_ADD__BUSINESS_PROPERTY_LOSSES_FROM_PRIOR_FIVE_YEARS__ONCE_F5B687DD", crate::fixture_english!("Add \"Business-property losses from 2021–2025\" once for the return: choose \"No net business-property loss on a 2021–2025 return\" or enter the losses.")),
    )?;
    if !history.valid() {
        return Err((
            "ISSUE_SECTION1231_COMPLETE_AND_RECONCILE_THE_SECTION_1231_50E2A327",
            crate::fixture_english!(
                "Complete and reconcile the section 1231 lookback records and current sources for these filers."
            ),
        ).into());
    }
    if !pool.inputs_complete {
        return Err((
            "ISSUE_SECTION1231_RESOLVE_ALL_PASSIVE_POOL_INPUT_AND_ACTIVITY_LIMIT_055CB3FF",
            crate::fixture_english!(
                "Resolve all passive-pool input and activity-limit issues before calculating the section 1231 lookback."
            ),
        ).into());
    }
    let lift = crate::extended_rules::annual_dollars_to_working_cents;
    let overflow = || {
        SourceError::from((
            "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
            "The property working amount exceeds its native range.",
        ))
    };
    let projection = crate::community::source_projection(draft, pool, amt, source_claims)?;
    let mut items = Vec::new();
    for entry in &crate::community::property_entries(draft) {
        if let Some((_, unrecaptured)) = k1_property_totals(&entry.income, amt) {
            let value = pool.k1_section1231.iter().find(|v| v.id == entry.id).ok_or((
                "ISSUE_SECTION1231_A_BUSINESS_PROPERTY_ACTIVITY_IS_MISSING_ITS_A7B2C775",
                crate::fixture_english!(
                    "A business-property activity is missing its resolved loss-limit result."
                ),
            ))?;
            items.push(Item {
                net: lift(value.net).map_err(|_| overflow())?,
                unrecaptured: lift(unrecaptured).map_err(|_| overflow())?,
            });
        }
    }
    for source in &projection.direct {
        let mut net = source.filer.section1231;
        if omit_elected_land
            && let Some(entry) = draft.income.iter().find(|entry| entry.id == source.income)
            && let Some(farm) = farm_income(&entry.income)
        {
            for (index, sale) in
                farm.land_sales.iter().enumerate().filter(|(_, sale)| sale.deferral)
            {
                let sale = sale.resolve_for_return_with_claims(draft, entry.id, farm, source_claims)
                        .ok_or(("ISSUE_FARM_LAND_LINKED_BASIS_NEEDED", "Resolve the linked farm parcel basis and exclusion history before section1231 netting."))?;
                let mut amount =
                    lift(sale.calculate_for(amt).section1231).map_err(|_| overflow())?;
                if matches!(entry.income, FederalIncome::FarmRental { .. }) {
                    let rows = pool.farm_rental_property_rows.get(&entry.id).ok_or_else(|| {
                        crate::activity_risk::SourceIssue::new(
                            entry.id,
                            crate::activity_risk::ProjectionIssue::Source(
                                "DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE",
                            ),
                        )
                    })?;
                    if let Some((_, _, allowed)) =
                        rows.character_rows.iter().find(|(row, class, _)| {
                            *row == crate::asset_sale::FarmRentalPropertyRow::Land(index)
                                && *class == 1
                        })
                    {
                        amount = lift(*allowed).map_err(|_| overflow())?;
                    }
                }
                net -= if matches!(entry.income, FederalIncome::Business { .. }) {
                    crate::community::SourceAmount::property(draft, entry.id, amount).filer
                } else {
                    amount
                };
            }
        }
        items.push(Item { net, unrecaptured: source.filer.unrecaptured });
    }
    // A sold rental's gain is recognized in full (section 469(g)).
    for (_, gain) in crate::asset_sale::allowed_rental_sources(
        crate::related_forms::RelatedFormContext::original(draft, source_claims),
        pool,
        amt,
    )? {
        items.push(Item {
            net: lift(gain.section1231).map_err(|_| overflow())?,
            unrecaptured: lift(gain.unrecaptured).map_err(|_| overflow())?,
        });
    }
    // Earlier-year installment sales (Form 6252 line 26 to line 4).
    for (_, gain, _) in earlier {
        items.push(Item {
            net: lift(gain.section1231).map_err(|_| overflow())?,
            unrecaptured: lift(gain.unrecaptured).map_err(|_| overflow())?,
        });
    }
    let losses = history
        .losses(amt)
        .into_iter()
        .map(|row| {
            Ok(LossYear {
                year: row.year,
                original: lift(row.original).map_err(|_| overflow())?,
                remaining: lift(row.remaining).map_err(|_| overflow())?,
            })
        })
        .collect::<Result<Vec<_>, SourceError>>()?;
    calculate(2026, &items, &losses).map(|value| Some(Result1231::from_cents(value))).map_err(
        |_| {
            SourceError::from((
                "ISSUE_SECTION1231_THE_SECTION_1231_HISTORY_OR_ACTIVITY_33E7BDF7",
                crate::fixture_english!(
                    "The section 1231 history or activity amounts cannot be reconciled."
                ),
            ))
        },
    )
}

// Potential section 199A items after section 1231 character determination.
// Current SSTB eligibility and later return limits remain separate. Do not net
// businesses together: they can have different qualified shares and wage limits.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PropertyQbiItem {
    pub id: i64,
    pub gain: i64,
    pub loss: i64,
    // The balance loss - current_loss retains original-year eligibility.
    pub current_loss: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PropertyCharacter {
    Capital,
    Ordinary,
    Recaptured(i64),
}

fn property_working_sources(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Vec<(i64, crate::community::PropertyCents)>, SourceError> {
    let mut values =
        crate::community::source_projection(draft, pool, amt, source_claims)?.cent_direct();
    let annual = crate::asset_sale::allowed_rental_sources(
        crate::related_forms::RelatedFormContext::original(draft, source_claims),
        pool,
        amt,
    )?
    .into_iter()
    .chain(earlier_sources(draft).into_iter().map(|(id, gain, _)| (id, gain)));
    for (id, value) in annual {
        values.push((
            id,
            crate::community::PropertyCents::annual(value)
                .map_err(|error| crate::activity_risk::SourceIssue::new(id, error.into()))?,
        ));
    }
    Ok(values)
}

pub(crate) fn activity_character_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: &Result1231,
    id: i64,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<PropertyCharacter, SourceError> {
    activity_character_cents_with_claims(draft, pool, character, id, amt, source_claims).map(
        |value| match value {
            PropertyCharacter::Recaptured(amount) => PropertyCharacter::Recaptured(dollars(amount)),
            value => value,
        },
    )
}

pub(crate) fn activity_character_cents_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: &Result1231,
    id: i64,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<PropertyCharacter, SourceError> {
    let working = character.in_cents()?;
    let character = &working;
    if character.net <= 0 {
        return Ok(PropertyCharacter::Ordinary);
    }
    if character.ordinary == 0 {
        return Ok(PropertyCharacter::Capital);
    }
    let direct = property_working_sources(draft, pool, amt, source_claims)?;
    let lift = crate::extended_rules::annual_dollars_to_working_cents;
    let mut amounts = pool
        .k1_section1231
        .iter()
        .map(|value| {
            Ok((
                value.id,
                lift(value.net).map_err(|error| {
                    crate::activity_risk::SourceIssue::new(value.id, error.into())
                })?,
            ))
        })
        .collect::<Result<Vec<_>, SourceError>>()?;
    amounts.extend(direct.iter().map(|(id, value)| (*id, value.section1231)));
    // Full recapture of nonnegative source results leaves no capital gain
    // to allocate: each business retains its own ordinary gain.
    if character.long_term == 0
        && amounts.iter().all(|(_, net)| *net >= 0)
        && let Some((_, net)) = amounts.iter().find(|(source, _)| *source == id)
    {
        return Ok(PropertyCharacter::Recaptured(*net));
    }
    // One business owns all current property amounts: no cross-business
    // allocation assumption is needed. Multiple businesses need source gains,
    // including their section 1250 classes, before assigning recapture.
    if amounts.len() == 1 && amounts[0].0 == id {
        return Ok(PropertyCharacter::Recaptured(character.ordinary));
    }
    rate_class_recapture_cents_with_claims(draft, pool, character, id, amt, None, source_claims)
        .map(PropertyCharacter::Recaptured)
}

pub(crate) fn component_character_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: &Result1231,
    id: i64,
    amt: bool,
    part: usize,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<PropertyCharacter, SourceError> {
    component_character_cents_with_claims(draft, pool, character, id, amt, part, source_claims).map(
        |value| match value {
            PropertyCharacter::Recaptured(amount) => PropertyCharacter::Recaptured(dollars(amount)),
            value => value,
        },
    )
}

pub(crate) fn component_character_cents_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: &Result1231,
    id: i64,
    amt: bool,
    part: usize,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<PropertyCharacter, SourceError> {
    let whole =
        activity_character_cents_with_claims(draft, pool, character, id, amt, source_claims)?;
    let PropertyCharacter::Recaptured(amount) = whole else {
        return Ok(whole);
    };
    let item =
        pool.k1_section1231.iter().find(|v| v.id == id).expect("resolved K-1 property source");
    // Every dollar of this K-1 is ordinary; losses retain that character too.
    if amount
        == crate::extended_rules::annual_dollars_to_working_cents(item.net)
            .map_err(|error| crate::activity_risk::SourceIssue::new(id, error.into()))?
        && character.in_cents()?.long_term == 0
    {
        return Ok(PropertyCharacter::Ordinary);
    }
    let parts = draft
        .income
        .iter()
        .find(|v| v.id == id)
        .map(|v| k1_property_parts(&v.income, amt))
        .expect("K-1 owner");
    if parts.iter().enumerate().all(|(i, p)| i == part || p.is_none_or(|p| p == (0, 0))) {
        return Ok(whole);
    }
    rate_class_recapture_cents_with_claims(
        draft,
        pool,
        &character.in_cents()?,
        id,
        amt,
        Some(part),
        source_claims,
    )
    .map(PropertyCharacter::Recaptured)
}

// Unique share of an amount drawn from a pool with a known source capacity.
// None means source facts do not determine the share; never assume pro rata.
pub(crate) fn determined_share(amount: i64, total: i64, source: i64) -> Option<i64> {
    if total < 0 || !(0..=total).contains(&amount) || !(0..=total).contains(&source) {
        return None;
    }
    let minimum = phase::floor(amount - (total - source), 0);
    let maximum = phase::cap(amount, [source]);
    (minimum == maximum).then_some(minimum)
}

// Exact Form4797 source cents for the two people on the same return.
// This retains the existing character result, not two separate lookbacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PropertyOwnerAmounts {
    pub ordinary: [i64; 2],
    pub long_term: [i64; 2],
    pub unrecaptured: [i64; 2],
}

// The caller supplies the actual already-calculated direct and allowed rental
// sources. Retained installment/exchange gains reuse the existing native
// source extractor and its seller identity. K-1 adapters remain separate:
// even canceling untraced amounts cannot establish an owner's character.
pub(crate) fn direct_owner_amounts(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: Option<&Result1231>,
    direct: &[(i64, crate::community::PropertyCents)],
    rentals: &[(i64, crate::farm_land::LandGain)],
) -> Option<PropertyOwnerAmounts> {
    if !pool.inputs_complete || !pool.k1_section1231.is_empty() {
        return None;
    }
    let mut ordinary = [0; 2];
    let mut net = [0; 2];
    let mut high = [0; 2];
    let mut present = [false; 2];
    let mut nonnegative_classes = true;
    let annual = rentals
        .iter()
        .copied()
        .chain(earlier_sources(draft).into_iter().map(|(id, value, _)| (id, value)))
        .map(|(id, value)| {
            crate::community::PropertyCents::annual(value).ok().map(|value| (id, value))
        })
        .collect::<Option<Vec<_>>>()?;
    for (id, value) in direct.iter().copied().chain(annual) {
        let owner = draft
            .income
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.owner.owned())
            .or_else(|| {
                draft
                    .adjustments
                    .as_ref()?
                    .employee_expenses
                    .iter()
                    .enumerate()
                    .find(|(position, _)| crate::asset_sale::employee_source_id(*position) == id)
                    .map(|(_, claim)| claim.owner)
            })
            .or_else(|| crate::installment_return::earlier_source_owner(draft, id))
            .or_else(|| crate::exchange::related_source_owner(draft, id))?;
        let index = usize::from(owner == crate::Owner::Spouse);
        ordinary[index] += value.ordinary;
        net[index] += value.section1231;
        high[index] += value.unrecaptured;
        present[index] |= value.section1231 != 0 || value.unrecaptured != 0;
        nonnegative_classes &=
            value.section1231 >= 0 && (0..=value.section1231).contains(&value.unrecaptured);
    }
    let mut result = PropertyOwnerAmounts { ordinary, long_term: [0; 2], unrecaptured: [0; 2] };
    let Some(character) = character else {
        return (!present.iter().any(|value| *value)).then_some(result);
    };
    let working = character.in_cents().ok()?;
    let character = &working;
    if net.iter().sum::<i64>() != character.net {
        return None;
    }
    if let [true, false] | [false, true] = present {
        let index = usize::from(present[1]);
        result.ordinary[index] += character.ordinary;
        result.long_term[index] = character.long_term;
        result.unrecaptured[index] = character.unrecaptured;
    } else if character.net <= 0 {
        for (amount, source) in result.ordinary.iter_mut().zip(net) {
            *amount += source;
        }
    } else if character.ordinary == 0 {
        result.long_term = net;
        for (index, amount) in result.unrecaptured.iter_mut().enumerate() {
            *amount = determined_share(character.unrecaptured, high.iter().sum(), high[index])?;
        }
    } else {
        if !nonnegative_classes {
            return None;
        }
        let total_classes = [high.iter().sum(), net.iter().sum::<i64>() - high.iter().sum::<i64>()];
        let high_recapture = phase::cap(character.ordinary, [total_classes[0]]);
        for index in 0..2 {
            let own = [high[index], net[index] - high[index]];
            let recaptured = class_recapture_amount(character.ordinary, total_classes, own)?;
            result.ordinary[index] += recaptured;
            result.long_term[index] = net[index] - recaptured;
            result.unrecaptured[index] =
                high[index] - determined_share(high_recapture, total_classes[0], high[index])?;
        }
    }
    (result.ordinary.iter().sum::<i64>() == ordinary.iter().sum::<i64>() + character.ordinary
        && result.long_term.iter().sum::<i64>() == character.long_term
        && result.unrecaptured.iter().sum::<i64>() == character.unrecaptured)
        .then_some(result)
}

// Notice97-59's native rate-class owner. Both business attribution and the
// same-return person projection use this one determined-capacity rule.
fn class_recapture_amount(amount: i64, totals: [i64; 2], own: [i64; 2]) -> Option<i64> {
    let high = phase::cap(amount, [totals[0]]);
    Some(
        determined_share(high, totals[0], own[0])?
            + determined_share(amount - high, totals[1], own[1])?,
    )
}

// Notice 97-59: recapture uses the 25% class before general long-term gain.
// Current supported K-1 property sources contain no 28% property class.
// Infer an allocation only when the source's minimum and maximum agree;
// allocating pro rata within a shared class is not an established default.

fn rate_class_recapture_cents_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: &CentResult1231,
    id: i64,
    amt: bool,
    component: Option<usize>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, SourceError> {
    let unresolved = (
        "ISSUE_SECTION1231_SECTION_1231_LOOKBACK_RECAPTURE_NEEDS_BUSINESS_LEVEL_F77D46EA",
        crate::fixture_english!(
            "Section 1231 lookback recapture needs business-level gain allocation before source character can be determined."
        ),
    );
    let lift = crate::extended_rules::annual_dollars_to_working_cents;
    let mut totals = [0i64; 2];
    let mut own = None;
    for item in &pool.k1_section1231 {
        let income = draft
            .income
            .iter()
            .find(|entry| entry.id == item.id)
            .map(|entry| &entry.income)
            .ok_or(unresolved)?;
        let (raw_net, high) = k1_property_totals(income, amt).ok_or(unresolved)?;
        // Suspended-loss offsets and within-source rate-class netting need
        // their own traced allocation; raw box 8c cannot resolve those cases.
        if item.net < 0 || item.net != raw_net || high > item.net {
            return Err(unresolved.into());
        }
        let classes =
            [lift(high).map_err(|_| unresolved)?, lift(item.net - high).map_err(|_| unresolved)?];
        for (total, amount) in totals.iter_mut().zip(classes) {
            *total = total.checked_add(amount).ok_or(unresolved)?;
        }
        if item.id == id {
            own = Some(if let Some(component) = component {
                let (net, high) = k1_property_parts(income, amt)[component].ok_or(unresolved)?;
                let resolved = if component == 0 {
                    item.business().net
                } else {
                    item.rentals
                        .iter()
                        .find(|r| r.real_estate == (component == 1))
                        .ok_or(unresolved)?
                        .net
                };
                if net < 0 || resolved != net || high > net {
                    return Err(unresolved.into());
                }
                [lift(high).map_err(|_| unresolved)?, lift(net - high).map_err(|_| unresolved)?]
            } else {
                classes
            });
        }
    }
    let direct = property_working_sources(draft, pool, amt, source_claims)?;
    for (source, value) in direct {
        if value.section1231 < 0 || value.unrecaptured > value.section1231 {
            return Err(unresolved.into());
        }
        let classes = [value.unrecaptured, value.section1231 - value.unrecaptured];
        for (total, amount) in totals.iter_mut().zip(classes) {
            *total = total.checked_add(amount).ok_or(unresolved)?;
        }
        if source == id {
            own = Some(classes);
        }
    }
    if totals.iter().sum::<i64>() != character.net {
        return Err(unresolved.into());
    }
    let own = own.ok_or(unresolved)?;
    class_recapture_amount(character.ordinary, totals, own).ok_or_else(|| unresolved.into())
}

#[cfg(test)]
pub(crate) fn property_qbi(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
) -> Result<Vec<PropertyQbiItem>, SourceError> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    property_qbi_with_claims(draft, pool, &source_claims)
}

pub(crate) fn property_qbi_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Vec<PropertyQbiItem>, SourceError> {
    let allocated_entries = crate::community::entries(draft);
    if !pool.k1_section1231.iter().any(|item| {
        allocated_entries.iter().any(|entry| entry.id == item.id && matches!(
            &entry.income, FederalIncome::Scorp { data } if crate::ScorpK1::from(data).property_tracking() && crate::ScorpK1::from(data).qbi_left_out().is_none()
        ))
    }) {
        return Ok(Vec::new());
    }
    let Some(character) = worksheet_with_claims(draft, pool, false, source_claims)? else {
        return Ok(Vec::new());
    };
    // Section 1231(a)(2) includes equality: zero net does not mean every
    // business's qualified gain/loss vanishes.
    let mut items = Vec::new();
    for item in pool.k1_section1231.iter().map(crate::extended_rules::K1PropertyResult::business) {
        let source = allocated_entries.iter().find_map(|entry| {
            if entry.id == item.id
                && let FederalIncome::Scorp { data } = &entry.income
            {
                return Some(crate::ScorpK1::from(data));
            }
            None
        });
        let Some(source) = source else { continue };
        if source
            .business_property()
            .map_err(|detail| SourceError::income(item.id, detail))?
            .is_none()
        {
            continue;
        }
        // A missing current statement omits current reported QBI. Actual
        // previously qualified suspended losses keep their loss-year status.
        if !source.property_tracking() || source.qbi_left_out().is_some() {
            continue;
        }
        let source = source.qbi_projection();
        let treatment = component_character_with_claims(
            draft,
            pool,
            &character,
            item.id,
            false,
            0,
            source_claims,
        )
        .map_err(SourceError::with_manual)?;
        if treatment == PropertyCharacter::Capital {
            items.push(PropertyQbiItem { id: item.id, gain: 0, loss: 0, current_loss: 0 });
            continue;
        }
        let qbi = source
            .property_qbi
            .calculation(&source, None)
            .map_err(|detail| SourceError::income(item.id, detail))?;
        // after_passive validates source completeness, original-year eligibility,
        // and the character-specific suspension rather than a combined loss.
        let entry_owner = allocated_entries
            .iter()
            .find(|entry| entry.id == item.id)
            .ok_or(PARTNER_QBI_SPLIT)?
            .owner;
        let packet =
            pool.post_pal.as_ref().filter(|packet| !packet.alternative).ok_or(PARTNER_QBI_SPLIT)?;
        let slot = packet
            .sources
            .iter()
            .find(|value| {
                value.income == item.id
                    && value.owner == entry_owner.owned()
                    && value.component == crate::extended_rules::PostPalComponent::ScorpBusiness
                    && value.common_pool
            })
            .ok_or(PARTNER_QBI_SPLIT)?;
        let held = slot
            .characters
            .iter()
            .find(|value| value.character == crate::extended_rules::PostPalCharacter::Section1231)
            .ok_or(PARTNER_QBI_SPLIT)?;
        let suspended =
            crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(held.suspended_cents)
                .map_err(|_| PARTNER_QBI_SPLIT)?;
        let released = qbi
            .after_passive(&source, suspended)
            .map_err(|detail| SourceError::income(item.id, detail))?;
        if let PropertyCharacter::Recaptured(amount) = treatment {
            items.push(PropertyQbiItem {
                id: item.id,
                gain: qbi
                    .qualified_recapture(&source, amount)
                    .map_err(|detail| SourceError::income(item.id, detail).with_manual())?,
                loss: 0,
                current_loss: 0,
            });
            continue;
        }
        items.push(PropertyQbiItem {
            id: item.id,
            gain: dollars(qbi.gain_cents),
            loss: dollars(released.qbi.get()),
            current_loss: qbi
                .current_allowed(&source, suspended)
                .map(dollars)
                .map_err(|detail| SourceError::income(item.id, detail))?,
        });
    }
    Ok(items)
}

// The approved partner listing convention qualifies released property carries
// when a current section 199A statement is attached. Keep this linear class
// allocation separate from ordinary-loss and S-corporation vintage records.
#[derive(Default)]
pub(crate) struct PartnerPropertyQbi {
    pub gain_cents: i64,
    pub current_loss_cents: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub historical_loss_cents: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub carries: [(
        crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
        crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    ); 3],
}

impl PartnerPropertyQbi {
    pub(crate) fn report(&self, id: i64, result: &mut FederalResult) {
        for (key, (loss, qualified)) in [
            "NOTE_PARTNER_PROPERTY_QBI_BASIS_CARRY",
            "NOTE_PARTNER_PROPERTY_QBI_RISK_CARRY",
            "NOTE_PARTNER_PROPERTY_QBI_PASSIVE_CARRY",
        ]
        .into_iter()
        .zip(self.carries)
        {
            if loss.get() > 0 {
                crate::federal_rules::note_once_scoped(
                    result,
                    Message::new(
                        key,
                        alloc::vec![
                            MessageArg::Number(id),
                            MessageArg::usd_cents(loss.get()),
                            MessageArg::usd_cents(qualified.get()),
                        ],
                    ),
                    crate::IssueContext::income_id(id),
                );
            }
        }
    }
}

const PARTNER_QBI_SPLIT: (&str, &str) = (
    "ISSUE_K1_SECTION1231_QBI_SPLIT",
    crate::fixture_english!(
        "The section 199A statement and loss records do not determine the qualified share of the section 1231 lookback recapture. Provide a business-level qualified allocation."
    ),
);

struct PartnerPropertyLoss {
    current: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    historical: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    suspended: (
        crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
        crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    ),
}

// Keep the existing two-bucket qualified/nonqualified rounding, then trace
// the current-source share within the allowed qualified amount. This keeps
// the approved linear class result while excluding old qualification from
// current SSTB eligibility. Returns current, historical and suspended QBI.
fn partner_property_loss(
    allowed: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    total: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    qualified: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    current: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
) -> Result<PartnerPropertyLoss, (&'static str, &'static str)> {
    let (allowed, total, qualified, current) =
        (allowed.get(), total.get(), qualified.get(), current.get());
    if !(0..=total).contains(&allowed)
        || !(0..=total).contains(&qualified)
        || !(0..=qualified).contains(&current)
    {
        return Err(PARTNER_QBI_SPLIT);
    }
    let qbi = crate::kernel::ratio::allocate_remaining(allowed, &[qualified, total - qualified])[0];
    let current = crate::kernel::ratio::allocate_remaining(qbi, &[current, qualified - current])[0];
    Ok(PartnerPropertyLoss {
        current: crate::kernel::carry::Amount::new(current)
            .expect("nonnegative current source share"),
        historical: crate::kernel::carry::Amount::new(qbi - current)
            .expect("nonnegative historical source share"),
        suspended: (
            crate::kernel::carry::closing_values::<crate::kernel::carry::Cents>(total, allowed)
                .map_err(|_| PARTNER_QBI_SPLIT)?,
            crate::kernel::carry::closing_values::<crate::kernel::carry::Cents>(qualified, qbi)
                .map_err(|_| PARTNER_QBI_SPLIT)?,
        ),
    })
}

#[cfg(test)]
pub(crate) fn partner_property_qbi(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    id: i64,
) -> Result<i64, SourceError> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    partner_property_qbi_with_claims(draft, pool, id, &source_claims)
}

#[cfg(test)]
pub(crate) fn partner_property_qbi_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    id: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, SourceError> {
    let value = partner_property_qbi_items_with_claims(draft, pool, id, None, source_claims)?;
    Ok(dollars(
        value.gain_cents - value.current_loss_cents.get() - value.historical_loss_cents.get(),
    ))
}

#[cfg(test)]
pub(crate) fn partner_property_qbi_items(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    id: i64,
    eligibility: Option<(i64, crate::FederalStatus)>,
) -> Result<PartnerPropertyQbi, SourceError> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    partner_property_qbi_items_with_claims(draft, pool, id, eligibility, &source_claims)
}

pub(crate) fn partner_property_qbi_items_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    id: i64,
    eligibility: Option<(i64, crate::FederalStatus)>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<PartnerPropertyQbi, SourceError> {
    let allocated_entries = crate::community::entries(draft);
    let Some(owner) = allocated_entries.iter().find_map(|entry| match &entry.income {
        FederalIncome::Partnership { data } if entry.id == id => Some(data),
        _ => None,
    }) else {
        return Ok(PartnerPropertyQbi::default());
    };
    let Some(statement) = &owner.qbi else {
        return Ok(PartnerPropertyQbi::default());
    };
    if !statement.valid(owner) {
        return Err(PARTNER_QBI_SPLIT.into());
    }
    let Some(property) = owner.regular.business_property(&owner.items())? else {
        return Ok(PartnerPropertyQbi::default());
    };
    if !pool.inputs_complete {
        return Err(PARTNER_QBI_SPLIT.into());
    }
    let entry_owner =
        allocated_entries.iter().find(|entry| entry.id == id).ok_or(PARTNER_QBI_SPLIT)?.owner;
    let packet =
        pool.post_pal.as_ref().filter(|packet| !packet.alternative).ok_or(PARTNER_QBI_SPLIT)?;
    let slot = packet
        .sources
        .iter()
        .find(|value| {
            value.income == id
                && value.owner == entry_owner.owned()
                && value.component == crate::extended_rules::PostPalComponent::PartnershipBusiness
                && value.common_pool
        })
        .ok_or(PARTNER_QBI_SPLIT)?;
    let held = slot
        .characters
        .iter()
        .find(|value| value.character == crate::extended_rules::PostPalCharacter::Section1231)
        .ok_or(PARTNER_QBI_SPLIT)?;
    let suspended =
        crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(held.suspended_cents)
            .map_err(|_| PARTNER_QBI_SPLIT)?;
    let item = pool
        .k1_section1231
        .iter()
        .find(|item| item.id == id)
        .map(crate::extended_rules::K1PropertyResult::business)
        .ok_or(PARTNER_QBI_SPLIT)?;
    let character =
        worksheet_with_claims(draft, pool, false, source_claims)?.ok_or(PARTNER_QBI_SPLIT)?;
    let treatment =
        component_character_with_claims(draft, pool, &character, id, false, 0, source_claims)?;
    if treatment == PropertyCharacter::Capital {
        return Ok(PartnerPropertyQbi::default());
    }
    let mut gain = phase::floor(statement.section1231_qbi, 0);
    let mut qualified = phase::floor(-statement.section1231_qbi, 0);
    for sale in owner.regular.sale_results(&owner.items())? {
        gain += phase::floor(sale.section1231, 0) * 100;
        qualified += phase::floor(-sale.section1231, 0) * 100;
    }
    // A uniquely determined lookback share is current ordinary gain. Resolve
    // it from the unphased source listing before one business eligibility pass.
    if let PropertyCharacter::Recaptured(amount) = treatment {
        let source = partner_property_qbi_losses(
            owner,
            &property,
            suspended,
            crate::kernel::carry::Amount::new(qualified).map_err(|_| PARTNER_QBI_SPLIT)?,
            crate::community::partner_ledger(draft, id, owner, &owner.regular).ok().as_ref(),
        )?;
        let net =
            dollars(gain - source.current_loss_cents.get() - source.historical_loss_cents.get());
        let gain = determined_share(amount, item.net, net).ok_or(PARTNER_QBI_SPLIT)?;
        return Ok(PartnerPropertyQbi { gain_cents: gain * 100, ..Default::default() });
    }
    if let Some((taxable, status)) = eligibility {
        let (applicable, range) = crate::business_rules::qbi_eligibility_factor(
            -qualified,
            Some(statement.service),
            false,
            taxable,
            status,
        )?;
        qualified = crate::kernel::ratio::fraction(qualified, applicable, range);
    }
    let mut value = partner_property_qbi_losses(
        owner,
        &property,
        suspended,
        crate::kernel::carry::Amount::new(qualified).map_err(|_| PARTNER_QBI_SPLIT)?,
        crate::community::partner_ledger(draft, id, owner, &owner.regular).ok().as_ref(),
    )?;
    value.gain_cents = gain;
    Ok(value)
}

fn partner_property_qbi_losses(
    owner: &crate::PartnerK1,
    property: &crate::k1_items::BusinessProperty,
    suspended: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    qualified: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    resolved_ledger: Option<&crate::k1_loss::LossLedger>,
) -> Result<PartnerPropertyQbi, (&'static str, &'static str)> {
    use crate::k1_loss::K1LossClass::Section1231;
    let amount = |value| {
        crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(value)
            .map_err(|_| PARTNER_QBI_SPLIT)
    };
    let default_ledger;
    let ledger = if let Some(ledger) = resolved_ledger {
        ledger
    } else {
        default_ledger = owner.ledger(&owner.regular)?;
        &default_ledger
    };
    let rental_losses = owner.regular.rentals.property_allowed(ledger)?.iter().sum::<i64>();
    let basis_allowed = ledger.basis_allowed[Section1231].get() - rental_losses;
    let risk_allowed = ledger.risk_allowed[Section1231].get() - rental_losses;
    let basis = partner_property_loss(
        amount(basis_allowed)?,
        amount(property.loss + property.basis_loss)?,
        amount(qualified.get() + property.basis_loss)?,
        qualified,
    )?;
    let risk = partner_property_loss(
        amount(risk_allowed)?,
        amount(basis_allowed + property.risk_loss)?,
        amount(basis.current.get() + basis.historical.get() + property.risk_loss)?,
        basis.current,
    )?;
    let passive = if owner.material_for_losses() == crate::participation::Participation::Yes {
        if !(0..=property.passive_loss).contains(&suspended.get()) {
            return Err(PARTNER_QBI_SPLIT);
        }
        PartnerPropertyLoss {
            current: risk.current,
            historical: amount(risk.historical.get() + property.passive_loss - suspended.get())?,
            suspended: (suspended, suspended),
        }
    } else {
        partner_property_loss(
            amount(risk_allowed + property.passive_loss - suspended.get())?,
            amount(risk_allowed + property.passive_loss)?,
            amount(risk.current.get() + risk.historical.get() + property.passive_loss)?,
            risk.current,
        )?
    };
    Ok(PartnerPropertyQbi {
        gain_cents: 0,
        current_loss_cents: passive.current,
        historical_loss_cents: passive.historical,
        carries: [basis.suspended, risk.suspended, passive.suspended],
    })
}

#[cfg(test)]
pub(crate) fn report(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
    result: &mut FederalResult,
) -> Option<Result1231> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    report_with_claims(draft, pool, amt, result, &source_claims)
}

pub(crate) fn report_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
    result: &mut FederalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<Result1231> {
    #[cfg(test)]
    let label = if amt { "AMT" } else { "regular" };
    // Lines 1b and 1c: the gain and the loss of the partial dispositions of
    // MACRS assets that lines 2, 11 and 26 include.
    if !amt {
        let [gain, loss] =
            match crate::asset_sale::partial_totals_with_claims(draft, pool, source_claims) {
                Ok(value) => value,
                Err(issue) => {
                    issue.report(result);
                    return None;
                }
            };
        for (line, id, amount, explanation) in [
            ("1b", "form4797_line1b", gain, msg!("EXPLAIN_N_FORM4797_LINE1B")),
            ("1c", "form4797_line1c", loss, msg!("EXPLAIN_N_FORM4797_LINE1C")),
        ] {
            if amount != 0 {
                crate::line_emission::push_line(
                    result,
                    Some(crate::FormLine::new(crate::TaxForm::Form4797, line)),
                    id,
                    amount,
                    &["income.business.assets", "income.rental"],
                    "https://www.irs.gov/instructions/i4797",
                    explanation,
                );
            }
        }
    }
    let computed = match worksheet_with_claims(draft, pool, amt, source_claims) {
        Ok(Some(value)) => value,
        Ok(None) => return None,
        Err(error) => {
            error.report_at(
                result,
                &crate::IssueContext::Section1231,
                #[cfg(test)]
                if amt { "AMT section 1231 lookback" } else { "Section 1231 lookback" },
            );
            return None;
        }
    };
    if !amt {
        emit_lines(
            result,
            &mut (),
            [
                (
                    "12",
                    "form4797_line12",
                    phase::cap(computed.net, [0]),
                    msg!("EXPLAIN_FORM4797_LINE12"),
                ),
                (
                    "13",
                    "form4797_line13",
                    phase::floor(computed.ordinary, 0),
                    msg!("EXPLAIN_FORM4797_LINE13"),
                ),
                ("8", "form4797_line8", computed.net, msg!("EXPLAIN_N_27")),
                (
                    "9",
                    "form4797_line9",
                    if computed.net > 0 {
                        // The whole nonrecaptured balance of the five preceding
                        // years, even where it exceeds the line 8 gain (i4797
                        // line 8 and its example).
                        computed.lookback
                    } else {
                        0
                    },
                    msg!("EXPLAIN_N_28"),
                ),
                ("10", "form4797_line10", computed.long_term, msg!("EXPLAIN_N_29")),
            ],
            |(line, id, amount, explanation), _, _| {
                Some(calculation_row(
                    Some(crate::FormLine::new(crate::TaxForm::Form4797, line)),
                    id,
                    LineValue::MoneyCents(amount * 100),
                    &[
                        "income.business.assets.disposal",
                        "income.rental.sale",
                        "screening.section1231",
                    ],
                    "https://www.irs.gov/instructions/i4797",
                    explanation,
                ))
            },
        );
    }
    report_k1_sources(draft, pool, amt, result);
    result.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
        "Section 1231 {label} {} worksheet: net after activity limits ${}; prior five-year nonrecaptured losses ${}; ordinary amount ${}; long-term amount ${}; business-asset section 1250 component ${}. Intermediate Form 4797/character worksheet, not final return totals. Later QBI, investment-tax and return-level loss rules remain separate.",
        computed.year, computed.net, computed.lookback, computed.ordinary, computed.long_term, computed.unrecaptured,
    ), Message::new(if amt { "NOTE_1231_AMT_WORKSHEET" } else { "NOTE_1231_REGULAR_WORKSHEET" }, alloc::vec![
        MessageArg::Number(computed.year as i64), MessageArg::usd_dollars(computed.net),
        MessageArg::usd_dollars(computed.lookback), MessageArg::usd_dollars(computed.ordinary),
        MessageArg::usd_dollars(computed.long_term), MessageArg::usd_dollars(computed.unrecaptured),
    ])));
    for year in &computed.history {
        result.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
            "Section 1231 {label} loss year {}: original deductible loss ${}; opening nonrecaptured balance ${}; recaptured this year ${}; expired ${}; eligible balance for next year ${}. Provisional until all current-year source amounts and return calculations are final.",
            year.year, year.original, year.opening, year.recaptured, year.expired, year.carried,
        ), Message::new(if amt { "NOTE_1231_AMT_LOSS_YEAR" } else { "NOTE_1231_REGULAR_LOSS_YEAR" }, alloc::vec![
            MessageArg::Number(year.year as i64), MessageArg::usd_dollars(year.original),
            MessageArg::usd_dollars(year.opening), MessageArg::usd_dollars(year.recaptured),
            MessageArg::usd_dollars(year.expired), MessageArg::usd_dollars(year.carried),
        ])));
    }
    if computed.net < 0 {
        result.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
            "Section 1231 {label} current net loss ${}: a new lookback balance cannot be finalized until the loss taken into account in taxable income is established. Do not automatically carry the full intermediate loss as a new history entry.", -computed.net,
        ), Message::new(if amt { "NOTE_1231_AMT_CURRENT_LOSS" } else { "NOTE_1231_REGULAR_CURRENT_LOSS" }, alloc::vec![MessageArg::usd_dollars(-computed.net)])));
    } else {
        let carried = computed
            .next_history(0)
            .expect("no current net loss to add")
            .iter()
            .map(|v| v.remaining)
            .sum::<i64>();
        result.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
            "Section 1231 {label} prior-loss balances remaining eligible for {}: ${carried}. Retain the loss-year detail after final review; this is not a deduction carried forward.", computed.year + 1,
        ), Message::new(if amt { "NOTE_1231_AMT_REMAINING" } else { "NOTE_1231_REGULAR_REMAINING" }, alloc::vec![
            MessageArg::Number((computed.year + 1) as i64), MessageArg::usd_dollars(carried),
        ])));
    }
    if !amt {
        match property_qbi_with_claims(draft, pool, source_claims) {
            Ok(items) => {
                for item in items {
                    result.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
                        "S-corporation document {}, section 1231 potential QBI after capital exclusion: gain ${}; loss ${}. Current-source qualified loss ${}; released historical qualified loss ${}. Current-year SSTB eligibility and return-level limits remain unresolved; historical loss eligibility is retained. These are intermediate business items, not a final QBI deduction.",
                        item.id, item.gain, item.loss, item.current_loss, item.loss - item.current_loss,
                    ), Message::new("NOTE_1231_SCORP_POTENTIAL_QBI", alloc::vec![
                        MessageArg::Number(item.id), MessageArg::usd_dollars(item.gain),
                        MessageArg::usd_dollars(item.loss), MessageArg::usd_dollars(item.current_loss),
                        MessageArg::usd_dollars(item.loss - item.current_loss),
                    ])));
                }
            }
            Err(error) => {
                let (key, _message) = error.detail;
                #[cfg(test)]
                let message = _message;
                crate::issue_localized!(
                    result,
                    &error.context,
                    "Section 1231 QBI character",
                    key,
                    message,
                )
            }
        }
    }
    Some(computed)
}

// Add the rental property's ordinary character to its existing rental QBI
// activity, preserving that activity's single set of wages and UBIA.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RentalPropertyQbi {
    pub current: Vec<(usize, i64, Option<i64>)>,
    pub historical: Vec<(i64, i64)>,
}

#[cfg(test)]
pub(crate) fn rental_property_qbi(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
) -> Result<RentalPropertyQbi, SourceError> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    rental_property_qbi_with_claims(draft, pool, &source_claims)
}

pub(crate) fn rental_property_qbi_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<RentalPropertyQbi, SourceError> {
    for (&id, prior) in &pool.operating_prior_allowed {
        if *prior != Some(0)
            && draft.income.iter().any(|entry| {
                entry.id == id
                    && match &entry.income {
                        FederalIncome::Rental { data } => {
                            matches!(data.qbi_business, crate::RentalBusiness::Yes { .. })
                        }
                        FederalIncome::FarmRental { data } => {
                            matches!(data.qbi_business, crate::RentalBusiness::Yes { .. })
                        }
                        _ => false,
                    }
            })
        {
            return Err(SourceError::required(id, (
                "ACTIVITY_QBI_HISTORY_UNAVAILABLE",
                crate::fixture_english!(
                    "Released prior at-risk losses need their original-year qualified business income status before completing the QBI deduction."
                ),
            )).with_manual());
        }
    }
    let farm_rentals: Vec<_> = pool
        .qbi_source_ids
        .iter()
        .enumerate()
        .filter(|(_, id)| {
            draft.income.iter().any(|entry| {
                entry.id == **id && matches!(entry.income, FederalIncome::FarmRental { .. })
            })
        })
        .collect();
    let k1_rentals =
        pool.k1_section1231.iter().any(|v| v.rentals.iter().any(|r| r.qbi_index.is_some()));
    let has_direct_qbi_source = draft.income.iter().any(|entry| {
        matches!(
            &entry.income, FederalIncome::Rental { data }
                if matches!(data.qbi_business, crate::RentalBusiness::Yes { .. })
                    || data.at_risk.as_ref().is_some_and(|risk| risk.prior_sale(false) > 0)
        )
    });
    let direct_rentals = if has_direct_qbi_source {
        crate::asset_sale::allowed_rental_sources(
            crate::related_forms::RelatedFormContext::original(draft, source_claims),
            pool,
            false,
        )?
    } else {
        Vec::new()
    };
    if !k1_rentals && farm_rentals.is_empty() && direct_rentals.is_empty() {
        return Ok(RentalPropertyQbi { current: Vec::new(), historical: Vec::new() });
    }
    let character = worksheet_with_claims(draft, pool, false, source_claims)?;
    let mut amounts = Vec::new();
    let mut historical = Vec::new();
    // i8995 / Reg1.199A-3(b)(1)(iv): historical qualification is independent
    // of today's rental status. Previously suspended losses use a separate
    // business with no wages/UBIA; capital character remains excluded.
    for (id, value) in direct_rentals {
        let entries = crate::community::property_entries(draft);
        let Some(source) = entries.iter().find_map(|entry| match &entry.income {
            FederalIncome::Rental { data } if entry.id == id => Some(data),
            _ => None,
        }) else {
            continue;
        };
        let qualified = matches!(source.qbi_business, crate::RentalBusiness::Yes { .. });
        let prior = source.at_risk.as_ref().map_or(0, |risk| risk.prior_sale(false));
        if !qualified && prior == 0 {
            continue;
        }
        let unavailable = || {
            SourceError::required(id, (
            "ACTIVITY_QBI_HISTORY_UNAVAILABLE",
            crate::fixture_english!("The rental's released property-sale losses need their original-year QBI worksheets and allowed at-risk/passive release."),
        )).with_manual()
        };
        let (prior_native, risk_release, final_release) =
            crate::asset_sale::rental_prior_property_release(
                crate::related_forms::RelatedFormContext::original(draft, source_claims),
                pool,
                id,
                source,
            )?;
        let qualified_release =
            crate::rental_property_history::qualified_release(source, risk_release, final_release)
                .map_err(|_| unavailable())?;
        // Match the source owner's prior native row: nonqualified source cents
        // round first; qualified history receives its actual native residual.
        let [_, qualified_prior] = crate::kernel::ratio::project_native_pair(
            prior_native,
            [final_release - qualified_release, qualified_release],
            0,
        )
        .filter(|parts| parts.iter().all(|part| *part >= 0))
        .ok_or_else(unavailable)?;
        let current_section = value.section1231 + prior_native;
        let treatment = if value.section1231 == 0 && prior == 0 {
            PropertyCharacter::Ordinary
        } else {
            let character = character.as_ref().ok_or_else(|| {
                SourceError::required(id, (
                    "RENTAL_PROPERTY_CHARACTER_UNAVAILABLE",
                    crate::fixture_english!("Resolve the sold rental's section 1231 character before completing its qualified business income."),
                )).with_manual()
            })?;
            activity_character_with_claims(draft, pool, character, id, false, source_claims)
                .map_err(SourceError::with_manual)?
        };
        let (current, history) = match treatment {
            PropertyCharacter::Capital => (0, 0),
            PropertyCharacter::Ordinary => {
                (if qualified { current_section } else { 0 }, qualified_prior)
            }
            PropertyCharacter::Recaptured(amount) => {
                // Full ordinary treatment keeps each current gain and original
                // qualified loss. Partial lookback needs a uniquely determined
                // qualified share, exactly as the existing K-1 owner requires.
                if amount == value.section1231 {
                    (if qualified { current_section } else { 0 }, qualified_prior)
                } else {
                    let net = if qualified { current_section } else { 0 } - qualified_prior;
                    (determined_share(amount, value.section1231, net).ok_or_else(unavailable)?, 0)
                }
            }
        };
        if history > 0 {
            historical.push((id, history));
        }
        let amount = if qualified { value.ordinary } else { 0 } + current;
        if amount == 0 {
            continue;
        }
        let mut indices =
            pool.qbi_source_ids.iter().enumerate().filter(|(_, source)| **source == id);
        let index = indices.next().map(|(index, _)| index);
        if index.is_none() || indices.next().is_some() {
            return Err(SourceError::required(id, (
                "RENTAL_PROPERTY_QBI_ACTIVITY_UNAVAILABLE",
                crate::fixture_english!("Resolve the sold rental's single qualified activity before adding its ordinary property income."),
            )).with_manual());
        }
        amounts.push((index.expect("one resolved rental activity"), amount, None));
    }
    if let Some(character) = &character {
        for item in &pool.k1_section1231 {
            for rental in &item.rentals {
                let Some(index) = rental.qbi_index else {
                    continue;
                };
                let treatment = component_character_with_claims(
                    draft,
                    pool,
                    character,
                    item.id,
                    false,
                    if rental.real_estate { 1 } else { 2 },
                    source_claims,
                )
                .map_err(SourceError::with_manual)?;
                amounts.push((
                    index,
                    match treatment {
                        PropertyCharacter::Ordinary => rental.net,
                        PropertyCharacter::Capital => 0,
                        PropertyCharacter::Recaptured(amount) => amount,
                    },
                    Some(match treatment {
                        PropertyCharacter::Ordinary => rental.net_cents,
                        PropertyCharacter::Capital => 0,
                        // The resolved partial Form4797 ordinary amount is a report boundary.
                        PropertyCharacter::Recaptured(amount) => amount.checked_mul(100).ok_or_else(|| SourceError::required(item.id, (
                            "RENTAL_PROPERTY_QBI_ACTIVITY_UNAVAILABLE",
                            crate::fixture_english!("Resolve the sold rental's single qualified activity before adding its ordinary property income."),
                        )).with_manual())?,
                    }),
                ));
            }
        }
    }
    // Form4835 keeps one qualified activity and one wage/UBIA set. The
    // existing direct-property owner supplies its allowed ordinary character;
    // neither capital gain nor an unresolved lookback becomes operating QBI.
    for (index, &id) in farm_rentals {
        if pool
            .farm_rental_property_rows
            .get(&id)
            .is_some_and(|rows| rows.prior_section1231_allowed > 0)
        {
            return Err(SourceError::required(id, (
                "ACTIVITY_QBI_HISTORY_UNAVAILABLE",
                crate::fixture_english!(
                    "Released prior at-risk losses need their original-year qualified business income status before completing the QBI deduction."
                ),
            )).with_manual());
        }
        let cents = crate::farm_land::qbi_cents_with_claims(
            draft,
            pool,
            character.as_ref(),
            id,
            source_claims,
        )
        .map_err(SourceError::with_manual)?;
        amounts.push((index, dollars(cents), Some(cents)));
    }
    Ok(RentalPropertyQbi { current: amounts, historical })
}

// Form 4797 line 2: the residual original/historical/rental K-1 components.
// Current section 179 sale gains and allowed losses have per-property rows.
fn report_k1_sources(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
    result: &mut FederalResult,
) {
    for entry in &crate::community::property_entries(draft) {
        if k1_property_totals(&entry.income, amt).is_some()
            && let Some(value) = pool.k1_section1231.iter().find(|v| v.id == entry.id)
        {
            let partner = matches!(entry.income, FederalIncome::Partnership { .. });
            let kind = if partner { "partnership" } else { "scorp" };
            let prefix = if amt { "amt_" } else { "" };
            crate::line_emission::push_line(
                result,
                (!amt).then(|| crate::FormLine::new(crate::TaxForm::Form4797, "2")),
                &alloc::format!("{prefix}{kind}_{}_section1231", entry.id),
                dollars(value.net_cents - value.section179_sales_cents.iter().sum::<i64>()) * 100,
                &[if partner { "income.partnership" } else { "income.scorp.section1231" }],
                "https://www.irs.gov/instructions/i4797",
                if partner {
                    msg!("EXPLAIN_PARTNERSHIP_N_SECTION1231")
                } else {
                    msg!("EXPLAIN_SCORP_N_SECTION1231")
                },
            );
        }
    }
}

// The property producer and the operating-earnings owner use the same actual
// participation, while section1411 keeps their tax characters independent.
pub(crate) fn direct_property_is_passive(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
) -> bool {
    form_context.draft.income.iter().any(|entry| {
        entry.id == id
            && match &entry.income {
                FederalIncome::FarmRental { .. } => true,
                FederalIncome::Business { .. } => {
                    form_context.participation().get(id) == crate::participation::Participation::No
                }
                _ => false,
            }
    })
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PropertyNiit {
    // Form 8960 lines 5a/5b: ordinary property amount plus exclusions. The
    // return's already loss-limited capital amount remains in line 5a.
    pub adjustment: i64,
    pub next_capital_adjustment: i64,
    // Evidence from the individual native property components, before any
    // signed netting. Caller-owned NUA/self-rental exclusions are separate.
    pub current_components_known_zero: bool,
}

#[cfg(test)]
pub(crate) fn property_niit(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: &Result1231,
    capital_net: i64,
    capital_income: i64,
    prior_difference: i64,
    other_exclusions: i64,
) -> Result<PropertyNiit, SourceError> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    property_niit_with_claims(
        crate::related_forms::RelatedFormContext::original(draft, &source_claims),
        pool,
        character,
        capital_net,
        capital_income,
        prior_difference,
        other_exclusions,
    )
}

pub(crate) fn property_niit_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    pool: &crate::extended_rules::RentalResult,
    character: &Result1231,
    capital_net: i64,
    capital_income: i64,
    prior_difference: i64,
    other_exclusions: i64,
) -> Result<PropertyNiit, SourceError> {
    let crate::related_forms::RelatedFormContext { draft, claims: source_claims, .. } =
        form_context;
    let annual_character = character;
    let working = character.in_cents()?;
    let character = &working;
    let lift = crate::extended_rules::annual_dollars_to_working_cents;
    let overflow = |_| {
        SourceError::from((
            "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
            "The property working amount exceeds its native range.",
        ))
    };
    let direct =
        crate::asset_sale::allowed_direct_sources_with_claims(draft, pool, false, source_claims)?;
    // A sold rental's gain is investment income (passive or rental).
    let rentals = crate::asset_sale::allowed_rental_sources(
        crate::related_forms::RelatedFormContext::original(draft, source_claims),
        pool,
        false,
    )?
    .into_iter()
    .map(|(id, value)| Ok((id, crate::community::PropertyCents::annual(value).map_err(overflow)?)))
    .collect::<Result<Vec<_>, SourceError>>()?;
    // An earlier-year installment sale of a business you materially
    // participated in is excluded; a rental's is investment income.
    let earlier = earlier_sources(draft)
        .into_iter()
        .map(|(id, value, business)| {
            Ok((id, crate::community::PropertyCents::annual(value).map_err(overflow)?, business))
        })
        .collect::<Result<Vec<_>, SourceError>>()?;
    if !pool.inputs_complete
        || pool
            .k1_section1231
            .iter()
            .map(|value| lift(value.net).map_err(overflow))
            .collect::<Result<Vec<_>, SourceError>>()?
            .iter()
            .sum::<i64>()
            + direct.iter().map(|(_, v)| v.section1231).sum::<i64>()
            + rentals.iter().map(|(_, v)| v.section1231).sum::<i64>()
            + earlier.iter().map(|(_, v, _)| v.section1231).sum::<i64>()
            != character.net
    {
        return Err((
            "ISSUE_SECTION1231_RESOLVE_AND_RECONCILE_THE_PROPERTY_ACTIVITY_FFC5EB09",
            crate::fixture_english!(
                "Resolve and reconcile the property activity pool before investment-tax classification."
            ),
        ).into());
    }
    let mut excluded = 0;
    // The Form 4797 ordinary amounts of passive rental sources (a sold rental's section 1250
    // additional depreciation, a farm rental's recapture): net investment income like the
    // rental's other property gain, so Form 8960 line 5b removes none of it.
    let home_casualty = crate::kernel::ratio::checked_sum(pool.prior_home_casualty_by_owner)
        .ok_or((
            "RENTAL_HOME_PRIOR_CARRY_RECORDS",
            "The prior rental casualty amount is unavailable.",
        ))?;
    let mut passive_ordinary = crate::kernel::ratio::checked_sum(
        rentals.iter().map(|(_, v)| v.ordinary).chain(core::iter::once(home_casualty)),
    )
    .ok_or((
        "RENTAL_HOME_PRIOR_CARRY_RECORDS",
        "The prior rental casualty amount is unavailable.",
    ))?;
    let mut passive_ordinary_components_zero = rentals.iter().all(|(_, v)| v.ordinary == 0)
        && pool.prior_home_casualty_by_owner.iter().all(|amount| *amount == 0);
    let mut section1231_components_zero = true;
    let mut excluded_components_zero = true;
    let mut excluded_sources = Vec::new();
    let mut all_excluded = true;
    let mut all_included = true;
    for value in &pool.k1_section1231 {
        let (trade, all_nonpassive_rows) = draft
            .income
            .iter()
            .find_map(|entry| {
                if entry.id != value.id {
                    return None;
                }
                match &entry.income {
                    FederalIncome::Scorp { data } => {
                        let data = &crate::ScorpK1::from(data);
                        Some((
                            data.trade_kind,
                            data.nonpassive() == crate::participation::Participation::Yes
                                && data.passive.loss == 0
                                && data
                                    .section1231
                                    .as_ref()
                                    .is_none_or(|property| property.passive_loss == 0)
                                && value.rentals.is_empty(),
                        ))
                    }
                    FederalIncome::Partnership { data } => Some((
                        data.trade_kind,
                        data.material_for_losses() == crate::participation::Participation::Yes
                            && data.regular.passive_loss == 0
                            && data
                                .regular
                                .section1231
                                .as_ref()
                                .is_none_or(|property| property.passive_loss == 0)
                            && value.rentals.is_empty(),
                    )),
                    _ => None,
                }
            })
            .ok_or((
                "ISSUE_SECTION1231_A_PROPERTY_INVESTMENT_TAX_RESULT_IS_MISSING_A060069C",
                crate::fixture_english!(
                    "A property investment-tax result is missing its source business."
                ),
            ))?;
        let business = value.business();
        section1231_components_zero &= business.net == 0;
        let own_excluded = match trade {
            // The actual participation branch and absent passive sources prove
            // every printed row nonpassive, even when the raw rows sum to zero.
            crate::ScorpTradeKind::Ordinary if all_nonpassive_rows => business.net,
            crate::ScorpTradeKind::Ordinary => business.nonpassive,
            crate::ScorpTradeKind::FinancialTrading => 0,
        };
        let own_excluded = lift(own_excluded).map_err(overflow)?;
        let business_net = lift(business.net).map_err(overflow)?;
        excluded_sources.push((value.id, business_net, own_excluded, Some(0)));
        excluded += own_excluded;
        all_excluded &= business_net == own_excluded;
        all_included &= own_excluded == 0;
        excluded_components_zero &= own_excluded == 0;
        for rental in &value.rentals {
            section1231_components_zero &= rental.net == 0;
            // These signed operands can cancel inside one rental, too.
            excluded_components_zero &= !rental.exclude_niit
                || (rental.nonpassive_cents == 0 && rental.niit_disposition_loss == 0);
            let own_excluded = if rental.exclude_niit {
                rental.nonpassive_cents + rental.niit_disposition_loss
            } else {
                0
            };
            excluded_sources.push((
                value.id,
                lift(rental.net).map_err(overflow)?,
                own_excluded,
                Some(if rental.real_estate { 1 } else { 2 }),
            ));
            excluded += own_excluded;
            all_excluded &= lift(rental.net).map_err(overflow)? == own_excluded;
            all_included &= own_excluded == 0;
        }
    }
    for (id, value) in &direct {
        section1231_components_zero &= value.section1231 == 0;
        // Passive property retains investment character independently of
        // the source's Schedule C self-employment earnings.
        if direct_property_is_passive(form_context, *id) {
            excluded_sources.push((*id, value.section1231, 0, None));
            all_excluded &= value.section1231 == 0;
            passive_ordinary += value.ordinary;
            passive_ordinary_components_zero &= value.ordinary == 0;
            continue;
        }
        // The business is on the return, or the source is a Form 2106 vehicle
        // (an employee's trade or business, which the sale does not take out
        // of the business-property exclusion). Passive sources were classified
        // above from this invocation's actual participation view.
        if *id >= 0
            && !draft
                .income
                .iter()
                .any(|e| e.id == *id && matches!(e.income, FederalIncome::Business { .. }))
        {
            return Err((
                "ISSUE_SECTION1231_DIRECT_BUSINESS_PROPERTY_SALE_BUSINESS_IS_MISSING_4887B9F5",
                crate::fixture_english!("Direct business-property sale business is missing."),
            )
                .into());
        }
        excluded += value.section1231;
        excluded_sources.push((*id, value.section1231, value.section1231, None));
        all_included &= value.section1231 == 0;
        excluded_components_zero &= value.section1231 == 0;
    }
    for (id, value) in &rentals {
        section1231_components_zero &= value.section1231 == 0;
        excluded_sources.push((*id, value.section1231, 0, None));
        all_excluded &= value.section1231 == 0;
    }
    for (id, value, business) in &earlier {
        section1231_components_zero &= value.section1231 == 0;
        let own_excluded = if *business { value.section1231 } else { 0 };
        excluded += own_excluded;
        excluded_sources.push((*id, value.section1231, own_excluded, None));
        all_excluded &= value.section1231 == own_excluded;
        all_included &= own_excluded == 0;
        excluded_components_zero &= own_excluded == 0;
    }
    // Direct Part II ordinary income is absent from the NII baseline and is
    // excluded for these materially participating farms: no add/subtract needed.
    let excluded_capital = if character.net <= 0 || character.long_term == 0 {
        0
    } else if character.ordinary == 0 {
        excluded
    } else if all_included {
        0
    } else if all_excluded {
        character.long_term
    } else {
        let mut capital = 0;
        for (id, net, own_excluded, component) in excluded_sources {
            if own_excluded == 0 {
                continue;
            }
            if own_excluded != net {
                return Err((
                    "ISSUE_SECTION1231_MIXED_INVESTMENT_TAX_TREATMENT_WITHIN_A_PROPERTY_E12DAE20",
                    crate::fixture_english!(
                        "Mixed investment-tax treatment within a property source needs traced ordinary/capital allocation."
                    ),
                ).into());
            }
            let PropertyCharacter::Recaptured(amount) = (match component {
                Some(part) => component_character_cents_with_claims(
                    draft,
                    pool,
                    annual_character,
                    id,
                    false,
                    part,
                    source_claims,
                )?,
                None => activity_character_cents_with_claims(
                    draft,
                    pool,
                    annual_character,
                    id,
                    false,
                    source_claims,
                )?,
            }) else {
                return Err((
                    "ISSUE_SECTION1231_RESOLVE_SOURCE_RECAPTURE_BEFORE_INVESTMENT_TAX_CLASSIFICATION_031FC7BF",
                    crate::fixture_english!(
                        "Resolve source recapture before investment-tax classification."
                    ),
                ).into());
            };
            capital += own_excluded - amount;
        }
        capital
    };
    let [capital_adjustment, next_capital_adjustment] = crate::capital::niit_adjustment(
        lift(prior_difference).map_err(overflow)?,
        -excluded_capital + lift(other_exclusions).map_err(overflow)?,
        lift(capital_net).map_err(overflow)?,
        lift(capital_income).map_err(overflow)?,
    );
    Ok(PropertyNiit {
        adjustment: dollars(
            character.ordinary - (excluded - excluded_capital)
                + capital_adjustment
                + passive_ordinary,
        ),
        next_capital_adjustment: dollars(next_capital_adjustment),
        // A positive wholly capital pool has no ordinary current effect.
        // At net zero, prove each source zero rather than accepting offsetting
        // ordinary amounts from opposite owners as absent current effects.
        current_components_known_zero: character.ordinary == 0
            && (character.net > 0 || section1231_components_zero)
            && excluded_components_zero
            && passive_ordinary_components_zero,
    })
}

// A loss offset by the other character's gain has one unambiguous source.
// Histories of that loss character retain their qualified proportions; losses
// in the gain character require a separate allocation.
#[derive(Clone, Copy)]
pub(crate) enum ResolvedLoss {
    Operating(crate::kernel::carry::Amount<crate::kernel::carry::Cents>),
    Property(crate::kernel::carry::Amount<crate::kernel::carry::Cents>),
}
impl ResolvedLoss {
    pub(crate) fn amount(self) -> crate::kernel::carry::Amount<crate::kernel::carry::Cents> {
        match self {
            Self::Operating(v) | Self::Property(v) => v,
        }
    }
}
const LOSS_GAIN: (&str, &str) = (
    "ISSUE_SECTION1231_LOSS_GAIN_ALLOCATION_REQUIRES_ONE_ORDINARY_NONPASSIVE_39E47853",
    crate::fixture_english!(
        "Loss/gain allocation requires one ordinary nonpassive loss character and no loss history in the gain character."
    ),
);

fn opposite_loss(
    ordinary: i64,
    ordinary_history: [i64; 3],
    property: &crate::K1Section1231,
    parts: crate::scorp::ActivityCharacters,
) -> Result<ResolvedLoss, (&'static str, &'static str)> {
    if parts.prior_released != [0, 0] {
        return Err(LOSS_GAIN);
    }
    if ordinary <= 0
        && property.net > 0
        && parts.nonpassive[0] < 0
        && parts.nonpassive[1] == property.net
        && property.basis_loss == 0
        && property.risk_loss == 0
        && property.passive_loss == 0
    {
        Ok(ResolvedLoss::Operating(
            crate::kernel::carry::Amount::new(-parts.nonpassive[0])
                .expect("resolved ordinary loss is nonnegative"),
        ))
    } else if ordinary > 0
        && property.net <= 0
        && parts.nonpassive[1] < 0
        && parts.nonpassive[0] == ordinary
        && ordinary_history == [0; 3]
    {
        Ok(ResolvedLoss::Property(
            crate::kernel::carry::Amount::new(-parts.nonpassive[1])
                .expect("resolved property loss is nonnegative"),
        ))
    } else {
        Err(LOSS_GAIN)
    }
}

pub(crate) fn loss_with_gain<'a>(
    owner: impl Into<crate::scorp_amt::ScorpSource<'a>>,
) -> Result<ResolvedLoss, (&'static str, &'static str)> {
    let owner = owner.into();
    let property = owner.section1231.as_ref().ok_or(LOSS_GAIN)?;
    if owner.trade_kind != crate::ScorpTradeKind::Ordinary
        || owner.participation.as_ref().map(|v| v.material())
            != Some(crate::participation::Participation::Yes)
    {
        return Err(LOSS_GAIN);
    }
    let parts = owner.pool_activity_result()?.characters.ok_or(LOSS_GAIN)?;
    opposite_loss(
        owner.ordinary,
        [owner.prior_loss, owner.risk.prior_loss, owner.passive.loss],
        property,
        parts,
    )
}

pub(crate) fn partner_loss_with_gain(
    owner: &crate::PartnerK1,
    record: &crate::PartnerActivityRecord,
) -> Result<ResolvedLoss, (&'static str, &'static str)> {
    let property = record.section1231.as_ref().ok_or(LOSS_GAIN)?;
    if owner.trade_kind != crate::ScorpTradeKind::Ordinary
        || owner.material_for_losses() != crate::participation::Participation::Yes
        || owner.guaranteed_services != 0
        || owner.guaranteed_capital != 0
        || owner.reported_se != Some(0)
        || owner.pensions != 0
        || owner.retirement != crate::PartnerRetirement::None
        || !matches!(owner.health_insurance, crate::BusinessHealthInsurance::None)
        || owner.prior_fuel_credit.is_some()
        || owner.section179 != 0
        || owner.section179_carryover != 0
        || owner.disposition != Some(crate::K1Disposition::Kept)
        || record.rentals != crate::K1RentalAmounts::default()
        || !owner.section179_sales.is_empty()
        || !owner.income.is_empty()
        || !owner.deductions.is_empty()
    {
        return Err(LOSS_GAIN);
    }
    let parts = owner.pool(record)?.combined.characters.ok_or(LOSS_GAIN)?;
    opposite_loss(
        record.ordinary,
        [record.basis.prior_loss, record.risk.prior_loss, record.passive_loss],
        property,
        parts,
    )
}

// Resolve which part of section 461(l) deferral belongs to property losses.
// Opposite gain/loss characters in one business have a single loss source.
// Otherwise support property-only losses after earlier limits and histories;
// still-passive losses stay suspended. Mixed loss sources remain unresolved.
pub(crate) fn property_excess_allocation(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: &Result1231,
    excess: i64,
    amt: bool,
    direct_ordinary: i64,
    direct: &[(i64, crate::community::PropertyCents)],
) -> Result<i64, (&'static str, &'static str)> {
    let allocated_entries = crate::community::entries(draft);
    // A zero return-wide property net creates no deductible current
    // property loss or new lookback history to defer, even when operating
    // losses make Form461 apply. QBI attribution is validated separately.
    if excess == 0 || character.net == 0 {
        return Ok(0);
    }
    let message = (
        "ISSUE_SECTION1231_PROPERTY_EXCESS_BUSINESS_LOSS_REQUIRES_RESOLVED_975FCF28",
        crate::fixture_english!(
            "Property excess business loss requires resolved operating/property source allocation."
        ),
    );
    // Existing single-business EBL attribution does not resolve a K-1's
    // separate rental property activities. Retain its named unresolved
    // allocation instead of treating their aggregate as box 1 property.
    if pool.k1_section1231.iter().any(|source| !source.rentals.is_empty()) {
        return Err(message);
    }
    let businesses: Vec<_> = allocated_entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.income,
                FederalIncome::Business { .. }
                    | FederalIncome::Rental { .. }
                    | FederalIncome::FarmRental { .. }
                    | FederalIncome::Scorp { .. }
                    | FederalIncome::Partnership { .. }
            )
        })
        .collect();
    if pool.inputs_complete
        && businesses.len() == 1
        && let FederalIncome::Partnership { data: owner } = &businesses[0].income
    {
        let record = if amt { owner.alternative.as_ref().ok_or(message)? } else { &owner.regular };
        if record.ordinary != 0 || character.net > 0 {
            let loss = partner_loss_with_gain(owner, record)?;
            let parts = owner.pool(record)?.combined.characters.ok_or(message)?;
            if excess > 0
                && pool.partners.len() == 1
                && excess
                    <= match loss {
                        ResolvedLoss::Operating(_) => -pool.partners[0].net,
                        ResolvedLoss::Property(_) => -character.net,
                    }
                && pool.k1_section1231.len() == 1
                && pool.partners.len() == 1
                && pool.k1_section1231[0].id == businesses[0].id
                && pool.k1_section1231[0].net == character.net
                && character.net == dollars(parts.nonpassive[1])
                && pool.k1_section1231[0].suspended == dollars(-parts.passive[1])
                && pool
                    .k1_amounts
                    .iter()
                    .find(|amount| amount.id == businesses[0].id)
                    .and_then(|amount| amount.source_net_cents)
                    == Some(parts.nonpassive[0])
            {
                return Ok(match loss {
                    ResolvedLoss::Operating(_) => 0,
                    ResolvedLoss::Property(_) => excess,
                });
            }
            return Err(message);
        }
    }
    if pool.inputs_complete
        && businesses.len() == 1
        && direct.len() == 1
        && businesses[0].id == direct[0].0
        && pool.k1_section1231.is_empty()
        && character.in_cents().map_err(|_| message)?.net == direct[0].1.section1231
        && excess > 0
    {
        // One business with one net loss character: income in the other
        // character offsets that loss without requiring an allocation election.
        if character.net < 0 && direct_ordinary >= 0 && excess <= -character.net {
            return Ok(excess);
        }
        if character.net >= 0 && direct_ordinary < 0 && excess <= -direct_ordinary {
            return Ok(0);
        }
    }
    if pool.inputs_complete && character.net != 0 && excess > 0 {
        let owners: Vec<_> = allocated_entries
            .iter()
            .filter_map(|entry| match &entry.income {
                FederalIncome::Scorp { data } => Some(crate::ScorpK1::from(data)),
                _ => None,
            })
            .collect();
        let other_business = allocated_entries.iter().any(|entry| {
            matches!(
                entry.income,
                FederalIncome::Business { .. }
                    | FederalIncome::Rental { .. }
                    | FederalIncome::FarmRental { .. }
                    | FederalIncome::Partnership { .. }
            )
        });
        if owners.len() == 1 && !other_business {
            let owner: crate::scorp_amt::ScorpSource<'_> = if amt {
                owners[0].amt.as_ref().ok_or(message)?.source(&owners[0])?
            } else {
                (&owners[0]).into()
            };
            if owner.ordinary != 0 || character.net > 0 {
                let loss = loss_with_gain(&owner)?;
                let parts = owner.pool_activity_result()?.characters.ok_or(message)?;
                if excess
                    <= match loss {
                        ResolvedLoss::Operating(_) => -pool.scorp_business,
                        ResolvedLoss::Property(_) => -character.net,
                    }
                    && pool.k1_section1231.len() == 1
                    && pool.k1_section1231[0].net == character.net
                    && pool.k1_section1231[0].net == dollars(parts.nonpassive[1])
                    && pool.k1_section1231[0].suspended == dollars(-parts.passive[1])
                    && pool
                        .k1_amounts
                        .iter()
                        .find(|amount| amount.id == pool.k1_section1231[0].id)
                        .and_then(|amount| amount.source_net_cents)
                        == Some(parts.nonpassive[0])
                {
                    return Ok(match loss {
                        ResolvedLoss::Operating(_) => 0,
                        ResolvedLoss::Property(_) => excess,
                    });
                }
                return Err(message);
            }
        }
    }
    if !pool.inputs_complete
        || character.net >= 0
        || excess < 0
        || excess > -character.net
        || allocated_entries.iter().any(|entry| {
            matches!(
                entry.income,
                FederalIncome::Business { .. }
                    | FederalIncome::Rental { .. }
                    | FederalIncome::FarmRental { .. }
            )
        })
    {
        return Err(message);
    }
    let mut total = 0i64;
    let mut count = 0usize;
    for entry in &crate::community::property_entries(draft) {
        let parts = match &entry.income {
            FederalIncome::Partnership { data: owner } => {
                let record =
                    if amt { owner.alternative.as_ref().ok_or(message)? } else { &owner.regular };
                if owner.trade_kind != crate::ScorpTradeKind::Ordinary
                    || owner.material_for_losses() != crate::participation::Participation::Yes
                    || record.ordinary != 0
                    || record.basis.prior_loss != 0
                    || record.risk.prior_loss != 0
                    || record.passive_loss != 0
                    || owner.guaranteed_services != 0
                    || owner.guaranteed_capital != 0
                {
                    return Err(message);
                }
                let Some(property) = record.business_property(&owner.items_for_record(record)?)?
                else {
                    continue;
                };
                if property.gain != 0 {
                    return Err(message);
                }
                let activity = owner.pool(record)?;
                if activity.items.net() != 0 {
                    return Err(message);
                }
                activity.combined.characters.ok_or(message)?
            }
            FederalIncome::Scorp { data: owner } => {
                let owner = &crate::ScorpK1::from(owner);
                let owner: crate::scorp_amt::ScorpSource<'_> = if amt {
                    owner.amt.as_ref().ok_or(message)?.source(owner)?
                } else {
                    owner.into()
                };
                if owner.trade_kind != crate::ScorpTradeKind::Ordinary
                    || owner.participation.as_ref().map(|v| v.material())
                        != Some(crate::participation::Participation::Yes)
                    || owner.ordinary != 0
                    || owner.prior_loss != 0
                    || owner.risk.prior_loss != 0
                    || owner.passive.loss != 0
                {
                    return Err(message);
                }
                let Some(property) = owner.business_property()? else {
                    continue;
                };
                if property.gain != 0 {
                    return Err(message);
                }
                let activity = owner.pool_activity_result()?;
                if activity.items.net() != 0 {
                    return Err(message);
                }
                activity.characters.ok_or(message)?
            }
            _ => continue,
        };
        let resolved = pool.k1_section1231.iter().find(|v| v.id == entry.id).ok_or(message)?;
        if parts.nonpassive != [0, resolved.net_cents]
            || parts.passive[0] != 0
            || resolved.nonpassive_cents != resolved.net_cents
            || resolved.suspended_cents.get() != -parts.passive[1]
        {
            return Err(message);
        }
        total = total.checked_add(resolved.net).ok_or(message)?;
        count += 1;
    }
    if count == 0 || count != pool.k1_section1231.len() || total != character.net {
        return Err(message);
    }
    Ok(excess)
}

// Call only after all return checks succeed and no section 461(l) property-loss
// allocation remains. The resolved pool already excludes suspended losses.
pub(crate) fn record_final_history(
    character: &Result1231,
    deferred: i64,
    amt: bool,
    result: &mut FederalResult,
) {
    let prefix = if amt { "amt_section1231" } else { "section1231" };
    #[cfg(test)]
    let label = if amt { "AMT" } else { "regular" };
    let current = phase::floor(-character.net, 0) - deferred;
    let history = character.next_history(current).expect("validated final section 1231 loss");
    let source = "https://www.irs.gov/publications/p544";
    crate::line_emission::line(
        result,
        &alloc::format!("{prefix}_current_loss_history"),
        current,
        &["income.scorp.section1231", "income.partnership"],
        source,
        msg!("EXPLAIN_N_CURRENT_LOSS_HISTORY"),
    );
    for year in &history {
        crate::line_emission::line(
            result,
            &alloc::format!("{prefix}_{}_next_lookback", year.year),
            year.remaining,
            &["screening.section1231", "income.scorp.section1231", "income.partnership"],
            source,
            msg!("EXPLAIN_N_N_NEXT_LOOKBACK"),
        );
        result.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
            "Finalized section 1231 {label} history for {}: loss year {}, original deductible loss ${}, remaining balance ${}. Keep the year and original amount with this balance; it is not a second loss deduction.",
            character.year + 1, year.year, year.original, year.remaining,
        ), Message::new(if amt { "NOTE_1231_AMT_FINAL_HISTORY" } else { "NOTE_1231_REGULAR_FINAL_HISTORY" }, alloc::vec![
            MessageArg::Number((character.year + 1) as i64), MessageArg::Number(year.year as i64),
            MessageArg::usd_dollars(year.original), MessageArg::usd_dollars(year.remaining),
        ])));
    }
}

// Form 4952 disposition amounts, [ordinary, long-term capital]. Property of
// a nonpassive business is investment property when participation is not
// material (section 163(d)(5)(A)(ii)); passive business property is excluded.

pub(crate) fn investment_interest_amounts_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: &Result1231,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<[i64; 2], SourceError> {
    let allocated_entries = crate::community::entries(draft);
    if !pool.inputs_complete {
        return Err(SourceError::from((
            "ISSUE_SECTION1231_RESOLVE_THE_PROPERTY_ACTIVITY_POOL_BEFORE_05F7B354",
            crate::fixture_english!(
                "Resolve the property activity pool before the investment-interest worksheet."
            ),
        )));
    }
    let mut eligible = Vec::new();
    for item in pool.k1_section1231.iter().map(crate::extended_rules::K1PropertyResult::business) {
        if allocated_entries
            .iter()
            .find(|e| e.id == item.id)
            .and_then(|e| k1_property(&e.income, amt))
            .is_none()
        {
            continue;
        }
        if let Some(partner) = allocated_entries.iter().find_map(|entry| match &entry.income {
            FederalIncome::Partnership { data } if entry.id == item.id => Some(data),
            _ => None,
        }) {
            if partner.trade_kind == crate::ScorpTradeKind::Ordinary
                || partner.participation.as_ref().map_or(
                    crate::participation::Participation::Unavailable,
                    crate::PartnerParticipation::material,
                ) == crate::participation::Participation::Yes
            {
                continue;
            }
            if partner.participation.as_ref().map_or(
                crate::participation::Participation::Unavailable,
                crate::PartnerParticipation::material,
            ) == crate::participation::Participation::Unavailable
            {
                return Err(SourceError::income(
                    item.id,
                    (
                        "ISSUE_SCORP_RESOLVE_TRADING_BUSINESS_CLASSIFICATION_AND_MATERIAL_PARTICIPA_EDFB91CC",
                        crate::fixture_english!(
                            "Resolve trading-business classification and material participation for the investment-interest income limit."
                        ),
                    ),
                ));
            }
            if (if amt { partner.alternative.as_ref() } else { Some(&partner.regular) })
                .and_then(|record| record.section1231.as_ref())
                .is_some_and(|value| dollars(value.passive_loss) != item.suspended)
            {
                return Err(SourceError::income(item.id, (
                    "ISSUE_SECTION1231_RELEASED_FORMER_PASSIVE_PROPERTY_LOSSES_NEED_SEPARATE_EB3B9611",
                    crate::fixture_english!("Released former-passive property losses need separate investment-interest attribution before combining trading property amounts."),
                )).with_manual());
            }
            eligible.push(item);
            continue;
        }
        let owner = allocated_entries
            .iter()
            .find_map(|entry| {
                if entry.id == item.id
                    && let FederalIncome::Scorp { data } = &entry.income
                {
                    Some(crate::ScorpK1::from(data))
                } else {
                    None
                }
            })
            .ok_or((
                "ISSUE_SECTION1231_AN_INVESTMENT_INTEREST_PROPERTY_RESULT_HAS_NO_5284D6A5",
                crate::fixture_english!(
                    "An investment-interest property result has no matching source business."
                ),
            ))?;
        let alternative;
        let source = if amt {
            alternative = owner
                .amt
                .as_ref()
                .ok_or((
                    "ISSUE_SECTION1231_PROVIDE_INDEPENDENT_AMT_PROPERTY_RECORDS_FOR_50C0DB6F",
                    crate::fixture_english!(
                        "Provide independent AMT property records for investment interest."
                    ),
                ))?
                .source(&owner)
                .map_err(|detail| SourceError::income(item.id, detail))?;
            &alternative
        } else {
            &owner
        };
        if !source
            .investment_interest_eligible()
            .map_err(|detail| SourceError::income(item.id, detail))?
        {
            continue;
        }
        if source
            .section1231
            .as_ref()
            .is_some_and(|value| dollars(value.passive_loss) != item.suspended)
        {
            return Err(SourceError::income(item.id, (
                "ISSUE_SECTION1231_RELEASED_FORMER_PASSIVE_PROPERTY_LOSSES_NEED_SEPARATE_EB3B9611",
                crate::fixture_english!(
                    "Released former-passive property losses need separate investment-interest attribution before combining trading property amounts."
                ),
            )).with_manual());
        }
        eligible.push(item);
    }
    if eligible.is_empty() {
        return Ok([0, 0]);
    }
    if pool.k1_section1231.iter().all(|v| v.rentals.is_empty())
        && eligible.len() == pool.k1_section1231.len()
        && crate::asset_sale::allowed_direct_sources_with_claims(draft, pool, amt, source_claims)?
            .iter()
            .all(|(_, value)| value.section1231 == 0)
        && crate::asset_sale::allowed_rental_sources(
            crate::related_forms::RelatedFormContext::original(draft, source_claims),
            pool,
            amt,
        )?
        .iter()
        .all(|(_, value)| value.section1231 == 0)
        && earlier_sources(draft).iter().all(|(_, v, _)| v.section1231 == 0)
    {
        return Ok([character.ordinary, character.long_term]);
    }
    let mut amounts = [0, 0];
    for item in eligible {
        match component_character_with_claims(
            draft,
            pool,
            character,
            item.id,
            amt,
            0,
            source_claims,
        )
        .map_err(SourceError::with_manual)?
        {
            PropertyCharacter::Ordinary => amounts[0] += item.net,
            PropertyCharacter::Capital => amounts[1] += item.net,
            PropertyCharacter::Recaptured(amount) => {
                amounts[0] += amount;
                amounts[1] += item.net - amount;
            }
        }
    }
    Ok(amounts)
}
