use crate::kernel::ratio::exact::{self, Exact};
use crate::validation::ValidationIssue;
// Coverage: Form 8379 (approved Unit 13 work, checkpoint 2026-10-10).
// The editor uses platform-owned parent-relative numeric bounds, constrained
// choices and the ordinary accepted-save lifecycle. Final-owner limitations
// remain explicit below; missing allocations do not alter ordinary tax.
// Sources: final form November 2023/instructions November 2024; December 2026
// form and instructions DRAFT (instructions proof September 22, 2026), for
// tax years 2026 onward. Draft 2027 worksheet placeholders are not constants.
// https://www.irs.gov/pub/irs-dft/i8379--dft.pdf (printed pages 3-5)
// https://www.irs.gov/irm/part21/irm_21-004-006r (21.4.6.5.6.1-3)
// https://www.irs.gov/irb/2004-30_IRB (Rev. Rul. 2004-71 through 74)
// Native K-1 box20AI additional tax retains settled source cents and the
// existing joint Schedule2 line13h amount. Selected-owner rounding uses the
// shared kernel; it enters line18 and separate liabilities exactly once.
// Part III preserves accepted joint amounts and source ownership. Line 15 is
// 1040 line 12e only; supplementary deductions enter the liability worksheet.
// Line 18 includes Schedule 2 line 1z plus Part II, removing line 1z from the
// proportional tax base once. ACTC follows child claimants; EIC uses the
// hypothetical separate credits with the joint child count and table column.
// Mixed child pools use approved explicit feasibility-bounded choices, not a
// purported IRS formula. Payments retain actual payer and property source;
// only joint payments use an agreement or the separate-tax ratio. An undefined
// ratio requires an agreement. Withholding reconciles 25a/b/c independently,
// with actual Additional Medicare credit and excess Social Security once.
// Community refund rules are a separate source-property/debt overlay. Part III
// shows separately earned amounts; no Form 8958 division is reapplied.
// Verified historical ruling patterns retain their assumptions. Multistate,
// unverified collection regimes or missing final-owner feeds leave out only
// the named Form 8379 benefit, retaining known allocations and the ordinary
// federal estimate. No source preview, equal-share repair, loss rescaling or
// unknown residual supplies an exclusive owner's final amount.
// Portfolio capital attribution retains the actual collector's source deltas,
// QSBS rows, retained installment/exchange filers, Form6781 line1 owners and
// approved absorbed carryback facts. Regular prior short/long shares stay
// distinct from the independent Form8960 difference. Joint netting and annual
// loss floors are unchanged; genuinely shared allowed losses expose a direct
// bounded choice. Native Other/NEC/MISC/K/refund contributions use their
// actual collectors and combined recovery worksheets, not a page preview or
// duplicated routed receipt. Olympic exclusions keep the actual recipient;
// annuity, savings-penalty and substitute-payment NII follow native sources.
// Accepted LTC receipts retain native source-ID interest cents independently
// of taxable care benefits; linked annuity income enters its original owner once.
// Tests: unit13_ltc_niit_sources; ordinary joint financial/IRS amounts unchanged.
// Accepted unemployment uses the native net after same-year repayments and
// retains only unrouted grants/RTAA/family leave at that physical source.
// Alimony, jury pay, educational scholarships and casual gambling retain the
// native noninvestment character; none obscures another source's NIIT owner.
// Tests: unit13_niit_noninvestment_sources; ordinary joint amounts unchanged.
// Personal-property rentals consume the one native same-system post-PAL cents
// packet: actual receipts on13b, allowed current/prior/depreciation costs on14,
// category-rounded net for NIIT ownership. No separate pool or release split.
// Tests: unit13_personal_rental_sources; fractional prior/system/depreciation,
// recharacterization, signed-owner refusal and actual Save/reopen/restart.
// ScheduleE sources retain actual post-activity-limit dollars and source
// capital-character cuts. Native K-1 portfolio parts retain their owner and
// qualified-dividend character; later property/EBL/NII feeds remain distinct.
// Separate liability retains true short/long character and admitted source
// 28%/25% pools, reusing capital::rate_pools and the native tax worksheet.
// Joint final special pools never stand in for an owner's source character.
// Form4952 elections retain native investment inputs and the one joint pool;
// direct class choices use actual source capacities, not separate loss floors.
// Final property/estate-investment carry and mixed-history
// source adapters remain distinct; missing source parts do not become zero.
// Missing final components and unresolved passive/property/signed NII remain
// explicit unavailable benefits rather than zero or a residual owner repair.
// Pure current NUA exclusions retain each confirmed collector result and its
// actual sale owner; the one signed native category uses shared projection.
// Prior/current mixtures and unknown non-NUA contributors stay unavailable.
// Tests: unit13_current_nua_sources; native financial and Save/restart controls.
// Stored changes extend the existing Draft and dated-payment facts; runtime
// projections do not persist derived totals or request catalogs. Client rules
// have server twins rebuilt from accepted return facts.
//   Source columns round the selected injured owner first, preserving each
//   native joint amount and exact signed residual. Transient capital/election
//   source views retain both rounding contexts for the shared no-claim API.
//   QSBS source cohorts retain their actual admitted28% subset at the native
//   issuer owner, including fully excluded and cent-precision source cases.
//   Education/ABLE/medical sources retain actual recipient and completed
//   native income/tax boundaries; unlinked corrections retain their account
//   owner without a fabricated income ID. Tests include same-draft comparisons,
//   cents, actual public Save/restart and source-recipient correction.
//   Direct royalties retain the same native expense/depletion results and
//   actual whole-dollar Form4952 charge. Line13b and Form8960 use that one
//   source net; signed NII or other unresolved deductions remain explicit.
// Tests: src/tests/attribution/injured_spouse.rs; injured_spouse_* interview regressions;
// runtime retirement/disaster/QCD owner projection regressions.
use crate::PersonChoice;
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::{form_line, line_cents};
use crate::{
    FederalDraft, FederalIncome, FederalResult, FederalStatus, FormLine, Message, MessageArg,
    Owner, TaxForm,
};
use alloc::string::String;
use alloc::{format, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub use crate::injured_spouse_model::{
    EstimatedSplit, InjuredClaimantFacts, InjuredCommunityFacts, InjuredDebtFacts,
    InjuredLoanBorrower, InjuredSpouseAllocation, OffsetDebt,
};

mod accounts;
mod construction;
pub(crate) mod income_categories;
pub(crate) mod niit_deductions;
pub(crate) mod niit_retained;
pub use construction::{InjuredAmountAnswered, InjuredAmountInitial, InjuredSpouseInitial};
mod capital;
mod investment;
mod refund_funding;
pub(crate) use accounts::AccountSources;
pub(crate) use investment::{InvestmentElectionSources, InvestmentSources};
pub(crate) mod owners;
pub(crate) use capital::{CapitalOwners, CapitalSources, niit_difference_owners};
pub(crate) use owners::separate_taxes;
use owners::{AllocationProblem, SourceSelections};

/// Injured Spouse
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InjuredSpouse {
    /// Injured spouse
    pub injured: Owner,
    /// Debt the refund pays
    pub debt: OffsetDebt,
    /// The injured spouse is legally obligated to pay this debt
    pub obligated: bool,
    /// Community-property residence during 2026
    pub community: InjuredCommunity,
    /// Joint estimated payments
    /// Separate payments already belong to their payer. Agree on the joint amount, or use the IRS separate-tax formula.
    pub estimated: EstimatedSplit,
    /// Who is legally obligated on the loans whose interest is deducted
    /// Leave empty when there is no student-loan deduction. A borrower who is only a dependent student does not make a spouse legally obligated.
    /// [NullText]
    /// No student-loan interest deduction to allocate
    /// [SetText]
    /// The legally obligated student-loan borrower is
    pub student_loan_borrower: Option<InjuredLoanBorrower>,
    /// Previously answered shared amounts
    /// Review the injured spouse's part of each amount. The other spouse receives the rest.
    #[libertas_size(min = 0)]
    #[libertas_read_only]
    pub allocations: Vec<InjuredAmountPage>,
    /// New shared amounts to allocate
    /// Enter the injured spouse's part of each amount. The other spouse receives the rest.
    #[libertas_size(min = 0)]
    #[libertas_read_only]
    pub new_allocations: Vec<InjuredAmountPage>,
    /// Dependents whose separate-return claimant is needed
    #[libertas_size(min = 0)]
    #[libertas_read_only]
    pub claimants: Vec<InjuredClaimant>,
    /// Joint estimated payments available to divide
    /// [NullText]
    /// The exact capacity exceeds the displayed amount's native representation
    /// [SetText]
    /// Available to divide
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    #[libertas_default($null)]
    pub joint_estimated: Option<i64>,
    /// Actual student-loan deduction on the joint return
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub student_loan_deduction: i64,
    /// Available community-property states
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub states: Vec<InjuredStateChoice>,
    /// Dependents on this return
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub people: Vec<PersonChoice>,
    /// Accepted component catalog
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub components: Vec<InjuredComponentChoice>,
    /// Applicable debt facts
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_debt_facts: Vec<i32>,
    /// Children's credits already attributed to the injured spouse
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub known_child_credit_injured: i64,
    /// Children's credits already attributed to the other spouse
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub known_child_credit_other: i64,
    /// Combined child-credit ceiling for the injured spouse's claimants
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub child_credit_cap_injured: i64,
    /// Combined child-credit ceiling for the other spouse's claimants
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub child_credit_cap_other: i64,
    /// Allocation rows
    #[libertas_read_only]
    #[libertas_output_only]
    pub known: Vec<InjuredPrintedRow>,
    /// Prospective protected part of the joint overpayment
    /// Before later IRS refunds, offsets or outstanding debts. The IRS makes the final computation.
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub estimated_share: Option<i64>,
    /// Allocation explanation
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub explanation: Vec<u8>,
}

/// Injured State Choice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InjuredStateChoice {
    /// State code
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: u16,
    /// State
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}

/// Injured Component Choice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InjuredComponentChoice {
    /// Component
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_size(min = 1)]
    pub id: String,
    /// Item and return line
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}

pub use crate::return_carriers::InjuredAmountPage;

/// Injured Claimant
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InjuredClaimant {
    /// Dependent
    #[libertas_enum_source("$.value.people")]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_read_only]
    pub person: i64,
    /// Who would claim this dependent on separate returns
    /// Use the qualifying-child or qualifying-relative rules, including any tie-breaker and a valid release of the claim.
    pub claimant: Owner,
}

pub use crate::return_carriers::InjuredPrintedRow;

