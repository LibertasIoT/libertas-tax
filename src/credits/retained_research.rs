pub use crate::retained_research_model::{
    ResearchCapitalPart, ResearchCapitalParts, ResearchLossPart, ResearchLossParts,
};
use crate::validation::ValidationIssue;
// Retained source identity belongs to the existing loss owner. These portions
// never change its deduction, absorption, carryover or AMT calculation.
use crate::{CapitalLossOrigin, InvestmentCapitalParts, PriorAmtCapitalReturn, PriorReturnOwners};
use alloc::{collections::BTreeMap, vec::Vec};
use libertas_macros::*;

/// Retained operating-loss portions belonging to current businesses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ResearchLossPartsPage {
    /// Identified business portions
    Parts {
        /// Portions of the existing balance; the remainder belongs to other operating businesses
        #[libertas_size(min = 1)]
        /// ----
        /// Business portion
        sources: Vec<ResearchLossPartPage>,
    },
}
/// One business's portion of this retained loss
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ResearchLossPartPage {
    /// Actual business on this return that made the loss
    #[libertas_enum_source("$.research_businesses")]
    pub source: i64,
    /// Part of this year's unused regular loss
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
    pub amount: i64,
}
/// Regular loss from a retained return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("RetainedLossSourceOptions")]
pub struct NolYearPage {
    /// Calendar loss year
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 1900, max = 2025)]
    pub year: u32,
    /// Unused regular-tax loss entering 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_calculation_request("RetainedLossSourceOptions")]
    pub balance: i64,
    /// Each spouse's actual surviving regular loss
    /// Only for a Form 8379 claim. Use the retained source records for this year after all earlier use. This does not change the joint deduction or AGI.
    /// [NullText]
    /// The surviving spouse shares have not been established
    /// [SetText]
    /// The actual surviving spouse shares are
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_injured_source_facts")]
    pub injured_owners: Option<crate::InjuredNolOwners>,
    /// Actual PTP excess-business-loss character, from retained loss-tracking records
    #[libertas_default(Unavailable)]
    pub ptp_ebl: crate::PtpEblNol,
    /// Available retained-loss source question
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.retained_research_choices")]
    #[libertas_output_only]
    pub allowed_research_parts: Vec<i32>,
    /// Current businesses that made this retained loss
    /// Use the actual loss-source records. The remaining balance belongs to other operating businesses.
    /// [NullText]
    /// None of this retained loss belongs to a current research business
    /// [SetText]
    /// These portions belong to current businesses
    #[libertas_constrained_by("^.allowed_research_parts")]
    pub research_parts: Option<ResearchLossPartsPage>,
}
impl From<NolYearPage> for crate::NolYear {
    fn from(v: NolYearPage) -> Self {
        Self {
            year: v.year,
            balance: v.balance,
            injured_owners: v.injured_owners,
            ptp_ebl: v.ptp_ebl,
            research_parts: v.research_parts.map(|ResearchLossPartsPage::Parts { sources }| {
                ResearchLossParts::Parts {
                    sources: sources
                        .into_iter()
                        .map(|p| ResearchLossPart { source: p.source, amount: p.amount })
                        .collect(),
                }
            }),
        }
    }
}
impl From<crate::NolYear> for NolYearPage {
    fn from(v: crate::NolYear) -> Self {
        Self {
            year: v.year,
            balance: v.balance,
            injured_owners: v.injured_owners,
            ptp_ebl: v.ptp_ebl,
            allowed_research_parts: Vec::new(),
            research_parts: v.research_parts.map(|ResearchLossParts::Parts { sources }| {
                ResearchLossPartsPage::Parts {
                    sources: sources
                        .into_iter()
                        .map(|p| ResearchLossPartPage { source: p.source, amount: p.amount })
                        .collect(),
                }
            }),
        }
    }
}
pub(crate) fn choices(balance: i64, available: &[i32]) -> Vec<i32> {
    if balance > 0 && available.contains(&0) { alloc::vec![0] } else { Vec::new() }
}
pub(crate) fn available(d: &crate::FederalDraft) -> Vec<i32> {
    let current = d.income.iter().any(|entry| match &entry.income {
        crate::FederalIncome::Partnership { data } => {
            data.credits.iter().any(|c| c.credit().research())
        }
        crate::FederalIncome::Scorp { data } => {
            let data = &crate::ScorpK1::from(data);
            data.credits.iter().any(|c| c.credit().research())
        }
        crate::FederalIncome::PartnershipActivities { data } => {
            data.entity.credits.iter().any(|c| c.credit().research())
        }
        crate::FederalIncome::ScorpActivities { data } => {
            data.entity.credits.iter().any(|c| c.credit().research())
        }
        _ => false,
    });
    let catalog = crate::business_credit_page::research_businesses(d);
    let history = d.credits.as_ref().and_then(|c| c.business_credits.as_ref()).is_some_and(|h| {
        h.carryforwards.iter().any(|c| {
            c.credit.research()
                && c.research_source.is_some_and(|id| catalog.iter().any(|source| source.id == id))
        }) || h.credits.iter().any(|c| {
            c.credit.research()
                && crate::general_business_credit::resolve_source(d, &c.source).is_some()
        })
    });
    if current || history { alloc::vec![0] } else { Vec::new() }
}

