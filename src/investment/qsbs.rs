// Coverage: Section 1202 qualified small business stock (B7a, 2026-10-09). QsbIssuerSale (one issuing
//   corporation's 2026 sales of original-issue stock held more than five years, issued August 11,
//   1993 through July 4, 2025: 50, 75 or 100 percent exclusion by issue date, 60 percent with the
//   empowerment-zone increase, the per-issuer limit of the greater of $10,000,000 (MFS $5,000,000)
//   less prior eligible gain and ten times the issuer stock basis sold this year, the included part
//   of a 50, 60 or 75 percent exclusion as 28 percent rate gain, the 7 percent AMT preference of
//   Form 6251 line 2h for stock acquired before September 28, 2010; IRC 1202, 1(h)(4) and (7),
//   57(a)(7); Instructions for Schedule D) gains its page, QsbsSale, on Investment sales, with the
//   sale rows, box 4 and the Form 8949 box imported and the issuer facts entered. Stock issued
//   after July 4, 2025 falls under the P.L. 119-21 section 70431 three-, four- and five-year tiers,
//   whose shortest holding period cannot be met by a 2026 sale (earliest July 5, 2028), so such a
//   lot is refused by QSBS_LOT_OUTSIDE_SECTION_1202_DATES; the limits and dates are in
//   year_constants.rs. The taxable gain is reported on Form 8949 with code Q in the box the page
//   names (D, E or F) and totals on Schedule D line 8b, 9 or 10; the excluded part is column (g)
//   negative. No qualification answer of Yes, or an incomplete history, taxes the whole gain
//   (WARN_QSBS_ASSUMED_NOT_QUALIFIED, WARN_QSBS_EXCLUSION_LEFT_OUT); special treatment leaves the
//   sale out with WARN_QSBS_SALE_LEFT_OUT. Forms: 8949 (code Q), Schedule D, 6251 line 2h, 28%
//   Rate Gain Worksheet line 2. Tests: src/tests/investment/investment_kinds.rs qsbs_*.
//   Client lot-date validation uses typed calendar dates and add_months for the
//   strict five-year hold, matching the unchanged native supported-date gate.
//   Tests: investment_kinds validation_audit_f11_* (fresh/null/reopen/rejected state).
//   Native source outputs retain admitted per-entry/owner28% cents and actual
//   cohort/zone taxable rows; issuer totals and uniform-cap rounding are unchanged.
//   Existing refused mixed caps/history remain unavailable, not guessed owners.
//   Evidence: tests/unit13_qsbs_source.rs (mixed dates/owners/zone/caps/cents/Save).
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::year_constants as yc;
use crate::{FederalDraft, FederalIncome, FederalResult};
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

const MAX: i64 = 100_000_000_000;
const SOURCE: &str = "https://www.irs.gov/instructions/i1040sd";

/// Qualified small-business stock lot
/// A directly purchased original-issue lot sold for cash in 2026. Enter separate lots for different issue dates or original bases. Gifts, inheritances, exchanged shares, pass-through shares, installment sales and section 1045 rollovers require their separate rules.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct QsbStockLot {
    /// Original-issue acquisition date
    /// This path covers shares acquired August 11, 1993 through July 4, 2025 and held more than five years. The acquisition date determines the 50%, 75% or 100% exclusion. In a 2026 direct sale, shares issued after July 4, 2025 cannot yet satisfy their minimum holding period.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub issued: u32,
    /// 2026 cash sale date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub sold: u32,
    /// Original issue price and capitalized acquisition costs for this sold lot
    /// Do not add later capital contributions or another basis increase. This same unchanged amount is both adjusted sale basis and the original-issue basis used for the ten-times-basis limit.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub basis: i64,
    /// Net cash proceeds after selling costs
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub proceeds: i64,
    /// Gain attributable to time through December 31, 2018, for the 60% empowerment-zone exclusion
    /// Complete only when this lot was acquired December 22, 2000 through February 17, 2009 and the issuer qualified as an empowerment-zone business for the required period. Establish value at December 31, 2018 (or an earlier cessation date) and limit this amount to actual 2026 realized gain. Enter gain, not proceeds or a second stock basis.
    /// [NullText]
    /// No pre-2019 empowerment-zone gain is established
    /// [SetText]
    /// The gain attributable to time through December 31, 2018 is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub zone_gain_through_2018: Option<i64>,
}