/// Injured Community
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum InjuredCommunity {
    /// No community-property residence during 2026
    No,
    /// Lived in a community-property state during 2026
    Yes {
        /// States where you were domiciled during 2026
        #[libertas_enum_source("$.value.states")]
        // Tax structure: the nine community-property states, each selected at most once.
        #[libertas_size(min = 1, max = 9)]
        #[libertas_unordered]
        states: Vec<u16>,
        /// Federal tax debt timing and use
        /// Needed for community-property federal tax debt. Other debt types use NotApplicable.
        #[libertas_constrained_by("$.value.allowed_debt_facts")]
        debt_facts: InjuredDebtFacts,
    },
}

pub use crate::return_carriers::InjuredAmount;

// The joint estimated tax and extension payments (Form 1040 line 26 and
// Schedule 3 line 10) the agreed split may not exceed, in cents, from the
// accepted payments page.
pub(crate) fn joint_estimated(d: &FederalDraft) -> crate::kernel::ratio::exact::Exact {
    use crate::kernel::ratio::exact::whole;
    d.payments.as_ref().map_or_else(
        || whole(0),
        |p| {
            p.estimated
                .funding()
                .filter(|(_, payer, _)| *payer == crate::PaymentPayer::Joint)
                .map(|(amount, _, _)| whole(amount))
                .sum::<crate::kernel::ratio::exact::Exact>()
                + if p.extension_payer == Some(crate::PaymentPayer::Joint) {
                    whole(p.extension)
                } else {
                    whole(0)
                }
        },
    )
}

// Compare the actual exact source capacity, not its optional display projection.
pub(crate) fn issue(
    value: &InjuredSpouse,
    joint: &crate::kernel::ratio::exact::Exact,
) -> Option<ValidationIssue> {
    match value.estimated {
        EstimatedSplit::Agreed { injured }
            if injured < 0 || &crate::kernel::ratio::exact::whole(injured) > joint =>
        {
            Some(
                ValidationIssue::new("INJURED_SPOUSE_ESTIMATED_ABOVE_PAYMENTS")
                    .at_field("injured")
                    .in_field("estimated"),
            )
        }
        _ => None,
    }
}
pub(crate) fn rule(
    value: &InjuredSpouse,
    joint: &crate::kernel::ratio::exact::Exact,
) -> Result<(), &'static str> {
    issue(value, joint).map_or(Ok(()), |issue| Err(issue.key))
}

// The injured spouse's refund estimate the calculation reports, in cents.
pub(crate) const ESTIMATE_LINE: &str = "form_8379_injured_refund_estimate";

// Per-person amounts the calculation already holds, handed over where the
// Form 1040 lines are complete. Whole dollars unless named cents.
// Actual student bases consumed by the joint Form8863 owner. Both credit
// pools apply a common statutory factor, before the separate joint tax limit.
pub(crate) struct EducationSource {
    pub student: i64,
    pub aotc: i64,
    pub llc: i64,
}

// Actual eligible source bases, after each owner's retained loss. The joint
// Form8995/8995-A continues to own every limit and the allowed deduction.
pub(crate) struct QbiSource {
    pub weights: [i64; 2],
    pub dpad: [i64; 2], // actual notice amounts in cents
    pub allowed_dpad: i64,
}

// Schedule2 line13h: the native collector's joint dollars and settled K-1
// source cents. Selection controls only the existing first-owner rounding;
// neither a claim nor a reconstructed tax calculation owns these amounts.
#[derive(Clone, Copy)]
pub(crate) struct DeferredCompensationTaxSources {
    pub(crate) joint_dollars: i64,
    pub(crate) cents: [i64; 2],
}
impl DeferredCompensationTaxSources {
    pub(crate) fn owners(&self, first: usize) -> Option<[i64; 2]> {
        crate::kernel::ratio::project_rounded_pair(self.joint_dollars, self.cents, first)
    }
}

// Same-invocation K1 investment operands. Cents categories retain their
// independent annual boundaries; activity cuts retain the same actual source cents.
#[derive(Default)]
pub(crate) struct K1NiiSources {
    pub(crate) categories: [[i64; 2]; 4],
    pub(crate) royalties: [i64; 2],
    pub(crate) loans: [i64; 2],
    pub(crate) ordinary: [i64; 2],
    pub(crate) ordinary_cuts:
        Option<[crate::kernel::carry::Amount<crate::kernel::carry::Cents>; 2]>,
    // Native joint dollars: line7/excluded/estate7/estate67e/loan/ordinary.
    pub(crate) targets: [i64; 6],
}

pub(crate) struct JointFacts<'a> {
    pub(crate) income_categories: alloc::boxed::Box<income_categories::NativeIncomeCategories>,
    // The actual line1a operand, after native wage classifications and
    // employee reimbursements; raw box1 is not always line1a income.
    pub(crate) w2_wages: [i64; 2],
    pub(crate) k1_nii: &'a K1NiiSources,
    pub(crate) nii_rental: &'a crate::extended_rules::RentalResult,
    pub(crate) child: &'a crate::federal_rules::ChildTotals,
    // Actual Form8815 exclusion at its linked interest owner, native dollars.
    // A missing source remains unknown; no accepted bond record is known zero.
    pub(crate) savings_bond_exclusion: Option<[i64; 2]>,
    // Final native per-owner Form2555 dollars, borrowed after business/prior
    // coordination. Preserve its per-owner wages rounding and net exclusions.
    pub(crate) foreign: &'a crate::extended_rules::ForeignResult,
    pub(crate) foreign_business_housing: [i64; 2],
    // Cents assigned by the actual native Other/1099 source collectors.
    // An absent/unknown result cannot be replaced with a document preview.
    pub(crate) other_sources: alloc::collections::BTreeMap<i64, Option<Exact>>,
    pub(crate) accounts: AccountSources,
    pub(crate) deferred_compensation_tax: DeferredCompensationTaxSources,
    // Actual direct-property Form4255 source owner's final whole-dollar tax.
    // K-1 passed-through recapture remains a separate source family.
    pub(crate) investment_recapture: [i64; 2],
    // Actual Form4797 ordinary source cents after activity limits.
    // None preserves an unresolved property-character source.
    pub(crate) property_ordinary: Option<[i64; 2]>,
    pub(crate) mortgage_recapture: crate::mortgage_recapture::MortgageRecaptureSources,
    // Native net unemployment after accepted same-year repayments, cents.
    pub(crate) unemployment: [i64; 2],
    pub(crate) k1_portfolio: [[i64; 2]; 3], // interest/ordinary/qualified, source cents
    pub(crate) olympic: [i64; 2],           // source cents, before the joint AGI exclusion
    pub(crate) nii_substitute: [i64; 2],    // actual native substitute-payment cents
    // Final pre-annual-limit capital source nets from the joint owner.
    pub(crate) capital: Option<CapitalOwners>,
    pub(crate) investment_election: InvestmentElectionSources,
    pub(crate) eic_children: usize,
    pub(crate) ctc_ids: Vec<i64>,
    pub(crate) odc_ids: Vec<i64>,
    pub(crate) earned: [i64; 2],
    pub(crate) se: [i64; 2],
    pub(crate) half_se: [i64; 2],
    // The received-alimony collector's actual recapture deduction result,
    // returned as cents after its native whole-dollar worksheet rounding.
    pub(crate) received_alimony_recapture: [i64; 2],
    // Schedule 1 line 12 by the filer whose Form 2106 it is.
    pub(crate) employee_expenses: [i64; 2],
    pub(crate) ira: [i64; 2],
    pub(crate) hsa: [i64; 2],
    pub(crate) pension: [i64; 2],
    pub(crate) vehicle_repayment: [i64; 2],
    pub(crate) sep: [i64; 2],
    pub(crate) health: [i64; 2],
    pub(crate) educator: [i64; 2],
    // Cents.
    pub(crate) business: [i64; 2],
    pub(crate) regular_nol: Vec<crate::nol::InjuredNolPart>,
    pub(crate) ltc: &'a [crate::care_benefit_return::InjuredLtcPart],
    // Accepted native LTC source IDs and their interest cents, independently
    // of the taxable care-benefit pool. Missing source membership is unknown.
    pub(crate) nii_ltc_sources: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(crate) fund_credit: [i64; 2],
    pub(crate) qbi: Option<QbiSource>,
    // Eligible qualifying-person expenses from the joint Form2441 owner.
    pub(crate) care_sources: Vec<(i64, i64)>,
    // Spousal care has no dependent claimant; keep it a shared component.
    pub(crate) care_shared_expenses: i64,
    pub(crate) education_sources: &'a [EducationSource],
    // Actual joint dollars: AOTC nonrefundable, LLC, AOTC refundable.
    pub(crate) education_pools: [i64; 3],
    // Cents.
    pub(crate) withholding_categories: [[i64; 2]; 3],
    pub(crate) retirement: [[i64; 2]; 2],
    // Actual source cents from the same collector as Form8960. Qualified
    // pension income is excluded from the annuity investment-income part.
    pub(crate) nii_annuity: [i64; 2],
    pub(crate) nii_savings_penalty: [i64; 2],
    // Actual final ScheduleE royalty pool, whole dollars. Direct source nets
    // are retained in other_sources as cents after the native routed charge.
    pub(crate) nii_royalties: i64,
    // Complete native allowed personal-property expenses, genuine source cents.
    pub(crate) personal_expenses: Option<[i64; 2]>,
    // Cents.
    pub(crate) medicare_withheld: [i64; 2],
    pub(crate) excess_social: [i64; 2],
    pub(crate) extension: i64,
    // Cents.
    pub(crate) tips: [i64; 2],
    // Cents.
    pub(crate) overtime: [i64; 2],
    // Cents.
    pub(crate) medicare_wages: [i64; 2],
    pub(crate) itemized: bool,
}

// One Part III line: column (a) and the two spouses' parts, by owner index
// (0 the first spouse on the return, 1 the second).
#[derive(Clone, Copy)]
struct Part {
    complete: bool,
    joint: i64,
    owners: [i64; 2],
}

impl Default for Part {
    fn default() -> Self {
        Self { complete: true, joint: 0, owners: [0; 2] }
    }
}
impl Part {
    fn unavailable(&mut self, joint: i64) {
        self.joint += joint;
        self.complete = false;
    }
    fn add(&mut self, joint: i64, owners: impl Into<Option<[i64; 2]>>) {
        let Some(owners) = owners.into() else {
            self.unavailable(joint);
            return;
        };
        self.joint += joint;
        self.owners[0] += owners[0];
        self.owners[1] += owners[1];
        self.complete &= joint == owners[0] + owners[1];
    }
}

// Each owner's amount of the documents one part covers, in cents, and
// whether an owner has a document whose amount is not known.
#[derive(Clone, Default)]
struct Weights {
    cents: [Exact; 2],
    unknown: [bool; 2],
}