pub(crate) fn loss_issue(
    v: &crate::NolYear,
    catalog: Option<&[crate::RecordChoice]>,
) -> Option<ValidationIssue> {
    let Some(ResearchLossParts::Parts { sources }) = &v.research_parts else {
        return None;
    };
    if sources.is_empty() || sources.len() > 100 {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_PARTS_ABOVE_BALANCE")
                .at_field("sources")
                .in_field("research_parts"),
        );
    }
    if let Some((i, _)) =
        sources.iter().enumerate().find(|(_, p)| !(100..=100000000000).contains(&p.amount))
    {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_PARTS_ABOVE_BALANCE")
                .at_field("amount")
                .in_list("sources", i)
                .in_field("research_parts"),
        );
    }
    if sources.iter().map(|p| p.amount).sum::<i64>() > v.balance {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_PARTS_ABOVE_BALANCE")
                .at_field("sources")
                .in_field("research_parts"),
        );
    }
    if let Some((i, _)) = sources
        .iter()
        .enumerate()
        .find(|(i, p)| sources[..*i].iter().any(|old| old.source == p.source))
    {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_SOURCE_TWICE")
                .at_field("source")
                .in_list("sources", i)
                .in_field("research_parts"),
        );
    }
    if let Some(c) = catalog
        && let Some((i, _)) =
            sources.iter().enumerate().find(|(_, p)| !c.iter().any(|r| r.id == p.source))
    {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_SOURCE_UNAVAILABLE")
                .at_field("source")
                .in_list("sources", i)
                .in_field("research_parts"),
        );
    }
    None
}
/// Canonical retained source that owns a refusal. Indices refer to accepted
/// screening records, never directly to the caller's submitted document list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RetainedIssueOrigin {
    Other,
    Nol(usize),
    Capital(usize),
}
pub(crate) fn issue(d: &crate::FederalDraft) -> Option<(RetainedIssueOrigin, ValidationIssue)> {
    if let Some(key) = crate::retained_charity::rule(d) {
        return Some((RetainedIssueOrigin::Other, ValidationIssue::new(key)));
    }
    let Some(screen) = &d.screening else {
        return None;
    };
    let catalog = crate::business_credit_page::research_businesses(d);
    let available = available(d);
    if let crate::NolHistory::Records { years, .. } = &screen.nol {
        for (index, year) in years.iter().enumerate() {
            if let Some(issue) = loss_issue(year, Some(&catalog)) {
                return Some((RetainedIssueOrigin::Nol(index), issue));
            }
            if year.research_parts.is_some() && choices(year.balance, &available).is_empty() {
                return Some((
                    RetainedIssueOrigin::Nol(index),
                    ValidationIssue::new("RETAINED_RESEARCH_SOURCE_UNAVAILABLE")
                        .at_field("research_parts"),
                ));
            }
        }
    }
    for (index, record) in screen.capital_history.iter().enumerate() {
        if let Some(issue) = capital_issue(record, &catalog) {
            return Some((RetainedIssueOrigin::Capital(index), issue));
        }
        if record.research_parts.is_some() && available.is_empty() {
            return Some((
                RetainedIssueOrigin::Capital(index),
                ValidationIssue::new("RETAINED_RESEARCH_SOURCE_UNAVAILABLE")
                    .at_field("research_parts"),
            ));
        }
    }
    None
}
pub(crate) fn rule(d: &crate::FederalDraft) -> Option<&'static str> {
    issue(d).map(|(_, issue)| issue.key)
}