impl QsbStockLot {
    // The dates a 2026 sale of original-issue stock must meet: issued from
    // August 11, 1993 through July 4, 2025, sold in 2026 and more than five
    // years after the issue.
    fn dates_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if !crate::federal_rules::valid_date(self.issued)
            || !(yc::SECTION_1202_FIRST_ISSUE_DATE..=yc::SECTION_1202_LAST_ORIGINAL_TIER_ISSUE_DATE)
                .contains(&self.issued)
        {
            return Some(
                ValidationIssue::new("QSBS_LOT_OUTSIDE_SECTION_1202_DATES").at_field("issued"),
            );
        }
        if !crate::federal_rules::valid_date(self.sold)
            || self.sold / 10000 != 2026
            || self.sold <= self.issued + yc::SECTION_1202_HOLDING_YEARS_AS_DATE
        {
            return Some(
                ValidationIssue::new("QSBS_LOT_OUTSIDE_SECTION_1202_DATES").at_field("sold"),
            );
        }
        None
    }
    pub(crate) fn dates_valid(&self) -> bool {
        self.dates_issue().is_none()
    }

    // The first page relationship this lot fails, by its key (the client
    // rules above and their server twins): a loss or break-even sale of
    // this stock is a 1099-B sale, not a section 1202 gain, and a zone-gain
    // figure must fit the lot's gain and the zone dates.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if let Some(issue) = self.dates_issue() {
            return Some(issue);
        }
        if self.proceeds <= self.basis {
            return Some(
                crate::validation::ValidationIssue::new("QSBS_LOT_PROCEEDS_NOT_ABOVE_BASIS")
                    .at_field("proceeds"),
            );
        }
        let zone = self.zone_gain_through_2018?;
        if !(0..=self.proceeds - self.basis).contains(&zone) {
            return Some(
                crate::validation::ValidationIssue::new("QSBS_LOT_ZONE_GAIN_OUTSIDE_THE_LOT_GAIN")
                    .at_field("zone_gain_through_2018"),
            );
        }
        if !(yc::SECTION_1202_ZONE_FIRST_DATE..=yc::SECTION_1202_FIFTY_PERCENT_LAST_DATE)
            .contains(&self.issued)
        {
            return Some(
                crate::validation::ValidationIssue::new("QSBS_LOT_ZONE_GAIN_OUTSIDE_ZONE_DATES")
                    .at_field("zone_gain_through_2018"),
            );
        }
        None
    }

    pub(crate) fn valid(&self) -> bool {
        self.dates_valid()
            && (1..=MAX).contains(&self.basis)
            && (0..=MAX).contains(&self.proceeds)
            && self.proceeds > self.basis
            && self.zone_gain_through_2018.is_none_or(|zone| {
                (yc::SECTION_1202_ZONE_FIRST_DATE..=yc::SECTION_1202_FIFTY_PERCENT_LAST_DATE)
                    .contains(&self.issued)
                    && (0..=self.proceeds - self.basis).contains(&zone)
            })
    }

    fn exclusion_percentage(&self) -> i64 {
        if self.issued <= yc::SECTION_1202_FIFTY_PERCENT_LAST_DATE {
            50
        } else if self.issued <= yc::SECTION_1202_SEVENTY_FIVE_PERCENT_LAST_DATE {
            75
        } else {
            100
        }
    }
}

