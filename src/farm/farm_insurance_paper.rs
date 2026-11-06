//! Printed crop-insurance amounts remain source-owned; farms retain only missing facts.
use crate::{CropDeferral, CropPayment, FarmInsurance, FederalDraft, FederalIncome};
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use libertas_macros::*;

/// Crop-insurance income timing
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum CropPaperTiming {
    /// Include current proceeds in this year's income
    Include,
    /// Elect eligible physical-loss proceeds into the following year
    /// The cash-method election covers every eligible payment of this farming business. Retain the required election statement and source records.
    Defer,
}

/// Missing facts for related crop-insurance statements
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct CropInsuranceFacts {
    /// Accepted business income-timing answer
    pub timing: Option<CropPaperTiming>,
    /// Accepted physical-loss classifications by printed component identity
    pub papers: Vec<CropPaperFacts>,
    /// Normal sales practice for identified crops
    pub practice: Vec<crate::FarmCropPractice>,
}

/// Printed crop-insurance classification
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CropPaperFacts {
    /// Accepted crop-insurance component identity
    pub source: i64,
    /// Character of this statement's proceeds
    pub classification: CropPaperClass,
}

/// Character of the printed proceeds
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum CropPaperClass {
    /// No qualifying physical crop damage
    /// Price-only, rainfall-index or livestock-revenue payments without qualifying physical crop loss cannot be deferred.
    Nondeferrable,
    /// All proceeds are for one physically damaged crop
    OneCrop {
        /// Damaged crop
        #[libertas_size(min = 1)]
        crop: String,
        /// Year the crop was damaged
        #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
        year: u32,
    },
    /// All proceeds are for several physically damaged crops
    SeveralCrops {
        /// Physical-loss portions from retained records
        #[libertas_size(min = 2)]
        portions: Vec<crate::farm_page::CropLoss>,
    },
    /// Physical crop damage and nondeferrable proceeds
    /// The remaining printed proceeds are nondeferrable. Enter only physical-loss portions from retained records, never the printed total again.
    Mixed {
        /// Physical-loss portions
        #[libertas_size(min = 1)]
        portions: Vec<crate::farm_page::CropLoss>,
    },
}

impl CropPaperClass {
    pub(crate) fn payments(&self, amount: i64) -> Option<Vec<CropPayment>> {
        let physical = |crop: &str, year, amount| {
            (!crop.trim().is_empty() && amount > 0).then(|| CropPayment::Physical {
                crop: crop.into(),
                year,
                amount,
            })
        };
        match self {
            Self::Nondeferrable => Some(alloc::vec![CropPayment::Other { amount }]),
            Self::OneCrop { crop, year } => Some(alloc::vec![physical(crop, *year, amount)?]),
            Self::SeveralCrops { portions } | Self::Mixed { portions } => {
                let mut payments = Vec::new();
                let mut total = 0i64;
                for portion in portions {
                    total = total.checked_add(portion.amount)?;
                    payments.push(physical(&portion.crop, portion.year, portion.amount)?);
                }
                if matches!(self, Self::SeveralCrops { .. }) {
                    if total != amount {
                        return None;
                    }
                } else {
                    if total >= amount {
                        return None;
                    }
                    payments.push(CropPayment::Other { amount: amount - total });
                }
                Some(payments)
            }
        }
    }
}

