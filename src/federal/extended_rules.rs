#[path = "rules/rentals/adjustments.rs"]
mod rental_adjustments_topic;
#[path = "rules/rentals/direct.rs"]
mod rental_direct;
#[path = "rules/rentals/finalization.rs"]
mod rental_finalization;
#[path = "rules/rentals/linked_sources.rs"]
mod rental_linked_sources;
#[path = "rules/rentals/reporting.rs"]
mod rental_reporting;
#[path = "rules/rentals/settlement.rs"]
mod rental_settlement;
#[path = "rules/rentals/setup.rs"]
mod rental_setup;
#[path = "rules/rentals/sources.rs"]
mod rental_sources_topic;
#[path = "rules/rentals/types.rs"]
mod rental_types_topic;

pub(crate) use rental_adjustments_topic::*;
pub(crate) use rental_sources_topic::*;
pub(crate) use rental_types_topic::*;
pub use rental_types_topic::{
    PostPalCharacterAmounts, PostPalSourceAmounts, PostPalSources, RentalOperatingSource,
};
#[path = "rules/amt/mod.rs"]
mod amt_topic;
#[path = "rules/credits/mod.rs"]
mod credits_topic;
#[path = "rules/deductions/mod.rs"]
mod deductions_topic;
#[path = "rules/foreign/mod.rs"]
mod foreign_topic;
#[path = "rules/investment/mod.rs"]
mod investment_topic;

pub(crate) use amt_topic::*;
pub(crate) use credits_topic::*;
pub(crate) use deductions_topic::*;
pub(crate) use foreign_topic::*;
pub(crate) use investment_topic::*;