/// Direct sale of qualified small-business stock — section 1202
/// Enter all 2026 sales for one issuing corporation, using the same issuer name and prior-gain history if multiple source records or spouses are entered. The exclusion limit applies once to all sales from that issuer on this return, not once per lot or document.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QsbIssuerSale {
    /// Exact legal name of the issuing corporation
    /// Use the same name on every entry for this issuer. Do not use a broker name, brand, predecessor name or ticker in place of the legal issuer identity.
    #[libertas_size(min = 1)]
    pub issuer: String,
    /// Eligible gain already taken into account under section 1202 for this issuer before 2026
    /// Reconcile every prior return of the current taxpayer and, for a joint return, both spouses. Allocate gain taken into account on an earlier joint return equally to its spouses for later years. Enter zero only after checking the complete history. Do not enter prior sale proceeds or prior taxable gain here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_eligible_gain: i64,
    /// Bought other than for cash at original issue, or a gift, inheritance, exchange, pass-through ownership, installment sale, rollover or another special treatment
    /// Yes leaves this sale out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
    /// Qualified domestic C corporation and gross-assets test confirmed for every issue date
    /// The issuer was a domestic C corporation. Aggregate assets of the issuer, its predecessors and its controlled group did not exceed $50 million immediately before and after each original issue; use the statutory contribution valuation rules. No applies no section 1202 exclusion, so the whole gain is taxed.
    pub issuer_qualified: bool,
    /// Active qualified business confirmed throughout substantially all of each holding period
    /// At least 80% of asset value was used in a qualified active business. Exclude disallowed professional services, finance, investment, farming, mining and hospitality businesses; account for subsidiaries, working capital and permitted exceptions from issuer records. No applies no section 1202 exclusion, so the whole gain is taxed.
    pub active_business: bool,
    /// No disqualifying issuer redemption or other section 1202 exception
    /// Check the applicable shareholder and significant redemption windows around every issuance; confirm no special rule or election changes qualification, gain recognition or tax character. No applies no section 1202 exclusion, so the whole gain is taxed.
    pub no_disqualification: bool,
    /// Extra 60% empowerment-zone exclusion applies to one or more sold lots
    /// For shares acquired December 22, 2000 through February 17, 2009, confirm No only after checking the issuer's empowerment-zone status and gain through 2018. Choose Yes when the issuer qualified and the source valuation determines the eligible gain for each affected lot. Check the issuer's retained zone certification and valuation; an unavailable historical certification or valuation leaves the increase unclaimed with a notice. Leave absent when all lots are outside this issue-date interval.
    // When an applicable issuer's historical zone certification or valuation is unavailable,
    // purchase/sale amounts cannot reconstruct it; WARN_QSBS_ZONE_ASSUMED_NO omits the increase.
    // Outside the statutory issue-date interval the absent answer remains not applicable.
    pub zone_60_exclusion: Option<bool>,
    /// Complete issuer history and all current-year lots confirmed
    /// The prior-gain figure and every 2026 disposition of this issuer's qualifying shares on the filing unit are included once. A joint return needs its spouses' separate and joint prior-return history. Your prior returns' section 1202 exclusions for this issuer show it. No applies no section 1202 exclusion, so the whole gain is taxed, with a notice.
    pub complete_history: bool,
    /// 2026 sold lots
    pub lots: Vec<QsbStockLot>,
    /// Form 8949 box of the sales
    /// Box D when the broker reported the basis to the IRS, E when it did not, F when no Form 1099-B was received.
    pub reporting: crate::BasisReporting,
    /// Federal income tax withheld from these sales
    /// Allocate payer-statement withholding once; do not repeat the same Form 1099 amount on another entry.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub withholding: i64,
}

impl QsbIssuerSale {
    pub(crate) fn valid(&self) -> bool {
        !self.issuer.trim().is_empty()
            && (0..=MAX).contains(&self.prior_eligible_gain)
            && (0..=MAX).contains(&self.withholding)
            && (1..=100).contains(&self.lots.len())
            && self.lots.iter().all(QsbStockLot::valid)
    }

    // The exclusion applies only with confirmed issuer qualification and a
    // complete history; otherwise the whole gain is taxed.
    fn excluded(&self) -> bool {
        self.complete_history
            && [self.issuer_qualified, self.active_business, self.no_disqualification]
                .iter()
                .all(|answer| *answer)
    }

    // The zone increase is omitted when issuer source records are unavailable:
    // an absent applicable answer, or Yes with no source zone gain entered.
    // Only a Yes uses the entered zone gain.
    fn zone_assumed_no(&self) -> bool {
        match self.zone_60_exclusion {
            Some(false) => false,
            Some(true) => {
                self.lots.iter().filter_map(|lot| lot.zone_gain_through_2018).sum::<i64>() == 0
            }
            None => self.lots.iter().any(|lot| {
                (yc::SECTION_1202_ZONE_FIRST_DATE..=yc::SECTION_1202_FIFTY_PERCENT_LAST_DATE)
                    .contains(&lot.issued)
            }),
        }
    }