// Project retained cents to the joint rounded line; never rescale a source or
// fill an unknown source with the other owner's residual.
struct MedicareWithholdingProjection {
    targets: [i64; 2],
    owners: [Option<[i64; 2]>; 2],
}

// Project the actual native credit pools; never recompute Form8959 tax.
fn medicare_withholding_projection(
    d: &FederalDraft,
    f: &JointFacts<'_>,
    first: usize,
) -> Option<MedicareWithholdingProjection> {
    let Some(source) = f.income_categories.medicare_withholding.as_ref() else {
        // The former explicit owner array supports only its closed ordinary
        // W2 context. It cannot establish railroad/uncollected source zero.
        let joint = dollars(f.medicare_withheld[0].checked_add(f.medicare_withheld[1])?);
        if joint < 0 {
            return None;
        }
        let missing = d.income.iter().any(|entry| matches!(
            &entry.income, FederalIncome::Wage { data }
            if { let data = crate::FederalWage::from(data); matches!(data.payroll, crate::W2Payroll::Railroad { .. }) || data.uncollected_tax.is_some() }
        ));
        return Some(MedicareWithholdingProjection {
            targets: [joint, 0],
            owners: if missing {
                [None; 2]
            } else {
                [
                    crate::kernel::ratio::project_rounded_pair(joint, f.medicare_withheld, first),
                    Some([0; 2]),
                ]
            },
        });
    };
    let targets = [source.ordinary_dollars, source.rrta_dollars];
    if targets.iter().any(|value| *value < 0) || targets[0].checked_add(targets[1]).is_none() {
        return None;
    }
    let project = |joint: i64, raw: [i64; 2], printed: Option<i64>| {
        let net = raw[0].checked_add(raw[1])?;
        if printed.unwrap_or(dollars(positive(net))) != joint {
            return None;
        }
        if joint == 0 {
            return Some([0; 2]);
        }
        // Signed raw differences stay retained. Only positive native
        // excess can supply the credit; a sole eligible source owns J.
        match [raw[0] > 0, raw[1] > 0] {
            [true, false] => Some([joint, 0]),
            [false, true] => Some([0, joint]),
            [true, true] if printed.is_none() => {
                crate::kernel::ratio::project_rounded_pair(joint, raw, first)
            }
            [true, true] => crate::kernel::ratio::project_native_pair(joint, raw, first)
                .filter(|owners| owners.iter().all(|owner| *owner >= 0)),
            [false, false] => None,
        }
    };
    let ordinary = (|| {
        if source
            .ordinary_withheld_cents
            .iter()
            .chain(source.ordinary_wages_cents.iter())
            .any(|value| *value < 0)
            || source.ordinary_regular_cents < 0
        {
            return None;
        }
        let wages = source.ordinary_wages_cents[0].checked_add(source.ordinary_wages_cents[1])?;
        let withheld =
            source.ordinary_withheld_cents[0].checked_add(source.ordinary_withheld_cents[1])?;
        if positive(withheld.checked_sub(source.ordinary_regular_cents)?)
            != source.ordinary_net_cents
        {
            return None;
        }
        let regular = if wages == 0 {
            if source.ordinary_regular_cents != 0 {
                return None;
            }
            [0; 2]
        } else {
            // Allocate the already calculated native regular Cents boundary
            // over its actual source wages, using the declared first/residual.
            crate::kernel::ratio::project_fraction_pair(
                source.ordinary_regular_cents,
                source.ordinary_wages_cents,
                source.ordinary_regular_cents,
                wages,
                first,
            )?
        };
        let ordinary = [
            source.ordinary_withheld_cents[0].checked_sub(regular[0])?,
            source.ordinary_withheld_cents[1].checked_sub(regular[1])?,
        ];
        if positive(ordinary[0].checked_add(ordinary[1])?) != source.ordinary_net_cents {
            return None;
        }
        // Form 8959 instructions: line 22 uses rounded lines 19 and 21.
        let lines = crate::forms::form_8959::ordinary_withholding_lines(withheld, wages);
        project(targets[0], ordinary, Some(lines.additional))
    })();
    let railroad = if source.railroad_unavailable
        || source.rrta_withheld_cents.iter().any(|value| *value < 0)
    {
        None
    } else {
        project(targets[1], source.rrta_withheld_cents, None)
    };
    let owners = [ordinary, railroad];
    Some(MedicareWithholdingProjection { targets, owners })
}

// A final component whose only contributing source belongs to one spouse.
// Mixed owners still need the owning final output or an allowed direct choice.
fn single_or_exact(amount: i64, cents: [i64; 2], first: usize) -> Option<[i64; 2]> {
    match cents {
        [first, 0] if first != 0 => Some([amount, 0]),
        [0, second] if second != 0 => Some([0, amount]),
        _ => crate::kernel::ratio::project_rounded_pair(amount, cents, first),
    }
}

// The instructions' rule for an item that belongs to neither spouse alone:
// divided equally; the odd dollar goes to the first spouse on the return.
fn equal(amount: i64) -> [i64; 2] {
    let second = crate::kernel::ratio::trunc_i64(amount, 2);
    [amount - second, second]
}

fn bucket(amount: i64, weights: Weights, first: usize) -> Option<[i64; 2]> {
    if weights.unknown.iter().any(|v| *v) {
        None
    } else {
        let total = weights.cents.iter().sum::<Exact>();
        exact::project_rounded_pair(amount, &total, weights.cents, first)
    }
}

fn f1040(r: &FederalResult, line: &str) -> i64 {
    r.return_lines
        .iter()
        .find(|l| {
            l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form1040 && f.line == line)
        })
        .map_or(0, |l| crate::kernel::ratio::trunc_i64(l.amount, 100))
}

fn calc(r: &FederalResult, id: &str) -> i64 {
    r.lines.iter().find(|l| l.id == id).map_or(0, |l| dollars(l.amount))
}

// The same native Form1062 result supplies the reported offset. It is a
// tax-deferral bookkeeping amount, never an owner-allocable cash credit.
fn farmland_installment_offset(r: &FederalResult) -> i64 {
    r.farmland_installments.as_ref().map_or(0, |value| dollars(value.applicable))
}

fn owner_index(owner: Owner) -> usize {
    usize::from(owner == Owner::Spouse)
}

// Where one document's amount belongs within line 13b.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Part13b {
    Social,
    Capital,
    Business,
    Interest,
    Dividends,
    Jury,
    Unemployment,
    Rest,
}

// A document's amounts in the parts of line 13b: (part, cents or unknown).
// The explicit arms below read the stored document; the filing status that
// `interview::document_amount` needs is kept for the caller's seam (F15, S6).
pub(crate) fn parts_13b(income: &FederalIncome, _separate: bool) -> Vec<(Part13b, Option<i64>)> {
    use FederalIncome as I;
    match income {
        I::Wage { data } => {
            let data = crate::FederalWage::from(data);
            alloc::vec![(Part13b::Rest, data.total_income().ok().map(|v| v - data.wages))]
        }
        I::SocialSecurity { data } => {
            alloc::vec![(Part13b::Social, Some(crate::FederalSocial::from(data).benefits))]
        }
        I::Dividend { data } => alloc::vec![
            (Part13b::Dividends, Some(data.form.ordinary.unwrap_or(0))),
            (Part13b::Capital, Some(data.form.capital_distributions.unwrap_or(0))),
        ],
        I::Business { .. } => alloc::vec![(Part13b::Business, Some(0))],
        // Excluded from income: nothing on the return.
        I::DisasterMitigation { .. }
        | I::TrumpAbleRollover { .. }
        | I::EducationAccountExpenses { .. }
        | I::LtcInsured { .. }
        | I::PropertyAcquisition { .. } => {
            alloc::vec![(Part13b::Rest, Some(0))]
        }
        // The kinds the estimate's working copy changes (an excluded or left-out
        // payment, a firm net of its costs, a scholarship's taxable part): the
        // allocation reads the amount that is on the return itself, not the
        // row's preview (F3, 2026-10-09).
        I::HealthLoanAssistance { data } => alloc::vec![(Part13b::Rest, Some(data.on_return()))],
        I::WildfireRelief { data } => alloc::vec![(Part13b::Rest, Some(data.on_return()))],
        I::DisasterRelief { data } => alloc::vec![(Part13b::Rest, Some(data.on_return()))],
        I::HobbySales { data } => alloc::vec![(Part13b::Rest, Some(data.on_return()))],
        I::Scholarship { data } => alloc::vec![(Part13b::Rest, Some(data.taxable().0))],
        I::Broker { .. }
        | I::Sale { .. }
        | I::InstallmentSale { .. }
        | I::NuaSale { .. }
        | I::EmployeeStockSale { .. }
        | I::EsppSale { .. }
        | I::EsppSaleSource { .. }
        | I::IsoSaleSource { .. }
        | I::NuaSaleSource { .. }
        | I::IsoSale { .. }
        | I::WorthlessSecurity { .. }
        | I::SmallBusinessStock { .. }
        | I::ReturnOfCapital { .. }
        | I::UndistributedGain { .. }
        | I::QualifiedSmallBusinessSale { .. }
        | I::OpportunityFund { .. }
        | I::OpportunityFundDeferral { .. }
        | I::HomeSale { .. }
        | I::NonbusinessBadDebt { .. }
        | I::Section1256 { .. } => {
            alloc::vec![(Part13b::Capital, None)]
        }
        I::Retirement { .. } | I::InheritedIra { .. } | I::Roth { .. } => {
            alloc::vec![(Part13b::Rest, Some(0))]
        }
        I::Interest { data } => {
            alloc::vec![(Part13b::Interest, Some(crate::FederalInterest::from(data).net_taxable()))]
        }
        I::Oid { data } => {
            alloc::vec![(Part13b::Interest, Some(crate::FederalOid::from(data).net_taxable()))]
        }
        I::LtcBenefits { data } => alloc::vec![(Part13b::Interest, Some(data.record().interest))],
        I::GovernmentPayment { data } => {
            let payments = crate::FederalUnemployment::from(data).amount;
            let refund = if data.form.state_or_local_income_tax_refunds == 0 {
                Some(0)
            } else {
                let crate::FederalTaxRefund { amount, history, .. } = data.into();
                history.taxable(amount).ok()
            };
            alloc::vec![(Part13b::Rest, refund.and_then(|refund| payments.checked_add(refund)))]
        }
        I::EmployeeWages { data } => alloc::vec![(Part13b::Rest, data.income().ok())],
        I::Able { data } => {
            alloc::vec![(Part13b::Rest, crate::AbleDistribution::from(data).taxable())]
        }
        I::TrumpDeath { data } => {
            alloc::vec![(Part13b::Rest, crate::TrumpDeathAcquisition::from(data).taxable())]
        }
        I::Gambling { data } => {
            alloc::vec![(Part13b::Rest, Some(crate::FederalGambling::from(data).winnings))]
        }
        I::CasualtyLivingInsurance { data } => {
            alloc::vec![(Part13b::Rest, Some(data.included_in_return_year()))]
        }
        I::JuryDuty { data } => alloc::vec![(Part13b::Jury, Some(data.received))],
        I::Alimony { data } => alloc::vec![(Part13b::Rest, Some(data.amount()))],
        I::BondRedemption { .. } => alloc::vec![],
        // Final-limit feeds are required for these remaining source kinds.
        I::Partnership { .. }
        | I::Scorp { .. }
        | I::PartnershipActivities { .. }
        | I::ScorpActivities { .. }
        | I::EstateTrustPortfolio { .. }
        | I::PersonalDebt { .. }
        | I::NonbusinessProperty { .. }
        | I::OtherIncome { .. }
        | I::Nec1099 { .. }
        | I::Misc1099 { .. }
        | I::K1099 { .. }
        | I::ForeignEmployment { .. }
        | I::Rental { .. }
        | I::Royalty { .. }
        | I::Cooperative { .. }
        | I::SavingsBonds { .. }
        | I::ChildElection { .. }
        | I::EducationDistribution { .. }
        | I::FarmRental { .. }
        | I::MedicalDistribution { .. } => alloc::vec![(Part13b::Rest, None)],
    }
}

