use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::line_emission::emit as emit_lines;
use crate::qsbs::{QsbIssuerSale, QsbStockLot};
use crate::year_constants as yc;
use crate::{FederalDraft, FederalIncome, FederalResult};
use alloc::{collections::BTreeMap, string::String, vec::Vec};

const SOURCE: &str = "https://www.irs.gov/instructions/i1040sd";

impl QsbStockLot {
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

impl QsbIssuerSale {
    // The exclusion applies only with confirmed issuer qualification and a
    // complete history; otherwise the whole gain is taxed.
    fn excluded(&self) -> bool {
        self.complete_history == Some(true)
            && self.prior_eligible_gain.is_some()
            && [self.issuer_qualified, self.active_business, self.no_disqualification]
                .iter()
                .all(|answer| *answer == Some(true))
    }

    // The zone increase is omitted when issuer source records are unavailable:
    // an absent applicable answer, or Yes with no source zone gain entered.
    // Only a Yes uses the entered zone gain.
    fn zone_assumed_no(&self) -> bool {
        match self.zone_60_exclusion.and_then(crate::QsbsZoneCertification::value) {
            Some(false) => false,
            Some(true) => {
                self.lots
                    .iter()
                    .filter_map(|lot| {
                        lot.zone_gain_through_2018.and_then(crate::QsbsZoneValuation::gain)
                    })
                    .sum::<i64>()
                    == 0
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
                if data.complete_history != Some(true) {
                    data.note("WARN_QSBS_EXCLUSION_LEFT_OUT", r);
                }
                let gain: i64 = data.lots.iter().map(|lot| lot.proceeds - lot.basis).sum();
                result.taxable_gain += gain;
                result.rows.push(row(entry, data, gain));
                result.sources.push(QsbSource {
                    id: entry.id,
                    owner: entry.owner.owned(),
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
            by_issuer.entry(data.issuer_key()).or_default().push((entry, data));
        }
    }
    for entries in by_issuer.values() {
        // Every entry admitted here passed excluded(), requiring an explicit prior gain.
        let prior =
            entries[0].1.prior_eligible_gain.expect("excluded() requires answered issuer history");
        crate::require_localized!(
            r,
            entries.iter().all(|(_, data)| data.prior_eligible_gain == Some(prior)),
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
                if data.zone_60_exclusion == Some(crate::QsbsZoneCertification::Yes) {
                    let zone = lot
                        .zone_gain_through_2018
                        .and_then(crate::QsbsZoneValuation::gain)
                        .unwrap_or(0);
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
        let source_known = entries.iter().all(|(_, data)| data.prior_eligible_gain == Some(prior))
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
                owner: entry.owner.owned(),
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