    fn note(&self, key: &str, r: &mut FederalResult) {
        crate::federal_rules::note_once(
            r,
            crate::Message::new(key, alloc::vec![crate::MessageArg::Text(self.issuer.clone())]),
        );
    }
}

// A fund's section 1202 gain through a regulated investment company (IRC 1202(g)(4)(B); Form
// 1099-DIV box 2c, Form 2439 box 1c), in cents: the fund's gain is one issuer's, so the per-issuer
// limit (the greater of $10,000,000, $5,000,000 if married filing separately, and ten times the
// fund's basis, unknown here and so not used) caps what is eligible. The exclusion follows the
// date the fund bought the stock; no answer other than None gives no exclusion.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FundGain {
    pub excluded: i64,
    // The 28 percent rate gain left after a 50 or 75 percent exclusion.
    pub rate_28: i64,
    pub amt_preference: i64,
}

pub(crate) fn fund_gain(
    gain: i64,
    exclusion: Option<crate::FundQsbsExclusion>,
    separate: bool,
) -> FundGain {
    let percent = match exclusion {
        Some(crate::FundQsbsExclusion::Fifty) => 50,
        Some(crate::FundQsbsExclusion::SeventyFive) => 75,
        Some(crate::FundQsbsExclusion::Hundred) => 100,
        Some(crate::FundQsbsExclusion::None) | None => 0,
    };
    let limit = if separate {
        yc::SECTION_1202_ISSUER_LIMIT_SEPARATE_CENTS
    } else {
        yc::SECTION_1202_ISSUER_LIMIT_CENTS
    };
    let eligible = phase::cap(gain, [limit]);
    let excluded = crate::kernel::ratio::fraction(eligible, percent, 100);
    let older = percent == 50 || percent == 75;
    FundGain {
        excluded,
        rate_28: if older { eligible - excluded } else { 0 },
        amt_preference: if older {
            crate::kernel::ratio::fraction(excluded, yc::SECTION_1202_AMT_PREFERENCE_PERCENT, 100)
        } else {
            0
        },
    }
}

#[derive(Default)]
pub(crate) struct QsbResult {
    pub taxable_gain: i64,
    pub excluded_gain: i64,
    pub withholding: i64,
    pub rate_28_gain: i64,
    pub amt_preference: i64,
    // Actual admitted source cents, before any owner dollar projection.
    pub sources: Vec<QsbSource>,
    // One Form 8949 row for each entry, in the box (D, E, F) its page names:
    // Schedule D lines 8b, 9 and 10, with the excluded part as code Q's negative
    // column (g).
    pub rows: Vec<crate::capital::Form8949Row>,
}

// Retained at the native issuer/cohort calculation, never inferred by a
// downstream owner from a final joint rate pool. None is an existing refused
// cap/history allocation, not an assumed zero or a new taxpayer question.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct QsbSource {
    pub id: i64,
    pub owner: crate::Owner,
    pub taxable_gain: i64,
    pub withholding: i64,
    pub rate_28_gain: Option<i64>,
}

// The entry's Form 8949 row: its lots' proceeds and basis, and the taxable gain as column (h).
fn row(
    entry: &crate::FederalIncomeEntry,
    data: &QsbIssuerSale,
    taxable: i64,
) -> crate::capital::Form8949Row {
    let proceeds: i64 = data.lots.iter().map(|lot| lot.proceeds).sum();
    let basis: i64 = data.lots.iter().map(|lot| lot.basis).sum();
    let index = box_index(&data.reporting);
    crate::capital::Form8949Row {
        entry: entry.id,
        row: 1,
        line: 5 + index,
        box_letter: ['D', 'E', 'F'][index],
        label: crate::interview::income_display_arg(entry),
        proceeds,
        basis,
        adjustment: taxable - (proceeds - basis),
        codes: if taxable == proceeds - basis { String::new() } else { "Q".into() },
        summary: false,
    }
}

