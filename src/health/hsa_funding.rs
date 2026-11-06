pub use crate::hsa_source::{HsaFunding, HsaFundingBasis, HsaFundingUseHistory};
use crate::validation::ValidationIssue;
// Notice 2008-51: qualified current-year IRA funding and its separate testing period.
// Actual source identity: accepted Form1099-R payment IDs bound each exclusion;
// the retirement owner resolves inherited basis groups and receives their summed
// exclusions separately. No annual group can supply another payment's gross.
// Evidence: tests/passthrough inherited_hsa_funding_caps_actual_parent_and_preserves_group_projection,
// inherited_hsa_payment_keeps_qcd_and_estate_group_footing_once,
// hsa_payment_reference_rejects_wrong_owner_kind_and_inherited_group;
// tests/interview hsa_actual_payment_picker_saves_reopens_restarts_and_rejects_forged_sources.
// Own nonqualified Roth withdrawals with confirmed opening history and no current
// incoming basis use the existing Roth ordering allocator; exact funding classes
// are removed once from annual ordinary income and conversion recapture. With
// retained regular contributions, each recorded value must cover its transfer
// plus opening basis and all annual regular contributions; this sufficient bound
// establishes zero funding basis without choosing deposit timing. Evidence:
// tests/passthrough roth_earlier_*; tests/interview
// roth_earlier_funding_public_save_reopen_and_source_change_boundaries. General
// traditional and other current-incoming coordination remains unverified.
// Funding-first confirmations cannot contradict actual same-pool ordinary dates.
// Evidence: tests/passthrough roth_funding_first_confirmation_* and the public
// inherited Roth lifecycle case.
// Nonqualified inherited Roth funding coordinates earlier ordinary payments
// using the same accepted beneficiary/decedent pool and exact basis classes.
// Evidence: tests/passthrough inherited_roth_earlier_funding_* and
// tests/interview inherited_roth_earlier_funding_public_save_reopen_and_source_correction.
use crate::federal_rules::valid_date;
use crate::kernel::phase;
use crate::kernel::ratio::exact;
use crate::kernel::ratio::positive;
use crate::line_emission::line_cents;
use crate::{
    FederalDraft, FederalIncome, FederalResult, HsaCoverage, HsaEligibilityLoss, HsaLossReason,
    Owner, RetirementKind, RetirementTaxable, RetirementTreatment,
};
use alloc::collections::BTreeMap;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Qualified 2026 IRA-to-HSA transfer
/// A completed direct transfer from the owner's traditional or Roth IRA to their own HSA, or from an IRA they inherited. Enter its Form 1099-R once in Retirement and choose it here. This transfer is not a personal contribution, deduction or ordinary rollover. Prior-basis funding uses source balances and the owner-level IRA worksheet. Inherited sources use only the linked beneficiary/deceased-owner group. Dated nonqualified Roth withdrawals can be coordinated from the applicable opening history; other earlier basis activity needs further support.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HsaFundingRow {
    /// Whose HSA
    #[libertas_default(Taxpayer)]
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Form 1099-R of the transfer
    /// Choose the actual same-owner Form 1099-R payment. For an inherited payment, its existing beneficiary/deceased-owner group supplies the basis; the annual group is not a payment.
    #[libertas_enum_source("$.ira_sources")]
    pub source: i64,
    /// Was the funding IRA an ongoing SEP or SIMPLE IRA?
    /// Yes if an employer contribution was made for that account's plan year ending with or within the distribution tax year, including a contribution made later for that plan year. Check the actual funding account, not other IRAs or HSA employer contributions.
    pub ongoing_sep_simple: bool,
    /// HSA receipt date in 2026
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub funded: u32,
    /// Amount actually transferred directly
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// IRA basis treatment
    pub basis: HsaFundingBasis,
    /// Eligible HDHP coverage at the time of transfer
    /// This can differ from coverage on the first day of that month. No disqualifying insurance, Medicare enrollment or dependent status at transfer.
    pub coverage: HsaCoverage,
    /// First known loss of eligibility after transfer, if any
    /// Enter the first loss even if coverage later returned. The testing period ends on the last day of the same month in 2027. Absence means no known loss; retain coverage records until the period ends.
    /// [NullText]
    /// No loss of HSA eligibility is known after this transfer
    /// [SetText]
    /// The first known loss of HSA eligibility was
    pub loss: Option<HsaEligibilityLoss>,
}
impl HsaFundingRow {
    pub(crate) fn record(&self) -> HsaFunding {
        HsaFunding {
            income: self.source,
            ongoing_sep_simple: self.ongoing_sep_simple,
            funded: self.funded,
            amount: self.amount,
            basis: self.basis.clone(),
            coverage: self.coverage,
            loss: self.loss.clone(),
        }
    }
    // The rules the page checks, checked again on save with the same keys.

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.ongoing_sep_simple {
            Some(
                crate::validation::ValidationIssue::new("HSA_FUNDING_ONGOING_SEP_SIMPLE")
                    .at_field("ongoing_sep_simple"),
            )
        } else if self.basis.traditional_date().is_some_and(|date| {
            !valid_date(date) || date / 10000 != crate::year_constants::TAX_YEAR
        }) {
            Some(
                crate::validation::ValidationIssue::new("HSA_FUNDING_DISTRIBUTION_YEAR")
                    .at_field("distributed")
                    .in_field("basis"),
            )
        } else if self.basis.traditional_date().is_some_and(|date| date > self.funded) {
            Some(
                crate::validation::ValidationIssue::new("HSA_FUNDING_DISTRIBUTION_AFTER_RECEIPT")
                    .at_field("distributed")
                    .in_field("basis"),
            )
        } else if self.loss.as_ref().is_some_and(|v| v.date < self.funded) {
            Some(
                crate::validation::ValidationIssue::new("HSA_FUNDING_LOSS_BEFORE_FUNDING")
                    .at_field("date")
                    .in_field("loss"),
            )
        } else if self.coverage == HsaCoverage::None {
            Some(
                crate::validation::ValidationIssue::new("HSA_FUNDING_NEEDS_COVERAGE")
                    .at_field("coverage"),
            )
        } else {
            None
        }
    }
}
impl HsaFunding {
    // The source event and HSA receipt have different statutory roles. Roth's
    // date remains on its actual payment; traditional funding owns only its
    // otherwise missing actual date. Never substitute the receipt date.
    pub(crate) fn distribution_date(&self, d: &FederalDraft) -> Option<u32> {
        let source = crate::retirement_return::ira_funding_source(d, self.income)?;
        match &source.entry.income {
            FederalIncome::Roth { data } if matches!(self.basis, HsaFundingBasis::Roth { .. }) => {
                Some(crate::RothWithdrawal::from(data).distributed)
            }
            FederalIncome::Retirement { .. } => self.basis.traditional_date(),
            _ => None,
        }
    }
    pub(crate) fn row(&self, owner: Owner) -> HsaFundingRow {
        HsaFundingRow {
            owner,
            source: self.income,
            ongoing_sep_simple: self.ongoing_sep_simple,
            funded: self.funded,
            amount: self.amount,
            basis: self.basis.clone(),
            coverage: self.coverage,
            loss: self.loss.clone(),
        }
    }
}