pub(crate) fn farm_mut(entry: &mut crate::FederalIncomeEntry) -> Option<&mut crate::FarmIncome> {
    match &mut entry.income {
        FederalIncome::Business { data } => match &mut data.activity {
            crate::BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        FederalIncome::FarmRental { data } => Some(&mut data.farm),
        _ => None,
    }
}

pub(crate) fn papers(
    d: &FederalDraft,
    activity: i64,
    claims: &BTreeMap<i64, i64>,
) -> Vec<crate::related_forms::CenterDocument> {
    let Some(target) = crate::farm::sources(d).find(|s| s.entry.id == activity) else {
        return Vec::new();
    };
    crate::related_forms::catalog(d)
        .into_iter()
        .filter(|component| {
            component.kind == crate::related_forms::DocumentKind::Form1099MiscCropInsurance
                && claims.get(&component.id) == Some(&activity)
                && component.eligible.contains(&activity)
                && d.income
                    .iter()
                    .any(|e| e.id == component.form_id && e.owner == target.entry.owner)
        })
        .collect()
}

/// Resolve whole-activity insurance without changing accepted data or printed amounts.
pub(crate) fn resolved(
    d: &FederalDraft,
    activity: i64,
    claims: &BTreeMap<i64, i64>,
) -> Option<FarmInsurance> {
    let source = crate::farm::sources(d).find(|s| s.entry.id == activity)?;
    let paper = papers(d, activity, claims);
    if paper.is_empty() && source.farm.insurance_papers.timing.is_none() {
        return Some(source.farm.insurance.kernel());
    }
    let facts = &source.farm.insurance_papers;
    let (mut current, previous_deferred, withholding) = match &source.farm.insurance {
        FarmInsuranceRecords::None => (Vec::new(), 0, 0),
        FarmInsuranceRecords::Payments { current, previous_deferred, withholding } => {
            (current.clone(), *previous_deferred, *withholding)
        }
    };
    let mut complete = true;
    for component in paper {
        let amount = component.summary.amount.unwrap_or(0);
        let classified = facts
            .papers
            .iter()
            .find(|p| p.source == component.id)
            .and_then(|p| p.classification.payments(amount));
        if let Some(classified) = classified {
            current.extend(classified);
        } else {
            complete = false;
            current.push(CropPayment::Other { amount });
        }
    }
    let election = if matches!(facts.timing, Some(CropPaperTiming::Defer)) && complete {
        CropDeferral::Defer { practice: facts.practice.clone() }
    } else {
        CropDeferral::Include
    };
    Some(FarmInsurance::Payments { current, previous_deferred, election, withholding })
}

pub(crate) fn income_delta(d: &FederalDraft, activity: i64, claims: &BTreeMap<i64, i64>) -> i64 {
    let Some(source) = crate::farm::sources(d).find(|s| s.entry.id == activity) else {
        return 0;
    };
    let cash = matches!(source.farm.accounting, crate::FarmAccounting::Cash { .. });
    resolved(d, activity, claims)
        .map_or(0, |insurance| insurance.income(cash) - source.farm.insurance.income(cash))
}

pub(crate) fn report_value(
    d: &FederalDraft,
    activity: i64,
    claims: &BTreeMap<i64, i64>,
) -> Option<FarmInsurance> {
    use crate::community::Divide;
    let source = crate::farm::sources(d).find(|s| s.entry.id == activity)?;
    let value = resolved(d, activity, claims)?;
    Some(if source.rental {
        d.setup
            .as_ref()
            .and_then(|setup| crate::community::entry_share(setup, source.entry))
            .map_or_else(|| value.clone(), |share| value.divided(share))
    } else {
        value
    })
}

/// Facts affected by an ordinary source correction follow the accepted source graph.
pub(crate) fn follow_sources(before: &FederalDraft, draft: &mut FederalDraft) {
    let old_claims = crate::related_forms::effective_claimants(before);
    let claims = crate::related_forms::effective_claimants(draft);
    let updates: Vec<_> = crate::farm::sources(draft)
        .map(|source| {
            let current = papers(draft, source.entry.id, &claims);
            let old = papers(before, source.entry.id, &old_claims);
            let owner_changed = before
                .income
                .iter()
                .find(|e| e.id == source.entry.id)
                .is_some_and(|e| e.owner != source.entry.owner);
            let has_papers = !current.is_empty();
            let retained: Vec<_> = source
                .farm
                .insurance_papers
                .papers
                .iter()
                .filter(|facts| {
                    !owner_changed
                        && current.iter().any(|p| {
                            p.id == facts.source
                                && old.iter().any(|prior| prior.id == p.id)
                                && before
                                    .income
                                    .iter()
                                    .find(|e| e.id == p.form_id)
                                    .zip(draft.income.iter().find(|e| e.id == p.form_id))
                                    .is_some_and(|(old, new)| {
                                        if old.owner != new.owner {
                                            return false;
                                        }
                                        let (
                                            FederalIncome::Misc1099 { data: old },
                                            FederalIncome::Misc1099 { data: new },
                                        ) = (&old.income, &new.income)
                                        else {
                                            return false;
                                        };
                                        old.form.payer == new.form.payer
                                            && old.form.recipient == new.form.recipient
                                            && (old.form.crop_insurance == new.form.crop_insurance
                                                || matches!(
                                                    facts.classification,
                                                    CropPaperClass::OneCrop { .. }
                                                        | CropPaperClass::Nondeferrable
                                                ))
                                    })
                        })
                })
                .cloned()
                .collect();
            (source.entry.id, retained, has_papers, owner_changed)
        })
        .collect();
    for (id, papers, has_papers, owner_changed) in updates {
        if let Some(farm) = draft.income.iter_mut().find(|e| e.id == id).and_then(farm_mut) {
            farm.insurance_papers.papers = papers;
            if owner_changed || (!has_papers && farm.insurance.current_total() == 0) {
                farm.insurance_papers = CropInsuranceFacts::default();
            }
        }
    }
    let current_crops: Vec<_> = crate::farm::sources(draft)
        .map(|source| (source.entry.id, crop_names(draft, source.entry.id, &claims)))
        .collect();
    for (id, crops) in current_crops {
        if let Some(farm) = draft.income.iter_mut().find(|entry| entry.id == id).and_then(farm_mut)
        {
            farm.insurance_papers.practice.retain(|practice| crops.contains(&practice.crop));
        }
    }
}

/// Crop-insurance statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_link("OpenCropPaperFacts")]
pub struct CropPaperLink {
    /// Accepted statement component
    #[libertas_hidden]
    pub source: i64,
    /// Printed payment
    #[libertas_read_only]
    pub summary: crate::RelatedFormSummary,
}

/// Crop whose normal sales practice supports the election
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_link("OpenCropPractice")]
pub struct CropPracticeLink {
    /// Damaged crop
    #[libertas_read_only]
    pub crop: String,
}