fn box_index(reporting: &crate::BasisReporting) -> usize {
    match reporting {
        crate::BasisReporting::ReportedToIrs => 0,
        crate::BasisReporting::NotReported => 1,
        crate::BasisReporting::NoForm1099B => 2,
    }
}

pub(crate) fn calculate(d: &FederalDraft, r: &mut FederalResult) -> QsbResult {
    let mut by_issuer: BTreeMap<String, Vec<(&crate::FederalIncomeEntry, &QsbIssuerSale)>> =
        BTreeMap::new();
    let mut result = QsbResult::default();
    for entry in &d.income {
        if let FederalIncome::QualifiedSmallBusinessSale { data } = &entry.income {
            if !data.valid() {
                continue;
            }
            if !data.excluded() {
                // No exclusion: the whole long-term gain is taxed.
                if !data.complete_history {
                    data.note("WARN_QSBS_EXCLUSION_LEFT_OUT", r);
                }
                let gain: i64 = data.lots.iter().map(|lot| lot.proceeds - lot.basis).sum();
                result.taxable_gain += gain;
                result.rows.push(row(entry, data, gain));
                result.sources.push(QsbSource {
                    id: entry.id,
                    owner: entry.owner,
                    taxable_gain: gain,
                    withholding: data.withholding,
                    rate_28_gain: Some(0),
                });
                result.withholding += data.withholding;
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("realized_gain", gain, msg!("EXPLAIN_QSBS_N_N_REALIZED_GAIN")),
                        ("taxable_gain", gain, msg!("EXPLAIN_QSBS_N_N_TAXABLE_GAIN")),
                    ],
                    |(suffix, amount, explanation), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!("qsbs_{}_{}", entry.id, suffix),
                            LineValue::MoneyCents(amount),
                            &["income.qualified_small_business_sale"],
                            SOURCE,
                            explanation,
                        ))
                    },
                );
                continue;
            }
            if data.zone_assumed_no() {
                data.note("WARN_QSBS_ZONE_ASSUMED_NO", r);
            }
            by_issuer
                .entry(data.issuer.trim().to_ascii_lowercase())
                .or_default()
                .push((entry, data));
        }
    }
    for entries in by_issuer.values() {
        let prior = entries[0].1.prior_eligible_gain;
        crate::require_localized!(
            r,
            entries.iter().all(|(_, data)| data.prior_eligible_gain == prior),
            &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            "Section 1202 issuer history",
            "ISSUE_ALL_ENTRIES_FOR_ONE_ISSUING_CORPORATION_MUST_USE_ONE_RECONCILED_PRI_9F0EB02F",
            "All entries for one issuing corporation must use one reconciled prior eligible-gain amount; the exclusion cap is shared across its current-year lots.",
        );
        let mut gain = 0i64;
        let mut basis = 0i64;
        let mut zone_gain = 0i64;
        let mut cohort_gain = [0i64; 3];
        let mut source_cohorts = Vec::new();
        let mut source_zones = Vec::new();
        for (_, data) in entries {
            let mut own_cohorts = [0; 3];
            let mut own_zone = 0;
            for lot in &data.lots {
                let lot_gain = lot.proceeds - lot.basis;
                gain += lot_gain;
                basis += lot.basis;
                // Only a Yes zone answer uses the entered zone gain.
                if data.zone_60_exclusion == Some(true) {
                    let zone = lot.zone_gain_through_2018.unwrap_or(0);
                    zone_gain += zone;
                    own_zone += zone;
                }
                let cohort = match lot.exclusion_percentage() {
                    50 => 0,
                    75 => 1,
                    _ => 2,
                };
                cohort_gain[cohort] += lot_gain;
                own_cohorts[cohort] += lot_gain;
            }
            source_cohorts.push(own_cohorts);
            source_zones.push(own_zone);
            result.withholding += data.withholding;
        }
        let separate = d
            .setup
            .as_ref()
            .is_some_and(|setup| setup.filing_status() == crate::FederalStatus::Separate);
        let limit = phase::floor(
            phase::floor(
                (if separate {
                    yc::SECTION_1202_ISSUER_LIMIT_SEPARATE_CENTS
                } else {
                    yc::SECTION_1202_ISSUER_LIMIT_CENTS
                } - prior),
                0,
            ),
            basis * yc::SECTION_1202_BASIS_MULTIPLE,
        );
        let eligible = phase::cap(gain, [limit]);
        let mixed_cohorts = cohort_gain.iter().filter(|amount| **amount > 0).count() > 1;
        crate::require_localized!(
            r,
            !mixed_cohorts || eligible == gain,
            &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            "Section 1202 mixed acquisition cohorts",
            "ISSUE_A_BINDING_ISSUER_WIDE_SECTION_1202_ELIGIBLE_GAIN_CAP_WITH_50_75_OR_78E873C5",
            "A binding issuer-wide section 1202 eligible-gain cap with 50%, 75% or 100% acquisition cohorts needs an explicit allocation of that cap across cohorts.",
        );
        crate::require_localized!(
            r,
            zone_gain == 0 || zone_gain == gain || eligible == gain,
            &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            "Section 1202 zone issuer limit",
            "ISSUE_A_BINDING_ISSUER_WIDE_SECTION_1202_LIMIT_WITH_BOTH_EMPOWERMENT_ZONE_0C92F677",
            "A binding issuer-wide section 1202 limit with both empowerment-zone and ordinary gain needs a verified allocation between 60% and 50% eligible gain.",
        );
        let excluded_50 = crate::kernel::ratio::fraction(cohort_gain[0], 50, 100);
        let excluded_75 = crate::kernel::ratio::fraction(cohort_gain[1], 75, 100);
        let zone_increase = crate::kernel::ratio::fraction(zone_gain, 10, 100);
        let (excluded, amt_preference) = if eligible == gain {
            let older_excluded = excluded_50 + excluded_75 + zone_increase;
            (
                older_excluded + cohort_gain[2],
                crate::kernel::ratio::fraction(
                    older_excluded,
                    yc::SECTION_1202_AMT_PREFERENCE_PERCENT,
                    100,
                ),
            )
        } else {
            // Under a binding cap, every eligible dollar must have one rate.
            // The all-zone case is uniformly 60%; a mixture of zone and
            // ordinary gain still needs an explicit cap allocation.
            let percentage =
                if zone_gain == gain { 60 } else { entries[0].1.lots[0].exclusion_percentage() };
            let excluded = crate::kernel::ratio::fraction(eligible, percentage, 100);
            (
                excluded,
                if percentage < 100 {
                    crate::kernel::ratio::fraction(
                        excluded,
                        yc::SECTION_1202_AMT_PREFERENCE_PERCENT,
                        100,
                    )
                } else {
                    0
                },
            )
        };
        let taxable = gain - excluded;
        let rate_28 = eligible - excluded;
        let source_known = entries.iter().all(|(_, data)| data.prior_eligible_gain == prior)
            && (!mixed_cohorts || eligible == gain)
            && (zone_gain == 0 || zone_gain == gain || eligible == gain);
        let source_gains: Vec<i64> = source_cohorts.iter().map(|c| c.iter().sum()).collect();
        let (taxable_sources, rate_sources) = if eligible == gain {
            // Each admitted older cohort owns its included amount. The native
            // issuer rounding stays unchanged, with its residual projected only
            // among sources of that actual cohort. A 100% lot owns no taxable
            // or28% gain. Zone weights retain each source's actual10% increase.
            let weights_50: Vec<i64> = source_cohorts
                .iter()
                .zip(&source_zones)
                .map(|(c, zone)| c[0] * 50 - zone * 10)
                .collect();
            let weights_75: Vec<i64> = source_cohorts.iter().map(|c| c[1]).collect();
            let older_50 = crate::kernel::ratio::allocate_remaining(
                cohort_gain[0] - excluded_50 - zone_increase,
                &weights_50,
            );
            let older_75 =
                crate::kernel::ratio::allocate_remaining(cohort_gain[1] - excluded_75, &weights_75);
            let sources: Vec<i64> = older_50.iter().zip(&older_75).map(|(a, b)| a + b).collect();
            (sources.clone(), sources)
        } else {
            // Preserve the admitted uniform-cohort binding-cap source policy:
            // gain-proportional native allocation, later sources take residuals.
            // The existing refused mixed/partial-zone case remains unavailable.
            (
                crate::kernel::ratio::allocate_remaining(taxable, &source_gains),
                crate::kernel::ratio::allocate_remaining(rate_28, &source_gains),
            )
        };
        for (index, (entry, data)) in entries.iter().enumerate() {
            let own = taxable_sources[index];
            result.rows.push(row(entry, data, own));
            result.sources.push(QsbSource {
                id: entry.id,
                owner: entry.owner,
                taxable_gain: own,
                withholding: data.withholding,
                rate_28_gain: source_known.then_some(rate_sources[index]),
            });
        }
        result.excluded_gain += excluded;
        result.taxable_gain += taxable;
        result.rate_28_gain += rate_28;
        result.amt_preference += amt_preference;
        emit_lines(
            r,
            &mut (),
            [
                ("realized_gain", gain, msg!("EXPLAIN_QSBS_N_N_REALIZED_GAIN")),
                ("exclusion_limit", limit, msg!("EXPLAIN_QSBS_N_N_EXCLUSION_LIMIT")),
                ("excluded_gain", excluded, msg!("EXPLAIN_QSBS_N_N_EXCLUDED_GAIN")),
                ("zone_gain_through_2018", zone_gain, msg!("EXPLAIN_QSBS_N_N_ZONE_GAIN_THROUGH_N")),
                ("taxable_gain", taxable, msg!("EXPLAIN_QSBS_N_N_TAXABLE_GAIN")),
                ("rate_28_gain", rate_28, msg!("EXPLAIN_QSBS_N_N_RATE_N_GAIN")),
                ("amt_preference", amt_preference, msg!("EXPLAIN_QSBS_N_N_AMT_PREFERENCE")),
            ],
            |(suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("qsbs_{}_{}", entries[0].0.id, suffix),
                    LineValue::MoneyCents(amount),
                    &["income.qualified_small_business_sale"],
                    SOURCE,
                    explanation,
                ))
            },
        );
    }
    result
}