/// An actual IRA payment offered for qualified HSA funding
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HsaFundingSourceChoice {
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_number(min = 1)]
    pub id: i64,
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: alloc::vec::Vec<u8>,
    #[libertas_read_only]
    pub owner: Owner,
    #[libertas_read_only]
    pub kind: HsaFundingSourceKind,
    /// [NullText]
    /// This payment is not linked to an inherited IRA group
    /// [SetText]
    /// The inherited account's original owner died on
    #[libertas_date_only]
    #[libertas_read_only]
    pub inherited_died: Option<u32>,
    /// [NullText]
    /// No death is recorded for this payment's owner
    /// [SetText]
    /// This payment's owner died on
    #[libertas_date_only]
    #[libertas_read_only]
    pub owner_died: Option<u32>,
}
/// Canonical date owner for an actual funding source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HsaFundingSourceKind {
    Traditional,
    Roth {
        #[libertas_date_only]
        #[libertas_read_only]
        distributed: u32,
    },
}
#[derive(Default)]
pub(crate) struct FundingResult {
    // Income left out of an owner's distributions by entry: qualified HSA funding, and a
    // pre-2026 excess paid back (Form 5329 line 12, ira::exclude_returned_prior_excess).
    pub excluded: BTreeMap<i64, i64>,
    pub inherited_excluded: BTreeMap<i64, i64>,
    pub basis_used: [i64; 2],
    pub inherited_basis_used: BTreeMap<i64, i64>,
    pub roth_basis_used: [i64; 2],
    // Exact opening classes used by supported funding after ordinary Roth
    // withdrawals. Annual Roth income/recapture removes these classes once.
    pub roth_ordered_basis: [Option<crate::RothOrdering>; 2],
    // Exact inherited regular/conversion classes consumed by dated funding,
    // keyed by the existing beneficiary/deceased-owner group.
    pub inherited_roth_ordered_basis: BTreeMap<i64, crate::InheritedRothBasis>,
}
impl FundingResult {
    pub(crate) fn for_inherited(&self, id: i64) -> i64 {
        self.inherited_excluded.get(&id).copied().unwrap_or(0)
    }
    pub(crate) fn for_entry(&self, id: i64) -> i64 {
        self.excluded.get(&id).copied().unwrap_or(0)
    }
}
impl HsaFunding {
    pub(crate) fn recapture(&self) -> i64 {
        if self
            .loss
            .as_ref()
            .is_some_and(|v| v.date / 10000 == 2026 && v.reason == HsaLossReason::Other)
        {
            self.amount
        } else {
            0
        }
    }
}