fn weights_13b(
    d: &FederalDraft,
    part: Part13b,
    other_sources: &alloc::collections::BTreeMap<i64, Option<Exact>>,
    accounts: &AccountSources,
) -> Weights {
    let mut w = Weights::default();
    for e in &d.income {
        // Account outputs were already projected at their native boundaries.
        // Exclude only those actual source IDs before adding generic Rest.
        if part == Part13b::Rest && accounts.contains(e.id) {
            continue;
        }
        let parts = if matches!(e.income, FederalIncome::ForeignEmployment { .. }) {
            alloc::vec![(Part13b::Rest, Some(exact::whole(0)))]
        } else if let Some(actual) = other_sources.get(&e.id) {
            alloc::vec![(Part13b::Rest, actual.clone())]
        } else {
            crate::investment_statements::components(e).map_or_else(
                || {
                    parts_13b(&e.income, crate::federal_rules::separate_return(d))
                        .into_iter()
                        .map(|(part, value)| (part, value.map(exact::whole)))
                        .collect()
                },
                |v| {
                    alloc::vec![
                        (Part13b::Interest, Some(exact::whole(v.interest))),
                        (Part13b::Dividends, Some(exact::whole(v.ordinary_dividends))),
                        (Part13b::Capital, Some(exact::whole(v.capital_distributions))),
                    ]
                },
            )
        };
        for (p, amount) in parts {
            if p == part {
                if amount.as_ref().is_some_and(|cents| *cents == exact::whole(0)) {
                    continue;
                }
                let Some((weights, denominator)) = e.accepted_assignment().owner_weights() else {
                    w.unknown = [true; 2];
                    continue;
                };
                if part == Part13b::Rest {
                    // Rest and its native removals retain the original carrier;
                    // independently projected categories use actual ownership.
                    let owner = owner_index(e.owner.owned());
                    match amount {
                        Some(cents) => w.cents[owner] += cents,
                        None => w.unknown[owner] = true,
                    }
                } else {
                    for (owner, weight) in weights.into_iter().enumerate() {
                        if weight == 0 {
                            continue;
                        }
                        match &amount {
                            Some(cents) => {
                                w.cents[owner] += cents
                                    * exact::ratio(weight, denominator)
                                        .expect("accepted ownership denominator is positive");
                            }
                            None => w.unknown[owner] = true,
                        }
                    }
                }
            }
        }
    }
    w
}

fn sole_class_owner(draft: &FederalDraft, part: Part13b) -> Option<usize> {
    let mut present = [false; 2];
    for entry in &draft.income {
        for (class, amount) in
            parts_13b(&entry.income, crate::federal_rules::separate_return(draft))
        {
            if class == part && amount.is_none() {
                return None;
            }
            if class == part && amount != Some(0) {
                let (weights, _) = entry.accepted_assignment().owner_weights()?;
                for (owner, weight) in weights.into_iter().enumerate() {
                    present[owner] |= weight != 0;
                }
            }
        }
    }
    match present {
        [true, false] => Some(0),
        [false, true] => Some(1),
        _ => None,
    }
}

fn documents_by_owner(
    d: &FederalDraft,
    pick: impl Fn(&crate::FederalIncomeEntry) -> i64,
) -> Option<[Exact; 2]> {
    let mut cents = core::array::from_fn(|_| exact::whole(0));
    for entry in &d.income {
        let amount = pick(entry);
        if amount == 0 {
            continue;
        }
        let (weights, denominator) = entry.accepted_assignment().owner_weights()?;
        for (owner, weight) in weights.into_iter().enumerate() {
            cents[owner] += exact::whole(amount) * exact::ratio(weight, denominator)?;
        }
    }
    Some(cents)
}

#[derive(Clone, Copy)]
enum EqualAllocation {
    Educator,
    Ira,
    StudentLoan,
    Adjustment,
    Itemized,
    Charity,
    Nonrefundable,
    Refundable,
}
impl EqualAllocation {
    fn message_key(self) -> &'static str {
        match self {
            Self::Educator => "INJURED_COMPONENT_EDUCATOR",
            Self::Ira => "INJURED_COMPONENT_IRA",
            Self::StudentLoan => "INJURED_COMPONENT_STUDENT_LOAN",
            Self::Adjustment => "INJURED_COMPONENT_ADJUSTMENT",
            Self::Itemized => "INJURED_COMPONENT_ITEMIZED",
            Self::Charity => "INJURED_COMPONENT_CHARITY",
            Self::Nonrefundable => "INJURED_COMPONENT_NONREFUNDABLE",
            Self::Refundable => "INJURED_COMPONENT_REFUNDABLE",
        }
    }
}

#[cfg(test)]
fn part_name(part: Part13b) -> &'static str {
    match part {
        Part13b::Social => "Form 1040 line 6b, taxable Social Security benefits",
        Part13b::Capital => "Form 1040 line 7a, capital gain or loss",
        Part13b::Business => "Schedule 1 lines 3 and 6, business and farm income",
        Part13b::Interest => "Form 1040 line 2b, taxable interest",
        Part13b::Dividends => "Form 1040 line 3b, ordinary dividends",
        Part13b::Jury => "Schedule 1 line 8h, jury pay",
        Part13b::Unemployment => "Schedule 1 line 7, net unemployment",
        Part13b::Rest => "the other income on Form 1040 lines 1b-8",
    }
}