use crate::kernel::source_fact::SourceFact;
use alloc::vec::Vec;
// Coverage: Form6251 presentation — partial PRINT projection (2026-10-13).
//   A positive AMT liability projects lines4–11 from the actual exemption,
//   foreign-credit and unaveraged regular-tax comparison owners, without a
//   second computation or changed rounding. Other filing conditions and
//   PartI/III projection remain incomplete. Tests: schedules_1_3.rs corpus
//   comparison footings and explicit ISO case; IRS2026 draft f6251 PartII.
// Coverage: Foreign income tax deduction (2026-10-10).
//   ForeignTaxClaim/Page::Deduction preserves the explicit choice through the normal
//   editor lifecycle. foreign_tax_deduction consumes accepted INT/DIV taxes, every
//   attached K-3 category through its K-1 basis allowance, and additional paid-tax
//   rows once. Owner-specific net Form2555 amounts remove excluded-earned-income tax.
//   ScheduleA6/7 is outside SALT,6251 line2a adds it back,8960 line9b uses known NII
//   attribution, and portfolio (not passive-activity) tax reduces regular4952 income.
//   Unknown K-3/released-carry character omits only a named uncertain benefit and
//   retains the4952 completeness bound.901(j)(3) tax stays with its source country;
//   direct Ordinary Accrued rows without payment evidence omit the potential cash
//   deduction with a named notice; K-3 entity paid/accrued treatment is preserved.
//   Sources: Pub514(2025),i1116(2026 draft9/28),i6251/i8960(2025),f4952(2025)p4,
//   IRC164/275/901(j)(3),26CFR1.1411-4(f),1.702-1(a)(6),1.1366-1(a)(2)(iv).
//   Tests: src/tests/foreign/foreign_deduction.rs; src/tests/foreign/interview.rs foreign_deduction_*.
//   Stored-shape change: appended Deduction and new accepted personal-record types;
//   Retained investment components preserve nominee/death/instrument ownership;
//   current estate qualified payee statements join their accepted owner once.
//   Direct source taxes share the existing early royalty routing projection;
//   unknown/automatic expense endpoints cannot invent allowance/carry priority.
// Coverage: Form 1116 / Schedule B (1116) — Supported (audited 2026-09-24 through 2026-09-27).
//   The ordinary claim lives in form_1116.rs (typed Form 1116 records for
//   each category and tax system with country columns, HTKO and Total, and Schedule B for each
//   category; every line emitted with its form line) and foreign_capital.rs (Worksheets A and B,
//   the Worksheet for Line 18). This function only routes the claim. It supersedes every "Refused by
//   name" list below: foreign branch and section 901(j) categories, a business run abroad chosen
//   from Schedule C, foreign-source sales, accrued taxes, the Form 8978 decrease of line 20, the K-3
//   foreign branch column and capital lines, K-1 amounts without a K-3, royalties, farms, wagering,
//   other itemized classes, the AMT adjustments, student, vehicle and business interest and a
//   savings penalty are calculated (conservatively, with one note each); a checked situation, a
//   Pub. 514 rate-group mix and a tax figured on Form 8615 or Schedule J leave the credit out with a
//   note. See the Coverage block of form_1116.rs.
//   Foreign earned income excluded on Form 2555 no longer refuses the claim.
//   ForeignOrdinaryTax gains an owner (Employment sources reconcile per person with Form 2555 line 19:
//   W-2 box 1 pay for work abroad plus foreign-employer pay). Line 1a leaves out the person's
//   excluded wages (CreditBasis.excluded_income, Form 2555 line 43); lines 3d and 3e keep them, so
//   the deduction apportionment is unchanged; line 12 reduces each person's Employment taxes by
//   line 43 over line 27 less the housing deduction (line 50), for the regular and the AMT credit
//   (ids foreign_general_excluded_tax_reduction, foreign_general_amt_excluded_tax_reduction). The
//   housing deduction joins the line 3b adjustments. A foreign service business keeps
//   ISSUE_FTC_FOREIGN_BUSINESS. Tests: src/tests/foreign/work_abroad.rs f4_* and the two-spouse case.
//   Line 20 (regular tax) excludes any tax from Form 4972 on Form 1040 line 16
//   (2026 draft i1116): CreditBasis.regular is regular_liability less lump_sum_tax. The separate
//   Form 1116 limitation for a lump sum (box g, section 402(d)(7)) is not calculated: a 1099-R
//   carries no foreign tax and the App claims no credit on the lump sum.
//   A K-1's Schedule K-3 Parts II and III are Form 1116 sources (schedule_k3.rs;
//   Coverage: Schedule K-3 Supported for Parts I-III): Part II line 24 columns (c)/(d) by country
//   and category are line 1a (a passive row carries the same ForeignWithholding answer as a 1099
//   source and joins the withholding groups before the high-tax kickout), the K-3 gross income
//   replaces the K-1's net amounts on line 3e (CreditBasis.k3_correction), Section 2 deductions of
//   the categories are line 2, column (f) deductions join line 3b, interest lines 39-43 are line 4b
//   (column (f) business and passive activity interest by the Part III Section 2 line 6b/6d assets
//   pooled over the K-3s only), Part III Section 4 line 1 taxes are line 8 and line 2 reductions are
//   line 12 (spread by income, each row floored at zero); unused taxes are Schedule B line 6
//   through the existing carryover and carryback code. Review ids foreign_k3_*. A K-1 box 21 /
//   box 16 code F tax without a K-3 is never credited (WARN_K1_FOREIGN_TAX_WITHOUT_K3) and counts
//   toward the section 904(j) $300/$600 test; payee_statement_taxes adds the K-3 passive taxes to
//   the 1099 box 6/7 taxes, which are stored on FederalInterest/FederalDividend and read from
//   there (an Ordinary claim no longer loses them on reopen; the
//   election holds only its eligibility answer: no typed statement list). Refused by name:
//   ISSUE_FTC_PASS_THROUGH (a partnership or S corporation K-1 with a Schedule E Part II amount
//   and no K-3, or whose loss or deductions are limited this year), ISSUE_FTC_K3_INTEREST. Tests:
//   src/tests/foreign/foreign.rs k3_* and k1_foreign_tax_without_a_k3_is_never_credited, src/tests/cross/interview.rs
//   k3_*, src/tests/passthrough/passthrough.rs k3_ai_import_selects_the_printed_lines_only.
//   Lines: General (employee pay) and passive (interest, dividends) categories; Part I
//   deduction allocation (standard or itemized, state tax, mortgage and investment interest, IRA,
//   senior deduction); high-tax kickout; passive-group loss netting; Part III regular and AMT
//   limits (normal or simplified election); Schedule B carryover/carryback history; OFL/SLL
//   accounts and recapture; $300/$600 election. Line 1a qualified dividend rate
//   adjustment (0% left out, 15%/20% at 0.4054/0.5405, AMT 0.5357/0.7143, ratable across the
//   QDCG worksheet lines 9/17/20 or Form 6251 lines 23/30/33, after the high-tax test per
//   1.904-4(c)(2)(iii)); Worksheet for Line 18 and its AMT form with worldwide qualified
//   dividends and Form 1099-DIV capital gain distributions; required only when QDCG line 5 > 0
//   and line 23 < line 24 (AMT: line 17 > 0 and line 38 < line 39); adjustment exception
//   (line 5 within the 24% bracket, foreign qualified dividends < $20,000, AMT line 17 within
//   the 26% bracket) presumed unless the filer chooses ForeignRateAdjustment::Adjust
//   (1.904(b)-1(b)(3), (e)(2)). ForeignTaxCategory::Passive carries the qualified share;
//   FOREIGN_QUALIFIED_ABOVE_INCOME on both sides. U.S.-source-only Schedule D returns:
//   Form 8949 sales, capital loss carryovers and 1099-DIV boxes 2a/2b/2d; line 3e counts each
//   sale's gain before losses (CreditBasis::capital_gross); RateBands follows the Schedule D Tax
//   Worksheet (lines 18/21/22/30/33/39/42; equals the QDCG worksheet without 25%/28% gain) and
//   Form 6251 Part III (lines 23/30/33/36 with SDTW lines 14/21); Worksheet for Line 18 lines
//   2-5 (0.2432/0.3243) and AMT line 5 (0.1071, AMT lines 2-3 skipped); exception test on SDTW
//   line 18. Refused by name: "Form 1116 capital gains" (other Schedule D sources: employee
//   stock/ESPP/ISO/NUA/debt-property/retirement-property sales, 1231/4797, home, K-1, 2439,
//   1202; any AMT-different sale or carryover; Form 4952 line 4g; Form 8615/Schedule J tax);
//   "Form 1116 special situations" (treaty re-sourcing, refunds and
//   redeterminations, accrued/contested taxes, branches/QBUs, CFC/10%-owned companies,
//   financial-services income, foreign-source capital gains). Every other Part I fact pattern is calculated or
//   refused by name (section "Foreign tax credit", message naming the Form 1116 line).
//   Calculated: Schedule 1 Part II adjustments (educator, HSA, MSA, pension, alimony paid,
//   67(e)) on line 3b with the IRA deduction (i1116 line 3b; Pub 514 "Deductions not
//   definitely related"), for regular and AMT. The half SE tax, owner SEP/SIMPLE/
//   qualified plan, SE health insurance, remitted jury pay, savings penalty and the Form 3903
//   moving deduction (incident to the new post's compensation) are definitely
//   related to U.S. income (Pub 514 "Definitely related": incurred in the activity or with the
//   property producing the income; the App calculates only U.S. businesses, jury service and
//   bank interest here) and stay off Form 1116 (CreditBasis::us_adjustments, restored only to
//   line 3e); a penalty whose account interest could be foreign source (foreign passive
//   income above the other accounts' interest and dividends) is refused on line 2. Student loan and Schedule 1-A vehicle loan interest on line 4b, assigned to U.S.
//   income under the $5,000 small-foreign-income allocation; the QBI and Schedule 1-A tips,
//   overtime and vehicle deductions reduce only line 18 (line 11b less line 14 plus the senior
//   deduction; QBI is U.S.-connected income under section 199A(c)(3)(A)(i), tips and overtime
//   relate to U.S.-reported pay); U.S. Schedule C line 7 gross income and Schedule E line 3
//   rents replace their nets on line 3e, their expenses being definitely related to U.S.
//   income (CreditBasis::gross_correction); section 1244 ordinary losses on domestic stock
//   added back to line 3e; tax-exempt interest stays out of line 3e. Refused by name: foreign
//   service businesses (lines 1a/2), Form 2555 exclusions or election (lines 3d/3e/12), Schedule
//   F farms and professional gambling (line 3e), Form 461 excess business loss (lines 3b/3e; a
//   prior NOL is refused earlier as "Regular loss history"), K-1 business or rental income and
//   S corporation loan gain (lines 2/3e), royalties, personal-property rentals and rental sales
//   (line 3e), other itemized classes (lines 2-3b), itemized above AGI (lines 3a/18), a
//   qualified disaster loss on the standard deduction (line 3a), Form 6251 adjustments other
//   than itemized deductions, the state refund, QBI and Schedule 1-A (AMT lines 3e/18), line
//   4b interest without the $5,000 allocation, a savings penalty on possibly foreign-source
//   interest (line 2), and a claim with no source (Part II).
//   Evidence: extended_rules.rs (foreign_tax_credit, RateBands, foreign_*_limit_income),
//   extended_model.rs (ForeignTaxClaim, ForeignRateAdjustment), foreign_credit_history.rs,
//   foreign_losses.rs, foreign_loss_history.rs, foreign_state_tax.rs; src/tests/foreign/foreign.rs
//   ordinary_foreign_credit_limits_both_regular_and_amt_taxes,
//   small_foreign_credit_election_caps_and_ordering,
//   foreign_carryback_allocates_separate_prior_limits_without_claiming_refund,
//   form_1116_qualified_dividends_and_capital_gain_distributions_match_hand_computation,
//   form_1116_splits_foreign_qualified_dividends_ratably_across_zero_and_fifteen_percent,
//   form_1116_adjustment_exception_ends_at_20000_of_foreign_qualified_dividends,
//   form_1116_adjustment_exception_ends_at_the_top_of_the_24_percent_bracket,
//   form_1116_no_adjustment_when_qdcg_line_5_is_zero,
//   form_1116_refuses_schedule_d_gains_and_foreign_qualified_above_the_return,
//   form_1116_schedule_d_return_with_25_and_28_percent_gain_matches_hand_computation,
//   form_1116_capital_loss_limit_counts_gross_gains_on_line_3e,
//   form_1116_unrecaptured_gain_enters_the_25_percent_group_only_above_the_24_percent_bracket,
//   form_1116_short_term_loss_first_offsets_the_28_percent_group,
//   form_1116_adjustments_on_line_3b_and_qbi_below_line_18_match_hand_computation,
//   form_1116_us_schedule_c_enters_line_3e_at_gross_income,
//   form_1116_ira_deduction_stays_on_line_3b_beside_an_owner_sep_that_does_not,
//   form_1116_refuses_a_savings_penalty_on_possibly_foreign_interest,
//   form_1116_us_rental_enters_line_3e_at_gross_rents,
//   form_1116_student_loan_interest_goes_to_us_income_under_the_5000_election,
//   form_1116_small_foreign_interest_election_ends_above_5000,
//   form_1116_tips_deduction_reduces_line_18_only_and_floors_at_zero,
//   form_1116_tax_exempt_interest_stays_out_of_line_3e,
//   form_1116_refuses_each_unallocated_pattern_by_line,
//   src/tests/foreign/interview.rs
//   foreign_qualified_above_source_income_rejects_on_both_sides; sources i1116 (2025), i1116sb,
//   i6251, i1040sd (Schedule D Tax Worksheet), Pub 514, 26 CFR 1.904-4/1.904(b)-1/1.904(g)-3.
// Coverage: Form 8582 — Supported (audited 2026-09-24).
//   Lines: $25,000/$12,500 special allowance with MAGI phaseout (MFS lived-together = 0); Parts
//   VI/VII unallowed-loss allocation across activities; Part IX character allocation;
//   former-passive-activity own-income release; prior unallowed losses per activity, regular and
//   AMT separately; shared pool for current ordinary domestic Schedule C, rentals,
//   1065 and1120S K-1s. Current C needs established no-prior/special scope; its
//   operating cents return to C rather than Schedule E. Refused: dispositions of K-1
//   rentals and a prior suspended loss of a rental now treated as nonpassive (rental confirmation
//   gate, extended_model.rs). A publicly traded partnership's ordinary and kept passive
//   box 2/3 rental items share its own section 469(k) pool, outside Form 8582 and its allowance.
//   PTP rental-property, professional, disposition and rental-QBI combinations retain named
//   source boundaries. Form 8582-CR for Form 3800 passive credits: general_business_credit.rs.
//   Lines 1d, 2d, 3, 4-9 (Part II, when the rental real estate with
//   active participation and the whole pool both show a loss), 10 and 11 are emitted as
//   form8582_line_N rows (regular pass only, when the return has a passive activity); line 9 is
//   RentalResult.special_allowance, which Form 8582-CR line 35 subtracts from the allowance.
//   Test: src/tests/credits/housing_credit.rs fixture C, F.
//   Lines 10 and 11 use each
//   activity's income before its prior-year loss (PassiveEntry.prior, Part IV/V column (a));
//   line 6 is never below zero and line 7 follows it; line 8 is half of line 7 with a half dollar
//   rounded up; a married filing separately filer who lived with the spouse has no line 1d and no
//   lines 4-9 (Part V, line 10); a real estate professional's rental loss stays out of line 6
//   (section 469(i)(3)(E)(iv)); Schedule E line 22 is the net loss plus the prior-year loss less what
//   stays suspended (Part VIII column (c)), also for a professional's nonpassive loss and for a
//   property sold in full; a PTP's box 1 is PassiveEntry.ptp, never in the pool.
//   Evidence: kernel/loss_limits.rs (passive_pool, passive_characters); native allowance below,
//   scorp.rs (former passive); src/tests/activity/passthrough.rs
//   personal_rental_loss_is_suspended_without_real_estate_special_allowance,
//   live_passive_k1_profit_offsets_rental_loss_before_special_allowance; src/tests/federal/general.rs
//   kernel/loss_limits_tests.rs published_character_example_keeps_source_cents_and_native_dollars_distinct;
//   src/tests/activity/business.rs passive_pool_business_and_rental_share_income_but_not_rental_allowance;
//   source Pub 925, i8582.
//   Kept-rental partial dispositions share the existing
//   passive activity and special allowance. Part IV/V retains income and loss
//   on different forms in separate columns; Part IX allocates suspension to
//   Schedule E and each Form 4797 part. rental_partial_cuts supplies the same
//   post-limit property amounts to every downstream owner, independently for
//   regular tax and AMT. Tests: src/tests/investment/investment.rs unit40_*; i8582, Pub 925.
//   A rental sold in full (FederalRental.sale) is a section 469(g)
//   disposition: current net less prior loss plus the Form 4797 gain; an overall loss is
//   nonpassive (MAGI includes it with the gain), an overall gain enters the pool as passive income
//   with the gain removed from Schedule E income (PassiveEntry.disposition_gain). 2026
//   depreciation runs through the month of sale (depreciation_for_year); sold property has no UBIA.
//   Test: src/tests/activity/investment.rs rental_sale_overall_loss_is_nonpassive_at_the_469g_boundary.
//   A like-kind exchange is not a fully taxable disposition (section
//   469(g)(1)(A)): its recognized gain is passive income of the activity (disposition_gain), the
//   suspended losses stay with the entry that continues as the property received; an exchange
//   taxed as a sale releases them except when the other side is a related person.
//   A Form 4835 farm rental is one more PassiveEntry (farm_rental, Form 8582
//   line 1 with the allowance when the owner actively participated, line 2 without), kept passive
//   for a real estate professional (WARN_FARM_RENTAL_PASSIVE_ASSUMED); its allowed amount is
//   RentalResult.farm_rental, Schedule E line 40, and its own land and asset sales stay outside the
//   pool. Tests: src/tests/farm/farm_rental.rs fixtures B and D.
//   K-1 (1065 and 1120-S) box 2 and box 3 activities are pool entries
//   (PassiveEntry.pass_through, pass_through_rentals): box 2 with active participation in Part IV
//   with the allowance, box 2 otherwise and box 3 in Part V; a real estate professional's box 2
//   (with the recorded status) is nonpassive, its carryover released against its own income, its
//   loss kept out of MAGI and its net reversed by the recorded NIIT safe harbor; lines
//   {amt_}{partnership,scorp}_N_{rental_real_estate,other_rental}_{net,suspended} (Schedule E line
//   28), RentalResult.pass_through_rental_nonpassive (EIC), QBI for an activity the statement lists.
//   Tests: src/tests/passthrough/k1_rental.rs.
//   ProfessionalActive records
//   current allowance eligibility separately from professional materiality.
//   An elected filer's direct/K-1 former-passive losses share one income
//   limit through ActivityCharacters, preserving source and regular/AMT
//   histories. Member dispositions wait for the entire activity; line43
//   includes allowed ordinary and property amounts. Tests: unit4_*.
//   Joint filers share the annual qualification/election context when either
//   independently qualifies, including materiality, whole-activity disposal
//   and the NIIT safe harbor. Personal qualification, hours and source histories
//   remain separate. One existing election editor controls revocation; another
//   filer's default false never revokes it. Sources: 26CFR1.469-1T(j),
//   1.469-9(c)(4),(e),(g),(h); i8582/i1040se. Actual client/Avro controls:
//   src/tests/cross/interview.rs unit4_*; fixtures A-H: src/tests/activity/rental_specials.rs.
// Coverage: Schedule R — Supported (audited 2026-09-24).
//   Lines: Age/disability qualification incl. married-separate lived-apart rule, disability-income
//   ceilings, nontaxable SS/pension reduction, AGI excess, 15% credit, tax cap; repayment-year
//   reconciliation. Disability income limited to qualifying employer pensions (other sources
//   refused).
//   Evidence: M/extended_rules.rs elderly_credit, extended_model.rs DisabilityIncome,
//   claim_repayment.rs; T/family.rs elderly_credit_schedule_r_boundaries_and_joint_age_count;
//   T/credits.rs low_income_retiree_gets_result_and_exempt_benefits_reduce_elderly_credit;
//   T/repayment.rs repayment_senior_reconciles_original_elderly_credit_after_tax_cap; IRS i1040sr.