/// Qualified small business stock sold (section 1202)
/// Stock you bought at original issue from a C corporation and sold in 2026 after holding it more than five years. One for each issuing corporation, with every 2026 sale of its shares; the exclusion and its limit apply once per corporation.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the 2026 sales of one corporation's stock from a Form 1099-B or a paying agent's or closing statement exactly. Check the tax year against the supplied year; report a mismatch. Return the issuing corporation's legal name as printed as issuer. Return one lots row for each sale of that corporation's shares: box 1b date acquired as issued, box 1c date sold as sold, box 1d proceeds as proceeds and box 1e cost or other basis as basis; report a blank or illegible date or amount as an error, and never combine rows. If the document shows sales of more than one corporation's stock, report an error saying each corporation needs its own entry. Return ReportedToIrs when a Form 1099-B shows basis reported to the IRS (box 12 checked, or Form 8949 box D indicated), NotReported when it shows basis not reported (box 12 unchecked, or box E indicated), and NoForm1099B when the document is not a Form 1099-B. Read box 4 federal income tax withheld into withholding, returning zero when it is clearly blank. Do not read wash sales or empowerment-zone values, and do not decide whether the stock qualifies or the gain is excluded; the filer answers those. Copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct QsbsSale {
    /// Issuing corporation
    /// Its exact legal name, the same on every entry for it; not a broker, brand or ticker.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_size(min = 1)]
    pub issuer: String,
    /// 2026 sales
    /// ----
    /// Sale
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub lots: Vec<QsbStockLot>,
    /// Basis reported to the IRS
    #[libertas_llm_doc_field]
    #[libertas_default(NoForm1099B)]
    pub reporting: crate::BasisReporting,
    /// Federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Eligible gain already taken into account under section 1202 for this issuer before 2026
    /// Reconcile every prior return of the current taxpayer and, for a joint return, both spouses. Allocate gain taken into account on an earlier joint return equally to its spouses for later years. Enter zero only after checking the complete history. Do not enter prior sale proceeds or prior taxable gain here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_eligible_gain: i64,
    /// Qualified domestic C corporation and gross-assets test confirmed for every issue date
    /// The issuer was a domestic C corporation. Aggregate assets of the issuer, its predecessors and its controlled group did not exceed $50 million immediately before and after each original issue; use the statutory contribution valuation rules. No applies no section 1202 exclusion, so the whole gain is taxed.
    pub issuer_qualified: bool,
    /// Active qualified business confirmed throughout substantially all of each holding period
    /// At least 80% of asset value was used in a qualified active business. Exclude disallowed professional services, finance, investment, farming, mining and hospitality businesses; account for subsidiaries, working capital and permitted exceptions from issuer records. No applies no section 1202 exclusion, so the whole gain is taxed.
    pub active_business: bool,
    /// No disqualifying issuer redemption or other section 1202 exception
    /// Check the applicable shareholder and significant redemption windows around every issuance; confirm no special rule or election changes qualification, gain recognition or tax character. No applies no section 1202 exclusion, so the whole gain is taxed.
    pub no_disqualification: bool,
    /// Extra 60% empowerment-zone exclusion applies to one or more sold lots
    /// For shares acquired December 22, 2000 through February 17, 2009, confirm No only after checking the issuer's empowerment-zone status and gain through 2018. Choose Yes when the issuer qualified and the source valuation determines the eligible gain for each affected lot. Check the issuer's retained zone certification and valuation; an unavailable historical certification or valuation leaves the increase unclaimed with a notice. Leave absent when all lots are outside this issue-date interval.
    /// [NullText]
    /// No additional empowerment-zone exclusion is established
    /// [SetText]
    /// The additional empowerment-zone exclusion applies, answered as
    #[libertas_default($null)]
    // When an applicable issuer's historical zone certification or valuation is unavailable,
    // purchase/sale amounts cannot reconstruct it; WARN_QSBS_ZONE_ASSUMED_NO omits the increase.
    // Outside the statutory issue-date interval the absent answer remains not applicable.
    pub zone_60_exclusion: Option<bool>,
    /// Complete issuer history and all current-year lots confirmed
    /// The prior-gain figure and every 2026 disposition of this issuer's qualifying shares on the filing unit are included once. A joint return needs its spouses' separate and joint prior-return history. Your prior returns' section 1202 exclusions for this issuer show it. No applies no section 1202 exclusion, so the whole gain is taxed, with a notice.
    pub complete_history: bool,
    /// Bought other than for cash at original issue, or a gift, inheritance, exchange, pass-through ownership, installment sale, rollover or another special treatment
    /// Yes leaves this sale out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}