// A native source location is not a submitted Page index. The adapter must
// correlate a Transfer by owner and its full facts; Account uses that owner;
// Selection names the Page's shared funding collection explicitly.
#[derive(Clone, Debug)]
pub(crate) enum FundingIssueLocation {
    Transfer { owner: Owner, transfer: HsaFunding },
    Account { owner: Owner },
    Selection { owner: Owner },
}
#[derive(Clone, Debug)]
pub(crate) struct FundingReferenceIssue {
    pub location: FundingIssueLocation,
    pub issue: ValidationIssue,
    // Actual income source participating in the failed graph relation.
    pub income_source: Option<i64>,
}
// One transfer's existing reference/date guards, expressed in HsaFundingRow
// field names. Call this directly while enumerating actual submitted rows.
pub(crate) fn transfer_reference_issue(
    d: &FederalDraft,
    owner: Owner,
    transfer: &HsaFunding,
) -> Option<ValidationIssue> {
    let Some(source) = crate::retirement_return::ira_funding_source(d, transfer.income)
        .filter(|source| source.entry.owner == crate::DocumentOwner::from(owner))
    else {
        return Some(ValidationIssue::new("HSA_FUNDING_SOURCE_NOT_OFFERED").at_field("source"));
    };
    if let Some(issue) = transfer.row(owner).issue() {
        return Some(issue);
    }
    let Some(distributed) = transfer.distribution_date(d) else {
        return Some(ValidationIssue::new("HSA_FUNDING_SOURCE_NOT_OFFERED").at_field("source"));
    };
    if !valid_date(distributed) || distributed / 10000 != crate::year_constants::TAX_YEAR {
        return Some(ValidationIssue::new("HSA_FUNDING_DISTRIBUTION_YEAR").at_field("source"));
    }
    if distributed > transfer.funded {
        return Some(
            ValidationIssue::new("HSA_FUNDING_DISTRIBUTION_AFTER_RECEIPT").at_field("funded"),
        );
    }
    if source.inherited.is_some_and(|(_, group)| group.died > distributed) {
        let issue = ValidationIssue::new("HSA_FUNDING_BEFORE_INHERITED_DEATH");
        return Some(if transfer.basis.traditional_date().is_some() {
            issue.at_field("distributed").in_field("basis")
        } else {
            issue.at_field("source")
        });
    }
    if d.people
        .as_ref()
        .and_then(|people| {
            if owner == Owner::Taxpayer { Some(&people.taxpayer) } else { people.spouse.as_ref() }
        })
        .and_then(|person| person.death_date())
        .is_some_and(|date| transfer.funded > date)
    {
        return Some(
            ValidationIssue::new("HSA_FUNDING_OWNER_DIED_BEFORE_RECEIPT").at_field("funded"),
        );
    }
    None
}
// Preserve account order, then all transfer guards, Roth consistency, lifetime.
pub(crate) fn source_reference_issue(d: &FederalDraft) -> Option<FundingReferenceIssue> {
    for account in &d.adjustments.as_ref()?.hsa {
        for transfer in &account.funding {
            if let Some(issue) = transfer_reference_issue(d, account.owner, transfer) {
                return Some(FundingReferenceIssue {
                    location: FundingIssueLocation::Transfer {
                        owner: account.owner,
                        transfer: transfer.clone(),
                    },
                    issue,
                    income_source: Some(transfer.income),
                });
            }
        }
        if let Some((transfer, conflict)) = account.funding.iter().find_map(|transfer| {
            matches!(transfer.basis, HsaFundingBasis::Roth { before_other_activity: true, .. })
                .then(|| crate::roth::funding_first_conflict(d, transfer.income))
                .flatten()
                .map(|source| (transfer, source))
        }) {
            return Some(FundingReferenceIssue {
                location: FundingIssueLocation::Transfer {
                    owner: account.owner,
                    transfer: transfer.clone(),
                },
                issue: ValidationIssue::new("HSA_FUNDING_ORDER_CONTRADICTED")
                    .at_field("before_other_activity")
                    .in_field("basis"),
                income_source: Some(conflict),
            });
        }
        if let Some(issue) = lifetime_issue(d, account) {
            return Some(issue);
        }
    }
    None
}
pub(crate) fn source_reference_rule(d: &FederalDraft) -> Option<&'static str> {
    source_reference_issue(d).map(|failure| failure.issue.key)
}
pub(crate) fn lifetime_issue(
    d: &FederalDraft,
    account: &crate::FederalHsa,
) -> Option<FundingReferenceIssue> {
    if account.funding.is_empty() {
        return None;
    }
    let mut transfers: alloc::vec::Vec<_> = account.funding.iter().collect();
    transfers.sort_by_key(|transfer| transfer.distribution_date(d));
    let second_allowed = transfers.len() != 2
        || (transfers[0].coverage == HsaCoverage::SelfOnly
            && transfers[1].coverage == HsaCoverage::Family
            && transfers[0].distribution_date(d).is_some_and(|first| {
                transfers[1].distribution_date(d).is_some_and(|second| {
                    valid_date(first)
                        && valid_date(second)
                        && first / 10000 == crate::year_constants::TAX_YEAR
                        && second / 10000 == crate::year_constants::TAX_YEAR
                        && first / 100 < second / 100
                        && account.coverage.month((first / 100 % 100 - 1) as usize)
                            == HsaCoverage::SelfOnly
                        && account.coverage.month((second / 100 % 100 - 1) as usize)
                            == HsaCoverage::Family
                })
            }));
    let (location, issue) = if !account.uses.is_empty() {
        (
            FundingIssueLocation::Account { owner: account.owner },
            ValidationIssue::new("HSA_FUNDING_LIFETIME").at_field("uses"),
        )
    } else if !account.funding_history.is_empty() {
        (
            FundingIssueLocation::Account { owner: account.owner },
            ValidationIssue::new("HSA_FUNDING_LIFETIME").at_field("funding_history"),
        )
    // IRC 408(d)(9): one lifetime funding transfer, plus the permitted
    // self-only to family coverage transfer in that same year.
    } else if transfers.len() > 2 || !second_allowed {
        (
            FundingIssueLocation::Selection { owner: account.owner },
            ValidationIssue::new("HSA_FUNDING_LIFETIME").at_field("funding"),
        )
    } else {
        return None;
    };
    Some(FundingReferenceIssue { location, issue, income_source: None })
}
fn lifetime_rule(d: &FederalDraft, account: &crate::FederalHsa) -> Option<&'static str> {
    lifetime_issue(d, account).map(|failure| failure.issue.key)
}