// Calculates Form 8379 Part III and the estimate of the injured spouse's
// share of the joint refund, or refuses what it cannot allocate by name.
pub(crate) fn allocate(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    r: &mut FederalResult,
    a: &InjuredSpouseAllocation,
    f: &JointFacts,
    pre_election_overpayment: i64,
) {
    let d = form_context.draft;
    let refuse = |r: &mut FederalResult, key: &str, #[cfg(test)] message: &str| {
        let context = match key {
            "ISSUE_8379_JOINT_RETURN_REQUIRED" => {
                crate::IssueContext::Section(crate::TaxSection::Setup)
            }
            "ISSUE_8379_SPOUSE_ONLY_DEBT_REQUIRED"
            | "ISSUE_8379_INJURED_SPOUSE_OWES_DEBT"
            | "WARN_8379_COMMUNITY_ESTIMATE_UNAVAILABLE" => {
                crate::IssueContext::Section(crate::TaxSection::InjuredSpouse)
            }
            _ => crate::IssueContext::Section(crate::TaxSection::InjuredSpouse).manual(),
        };
        crate::federal_rules::note_once_scoped(r, Message::new(key, alloc::vec![]), context);
        #[cfg(test)]
        let _ = message;
    };
    let Some(setup) = d.setup.as_ref() else {
        return;
    };
    if setup.filing_status() != FederalStatus::Joint {
        refuse(
            r,
            "ISSUE_8379_JOINT_RETURN_REQUIRED",
            #[cfg(test)]
            "Form 8379 line 2: an injured spouse allocation needs a joint return; you are not an injured spouse.",
        );
        return;
    }
    let mut stop = false;
    if a.debt == OffsetDebt::NoneOfThese {
        refuse(
            r,
            "ISSUE_8379_SPOUSE_ONLY_DEBT_REQUIRED",
            #[cfg(test)]
            "Form 8379 line 3: only a joint refund applied to past-due federal tax, state income tax, state unemployment compensation, child support, spousal support or a federal nontax debt owed by the other spouse alone can be allocated; you are not an injured spouse.",
        );
        stop = true;
    }
    if a.obligated {
        refuse(
            r,
            "ISSUE_8379_INJURED_SPOUSE_OWES_DEBT",
            #[cfg(test)]
            "Form 8379 line 4: an injured spouse who is legally obligated to pay the past-due amount is not an injured spouse. For a federal tax debt you both owe, see innocent spouse relief (Form 8857).",
        );
        stop = true;
    }
    let community = matches!(a.community, InjuredCommunityFacts::Yes { .. });
    if stop {
        return;
    }
    // A structural walk never reruns the native calculation for this fact.
    // Locate only an established final-source need at its actual income topic.
    // This advisory does not invalidate ordinary tax or block incomplete Saves.
    for entry in &d.income {
        if entry.refund_management.is_none() && management_needed(d, r, entry) == Some(true) {
            crate::federal_rules::note_once_scoped(
                r,
                Message::new("INJURED_SPOUSE_WITHHOLDING_MANAGEMENT_NEEDED", alloc::vec![]),
                crate::interview::income_issue_context(entry),
            );
        }
    }
    let inj = owner_index(a.injured);
    let oth = 1 - inj;
    let Some(projection) = owners::project_sources(form_context, r, Some(a), f, None) else {
        if f.accounts.income_joint().is_none() {
            refuse(
                r,
                "WARN_8379_ACCOUNT_OWNER_UNAVAILABLE",
                #[cfg(test)]
                "The native account owner allocation cannot be represented.",
            );
        }
        return;
    };
    let agi = projection.printed_agi();
    let tax = projection.printed_income_taxes();
    let separate = owners::printed_separate_taxes(d, &projection);
    let owners::OwnerProjection {
        l13a,
        l13b,
        l14,
        l15,
        l16,
        l17,
        l18,
        supplemental_qbi,
        plan,
        missing,
        mut problems,
        ..
    } = projection;
    if let Some(qbi) = supplemental_qbi {
        for (column, amount) in [("a", qbi.joint), ("b", qbi.owners[inj]), ("c", qbi.owners[oth])] {
            crate::line_emission::line(
                r,
                &format!("form_8379_supplemental_qbi_{column}"),
                amount,
                &["qbi_deduction", "screening.qbi_history", "income"],
                "https://www.irs.gov/pub/irs-dft/i8379--dft.pdf",
                Message::new("EXPLAIN_FORM8379_QBI_ALLOCATION", alloc::vec![]),
            );
        }
    }
    let repayments = calc(r, "schedule_2_credit_repayments_total");
    let excess = f.excess_social[0] + f.excess_social[1];
    // Line 19: Form 1040 line 25d and excess Social Security by owner.
    let mut l19 = Part::default();
    let medicare_credit = medicare_withholding_projection(d, f, inj);
    let native_withholding = [f1040(r, "25a"), f1040(r, "25b"), f1040(r, "25c")];
    let packet = r.withholding_sources.as_deref();
    let packet_footing = packet.map(|source| {
        source.joint_dollars[0] == native_withholding[0]
            && source.joint_dollars[1] == native_withholding[1]
            && (0..=native_withholding[2]).contains(&source.joint_dollars[2])
    });
    let packet_owners = (packet_footing == Some(true))
        .then(|| packet.and_then(|source| source.projected(d, inj)))
        .flatten();
    for category in 0..3 {
        let (actual, owners) = match packet_footing {
            Some(true) => {
                let actual = packet.unwrap().joint_dollars[category];
                (actual, packet_owners.map(|owners| owners[category]))
            }
            // A present incomplete/mismatched native packet is not a legacy
            // context. Retain the full joint amount without fabricating owners.
            Some(false) => (native_withholding[category], None),
            None => {
                // Old synthetic contexts retain their original checked source
                // projection; no real native packet is replaced by this path.
                let actual = if category == 2 {
                    match &medicare_credit {
                        Some(credit) => native_withholding[2] - credit.targets.iter().sum::<i64>(),
                        None => native_withholding[2],
                    }
                } else {
                    native_withholding[category]
                };
                (
                    actual,
                    crate::kernel::ratio::project_rounded_pair(
                        actual,
                        f.withholding_categories[category],
                        inj,
                    ),
                )
            }
        };
        match owners {
            Some(owners) => l19.add(actual, owners),
            None => {
                l19.unavailable(actual);
                problems.push(AllocationProblem { #[cfg(test)] english: String::from("Form 8379 line 19: a rounded withholding category does not reconcile to its source-owner projection."), display: Message::new("ISSUE_8379_WITHHOLDING_UNTRACED", alloc::vec![]) });
            }
        }
    }
    if packet_footing != Some(false) {
        // Packet25c is Other only. The remaining native25c is reported
        // Medicare credit, not raw per-document withholding precision.
        let remaining = packet.map(|source| native_withholding[2] - source.joint_dollars[2]);
        match medicare_credit {
            Some(MedicareWithholdingProjection { targets, owners })
                if remaining.is_none_or(|actual| targets.iter().sum::<i64>() == actual) =>
            {
                for (target, owners) in targets.into_iter().zip(owners) {
                    match owners {
                        Some(owners) => l19.add(target, owners),
                        None => l19.unavailable(target),
                    }
                }
            }
            _ => match remaining {
                Some(0) => l19.add(0, [0; 2]),
                Some(actual) => l19.unavailable(actual),
                // Legacy base already retains the full native25c when this
                // projection is absent. Its unknown owner remains unavailable.
                None => l19.unavailable(0),
            },
        }
    }
    l19.add(excess, f.excess_social);
    r.injured_spouse = Some(plan);
    // Line 20: Form 1040 line 26 and the extension payment.
    let mut l20 = Part::default();
    let paid = f1040(r, "26") + f.extension;
    use crate::kernel::ratio::exact::{dollars as payment_dollars, whole as payment_whole};
    let mut direct: [[crate::kernel::ratio::exact::Exact; 2]; 2] =
        core::array::from_fn(|_| core::array::from_fn(|_| payment_whole(0)));
    if let Some(owners) = &f.income_categories.estate_estimated_cents_by_owner {
        direct[0] = owners.clone();
    }
    let mut joint_by_category = [payment_whole(0), payment_whole(0)];
    let mut payment_complete = true;
    let mut payment_funding = refund_funding::PaymentFunding {
        estate_cents: f.income_categories.estate_estimated_cents_by_owner.clone(),
        ..Default::default()
    };
    if let Some(pay) = &d.payments {
        for (amount, payer, _) in pay.estimated.funding() {
            match payer {
                crate::PaymentPayer::Taxpayer => direct[0][0] += payment_whole(amount),
                crate::PaymentPayer::Spouse => direct[0][1] += payment_whole(amount),
                crate::PaymentPayer::Joint => joint_by_category[0] += payment_whole(amount),
            }
        }
        match pay.extension_payer {
            Some(crate::PaymentPayer::Taxpayer) => direct[1][0] += payment_whole(pay.extension),
            Some(crate::PaymentPayer::Spouse) => direct[1][1] += payment_whole(pay.extension),
            Some(crate::PaymentPayer::Joint) => {
                joint_by_category[1] += payment_whole(pay.extension)
            }
            None if pay.extension != 0 => payment_complete = false,
            None => {}
        }
    }
    let joint = &joint_by_category[0] + &joint_by_category[1];
    // These nonnegative Dollars are subsets of the separately rounded native
    // categories. The checked complete payment output bounds their sums.
    let mut payments = [0i64; 2];
    let mut joint_paid = 0;
    // Form 1040 line26 and Schedule3 extension payments round separately.
    // Preserve each printed category before adding its owner columns.
    for (category, actual) in [f1040(r, "26"), f.extension].iter().enumerate() {
        let exact_direct = &direct[category][0] + &direct[category][1];
        let Some(direct_total) = payment_dollars(&exact_direct) else {
            payment_complete = false;
            continue;
        };
        if payment_dollars(&(&exact_direct + &joint_by_category[category])) != Some(*actual) {
            payment_complete = false;
        }
        if let Some(owners) = crate::kernel::ratio::exact::project_rounded_pair(
            direct_total,
            &exact_direct,
            direct[category].clone(),
            inj,
        ) {
            payments[0] += owners[0];
            payments[1] += owners[1];
            payment_funding.direct_dollars[category] = owners;
        } else {
            payment_complete = false;
        }
        joint_paid += actual - direct_total;
        payment_funding.joint_dollars[category] = actual - direct_total;
    }
    if !payment_complete || joint_paid < 0 {
        l20.complete = false;
        problems.push(AllocationProblem { #[cfg(test)] english: String::from("Form 8379 line20: every printed payment category needs its accepted payer; estate-allocated payments need their owning allocation feed."), display: Message::new("WARN_8379_PAYMENT_OWNER_UNAVAILABLE", alloc::vec![]) });
    } else {
        let joint_owners = match (a.estimated, separate) {
            (EstimatedSplit::Agreed { injured }, _)
                if injured >= 0 && payment_whole(injured) <= joint =>
            {
                let mut owners = [0; 2];
                owners[inj] = phase::cap(dollars(injured), [joint_paid]);
                owners[oth] = joint_paid - owners[inj];
                owners
            }
            (EstimatedSplit::IrsFormula, _) if joint == payment_whole(0) => [0; 2],
            (EstimatedSplit::IrsFormula, Some(separate)) => {
                crate::kernel::ratio::native_fraction_pair(joint_paid, separate, 0)
            }
            _ => {
                l20.complete = false;
                problems.push(AllocationProblem { #[cfg(test)] english: String::from("Form 8379 joint estimated-payment ratio is undefined; enter the spouses' agreed joint-payment allocation."), display: Message::new("WARN_8379_JOINT_PAYMENT_SPLIT_NEEDED", alloc::vec![]) });
                [0; 2]
            }
        };
        payments[0] += joint_owners[0];
        payments[1] += joint_owners[1];
    }
    l20.add(paid, payments);

    let parts = [
        (
            "13a",
            [
                "EXPLAIN_FORM8379_WAGES_JOINT",
                "EXPLAIN_FORM8379_WAGES_INJURED",
                "EXPLAIN_FORM8379_WAGES_OTHER",
            ],
            l13a,
        ),
        (
            "13b",
            [
                "EXPLAIN_FORM8379_OTHER_INCOME_JOINT",
                "EXPLAIN_FORM8379_OTHER_INCOME_INJURED",
                "EXPLAIN_FORM8379_OTHER_INCOME_OTHER",
            ],
            l13b,
        ),
        (
            "14",
            [
                "EXPLAIN_FORM8379_ADJUSTMENTS_JOINT",
                "EXPLAIN_FORM8379_ADJUSTMENTS_INJURED",
                "EXPLAIN_FORM8379_ADJUSTMENTS_OTHER",
            ],
            l14,
        ),
        (
            "15",
            [
                "EXPLAIN_FORM8379_DEDUCTIONS_JOINT",
                "EXPLAIN_FORM8379_DEDUCTIONS_INJURED",
                "EXPLAIN_FORM8379_DEDUCTIONS_OTHER",
            ],
            l15,
        ),
        (
            "16",
            [
                "EXPLAIN_FORM8379_NONREFUNDABLE_JOINT",
                "EXPLAIN_FORM8379_NONREFUNDABLE_INJURED",
                "EXPLAIN_FORM8379_NONREFUNDABLE_OTHER",
            ],
            l16,
        ),
        (
            "17",
            [
                "EXPLAIN_FORM8379_REFUNDABLE_JOINT",
                "EXPLAIN_FORM8379_REFUNDABLE_INJURED",
                "EXPLAIN_FORM8379_REFUNDABLE_OTHER",
            ],
            l17,
        ),
        (
            "18",
            [
                "EXPLAIN_FORM8379_OTHER_TAX_JOINT",
                "EXPLAIN_FORM8379_OTHER_TAX_INJURED",
                "EXPLAIN_FORM8379_OTHER_TAX_OTHER",
            ],
            l18,
        ),
        (
            "19",
            [
                "EXPLAIN_FORM8379_WITHHOLDING_JOINT",
                "EXPLAIN_FORM8379_WITHHOLDING_INJURED",
                "EXPLAIN_FORM8379_WITHHOLDING_OTHER",
            ],
            l19,
        ),
        (
            "20",
            [
                "EXPLAIN_FORM8379_PAYMENTS_JOINT",
                "EXPLAIN_FORM8379_PAYMENTS_INJURED",
                "EXPLAIN_FORM8379_PAYMENTS_OTHER",
            ],
            l20,
        ),
    ];
    for (line, keys, part) in parts {
        // A known native total cannot stand in for an unresolved W-2 versus
        // other-income class target. Preserve the ordinary return, and omit
        // these printed rows until their joint source classes are determined.
        if matches!(line, "13a" | "13b") && f.income_categories.source_class_unavailable {
            continue;
        }
        for (column, amount, key) in [
            ("a", part.joint, keys[0]),
            ("b", part.owners[inj], keys[1]),
            ("c", part.owners[oth], keys[2]),
        ] {
            if column != "a" && !part.complete {
                continue;
            }
            form_line(
                r,
                FormLine::new(TaxForm::Form8379, &format!("{line}({column})")),
                &format!("form_8379_line_{line}_{column}"),
                amount,
                &["injured_spouse", "income", "payments"],
                "https://www.irs.gov/instructions/i8379",
                Message::new(key, alloc::vec![]),
            );
        }
    }

    if let Some(plan) = r.injured_spouse.as_mut() {
        for (line, key) in [
            ("13a", "EXPLAIN_FORM8379_WAGES_JOINT"),
            ("13b", "EXPLAIN_FORM8379_OTHER_INCOME_JOINT"),
            ("14", "EXPLAIN_FORM8379_ADJUSTMENTS_JOINT"),
            ("15", "EXPLAIN_FORM8379_DEDUCTIONS_JOINT"),
            ("16", "EXPLAIN_FORM8379_NONREFUNDABLE_JOINT"),
            ("17", "EXPLAIN_FORM8379_REFUNDABLE_JOINT"),
            ("18", "EXPLAIN_FORM8379_OTHER_TAX_JOINT"),
            ("19", "EXPLAIN_FORM8379_WITHHOLDING_JOINT"),
            ("20", "EXPLAIN_FORM8379_PAYMENTS_JOINT"),
        ] {
            let part = match line {
                "13a" => l13a,
                "13b" => l13b,
                "14" => l14,
                "15" => l15,
                "16" => l16,
                "17" => l17,
                "18" => l18,
                "19" => l19,
                _ => l20,
            };
            if !part.complete {
                continue;
            }
            plan.known.push(InjuredPrintedRow {
                label: Message::new(key, alloc::vec![]).encode(),
                joint: part.joint * 100,
                injured: part.owners[inj] * 100,
                other: part.owners[oth] * 100,
            });
        }
    }
    let farmland_offset = farmland_installment_offset(r);
    if farmland_offset != 0 {
        refuse(
            r,
            "WARN_8379_FARMLAND_ESTIMATE_UNAVAILABLE",
            #[cfg(test)]
            "The Form 1062 tax deferral remains in the ordinary estimate; its combined Form 8379 refund allocation is unavailable. Part III remains available.",
        );
    }
    if missing {
        crate::federal_rules::note_once_scoped(
            r,
            Message::new("WARN_8379_ALLOCATION_CHOICES_NEEDED", alloc::vec![]),
            crate::IssueContext::Section(crate::TaxSection::InjuredSpouse),
        );
        return;
    }
    if !problems.is_empty() {
        for problem in problems {
            crate::federal_rules::note_once_scoped(
                r,
                problem.display,
                crate::IssueContext::Section(crate::TaxSection::InjuredSpouse).manual(),
            );
            #[cfg(test)]
            let _ = problem.english;
        }
        return;
    }

    if farmland_offset != 0 {
        return;
    }

    // IRM 21.4.6.5.6.2: the EIC by each spouse's separate EIC.
    let eic = f1040(r, "27a");
    let mut eic_percent = 0;
    if eic > 0 {
        let separate_eic = [0, 1].map(|i| {
            crate::forms::schedule_eic::eic_amount(f.earned[i], agi[i], f.eic_children, true)
        });
        let sum = separate_eic[0] + separate_eic[1];
        if sum > 0 {
            eic_percent = fraction(10000, separate_eic[inj], sum);
        } else {
            let losses = [0, 1].map(|i| agi[i] < 0);
            match losses {
                [true, false] | [false, true] => {
                    eic_percent = if losses[inj] { 10000 } else { 0 };
                }
                _ => {
                    refuse(
                        r,
                        "ISSUE_8379_EIC_UNATTRIBUTABLE",
                        #[cfg(test)]
                        "Form 8379: the earned income credit on Form 1040 line 27a cannot be allocated, because neither spouse's separate earned income or loss explains it.",
                    );
                    return;
                }
            }
        }
    }
    let eic_cents = fraction(eic * 100, eic_percent, 10000);
    // Credit elections follow offsets; allocation uses the finalized overpayment
    // before the planned line 36 split (IRM 21.4.6.5.2).
    let refund = pre_election_overpayment;

    let estimate = if community {
        let Some(share) =
            community_refund(form_context, r, a, &payment_funding, refund, eic * 100, eic_percent)
        else {
            refuse(
                r,
                "WARN_8379_COMMUNITY_ESTIMATE_UNAVAILABLE",
                #[cfg(test)]
                "The Form 8379 refund estimate needs a verified governing state, debt regime and complete payment property sources. Your ordinary federal estimate remains available.",
            );
            return;
        };
        share
    } else {
        // Lines 6-9 of Part I, from the allocation.
        let line_6 = l19.owners[inj] > 0 || l20.owners[inj] > 0;
        let line_7 = f.earned[inj] > 0;
        let line_8 = line_7 && (eic > 0 || f1040(r, "28") > 0);
        let line_9 = l17.owners[inj] > 0;
        if !(line_6 || line_8 || line_9) {
            refuse(
                r,
                "ISSUE_8379_NO_ELIGIBLE_REFUND_SHARE",
                #[cfg(test)]
                "Form 8379 lines 6-9: the injured spouse made no payments, had no withholding, claims no earned income credit or additional child tax credit and no refundable credit; you are not an injured spouse.",
            );
            return;
        }
        // IRM 21.4.6.5.6.1: the injured spouse's percentage to four
        // decimals, times the joint tax before credits and other taxes.
        let percent =
            if tax[0] + tax[1] == 0 { 5000 } else { fraction(10000, tax[inj], tax[0] + tax[1]) };
        let share = fraction((f1040(r, "18") - repayments) * 100, percent, 10000);
        // IRM 21.4.6.5.6: with no base tax after nonrefundable credits,
        // every nonrefundable credit goes to the injured spouse.
        let credits = if f1040(r, "22") == 0 { f1040(r, "21") } else { l16.owners[inj] };
        let liability = positive(share - credits * 100) + l18.owners[inj] * 100;
        let contribution = (l19.owners[inj] + l20.owners[inj] + l17.owners[inj]) * 100 + eic_cents;
        line_cents(
            r,
            "form_8379_injured_share_of_tax",
            share,
            &["form_8379_line_13a_b", "form_8379_line_15_b", "income_tax"],
            "https://www.irs.gov/irm/part21/irm_21-004-006r",
            Message::new(
                "EXPLAIN_FORM_N_INJURED_SHARE_OF_TAX",
                alloc::vec![
                    MessageArg::Decimal { units: percent, scale: 2 },
                    MessageArg::usd_dollars(tax[inj]),
                    MessageArg::usd_dollars(tax[oth]),
                ],
            ),
        );
        line_cents(
            r,
            "form_8379_injured_liability",
            liability,
            &["form_8379_injured_share_of_tax", "form_8379_line_16_b", "form_8379_line_18_b"],
            "https://www.irs.gov/irm/part21/irm_21-004-006r",
            msg!("EXPLAIN_FORM_N_INJURED_LIABILITY"),
        );
        line_cents(
            r,
            "form_8379_injured_contribution",
            contribution,
            &["form_8379_line_17_b", "form_8379_line_19_b", "form_8379_line_20_b"],
            "https://www.irs.gov/irm/part21/irm_21-004-006r",
            msg!("EXPLAIN_FORM_N_INJURED_CONTRIBUTION"),
        );
        phase::cap(refund, [positive(contribution - liability)])
    };
    line_cents(
        r,
        "form_8379_injured_eic",
        eic_cents,
        &["earned_income_credit"],
        "https://www.irs.gov/irm/part21/irm_21-004-006r",
        msg!("EXPLAIN_FORM_N_INJURED_EIC"),
    );
    line_cents(
        r,
        ESTIMATE_LINE,
        estimate,
        &["form_8379_line_13a_a", "refund"],
        "https://www.irs.gov/irm/part21/irm_21-004-006r",
        msg!("EXPLAIN_INJURED_SPOUSE"),
    );
    #[cfg(test)]
    let note = {
        let note = format!(
            "Form 8379 estimate: the injured spouse's share of the joint refund is about ${}. The IRS makes the final computation after you file: {}",
            dollars(estimate),
            if community {
                "in a community property state it applies the state's rules; for a debt other than federal tax, half the refund other than the earned income credit, plus the injured spouse's part of the earned income credit, is generally the injured spouse's."
            } else {
                "it figures each spouse's married filing separately tax from this allocation, shares the joint tax in that proportion, and refunds the injured spouse's withholding, payments and credits less their share."
            }
        );
        note
    };
    r.review_notes.push(crate::federal_model::ReviewNote::localized(
        #[cfg(test)]
        note,
        Message::new(
            if community {
                "NOTE_INJURED_SPOUSE_COMMUNITY_ESTIMATE"
            } else {
                "NOTE_INJURED_SPOUSE_IRS_ESTIMATE"
            },
            alloc::vec![MessageArg::usd_cents(estimate)],
        ),
    ));
}

impl InjuredCommunityFacts {
    pub(crate) fn page(&self) -> InjuredCommunity {
        match self {
            Self::No => InjuredCommunity::No,
            Self::Yes { states, debt_facts } => {
                InjuredCommunity::Yes { states: states.clone(), debt_facts: *debt_facts }
            }
        }
    }
    pub(crate) fn texas(&self) -> bool {
        matches!(self, Self::Yes { states, .. } if states.contains(&7))
    }
}
impl InjuredCommunity {
    fn accepted(&self) -> InjuredCommunityFacts {
        match self {
            Self::No => InjuredCommunityFacts::No,
            Self::Yes { states, debt_facts } => {
                InjuredCommunityFacts::Yes { states: states.clone(), debt_facts: *debt_facts }
            }
        }
    }
}
impl InjuredSpouseAllocation {
    pub(crate) fn from_page(value: &InjuredSpouse) -> Self {
        Self {
            injured: value.injured,
            debt: value.debt,
            obligated: value.obligated,
            community: value.community.accepted(),
            estimated: value.estimated,
            student_loan_borrower: value.student_loan_borrower,
            allocations: value
                .allocation_rows()
                .map(|a| InjuredAmount { component: a.component.clone(), injured: a.injured })
                .collect(),
            claimants: value
                .claimants
                .iter()
                .map(|c| InjuredClaimantFacts { person: c.person, claimant: c.claimant })
                .collect(),
        }
    }
    pub(crate) fn defaults() -> Self {
        Self {
            injured: Owner::Taxpayer,
            debt: OffsetDebt::FederalNontax,
            obligated: false,
            community: InjuredCommunityFacts::No,
            estimated: EstimatedSplit::IrsFormula,
            student_loan_borrower: None,
            allocations: Vec::new(),
            claimants: Vec::new(),
        }
    }
    // Call only after allocation validation. Partitioning the editor into new
    // and answered rows must not change the canonical stored component order.
    pub(crate) fn order_allocations(&mut self, result: &FederalResult) {
        let required =
            result.injured_spouse.as_ref().map(|plan| plan.required.as_slice()).unwrap_or(&[]);
        self.allocations.sort_by_key(|answer| {
            required
                .iter()
                .position(|row| row.component == answer.component)
                .expect("validated allocation component")
        });
    }
    pub(crate) fn page(&self, draft: &FederalDraft, result: &FederalResult) -> InjuredSpouse {
        let plan = result.injured_spouse.as_ref();
        let allocations = plan.map_or_else(Vec::new, |p| p.required.clone());
        let components = allocations
            .iter()
            .map(|a| InjuredComponentChoice { id: a.component.clone(), label: a.label.clone() })
            .collect();
        let people = draft
            .dependents
            .iter()
            .map(|p| PersonChoice { id: p.id, label: p.name(), valid_ssn: p.valid_ssn })
            .collect();
        let claimants = draft
            .dependents
            .iter()
            .map(|p| InjuredClaimant {
                person: p.id,
                claimant: self
                    .claimants
                    .iter()
                    .find(|c| c.person == p.id)
                    .map_or(Owner::Taxpayer, |c| c.claimant),
            })
            .collect();
        InjuredSpouse {
            injured: self.injured,
            debt: self.debt,
            obligated: self.obligated,
            community: self.community.page(),
            estimated: self.estimated,
            student_loan_borrower: self.student_loan_borrower,
            student_loan_deduction: calc(result, "schedule_1_student_loan_interest") * 100,
            allocations,
            new_allocations: Vec::new(),
            claimants,
            joint_estimated: crate::kernel::ratio::exact::rounded(&joint_estimated(draft)),
            states: state_choices(),
            people,
            components,
            allowed_debt_facts: allowed_debt_facts(&self.community, self.debt),
            known_child_credit_injured: plan.map_or(0, |p| p.known_child[0]),
            known_child_credit_other: plan.map_or(0, |p| p.known_child[1]),
            child_credit_cap_injured: plan.map_or(0, |p| p.child_caps[0]),
            child_credit_cap_other: plan.map_or(0, |p| p.child_caps[1]),
            known: plan.map_or_else(Vec::new, |p| p.known.clone()),
            estimated_share: result.lines.iter().find(|l| l.id == ESTIMATE_LINE).map(|l| l.amount),
            explanation: crate::common::Message::new(
                "EXPLAIN_INJURED_SPOUSE_LIMITS",
                alloc::vec![],
            )
            .encode(),
        }
    }
}
impl InjuredSpouse {
    // Both service-owned arrays ask once for each component. Only complete
    // submitted values reach this iterator and the unchanged stored vector.
    pub(crate) fn allocation_rows(&self) -> impl Iterator<Item = &InjuredAmountPage> {
        self.allocations.iter().chain(&self.new_allocations)
    }
}
pub(crate) use crate::return_carriers::AllocationPlan;

fn state_choices() -> Vec<InjuredStateChoice> {
    [
        "ARIZONA",
        "CALIFORNIA",
        "IDAHO",
        "LOUISIANA",
        "NEVADA",
        "NEW_MEXICO",
        "TEXAS",
        "WASHINGTON",
        "WISCONSIN",
    ]
    .iter()
    .enumerate()
    .map(|(i, state)| InjuredStateChoice {
        id: (i + 1) as u16,
        label: Message::new(format!("INJURED_STATE_{state}"), alloc::vec![]).encode(),
    })
    .collect()
}
fn debt_fact_index(value: InjuredDebtFacts) -> i32 {
    match value {
        InjuredDebtFacts::NotApplicable => {
            libertas_macros::variant_index!(InjuredDebtFacts::NotApplicable) as i32
        }
        InjuredDebtFacts::BeforeMarriage => {
            libertas_macros::variant_index!(InjuredDebtFacts::BeforeMarriage) as i32
        }
        InjuredDebtFacts::DuringMarriageNotFamily => {
            libertas_macros::variant_index!(InjuredDebtFacts::DuringMarriageNotFamily) as i32
        }
        InjuredDebtFacts::DuringMarriageFamily => {
            libertas_macros::variant_index!(InjuredDebtFacts::DuringMarriageFamily) as i32
        }
    }
}
fn allowed_debt_facts(community: &InjuredCommunityFacts, debt: OffsetDebt) -> Vec<i32> {
    if debt == OffsetDebt::FederalTax && matches!(community, InjuredCommunityFacts::Yes { .. }) {
        [
            InjuredDebtFacts::BeforeMarriage,
            InjuredDebtFacts::DuringMarriageNotFamily,
            InjuredDebtFacts::DuringMarriageFamily,
        ]
        .map(debt_fact_index)
        .to_vec()
    } else {
        alloc::vec![debt_fact_index(InjuredDebtFacts::NotApplicable)]
    }
}
pub(crate) fn texas(draft: &FederalDraft) -> bool {
    draft.injured_spouse.as_ref().is_some_and(|a| a.community.texas())
}

struct AllocationBounds {
    minimum: i64,
    maximum: i64,
    initial: Option<i64>,
}
impl From<(i64, i64)> for AllocationBounds {
    fn from((minimum, maximum): (i64, i64)) -> Self {
        Self { minimum, maximum, initial: None }
    }
}

fn shared_amount(
    plan: &mut AllocationPlan,
    a: &SourceSelections,
    key: &str,
    label: &str,
    joint: i64,
    bounds: impl Into<AllocationBounds>,
    missing: &mut bool,
) -> Option<[i64; 2]> {
    let cents = shared_amount_cents(plan, a, key, label, joint * 100, bounds, missing)?;
    // Existing annual-Dollar callers round the selected injured owner first
    // and give the other owner the exact native joint-Dollar remainder.
    Some(crate::kernel::ratio::native_project_pair(joint, cents, owner_index(a.injured)))
}

// The same existing bounded source choice, before its caller's reporting
// boundary. Selection and request metadata use Cents; an annual-Dollar
// caller expresses its reported joint amount in Cents without claiming source
// precision. Raw source callers retain their actual Cents until reporting.
fn shared_amount_cents(
    plan: &mut AllocationPlan,
    a: &SourceSelections,
    key: &str,
    label: &str,
    joint: i64,
    bounds: impl Into<AllocationBounds>,
    missing: &mut bool,
) -> Option<[i64; 2]> {
    let AllocationBounds { minimum, maximum, initial } = bounds.into();
    if joint == 0 {
        return Some([0, 0]);
    }
    let choice = a.allocations.iter().filter(|v| v.component == key).collect::<Vec<_>>();
    let answer = match choice.as_slice() {
        [value] if (minimum..=maximum).contains(&value.injured) => Some(value.injured),
        [] => initial
            .or_else(|| (minimum == maximum).then_some(minimum))
            .filter(|value| (minimum..=maximum).contains(value)),
        _ => None,
    };
    let selected = if let Some(answer) = answer {
        plan.accepted.push(InjuredAmount { component: String::from(key), injured: answer });
        answer
    } else {
        *missing = true;
        // Internal incomplete-plan arithmetic only. Construction never sends
        // this bound as an answer; the required control starts unanswered.
        minimum
    };
    plan.required.push(InjuredAmountPage {
        component: String::from(key),
        label: Message::new(label, alloc::vec![]).encode(),
        joint,
        minimum,
        maximum,
        injured: selected,
    });
    crate::kernel::attribution::accepted_interval(joint, (minimum, maximum), a.injured, answer?)
}
fn child_pool(
    plan: &mut AllocationPlan,
    a: &SourceSelections,
    eligible: &[(i64, i64)],
    joint: i64,
    key: &str,
    missing: &mut bool,
) -> Option<[i64; 2]> {
    let mut claimant_missing = false;
    let mut counts = [0i64; 2];
    let mut caps = [0i64; 2];
    for (id, cap) in eligible {
        match a.claimants.iter().filter(|c| c.person == *id).collect::<Vec<_>>().as_slice() {
            [claimant] => {
                let owner = owner_index(claimant.claimant);
                counts[owner] += 1;
                caps[owner] += cap;
            }
            _ => {
                *missing = true;
                claimant_missing = true;
            }
        }
    }
    if joint == 0 {
        return Some([0; 2]);
    }
    if claimant_missing {
        return None;
    }
    let inj = owner_index(a.injured);
    let other = 1 - inj;
    if counts[0] == 0 || counts[1] == 0 {
        let owner = if counts[1] > 0 { Owner::Spouse } else { Owner::Taxpayer };
        let owners = crate::kernel::attribution::exclusive(joint, owner);
        plan.known_child[0] += owners[inj] * 100;
        plan.known_child[1] += owners[other] * 100;
        Some(owners)
    } else {
        let (minimum, maximum) =
            crate::kernel::attribution::native_capacity_interval(joint, caps, a.injured);
        shared_amount(
            plan,
            a,
            key,
            if key == "actc_pool" { "INJURED_COMPONENT_ACTC" } else { "INJURED_COMPONENT_CTC_ODC" },
            joint,
            (minimum * 100, maximum * 100),
            missing,
        )
    }
}

pub(crate) fn facts_issue(
    value: &InjuredSpouse,
    draft: &FederalDraft,
    result: &FederalResult,
) -> Option<ValidationIssue> {
    if let Some(issue) = issue(value, &joint_estimated(draft)) {
        return Some(issue);
    }
    if calc(result, "schedule_1_student_loan_interest") != 0
        && value.student_loan_borrower.is_none()
    {
        return Some(
            ValidationIssue::new("INJURED_SPOUSE_LOAN_BORROWER_REQUIRED")
                .at_field("student_loan_borrower"),
        );
    }
    if value.claimants.len() != draft.dependents.len() || value.claimants.len() > 100 {
        return Some(
            ValidationIssue::new("INJURED_SPOUSE_CLAIMANTS_NOT_CURRENT").at_field("claimants"),
        );
    }
    if let Some((i, _)) = value.claimants.iter().enumerate().find(|(i, c)| {
        !draft.dependents.iter().any(|p| p.id == c.person)
            || value.claimants[..*i].iter().any(|p| p.person == c.person)
    }) {
        return Some(
            ValidationIssue::new("INJURED_SPOUSE_CLAIMANTS_NOT_CURRENT")
                .at_field("person")
                .in_list("claimants", i),
        );
    }
    if let InjuredCommunity::Yes { states, .. } = &value.community {
        if states.is_empty() || states.len() > 9 {
            return Some(
                ValidationIssue::new("INJURED_SPOUSE_COMMUNITY_FACTS_NOT_CURRENT")
                    .at_field("states")
                    .in_field("community"),
            );
        }
        if let Some((i, _)) = states
            .iter()
            .enumerate()
            .find(|(i, state)| !(1..=9).contains(*state) || states[..*i].contains(state))
        {
            return Some(
                ValidationIssue::new("INJURED_SPOUSE_COMMUNITY_FACTS_NOT_CURRENT")
                    .at_list_item("states", i)
                    .in_field("community"),
            );
        }
    }
    None
}
// The debt facts must be among the choices for the debt and community answers.
// A Save checks this after it has refreshed a stale choice list, so a person
// who just chose a federal tax debt receives the new choices, not a refusal.
pub(crate) fn debt_facts_issue(value: &InjuredSpouse) -> Option<ValidationIssue> {
    let InjuredCommunity::Yes { debt_facts, .. } = &value.community else {
        return None;
    };
    (!allowed_debt_facts(&value.community.accepted(), value.debt)
        .contains(&debt_fact_index(*debt_facts)))
    .then(|| {
        ValidationIssue::new("INJURED_SPOUSE_COMMUNITY_FACTS_NOT_CURRENT")
            .at_field("debt_facts")
            .in_field("community")
    })
}
pub(crate) fn allocation_issue(
    value: &InjuredSpouse,
    draft: &FederalDraft,
    result: &FederalResult,
) -> Option<ValidationIssue> {
    if let Some(issue) = facts_issue(value, draft, result) {
        return Some(issue);
    }
    let required = result.injured_spouse.as_ref().map(|p| p.required.as_slice()).unwrap_or(&[]);
    let rows: Vec<_> = value
        .allocations
        .iter()
        .enumerate()
        .map(|(index, row)| ("allocations", index, row))
        .chain(
            value
                .new_allocations
                .iter()
                .enumerate()
                .map(|(index, row)| ("new_allocations", index, row)),
        )
        .collect();
    let field = if value.new_allocations.is_empty() { "allocations" } else { "new_allocations" };
    if rows.len() != required.len() {
        return Some(
            ValidationIssue::new("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT").at_field(field),
        );
    }
    for (position, (list, index, a)) in rows.iter().enumerate() {
        if rows[..position].iter().any(|(_, _, previous)| previous.component == a.component) {
            return Some(
                ValidationIssue::new("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT")
                    .at_field("component")
                    .in_list(list, *index),
            );
        }
        let Some(row) = required.iter().find(|v| v.component == a.component) else {
            return Some(
                ValidationIssue::new("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT")
                    .at_field("component")
                    .in_list(list, *index),
            );
        };
        if !(row.minimum..=row.maximum).contains(&a.injured) {
            return Some(
                ValidationIssue::new("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT")
                    .at_field("injured")
                    .in_list(list, *index),
            );
        }
    }
    if let Some(plan) = &result.injured_spouse {
        let mut total = plan.known_child;
        for row in &plan.required {
            if row.component == "ctc_odc_pool" || row.component == "actc_pool" {
                let Some(amount) = value
                    .allocation_rows()
                    .find(|a| a.component == row.component)
                    .map(|a| a.injured)
                else {
                    return Some(
                        ValidationIssue::new("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT")
                            .at_field(field),
                    );
                };
                total[0] += amount;
                total[1] += row.joint - amount;
            }
        }
        if [0, 1].iter().any(|i| total[*i] > plan.child_caps[*i]) {
            return Some(
                ValidationIssue::new("INJURED_SPOUSE_COMBINED_CHILD_CREDIT_ABOVE_CAP")
                    .at_field(field),
            );
        }
    }
    None
}
#[cfg(test)]
pub(crate) fn allocation_rule(
    value: &InjuredSpouse,
    draft: &FederalDraft,
    result: &FederalResult,
) -> Result<(), &'static str> {
    allocation_issue(value, draft, result).map_or(Ok(()), |issue| Err(issue.key))
}

pub(crate) fn community_answers(draft: &FederalDraft) -> Vec<i32> {
    if draft.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Joint)
        && draft
            .injured_spouse
            .as_ref()
            .is_some_and(|a| matches!(a.community, InjuredCommunityFacts::Yes { .. }))
    {
        [
            crate::CommunityShare::Community,
            crate::CommunityShare::Separate,
            crate::CommunityShare::Partly { percent: 50 },
        ]
        .map(crate::community::index)
        .to_vec()
    } else {
        crate::community::allowed_answers(draft.setup.as_ref())
    }
}
pub(crate) fn community_row_issue(
    draft: &FederalDraft,
    answer: crate::CommunityShare,
) -> Option<crate::validation::ValidationIssue> {
    if community_answers(draft) == crate::community::allowed_answers(draft.setup.as_ref()) {
        return crate::community::row_issue(draft.setup.as_ref(), answer);
    }
    if !community_answers(draft).contains(&crate::community::index(answer)) {
        return Some(
            crate::validation::ValidationIssue::new("COMMUNITY_ANSWER_NOT_ALLOWED").at_record(),
        );
    }
    if matches!(answer, crate::CommunityShare::Partly { percent } if !(1..=99).contains(&percent)) {
        return Some(
            crate::validation::ValidationIssue::new("COMMUNITY_PERCENT_OUT_OF_RANGE")
                .at_field("percent"),
        );
    }
    None
}
pub(crate) fn management_needed(
    draft: &FederalDraft,
    result: &FederalResult,
    entry: &crate::FederalIncomeEntry,
) -> Option<bool> {
    refund_funding::management_needed(draft, result, entry)
}

#[derive(Default)]
struct RefundFunds {
    separate: [crate::kernel::ratio::exact::Exact; 2],
    community: crate::kernel::ratio::exact::Exact,
    liable_earned: crate::kernel::ratio::exact::Exact,
    liable_managed: crate::kernel::ratio::exact::Exact,
}
impl RefundFunds {
    fn source(
        &mut self,
        amount: crate::kernel::ratio::exact::Exact,
        community: crate::kernel::ratio::exact::Exact,
        owner: Owner,
        allocation: (Owner, u16),
        management: Option<crate::RefundManagement>,
        earned: bool,
    ) -> Option<()> {
        let (injured, state) = allocation;
        use crate::kernel::ratio::exact::whole;
        if amount < whole(0) || community < whole(0) || community > amount {
            return None;
        }
        self.separate[owner_index(owner)] += amount - &community;
        self.community += &community;
        if earned && owner != injured {
            self.liable_earned += &community;
        }
        if state == 7 && community > whole(0) {
            match management? {
                crate::RefundManagement::Joint => self.liable_managed += &community,
                crate::RefundManagement::Taxpayer if injured == Owner::Spouse => {
                    self.liable_managed += &community
                }
                crate::RefundManagement::Spouse if injured == Owner::Taxpayer => {
                    self.liable_managed += &community
                }
                crate::RefundManagement::Taxpayer | crate::RefundManagement::Spouse => {}
                crate::RefundManagement::NotApplicable => return None,
            }
        }
        Some(())
    }
}
fn community_refund(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    r: &FederalResult,
    a: &InjuredSpouseAllocation,
    payment_funding: &refund_funding::PaymentFunding,
    refund: i64,
    eic: i64,
    eic_percent: i64,
) -> Option<i64> {
    let d = form_context.draft;
    let InjuredCommunityFacts::Yes { states, debt_facts } = &a.community else {
        return None;
    };
    let [state] = states.as_slice() else {
        return None;
    };
    // A credit's allocated person does not establish the property of its
    // refundable funding. Only EIC has its own verified treatment below;
    // other positive refundable credits must not ride cash-property weights.
    if r.refundable_credits?.checked_sub(eic)? != 0 {
        return None;
    }
    let mut funds = RefundFunds::default();
    let earned_needed = a.debt == OffsetDebt::FederalTax
        && matches!(*state, 1 | 9)
        && *debt_facts == InjuredDebtFacts::BeforeMarriage;
    refund_funding::add_withholding(form_context, r, a.injured, *state, earned_needed, &mut funds)?;
    refund_funding::add_payments(d, payment_funding, a.injured, *state, &mut funds)?;
    use crate::kernel::ratio::exact::{ratio, rounded, whole};
    let all = &funds.community + &funds.separate[0] + &funds.separate[1];
    if all == whole(0) {
        return (refund == 0).then_some(0);
    }
    // These are reported category payment weights. Untraced credits cannot
    // be attributed to community funds or repaired by flattening raw cash.
    if all != whole(r.payments?) {
        return None;
    }
    let inj = owner_index(a.injured);
    let non_eic = positive(refund - eic);
    let eic_refund = refund - non_eic;
    if a.debt != OffsetDebt::FederalTax {
        // Double the source units so a half cent is not discarded before
        // the final refund share is rounded to cents.
        let injured_non_eic = whole(non_eic) * (&funds.separate[inj] * whole(2) + &funds.community)
            / (&all * whole(2));
        let protected = injured_non_eic + whole(eic_refund) * ratio(eic_percent, 10000)?;
        return Some(phase::cap(refund, [rounded(&protected)?]));
    }
    // Published RR2004-71 through74 examples contain no EIC. Broader federal
    // tax-debt/EIC property reach requires its owning collection-law source.
    if eic > 0 {
        return None;
    }
    let protected_community = match (*state, *debt_facts) {
        (1 | 9, InjuredDebtFacts::BeforeMarriage) => &funds.community - &funds.liable_earned,
        (9, InjuredDebtFacts::DuringMarriageNotFamily) => funds.community.clone(),
        (9, InjuredDebtFacts::DuringMarriageFamily) => whole(0),
        (2..=4, InjuredDebtFacts::BeforeMarriage) => whole(0),
        (5 | 6 | 8, InjuredDebtFacts::BeforeMarriage) => funds.community.clone(),
        (7, InjuredDebtFacts::BeforeMarriage) => &funds.community - &funds.liable_managed,
        _ => return None,
    };
    let protected =
        whole(refund) * (&funds.separate[inj] * whole(2) + protected_community) / (&all * whole(2));
    Some(phase::cap(refund, [rounded(&protected)?]))
}