/// Accepted timing answer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AcceptedCropTiming {
    /// Timing
    pub timing: CropPaperTiming,
}

/// Accepted statement classification
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AcceptedCropClassification {
    /// Proceeds classification
    pub classification: CropPaperClass,
}

pub(crate) fn crop_names(
    d: &FederalDraft,
    activity: i64,
    claims: &BTreeMap<i64, i64>,
) -> Vec<String> {
    let mut crops = Vec::new();
    if let Some(FarmInsurance::Payments { current, .. }) = resolved(d, activity, claims) {
        for payment in current {
            if let CropPayment::Physical { crop, year, .. } = payment
                && year == crate::year_constants::TAX_YEAR
                && !crops.contains(&crop)
            {
                crops.push(crop);
            }
        }
    }
    crops
}

pub(crate) fn missing(d: &FederalDraft, activity: i64, claims: &BTreeMap<i64, i64>) -> bool {
    let Some(source) = crate::farm::sources(d).find(|s| s.entry.id == activity) else {
        return false;
    };
    let papers = papers(d, activity, claims);
    if (papers.is_empty() && source.farm.insurance.current_total() == 0)
        || !matches!(source.farm.accounting, crate::FarmAccounting::Cash { .. })
    {
        return false;
    }
    match source.farm.insurance_papers.timing {
        None => true,
        Some(CropPaperTiming::Include) => false,
        Some(CropPaperTiming::Defer) => {
            papers.iter().any(|p| {
                !source.farm.insurance_papers.papers.iter().any(|f| {
                    f.source == p.id
                        && f.classification.payments(p.summary.amount.unwrap_or(0)).is_some()
                })
            }) || resolved(d, activity, claims)
                .is_some_and(|insurance| insurance.source_issue().is_some())
        }
    }
}

/// Additional crop-insurance and disaster payments absent from related statements
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FarmInsuranceRecords {
    /// No additional payments or prior-year deferral
    None,
    /// Additional payment records
    Payments {
        /// Current payments absent from related printed statements
        current: Vec<CropPayment>,
        /// Proceeds deferred into this year on the prior return
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        previous_deferred: i64,
        /// Federal withholding absent from Documents
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        withholding: i64,
    },
}
impl FarmInsuranceRecords {
    pub(crate) fn kernel(&self) -> FarmInsurance {
        match self {
            Self::None => FarmInsurance::None,
            Self::Payments { current, previous_deferred, withholding } => FarmInsurance::Payments {
                current: current.clone(),
                previous_deferred: *previous_deferred,
                withholding: *withholding,
                election: CropDeferral::Include,
            },
        }
    }
    pub(crate) fn income(&self, cash: bool) -> i64 {
        self.kernel().income(cash)
    }
    pub(crate) fn current_total(&self) -> i64 {
        self.kernel().current_total()
    }
    pub(crate) fn valid(&self) -> bool {
        self.kernel().valid()
    }
    pub(crate) fn source_issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.kernel().source_issue()
    }
    pub(crate) fn withholding(&self) -> i64 {
        match self {
            Self::None => 0,
            Self::Payments { withholding, .. } => *withholding,
        }
    }
}
crate::community::divide_enum!(FarmInsuranceRecords {
    records: [Payments { amounts: [current, previous_deferred, withholding], facts: [] }],
    units: [None],
    tuples: [],
});

/// Crop-insurance timing and details for this farm
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_link("OpenCropInsuranceFacts")]
pub struct CropInsuranceActivityLink {
    /// Accepted activity identity
    #[libertas_hidden]
    pub activity: i64,
    /// Farm or farm rental
    #[libertas_read_only]
    #[libertas_formatted_text]
    pub label: Vec<u8>,
}
pub(crate) fn activity_links(d: &FederalDraft) -> Vec<CropInsuranceActivityLink> {
    let claims = crate::related_forms::effective_claimants(d);
    crate::farm::sources(d)
        .filter(|s| {
            s.farm.insurance.current_total() > 0 || !papers(d, s.entry.id, &claims).is_empty()
        })
        .map(|s| CropInsuranceActivityLink {
            activity: s.entry.id,
            label: crate::interview::income_issue_context(s.entry).label().encode(),
        })
        .collect()
}