pub(crate) fn calculate(d: &FederalDraft, r: &mut FederalResult) -> FundingResult {
    let mut result = FundingResult::default();
    let Some(a) = &d.adjustments else {
        return result;
    };
    for h in &a.hsa {
        let person = d.people.as_ref().and_then(|p| {
            if h.owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
        });
        let mut transfers: alloc::vec::Vec<_> = h.funding.iter().collect();
        transfers.sort_by_key(|v| v.distribution_date(d));
        crate::require_localized!(
            r,
            lifetime_rule(d, h).is_none(),
            &crate::IssueContext::Section(crate::TaxSection::Hsa),
            "HSA funding",
            "ISSUE_QUALIFIED_FUNDING_IS_ONCE_PER_LIFETIME_WITH_ONLY_A_SECOND_TRANSFER_ACB9D00B",
            "Qualified funding is once per lifetime, with only a second transfer in a later month of the same year after self-only changes to family coverage.",
        );
        for transfer in &transfers {
            let consistent =
                h.funding.iter().all(|other| {
                    transfer.consistent_loss(
                        other.funded,
                        other.funded / 100 + 100,
                        other.loss.as_ref(),
                    )
                }) && transfer.consistent_loss(20261201, 202712, h.last_month_loss.as_ref())
                    && h.last_month_history.as_ref().is_none_or(|old| {
                        transfer.consistent_loss(20251201, 202612, old.loss.as_ref())
                    });
            crate::require_localized!(
                r,
                consistent,
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "HSA funding",
                "ISSUE_USE_CONSISTENT_FIRST_LOSS_FACTS_FOR_OVERLAPPING_CURRENT_FUNDING_PRI_3F5AFB4B",
                "Use consistent first-loss facts for overlapping current funding, prior last-month and current last-month testing periods.",
            );
        }
        let owner = usize::from(h.owner == Owner::Spouse);
        let basis = a.ira_basis.iter().find(|v| v.owner == h.owner);
        let mut remaining_basis = basis.map_or_else(|| exact::whole(0), |v| v.line_2_exact());
        let roth = a.roth_history.iter().find(|v| v.owner == h.owner);
        let ordered_roth = crate::roth::earlier_funding(d, h.owner);
        let ordered_inherited_roth: BTreeMap<_, _> = d
            .income
            .iter()
            .filter(|e| {
                e.owner == crate::DocumentOwner::from(h.owner)
                    && matches!(&e.income, FederalIncome::InheritedIra { data } if data.roth())
            })
            .filter_map(|e| {
                crate::roth::inherited_earlier_funding(d, e.id).map(|ordered| (e.id, ordered))
            })
            .collect();
        let mut remaining_roth_basis = roth.and_then(|v| v.ordering.as_ref()).map_or(0, |v| {
            v.regular + v.conversions.iter().map(|c| c.taxable + c.nontaxable).sum::<i64>()
        });
        let mut cumulative = 0;
        for transfer in transfers {
            let source_date = transfer.distribution_date(d);
            // Zero cannot be admitted below; it is not a deemed date for an
            // unresolved or mismatched source.
            let distributed = source_date.unwrap_or(0);
            let resolved = crate::retirement_return::ira_funding_source(d, transfer.income)
                .filter(|source| source.entry.owner == crate::DocumentOwner::from(h.owner));
            let source = resolved.as_ref().map(|source| source.entry);
            let inherited = resolved.as_ref().and_then(|source| source.inherited);
            let valid_source = if let Some((group, data)) = inherited {
                data.died <= distributed
                    && source.is_some_and(|e| match &e.income {
                        FederalIncome::Retirement { data: source } => {
                            let data = &crate::FederalRetirement::from(source);
                            matches!(
                                data.ordinary_distribution,
                                Some(RetirementTreatment::Ordinary)
                                    | Some(RetirementTreatment::Early)
                            )
                        }
                        FederalIncome::Roth { data: source } => {
                            let data = &crate::RothWithdrawal::from(source);
                            data.distributed <= transfer.funded
                        }
                        _ => false,
                    })
                    && if data.roth() {
                        data.roth_basis().is_some()
                            && matches!(transfer.basis, HsaFundingBasis::Roth { value, before_other_activity }
                                if transfer.amount <= value && ((before_other_activity && crate::roth::funding_first_consistent(d, transfer.income)) || ordered_inherited_roth.get(&group)
                                    .is_some_and(|ordered| ordered.basis.contains_key(&transfer.income))))
                    } else {
                        match transfer.basis {
                            HsaFundingBasis::Pretax { .. } => data.basis().is_none(),
                            HsaFundingBasis::PriorBasis {
                                value, before_other_activity, ..
                            } => {
                                data.basis().is_some()
                                    && transfer.amount <= value
                                    && before_other_activity
                            }
                            _ => false,
                        }
                    }
            } else {
                source.is_some_and(|e| match &e.income {
                    FederalIncome::Retirement { data: source } => {
                        let v = &crate::FederalRetirement::from(source);
                        v.kind == RetirementKind::Ira
                            && match transfer.basis {
                                HsaFundingBasis::Roth { .. } => false,
                                HsaFundingBasis::Pretax { .. } => {
                                    basis.is_none()
                                        && (v.taxable == Some(crate::RetirementTaxable::Payer)
                                            && v.box_2a == v.gross)
                                }
                                HsaFundingBasis::PriorBasis { value, .. } => {
                                    basis.is_some_and(|b| {
                                        b.reservist_repayments
                                            .iter()
                                            .flat_map(|p| &p.payments)
                                            .all(|p| p.date >= distributed)
                                    }) && basis.is_none_or(|b| {
                                        b.special_repayments
                                            .iter()
                                            .flat_map(|p| &p.payments)
                                            .all(|p| p.date >= distributed)
                                    }) && v.taxable == Some(RetirementTaxable::IraBasis)
                                        && transfer.amount <= value
                                }
                            }
                            && matches!(
                                v.ordinary_distribution,
                                Some(RetirementTreatment::Ordinary)
                                    | Some(RetirementTreatment::Early)
                            )
                    }
                    FederalIncome::Roth { data: source } => {
                        let data = &crate::RothWithdrawal::from(source);
                        match transfer.basis {
                            HsaFundingBasis::Roth { value, before_other_activity } => {
                                data.distributed <= transfer.funded
                                    && transfer.amount <= value
                                    && ((before_other_activity
                                        && crate::roth::funding_first_consistent(
                                            d,
                                            transfer.income,
                                        ))
                                        || ordered_roth.as_ref().is_some_and(|ordered| {
                                            ordered.basis.contains_key(&e.id)
                                        }))
                                    && roth.is_some_and(|v| v.ordering.is_some())
                            }
                            _ => false,
                        }
                    }
                    _ => false,
                })
            };
            let valid = transfer.valid()
                && valid_source
                && source_date.is_some_and(|date| {
                    valid_date(date)
                        && date / 10000 == crate::year_constants::TAX_YEAR
                        && date <= transfer.funded
                })
                && lifetime_rule(d, h).is_none()
                && (matches!(transfer.basis, HsaFundingBasis::Pretax { .. })
                    || a.qualified_charity
                        .iter()
                        .flat_map(|q| &q.payments)
                        .filter(|p| {
                            p.inherited.is_some()
                                && p.inherited == inherited.map(|(id, _)| id)
                                && p.amount > 0
                        })
                        .all(|p| p.date >= distributed))
                && person.is_some_and(|p| {
                    p.birth_date <= distributed
                        && p.death_date().is_none_or(|v| transfer.funded <= v)
                        && transfer.loss.as_ref().is_none_or(|loss| {
                            (loss.reason != HsaLossReason::Death
                                || loss.date / 10000 == 2027
                                || p.death_date() == Some(loss.date))
                                && p.death_date().is_none_or(|v| loss.date <= v)
                        })
                });
            crate::require_localized!(
                r,
                valid,
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "HSA funding",
                "ISSUE_LINK_A_SAME_OWNER_IRA_DISTRIBUTION_AND_MATCHING_TRADITIONAL_OR_ROTH_2FFBD8E7",
                "Link a same-owner IRA distribution and matching traditional or Roth basis method. Roth source distributions cannot follow HSA receipt and need complete opening history. Basis funding needs dated aggregate balances. Earlier own nonqualified Roth withdrawals require confirmed opening history and distinct actual payment dates. Current regular contributions are supported only when each recorded Roth value covers its transfer plus all opening basis and retained annual regular contributions; other incoming basis or earlier activity needs further support. Inherited sources need the linked beneficiary/deceased-owner history and must follow the original owner's death. Nonqualified inherited Roth funding also supports dated ordinary withdrawals in that group; its opening regular and conversion basis remains separate from personal accounts and other deceased owners. A funding-first answer cannot override an entered earlier ordinary withdrawal date in that same Roth group.",
            );
            cumulative += transfer.amount;
            let maximum = if transfer.coverage == HsaCoverage::Family {
                crate::year_constants::HSA_FAMILY_LIMIT_2026_CENTS
            } else {
                crate::year_constants::HSA_SELF_ONLY_LIMIT_2026_CENTS
            } + if person.is_some_and(|p| crate::hsa::catchup_age(p.birth_date, 2026))
            {
                crate::year_constants::HSA_CATCH_UP_CENTS
            } else {
                0
            };
            crate::require_localized!(
                r,
                cumulative <= maximum,
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "HSA funding",
                "ISSUE_FUNDING_EXCEEDS_THE_ANNUAL_AMOUNT_FOR_THE_COVERAGE_AT_TRANSFER_AND_C6E212C2",
                "Funding exceeds the annual amount for the coverage at transfer and the owner's age-55 catch-up; both permitted transfers share the family maximum.",
            );
            if valid {
                *result.excluded.entry(transfer.income).or_default() += transfer.amount;
                let used = if let Some((group, data)) = inherited {
                    *result.inherited_excluded.entry(group).or_default() += transfer.amount;
                    if let Some(ordered) = ordered_inherited_roth.get(&group) {
                        let consumed = ordered.basis[&transfer.income];
                        *result.inherited_basis_used.entry(group).or_default() += consumed;
                        Some((consumed, "income.inherited_ira_or_roth"))
                    } else {
                        let (opening, qualified) = if data.roth() {
                            (
                                data.roth_basis().map_or(0, |b| b.regular + b.converted),
                                data.roth_qualified(),
                            )
                        } else {
                            (data.basis().map_or(0, |b| b.prior), false)
                        };
                        let already = result.inherited_basis_used.get(&group).copied().unwrap_or(0);
                        let remaining = crate::kernel::carry::native_remaining::<
                            crate::kernel::carry::Cents,
                        >(opening, already)
                        .get();
                        match transfer.basis {
                            HsaFundingBasis::PriorBasis { value, .. }
                            | HsaFundingBasis::Roth { value, .. } => {
                                let taxable =
                                    if qualified { 0 } else { positive(value - remaining) };
                                let consumed =
                                    phase::cap(positive(transfer.amount - taxable), [remaining]);
                                *result.inherited_basis_used.entry(group).or_default() += consumed;
                                Some((consumed, "income.inherited_ira_or_roth"))
                            }
                            HsaFundingBasis::Pretax { .. } => None,
                        }
                    }
                } else {
                    match transfer.basis {
                        HsaFundingBasis::PriorBasis { value, .. } => {
                            let taxable =
                                (exact::whole(value) - &remaining_basis).max(exact::whole(0));
                            let consumed = (exact::whole(transfer.amount) - taxable)
                                .max(exact::whole(0))
                                .min(remaining_basis.clone());
                            // The actual transfer bounds its consumed basis Cents.
                            let consumed =
                                exact::rounded(&consumed).expect("basis use bounded by transfer");
                            remaining_basis = (remaining_basis
                                - crate::kernel::ratio::exact::whole(consumed))
                            .max(crate::kernel::ratio::exact::whole(0));
                            result.basis_used[owner] += consumed;
                            Some((consumed, "adjustments.ira_basis"))
                        }
                        HsaFundingBasis::Roth { value, .. } => {
                            if let Some(ordered) = &ordered_roth {
                                let consumed = ordered.basis[&transfer.income];
                                result.roth_basis_used[owner] += consumed;
                                Some((consumed, "adjustments.roth_history"))
                            } else {
                                let exempt = person.is_some_and(|p| {
                                    source.is_some_and(|e| {
                                        e.income.roth().is_some_and(|data| {
                                            data.distributed
                                                >= crate::roth_source::age_half(p.birth_date)
                                        })
                                    })
                                }) || source.is_some_and(|e| {
                                    e.income
                                        .roth()
                                        .is_some_and(|data| data.disabled_since().is_some())
                                });
                                let qualified =
                                    exempt && roth.is_some_and(|v| v.first_year <= 2021);
                                let taxable = if qualified {
                                    0
                                } else {
                                    positive(value - remaining_roth_basis)
                                };
                                let consumed = phase::cap(
                                    positive(transfer.amount - taxable),
                                    [remaining_roth_basis],
                                );
                                remaining_roth_basis = crate::kernel::carry::native_balance(
                                    remaining_roth_basis,
                                    consumed,
                                );
                                result.roth_basis_used[owner] += consumed;
                                Some((consumed, "adjustments.roth_history"))
                            }
                        }
                        HsaFundingBasis::Pretax { .. } => None,
                    }
                };
                if let Some((consumed, context)) = used {
                    line_cents(
                        r,
                        &alloc::format!(
                            "hsa_funding_basis_{}_{}",
                            transfer.income,
                            transfer.funded
                        ),
                        consumed,
                        &["adjustments.hsa.funding", context],
                        "https://www.irs.gov/irb/2008-25_IRB",
                        msg!("EXPLAIN_HSA_FUNDING_BASIS_N_N"),
                    );
                }
            }
            line_cents(
                r,
                &alloc::format!("hsa_funding_{}_{}", transfer.income, transfer.funded),
                transfer.amount,
                &["adjustments.hsa.funding", "income.retirement", "income.roth"],
                "https://www.irs.gov/irb/2008-25_IRB",
                msg!("EXPLAIN_HSA_FUNDING_N_N"),
            );
        }
        // Only hand the annual owner the complete class removal after every
        // projected source actually received its entire exclusion. A rejected
        // second transfer cannot remove basis merely because the first passed.
        if let Some(ordered) = ordered_roth
            && ordered.basis.keys().all(|id| {
                d.income.iter().find(|e| e.id == *id).is_some_and(|e| {
                    e.income.roth().is_some_and(|data| result.for_entry(*id) == data.gross)
                })
            })
        {
            result.roth_ordered_basis[owner] = Some(ordered.consumed);
        }
        for (group, ordered) in ordered_inherited_roth {
            if ordered.basis.keys().all(|id| {
                d.income.iter().find(|e| e.id == *id).is_some_and(|e| {
                    e.income.roth().is_some_and(|data| result.for_entry(*id) == data.gross)
                })
            }) {
                result.inherited_roth_ordered_basis.insert(group, ordered.consumed);
            }
        }
    }
    for (id, amount) in &result.excluded {
        let gross = d
            .income
            .iter()
            .find(|v| v.id == *id)
            .and_then(|e| match &e.income {
                FederalIncome::Retirement { data: source } => {
                    let data = &crate::FederalRetirement::from(source);
                    Some(data.gross)
                }
                FederalIncome::Roth { data: source } => {
                    let data = &crate::RothWithdrawal::from(source);
                    Some(data.gross)
                }
                _ => None,
            })
            .unwrap_or(0);
        crate::require_localized!(
            r,
            *amount <= gross,
            &crate::IssueContext::Section(crate::TaxSection::Hsa),
            "HSA funding",
            "ISSUE_QUALIFIED_FUNDING_CANNOT_EXCEED_THE_LINKED_SOURCE_IRA_DISTRIBUTION_FDB93B9C",
            "Qualified funding cannot exceed the linked source IRA distribution. QCDs use only the remaining source dollars.",
        );
    }
    result
}