// 2026 parameters: Rev. Proc. 2025-32. Worksheet structure: IRS Forms 2555,
// 6251 (2026 draft), 8582, 5329 and Publications 527/925. Not filing output.
use crate::dates::ordinal;
use crate::federal_rules::{printed_line, valid_birth, valid_date, valid_service_date};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact};
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::{form_line, line, line_cents, push_line};
use crate::*;
use core::cmp::{max, min};

const RENTAL: &str = "https://www.irs.gov/publications/p527";
const PASSIVE: &str = "https://www.irs.gov/publications/p925";
const FOREIGN: &str = "https://www.irs.gov/instructions/i2555";
const AMT: &str = "https://www.irs.gov/pub/irs-dft/f6251--dft.pdf";

// Forward the existing invocation operands and its borrowed home context together.

#[allow(clippy::too_many_arguments)]
fn rentals_with_prepared_sources_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_use: &HomeUseContext,
    sources: (&crate::care::CareDeductions, Option<&crate::Section179PreparedAllocation>),
    selected_ptp: Option<&alloc::collections::BTreeMap<i64, crate::ptp_passive::PtpPalResult>>,
    native_section179: Option<alloc::collections::BTreeMap<i64, i64>>,
    regular_section179: Option<&crate::Section179PreparedAllocation>,
    regular_ptp: Option<&alloc::collections::BTreeMap<i64, crate::ptp_passive::PtpPalResult>>,
) -> RentalResult {
    let d = form_context.draft;
    let source_claims = form_context.claims;
    let (care, section179) = sources;
    let ptp_sources = selected_ptp.cloned().unwrap_or_default();
    let section179_allocations = core::cell::OnceCell::new();
    if let Some(source) = native_section179 {
        section179_allocations.set(source).expect("one actual native section179 allocation");
    }
    if let Some(source) = section179 {
        debug_assert_eq!(
            source.income.system,
            if amt {
                crate::Section179IncomeSystem::Alternative
            } else {
                crate::Section179IncomeSystem::Regular
            }
        );
        section179_allocations
            .set(
                source
                    .income
                    .k1
                    .iter()
                    .map(|(&id, allocation)| (id, allocation.allowed_cents))
                    .collect::<alloc::collections::BTreeMap<_, _>>(),
            )
            .expect("one bound selected-system section179 allocation");
    }
    // Initialize at the first pre-existing consumer read, preserving admission
    // ordering and avoiding a new calculation on unrelated rental-only paths.
    let section179_allowed = |id: i64| {
        section179_allocations
            .get_or_init(|| crate::section179::calculate_with_claims(d, source_claims).pass_through)
            .get(&id)
            .copied()
            .unwrap_or(0)
    };
    let context = rental_setup::RentalContext { form: form_context, amt, home_use };
    let rental_setup::RentalSetup {
        allocated_entries,
        native_scorps,
        initial_issues,
        mid_quarter,
        professional_groups,
        regular_groups,
        personal_inputs,
        personal,
        personal_profit_cents,
        personal_profit,
        magi,
    } = match rental_setup::prepare(&context, magi, r) {
        Ok(value) => value,
        Err(issue) => {
            return RentalResult { source_issue: Some(issue), ..Default::default() };
        }
    };
    // Publicly traded partnerships' passive net income and suspended losses
    // (section 469(k)), kept outside the common pool.
    let ptp_income = 0;
    let ptp_suspended = 0;
    let rental_direct::DirectOutput {
        magi,
        prior_home_casualty_by_owner,
        prior_home_casualty_by_id,
        source_issue,
        nonpassive_income,
        health_work_income_residual_cents,
        part_one,
        profits,
        rental_niit_adjustment,
        income_by_id,
        nii_adjustment_by_id,
        operating_sources,
        foreign_sources,
        operating_prior_allowed,
        values,
        home_activities,
        released_rental_losses,
        nonpassive_rental_losses,
    } = rental_direct::project(
        &context,
        rental_direct::DirectInputs {
            allocated_entries: &allocated_entries,
            personal: &personal,
            mid_quarter,
            professional_groups: &professional_groups,
            regular_groups: &regular_groups,
            magi,
        },
        r,
    );
    let mut amt_sources = alloc::vec::Vec::new();
    let rental_linked_sources::LinkedOutput {
        mut source_issue,
        ptp_sources,
        operating_sources,
        operating_prior_allowed,
        values,
        nii_adjustment_by_id,
        profits,
        nonpassive_income,
        magi,
        rental_niit_adjustment,
        ptp_income,
        ptp_suspended,
        farm_rental_passive_property,
        farm_rental_passive_sources,
        farm_rental_property,
        farm_rental_property_rows,
        k1_ledgers,
        has_scorp,
        estate_trust_business,
        estate_trust_nonpassive,
        native_partner_se,
    } = match rental_linked_sources::collect(
        &context,
        rental_linked_sources::LinkedSourceInputs {
            allocated_entries: &allocated_entries,
            native_scorps: &native_scorps,
            amt_sources: &mut amt_sources,
            professional_groups: &professional_groups,
            regular_groups: &regular_groups,
            mid_quarter,
            section179,
            regular_section179,
            regular_ptp,
            section179_allowed: &section179_allowed,
        },
        rental_linked_sources::LinkedInput {
            source_issue,
            ptp_sources,
            operating_sources,
            operating_prior_allowed,
            values,
            nii_adjustment_by_id,
            profits,
            nonpassive_income,
            magi,
            rental_niit_adjustment,
            ptp_income,
            ptp_suspended,
        },
        r,
    ) {
        Ok(value) => value,
        Err(issue) => {
            return RentalResult { source_issue: Some(issue), ..Default::default() };
        }
    };
    let mut pool = Vec::new();
    let settlement = match rental_settlement::settle(
        &context,
        rental_settlement::SettlementInput {
            values: &values,
            personal_inputs: &personal_inputs,
            initial_issues,
            magi,
            personal_profit,
            personal_profit_cents,
            nonpassive_rental_losses,
            released_rental_losses,
        },
        &mut pool,
        r,
    ) {
        Ok(value) => value,
        Err(rental_settlement::SettlementStop::Source(issue)) => {
            return RentalResult { source_issue: Some(issue), ..Default::default() };
        }
        Err(rental_settlement::SettlementStop::Pool(error)) => {
            return RentalResult { pool_issue: Some(error), ..Default::default() };
        }
    };
    let inputs_complete = settlement.inputs_complete;
    let reporting = rental_reporting::report(
        &context,
        rental_reporting::ReportingInput {
            allocated_entries: &allocated_entries,
            personal: &personal,
            care,
            section179,
            ptp_sources: &ptp_sources,
            farm_rental_passive_property: &farm_rental_passive_property,
            farm_rental_passive_sources: &farm_rental_passive_sources,
            values,
            personal_inputs,
            magi,
            personal_profit,
            nonpassive_income,
            health_work_income_residual_cents,
            part_one,
            profits,
            rental_niit_adjustment,
            ptp_income,
            ptp_suspended,
            estate_trust_business,
            estate_trust_nonpassive,
            prior_home_casualty_by_owner,
            prior_home_casualty_by_id,
            income_by_id,
            nii_adjustment_by_id,
            operating_sources,
            foreign_sources,
            operating_prior_allowed,
            home_activities,
            farm_rental_property,
            farm_rental_property_rows,
            k1_ledgers,
            has_scorp,
            section179_allocations,
        },
        settlement,
        &mut source_issue,
        r,
    );
    rental_finalization::finish(
        &context,
        rental_finalization::FinalizationInput {
            allocated_entries: &allocated_entries,
            care,
            native_partner_se,
            inputs_complete,
            source_issue,
        },
        reporting,
        r,
    )
}