/// Retained regular capital-loss portions belonging to actual businesses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ResearchCapitalPartsPage {
    /// Identified business portions
    Parts {
        /// Portions of the calculated carryover; the remainder is unrelated to current research businesses
        #[libertas_size(min = 1)]
        /// ----
        /// Business portion
        sources: Vec<ResearchCapitalPartPage>,
    },
}
/// One business's portion of the calculated regular carryover
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ResearchCapitalPartPage {
    /// Actual business on this return that made the loss
    #[libertas_enum_source("$.research_businesses")]
    pub source: i64,
    /// Regular short-term portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub short: i64,
    /// Regular long-term portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub long: i64,
}
/// 2025 return with a regular or AMT capital loss
/// Copy the filed return's amounts. The App calculates the carryovers.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("RetainedCapitalSourceOptions")]
pub struct PriorCapitalReturnPage {
    /// Who filed this 2025 return
    pub owners: PriorReturnOwners,
    /// Filing status on that return
    pub status: crate::FederalStatus,
    /// 2025 Form 1040 line 11b adjusted gross income
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_calculation_request("RetainedCapitalSourceOptions")]
    pub agi: i64,
    /// 2025 Form 1040 line 14 total deductions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_calculation_request("RetainedCapitalSourceOptions")]
    pub deductions: i64,
    /// 2025 Schedule 1-A line 37 enhanced deduction for seniors
    /// Enter the amount already included in Form 1040 line 14 above, or zero if none. Do not include the age/blindness addition to the standard deduction. The enhanced deduction is added back only when measuring capital-loss use, not when calculating tax.
    #[libertas_money("USD", 2)]
    // Section 151 tax structure: at most two $6,000 enhanced senior deductions on the retained 2025 return.
    #[libertas_number(min = 0, max = 1200000)]
    #[libertas_calculation_request("RetainedCapitalSourceOptions")]
    pub senior_deduction: i64,
    /// 2025 Schedule D line 7 net short-term gain or loss
    /// Enter a loss as negative.
    #[libertas_money("USD", 2)]
    // Temporary Unit 47 exclusion: capital_caps still consumes the unchecked short + long sum; widen its receiver before removing this bound.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_calculation_request("RetainedCapitalSourceOptions")]
    pub short: i64,
    /// 2025 Schedule D line 15 net long-term gain or loss
    /// Enter a loss as negative.
    #[libertas_money("USD", 2)]
    // Temporary Unit 47 exclusion: capital_caps still consumes the unchecked short + long sum; widen its receiver before removing this bound.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_calculation_request("RetainedCapitalSourceOptions")]
    pub long: i64,
    /// Loss deducted on 2025 Schedule D line 21, entered as positive
    #[libertas_money("USD", 2)]
    // Section 1211(b) tax structure: the annual net capital-loss deduction cannot exceed $3,000.
    #[libertas_number(min = 0, max = 300000)]
    #[libertas_calculation_request("RetainedCapitalSourceOptions")]
    pub deducted_loss: i64,
    /// Part of the loss carried to 2026 that came from section 1256 contracts
    /// Enter the part of your 2025 capital loss carryover that came from a net section 1256 contracts loss (Form 6781, Publication 550 "Net section 1256 contracts loss"), as a positive amount, or zero when none of it did. A 2026 carryback election can then count it as a 2026 section 1256 loss.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_1256: i64,
    /// Separate 2025 AMT capital-loss worksheet
    /// Supply retained AMT worksheet amounts when its carryover differs from regular tax, including an AMT-only loss. Leave absent only when the records establish equal carryovers, not when AMT records are unavailable.
    /// [NullText]
    /// AMT capital-loss carryovers equal the regular carryovers
    /// [SetText]
    /// The different AMT capital-loss worksheet is
    #[libertas_default($null)]
    pub amt: Option<PriorAmtCapitalReturn>,
    /// Retained loss balances by original loss year
    /// Optional supporting history from reconciled loss worksheets, after 2025 use. Leave empty when unavailable. Include every surviving year if supplied; totals must match the carryovers calculated from this return. Do not invent years from the aggregate Schedule D amounts. This records provenance; general canceled-debt reductions still require further worksheets.
    pub origins: Vec<CapitalLossOrigin>,
    /// These losses belong entirely to the people filing this return
    /// No allocation to a former spouse, canceled-debt basis reduction, unresolved special capital-gain category or AMT adjustment applies. Any different AMT carryover is supplied in the separate worksheet above. The prior Schedule D and any divorce or estate documents show it. No leaves this prior-year capital loss carryover out of the estimate with a notice; that situation is not supported.
    #[libertas_calculation_request("RetainedCapitalSourceOptions")]
    pub ordinary_history: bool,
    /// Investment-property portions of the calculated carryovers
    /// Use retained loss-source records. Absence means attribution is unavailable, not that no loss is investment-related. These portions do not change the ordinary Schedule D carryover.
    /// [NullText]
    /// Investment-property loss attribution is unavailable
    /// [SetText]
    /// Investment-property loss portions are
    #[libertas_default($null)]
    pub investment_parts: Option<InvestmentCapitalParts>,
    /// Prior Form 8960 excluded capital-loss difference
    /// Positive amount from line 2(i), column A, of the retained 2025 Form 8960 net-gains worksheet. Enter zero when that worksheet establishes none. Leave absent only when no regular loss carries forward. Section 163(d) investment-property attribution above does not establish the separate section 1411 amount. This is not the ordinary Schedule D carryover; the App calculates that from the filed return.
    /// [NullText]
    /// No regular capital loss carries forward
    /// [SetText]
    /// The prior Form 8960 excluded capital-loss difference is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub niit_difference: Option<i64>,
    /// Each spouse’s surviving capital-loss amounts
    /// Only for a Form 8379 claim. Use actual source records; this never changes the joint calculation.
    /// [NullText]
    /// The surviving spouse shares have not been established
    /// [SetText]
    /// The actual surviving spouse shares are
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_injured_source_facts")]
    pub injured_owners: Option<crate::InjuredCapitalLossOwners>,
    /// Actual owners of the retained Form 8960 excluded-loss difference
    /// Only for a Form 8379 claim. This is separate from regular and AMT capital-loss shares and from Form 4952 investment-property portions. Use each spouse's retained section 1411 records.
    /// [NullText]
    /// Owners of the retained Form 8960 difference have not been established
    /// [SetText]
    /// The retained Form 8960 difference belongs to
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_injured_retained_owners")]
    pub injured_niit_owners: Option<crate::InjuredRetainedOwners>,
    /// Available regular-capital source question
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.retained_research_choices")]
    #[libertas_output_only]
    pub allowed_research_parts: Vec<i32>,
    /// Current businesses that made this retained regular capital loss
    /// Identify the actual loss sources. These portions do not change Schedule D, AMT or investment-interest attribution.
    /// [NullText]
    /// None of this regular capital loss belongs to a current research business
    /// [SetText]
    /// These regular loss portions belong to current businesses
    #[libertas_constrained_by("^.allowed_research_parts")]
    pub research_parts: Option<ResearchCapitalPartsPage>,
}
impl From<PriorCapitalReturnPage> for crate::PriorCapitalReturn {
    fn from(v: PriorCapitalReturnPage) -> Self {
        Self {
            owners: v.owners,
            status: v.status,
            agi: v.agi,
            deductions: v.deductions,
            senior_deduction: v.senior_deduction,
            short: v.short,
            long: v.long,
            deducted_loss: v.deducted_loss,
            section_1256: v.section_1256,
            amt: v.amt,
            origins: v.origins,
            ordinary_history: v.ordinary_history,
            investment_parts: v.investment_parts,
            niit_difference: v.niit_difference,
            injured_owners: v.injured_owners,
            injured_niit_owners: v.injured_niit_owners,
            research_parts: v.research_parts.map(|ResearchCapitalPartsPage::Parts { sources }| {
                ResearchCapitalParts::Parts {
                    sources: sources
                        .into_iter()
                        .map(|p| ResearchCapitalPart {
                            source: p.source,
                            short: p.short,
                            long: p.long,
                        })
                        .collect(),
                }
            }),
        }
    }
}
impl From<crate::PriorCapitalReturn> for PriorCapitalReturnPage {
    fn from(v: crate::PriorCapitalReturn) -> Self {
        Self {
            owners: v.owners,
            status: v.status,
            agi: v.agi,
            deductions: v.deductions,
            senior_deduction: v.senior_deduction,
            short: v.short,
            long: v.long,
            deducted_loss: v.deducted_loss,
            section_1256: v.section_1256,
            amt: v.amt,
            origins: v.origins,
            ordinary_history: v.ordinary_history,
            investment_parts: v.investment_parts,
            niit_difference: v.niit_difference,
            injured_owners: v.injured_owners,
            injured_niit_owners: v.injured_niit_owners,
            allowed_research_parts: Vec::new(),
            research_parts: v.research_parts.map(|ResearchCapitalParts::Parts { sources }| {
                ResearchCapitalPartsPage::Parts {
                    sources: sources
                        .into_iter()
                        .map(|p| ResearchCapitalPartPage {
                            source: p.source,
                            short: p.short,
                            long: p.long,
                        })
                        .collect(),
                }
            }),
        }
    }
}