impl QsbsSale {
    // The server-checked relationship the lot dates must meet (stock issued
    // from August 11, 1993 through July 4, 2025, sold in 2026 more than five
    // years later), by its key.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.lots
            .iter()
            .enumerate()
            .find_map(|(i, lot)| lot.issue().map(|issue| issue.in_list("lots", i)))
    }
    // The stored issuer sale this page writes.
    pub(crate) fn record(&self) -> QsbIssuerSale {
        QsbIssuerSale {
            issuer: self.issuer.clone(),
            prior_eligible_gain: self.prior_eligible_gain,
            special: self.special,
            issuer_qualified: self.issuer_qualified,
            active_business: self.active_business,
            no_disqualification: self.no_disqualification,
            zone_60_exclusion: self.zone_60_exclusion,
            complete_history: self.complete_history,
            lots: self.lots.clone(),
            reporting: self.reporting,
            withholding: self.withholding,
        }
    }
    // The reverse of record.
    pub(crate) fn from_record(v: &QsbIssuerSale) -> Self {
        Self {
            issuer: v.issuer.clone(),
            lots: v.lots.clone(),
            reporting: v.reporting,
            withholding: v.withholding,
            prior_eligible_gain: v.prior_eligible_gain,
            issuer_qualified: v.issuer_qualified,
            active_business: v.active_business,
            no_disqualification: v.no_disqualification,
            zone_60_exclusion: v.zone_60_exclusion,
            complete_history: v.complete_history,
            special: v.special,
        }
    }
}