pub(crate) fn capital_issue(
    v: &crate::PriorCapitalReturn,
    catalog: &[crate::RecordChoice],
) -> Option<ValidationIssue> {
    let Some(ResearchCapitalParts::Parts { sources }) = &v.research_parts else {
        return None;
    };
    let cap = crate::extended_rules::capital_caps(v);
    if !v.ordinary_history || cap.short + cap.long == 0 {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_SOURCE_UNAVAILABLE").at_field("research_parts"),
        );
    }
    if sources.is_empty() || sources.len() > 100 {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_PARTS_ABOVE_BALANCE")
                .at_field("sources")
                .in_field("research_parts"),
        );
    }
    for (i, p) in sources.iter().enumerate() {
        let field = if !(0..=100000000000).contains(&p.short) {
            Some("short")
        } else if !(0..=100000000000).contains(&p.long) || p.short + p.long == 0 {
            Some("long")
        } else {
            None
        };
        if let Some(field) = field {
            return Some(
                ValidationIssue::new("RETAINED_RESEARCH_PARTS_ABOVE_BALANCE")
                    .at_field(field)
                    .in_list("sources", i)
                    .in_field("research_parts"),
            );
        }
    }
    if sources.iter().map(|p| p.short).sum::<i64>() > cap.short
        || sources.iter().map(|p| p.long).sum::<i64>() > cap.long
    {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_PARTS_ABOVE_BALANCE")
                .at_field("sources")
                .in_field("research_parts"),
        );
    }
    if let Some((i, _)) = sources
        .iter()
        .enumerate()
        .find(|(i, p)| sources[..*i].iter().any(|old| old.source == p.source))
    {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_SOURCE_TWICE")
                .at_field("source")
                .in_list("sources", i)
                .in_field("research_parts"),
        );
    }
    if let Some((i, _)) =
        sources.iter().enumerate().find(|(_, p)| !catalog.iter().any(|r| r.id == p.source))
    {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_SOURCE_UNAVAILABLE")
                .at_field("source")
                .in_list("sources", i)
                .in_field("research_parts"),
        );
    }
    None
}
// Opening amounts come from the existing Schedule D owner. Allocate its whole
// dollars across source cents, preserving each term's balance and stable IDs.
pub(crate) fn capital_losses(d: &crate::FederalDraft) -> BTreeMap<i64, i64> {
    let mut result = BTreeMap::new();
    let Some(screen) = &d.screening else {
        return result;
    };
    for record in screen.capital_history.iter().filter(|v| v.ordinary_history) {
        let Some(ResearchCapitalParts::Parts { sources }) = &record.research_parts else {
            continue;
        };
        let cap = crate::extended_rules::capital_caps(record);
        for (term, opening) in [cap.short, cap.long].into_iter().enumerate() {
            let mut claims: BTreeMap<Option<i64>, i64> = sources
                .iter()
                .map(|p| (Some(p.source), if term == 0 { p.short } else { p.long }))
                .collect();
            claims.insert(None, opening - claims.values().sum::<i64>());
            let weights: Vec<_> = claims.values().copied().collect();
            for (id, part) in claims.into_keys().zip(crate::kernel::ratio::allocate_remaining(
                crate::kernel::ratio::dollars(opening),
                &weights,
            )) {
                if let Some(id) = id {
                    *result.entry(id).or_default() += part;
                }
            }
        }
    }
    result
}
