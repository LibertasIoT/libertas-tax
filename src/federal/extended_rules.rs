use crate::kernel::source_fact::SourceFact;
use alloc::vec::Vec;
// Coverage: Form6251 presentation — partial PRINT projection (2026-10-13).
//   A positive AMT liability projects lines4–11 from the actual exemption,
//   foreign-credit and unaveraged regular-tax comparison owners, without a
//   second computation or changed rounding. Other filing conditions and
//   PartI/III projection remain incomplete. Tests: schedules_1_3.rs corpus
//   comparison footings and explicit ISO case; IRS2026 draft f6251 PartII.
// Coverage: Foreign income tax deduction (original Unit 1 follow-up, 2026-10-10).
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
//   Dispatcher owns format/coverage reconciliation.
// Coverage: Form 1116 / Schedule B (1116) — Supported (audited 2026-09-24; T2-26 2026-09-26;
//   T2-26b 2026-09-27; T2-26c 2026-09-27; T2-26d 2026-09-27).
//   Unit B9-6 (2026-10-10): the ordinary claim now lives in form_1116.rs (typed Form 1116 records for
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
//   Unit B7b (2026-10-09): foreign earned income excluded on Form 2555 no longer refuses the claim.
//   ForeignOrdinaryTax gains an owner (Employment sources reconcile per person with Form 2555 line 19:
//   W-2 box 1 pay for work abroad plus foreign-employer pay). Line 1a leaves out the person's
//   excluded wages (CreditBasis.excluded_income, Form 2555 line 43); lines 3d and 3e keep them, so
//   the deduction apportionment is unchanged; line 12 reduces each person's Employment taxes by
//   line 43 over line 27 less the housing deduction (line 50), for the regular and the AMT credit
//   (ids foreign_general_excluded_tax_reduction, foreign_general_amt_excluded_tax_reduction). The
//   housing deduction joins the line 3b adjustments. A foreign service business keeps
//   ISSUE_FTC_FOREIGN_BUSINESS. Tests: src/tests/foreign/work_abroad.rs f4_* and the two-spouse case.
//   Unit B2 (2026-10-09): line 20 (regular tax) excludes any tax from Form 4972 on Form 1040 line 16
//   (2026 draft i1116): CreditBasis.regular is regular_liability less lump_sum_tax. The separate
//   Form 1116 limitation for a lump sum (box g, section 402(d)(7)) is not calculated: a 1099-R
//   carries no foreign tax and the App claims no credit on the lump sum.
//   Unit k3 (2026-10-09): a K-1's Schedule K-3 Parts II and III are Form 1116 sources (schedule_k3.rs;
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
//   there (design-improvements row 191: an Ordinary claim no longer loses them on reopen; the
//   election holds only its eligibility answer: no typed statement list). Refused by name:
//   ISSUE_FTC_PASS_THROUGH (a partnership or S corporation K-1 with a Schedule E Part II amount
//   and no K-3, or whose loss or deductions are limited this year), ISSUE_FTC_K3_INTEREST. Tests:
//   src/tests/foreign/foreign.rs k3_* and k1_foreign_tax_without_a_k3_is_never_credited, src/tests/cross/interview.rs
//   k3_*, src/tests/passthrough/passthrough.rs k3_ai_import_selects_the_printed_lines_only.
//   Lines: General (employee pay) and passive (interest, dividends) categories; Part I
//   deduction allocation (standard or itemized, state tax, mortgage and investment interest, IRA,
//   senior deduction); high-tax kickout; passive-group loss netting; Part III regular and AMT
//   limits (normal or simplified election); Schedule B carryover/carryback history; OFL/SLL
//   accounts and recapture; $300/$600 election. T2-26: line 1a qualified dividend rate
//   adjustment (0% left out, 15%/20% at 0.4054/0.5405, AMT 0.5357/0.7143, ratable across the
//   QDCG worksheet lines 9/17/20 or Form 6251 lines 23/30/33, after the high-tax test per
//   1.904-4(c)(2)(iii)); Worksheet for Line 18 and its AMT form with worldwide qualified
//   dividends and Form 1099-DIV capital gain distributions; required only when QDCG line 5 > 0
//   and line 23 < line 24 (AMT: line 17 > 0 and line 38 < line 39); adjustment exception
//   (line 5 within the 24% bracket, foreign qualified dividends < $20,000, AMT line 17 within
//   the 26% bracket) presumed unless the filer chooses ForeignRateAdjustment::Adjust
//   (1.904(b)-1(b)(3), (e)(2)). ForeignTaxCategory::Passive carries the qualified share;
//   FOREIGN_QUALIFIED_ABOVE_INCOME on both sides. T2-26b: U.S.-source-only Schedule D returns:
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
//   financial-services income, foreign-source capital gains). T2-26c: every other Part I fact pattern is calculated or
//   refused by name (section "Foreign tax credit", message naming the Form 1116 line).
//   Calculated: Schedule 1 Part II adjustments (educator, HSA, MSA, pension, alimony paid,
//   67(e)) on line 3b with the IRA deduction (i1116 line 3b; Pub 514 "Deductions not
//   definitely related"), for regular and AMT. T2-26d: the half SE tax, owner SEP/SIMPLE/
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
//   (the SBIC variant of this test was deleted with section 1242, B7a); src/tests/foreign/interview.rs
//   foreign_qualified_above_source_income_rejects_on_both_sides; sources i1116 (2025), i1116sb,
//   i6251, i1040sd (Schedule D Tax Worksheet), Pub 514, 26 CFR 1.904-4/1.904(b)-1/1.904(g)-3.
// Coverage: Form 8582 — Supported (audited 2026-09-24).
//   Lines: $25,000/$12,500 special allowance with MAGI phaseout (MFS lived-together = 0); Parts
//   VI/VII unallowed-loss allocation across activities; Part IX character allocation;
//   former-passive-activity own-income release; prior unallowed losses per activity, regular and
//   AMT separately; shared pool for rentals, 1065 and 1120S K-1s. Refused: dispositions of K-1
//   rentals and a prior suspended loss of a rental now treated as nonpassive (rental confirmation
//   gate, extended_model.rs). A publicly traded partnership is not refused for its box 1: its
//   passive result is limited by itself (section 469(k), F7), outside the pool and Form 8582, but
//   its box 2/3 rentals stay refused (ISSUE_K1_RENTAL_PTP). Form 8582-CR for Form 3800 passive credits: general_business_credit.rs.
//   Unit form-8586 (2026-10-09): lines 1d, 2d, 3, 4-9 (Part II, when the rental real estate with
//   active participation and the whole pool both show a loss), 10 and 11 are emitted as
//   form8582_line_N rows (regular pass only, when the return has a passive activity); line 9 is
//   RentalResult.special_allowance, which Form 8582-CR line 35 subtracts from the allowance.
//   Test: src/tests/credits/housing_credit.rs fixture C, F.
//   Unit F7 (2026-10-10, line audit E/8582, src/tests/cross/line_audit_e_8582.rs): lines 10 and 11 use each
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
//   Unit40 (2026-10-12): kept-rental partial dispositions share the existing
//   passive activity and special allowance. Part IV/V retains income and loss
//   on different forms in separate columns; Part IX allocates suspension to
//   Schedule E and each Form 4797 part. rental_partial_cuts supplies the same
//   post-limit property amounts to every downstream owner, independently for
//   regular tax and AMT. Tests: src/tests/investment/investment.rs unit40_*; i8582, Pub 925.
//   T2-35 (2026-09-26): a rental sold in full (FederalRental.sale) is a section 469(g)
//   disposition: current net less prior loss plus the Form 4797 gain; an overall loss is
//   nonpassive (MAGI includes it with the gain), an overall gain enters the pool as passive income
//   with the gain removed from Schedule E income (PassiveEntry.disposition_gain). 2026
//   depreciation runs through the month of sale (depreciation_for_year); sold property has no UBIA.
//   Test: src/tests/activity/investment.rs rental_sale_overall_loss_is_nonpassive_at_the_469g_boundary.
//   Unit B2 (2026-10-09): a like-kind exchange is not a fully taxable disposition (section
//   469(g)(1)(A)): its recognized gain is passive income of the activity (disposition_gain), the
//   suspended losses stay with the entry that continues as the property received; an exchange
//   taxed as a sale releases them except when the other side is a related person.
//   Unit B2 (2026-10-09): a Form 4835 farm rental is one more PassiveEntry (farm_rental, Form 8582
//   line 1 with the allowance when the owner actively participated, line 2 without), kept passive
//   for a real estate professional (WARN_FARM_RENTAL_PASSIVE_ASSUMED); its allowed amount is
//   RentalResult.farm_rental, Schedule E line 40, and its own land and asset sales stay outside the
//   pool. Tests: src/tests/farm/farm_rental.rs fixtures B and D.
//   Unit k1-rentals (2026-10-09): K-1 (1065 and 1120-S) box 2 and box 3 activities are pool entries
//   (PassiveEntry.pass_through, pass_through_rentals): box 2 with active participation in Part IV
//   with the allowance, box 2 otherwise and box 3 in Part V; a real estate professional's box 2
//   (with the recorded status) is nonpassive, its carryover released against its own income, its
//   loss kept out of MAGI and its net reversed by the recorded NIIT safe harbor; lines
//   {amt_}{partnership,scorp}_N_{rental_real_estate,other_rental}_{net,suspended} (Schedule E line
//   28), RentalResult.pass_through_rental_nonpassive (EIC), QBI for an activity the statement lists.
//   Tests: src/tests/passthrough/k1_rental.rs.
//   Unit4 follow-up (2026-10-13): ProfessionalActive records
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
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::lines::{form_line, line, line_cents, push_line};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact};
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::*;
use core::cmp::{max, min};

const RENTAL: &str = "https://www.irs.gov/publications/p527";
const PASSIVE: &str = "https://www.irs.gov/publications/p925";
const FOREIGN: &str = "https://www.irs.gov/instructions/i2555";
const AMT: &str = "https://www.irs.gov/pub/irs-dft/f6251--dft.pdf";

// A building's depreciation for one calendar year; `sold` limits the year of
// sale to the half-months before the middle of the month sold (mid-month
// convention, Pub 946 ch. 4). Earlier years reproduce the amounts allowed or
// allowable for the Form 4797 line 24 total.
fn rental_building_source_rule(b: &RentalBuilding) -> Result<(), (&'static str, &'static str)> {
    if b.casualty.as_ref().is_some_and(|source| source.issue(b.cost.saturating_sub(b.land)).is_some())
        || b.land < 0
        || b.cost < b.land
        || b.cost > 100_000_000_000
        || b.sale_history.as_ref().is_some_and(|history| !history.valid(b.cost - b.land))
        || !(0..=b.cost - b.land).contains(&b.credit_reduction)
        || matches!(b.acquired, crate::RentalAcquisition::Converted { fair_market_value } if !(0..=100_000_000_000).contains(&fair_market_value))
        || !valid_service_date(b.in_service)
        || b.in_service < 19870101
        // A part replaced or removed in 2026 leaves a building placed in
        // service before 2026 (Reg. 1.168(i)-8(d)): its cost is a part of the
        // building's, and it follows the date placed in service.
        || b.partial.as_ref().is_some_and(|part| {
            !(20260101..=20261231).contains(&part.date)
                || !crate::federal_rules::valid_date(part.date)
                || part.date <= b.in_service
                || b.in_service / 10000 >= 2026
                || !(1..b.cost - b.land).contains(&part.cost_of_part)
                || !(0..=100_000_000_000).contains(&part.proceeds)
        })
    {
        return Err((
            "ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS",
            crate::fixture_english!(
                "Review building cost, land allocation and service date; this path uses post-1986 residential or nonresidential GDS."
            ),
        ));
    }
    Ok(())
}

pub(crate) fn depreciation_for_year(
    b: &RentalBuilding,
    amt: bool,
    property_type: &RentalPropertyType,
    year: u32,
    sold: Option<u32>,
    energy_basis_reduction: crate::kernel::source_fact::SourceFact<i64>,
) -> Result<i64, (&'static str, &'static str)> {
    rental_building_source_rule(b)?;
    let unavailable = (
        crate::rental_energy::BASIS,
        "The energy property needs its actual prepared section 179D basis adjustment.",
    );
    let energy_basis_reduction = match energy_basis_reduction {
        crate::kernel::source_fact::SourceFact::Reported(value)
            if (0..=b.cost - b.land - b.rehabilitation_reduction()).contains(&value) =>
        {
            value
        }
        crate::kernel::source_fact::SourceFact::NotReported if b.energy.is_none() => 0,
        _ => return Err(unavailable),
    };
    // Section 179D(e) has its own basis input; it is never a credit or
    // depreciation recapture source. Original acquisition cost remains UBIA
    // under Treas.Reg.1.199A-2(c)(3)(i).
    // Section 50(c): a rehabilitation credit reduces the depreciable basis by
    // the whole credit determined.
    let basis = match b.acquired {
        crate::RentalAcquisition::Converted { fair_market_value } => {
            phase::cap(b.cost - b.land, [fair_market_value])
        }
        _ => b.cost - b.land,
    } - b.rehabilitation_reduction()
        - energy_basis_reduction;
    let reduction =
        crate::casualty::rental::continuing_reduction(b, amt, property_type, year, basis).map_err(
            |key| (key, "Reconcile the property's actual casualty basis and depreciation records."),
        )?;
    // Half-months of the year of sale before the mid-month disposition.
    let window = sold.map(|date| 2 * (date / 100 % 100).clamp(1, 12) as i32 - 1);
    // A part of the building replaced or removed in 2026 takes its cost out of
    // the depreciable basis from the month of the disposition (Reg.
    // 1.168(i)-8(f)(3)): the year is the mid-month share before the part
    // leaves and the building's share of the cost after it.
    let part = b.partial.as_ref().filter(|_| year == 2026);
    let basis_after =
        part.map(|p| fraction(basis, b.cost - b.land - p.cost_of_part, b.cost - b.land));
    type AmountResult = Result<i64, (&'static str, &'static str)>;
    let split = |current: i32, amount: &dyn Fn(i64, i32) -> AmountResult| -> AmountResult {
        match (part, basis_after) {
            (Some(p), Some(after)) => {
                let w = (2 * (p.date / 100 % 100).clamp(1, 12) as i32 - 1).min(current);
                Ok(amount(basis, w)? + amount(after, current - w)?)
            }
            _ => amount(basis, current),
        }
    };
    // The shared historical owner selects the original GDS/AMT clock and
    // computes its exact amount. Keep this rental owner's existing rounding
    // and partial-disposition allocation boundaries.
    let recovery = crate::historical_recovery::rental_building_recovery(
        property_type,
        b.in_service,
        b.commercial_1993_transition.as_ref(),
        amt,
    )?;
    let current = if reduction.is_some() {
        // The dated owner caps the actual calendar window to the original
        // remaining life, including an already fully recovered zero basis.
        24
    } else {
        crate::historical_recovery::rental_recovery_window(recovery, b.in_service, year) as i32
    };
    split(window.map_or(current, |w| current.min(w)), &|base, hm| {
        let amount = |window| {
            crate::historical_recovery::rental_building_amount(
                recovery,
                base,
                b.in_service,
                year,
                window,
                reduction,
            )
            .map(|amount| amount.rounded_cents())
            .map_err(|_| {
                (
                    "ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS",
                    "Review the original building recovery records.",
                )
            })
        };
        if reduction.is_none()
            && recovery == crate::historical_recovery::RentalBuildingRecovery::Commercial39
        {
            // Preserve the rental form's existing annual-table rounding
            // before applying its disposition/component fraction.
            Ok(fraction(amount(24)?, i64::from(hm), 24))
        } else {
            amount(hm as u32)
        }
    })
}

// Section 469(g)(3): the part of an overall loss that an installment sale of the
// entire interest makes nonpassive this year, and the part that stays
// suspended. The ratio is the gain recognized this year (the installment
// income and the recapture, which is taxed in full) to the total gain (the
// gross profit and the recapture). Whole dollars.
fn installment_release(overall: i64, form: &crate::kernel::installment::Schedule) -> (i64, i64) {
    let total = form.gross_profit + form.recapture;
    let recognized = phase::bound(form.income + form.recapture, 0, total);
    let released = if total > 0 {
        phase::bound(fraction(overall, recognized, total), 0, overall)
    } else {
        overall
    };
    (released, overall - released)
}

pub(crate) fn rental_net(v: &FederalRental) -> Result<i64, (&'static str, &'static str)> {
    if let Some(key) = v.payroll.rule() {
        return Err((
            key,
            crate::fixture_english!(
                "Reconcile documented qualified wages with the original payroll statements."
            ),
        ));
    }
    rental_net_for(v, false)
}

// Same de minimis safe-harbor election check as the shared Schedule C
// `expense_elections`, applied to `RentalExpense` instead of
// `FederalBusinessExpense`: the two record shapes differ only in their
// category enum, and this check reads only `amount` and `treatment`.
pub(crate) fn rental_expense_elections(
    r: &mut FederalResult,
    expenses: &[crate::extended_model::RentalExpense],
    context: &crate::IssueContext,
    #[cfg(test)] source: &str,
    prefix: &str,
) {
    for (index, expense) in expenses.iter().enumerate() {
        if let crate::BusinessExpenseKind::DeMinimis {
            largest_cost,
            financial_statement,
            eligible,
        } = expense.treatment
        {
            let ceiling = if financial_statement { 500_000 } else { 250_000 };
            // An ineligible expense is left out before the rules run.
            crate::require_localized!(
                r,
                eligible && largest_cost <= ceiling,
                context,
                source,
                "ISSUE_DE_MINIMIS_ELECTION_REQUIRES_CONFIRMED_ELIGIBLE_BOOK_EXPENSING_THE_20E65A9C",
                "De minimis election requires confirmed eligible book expensing, the annual election for all qualifying purchases, and invoice/item costs at most $2,500 without an applicable financial statement or $5,000 with one. Reclassify ineligible purchases under their ordinary rules; an over-limit item is not a partially deductible safe-harbor purchase.",
            );
            crate::require_localized!(
                r,
                expenses.iter().all(|other| match other.treatment {
                    crate::BusinessExpenseKind::DeMinimis {
                        financial_statement: other_statement,
                        ..
                    } => other_statement == financial_statement,
                    _ => true,
                }),
                context,
                source,
                "ISSUE_USE_CONSISTENT_APPLICABLE_FINANCIAL_STATEMENT_FACTS_FOR_THIS_ACTIVI_5F5D61F3",
                "Use consistent applicable-financial-statement facts for this activity's de minimis purchases.",
            );
            line_cents(
                r,
                &alloc::format!("{prefix}_expense_{index}_de_minimis"),
                expense.amount,
                &["income.expenses"],
                "https://www.irs.gov/businesses/small-businesses-self-employed/tangible-property-final-regulations",
                msg!("EXPLAIN_N_EXPENSE_N_DE_MINIMIS"),
            );
        }
    }
}

fn rental_net_for(v: &FederalRental, amt: bool) -> Result<i64, (&'static str, &'static str)> {
    rental_use_for(v, amt).map(|u| u.net)
}

// One property's Schedule E amounts after dividing expenses by days of use
// and, for a dwelling unit used as a home, after the Worksheet 5-1 limit.
// Amounts are cents.
pub(crate) struct RentalUse {
    // One computed source-cent amount shared by the deduction and basis owner.
    energy_deduction: i64,
    expense_rows: Vec<crate::extended_model::RentalExpenseAmount>,
    pub current_operating_for_risk: Option<Exact>,
    // Already business-only Form4797 PartIV income, before activity limits.
    pub asset_recapture_cents: i64,
    pub prior_at_risk_cents: i64,
    // Line 21 before passive limits; never below zero for a home.
    pub net: i64,
    // Depreciation deducted this year (rental portion, after the home limit).
    pub depreciation: i64,
    pub home_third_tier: Option<crate::casualty::home::ThirdTierResult>,
    // Personal use exceeded the greater of 14 days or 10% of fair rental days.
    pub home: bool,
    // A home rented fewer than 15 days: no income and no rental expenses.
    pub excluded: bool,
    // Worksheet 5-1 lines 7a and 7b.
    pub carry: [i64; 2],
    // Personal portion of mortgage interest and taxes entered as rental expenses.
    pub personal_fixed: Exact,
}

// The 2026 section 481(a) increase and decrease of a rental, in cents. A
// rental with personal use keeps no record (unknowns::supported).
fn method_amounts(v: &FederalRental) -> (i64, i64) {
    (
        crate::method_change::increase(&v.method_changes),
        crate::method_change::decrease(&v.method_changes),
    )
}

// Worksheet 5-1 lines 7a and 7b: the operating expenses and depreciation that
// carry to the next year.
fn carryover_rows(r: &mut FederalResult, prefix: &str, id: i64, carry: [i64; 2]) {
    emit_lines(
        r,
        &mut (),
        [
            (
                "home_operating_carryover",
                carry[0],
                msg!("EXPLAIN_0RENTAL_N_N_HOME_OPERATING_CARRYOVER"),
            ),
            (
                "home_depreciation_carryover",
                carry[1],
                msg!("EXPLAIN_0RENTAL_N_N_HOME_DEPRECIATION_CARRYOVER"),
            ),
        ],
        |(suffix, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("{prefix}rental_{id}_{suffix}"),
                LineValue::MoneyCents(amount),
                &["income.rental.personal_use"],
                RENTAL,
                explanation,
            ))
        },
    );
}

fn rental_use_for(v: &FederalRental, amt: bool) -> Result<RentalUse, (&'static str, &'static str)> {
    rental_use_with(v, amt, false, true, None)
}
fn rental_use_with(
    v: &FederalRental,
    amt: bool,
    mid_quarter: bool,
    itemizing: bool,
    fixed_allowed: Option<[Exact; 3]>,
) -> Result<RentalUse, (&'static str, &'static str)> {
    rental_use_core(
        v,
        amt,
        mid_quarter,
        itemizing,
        fixed_allowed,
        &RentalExpense::exact_rows(&v.expenses),
        None,
    )
}
pub(crate) fn rental_source_valid(v: &FederalRental) -> bool {
    v.payroll.rule().is_none() && rental_source_rule(v).is_ok()
}

fn rental_source_rule(v: &FederalRental) -> Result<(), (&'static str, &'static str)> {
    if v.property_type.business_participation().is_some_and(|material| matches!(v.qbi_business, RentalBusiness::Yes { material_participation } if material_participation != material)) {
        return Err(("SHORT_TERM_PARTICIPATION_AGREES", crate::fixture_english!("Use the same material-participation answer for the short-stay business and its QBI treatment.")));
    }
    let invalid_input = (
        "ISSUE_RENTAL_BUILDINGS_OR_EXPENSES_OUT_OF_LIMITS",
        crate::fixture_english!(
            "Enter the rental's building records and ordinary expenses within the supported limits."
        ),
    );
    if crate::rental_energy::source_issue(v).is_some() || !v.payroll.projected_shape_valid()
        || crate::activity_risk::source_issue(&v.at_risk, &v.history).is_some()
        || v.buildings.is_empty()
        || v.buildings.len() > 100
        || v.assets.len() > 100
        || v.assets.iter().any(|a| !a.valid() || a.available179() > 0)
        || !crate::method_change::valid(&v.method_changes)
        || v.rent < 0
        || v.rent > 100_000_000_000
        || v.expenses
            .iter()
            .any(|e| e.amount < 0 || matches!(e.treatment, crate::BusinessExpenseKind::DeMinimis { largest_cost, .. } if !(1..=100_000_000_000).contains(&largest_cost)))
        || match v.personal_use {
            crate::extended_model::RentalPersonalUse::No => false,
            crate::extended_model::RentalPersonalUse::Yes {
                fair_rental_days,
                personal_days,
                overlap_days,
                direct_expenses,
            } => fair_rental_days + personal_days > 365 || overlap_days > fair_rental_days || !(0..=100_000_000_000).contains(&direct_expenses),
        }
        || !(0..=100_000_000_000).contains(&v.home_operating_carryover)
        || crate::rental_home_carry::issue(v).is_some()
    {
        return Err(invalid_input);
    }
    let risk = v.at_risk.as_ref().ok_or(invalid_input)?;
    if crate::rental_property_history::history_issue(
        &v.property_qbi_history,
        risk.prior_sale(false),
    )
    .is_some()
    {
        return Err(invalid_input);
    }
    for building in &v.buildings {
        rental_building_source_rule(building)?;
        if let Some(issue) = crate::rental_recovery::source_rule(
            &v.property_type,
            building.in_service,
            building.commercial_1993_transition.as_ref(),
        ) {
            return Err((issue.key, "Review the actual 1993 commercial recovery source."));
        }
    }
    Ok(())
}

fn rental_use_core(
    v: &FederalRental,
    amt: bool,
    mid_quarter: bool,
    itemizing: bool,
    fixed_allowed: Option<[Exact; 3]>,
    expense_rows: &[crate::extended_model::RentalExpenseAmount],
    energy_context: Option<(&FederalDraft, i64, bool)>,
) -> Result<RentalUse, (&'static str, &'static str)> {
    rental_source_rule(v)?;
    let unavailable = rental_expense_amount_issue();
    let energy = if let Some((draft, source, whole)) = energy_context {
        crate::rental_energy::for_source(draft, source, whole)
    } else {
        if v.buildings.iter().any(|cost| cost.energy.is_some()) {
            return Err((
                crate::rental_energy::YEAR,
                "The section 179D claim needs the bound return year.",
            ));
        }
        crate::rental_energy::Prepared {
            reductions: alloc::vec![0; v.buildings.len()],
            boundaries: Vec::new(),
        }
    };
    if let Some(key) = energy.boundaries.first() {
        return Err((
            *key,
            "The energy property's source-dependent deduction and basis are unavailable.",
        ));
    }
    let depreciation_year =
        u32::try_from(energy_context.map_or(2026, |(draft, _, _)| draft.tax_year))
            .map_err(|_| (crate::rental_energy::YEAR, "The bound return year is unavailable."))?;
    let mut expense_rows = expense_rows.to_vec();
    let energy_total =
        crate::kernel::ratio::checked_sum(energy.reductions.iter().copied()).ok_or(unavailable)?;
    if energy_total > 0 {
        expense_rows.push(crate::extended_model::RentalExpenseAmount {
            category: RentalExpenseCategory::Other { label: "Section 179D".into() },
            amount: exact::whole(energy_total),
            treatment: BusinessExpenseKind::Ordinary,
        });
    }
    let expense_rows = expense_rows.as_slice();
    let prior = v.at_risk.as_ref().expect("validated source").prior(amt);
    let current_expenses = RentalExpense::total_amounts(expense_rows);
    let expenses = &current_expenses + exact::whole(prior);
    let mut depreciation_total = exact::whole(0);
    let mut asset_recapture = exact::whole(0);
    for asset in &v.assets {
        depreciation_total += exact::whole(asset.deduction_for(mid_quarter, amt));
        let alternative;
        let source = if amt {
            alternative = asset.amt_projection_or_self();
            &alternative
        } else {
            asset
        };
        if let Some(recapture) = source.recapture().or_else(|| source.section179_recapture()) {
            asset_recapture += exact::whole(recapture.excess());
        }
    }
    for (index, building) in v.buildings.iter().enumerate() {
        if building.energy.is_some() && energy.boundaries.contains(&crate::rental_energy::BASIS) {
            return Err((
                crate::rental_energy::BASIS,
                "This energy property needs its actual section 179D basis source.",
            ));
        }
        let energy_basis =
            crate::kernel::source_fact::SourceFact::Reported(energy.reductions[index]);
        let depreciation = if building.casualty.is_some() {
            crate::casualty::rental::current_depreciation(
                building, v, amt, depreciation_year, energy_basis,
            ).map_err(|key| (key, "Resolve this casualty's actual basis and continuing-depreciation records before estimating."))?
        } else {
            depreciation_for_year(
                building,
                amt,
                &v.property_type,
                depreciation_year,
                v.sale.date(),
                energy_basis,
            )?
        };
        depreciation_total += exact::whole(depreciation);
    }
    if let Some(exchange) = v.sale.exchange() {
        depreciation_total += exact::whole(exchange.replacement_depreciation(v, amt)?.total());
    }
    let prior_operating = exact::whole(v.home_operating_carryover);
    let prior_total = crate::rental_home_carry::total(v, amt).ok_or((
        crate::rental_home_carry::TOTAL,
        "Supply the actual prior home-use carry for this tax system.",
    ))?;
    let prior_depreciation = exact::whole(prior_total);
    // Project only the completed Worksheet 5-1/native operating outputs,
    // after wide expenses have passed through all actual limits and offsets.
    let finish = |net: Exact,
                  current_operating_for_risk: Option<Exact>,
                  depreciation: Exact,
                  home: bool,
                  excluded: bool,
                  carry: [Exact; 2],
                  personal_fixed: Exact| {
        Ok(RentalUse {
            energy_deduction: energy_total,
            expense_rows: expense_rows.to_vec(),
            // The recapture owner already supplies the business-only amount.
            // Add after rental/home prorating, before the shared risk/PAL stage;
            // it neither receives another personal share nor becomes home rent.
            net: exact::rounded(&(net + &asset_recapture)).ok_or(unavailable)?,
            current_operating_for_risk: current_operating_for_risk
                .map(|net| net + &asset_recapture),
            asset_recapture_cents: exact::rounded(&asset_recapture).ok_or(unavailable)?,
            prior_at_risk_cents: prior,
            depreciation: exact::rounded(&depreciation).ok_or(unavailable)?,
            home_third_tier: None,
            home,
            excluded,
            carry: [
                exact::rounded(&carry[0]).ok_or(unavailable)?,
                exact::rounded(&carry[1]).ok_or(unavailable)?,
            ],
            personal_fixed,
        })
    };
    let crate::extended_model::RentalPersonalUse::Yes {
        fair_rental_days,
        personal_days,
        overlap_days,
        direct_expenses,
    } = v.personal_use
    else {
        let (increase, decrease) = method_amounts(v);
        let income = exact::whole(v.rent) + exact::whole(increase);
        let current = income - &current_expenses - &depreciation_total - exact::whole(decrease);
        let net = &current - exact::whole(prior);
        let room = phase::floor(net.clone(), exact::whole(0));
        let operating_carried = phase::cap(room.clone(), [prior_operating.clone()]);
        let tiers = crate::rental_home_carry::tiers(v, amt, 0, &(&room - &operating_carried))
            .map_err(|key| (key, "Review the prior home-use carry sources."))?;
        let depreciation_carried = exact::whole(tiers.total_allowed_cents);
        let mut result = finish(
            net - &operating_carried - &depreciation_carried,
            Some(current - &operating_carried - &depreciation_carried),
            depreciation_total + &depreciation_carried,
            false,
            false,
            [prior_operating - operating_carried, prior_depreciation - depreciation_carried],
            exact::whole(0),
        )?;
        result.depreciation =
            tiers.allowed_depreciation_cents().map_or(result.depreciation, |allowed| {
                result.depreciation - tiers.total_allowed_cents + allowed
            });
        result.home_third_tier = Some(tiers);
        return Ok(result);
    };
    let (fair, personal) = (i64::from(fair_rental_days), i64::from(personal_days));
    let rental_part = |amount: &Exact| {
        if personal == 0 {
            amount.clone()
        } else {
            exact::rounded_exact(
                &(amount * exact::ratio(fair, fair + personal).expect("rental days")),
            )
        }
    };
    let personal_use_days = personal + i64::from(overlap_days);
    let fair_days_for_test = fair - i64::from(overlap_days);
    let home = personal_use_days > 14 && personal_use_days * 10 > fair_days_for_test;
    let fixed_expenses: alloc::vec::Vec<_> = expense_rows
        .iter()
        .filter(|expense| {
            matches!(
                expense.category,
                RentalExpenseCategory::MortgageInterest | RentalExpenseCategory::TaxesAndLicenses
            )
        })
        .cloned()
        .collect();
    let fixed_total = RentalExpense::total_amounts(&fixed_expenses);
    let personal_fixed = &fixed_total - rental_part(&fixed_total);
    if home && fair < 15 {
        return finish(
            exact::whole(0),
            None,
            exact::whole(0),
            home,
            true,
            [prior_operating, prior_depreciation],
            fixed_total,
        );
    }
    let fixed = rental_part(&fixed_total) + exact::whole(direct_expenses);
    if !home {
        let operating = rental_part(&(expenses - &fixed_total));
        let depreciation = rental_part(&depreciation_total);
        let net = exact::whole(v.rent) - fixed - operating - &depreciation;
        let room = phase::floor(net.clone(), exact::whole(0));
        let operating_carried = phase::cap(room.clone(), [prior_operating.clone()]);
        let tiers = crate::rental_home_carry::tiers(v, amt, 0, &(&room - &operating_carried))
            .map_err(|key| (key, "Review the prior home-use carry sources."))?;
        let depreciation_carried = exact::whole(tiers.total_allowed_cents);
        let mut result = finish(
            net - &operating_carried - &depreciation_carried,
            None,
            depreciation + &depreciation_carried,
            home,
            false,
            [prior_operating - operating_carried, prior_depreciation - depreciation_carried],
            personal_fixed,
        )?;
        result.depreciation =
            tiers.allowed_depreciation_cents().map_or(result.depreciation, |allowed| {
                result.depreciation - tiers.total_allowed_cents + allowed
            });
        result.home_third_tier = Some(tiers);
        return Ok(result);
    }
    let eligible = if itemizing {
        fixed_allowed.map_or_else(|| rental_part(&fixed_total), |allowed| allowed.into_iter().sum())
    } else {
        exact::whole(0)
    };
    let fixed = &eligible + exact::whole(direct_expenses);
    let operating = rental_part(&expenses) - eligible + prior_operating;
    let depreciation = rental_part(&depreciation_total);
    let line3 = phase::floor(exact::whole(v.rent) - &fixed, exact::whole(0));
    let operating_allowed = phase::cap(line3.clone(), [operating.clone()]);
    let line5 = line3 - &operating_allowed;
    let current_depreciation = exact::rounded(&depreciation).ok_or(unavailable)?;
    let tiers = crate::rental_home_carry::tiers(v, amt, current_depreciation, &line5)
        .map_err(|key| (key, "Review the prior home-use carry sources."))?;
    let depreciation_allowed = exact::whole(tiers.total_allowed_cents);
    let mut result = finish(
        exact::whole(v.rent) - fixed - &operating_allowed - &depreciation_allowed,
        None,
        depreciation_allowed.clone(),
        home,
        false,
        [operating - operating_allowed, exact::whole(tiers.carry.total_cents)],
        personal_fixed,
    )?;
    if let Some(allowed) = tiers.allowed_depreciation_cents() {
        result.depreciation = allowed;
    }
    result.home_third_tier = Some(tiers);
    Ok(result)
}

pub(crate) fn expense_output_unavailable(result: &mut FederalResult, source: Option<i64>) {
    let context = source.map_or(
        crate::IssueContext::Section(crate::TaxSection::Rentals),
        crate::IssueContext::income_id,
    );
    crate::issue_localized!(
        result,
        &context,
        "Rental or royalty expenses",
        "RENTAL_EXPENSE_AMOUNT_UNAVAILABLE",
        "A required rental or royalty expense output cannot be represented in its native amount. No final tax can be reported.",
    );
    crate::federal_rules::withhold_required_source_result(result);
}

fn rental_expense_amount_issue() -> (&'static str, &'static str) {
    (
        "RENTAL_EXPENSE_AMOUNT_UNAVAILABLE",
        crate::fixture_english!(
            "The completed rental expense, carryover or operating amount cannot be represented in its required native output. No final tax can be reported."
        ),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct K1PropertyResult {
    pub id: i64,
    pub net_cents: i64,
    pub section179_sales_cents: alloc::vec::Vec<i64>,
    pub suspended_cents: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub nonpassive_cents: i64,
    // Reported form projections only.
    pub net: i64,
    pub suspended: i64,
    // Retain the amount excluded from the passive pool, including prior loss
    // absorbed by the same activity. Later NIIT depends on this distinction.
    pub nonpassive: i64,
    pub rentals: alloc::vec::Vec<K1RentalPropertyResult>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct K1RentalPropertyResult {
    pub real_estate: bool,
    pub net_cents: i64,
    pub suspended_cents: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub nonpassive_cents: i64,
    // Reported form projections only.
    pub net: i64,
    pub suspended: i64,
    pub nonpassive: i64,
    pub exclude_niit: bool,
    pub niit_disposition_loss: i64,
    // Index of this activity's one QBI entry, including its wages and UBIA.
    pub qbi_index: Option<usize>,
}

impl K1PropertyResult {
    pub(crate) fn reported_net(&self, sales: &[i64]) -> Option<i64> {
        let raw_sales = sales.iter().copied().try_fold(0i64, i64::checked_add)?;
        let reported_sales =
            sales.iter().map(|amount| dollars(*amount)).try_fold(0i64, i64::checked_add)?;
        reported_sales.checked_add(dollars(self.net_cents.checked_sub(raw_sales)?))
    }

    pub(crate) fn business(&self) -> Self {
        let net_cents = self.net_cents - self.rentals.iter().map(|r| r.net_cents).sum::<i64>();
        let suspended_cents = crate::kernel::carry::closing_values::<crate::kernel::carry::Cents>(
            self.suspended_cents.get(),
            self.rentals.iter().map(|r| r.suspended_cents.get()).sum(),
        )
        .expect("rental character holds are part of the parent hold");
        let nonpassive_cents =
            self.nonpassive_cents - self.rentals.iter().map(|r| r.nonpassive_cents).sum::<i64>();
        Self {
            id: self.id,
            net_cents,
            suspended_cents,
            nonpassive_cents,
            net: self.net - self.rentals.iter().map(|r| r.net).sum::<i64>(),
            section179_sales_cents: self.section179_sales_cents.clone(),
            suspended: dollars(suspended_cents.get()),
            nonpassive: dollars(nonpassive_cents),
            rentals: alloc::vec::Vec::new(),
        }
    }
}

impl RentalResult {
    pub(crate) fn niit_owners(&self, draft: &FederalDraft) -> Option<[i64; 2]> {
        if !self.inputs_complete
            || self.income_by_id.values().sum::<i64>() != self.income
            || self.nii_adjustment_by_id.values().sum::<i64>()
                != self.scorp_niit_adjustment
                    + self.partner_niit_adjustment
                    + self.rental_niit_adjustment
                    + self.estate_trust_niit_adjustment
            || self.nii_adjustment_by_id.keys().any(|id| !self.income_by_id.contains_key(id))
        {
            return None;
        }
        let mut owners = [0; 2];
        for (id, income) in &self.income_by_id {
            let owner = draft.income.iter().find(|entry| entry.id == *id)?.owner;
            // Both operands are the existing owner's annual source dollars.
            // No raw-cost scaling or inferred release/basis split is involved.
            owners[usize::from(owner == Owner::Spouse)] +=
                income + self.nii_adjustment_by_id.get(id)?;
        }
        Some(owners)
    }

    fn add_property(
        &mut self,
        id: i64,
        net: i64,
        suspended: i64,
        nonpassive: i64,
        rental: Option<K1RentalPropertyResult>,
    ) {
        let index = match self.k1_section1231.iter().position(|v| v.id == id) {
            Some(index) => index,
            None => {
                self.k1_section1231.push(K1PropertyResult {
                    id,
                    net_cents: 0,
                    section179_sales_cents: alloc::vec::Vec::new(),
                    suspended_cents: crate::kernel::carry::Amount::default(),
                    nonpassive_cents: 0,
                    net: 0,
                    suspended: 0,
                    nonpassive: 0,
                    rentals: alloc::vec::Vec::new(),
                });
                self.k1_section1231.len() - 1
            }
        };
        let item = &mut self.k1_section1231[index];
        item.net_cents += net;
        item.suspended_cents = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
            item.suspended_cents.get() + suspended,
        )
        .expect("nonnegative property source hold");
        item.nonpassive_cents += nonpassive;
        item.net = dollars(item.net_cents);
        item.suspended = dollars(item.suspended_cents.get());
        item.nonpassive = dollars(item.nonpassive_cents);
        if let Some(rental) = rental {
            item.rentals.push(rental);
        }
    }
}

// Preserve partner ownership through the common pool; subsequent SE, QBI,
// EBL and NIIT calculations must not reinterpret these as rental or S-corp net.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PartnerPassiveResult {
    pub id: i64,
    pub net: i64,
    pub nonpassive: i64,
    pub suspended: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub prior_released: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct K1Amount {
    pub id: i64,
    pub net: i64,
    // Actual same-invocation source endpoint; net retains summed reported rows.
    pub source_net_cents: Option<i64>,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct K1Cuts {
    pub ordinary: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub short: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub long: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    // All three together.
    pub passive: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    // Ordinary Form 4797 loss held back, by business activity, actual source cents.
    // The source rows and QBI must use the same per-activity allocation.
    pub ordinary_by_id:
        alloc::vec::Vec<(i64, crate::kernel::carry::Amount<crate::kernel::carry::Cents>)>,
    // The same Part IX cuts retained by source, short/long actual source cents.
    pub capital_by_id:
        alloc::vec::Vec<(i64, [crate::kernel::carry::Amount<crate::kernel::carry::Cents>; 2])>,
    // Activity-item losses withheld from the return, by the actual source.
    pub income_by_id:
        alloc::vec::Vec<(i64, crate::kernel::carry::Amount<crate::kernel::carry::Cents>)>,
}

// Pub974 initial-income projection needs the loss BEFORE aggregation with an
// activity's own profit. A current1000/prior5000 activity has net−4000, but
// may have allowed prior loss1000 even when its final net is zero.
// All these rows are real-estate components; the supported K-1 owner admits
// only one box2 activity per accepted parent (box1/3 are distinct classes).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HealthRentalActivity {
    pub source: i64,
    pub current: i64,
    pub prior: i64,
    pub allowed_loss: i64,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct HealthPassiveProjection {
    pub deferred: bool,
    pub rentals: alloc::vec::Vec<HealthRentalActivity>,
    pub mixed_losses: bool,
    pub nonpassive_losses: bool,
    pub released_outside_pool: bool,
}

impl HealthPassiveProjection {
    pub(crate) fn initial_addback(&self) -> Result<i64, ()> {
        if !self.deferred {
            return Ok(0);
        }
        // Pub974 pp51/56 defers8582 PartsII/III and excludes real-estate
        // rental losses. The inspected primary sources do not establish a
        // general reallocation of competing other losses or disposed/former
        // passive releases. Do not claim either counterfactual as verified.
        if self.mixed_losses || self.nonpassive_losses || self.released_outside_pool {
            return Err(());
        }
        debug_assert!(self.rentals.iter().all(|v| v.source > 0
            && v.allowed_loss >= 0
            && v.allowed_loss <= positive(-v.current) + v.prior));
        Ok(self.rentals.iter().map(|v| v.allowed_loss).sum())
    }
}

// Actual personal-property inputs in cents, before PAL allocation or filing
// rounding. These are source inputs, not a claim about an allowed loss or a
// current/prior loss-release split. Regular history remains separate from AMT.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PersonalRentalInputs {
    pub owner: crate::Owner,
    pub receipts_cents: i64,
    pub regular_expenses_cents: i64,
    pub expenses_cents: i64,
    pub regular_prior_cents: i64,
    pub prior_cents: i64,
    pub investment_recharacterization: bool,
}

// Runtime source operands from the actual rental-use and at-risk invocation.
// Before/after amounts include the native owner's applicable prior deductions
// and home-use limits; they are not reconstructed current-only profit. The
// final ScheduleE amount remains annual Dollars after the existing PAL owner,
// not a precision feed made by multiplying that amount by100.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RentalOperatingSource {
    pub source: i64,
    pub owner: crate::Owner,
    pub alternative: bool,
    pub before_at_risk_cents: i64,
    pub after_at_risk_cents: i64,
    pub prior_at_risk_cents: i64,
    pub prior_passive_cents: i64,
    pub prior_home_carry_cents: [i64; 2],
    pub home_limited: bool,
    pub excluded: bool,
    pub property_disposition: bool,
    pub filed_ordinary_dollars: Option<i64>,
}

// Native source identity, separate from an annual form row or a list index.
// An accepted K-1 parent can have ordinary, box2 and box3 pool entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostPalComponent {
    PersonalProperty,
    DirectRental,
    FarmRental,
    PartnershipBusiness,
    PartnershipGroup { group: i64 },
    ScorpBusiness,
    ScorpGroup { group: i64 },
    PartnershipRealEstate,
    PartnershipOtherRental,
    ScorpRealEstate,
    ScorpOtherRental,
    EstateActivity { activity: i64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostPalCharacter {
    ScheduleE,
    PersonalExpense,
    OrdinaryProperty,
    PropertyDisposition,
    Section1231,
    ShortCapital,
    LongCapital,
    DebtCancellation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostPalCharacterAmounts {
    pub character: PostPalCharacter,
    pub net_cents: i64,
    pub suspended_cents: i64,
    pub allowed_net_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostPalSourceAmounts {
    pub income: i64,
    pub owner: crate::Owner,
    pub component: PostPalComponent,
    // False only for positive personal income recharacterized by its native
    // owner. Its known zero hold cannot offset another passive loss.
    pub common_pool: bool,
    // The common contribution, not the separate nonpassive/PTP or own-income
    // release. Those results remain with their native owners. Personal
    // and K-1 operands are actual cents; other adapters retain their annual boundary.
    pub net_cents: i64,
    // The incoming Form8582 prior column, not an inferred current/prior release.
    pub prior_cents: i64,
    pub suspended_cents: i64,
    pub allowed_net_cents: i64,
    pub characters: alloc::vec::Vec<PostPalCharacterAmounts>,
}

// One complete same-invocation result. Personal and K-1 inputs retain actual
// source cents; other owners retain their explicitly reported annual adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostPalSources {
    pub alternative: bool,
    pub total_loss_cents: i64,
    pub suspended_cents: i64,
    pub allowed_loss_cents: i64,
    pub special_allowance_cents: i64,
    pub sources: alloc::vec::Vec<PostPalSourceAmounts>,
}

fn post_pal_character(
    character: PostPalCharacter,
    net_cents: i64,
    suspended_cents: i64,
) -> PostPalCharacterAmounts {
    PostPalCharacterAmounts {
        character,
        net_cents,
        suspended_cents,
        allowed_net_cents: net_cents + suspended_cents,
    }
}

fn post_pal_class_amounts(
    character: PostPalCharacter,
    amount: &crate::kernel::loss_limits::CharacterSettlement<
        '_,
        usize,
        crate::kernel::carry::Cents,
    >,
) -> PostPalCharacterAmounts {
    PostPalCharacterAmounts {
        character,
        net_cents: amount.net,
        suspended_cents: amount.held(),
        allowed_net_cents: amount.allowed_net(),
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct RentalResult {
    // Required source failures travel with this invocation, including cached
    // owner snapshots; consumers withhold dependent amounts rather than using
    // an empty pool as a known-zero result.
    pub source_issue: Option<crate::activity_risk::SourceIssue>,
    pub pool_issue: Option<crate::kernel::loss_limits::LossLimitError>,
    // One actual return179 allocation for this source invocation. A supplied
    // selected-system result is never reconstructed from PAL or raw K-1 boxes.
    // None retains the legacy lazy owner call until that source is read.
    pub section179_allocations: Option<alloc::collections::BTreeMap<i64, i64>>,
    // Gross allowed net-loss components entering Form461, before unrelated
    // business profits offset them; dollars, owned by their actual source.
    pub business_losses_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub qbi_source_ids: alloc::vec::Vec<i64>,
    pub inputs_complete: bool,
    pub income: i64,
    // Allowed prior home-use casualty loss, separately from Schedule E; signed cents.
    pub prior_home_casualty_by_owner: [i64; 2],
    // Actual Schedule E source dollars after activity limits, before Form461.
    // Personal-property rental expenses remain their Schedule1 owner's work.
    pub income_by_id: alloc::collections::BTreeMap<i64, i64>,
    // The actual Form8960 line4b changes at each native activity invocation,
    // in already settled annual dollars, including known zero for each source.
    pub nii_adjustment_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub personal_inputs: alloc::collections::BTreeMap<i64, PersonalRentalInputs>,
    // One source per actual successful rental-use invocation in this tax view.
    // Missing source or final amount remains absent, never an inferred zero.
    pub operating_sources: alloc::collections::BTreeMap<i64, RentalOperatingSource>,
    // Actual Form4835 property sources after the same source's Form6198.
    // Native Form4797 dollars remain the recognized property boundary.
    pub farm_rental_property: alloc::collections::BTreeMap<i64, crate::farm_land::LandGain>,
    pub farm_rental_property_rows:
        alloc::collections::BTreeMap<i64, crate::asset_sale::FarmRentalPropertyRows>,
    pub operating_prior_allowed: alloc::collections::BTreeMap<i64, Option<i64>>,
    // None if a required pool source is incomplete. A missing row is never a
    // zero suspension; only a complete actual source can carry that result.
    pub post_pal: Option<PostPalSources>,
    // Admitted positive nonpassive rental source cents beyond the filing
    // projection. Passive loss pools and prior-loss releases still own their
    // annual-dollar result; do not reconstruct their residuals here.
    pub health_work_income_residual_cents: i64,
    // The net passive income of publicly traded partnerships, inside `income`: Form 4952
    // line 4a investment income (2026 draft i4952 line 4a; section 469(k)).
    pub ptp_income: i64,
    pub qbi: i64,
    pub passive_loss: i64,
    pub scorp_niit_adjustment: i64,
    pub partner_niit_adjustment: i64,
    // Form 8960 line 4b: a real-estate professional's safe-harbor rentals
    // (Reg. 1.1411-4(g)(7)) reversed out of the Schedule E net, regular tax.
    pub rental_niit_adjustment: i64,
    // Schedule K-1 (Form 1041) box 6 net after box 9 deductions (passive and
    // nonpassive), its nonpassive part, and the Form 8960 line 4b reversal of
    // that nonpassive part. Boxes 7 and 8 stay in `income` as passive rent.
    pub estate_activities: alloc::vec::Vec<crate::estate_trust::EstateActivityResult>,
    // Schedule E Part III columns (c), (d), (e), (f), retained from the
    // actual allowed activity rows, whole dollars.
    pub estate_columns: [i64; 4],
    pub estate_trust_business: i64,
    pub estate_trust_nonpassive: i64,
    pub estate_trust_niit_adjustment: i64,
    pub scorp_business: i64,
    // Actual final ordinary business amounts after basis/risk/PAL and the
    // existing179/care deductions, before461. This is a source retention
    // of ordinary_pass_through_business, not income_by_id's mixed classes.
    pub ordinary_business_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub partners: alloc::vec::Vec<PartnerPassiveResult>,
    // Each partnership and S corporation K-1's amount in Schedule E Part II
    // (box 1, guaranteed payments and the box 2 and 3 activities, less its
    // section 179) and the passive loss still suspended, by K-1 identity.
    pub k1_amounts: alloc::vec::Vec<K1Amount>,
    // What each K-1's basis and at-risk limits allowed and held back, by
    // class, in the regular tax system (unit B8).
    pub k1_ledgers: alloc::vec::Vec<(i64, crate::k1_loss::LossLedger)>,
    pub partner_se: alloc::vec::Vec<(i64, crate::partnership::PartnerSeResult)>,
    // Form8958 provenance for the actual partner on the other return. These
    // sources never enter this filer's SE, health or retirement deductions.
    pub counterpart_partner_se: alloc::vec::Vec<(i64, crate::partnership::PartnerSeResult)>,
    pub scorp_nonpassive: i64,
    // K-1 box 2 activities a real estate professional materially
    // participated in: nonpassive, so not passive investment income.
    pub pass_through_rental_nonpassive: i64,
    pub personal_expense_adjustment: i64,
    pub activities: alloc::vec::Vec<crate::business_rules::QbiActivity>,
    // Source identity for negative allowed QBI; no second QBI calculation.
    pub qbi_loss_sources: alloc::vec::Vec<i64>,
    pub scorp_suspensions:
        alloc::vec::Vec<(i64, crate::kernel::carry::Amount<crate::kernel::carry::Cents>)>,
    pub k1_section1231: alloc::vec::Vec<K1PropertyResult>,
    pub scorp_qbi_ordinary_suspensions:
        alloc::vec::Vec<(i64, crate::kernel::carry::Amount<crate::kernel::carry::Cents>)>,
    // Unit B9-1: what the passive rules hold back of the activity items (code
    // R, code S), whole dollars: the loss that does not reach Form 4797 or
    // Schedule D this year.
    pub k1_cuts: K1Cuts,
    // Kept rentals: Form 8582 disallowance of partial dispositions, by
    // income identity and Form 4797 Part II / Part I, whole dollars.
    pub rental_partial_cuts: alloc::vec::Vec<(i64, [i64; 2])>,
    // Schedule E line 26 before royalties: each rental property's income or
    // allowed loss (lines 21 and 22), whole dollars.
    pub part_one: i64,
    // Schedule E line 40: each Form 4835 farm rental's income or allowed
    // loss (line 32 or 34c), whole dollars.
    pub farm_rental: i64,
    // Form 8582's combined passive result before the loss limits: every
    // passive activity's net less its prior-year unallowed losses, plus
    // personal-property rental profit. Form 8582-CR line 6 is zero when this
    // is zero or less.
    pub net_passive_income: i64,
    // Form 8582 line 9: the losses of rental real estate with active
    // participation that the special allowance let you deduct. Form 8582-CR
    // line 35 takes it from the $25,000 ($12,500) allowance that remains.
    pub special_allowance: i64,
    // Form 8582 line 6 modified adjusted gross income, not below zero: Form
    // 8582-CR line 22 for the rehabilitation credit's special allowance.
    pub modified_agi: i64,
    pub health_ptc: HealthPassiveProjection,
}

impl RentalResult {
    pub(crate) fn section179_allowed_cents(&self, d: &FederalDraft, id: i64) -> i64 {
        self.section179_allocations.as_ref().map_or_else(
            || crate::section179::pass_through_allowed(d, id),
            |sources| sources.get(&id).copied().unwrap_or(0),
        )
    }

    // Final allowed personal-property cost from the same system's one PAL
    // invocation. Neither a missing source nor an incomplete invocation is zero.
    pub(crate) fn personal_expenses_cents(&self) -> Option<alloc::collections::BTreeMap<i64, i64>> {
        let packet = self.post_pal.as_ref()?;
        self.personal_inputs
            .iter()
            .map(|(&id, input)| {
                let source = packet.sources.iter().find(|source| {
                    source.income == id && source.component == PostPalComponent::PersonalProperty
                })?;
                Some((
                    id,
                    crate::kernel::carry::native_balance(
                        input.expenses_cents + input.prior_cents,
                        source.suspended_cents,
                    ),
                ))
            })
            .collect()
    }

    // Pub. 596 Worksheet 1 lines 11–13: exclude nonpassive amounts and
    // passive amounts already included in earned income. SE and participation
    // are independent classifications; the exclusions must not overlap.
    pub(crate) fn eic_passive_income(&self) -> i64 {
        self.income
            - self.scorp_nonpassive
            - self.pass_through_rental_nonpassive
            - self.estate_trust_nonpassive
            - self.partners.iter().map(|v| v.nonpassive).sum::<i64>()
            - self.partner_se.iter().map(|(_, v)| v.passive_earnings).sum::<i64>()
    }

    // Ordinary trade/business amounts after basis, risk and passive limits.
    // Do not use QBI eligibility or SE classification to decide Form 461 scope.
    pub(crate) fn ordinary_pass_through_business(&self) -> i64 {
        self.scorp_business
            + self.estate_trust_business
            + self.partners.iter().map(|v| v.net).sum::<i64>()
    }
}

// Source adapters attach presentation/tax facts before entering the passive
// pool. The allocator never recovers an activity's type by looking up its ID.
// The selected system's known eligible business income is used by loss
// limitations. Section199A's property projection belongs only to regular tax.
struct UniformBusinessIncome {
    income: i64,
    regular_qbi: Option<crate::business_rules::QbiActivity>,
}

struct PassiveEntry<'a> {
    id: i64,
    net: i64,
    // Actual K-1 owner-stage amounts. Annual fields are report projections.
    source_cents: Option<K1PassiveCents>,
    rental_allowance: bool,
    // The part of the activity's prior unallowed loss that the special
    // allowance does not admit: it arose in a year without active
    // participation (section 469(i)(1); Form 8582 Part V, Worksheet 3).
    ineligible_prior: i64,
    // The prior unallowed loss netted into `net` (Form 8582 column (c)); the
    // activity's current result is `net + prior`.
    prior: i64,
    personal: bool,
    scorp: Option<&'a crate::ScorpK1>,
    partner: bool,
    source_group: Option<i64>,
    characters: Option<crate::scorp::ActivityCharacters>,
    // Unit B9-1: the activity items (code E, R and S) in the activity's net,
    // and whether the net is in the pool (a passive activity) or they are
    // nonpassive.
    items: Option<(crate::k1_items::ActivityItems, bool)>,
    nonpassive: i64,
    prior_released: i64,
    // Only when every allowed dollar has the same QBI treatment. Mixed K-1
    // statements and historical vintages need their own post-limit allocation.
    uniform_business: Option<UniformBusinessIncome>,
    // Form 4797 gain of a rental sold in full this year (section 469(g)):
    // part of the activity's passive net, reported on Form 4797, not E.
    disposition_gain: i64,
    // A kept rental whose partial building dispositions share this pool entry.
    rental_partial: bool,
    // Schedule K-1 (Form 1041) boxes 6–8 passive income: Schedule E line 33
    // column (d), never Part I, never a rental allowance or suspended loss.
    estate_trust: bool,
    estate_activity: Option<(i64, &'a crate::EstateActivity)>,
    // A Form 4835 farm rental: its allowed amount is Schedule E line 40,
    // never Part I.
    farm_rental: bool,
    // A K-1 box 2 or box 3 rental activity (unit k1-rentals): Schedule E
    // line 28 of its K-1, never Part I.
    pass_through: Option<PassThroughRental>,
    released: bool,
    // A publicly traded partnership's passive net (section 469(k)): outside
    // the common pool. A gain is income, a loss stays suspended with its PTP.
    ptp: i64,
}

// Same-invocation forwarding only; the normal loss owners and passive pool
// compute these amounts. No reported-dollar reconstruction enters the pool.
struct K1PassiveCents {
    // Separately stated Schedule E items, in source order: guaranteed services,
    // guaranteed capital, prior fuel recovery, and the PTP's own section179.
    // These retain their distinct reported rows/column while stages keep cents.
    separate_nonpassive: [i64; 4],
    net: i64,
    prior: i64,
    ineligible_prior: i64,
    nonpassive: i64,
    prior_released: i64,
    disposition_gain: i64,
    ptp: i64,
}

#[derive(Clone)]
struct PassThroughRental {
    // "partnership" or "scorp", the K-1 kind's calculation-id prefix.
    kind: &'static str,
    real_estate: bool,
    material: bool,
    qbi: Option<crate::K1RentalQbi>,
    exclude_niit: bool,
    niit_disposition_loss: i64,
}

// K-1 box 2 and 3 activities of one K-1 in this pass (unit k1-rentals),
// with the regular-tax activities MAGI and the NIIT safe harbor read in both
// passes. Returns [nonpassive income, MAGI, passive profit, NIIT
// adjustment]. MAGI includes a professional's nonpassive income but not
// its allowed loss (i8582 line 6); passive amounts never enter it.
fn pass_through_rentals<'a>(
    values: &mut alloc::vec::Vec<PassiveEntry<'a>>,
    id: i64,
    kind: &'static str,
    current: alloc::vec::Vec<crate::k1_rental::RentalActivity>,
    regular: &[crate::k1_rental::RentalActivity],
    niit_safe_harbor: bool,
    grouped: bool,
) -> [i64; 4] {
    let mut sums = [0; 4];
    for activity in regular {
        if !grouped || !activity.real_estate {
            sums[1] +=
                if activity.released { activity.nonpassive } else { positive(activity.nonpassive) };
        }
        if niit_safe_harbor && activity.material {
            sums[3] -= activity.characters.map_or(activity.nonpassive, |parts| parts.nonpassive[0])
                + activity.niit_disposition_loss[0];
        }
    }
    for activity in current {
        sums[0] += activity.nonpassive;
        sums[2] += positive(activity.passive);
        values.push(PassiveEntry::pass_through(id, kind, activity, niit_safe_harbor));
    }
    sums.map(dollars)
}

impl PassiveEntry<'_> {
    fn component(&self) -> PostPalComponent {
        if self.personal {
            PostPalComponent::PersonalProperty
        } else if let Some((_, activity)) = self.estate_activity {
            PostPalComponent::EstateActivity { activity: activity.id }
        } else if let Some(rental) = &self.pass_through {
            match (rental.kind, rental.real_estate) {
                ("partnership", true) => PostPalComponent::PartnershipRealEstate,
                ("partnership", false) => PostPalComponent::PartnershipOtherRental,
                ("scorp", true) => PostPalComponent::ScorpRealEstate,
                ("scorp", false) => PostPalComponent::ScorpOtherRental,
                _ => unreachable!("native K1 rental source kind"),
            }
        } else if let Some(group) = self.source_group {
            if self.partner {
                PostPalComponent::PartnershipGroup { group }
            } else {
                PostPalComponent::ScorpGroup { group }
            }
        } else if self.partner {
            PostPalComponent::PartnershipBusiness
        } else if self.scorp.is_some() {
            PostPalComponent::ScorpBusiness
        } else if self.farm_rental {
            PostPalComponent::FarmRental
        } else {
            PostPalComponent::DirectRental
        }
    }

    fn real_estate(&self) -> bool {
        self.pass_through.as_ref().map_or(
            !self.personal && self.scorp.is_none() && !self.partner && !self.estate_trust,
            |rental| rental.real_estate,
        )
    }

    // One K-1 rental activity, its loss limits already applied.
    fn pass_through(
        id: i64,
        kind: &'static str,
        activity: crate::k1_rental::RentalActivity,
        niit_safe_harbor: bool,
    ) -> Self {
        PassiveEntry {
            source_cents: Some(K1PassiveCents {
                separate_nonpassive: [0; 4],
                net: activity.passive,
                prior: activity.prior,
                ineligible_prior: activity.ineligible_prior,
                nonpassive: activity.nonpassive,
                prior_released: activity.prior_released,
                disposition_gain: activity.property_gain,
                ptp: 0,
            }),
            id,
            net: dollars(activity.passive),
            rental_allowance: activity.allowance,
            ineligible_prior: dollars(activity.ineligible_prior),
            prior: dollars(activity.prior),
            personal: false,
            scorp: None,
            partner: false,
            source_group: None,
            characters: activity.characters,
            items: None,
            nonpassive: dollars(activity.nonpassive),
            prior_released: dollars(activity.prior_released),
            uniform_business: None,
            disposition_gain: dollars(activity.property_gain),
            rental_partial: false,
            estate_trust: false,
            estate_activity: None,
            farm_rental: false,
            ptp: 0,
            released: activity.released,
            pass_through: Some(PassThroughRental {
                kind,
                real_estate: activity.real_estate,
                material: activity.material,
                qbi: activity.qbi,
                exclude_niit: niit_safe_harbor && activity.material,
                niit_disposition_loss: activity.niit_disposition_loss[1],
            }),
        }
    }
}

// The page that records a filer's real estate professional status: the
// owner's own, as a correction of the rental or K-1 that needs it.
pub(crate) fn status_context(
    d: &FederalDraft,
    e: &crate::FederalIncomeEntry,
) -> crate::IssueContext {
    crate::interview::income_issue_context(e).corrective(
        if crate::community::professional_owner(d.setup.as_ref(), e.owner) == Owner::Spouse {
            crate::TaxSection::Spouse
        } else {
            crate::TaxSection::People
        },
    )
}

// All supported rentals are passive; dispositions and changes of historical character are screened.
// Special allowance applies only to active-participation loss activities. Other rental
// profit offsets passive losses before the allowance; portfolio income never does.
#[cfg(test)]
pub(crate) fn rentals(d: &FederalDraft, magi: i64, r: &mut FederalResult) -> RentalResult {
    rentals_for(d, magi, r, false)
}

// 26 CFR 1.55-1(b): use regular-tax MAGI for the allowance in both engines.
// Only the activity amounts and suspended-loss balances differ for AMT.
#[cfg(test)]
pub(crate) fn rentals_for(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
) -> RentalResult {
    rentals_for_home(d, magi, r, amt, [magi, magi])
}
#[cfg(test)]
pub(crate) fn rentals_for_home(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_agi: [i64; 2],
) -> RentalResult {
    rentals_for_home_with_care(d, magi, r, amt, home_agi, &crate::care::CareDeductions::default())
}
#[cfg(test)]
pub(crate) fn rentals_for_home_with_care(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_agi: [i64; 2],
    care: &crate::care::CareDeductions,
) -> RentalResult {
    rentals_for_home_with_section179(d, magi, r, amt, home_agi, (care, None))
}

// The selected allocator owns amounts and availability; this adapter only
// forwards that same result. The legacy entry keeps its original lazy regular
// owner calls, including its existing unresolved PTP/AMT routing boundaries.
#[cfg(test)]
pub(crate) fn rentals_for_home_with_section179(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_agi: [i64; 2],
    sources: (&crate::care::CareDeductions, Option<&crate::Section179PreparedAllocation>),
) -> RentalResult {
    rentals_with_prepared_sources(d, magi, r, amt, home_agi, sources, None, None, None)
}

// Forward the existing invocation operands and its borrowed home context together.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rentals_for_home_with_selected_section179(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_agi: [i64; 2],
    care: &crate::care::CareDeductions,
    selected: Option<&[crate::section179_return::Selected; 2]>,
    home: Option<&crate::home_office::HomeContext>,
) -> RentalResult {
    let regular_section179 = selected.map(|sources| &sources[0].allocation);
    let selected = selected.map(|sources| &sources[usize::from(amt)]);
    let native = if selected.is_none() && !d.home_sites.is_empty() {
        let source = home
            .ok_or(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                0,
                "HOME_CONTEXT_REQUIRED",
            ))
            .and_then(|home| crate::section179::calculate_with_home(d, home));
        match source {
            Ok(source) => Some(source.pass_through),
            Err(issue) => {
                let (id, key) = match &issue {
                    crate::Section179PreparedIssue::SourceBoundaryUnavailable(id, key)
                    | crate::Section179PreparedIssue::InvalidSource(id, key) => (*id, *key),
                    _ => (0, "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE"),
                };
                crate::section179_return::Issue::Prepared(issue).report(r);
                return RentalResult {
                    source_issue: Some(crate::activity_risk::SourceIssue::new(
                        id,
                        crate::activity_risk::ProjectionIssue::Source(key),
                    )),
                    ..Default::default()
                };
            }
        }
    } else {
        None
    };
    rentals_with_prepared_sources(
        d,
        magi,
        r,
        amt,
        home_agi,
        (care, selected.map(|source| &source.allocation)),
        selected.map(|source| &source.ptp),
        native,
        regular_section179,
    )
}

// Forward the existing invocation operands and its borrowed home context together.
#[allow(clippy::too_many_arguments)]
fn rentals_with_prepared_sources(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_agi: [i64; 2],
    sources: (&crate::care::CareDeductions, Option<&crate::Section179PreparedAllocation>),
    selected_ptp: Option<&alloc::collections::BTreeMap<i64, crate::ptp_passive::PtpPalResult>>,
    native_section179: Option<alloc::collections::BTreeMap<i64, i64>>,
    regular_section179: Option<&crate::Section179PreparedAllocation>,
) -> RentalResult {
    let (care, section179) = sources;
    let selected_ptp = |id: i64| selected_ptp.and_then(|sources| sources.get(&id));
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
            .get_or_init(|| crate::section179::calculate(d).pass_through)
            .get(&id)
            .copied()
            .unwrap_or(0)
    };
    let allocated_entries = crate::community::entries(d);
    let initial_issues = r.issues.len();
    let mut prior_home_casualty_by_owner = [0i64; 2];
    let mut source_issue = None;
    macro_rules! rental_source {
        ($value:expr) => {
            match $value {
                Ok(value) => value,
                Err(issue) => {
                    issue.report(r);
                    source_issue.get_or_insert(issue);
                    continue;
                }
            }
        };
    }
    macro_rules! rental_amount {
        ($id:expr, $value:expr) => {
            match $value {
                Ok(value) => value,
                Err(issue) => {
                    issue.report($id, r);
                    source_issue.get_or_insert(crate::activity_risk::SourceIssue::new($id, issue));
                    continue;
                }
            }
        };
    }

    if let Some(key) = rental_mortgage_rule(d) {
        for section in [crate::TaxSection::Mortgage, crate::TaxSection::Rentals] {
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(section),
                "Rental mortgage loans",
                key,
                "Correct the rental's canonical saved Mortgage loans, duplicate allocations or points schedule before estimating.",
            );
        }
    }

    let mid_quarter = crate::depreciation::mid_quarter(d);
    // The election is annual taxpayer-level grouping, not a rewrite of any
    // accepted source history. AMT projects its own amounts; MAGI and NIIT
    // continue to read the regular-tax group.
    let professional_groups = match professional_former_passive(d, amt, home_agi) {
        Ok(value) => value,
        Err(issue) => {
            issue.report(r);
            return RentalResult { source_issue: Some(issue), ..Default::default() };
        }
    };
    let regular_groups = if amt {
        match professional_former_passive(d, false, home_agi) {
            Ok(value) => value,
            Err(issue) => {
                issue.report(r);
                return RentalResult { source_issue: Some(issue), ..Default::default() };
            }
        }
    } else {
        professional_groups.clone()
    };
    // Retain the actual system-specific cost/history before the existing
    // annual-dollar stages. Never recover these cents from a rounded net.
    let personal_inputs: alloc::collections::BTreeMap<_, _> = allocated_entries
        .iter()
        .filter_map(|entry| {
            let FederalIncome::NonbusinessProperty {
                kind: kind @ NonbusinessPropertyKind::PersonalRental { .. },
                receipts,
                expenses,
                ..
            } = &entry.income
            else {
                return None;
            };
            let regular_expenses_cents = *expenses + kind.rental_depreciation(mid_quarter, false);
            let expenses_cents = if amt {
                *expenses + kind.rental_depreciation(mid_quarter, true)
            } else {
                regular_expenses_cents
            };
            Some((
                entry.id,
                PersonalRentalInputs {
                    owner: entry.owner,
                    receipts_cents: *receipts,
                    regular_expenses_cents,
                    expenses_cents,
                    regular_prior_cents: kind.prior_rental_loss(false),
                    prior_cents: kind.prior_rental_loss(amt),
                    investment_recharacterization: kind.investment_rental(),
                },
            ))
        })
        .collect();
    // Schedule 1 supplies personal-property receipts, paid expenses and depreciation.
    // A nondepreciable-property loss remains passive: only positive income
    // is recharacterized. Current losses have no real-estate special allowance.
    let personal: alloc::vec::Vec<_> = allocated_entries
        .iter()
        .filter_map(|e| {
            let input = personal_inputs.get(&e.id)?;
            let prior = input.prior_cents;
            let net = input.receipts_cents - input.expenses_cents - prior;
            (net <= 0 || !input.investment_recharacterization).then_some((e.id, net, prior))
        })
        .collect();
    let personal_profit_cents = personal.iter().map(|(_, net, _)| positive(*net)).sum::<i64>();
    let personal_profit = dollars(personal_profit_cents);
    // MAGI always uses regular-tax history, even in the AMT pass. For a
    // passive activity remove its raw current net. For recharacterized profit
    // retain only the regular net after its own prior loss has been absorbed.
    let mut magi_adjustment_cents = 0;
    for e in &allocated_entries {
        if let FederalIncome::NonbusinessProperty {
            kind: kind @ NonbusinessPropertyKind::PersonalRental { .. },
            receipts,
            ..
        } = &e.income
        {
            let input = &personal_inputs[&e.id];
            let current = *receipts - input.regular_expenses_cents;
            let prior = input.regular_prior_cents;
            magi_adjustment_cents +=
                if kind.investment_rental() { phase::cap(current, [prior]) } else { current };
        }
    }
    let mut magi = magi - dollars(magi_adjustment_cents) + regular_groups.magi;
    let prefix = if amt { "amt_" } else { "" };
    let mut nonpassive_income = 0;
    let mut health_work_income_residual_cents = 0;
    // Publicly traded partnerships' passive net income and suspended losses
    // (section 469(k)), kept outside the common pool.
    let mut ptp_income = 0;
    let mut ptp_suspended = 0;
    let mut part_one = 0;
    let mut income_by_id: alloc::collections::BTreeMap<i64, i64> = allocated_entries
        .iter()
        .filter(|e| {
            matches!(
                e.income,
                FederalIncome::Rental { .. }
                    | FederalIncome::FarmRental { .. }
                    | FederalIncome::Partnership { .. }
                    | FederalIncome::Scorp { .. }
                    | FederalIncome::EstateTrustPortfolio { .. }
            )
        })
        .map(|e| (e.id, 0))
        .collect();
    let mut nii_adjustment_by_id: alloc::collections::BTreeMap<i64, i64> =
        income_by_id.keys().map(|id| (*id, 0)).collect();
    let mut operating_sources = alloc::collections::BTreeMap::new();
    let mut operating_prior_allowed = alloc::collections::BTreeMap::new();
    let mut profits = 0;
    let mut values: alloc::vec::Vec<_> = personal
        .iter()
        .filter(|(_, net, _)| *net < 0)
        .map(|(id, net, prior)| PassiveEntry {
            source_cents: None,
            id: *id,
            net: dollars(*net),
            rental_allowance: false,
            ineligible_prior: 0,
            prior: dollars(*prior),
            personal: true,
            scorp: None,
            partner: false,
            source_group: None,
            characters: None,
            items: None,
            nonpassive: 0,
            prior_released: 0,
            uniform_business: None,
            disposition_gain: 0,
            rental_partial: false,
            estate_trust: false,
            estate_activity: None,
            farm_rental: false,
            ptp: 0,
            pass_through: None,
            released: false,
        })
        .collect();
    let mut home_activities = alloc::vec::Vec::new();
    let mut released_rental_losses = false;
    let mut nonpassive_rental_losses = false;
    // Sec. 469(c)(7): a qualified real-estate professional (the People
    // page's status) who materially participates in a rental removes that
    // rental from the passive rules entirely (no Form 8582 allowance; the loss
    // offsets other income in full, subject to the at-risk and
    // excess-business-loss rules computed elsewhere). Each rental, and each
    // K-1 box 2 activity, records the material participation (or the
    // aggregation election) itself: RentalParticipation::Professional.
    // Reg. 1.1411-4(g)(7): a qualified professional with more than 500 hours
    // in each rental activity this year (or in 5 of the last 10 years) earns
    // the rent in a trade or business; it leaves net investment income.
    let mut rental_niit_adjustment = 0;
    for e in &allocated_entries {
        if let FederalIncome::Rental { data: original } = &e.income {
            let source_before = part_one;
            let source_niit_before = rental_niit_adjustment;
            let projected = original.clone();
            let whole_entry = d.income.iter().find(|entry| entry.id == e.id).expect("source ID");
            let expense_rows = rental_expense_rows(d, whole_entry);
            let property_entry = crate::community::property_entry(d, whole_entry);
            let FederalIncome::Rental { data: property } = &property_entry.income else {
                unreachable!("rental source")
            };
            let v = &projected;
            let label = alloc::format!("Rental: {}", e.label);
            if !amt && straight_lined_points(d, original) {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_RENTAL_POINTS_STRAIGHT_LINE",
                        alloc::vec![crate::MessageArg::Text(label.clone())],
                    ),
                    crate::interview::income_issue_context(e),
                );
            }
            let status =
                d.professional(crate::community::professional_owner(d.setup.as_ref(), e.owner));
            let re_professional = v.participation.professional()
                && status.is_some()
                && v.property_type.business_participation().is_none();
            // The recorded status is the filer's, not a rental's: a rental
            // that says Professional without it is not nonpassive. A
            // short-stay business variant is not rental real estate, so the
            // participation answer does not apply to it (Reg. 1.469-1T(e)(3)(ii)).
            if !amt {
                crate::require_localized!(
                    r,
                    !v.participation.professional()
                        || re_professional
                        || v.property_type.business_participation().is_some(),
                    &status_context(d, e),
                    &label,
                    "ISSUE_RENTAL_PROFESSIONAL_NEEDS_STATUS",
                    "A rental marked as one in which a real estate professional materially participated needs that status recorded for its owner on the People page.",
                );
            }
            let niit_safe_harbor = re_professional && status.is_some_and(|p| p.over_500_hours);
            let business_participation = v.property_type.business_participation();
            let allowance = business_participation.is_none() && v.participation.active();
            if !amt {
                crate::asset_sale::report_rental(d, r, e.id, &e.label, property);
            }
            if v.property_type.substantial_services() {
                if !amt {
                    crate::issue_localized!(
                        r,
                        &crate::interview::income_issue_context(e),
                        &label,
                        "ISSUE_SHORT_TERM_RENTAL_IS_A_BUSINESS",
                        "Substantial guest services belong on Schedule C. Enter the property in Business and remove this rental.",
                    );
                    crate::issue_localized!(
                        r,
                        &crate::interview::income_issue_context(e)
                            .corrective(crate::TaxSection::SelfEmployment),
                        &label,
                        "ISSUE_SHORT_TERM_RENTAL_IS_A_BUSINESS",
                        "Substantial guest services belong on Schedule C. Enter the property in Business and remove this rental.",
                    );
                }
                continue;
            }
            // How the property left, read once: the exchange (if any) whose
            // property received continues this rental.
            let exchange = v.sale.exchange();
            let itemizing = d.deductions.as_ref().is_none_or(|ded| {
                !matches!(
                    ded.choice,
                    crate::DeductionSelection::Standard
                        | crate::DeductionSelection::Automatic {
                            expenses: crate::ItemizedChoice::No
                        }
                )
            });
            match rental_use_core(
                v,
                amt,
                mid_quarter,
                itemizing,
                home_fixed_allowed(d, e.id, home_agi, amt),
                &expense_rows,
                Some((d, e.id, false)),
            ) {
                Ok(mut usage) => {
                    if let Some(tiers) = usage.home_third_tier {
                        if crate::rental_home_carry::total(v, amt).is_some_and(|amount| amount > 0)
                        {
                            for (suffix, amount, explanation) in [
                                (
                                    "home_third_tier_allowed",
                                    Some(tiers.total_allowed_cents),
                                    msg!("EXPLAIN_RENTAL_HOME_THIRD_TIER_ALLOWED"),
                                ),
                                (
                                    "home_casualty_allowed",
                                    tiers.allowed_casualty_cents(),
                                    msg!("EXPLAIN_RENTAL_HOME_CASUALTY_ALLOWED"),
                                ),
                                (
                                    "home_casualty_carryover",
                                    tiers.carry.casualty_cents,
                                    msg!("EXPLAIN_RENTAL_HOME_CASUALTY_CARRY"),
                                ),
                            ] {
                                if let Some(amount) = amount {
                                    push_line(
                                        r,
                                        None,
                                        &alloc::format!("{prefix}rental_{}_{suffix}", e.id),
                                        amount,
                                        &["income.rental.home_depreciation_carryover"],
                                        RENTAL,
                                        explanation,
                                    );
                                }
                            }
                        }
                        let casualty = tiers.allowed_casualty_cents();
                        // The combined balance remains known without its character;
                        // withhold only the dependent separated deduction/return.
                        let boundary = if casualty.is_none() {
                            Some(crate::rental_home_carry::CHARACTER)
                        } else if !usage.home && casualty.is_some_and(|amount| amount != 0) {
                            // The original casualty's recurring/nonrecurring passive
                            // character is not established by a combined prior carry.
                            Some("RENTAL_HOME_PRIOR_CASUALTY_ACTIVITY_UNAVAILABLE")
                        } else if matches!(v.qbi_business, RentalBusiness::Yes { .. })
                            && casualty.is_some_and(|amount| amount != 0)
                        {
                            Some("RENTAL_HOME_PRIOR_CASUALTY_QBI_UNAVAILABLE")
                        } else {
                            None
                        };
                        if let Some(key) = boundary {
                            carryover_rows(r, prefix, e.id, usage.carry);
                            crate::issue_localized!(
                                r,
                                &crate::interview::income_issue_context(e),
                                &label,
                                key,
                                "Supply the actual character of the prior home-use casualty carry before separated deductions are available."
                            );
                            source_issue.get_or_insert(crate::activity_risk::SourceIssue::new(
                                e.id,
                                crate::activity_risk::ProjectionIssue::Source(key),
                            ));
                            continue;
                        }
                        if let Some(casualty) = casualty.filter(|amount| *amount != 0) {
                            // HOME's casualty allowance goes to SectionB/Form4797;
                            // it is excluded from ScheduleE depreciation and net.
                            let owner = usize::from(e.owner == crate::Owner::Spouse);
                            let projected = usage
                                .net
                                .checked_add(casualty)
                                .zip(prior_home_casualty_by_owner[owner].checked_sub(casualty));
                            let Some((net, owner_casualty)) = projected else {
                                let issue = crate::activity_risk::ProjectionIssue::Arithmetic(
                                    crate::kernel::loss_limits::LossLimitError::Overflow,
                                );
                                issue.report(e.id, r);
                                source_issue.get_or_insert(crate::activity_risk::SourceIssue::new(
                                    e.id, issue,
                                ));
                                continue;
                            };
                            usage.net = net;
                            prior_home_casualty_by_owner[owner] = owner_casualty;
                        }
                    }
                    let prior_passive = [v.history.balance(false), v.history.balance(true)];
                    let ineligible = if allowance && !v.history.active_in_loss_year {
                        v.history.balance(amt)
                    } else {
                        0
                    };
                    // The regular-only QBI projection requires its original GDS
                    // period; AMT business-income bookkeeping needs no UBIA.
                    let ubia = if !amt && matches!(v.qbi_business, RentalBusiness::Yes { .. }) {
                        match (|| -> Result<i64, (&'static str, &'static str)> {
                            Ok(match &v.sale {
                                // Sold property is not held at year-end.
                                crate::RentalSale::Sold { .. } => 0,
                                _ => match exchange {
                                    // The property received continues the
                                    // buildings given up (Reg. 1.199A-2(c)(3)).
                                    Some(exchange) => exchange.ubia(v)?,
                                    None => v.buildings.iter().try_fold(0, |basis, b| {
                                        let recovery = crate::historical_recovery::rental_building_recovery(
                                            &v.property_type, b.in_service, b.commercial_1993_transition.as_ref(), false,
                                        )?;
                                        Ok::<_, (&'static str, &'static str)>(basis + if b.casualty.as_ref().is_none_or(|source| matches!(source.casualty.kind, crate::CasualtyKind::Damaged { retired: None, .. })) && crate::kernel::cost_recovery::historical::rental_elapsed_half_months(b.in_service, 2026) < recovery.half_months() {
                                            b.cost - b.land
                                        } else { 0 })
                                    })?,
                                },
                            })
                        })() {
                            Ok(value) => Some(value),
                            Err((key, _message)) => {
                                #[cfg(test)]
                                let message = _message;
                                crate::issue_localized!(
                                    r,
                                    &crate::interview::income_issue_context(e),
                                    &label,
                                    key,
                                    message
                                );
                                continue;
                            }
                        }
                    } else {
                        None
                    };
                    let qbi_activity = |income: i64| {
                        matches!(v.qbi_business, RentalBusiness::Yes { .. }).then(|| {
                            UniformBusinessIncome {
                                income,
                                regular_qbi: (!amt).then(|| crate::business_rules::QbiActivity {
                                    source_wages_cents: None,
                                    source_property_cents: None,
                                    source_income_cents: None,
                                    owner: Some(e.owner),
                                    income,
                                    wages: dollars(v.payroll.qbi_wages()),
                                    property: dollars(
                                        ubia.expect(
                                            "regular QBI requires its original recovery source",
                                        ) + v
                                            .assets
                                            .iter()
                                            .map(|a| {
                                                if a.qbi_property() {
                                                    a.business_basis()
                                                } else {
                                                    0
                                                }
                                            })
                                            .sum::<i64>(),
                                    ),
                                    active: matches!(
                                        v.qbi_business,
                                        RentalBusiness::Yes { material_participation: true, .. }
                                    ),
                                    service: Some(false),
                                    group: v.qbi_aggregation_group.clone(),
                                }),
                            }
                        })
                    };
                    // The Form 4797 gain of a sold or exchanged rental, and whether
                    // its disposition is fully taxable to an unrelated person: only
                    // that releases the suspended losses (section 469(g)(1)(A)). A
                    // like-kind exchange's recognized gain is passive income of
                    // the activity, whose losses stay with the property received.
                    let source_gain =
                        rental_source!(crate::asset_sale::rental_source(d, e.id, amt))
                            .map(|value| value.gain);
                    let releases_losses = exchange
                        .is_none_or(|exchange| exchange.releases_losses())
                        && professional_group_disposed(d, e.owner);
                    // Form 6198 precedes section 469. Its prior disallowance is
                    // a current line 19 expense; the remaining shortfall carries.
                    let before_at_risk_cents = usage.net;
                    let raw_sale = rental_amount!(
                        e.id,
                        crate::asset_sale::rental_sale_amount(property, mid_quarter, amt)
                    );
                    match rental_at_risk(v, e.id, raw_sale, amt, &mut usage, r) {
                        Ok(prior) => {
                            operating_prior_allowed.insert(e.id, prior);
                        }
                        Err(issue) => {
                            issue.report(e.id, r);
                            source_issue
                                .get_or_insert(crate::activity_risk::SourceIssue::new(e.id, issue));
                            continue;
                        }
                    }
                    let source = RentalOperatingSource {
                        source: e.id,
                        owner: e.owner,
                        alternative: amt,
                        before_at_risk_cents,
                        after_at_risk_cents: usage.net,
                        prior_at_risk_cents: usage.prior_at_risk_cents,
                        prior_passive_cents: prior_passive[usize::from(amt)],
                        prior_home_carry_cents: [
                            v.home_operating_carryover,
                            crate::rental_home_carry::total(v, amt)
                                .expect("completed rental source"),
                        ],
                        home_limited: usage.home,
                        excluded: usage.excluded,
                        property_disposition: !matches!(v.sale, crate::RentalSale::No),
                        filed_ordinary_dollars: None,
                    };
                    let net_before_at_risk = source.before_at_risk_cents;
                    operating_sources.insert(e.id, source);
                    if usage.personal_fixed > exact::whole(0) {
                        r.review_notes.push(crate::federal_model::ReviewNote::localized(
                            #[cfg(test)] alloc::format!("{label}: eligible personal mortgage interest, points, insurance and property taxes flow to Schedule A automatically when you itemize. Do not enter them again."),
                            crate::Message::new("NOTE_RENTAL_PERSONAL_EXPENSE_SHARE", alloc::vec![crate::MessageArg::Text(label.clone())]),
                        ));
                    }
                    if !usage.home
                        && (v.home_operating_carryover != 0
                            || crate::rental_home_carry::total(v, amt) != Some(0))
                    {
                        // Worksheet 5-1 lines 7a and 7b: what the rent left
                        // unused still carries to 2027 in a year that is not a home.
                        carryover_rows(r, prefix, e.id, usage.carry);
                    }
                    if usage.home {
                        // Pub 527 ch. 5: renting a home is not a passive activity; the
                        // Worksheet 5-1 limit replaces the passive-loss rules.
                        let released = if !matches!(v.sale, crate::RentalSale::No)
                            && source_gain.is_some()
                            && releases_losses
                        {
                            dollars(prior_passive[usize::from(amt)])
                        } else {
                            0
                        };
                        let net = dollars(usage.net) - released;
                        line(
                            r,
                            &alloc::format!("{prefix}rental_{}_suspended_loss", e.id),
                            dollars(prior_passive[usize::from(amt)]) - released,
                            &["income.rental"],
                            PASSIVE,
                            msg!("EXPLAIN_0RENTAL_N_SUSPENDED_LOSS"),
                        );
                        if !amt && (net < 0 || released > 0) {
                            form_line(
                                r,
                                FormLine::new(TaxForm::ScheduleE, "22"),
                                &alloc::format!("rental_{}_line22_allowed_loss", e.id),
                                phase::cap(net, [0]) - if net >= 0 { released } else { 0 },
                                &["income.rental"],
                                PASSIVE,
                                msg!("EXPLAIN_RENTAL_N_LINE22_NONPASSIVE_LOSS"),
                            );
                        }
                        // MAGI always uses the regular-tax amount.
                        let regular = if amt {
                            dollars(
                                rental_amount!(
                                    e.id,
                                    rental_use_core(
                                        v,
                                        false,
                                        mid_quarter,
                                        itemizing,
                                        home_fixed_allowed(d, e.id, home_agi, false),
                                        &expense_rows,
                                        Some((d, e.id, false)),
                                    )
                                    .map_err(|(key, _)| {
                                        crate::activity_risk::ProjectionIssue::Source(key)
                                    })
                                )
                                .net,
                            )
                        } else {
                            net
                        };
                        nonpassive_income += net;
                        if usage.net >= 0 && prior_passive[usize::from(amt)] == 0 {
                            health_work_income_residual_cents +=
                                usage.net - dollars(usage.net) * 100;
                        }
                        part_one += net;
                        magi += regular;
                        if let Some(activity) = qbi_activity(net) {
                            home_activities.push((e.id, activity));
                        }
                        carryover_rows(r, prefix, e.id, usage.carry);
                        if usage.excluded {
                            r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!("{label}: used as a home and rented fewer than 15 days. The rent is not income and no rental expenses or depreciation are deducted."), crate::Message::new("NOTE_LABEL_USED_AS_A_HOME_AND_RENTED_FEWER_THAN_15_DAYS_THE_R_1B4E3107", alloc::vec![crate::MessageArg::Text(alloc::string::String::from(&label))])));
                        }
                    } else if re_professional
                        || business_participation == Some(true)
                        || (dollars(usage.net) > 0
                            && (v.property_type == RentalPropertyType::SelfRental
                                || nondepreciable(v)))
                    {
                        let net = dollars(usage.net);
                        let prior = dollars(prior_passive[usize::from(amt)]);
                        // The activity's prior unallowed loss offsets its own
                        // income first, for a former passive activity (section
                        // 469(f)(1)(A)) and for recharacterized self-rental or
                        // nondepreciable-property income alike: only "net
                        // passive income (including prior-year unallowed
                        // losses)" is nonpassive (Pub. 925; Reg. 1.469-2(f)(6),
                        // 1.469-2T(d)(1)). A fully taxable disposition releases
                        // the rest (section 469(g)).
                        let grouped = professional_groups.source(e.id);
                        let regular_grouped = regular_groups.source(e.id);
                        let released = if let Some(grouped) = grouped {
                            dollars(grouped.parts.prior_released.iter().sum())
                        } else if !matches!(v.sale, crate::RentalSale::No)
                            && source_gain.is_some()
                            && releases_losses
                        {
                            prior
                        } else {
                            phase::cap(prior, [positive(net)])
                        };
                        let nonpassive = net - released;
                        nonpassive_income += nonpassive;
                        if usage.net >= 0 && prior_passive[usize::from(amt)] == 0 {
                            health_work_income_residual_cents += usage.net - net * 100;
                        }
                        part_one += nonpassive;
                        nonpassive_rental_losses |= net < 0;
                        // MAGI always uses the regular-tax amount.
                        let regular = if amt {
                            dollars(
                                rental_amount!(
                                    e.id,
                                    rental_use_core(
                                        v,
                                        false,
                                        mid_quarter,
                                        itemizing,
                                        home_fixed_allowed(d, e.id, home_agi, false),
                                        &expense_rows,
                                        Some((d, e.id, false)),
                                    )
                                    .map_err(|(key, _)| {
                                        crate::activity_risk::ProjectionIssue::Source(key)
                                    })
                                )
                                .net,
                            )
                        } else {
                            net
                        };
                        magi += if regular_grouped.is_some() {
                            0
                        } else if business_participation.is_some() {
                            regular - phase::cap(dollars(prior_passive[0]), [positive(regular)])
                        } else {
                            positive(
                                regular
                                    - phase::cap(dollars(prior_passive[0]), [positive(regular)]),
                            )
                        };
                        // A prior loss the activity's own income or disposition
                        // absorbs in full never enters Form 8582 (i8582 "Former
                        // passive activities"; Pub. 925 "Recharacterization of
                        // Passive Income"): line 22 reports it here, and the
                        // pool reports only a remaining passive loss.
                        let own_release = if prior <= released { released } else { 0 };
                        if !amt && prior <= released && (net < 0 || own_release > 0) {
                            form_line(
                                r,
                                FormLine::new(TaxForm::ScheduleE, "22"),
                                &alloc::format!("rental_{}_line22_allowed_loss", e.id),
                                phase::cap(net, [0]) - own_release,
                                &["income.rental"],
                                PASSIVE,
                                msg!("EXPLAIN_RENTAL_N_LINE22_NONPASSIVE_LOSS"),
                            );
                        }
                        if niit_safe_harbor
                            || v.property_type == RentalPropertyType::SelfRental
                            || (business_participation == Some(true)
                                && matches!(v.qbi_business, RentalBusiness::Yes { .. }))
                        {
                            rental_niit_adjustment -= regular
                                - regular_grouped.map_or(released, |grouped| {
                                    dollars(grouped.income_release.iter().sum())
                                });
                        }
                        if prior > 0 && prior <= released {
                            line(
                                r,
                                &alloc::format!("{prefix}rental_{}_suspended_loss", e.id),
                                0,
                                &["income.rental"],
                                PASSIVE,
                                msg!("EXPLAIN_0RENTAL_N_SUSPENDED_LOSS_2"),
                            );
                        } else if prior > 0 {
                            values.push(PassiveEntry {
                                source_cents: None,
                                id: e.id,
                                net: -(prior - released),
                                prior,
                                rental_allowance: business_participation.is_none() && allowance,
                                // The own-income release is already outside the pool.
                                // Only the remaining prior loss can be ineligible
                                // for the special allowance; the absorbed portion
                                // must not become apparent active profit.
                                ineligible_prior: phase::cap(
                                    dollars(ineligible),
                                    [prior - released],
                                ),
                                personal: false,
                                scorp: None,
                                partner: false,
                                source_group: None,
                                characters: None,
                                nonpassive,
                                prior_released: released,
                                uniform_business: qbi_activity(-(prior - released)),
                                disposition_gain: 0,
                                rental_partial: false,
                                estate_trust: false,
                                estate_activity: None,
                                farm_rental: false,
                                ptp: 0,
                                pass_through: None,
                                released: false,
                                items: None,
                            });
                        }
                        if let Some(activity) = qbi_activity(nonpassive) {
                            home_activities.push((e.id, activity));
                        }
                    } else if business_participation == Some(false) {
                        let prior = dollars(prior_passive[usize::from(amt)]);
                        let net = dollars(usage.net) - prior + source_gain.unwrap_or(0);
                        profits += positive(net);
                        values.push(PassiveEntry {
                            source_cents: None,
                            id: e.id,
                            net,
                            prior,
                            rental_allowance: false,
                            ineligible_prior: 0,
                            personal: false,
                            scorp: None,
                            partner: false,
                            source_group: None,
                            characters: None,
                            nonpassive: 0,
                            prior_released: 0,
                            uniform_business: qbi_activity(net),
                            disposition_gain: source_gain.unwrap_or(0),
                            rental_partial: rental_source!(
                                crate::asset_sale::kept_rental_partials(d, e.id, amt)
                            )
                            .is_some(),
                            estate_trust: false,
                            estate_activity: None,
                            farm_rental: false,
                            ptp: 0,
                            pass_through: None,
                            released: false,
                            items: None,
                        });
                    } else if let Some(gain) = source_gain
                        .filter(|_| releases_losses && !matches!(v.sale, crate::RentalSale::No))
                    {
                        // Section 469(g) (Pub 925, Form 8582 instructions): the
                        // whole property sold in a fully taxable sale. Its current
                        // and prior losses are combined with the sale gain; an
                        // overall loss is nonpassive and allowed in full, an
                        // overall gain is passive income that offsets other
                        // passive losses. Either way this activity's own losses
                        // are allowed.
                        // Section 469(g)(3), an installment sale of the entire
                        // interest: the overall loss (this year's net and the
                        // prior unallowed balance) is nonpassive in the ratio of
                        // the gain recognized this year to the total gain (i8582
                        // "Disposition of an Entire Interest"; Pub 925 "Installment
                        // sale of an entire interest"); the rest stays the
                        // property's unallowed loss, carried until the later
                        // payments recognize more gain. The recognized gain is
                        // passive income of the sale.
                        let installment_form = crate::asset_sale::rental_sale(v, amt)
                            .ok()
                            .flatten()
                            .and_then(|g| g.installment);
                        let own_net = dollars(usage.net);
                        let prior_balance = dollars(prior_passive[usize::from(amt)]);
                        if let Some(form) =
                            installment_form.as_ref().filter(|_| prior_balance > own_net)
                        {
                            let (released, remaining) =
                                installment_release(prior_balance - own_net, form);
                            // MAGI always uses the regular-tax amount.
                            let regular = if amt {
                                let regular_gain = rental_source!(
                                    crate::asset_sale::rental_source(d, e.id, false)
                                )
                                .map_or(0, |value| value.gain);
                                let regular_net = dollars(
                                    rental_amount!(
                                        e.id,
                                        rental_use_core(
                                            v,
                                            false,
                                            mid_quarter,
                                            itemizing,
                                            home_fixed_allowed(d, e.id, home_agi, false),
                                            &expense_rows,
                                            Some((d, e.id, false)),
                                        )
                                        .map_err(
                                            |(key, _)| {
                                                crate::activity_risk::ProjectionIssue::Source(key)
                                            }
                                        )
                                    )
                                    .net,
                                );
                                let regular_prior = dollars(prior_passive[0]);
                                let regular_released = crate::asset_sale::rental_sale(v, false)
                                    .ok()
                                    .flatten()
                                    .and_then(|g| g.installment)
                                    .filter(|_| regular_prior > regular_net)
                                    .map_or(regular_prior - regular_net, |f| {
                                        installment_release(regular_prior - regular_net, &f).0
                                    });
                                regular_gain - regular_released
                            } else {
                                gain - released
                            };
                            nonpassive_income -= released;
                            part_one -= released;
                            released_rental_losses |= positive(-dollars(usage.net))
                                + dollars(prior_passive[usize::from(amt)])
                                > 0;
                            magi += regular;
                            if let Some(activity) = qbi_activity(-released) {
                                home_activities.push((e.id, activity));
                            }
                            if !amt && released > 0 {
                                form_line(
                                    r,
                                    FormLine::new(TaxForm::ScheduleE, "22"),
                                    &alloc::format!("rental_{}_line22_allowed_loss", e.id),
                                    -released,
                                    &["income.rental", "income.rental.sale"],
                                    PASSIVE,
                                    msg!("EXPLAIN_RENTAL_N_LINE22_ALLOWED_LOSS"),
                                );
                            }
                            line(
                                r,
                                &alloc::format!("{prefix}rental_{}_suspended_loss", e.id),
                                remaining,
                                &["income.rental", "income.rental.sale"],
                                PASSIVE,
                                msg!("EXPLAIN_0RENTAL_N_SUSPENDED_LOSS"),
                            );
                            line(
                                r,
                                &alloc::format!("{prefix}rental_{}_sale_released_loss", e.id),
                                released,
                                &["income.rental", "income.rental.sale"],
                                PASSIVE,
                                msg!("EXPLAIN_0RENTAL_N_SALE_RELEASED_LOSS"),
                            );
                            profits += gain;
                            values.push(PassiveEntry {
                                source_cents: None,
                                id: e.id,
                                net: gain,
                                rental_allowance: allowance,
                                ineligible_prior: 0,
                                prior: 0,
                                personal: false,
                                scorp: None,
                                partner: false,
                                source_group: None,
                                characters: None,
                                items: None,
                                nonpassive: 0,
                                prior_released: 0,
                                uniform_business: qbi_activity(0),
                                disposition_gain: gain,
                                rental_partial: rental_source!(
                                    crate::asset_sale::kept_rental_partials(d, e.id, amt)
                                )
                                .is_some(),
                                estate_trust: false,
                                estate_activity: None,
                                farm_rental: false,
                                ptp: 0,
                                pass_through: None,
                                released: false,
                            });
                        } else {
                            let net = dollars(usage.net) - dollars(prior_passive[usize::from(amt)]);
                            let combined = net + gain;
                            if combined < 0 {
                                let regular =
                                    if amt {
                                        let regular_gain = rental_source!(
                                            crate::asset_sale::rental_source(d, e.id, false)
                                        )
                                        .map_or(0, |value| value.gain);
                                        dollars(
                                            rental_amount!(
                                                e.id,
                                                rental_use_core(
                                                    v,
                                                    false,
                                                    mid_quarter,
                                                    itemizing,
                                                    home_fixed_allowed(d, e.id, home_agi, false),
                                                    &expense_rows,
                                                    Some((d, e.id, false)),
                                                )
                                                .map_err(|(key, _)| {
                                                    crate::activity_risk::ProjectionIssue::Source(
                                                        key,
                                                    )
                                                })
                                            )
                                            .net,
                                        ) - dollars(prior_passive[0])
                                            + regular_gain
                                    } else {
                                        combined
                                    };
                                nonpassive_income += net;
                                part_one += net;
                                magi += regular;
                                if let Some(activity) = qbi_activity(net) {
                                    home_activities.push((e.id, activity));
                                }
                                if !amt {
                                    form_line(
                                        r,
                                        FormLine::new(TaxForm::ScheduleE, "22"),
                                        &alloc::format!("rental_{}_line22_allowed_loss", e.id),
                                        // Part VIII (c): this year's loss and the
                                        // prior-year loss the sale releases, even
                                        // when the activity's own profit absorbs part.
                                        -(positive(-dollars(usage.net))
                                            + dollars(prior_passive[usize::from(amt)])),
                                        &["income.rental", "income.rental.sale"],
                                        PASSIVE,
                                        msg!("EXPLAIN_RENTAL_N_LINE22_ALLOWED_LOSS"),
                                    );
                                }
                                line(
                                    r,
                                    &alloc::format!("{prefix}rental_{}_suspended_loss", e.id),
                                    0,
                                    &["income.rental", "income.rental.sale"],
                                    PASSIVE,
                                    msg!("EXPLAIN_0RENTAL_N_SUSPENDED_LOSS"),
                                );
                            } else {
                                profits += combined;
                                values.push(PassiveEntry {
                                    source_cents: None,
                                    id: e.id,
                                    net: combined,
                                    rental_allowance: allowance,
                                    ineligible_prior: dollars(ineligible),
                                    prior: dollars(prior_passive[usize::from(amt)]),
                                    personal: false,
                                    scorp: None,
                                    partner: false,
                                    source_group: None,
                                    characters: None,
                                    items: None,
                                    nonpassive: 0,
                                    prior_released: 0,
                                    uniform_business: qbi_activity(net),
                                    disposition_gain: gain,
                                    rental_partial: rental_source!(
                                        crate::asset_sale::kept_rental_partials(d, e.id, amt)
                                    )
                                    .is_some(),
                                    released: false,
                                    estate_trust: false,
                                    estate_activity: None,
                                    farm_rental: false,
                                    ptp: 0,
                                    pass_through: None,
                                });
                            }
                        }
                    } else {
                        let gain = source_gain.unwrap_or(0);
                        let own = dollars(usage.net) - dollars(prior_passive[usize::from(amt)]);
                        let net = own + gain;
                        profits += positive(net);
                        values.push(PassiveEntry {
                            source_cents: None,
                            id: e.id,
                            net,
                            rental_allowance: allowance,
                            ineligible_prior: dollars(ineligible),
                            prior: dollars(prior_passive[usize::from(amt)]),
                            personal: false,
                            scorp: None,
                            partner: false,
                            source_group: None,
                            characters: None,
                            items: None,
                            nonpassive: 0,
                            prior_released: 0,
                            uniform_business: qbi_activity(own),
                            disposition_gain: gain,
                            rental_partial: rental_source!(
                                crate::asset_sale::kept_rental_partials(d, e.id, amt)
                            )
                            .is_some(),
                            estate_trust: false,
                            estate_activity: None,
                            farm_rental: false,
                            ptp: 0,
                            pass_through: None,
                            released: false,
                        });
                    }
                    if !amt {
                        for (index, asset) in v.assets.iter().enumerate() {
                            for (suffix, amount) in [
                                ("bonus", asset.bonus_amount()),
                                ("macrs", asset.macrs_amount(mid_quarter)),
                            ] {
                                if amount == 0 {
                                    continue;
                                }
                                push_line(
                                    r,
                                    crate::depreciation::form_4562_line(asset, suffix)
                                        .map(|line| FormLine::new(TaxForm::Form4562, line)),
                                    &alloc::format!("rental_{}_asset_{index}_{suffix}", e.id),
                                    amount,
                                    &["income.rental.assets"],
                                    RENTAL,
                                    msg!("EXPLAIN_RENTAL_ASSET_DEPRECIATION"),
                                );
                            }
                        }
                    }
                    if !amt && usage.energy_deduction > 0 {
                        push_line(
                            r,
                            Some(FormLine::subrow(TaxForm::ScheduleE, "19")),
                            &alloc::format!("rental_{}_energy_deduction", e.id),
                            usage.energy_deduction,
                            &["income.rental.energy_buildings", "income.rental.buildings.energy"],
                            "https://www.irs.gov/instructions/i7205",
                            msg!("EXPLAIN_RENTAL_ENERGY_DEDUCTION"),
                        );
                    }
                    let commercial = v.property_type.nonresidential();
                    // The entered expenses and depreciation are the rental
                    // share only without personal use (or with no personal
                    // days and no carried or rental-only amounts).
                    let no_personal_split = (v.home_operating_carryover == 0
                        && crate::rental_home_carry::total(v, amt) == Some(0))
                        && match v.personal_use {
                            crate::extended_model::RentalPersonalUse::No => true,
                            crate::extended_model::RentalPersonalUse::Yes {
                                personal_days,
                                direct_expenses,
                                ..
                            } => personal_days == 0 && direct_expenses == 0,
                        };
                    if amt {
                        line(
                            r,
                            &alloc::format!("{prefix}rental_{}_depreciation", e.id),
                            dollars(usage.depreciation),
                            &["income.rental.buildings"],
                            RENTAL,
                            msg!("EXPLAIN_0RENTAL_N_DEPRECIATION"),
                        );
                    } else {
                        // Form 4562 "Who Must File": a rental needs the form
                        // only for a building placed in service in 2026
                        // (it has no section 179, listed property or
                        // amortization); otherwise its depreciation is on
                        // Schedule E line 18 alone. Line 22 is the rental
                        // share Schedule E line 18 deducts.
                        // The excess basis of a like-kind exchange's property
                        // received is new property placed in service in 2026.
                        let replacement = exchange.map_or_else(Default::default, |exchange| {
                            exchange.replacement_depreciation(v, false).unwrap_or_default()
                        });
                        let form_4562 = v.buildings.iter().any(|b| b.in_service / 10000 == 2026)
                            || replacement.excess > 0;
                        push_line(
                            r,
                            form_4562.then(|| FormLine::new(TaxForm::Form4562, "22")),
                            &alloc::format!("rental_{}_depreciation", e.id),
                            usage.depreciation,
                            &["income.rental.buildings"],
                            RENTAL,
                            if commercial {
                                msg!("EXPLAIN_RENTAL_N_DEPRECIATION")
                            } else {
                                msg!("EXPLAIN_RENTAL_N_DEPRECIATION_2")
                            },
                        );
                        // Each building's GDS by service year: 2026 on Part III
                        // 19i (residential) or 19j (nonresidential), earlier
                        // years on line 17. With personal use the rental share
                        // is divided for the whole property, not per building,
                        // so no building row is listed.
                        if form_4562 && no_personal_split {
                            for (index, b) in v.buildings.iter().enumerate() {
                                let line = if b.in_service / 10000 < 2026 {
                                    "17"
                                } else if commercial {
                                    "19j"
                                } else {
                                    "19i"
                                };
                                push_line(
                                    r,
                                    Some(FormLine::new(TaxForm::Form4562, line)),
                                    &alloc::format!(
                                        "rental_{}_building_{index}_depreciation",
                                        e.id
                                    ),
                                    depreciation_for_year(
                                        b,
                                        false,
                                        &v.property_type,
                                        u32::try_from(d.tax_year).expect("validated return year"),
                                        v.sale.date(),
                                        crate::kernel::source_fact::SourceFact::Reported(
                                            crate::rental_energy::for_source(d, e.id, false)
                                                .reductions[index],
                                        ),
                                    )
                                    .unwrap_or(0),
                                    &["income.rental.buildings"],
                                    RENTAL,
                                    msg!("EXPLAIN_RENTAL_N_BUILDING_N_DEPRECIATION"),
                                );
                            }
                            // The property received: the exchanged basis
                            // continues the building given up (line 17), the
                            // excess basis is new (19i or 19j).
                            let replacement_commercial = exchange.is_some_and(|exchange| {
                                matches!(
                                    exchange.replacement.used,
                                    crate::ReplacementUse::Rental {
                                        kind: RentalPropertyType::Commercial,
                                        ..
                                    }
                                )
                            });
                            for (line, suffix, amount, explanation) in [
                                (
                                    "17",
                                    "carryover",
                                    replacement.carryover,
                                    msg!("EXPLAIN_RENTAL_REPLACEMENT_CARRYOVER_DEPRECIATION"),
                                ),
                                (
                                    if replacement_commercial { "19j" } else { "19i" },
                                    "excess",
                                    replacement.excess,
                                    msg!("EXPLAIN_RENTAL_REPLACEMENT_EXCESS_DEPRECIATION"),
                                ),
                            ] {
                                if amount != 0 {
                                    push_line(
                                        r,
                                        Some(FormLine::new(TaxForm::Form4562, line)),
                                        &alloc::format!(
                                            "rental_{}_replacement_{suffix}_depreciation",
                                            e.id
                                        ),
                                        amount,
                                        &["income.rental.sale"],
                                        RENTAL,
                                        explanation,
                                    );
                                }
                            }
                        }
                    }
                    // Schedule E lines 3, 18, 20 and 21 of a property: the
                    // rent and the rental share of depreciation, total
                    // expenses and income after any personal-use division
                    // and Worksheet 5-1 limit. A home rented fewer than 15
                    // days is not on Schedule E.
                    let schedule_e = !amt && !usage.excluded;
                    if schedule_e {
                        emit_lines(
                            r,
                            &mut (),
                            [
                                (
                                    "3",
                                    "line3_rents",
                                    v.rent + method_amounts(v).0 + usage.asset_recapture_cents,
                                    msg!("EXPLAIN_RENTAL_N_N_N"),
                                ),
                                (
                                    "18",
                                    "line18_depreciation",
                                    usage.depreciation,
                                    msg!("EXPLAIN_RENTAL_N_N_N_2"),
                                ),
                                (
                                    "20",
                                    "line20_total_expenses",
                                    v.rent + method_amounts(v).0 + usage.asset_recapture_cents
                                        - net_before_at_risk,
                                    msg!("EXPLAIN_RENTAL_N_N_N_3"),
                                ),
                                (
                                    "21",
                                    "line21_income",
                                    net_before_at_risk,
                                    msg!("EXPLAIN_RENTAL_N_N_N_4"),
                                ),
                            ],
                            |(line, suffix, amount, explanation), _, _| {
                                Some(calculation_row(
                                    Some(FormLine::new(TaxForm::ScheduleE, line)),
                                    &alloc::format!("rental_{}_{suffix}", e.id),
                                    LineValue::MoneyCents(amount),
                                    &["income.rental"],
                                    RENTAL,
                                    explanation,
                                ))
                            },
                        );
                    }
                    // Schedule E shows the allowed rental share after Worksheet 5-1.
                    // Allocate limited operating expenses in printed-line order.
                    if !amt && !usage.excluded {
                        let amounts = rental_expense_lines(
                            v,
                            &usage,
                            itemizing,
                            home_fixed_allowed(d, e.id, home_agi, amt),
                        );
                        let amounts = match amounts {
                            Ok(amounts) => amounts,
                            Err((key, _message)) => {
                                crate::issue_localized!(
                                    r,
                                    &crate::interview::income_issue_context(e),
                                    &label,
                                    key,
                                    _message,
                                );
                                crate::federal_rules::withhold_required_source_result(r);
                                continue;
                            }
                        };
                        for (index, amount) in amounts.into_iter().enumerate() {
                            let (suffix, explanation) = RentalExpenseCategory::LINES[index];
                            if amount != 0 {
                                push_line(
                                    r,
                                    Some(FormLine::new(TaxForm::ScheduleE, printed_line(suffix))),
                                    &alloc::format!("rental_{}_expense_{suffix}", e.id),
                                    amount,
                                    &["income.rental.expenses"],
                                    RENTAL,
                                    crate::Message::new(explanation, alloc::vec![]),
                                );
                            }
                        }
                        crate::method_change::report(
                            &v.method_changes,
                            r,
                            &alloc::format!("rental_{}", e.id),
                            "income.rental.method_changes",
                            &crate::interview::income_display_arg(e),
                            &crate::interview::income_issue_context(e),
                        );
                    }
                }
                Err((key, _message)) => {
                    if [
                        crate::rental_home_carry::TOTAL,
                        crate::rental_home_carry::INVALID,
                        crate::rental_home_carry::OVERFLOW,
                    ]
                    .contains(&key)
                    {
                        source_issue.get_or_insert(crate::activity_risk::SourceIssue::new(
                            e.id,
                            crate::activity_risk::ProjectionIssue::Source(key),
                        ));
                    }
                    let context = crate::interview::income_issue_context(e);
                    // Regular and AMT evaluation share one correction per source,
                    // as do the downstream activity projections.
                    if !r.issues.iter().any(|issue| {
                        issue.context.identity() == context.identity() && issue.display.key == key
                    }) {
                        #[cfg(test)]
                        let message = _message;
                        crate::issue_localized!(r, &context, &label, key, message)
                    }
                }
            }
            *income_by_id.entry(e.id).or_default() += part_one - source_before;
            *nii_adjustment_by_id.entry(e.id).or_default() +=
                rental_niit_adjustment - source_niit_before;
        }
    }
    // Form 4835 farm rentals (unit B2): passive like a Schedule E rental, with
    // the special allowance when the owner actively participated; a real
    // estate professional is kept passive here because a landlord paid in
    // crop shares, by definition, did not take part in production. The
    // rental's own land and asset sales stay outside the pool (nonpassive
    // property income). A rental left out before the rules ran (unknowns.rs)
    // is not here.
    let mut farm_rental_property = alloc::collections::BTreeMap::new();
    let mut farm_rental_property_rows = alloc::collections::BTreeMap::new();
    for e in &allocated_entries {
        let FederalIncome::FarmRental { data: v } = &e.income else {
            continue;
        };
        let owner = usize::from(e.owner == Owner::Spouse);
        let projection = rental_source!(crate::farm_rental::risk_source(
            d,
            e.id,
            v,
            owner,
            mid_quarter,
            amt,
            Some(r)
        ));
        operating_sources.insert(
            e.id,
            RentalOperatingSource {
                source: e.id,
                owner: e.owner,
                alternative: amt,
                before_at_risk_cents: projection.before_risk,
                after_at_risk_cents: projection.operating,
                prior_at_risk_cents: projection.prior_risk,
                prior_passive_cents: v.history.balance(amt),
                prior_home_carry_cents: [0; 2],
                home_limited: false,
                excluded: false,
                property_disposition: !v.farm.land_sales.is_empty()
                    || v.assets.iter().any(|asset| {
                        !matches!(asset.disposal, crate::AssetDisposal::No)
                            || asset.partial.is_some()
                    }),
                filed_ordinary_dollars: None,
            },
        );
        operating_prior_allowed.insert(e.id, projection.prior_operating_allowed);
        farm_rental_property_rows.insert(e.id, projection.property_rows);
        if let Some(property) = projection.property {
            farm_rental_property.insert(e.id, property);
        }
        let net = dollars(projection.operating) - dollars(v.history.balance(amt));
        profits += positive(net);
        values.push(PassiveEntry {
            source_cents: None,
            id: e.id,
            net,
            rental_allowance: v.active_participation,
            ineligible_prior: ineligible_prior(v.active_participation, &v.history, amt),
            prior: dollars(v.history.balance(amt)),
            personal: false,
            scorp: None,
            partner: false,
            source_group: None,
            characters: None,
            items: None,
            nonpassive: 0,
            prior_released: 0,
            uniform_business: crate::farm_rental::qbi_activity(v, net).map(|activity| {
                UniformBusinessIncome {
                    income: activity.income,
                    regular_qbi: (!amt).then_some(activity),
                }
            }),
            disposition_gain: 0,
            rental_partial: false,
            estate_trust: false,
            estate_activity: None,
            farm_rental: true,
            ptp: 0,
            pass_through: None,
            released: false,
        });
    }
    // Each tax system uses its own retained source records and the same
    // basis -> at-risk -> passive engine. AMT's allowance still uses regular MAGI.
    let mut amt_sources = alloc::vec::Vec::new();
    let mut k1_ledgers = alloc::vec::Vec::new();
    if amt {
        for e in &allocated_entries {
            if let FederalIncome::Scorp { data } = &e.income {
                match data.amt.as_ref().ok_or(("ISSUE_SCORP_AMT_BASIS_AND_LOSS_RECORDS", crate::fixture_english!("Complete separate AMT K-1 basis and loss records; absence is not a zero adjustment."))).and_then(|v| v.source(data)) {
                    Ok(source) => {
                        if let Ok(basis) = source.basis_result() {
                            for (_name, key, amount) in [
                                ("ending stock basis", "NOTE_AMT_SCORP_ENDING_STOCK_BASIS", basis.stock),
                                ("basis suspended loss", "NOTE_AMT_SCORP_BASIS_SUSPENDED_LOSS", basis.suspended[crate::k1_loss::K1LossClass::Ordinary]),
                            ] {
                                let message = Message::new(key, alloc::vec![
                                    MessageArg::Number(e.id), MessageArg::usd_cents(amount.get()),
                                ]);
                                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                                    #[cfg(test)] message.english(),
                                    message,
                                ));
                            }
                            for (index, loan) in basis.loans.iter().enumerate() {
                                let message = Message::new("NOTE_AMT_SCORP_LOAN_ENDING_BASIS", alloc::vec![
                                    MessageArg::Number(e.id), MessageArg::Number((index + 1) as i64), MessageArg::usd_cents(loan.basis.get()),
                                ]);
                                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                                    #[cfg(test)] message.english(), message,
                                ));
                            }
                            if let Ok(risk) = source.risk_result(&basis) {
                                let message = Message::new("NOTE_AMT_SCORP_RISK_SUSPENDED", alloc::vec![
                                    MessageArg::Number(e.id), MessageArg::usd_cents(risk.ledger.risk_suspended[crate::k1_loss::K1LossClass::Ordinary].get()),
                                ]);
                                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                                    #[cfg(test)] message.english(), message,
                                ));
                                let message = match risk.year_end {
                                    Some(end) => Message::new("NOTE_AMT_SCORP_NONDEDUCTIBLE_WITH_END", alloc::vec![MessageArg::Number(e.id), MessageArg::usd_cents(risk.nondeductible_used.get()), MessageArg::usd_cents(risk.nondeductible_suspended.get()), MessageArg::usd_cents(end)]),
                                    None => Message::new("NOTE_AMT_SCORP_NONDEDUCTIBLE_NO_END", alloc::vec![MessageArg::Number(e.id), MessageArg::usd_cents(risk.nondeductible_used.get()), MessageArg::usd_cents(risk.nondeductible_suspended.get())]),
                                };
                                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                                    #[cfg(test)] message.english(), message,
                                ));
                            }
                        }
                        amt_sources.push((e.id, source));
                    },
                    Err((key, _message)) => {
#[cfg(test)] let message = _message;
crate::issue_localized!(r, &crate::IssueContext::ScorpAlternativeMinimum,"S-corporation AMT", key,message)},
                }
            }
        }
    }
    let mut has_scorp = false;
    for e in &allocated_entries {
        if let FederalIncome::Scorp { data } = &e.income {
            has_scorp = true;
            // The later passive carrier borrows the stable source record for
            // identity/QBI metadata. All AMT basis/risk/outcome calculations
            // retain this producer's typed capacity view below.
            let (source_record, source): (&crate::ScorpK1, crate::scorp_amt::ScorpSource<'_>) =
                if amt {
                    let Some((_, source)) = amt_sources.iter().find(|(id, _)| *id == e.id) else {
                        continue;
                    };
                    (&**source, source.into())
                } else {
                    (data, data.into())
                };
            let computed =
                crate::community::scorp_pool(d, e.id, &source, amt).and_then(|activity| {
                    let regular = if amt {
                        Some(crate::community::scorp_pool(d, e.id, data, false)?)
                    } else {
                        None
                    };
                    let magi_net = regular
                        .as_ref()
                        .map_or(activity.combined.nonpassive, |v| v.combined.nonpassive);
                    let regular_ledger = regular.as_ref().map_or(activity.ledger, |v| v.ledger);
                    let status = d
                        .professional(crate::community::professional_owner(
                            d.setup.as_ref(),
                            e.owner,
                        ))
                        .is_some();
                    let mut rentals = source.rental_activities(
                        &activity.ledger,
                        status,
                        professional_group_disposed(d, e.owner),
                    )?;
                    let mut regular_rentals = data.rental_activities(
                        &regular_ledger,
                        status,
                        professional_group_disposed(d, e.owner),
                    )?;
                    professional_groups.apply(
                        e.id,
                        &mut rentals,
                        source.rental_facts.carryover_active,
                    );
                    regular_groups.apply(
                        e.id,
                        &mut regular_rentals,
                        data.rental_facts.carryover_active,
                    );
                    Ok((activity, magi_net, rentals, regular_rentals))
                });
            match computed {
                Ok((activity, magi_net, rentals, regular_rentals)) => {
                    let [rental_nonpassive, rental_magi, rental_profits, rental_niit] =
                        pass_through_rentals(
                            &mut values,
                            e.id,
                            "scorp",
                            rentals,
                            &regular_rentals,
                            d.professional(crate::community::professional_owner(
                                d.setup.as_ref(),
                                e.owner,
                            ))
                            .is_some_and(|p| p.over_500_hours),
                            regular_groups.source(e.id).is_some(),
                        );
                    nonpassive_income += rental_nonpassive;
                    magi += rental_magi;
                    profits += rental_profits;
                    rental_niit_adjustment += rental_niit;
                    *nii_adjustment_by_id.entry(e.id).or_default() += rental_niit;
                    k1_ledgers.push((e.id, activity.ledger));
                    let net = activity.combined.passive;
                    nonpassive_income += dollars(activity.combined.nonpassive);
                    // The K-1 section 179 allowed on Form 4562 line 12
                    // (Schedule E line 28 column (j)) reduces AGI and MAGI.
                    magi += dollars(magi_net) - dollars(section179_allowed(e.id));
                    profits += dollars(positive(net));
                    values.push(PassiveEntry {
                        source_cents: Some(K1PassiveCents {
                            separate_nonpassive: [0; 4],
                            net,
                            prior: activity.combined.prior_pool,
                            ineligible_prior: 0,
                            nonpassive: activity.combined.nonpassive,
                            prior_released: activity.combined.prior_released,
                            disposition_gain: 0,
                            ptp: 0,
                        }),
                        id: e.id,
                        net: dollars(net),
                        rental_allowance: false,
                        ineligible_prior: 0,
                        prior: dollars(activity.combined.prior_pool),
                        personal: false,
                        scorp: Some(source_record),
                        partner: false,
                        source_group: None,
                        characters: activity.combined.characters,
                        items: activity
                            .items
                            .any()
                            .then_some((activity.items, source.items_in_pool())),
                        nonpassive: dollars(activity.combined.nonpassive),
                        prior_released: dollars(activity.combined.prior_released),
                        uniform_business: None,
                        disposition_gain: 0,
                        rental_partial: false,
                        estate_trust: false,
                        estate_activity: None,
                        farm_rental: false,
                        ptp: 0,
                        pass_through: None,
                        released: false,
                    });
                }
                Err((key, _message)) => {
                    #[cfg(test)]
                    let message = _message;
                    crate::issue_localized!(
                        r,
                        if amt {
                            &crate::IssueContext::ScorpAlternativeMinimum
                        } else {
                            &crate::IssueContext::Section(crate::TaxSection::PassThrough)
                        },
                        if amt { "S-corporation AMT" } else { "S-corporation passive pool" },
                        key,
                        message,
                    )
                }
            }
        }
    }
    for e in &allocated_entries {
        let FederalIncome::Partnership { data } = &e.income else {
            continue;
        };
        let computed = (|| {
            let source = if amt {
                data.alternative.as_ref().ok_or(("ISSUE_EXTENDED_RULES_COMPLETE_SEPARATE_AMT_PARTNERSHIP_ACTIVITY_RECORDS_ABC340B8", crate::fixture_english!("Complete separate AMT partnership activity records; missing data is not a zero adjustment.")))?
            } else {
                &data.regular
            };
            let fuel_recovery = if let Some(prior) = &data.prior_fuel_credit {
                if data.trade_kind != crate::ScorpTradeKind::Ordinary
                    || data.material_for_losses() != crate::participation::Participation::Yes
                    || [&data.regular, data.alternative.as_ref().unwrap_or(&data.regular)]
                        .into_iter()
                        .any(|record| {
                            record.basis.prior_loss != 0
                                || record.risk.prior_loss != 0
                                || record.passive_loss != 0
                                || record.carried != crate::K1LossCarryforwards::default()
                                || record.capital_carried != crate::PartnerCapitalCarry::default()
                        })
                {
                    return Err((
                        "ISSUE_EXTENDED_RULES_CONFIRM_THE_CASH_METHOD_2025_PARTNER_FUEL_F5415DDF",
                        crate::fixture_english!(
                            "A prior-year partner fuel credit recovery requires a continuing nonpassive ordinary U.S. partnership with no retained basis, at-risk or passive losses; other histories need a separate recovery worksheet."
                        ),
                    ));
                }
                dollars(prior.taxable_amount())
            } else {
                0
            };
            let activity = crate::community::partner_pool(d, e.id, data, source)?;
            let magi_net = if amt {
                crate::community::partner_pool(d, e.id, data, &data.regular)?.combined.nonpassive
            } else {
                activity.combined.nonpassive
            };
            let status = d
                .professional(crate::community::professional_owner(d.setup.as_ref(), e.owner))
                .is_some();
            let regular_ledger = if amt {
                crate::community::partner_pool(d, e.id, data, &data.regular)?.activity.ledger
            } else {
                activity.activity.ledger
            };
            let mut rentals = data.rental_activities(
                source,
                &activity.activity.ledger,
                status,
                professional_group_disposed(d, e.owner),
            )?;
            let mut regular_rentals = data.rental_activities(
                &data.regular,
                &regular_ledger,
                status,
                professional_group_disposed(d, e.owner),
            )?;
            professional_groups.apply(e.id, &mut rentals, data.rental_facts.carryover_active);
            regular_groups.apply(e.id, &mut regular_rentals, data.rental_facts.carryover_active);
            Ok((activity, magi_net, fuel_recovery, rentals, regular_rentals))
        })();
        match computed {
            Ok((activity, magi_net, fuel_recovery, rentals, regular_rentals)) => {
                let [rental_nonpassive, rental_magi, rental_profits, rental_niit] =
                    pass_through_rentals(
                        &mut values,
                        e.id,
                        "partnership",
                        rentals,
                        &regular_rentals,
                        d.professional(crate::community::professional_owner(
                            d.setup.as_ref(),
                            e.owner,
                        ))
                        .is_some_and(|p| p.over_500_hours),
                        regular_groups.source(e.id).is_some(),
                    );
                nonpassive_income += rental_nonpassive;
                magi += rental_magi;
                profits += rental_profits;
                rental_niit_adjustment += rental_niit;
                *nii_adjustment_by_id.entry(e.id).or_default() += rental_niit;
                // Guaranteed payments are nonpassive but are not ordinary activity
                // income that can release its former passive losses or add basis.
                // The partner's prior Form 4136 recovery is also separate from
                // the current K-1 box 1 and box 14 amounts.
                let guarantees =
                    dollars(data.guaranteed_services) + dollars(data.guaranteed_capital);
                k1_ledgers.push((e.id, activity.activity.ledger));
                // The selected PAL owner already classifies the section179
                // component; consume its nonpassive part once, not a second
                // blanket deduction from ordinary/nonpassive income below.
                let selected_nonpassive = selected_ptp(e.id).map_or(0, |source| {
                    source
                        .components
                        .iter()
                        .filter(|part| {
                            part.original.character
                                == crate::ptp_passive::PtpPalCharacter::Section179
                        })
                        .map(|part| part.original.nonpassive_cents)
                        .sum::<i64>()
                });
                nonpassive_income += dollars(activity.combined.nonpassive)
                    + guarantees
                    + fuel_recovery
                    + dollars(selected_nonpassive);
                magi += dollars(magi_net) + guarantees + fuel_recovery
                    - dollars(section179_allowed(e.id));
                // Section 469(k): a publicly traded partnership's passive
                // result is limited by itself and never joins the common
                // Form 8582 pool (2025 i8582, Special Instructions for PTPs).
                // Its net income is nonpassive income of the return; its net
                // loss stays suspended with the partnership.
                let ptp = if data.publicly_traded { activity.combined.passive } else { 0 };
                let pool_net = activity.combined.passive - ptp;
                if data.publicly_traded {
                    let (gain, held) = selected_ptp(e.id).map_or_else(
                        || {
                            crate::ptp_passive::single_activity_cents(e.id, ptp)
                                .expect("validated native PTP activity fits its own passive pool")
                        },
                        |source| (source.net_gain_cents, source.suspended_cents),
                    );
                    ptp_income += dollars(gain);
                    ptp_suspended += dollars(held);
                }
                profits += dollars(positive(pool_net));
                let separate_nonpassive = fuel_recovery.checked_mul(100).map(|fuel| {
                    [data.guaranteed_services, data.guaranteed_capital, fuel, selected_nonpassive]
                });
                let source_nonpassive = separate_nonpassive
                    .and_then(|parts| parts.into_iter().try_fold(0i64, i64::checked_add))
                    .and_then(|separate| activity.combined.nonpassive.checked_add(separate));
                let (Some(separate_nonpassive), Some(source_nonpassive)) =
                    (separate_nonpassive, source_nonpassive)
                else {
                    crate::issue_localized!(
                        r,
                        &crate::interview::income_issue_context(e),
                        "Partnership",
                        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                        "Review the document amounts; they must be within the supported input range."
                    );
                    continue;
                };
                values.push(PassiveEntry {
                    source_cents: Some(K1PassiveCents {
                        separate_nonpassive,
                        net: pool_net,
                        prior: if data.publicly_traded { 0 } else { activity.combined.prior_pool },
                        ineligible_prior: 0,
                        nonpassive: source_nonpassive,
                        prior_released: activity.combined.prior_released,
                        disposition_gain: 0,
                        ptp,
                    }),
                    id: e.id,
                    net: dollars(pool_net),
                    rental_allowance: false,
                    ineligible_prior: 0,
                    // A PTP's prior-year unallowed loss offsets only that PTP's
                    // income (section 469(k)): none of it is a Form 8582 entry.
                    prior: if data.publicly_traded {
                        0
                    } else {
                        dollars(activity.combined.prior_pool)
                    },
                    personal: false,
                    scorp: None,
                    partner: true,
                    source_group: None,
                    characters: activity.combined.characters,
                    items: activity.items.any().then_some((activity.items, data.items_in_pool())),
                    nonpassive: dollars(activity.combined.nonpassive)
                        + guarantees
                        + fuel_recovery
                        + dollars(selected_nonpassive),
                    prior_released: dollars(activity.combined.prior_released),
                    uniform_business: None,
                    disposition_gain: 0,
                    rental_partial: false,
                    estate_trust: false,
                    estate_activity: None,
                    farm_rental: false,
                    ptp: dollars(ptp),
                    pass_through: None,
                    released: false,
                });
            }
            Err((key, _message)) => {
                #[cfg(test)]
                let message = _message;
                crate::issue_localized!(
                    r,
                    &if key == "ISSUE_K1_PTP_PROPERTY" {
                        crate::interview::income_issue_context(e)
                    } else {
                        crate::IssueContext::Section(crate::TaxSection::PassThrough)
                    },
                    if amt { "Partnership AMT" } else { "Partnership passive pool" },
                    key,
                    message,
                )
            }
        }
    }
    // Actual partnership and S-corporation graphs share this return-level pool.
    // The parent identity owns return totals; the group identity owns PAL.
    for e in &allocated_entries {
        let (preliminary, partner) = match &e.income {
            FederalIncome::PartnershipActivities { data } => (data.before_section179(amt), true),
            FederalIncome::ScorpActivities { data } => (data.before_section179(amt), false),
            _ => continue,
        };
        let admitted = preliminary.is_ok();
        let computed = preliminary.and_then(|output| {
            let output = output.apply_section179(section179)?;
            let regular = if amt {
                let source = match &e.income {
                    FederalIncome::PartnershipActivities { data } => data.before_section179(false),
                    FederalIncome::ScorpActivities { data } => data.before_section179(false),
                    _ => unreachable!(),
                };
                Some(source?.apply_section179(regular_section179)?)
            } else {
                None
            };
            Ok((output, regular))
        });
        // Only a failure after preliminary admission belongs here. The
        // source/community admission boundary remains owned by collect.
        if let Err((issue, _message)) = &computed {
            let context = crate::interview::income_issue_context(e);
            if admitted
                && !r.issues.iter().any(|existing| {
                    existing.display.key == issue.key
                        && existing.context.source() == context.source()
                })
            {
                crate::issue_localized!(r, &context, "K-1 activity", issue.key, *_message);
            }
            continue;
        }
        let computed = computed.and_then(|(output, regular)| {
            if d.setup.as_ref().is_some_and(crate::community::applies) {
                return Err(crate::k1_activity::ordinary::UNAVAILABLE);
            }
            Ok((output, regular))
        });
        // collect owns the single scoped source/community boundary.
        // Neither tax-system pass may duplicate it or admit graph values.
        if let Ok((output, regular)) = computed {
            let regular_groups = regular.as_ref().map_or(&output.groups, |v| &v.groups);
            magi += dollars(regular_groups.iter().map(|g| g.nonpassive).sum::<i64>());
            nonpassive_income += dollars(output.groups.iter().map(|g| g.nonpassive).sum::<i64>());
            profits += dollars(output.groups.iter().map(|g| positive(g.passive)).sum::<i64>());
            if !partner {
                let message = Message::new(
                    if amt {
                        "NOTE_AMT_SCORP_ENDING_STOCK_BASIS"
                    } else {
                        "NOTE_SCORP_STOCK_BASIS"
                    },
                    alloc::vec![
                        MessageArg::Number(e.id),
                        MessageArg::usd_cents(output.basis.ending.get())
                    ],
                );
                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                    #[cfg(test)]
                    message.english(),
                    message,
                ));
            }
            if let Some(basis) = &output.basis.scorp {
                for (index, loan) in basis.loans.iter().enumerate() {
                    let loan_id = MessageArg::Number((index + 1) as i64);
                    for (key, value) in [
                        (
                            if amt {
                                "NOTE_AMT_SCORP_LOAN_ENDING_BASIS"
                            } else {
                                "NOTE_SCORP_LOAN_BASIS"
                            },
                            loan.basis,
                        ),
                        (
                            if amt {
                                "NOTE_AMT_SCORP_LOAN_RESTORATION"
                            } else {
                                "NOTE_SCORP_LOAN_RESTORATION"
                            },
                            loan.restoration,
                        ),
                    ] {
                        let money = MessageArg::usd_cents(value.get());
                        let args = if amt {
                            alloc::vec![MessageArg::Number(e.id), loan_id.clone(), money]
                        } else {
                            alloc::vec![MessageArg::Number(e.id), money, loan_id.clone()]
                        };
                        let message = Message::new(key, args);
                        r.review_notes.push(crate::federal_model::ReviewNote::localized(
                            #[cfg(test)]
                            message.english(),
                            message,
                        ));
                    }
                }
            }
            if !amt {
                for component in &output.components {
                    if component.section179 != 0 || component.section179_basis_suspended != 0 {
                        for (suffix, cents, explanation) in [
                            (
                                "section179",
                                component.section179_allowed,
                                msg!("EXPLAIN_N_N_N_SECTION179"),
                            ),
                            (
                                "section179_carryforward",
                                component.section179_carry,
                                msg!("EXPLAIN_N_N_N_SECTION179_CARRYFORWARD"),
                            ),
                        ] {
                            crate::kernel::lines::line_cents(
                                r,
                                &alloc::format!(
                                    "{}_{}_{suffix}",
                                    if partner { "partnership" } else { "scorp" },
                                    component.id
                                ),
                                cents,
                                &["income.partnership", "income.scorp"],
                                "https://www.irs.gov/instructions/i4562",
                                explanation,
                            );
                        }
                    }
                }
            }
            // Retain owner-stage precision in worksheet details.
            let mut detail = |key, amount, source_id, explanation| {
                let message = Message::new(
                    key,
                    alloc::vec![
                        MessageArg::Number(e.id),
                        MessageArg::usd_cents(amount),
                        MessageArg::Number(i64::from(d.tax_year)),
                        MessageArg::Nested(alloc::boxed::Box::new(explanation)),
                        MessageArg::Number(source_id),
                    ],
                );
                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                    #[cfg(test)]
                    message.english(),
                    message,
                ));
            };
            if partner {
                detail(
                    if amt {
                        "NOTE_AMT_PARTNERSHIP_ENTITY_BASIS_ENDING"
                    } else {
                        "NOTE_PARTNERSHIP_ENTITY_BASIS_ENDING"
                    },
                    output.basis.ending.get(),
                    e.id,
                    msg!("EXPLAIN_K1_ACTIVITY_ENTITY_BASIS"),
                );
            }
            for component in &output.components {
                for (key, amount, explanation) in [
                    (
                        match (partner, amt) {
                            (true, true) => "NOTE_AMT_PARTNERSHIP_COMPONENT_BASIS_ALLOWED",
                            (true, false) => "NOTE_PARTNERSHIP_COMPONENT_BASIS_ALLOWED",
                            (false, true) => "NOTE_AMT_SCORP_ACTIVITY_COMPONENT_BASIS_ALLOWED",
                            (false, false) => "NOTE_SCORP_ACTIVITY_COMPONENT_BASIS_ALLOWED",
                        },
                        component.basis_allowed.get(),
                        msg!("EXPLAIN_K1_ACTIVITY_BASIS_ALLOWED"),
                    ),
                    (
                        match (partner, amt) {
                            (true, true) => "NOTE_AMT_PARTNERSHIP_COMPONENT_BASIS_SUSPENDED",
                            (true, false) => "NOTE_PARTNERSHIP_COMPONENT_BASIS_SUSPENDED",
                            (false, true) => "NOTE_AMT_SCORP_ACTIVITY_COMPONENT_BASIS_SUSPENDED",
                            (false, false) => "NOTE_SCORP_ACTIVITY_COMPONENT_BASIS_SUSPENDED",
                        },
                        component.basis_suspended.get(),
                        msg!("EXPLAIN_K1_ACTIVITY_BASIS_SUSPENDED"),
                    ),
                    (
                        match (partner, amt) {
                            (true, true) => "NOTE_AMT_PARTNERSHIP_COMPONENT_RISK_ALLOWED",
                            (true, false) => "NOTE_PARTNERSHIP_COMPONENT_RISK_ALLOWED",
                            (false, true) => "NOTE_AMT_SCORP_ACTIVITY_COMPONENT_RISK_ALLOWED",
                            (false, false) => "NOTE_SCORP_ACTIVITY_COMPONENT_RISK_ALLOWED",
                        },
                        component.risk_allowed.get(),
                        msg!("EXPLAIN_K1_ACTIVITY_RISK_ALLOWED"),
                    ),
                    (
                        match (partner, amt) {
                            (true, true) => "NOTE_AMT_PARTNERSHIP_COMPONENT_RISK_SUSPENDED",
                            (true, false) => "NOTE_PARTNERSHIP_COMPONENT_RISK_SUSPENDED",
                            (false, true) => "NOTE_AMT_SCORP_ACTIVITY_COMPONENT_RISK_SUSPENDED",
                            (false, false) => "NOTE_SCORP_ACTIVITY_COMPONENT_RISK_SUSPENDED",
                        },
                        component.risk_suspended.get(),
                        msg!("EXPLAIN_K1_ACTIVITY_RISK_SUSPENDED"),
                    ),
                    (
                        match (partner, amt) {
                            (true, true) => "NOTE_AMT_PARTNERSHIP_COMPONENT_PASSIVE_OPENING",
                            (true, false) => "NOTE_PARTNERSHIP_COMPONENT_PASSIVE_OPENING",
                            (false, true) => "NOTE_AMT_SCORP_ACTIVITY_COMPONENT_PASSIVE_OPENING",
                            (false, false) => "NOTE_SCORP_ACTIVITY_COMPONENT_PASSIVE_OPENING",
                        },
                        component.prior_passive.get(),
                        msg!("EXPLAIN_K1_ACTIVITY_PASSIVE_OPENING"),
                    ),
                ] {
                    detail(key, amount, component.id, explanation);
                }
            }
            for group in output.groups {
                detail(
                    match (partner, amt) {
                        (true, true) => "NOTE_AMT_PARTNERSHIP_GROUP_BEFORE_PASSIVE",
                        (true, false) => "NOTE_PARTNERSHIP_GROUP_BEFORE_PASSIVE",
                        (false, true) => "NOTE_AMT_SCORP_ACTIVITY_GROUP_BEFORE_PASSIVE",
                        (false, false) => "NOTE_SCORP_ACTIVITY_GROUP_BEFORE_PASSIVE",
                    },
                    group.current,
                    group.id,
                    msg!("EXPLAIN_K1_ACTIVITY_BEFORE_PASSIVE"),
                );
                values.push(PassiveEntry {
                    source_cents: Some(K1PassiveCents {
                        separate_nonpassive: [0; 4],
                        net: group.passive,
                        prior: group.prior,
                        ineligible_prior: 0,
                        nonpassive: group.nonpassive,
                        prior_released: group.prior_released,
                        disposition_gain: 0,
                        ptp: 0,
                    }),
                    id: e.id,
                    net: dollars(group.passive),
                    rental_allowance: false,
                    ineligible_prior: 0,
                    prior: dollars(group.prior),
                    personal: false,
                    scorp: None,
                    partner,
                    source_group: Some(group.id),
                    characters: None,
                    items: None,
                    nonpassive: dollars(group.nonpassive),
                    prior_released: dollars(group.prior_released),
                    uniform_business: None,
                    disposition_gain: 0,
                    rental_partial: false,
                    estate_trust: false,
                    estate_activity: None,
                    farm_rental: false,
                    pass_through: None,
                    released: false,
                    ptp: 0,
                });
            }
        }
    }
    // Each fiduciary activity retains its own regular/AMT passive history.
    // Section 469(i)'s rental allowance never applies to beneficiary losses.
    let estate_trust_business = 0;
    let mut estate_trust_nonpassive = 0;
    for e in &allocated_entries {
        let FederalIncome::EstateTrustPortfolio { data } = &e.income else {
            continue;
        };
        for activity in &data.activities.activities {
            let current = dollars(activity.net(amt));
            let prior = dollars(activity.prior(amt));
            let material = activity.material();
            let parts = crate::scorp::ActivityCharacters::from_current_and_prior(
                [current, 0],
                [prior, 0],
                material,
                false,
            )
            .expect("validated beneficiary activity has one income character");
            let passive = parts.passive[0];
            let nonpassive = parts.nonpassive[0];
            nonpassive_income += nonpassive;
            magi += positive(nonpassive);
            estate_trust_nonpassive += nonpassive;
            *nii_adjustment_by_id.entry(e.id).or_default() -= nonpassive;
            profits += positive(passive);
            values.push(PassiveEntry {
                source_cents: None,
                id: e.id,
                net: passive,
                rental_allowance: false,
                ineligible_prior: 0,
                prior: prior - parts.prior_released[0],
                personal: false,
                scorp: None,
                partner: false,
                source_group: None,
                characters: None,
                nonpassive,
                prior_released: parts.prior_released[0],
                uniform_business: None,
                disposition_gain: 0,
                rental_partial: false,
                estate_trust: true,
                estate_activity: Some((e.id, activity)),
                items: None,
                farm_rental: false,
                pass_through: None,
                ptp: 0,
                released: false,
            });
        }
    }
    if let Some(missing) = values.iter().find(|entry| {
        (entry.partner
            || entry.scorp.is_some()
            || entry.source_group.is_some()
            || entry.pass_through.is_some())
            && entry.source_cents.is_none()
    }) {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::PassThrough),
            "Partnership activity allocation",
            "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE",
            "This estimate does not yet calculate the combined basis, at-risk and passive limits for these activity records."
        );
        return RentalResult {
            source_issue: Some(crate::activity_risk::SourceIssue::new(
                missing.id,
                crate::activity_risk::ProjectionIssue::Source(
                    "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE",
                ),
            )),
            ..Default::default()
        };
    }
    let inputs_complete = r.issues.len() == initial_issues;
    line(
        r,
        &alloc::format!("{prefix}rental_magi"),
        magi,
        &["income.non_rental", "adjustments"],
        PASSIVE,
        msg!("EXPLAIN_0RENTAL_MAGI"),
    );
    let pool_complete =
        r.issues.iter().all(|i| !amt && i.context == crate::IssueContext::ScorpAlternativeMinimum);
    let s = d.setup.as_ref().unwrap();
    let separate = s.filing_status() == FederalStatus::Separate;
    // Form 8582 Part I caution: married filing separately and lived with the
    // spouse at any time in the year, skip Part II (go to line 10), and use
    // Part V rather than Part IV (2025 i8582).
    let together = separate && !s.lived_apart_all_year();
    // Lines 5 through 8: $150,000 ($75,000 separate) less MAGI that is not
    // below zero (line 6), half of it (line 8, a half dollar rounds up), at
    // most $25,000 ($12,500 separate), section 469(i)(2), (3)(A), (5).
    let line5 = if separate {
        crate::year_constants::PASSIVE_ALLOWANCE_PHASEOUT_END_SEPARATE_DOLLARS
    } else {
        crate::year_constants::PASSIVE_ALLOWANCE_PHASEOUT_END_DOLLARS
    };
    let line6 = positive(magi);
    let line7 = positive(line5 - line6);
    let allowance = if together {
        0
    } else {
        phase::cap(
            if separate {
                crate::year_constants::PASSIVE_ALLOWANCE_SEPARATE_DOLLARS
            } else {
                crate::year_constants::PASSIVE_ALLOWANCE_DOLLARS
            },
            [phase::offset_steps(line7, 1, 2)],
        )
    };
    line(
        r,
        &alloc::format!("{prefix}personal_rental_passive_income"),
        personal_profit,
        &["income.nonbusiness_property"],
        PASSIVE,
        msg!("EXPLAIN_0PERSONAL_RENTAL_PASSIVE_INCOME"),
    );
    let net_cents: alloc::vec::Vec<_> = values
        .iter()
        .map(|entry| {
            if entry.personal {
                let input = &personal_inputs[&entry.id];
                input.receipts_cents - input.expenses_cents - input.prior_cents
            } else if let Some(source) = &entry.source_cents {
                source.net
            } else {
                entry.net * 100
            }
        })
        .collect();
    let prior_cents: alloc::vec::Vec<_> = values
        .iter()
        .map(|entry| {
            if entry.personal {
                personal_inputs[&entry.id].prior_cents
            } else if let Some(source) = &entry.source_cents {
                source.prior
            } else {
                entry.prior * 100
            }
        })
        .collect();
    let ineligible_cents: alloc::vec::Vec<_> = values
        .iter()
        .map(|entry| {
            entry
                .source_cents
                .as_ref()
                .map_or(entry.ineligible_prior * 100, |source| source.ineligible_prior)
        })
        .collect();
    let losses: alloc::vec::Vec<i64> = values.iter().map(|entry| positive(-entry.net)).collect();
    // Direct/farm rental history passes activity_risk's source check;
    // K-1 owners validate their carried losses before constructing activities.
    // Their ineligible prior is a remaining prior loss, never a signed net.
    let pool: alloc::vec::Vec<_> = values
        .iter()
        .enumerate()
        .map(|(index, entry)| crate::kernel::loss_limits::PassiveActivity {
            key: entry.id,
            net: net_cents[index],
            rental_allowance: entry.rental_allowance && !together,
            ineligible_prior: ineligible_cents[index],
        })
        .collect();
    let allocation = crate::kernel::loss_limits::passive_pool(SourceFact::Reported(
        crate::kernel::loss_limits::PoolInput {
            activities: &pool,
            other_passive_income: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                personal_profit_cents,
            )
            .expect("native passive income is nonnegative"),
            allowance: crate::kernel::carry::Amount::new(allowance * 100)
                .expect("native passive allowance is nonnegative"),
        },
    ));
    let allocation = match allocation {
        Ok(SourceFact::Reported(value)) => value,
        Ok(_) => unreachable!("reported pool remains reported"),
        Err(error @ crate::kernel::loss_limits::LossLimitError::Overflow) => {
            // Actual source sums exceeding the native pool receiver are
            // unavailable; do not panic or impose an entered-money ceiling.
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(crate::TaxSection::PassThrough),
                "Partnership activity allocation",
                "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE",
                "This estimate does not yet calculate the combined basis, at-risk and passive limits for these activity records."
            );
            return RentalResult { pool_issue: Some(error), ..Default::default() };
        }
        Err(error) => panic!("validated rental amounts fit the passive allocation: {error:?}"),
    };
    let suspended = allocation
        .annual_suspensions()
        .expect("validated rental amounts fit the annual passive projection");
    // The losses the passive income offsets come first; the special allowance
    // lets the rest through (the shared passive_pool settlement).
    let passive_profit_cents =
        personal_profit_cents + net_cents.iter().map(|net| positive(*net)).sum::<i64>();
    let special_allowance = dollars(allocation.special_allowance());
    let active: i64 = values
        .iter()
        .filter(|e| e.rental_allowance && !together)
        .map(|e| {
            e.source_cents.as_ref().map_or((e.net + e.ineligible_prior) * 100, |source| {
                source.net + source.ineligible_prior
            })
        })
        .sum();
    let combined_cents = personal_profit_cents + net_cents.iter().sum::<i64>();
    let health_ptc = HealthPassiveProjection {
        deferred: active < 0 && combined_cents < 0,
        rentals: values
            .iter()
            .zip(&suspended)
            .enumerate()
            .filter_map(|(index, (entry, carried))| {
                if !entry.real_estate() {
                    return None;
                }
                let current_cents =
                    entry.source_cents.as_ref().map_or(
                        (entry.net - entry.disposition_gain + entry.prior) * 100,
                        |source| source.net - source.disposition_gain + source.prior,
                    );
                let prior_cents =
                    entry.source_cents.as_ref().map_or(entry.prior * 100, |source| source.prior);
                let held_cents = entry.source_cents.as_ref().map_or(*carried * 100, |_| {
                    // Same selected activity slot as the passive settlement.
                    allocation.suspended()[index]
                });
                Some(HealthRentalActivity {
                    source: entry.id,
                    current: dollars(current_cents),
                    prior: dollars(prior_cents),
                    allowed_loss: dollars(positive(-current_cents) + prior_cents - held_cents),
                })
            })
            .collect(),
        mixed_losses: values
            .iter()
            .enumerate()
            .any(|(index, e)| !e.real_estate() && (net_cents[index] < 0 || prior_cents[index] > 0))
            && (passive_profit_cents > 0
                || values.iter().any(|e| {
                    e.real_estate()
                        && e.source_cents
                            .as_ref()
                            .map_or((e.net - e.disposition_gain + e.prior) * 100, |source| {
                                source.net - source.disposition_gain + source.prior
                            })
                            > 0
                })),
        // The special list names passive losses, whereas WorksheetX line1
        // says any RE rental losses. No inspected primary example resolves
        // their intersection with a concurrent nonpassive professional loss.
        nonpassive_losses: nonpassive_rental_losses
            || values.iter().any(|e| {
                e.real_estate()
                    && e.source_cents
                        .as_ref()
                        .map_or(e.nonpassive * 100, |source| source.nonpassive)
                        < 0
            }),
        released_outside_pool: values.iter().any(|e| {
            e.real_estate()
                && e.source_cents
                    .as_ref()
                    .map_or(e.disposition_gain != 0 || e.prior_released > 0, |source| {
                        source.disposition_gain != 0 || source.prior_released > 0
                    })
        }) || released_rental_losses,
    };
    if !amt
        && (values
            .iter()
            .enumerate()
            .any(|(index, entry)| !entry.released || net_cents[index] != 0)
            || personal_profit != 0)
    {
        let rows = |r: &mut FederalResult, printed: &str, id: &str, amount: i64, explanation| {
            form_line(
                r,
                FormLine::new(TaxForm::Form8582, printed),
                id,
                amount,
                &[
                    "income.nonbusiness_property",
                    "income.business",
                    "income.partnership",
                    "income.scorp",
                    "adjustments",
                ],
                PASSIVE,
                explanation,
            );
        };
        // Parts IV and V, column by column: (a) net income, (b) net loss and
        // (c) earlier years' unallowed losses (negative). A prior loss from a
        // year without active participation goes to Part V, not Part IV.
        let (mut part_four, mut part_five) = ([0i64; 3], [0i64; 3]);
        let column = |sums: &mut [i64; 3], current: i64, prior: i64| {
            sums[0] += positive(current);
            sums[1] -= positive(-current);
            sums[2] -= prior;
        };
        for (index, entry) in values.iter().enumerate() {
            if entry.released && net_cents[index] == 0 {
                continue;
            }
            let current = net_cents[index] + prior_cents[index];
            // Income and loss reported on different forms stay in their own
            // columns even when they belong to the same activity (i8582 IV/V).
            let property = if entry.rental_partial {
                rental_source!(crate::asset_sale::kept_rental_partials(d, entry.id, amt))
            } else {
                None
            };
            let property_net = property.as_ref().map_or(0, |v| v.gain * 100);
            let current = current - property_net;
            if let Some(property) = property {
                let sums = if entry.rental_allowance && !together {
                    &mut part_four
                } else {
                    &mut part_five
                };
                column(sums, property.ordinary * 100, 0);
                column(sums, property.section1231 * 100, 0);
            }
            if entry.rental_allowance && !together {
                column(&mut part_four, current, prior_cents[index] - ineligible_cents[index]);
                column(&mut part_five, 0, ineligible_cents[index]);
            } else {
                column(&mut part_five, current, prior_cents[index]);
            }
        }
        for (_, net, prior) in &personal {
            if *net >= 0 {
                column(&mut part_five, *net + *prior, *prior);
            }
        }
        let part_four = part_four.map(dollars);
        let part_five = part_five.map(dollars);
        // These entered annual columns own the printed form's subtotals.
        // The actual cent pool and its source packet retain their separate
        // amounts; do not substitute a rounded raw net for this footer.
        let active: i64 = part_four.iter().sum();
        let combined = active + part_five.iter().sum::<i64>();
        for (printed, id, amount, explanation) in [
            ("1a", "form8582_line_1a", part_four[0], msg!("EXPLAIN_FORM8582_LINE_1A")),
            ("1b", "form8582_line_1b", part_four[1], msg!("EXPLAIN_FORM8582_LINE_1B")),
            ("1c", "form8582_line_1c", part_four[2], msg!("EXPLAIN_FORM8582_LINE_1C")),
            ("1d", "form8582_line_1d", active, msg!("EXPLAIN_FORM8582_LINE_1D")),
            ("2a", "form8582_line_2a", part_five[0], msg!("EXPLAIN_FORM8582_LINE_2A")),
            ("2b", "form8582_line_2b", part_five[1], msg!("EXPLAIN_FORM8582_LINE_2B")),
            ("2c", "form8582_line_2c", part_five[2], msg!("EXPLAIN_FORM8582_LINE_2C")),
            ("2d", "form8582_line_2d", combined - active, msg!("EXPLAIN_FORM8582_LINE_2D")),
        ] {
            // Married filing separately and lived with the spouse: Part IV
            // is not used (lines 1a-1d are blank).
            if together && printed.starts_with('1') {
                continue;
            }
            rows(r, printed, id, amount, explanation);
        }
        rows(r, "3", "form8582_line_3", combined, msg!("EXPLAIN_FORM8582_LINE_3"));
        // A net loss: Part II applies when the rental real estate with active
        // participation also shows a loss; lines 10 and 11 total the losses
        // allowed (the passive income offsets, then the special allowance).
        if combined < 0 {
            if active < 0 {
                for (printed, id, amount, explanation) in [
                    (
                        "4",
                        "form8582_line_4",
                        phase::cap(-active, [-combined]),
                        msg!("EXPLAIN_FORM8582_LINE_4"),
                    ),
                    ("5", "form8582_line_5", line5, msg!("EXPLAIN_FORM8582_LINE_5")),
                    ("6", "form8582_line_6", line6, msg!("EXPLAIN_FORM8582_LINE_6")),
                    ("7", "form8582_line_7", line7, msg!("EXPLAIN_FORM8582_LINE_7")),
                    ("8", "form8582_line_8", allowance, msg!("EXPLAIN_FORM8582_LINE_8")),
                    ("9", "form8582_line_9", special_allowance, msg!("EXPLAIN_FORM8582_LINE_9")),
                ] {
                    rows(r, printed, id, amount, explanation);
                }
            }
            // Line 10 is the income on lines 1a and 2a (Part IV and V column (a)).
            let line_10 = part_four[0] + part_five[0];
            rows(r, "10", "form8582_line_10", line_10, msg!("EXPLAIN_FORM8582_LINE_10"));
            rows(
                r,
                "11",
                "form8582_line_11",
                line_10 + special_allowance,
                msg!("EXPLAIN_FORM8582_LINE_11"),
            );
        }
    }
    let source_owners: alloc::collections::BTreeMap<_, _> =
        allocated_entries.iter().map(|entry| (entry.id, entry.owner)).collect();
    let mut post_pal = PostPalSources {
        alternative: amt,
        total_loss_cents: allocation.total_loss(),
        suspended_cents: allocation.held_loss(),
        allowed_loss_cents: allocation.allowed_loss(),
        special_allowance_cents: allocation.special_allowance(),
        sources: alloc::vec::Vec::new(),
    };
    let source_indices: alloc::vec::Vec<_> = values
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            // A PTP's own limit is not this common invocation. Do not return
            // a common-pool zero as its final allowed or suspended result.
            if entry.partner
                && allocated_entries.iter().any(|source| {
                    source.id == entry.id
                        && matches!(&source.income, FederalIncome::Partnership { data } if data.publicly_traded)
                })
            {
                return None;
            }
            let source_index = post_pal.sources.len();
            let net = net_cents[index];
            let held = allocation.suspended()[index];
            post_pal.sources.push(PostPalSourceAmounts {
                income: entry.id,
                owner: source_owners[&entry.id],
                component: entry.component(),
                common_pool: true,
                net_cents: net,
                prior_cents: prior_cents[index],
                suspended_cents: held,
                allowed_net_cents: net + held,
                characters: alloc::vec![post_pal_character(
                    if entry.personal {
                        PostPalCharacter::PersonalExpense
                    } else {
                        PostPalCharacter::ScheduleE
                    },
                    net,
                    held,
                )],
            });
            Some(source_index)
        })
        .collect();
    // Positive and zero personal sources have a genuine complete zero hold,
    // including recharacterized income excluded from the common profit.
    for entry in &allocated_entries {
        let Some(input) = personal_inputs.get(&entry.id) else {
            continue;
        };
        let net = input.receipts_cents - input.expenses_cents - input.prior_cents;
        if net >= 0 {
            post_pal.sources.push(PostPalSourceAmounts {
                income: entry.id,
                owner: input.owner,
                component: PostPalComponent::PersonalProperty,
                common_pool: !input.investment_recharacterization || net == 0,
                net_cents: net,
                prior_cents: input.prior_cents,
                suspended_cents: 0,
                allowed_net_cents: net,
                characters: alloc::vec![post_pal_character(
                    PostPalCharacter::PersonalExpense,
                    net,
                    0,
                )],
            });
        }
    }
    let mut result = RentalResult {
        source_issue: source_issue.clone(),
        pool_issue: None,
        section179_allocations: section179_allocations.into_inner(),
        business_losses_by_id: alloc::collections::BTreeMap::new(),
        ordinary_business_by_id: alloc::collections::BTreeMap::new(),
        qbi_source_ids: alloc::vec::Vec::new(),
        health_ptc,
        net_passive_income: dollars(combined_cents),
        special_allowance,
        modified_agi: positive(magi),
        inputs_complete,
        income_by_id,
        prior_home_casualty_by_owner,
        nii_adjustment_by_id,
        personal_inputs,
        operating_sources,
        farm_rental_property,
        farm_rental_property_rows,
        operating_prior_allowed,
        post_pal: None,
        // Personal deductions belong to their Schedule1 owner. Sum actual
        // ScheduleE source allowances, rather than subtracting a common loss
        // then adding a separately rounded Personal residual back.
        income: profits
            - values
                .iter()
                .enumerate()
                .filter(|(_, entry)| !entry.personal)
                .map(|(index, _)| losses[index] - suspended[index])
                .sum::<i64>()
            + nonpassive_income
            + ptp_income,
        health_work_income_residual_cents,
        ptp_income,
        passive_loss: dollars(allocation.held_loss()) + ptp_suspended,
        scorp_niit_adjustment: 0,
        partner_niit_adjustment: 0,
        rental_niit_adjustment,
        estate_activities: alloc::vec::Vec::new(),
        estate_columns: [0; 4],
        estate_trust_business,
        estate_trust_nonpassive,
        estate_trust_niit_adjustment: -estate_trust_nonpassive,
        scorp_business: 0,
        partners: alloc::vec::Vec::new(),
        k1_amounts: alloc::vec::Vec::new(),
        k1_ledgers,
        partner_se: alloc::vec::Vec::new(),
        counterpart_partner_se: alloc::vec::Vec::new(),
        scorp_nonpassive: 0,
        pass_through_rental_nonpassive: 0,
        qbi: 0,
        // Filled from complete native allowed costs after source validation.
        personal_expense_adjustment: 0,
        part_one,
        farm_rental: 0,
        activities: alloc::vec::Vec::new(),
        qbi_loss_sources: alloc::vec::Vec::new(),
        scorp_suspensions: alloc::vec::Vec::new(),
        k1_section1231: alloc::vec::Vec::new(),
        scorp_qbi_ordinary_suspensions: alloc::vec::Vec::new(),
        k1_cuts: K1Cuts::default(),
        rental_partial_cuts: alloc::vec::Vec::new(),
    };
    // Preserve the existing annual display residuals; these are never raw
    // carry operands or a replacement for the same-invocation cent packet.
    let personal_display_balances: alloc::collections::BTreeMap<_, _> = values
        .iter()
        .zip(&suspended)
        .filter_map(|(entry, balance)| entry.personal.then_some((entry.id, *balance)))
        .collect();
    let mut professional_pass_through_rentals = 0;
    for (i, entry) in values.into_iter().enumerate() {
        let PassiveEntry {
            id,
            net,
            source_cents,
            personal,
            scorp,
            partner,
            source_group,
            characters,
            mut nonpassive,
            mut prior_released,
            uniform_business,
            disposition_gain,
            rental_partial,
            estate_trust,
            estate_activity,
            farm_rental,
            pass_through,
            prior,
            ptp,
            items,
            ..
        } = entry;
        // A publicly traded partnership's income and its still-suspended
        // loss, apart from the pool (section 469(k)).
        let bound_ptp = selected_ptp(id);
        let ptp_gain_cents = bound_ptp.map_or_else(
            || source_cents.as_ref().map_or(positive(ptp) * 100, |source| positive(source.ptp)),
            |source| source.net_gain_cents,
        );
        let ptp_gain = dollars(ptp_gain_cents);
        let ptp_ordinary_held_cents = bound_ptp.map_or_else(
            || source_cents.as_ref().map_or(positive(-ptp) * 100, |source| positive(-source.ptp)),
            |source| {
                source
                    .components
                    .iter()
                    .filter(|part| {
                        part.original.character == crate::ptp_passive::PtpPalCharacter::Ordinary
                    })
                    .map(|part| part.suspended_cents)
                    .sum::<i64>()
            },
        );
        let ptp_ordinary_held = dollars(ptp_ordinary_held_cents);
        let mut carried = suspended[i] + ptp_ordinary_held;
        // The part of it the ordinary (Schedule E) character keeps once the
        // activity items take their share (unit B9-1).
        let mut ordinary_suspended = suspended[i];
        let mut source_slot_held_cents = allocation.suspended()[i] + ptp_ordinary_held_cents;
        let mut exact_allowed_net_cents = net_cents[i] + allocation.suspended()[i];
        let reported_nonpassive = |cents| {
            let separately_stated =
                source_cents.as_ref().map_or([0; 4], |source| source.separate_nonpassive);
            dollars(cents - separately_stated.iter().sum::<i64>())
                + separately_stated.into_iter().map(dollars).sum::<i64>()
        };
        let mut exact_nonpassive_cents =
            source_cents.as_ref().map_or(nonpassive * 100, |source| source.nonpassive);
        let exact_disposition_cents =
            source_cents.as_ref().map_or(disposition_gain * 100, |source| source.disposition_gain);
        exact_allowed_net_cents -= exact_disposition_cents;
        let mut allowed_net = if source_cents.is_some() {
            dollars(exact_allowed_net_cents)
        } else if personal {
            dollars(net_cents[i] + allocation.suspended()[i])
        } else {
            positive(net) - (losses[i] - suspended[i])
        };
        // The sale gain is Form 4797 income; Schedule E keeps the rest.
        if source_cents.is_none() {
            allowed_net -= disposition_gain;
        }
        result.income -= disposition_gain;
        if disposition_gain != 0
            && let Some(source) = source_indices[i]
        {
            let character_nets = [net_cents[i] - exact_disposition_cents, exact_disposition_cents]
                .into_iter()
                .enumerate()
                .collect::<alloc::vec::Vec<_>>();
            let SourceFact::Reported(exact) = crate::kernel::loss_limits::passive_characters(
                SourceFact::Reported(crate::kernel::loss_limits::CharacterInput {
                    nets: &character_nets,
                    suspended: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                        allocation.suspended()[i],
                    )
                    .expect("native suspension is nonnegative"),
                }),
            )
            .expect("native disposition characters retain their common cent allowance") else {
                unreachable!("reported characters remain reported")
            };
            source_slot_held_cents = exact[0].held();
            post_pal.sources[source].characters =
                [PostPalCharacter::ScheduleE, PostPalCharacter::PropertyDisposition]
                    .into_iter()
                    .zip(&exact)
                    .map(|(character, amount)| post_pal_class_amounts(character, amount))
                    .collect();
        }
        if rental_partial
            && let Some(property) =
                rental_source!(crate::asset_sale::kept_rental_partials(d, id, amt))
        {
            // Reuse Part IX's character allocator. Schedule E, Part II and
            // Part I of Form 4797 retain their own allowed loss; subtracting
            // the raw property loss from the pool would allow it twice.
            let nets = [net - disposition_gain, property.ordinary, property.section1231];
            let character_nets = nets.into_iter().enumerate().collect::<alloc::vec::Vec<_>>();
            let SourceFact::Reported(split) = crate::kernel::loss_limits::passive_characters(
                SourceFact::Reported(crate::kernel::loss_limits::CharacterInput {
                    nets: &character_nets,
                    suspended: crate::kernel::carry::Amount::<crate::kernel::carry::Dollars>::new(
                        suspended[i],
                    )
                    .expect("native suspension is nonnegative"),
                }),
            )
            .expect("rental partial characters reconcile with their passive activity") else {
                unreachable!("reported characters remain reported")
            };
            let split: alloc::vec::Vec<_> = split.iter().map(|amount| amount.held()).collect();
            if let Some(source) = source_indices[i] {
                let character_nets = nets
                    .map(|net| net * 100)
                    .into_iter()
                    .enumerate()
                    .collect::<alloc::vec::Vec<_>>();
                let SourceFact::Reported(exact) = crate::kernel::loss_limits::passive_characters(
                    SourceFact::Reported(crate::kernel::loss_limits::CharacterInput {
                        nets: &character_nets,
                        suspended:
                            crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                                allocation.suspended()[i],
                            )
                            .expect("native suspension is nonnegative"),
                    }),
                )
                .expect("native rental characters retain their common cent allowance") else {
                    unreachable!("reported characters remain reported")
                };
                source_slot_held_cents = exact[0].held();
                post_pal.sources[source].characters = [
                    PostPalCharacter::ScheduleE,
                    PostPalCharacter::OrdinaryProperty,
                    PostPalCharacter::Section1231,
                ]
                .into_iter()
                .zip(&exact)
                .map(|(character, amount)| post_pal_class_amounts(character, amount))
                .collect();
            }
            ordinary_suspended = split[0];
            let property_cut = split[1] + split[2];
            allowed_net -= property_cut;
            result.income -= property_cut;
            result.rental_partial_cuts.push((id, [split[1], split[2]]));
            if !amt {
                for (index, kind, unallowed_message, allowed_message) in [
                    (
                        0,
                        "ordinary",
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_UNALLOWED"),
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_ALLOWED"),
                    ),
                    (
                        1,
                        "short_property",
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_UNALLOWED"),
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_ALLOWED"),
                    ),
                    (
                        2,
                        "section1231",
                        msg!("EXPLAIN_FORM8582_PART_IX_N_SECTION1231_UNALLOWED"),
                        msg!("EXPLAIN_FORM8582_PART_IX_N_SECTION1231_ALLOWED"),
                    ),
                ] {
                    if nets[index] < 0 {
                        emit_lines(
                            r,
                            &mut (),
                            [
                                ("unallowed", "IX (d)", split[index], unallowed_message),
                                ("allowed", "IX (e)", -nets[index] - split[index], allowed_message),
                            ],
                            |(suffix, column, amount, explanation), _, _| {
                                Some(calculation_row(
                                    Some(FormLine::new(TaxForm::Form8582, column)),
                                    &alloc::format!("form8582_part_ix_{id}_{kind}_{suffix}"),
                                    LineValue::MoneyDollars(amount),
                                    &["income.rental", "rental_magi"],
                                    PASSIVE,
                                    explanation,
                                ))
                            },
                        );
                    }
                }
            }
        }
        if let Some(parts) = characters {
            let source_parts = if source_cents.is_some() {
                parts
            } else {
                crate::scorp::ActivityCharacters {
                    passive: parts.passive.map(|value| value * 100),
                    nonpassive: parts.nonpassive.map(|value| value * 100),
                    prior_released: parts.prior_released.map(|value| value * 100),
                }
            };
            let character_nets =
                source_parts.passive.into_iter().enumerate().collect::<alloc::vec::Vec<_>>();
            let SourceFact::Reported(class_amounts) =
                crate::kernel::loss_limits::passive_characters(SourceFact::Reported(
                    crate::kernel::loss_limits::CharacterInput {
                        nets: &character_nets,
                        suspended:
                            crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                                allocation.suspended()[i],
                            )
                            .expect("native suspension is nonnegative"),
                    },
                ))
                .expect("validated character nets reconcile with the shared activity pool")
            else {
                unreachable!("reported characters remain reported")
            };
            let split: alloc::vec::Vec<_> =
                class_amounts.iter().map(|amount| dollars(amount.held())).collect();
            if let Some(source) = source_indices[i] {
                source_slot_held_cents = class_amounts[0].held();
                post_pal.sources[source].characters =
                    [PostPalCharacter::ScheduleE, PostPalCharacter::Section1231]
                        .into_iter()
                        .zip(&class_amounts)
                        .map(|(character, amount)| post_pal_class_amounts(character, amount))
                        .collect();
            }
            let parts = crate::scorp::ActivityCharacters {
                passive: source_parts.passive.map(dollars),
                nonpassive: source_parts.nonpassive.map(dollars),
                prior_released: source_parts.prior_released.map(dollars),
            };
            carried = split[0];
            ordinary_suspended = split[0];
            if partner {
                prior_released = parts.prior_released[0];
            }
            if scorp.is_some() {
                result.scorp_qbi_ordinary_suspensions.push((
                    id,
                    crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                        class_amounts[0].held(),
                    )
                    .expect("nonnegative ordinary source hold"),
                ));
            }
            if !amt
                && let Some((_, ledger)) = result.k1_ledgers.iter_mut().find(|(key, _)| *key == id)
            {
                let class = crate::k1_loss::K1LossClass::Section1231;
                ledger.passive_suspended[class] = crate::kernel::carry::Amount::new(
                    ledger.passive_suspended[class].get() + class_amounts[1].held(),
                )
                .expect("nonnegative source suspension");
            }
            // Part IX: an activity whose losses go to two forms or schedules
            // (ordinary on Schedule E, section 1231 on Form 4797): column (d)
            // the unallowed and column (e) the allowed part of each loss.
            if !amt {
                for (suffix, column, loss, unallowed, explanation) in [
                    (
                        "ordinary_unallowed",
                        "IX (d)",
                        parts.passive[0],
                        split[0],
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_UNALLOWED"),
                    ),
                    (
                        "ordinary_allowed",
                        "IX (e)",
                        parts.passive[0],
                        split[0],
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_ALLOWED"),
                    ),
                    (
                        "section1231_unallowed",
                        "IX (d)",
                        parts.passive[1],
                        split[1],
                        msg!("EXPLAIN_FORM8582_PART_IX_N_SECTION1231_UNALLOWED"),
                    ),
                    (
                        "section1231_allowed",
                        "IX (e)",
                        parts.passive[1],
                        split[1],
                        msg!("EXPLAIN_FORM8582_PART_IX_N_SECTION1231_ALLOWED"),
                    ),
                ] {
                    if loss >= 0 {
                        continue;
                    }
                    form_line(
                        r,
                        FormLine::new(TaxForm::Form8582, column),
                        &match &pass_through {
                            Some(rental) => alloc::format!(
                                "form8582_part_ix_{id}_{}_{suffix}",
                                if rental.real_estate {
                                    "rental_real_estate"
                                } else {
                                    "other_rental"
                                }
                            ),
                            None => alloc::format!("form8582_part_ix_{id}_{suffix}"),
                        },
                        if suffix.ends_with("_unallowed") { unallowed } else { -loss - unallowed },
                        &["income.partnership", "income.scorp"],
                        PASSIVE,
                        explanation,
                    );
                }
            }
            let property_net = dollars(class_amounts[1].allowed_net() + source_parts.nonpassive[1]);
            exact_allowed_net_cents = class_amounts[0].allowed_net();
            allowed_net = dollars(exact_allowed_net_cents);
            let all_nonpassive_cents =
                source_cents.as_ref().map_or(nonpassive * 100, |source| source.nonpassive);
            let other_nonpassive =
                all_nonpassive_cents - source_parts.nonpassive.iter().sum::<i64>();
            exact_nonpassive_cents = source_parts.nonpassive[0] + other_nonpassive;
            nonpassive = reported_nonpassive(exact_nonpassive_cents);
            // Form 4797 character is determined later. Never feed property
            // amounts into Schedule E ordinary income, QBI or ordinary NIIT.
            result.income -= property_net - disposition_gain;
            result.add_property(
                id,
                class_amounts[1].allowed_net() + source_parts.nonpassive[1],
                class_amounts[1].held(),
                source_parts.nonpassive[1],
                pass_through.as_ref().map(|rental| K1RentalPropertyResult {
                    real_estate: rental.real_estate,
                    net_cents: class_amounts[1].allowed_net() + source_parts.nonpassive[1],
                    suspended_cents:
                        crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                            class_amounts[1].held(),
                        )
                        .expect("nonnegative rental property source hold"),
                    nonpassive_cents: source_parts.nonpassive[1],
                    net: property_net,
                    suspended: split[1],
                    nonpassive: parts.nonpassive[1],
                    exclude_niit: rental.exclude_niit,
                    niit_disposition_loss: rental.niit_disposition_loss,
                    qbi_index: None,
                }),
            );
            if !amt
                && inputs_complete
                && let Some(source) = scorp
                && source.property_tracking()
                && source.qbi_left_out().is_none()
                && let Ok(basis) = source.basis_result()
            {
                let source = source.qbi_projection();
                source.property_qbi.report(
                    &source,
                    &basis,
                    id,
                    Some(
                        crate::kernel::carry::Amount::new(class_amounts[1].held())
                            .expect("nonnegative property source hold"),
                    ),
                    r,
                );
            }
            if !amt
                && inputs_complete
                && let Some(source) = scorp
                && let Some(statement) = &source.qbi_statement
                && source.qbi_left_out().is_none()
            {
                // This source-stage preview precedes current SSTB eligibility.
                // Do not route it into final QBI until property income is in AGI.
                match statement.activities(
                    source,
                    crate::kernel::carry::Amount::new(class_amounts[0].held())
                        .expect("nonnegative ordinary source hold"),
                    dollars(result.section179_allowed_cents(d, id)),
                ) {
                    Ok(items) => {
                        let current = items.first().map_or(0, |v| v.income);
                        let historical = -items.iter().skip(1).map(|v| v.income).sum::<i64>();
                        r.review_notes.push(crate::federal_model::ReviewNote::localized(
                            #[cfg(test)] alloc::format!("S-corporation document {id}, ordinary QBI source after shared activity limits: current ${current}; released historical qualified loss ${historical}. Property QBI is separate. Current amounts precede SSTB eligibility and remaining return limits; this is not a final QBI deduction."),
                            Message::new("NOTE_SCORP_ORDINARY_QBI_SOURCE", alloc::vec![
                                MessageArg::Number(id), MessageArg::usd_dollars(current), MessageArg::usd_dollars(historical),
                            ]),
                        ));
                    }
                    Err((key, _message)) => {
                        #[cfg(test)]
                        let message = _message;
                        crate::issue_localized!(
                            r,
                            &crate::IssueContext::income_id(id),
                            "S-corporation ordinary QBI source",
                            key,
                            message,
                        )
                    }
                }
            }
            #[cfg(test)]
            let tax_label = if amt { "AMT" } else { "regular" };
            #[cfg(test)]
            let entity = if partner { "Partnership" } else { "S-corporation" };
            r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
                "{entity} document {id}, {tax_label} passive character worksheet: ordinary net ${}, ordinary suspended loss ${}; section 1231 net ${property_net}, section 1231 suspended loss ${}. Nonpassive ordinary net ${nonpassive}; nonpassive section 1231 net ${}; prior loss absorbed by own income: ordinary ${}, section 1231 ${}. Intermediate allocation using resolved pool inputs; unresolved inputs may change these amounts. Section 1231 character and return-level limits are not final.", allowed_net + nonpassive, split[0], split[1], parts.nonpassive[1], parts.prior_released[0], parts.prior_released[1],
            ), Message::new(if partner { if amt { "NOTE_PARTNER_AMT_PASSIVE_CHARACTER" } else { "NOTE_PARTNER_REGULAR_PASSIVE_CHARACTER" } } else if amt { "NOTE_SCORP_AMT_PASSIVE_CHARACTER" } else { "NOTE_SCORP_REGULAR_PASSIVE_CHARACTER" }, alloc::vec![
                MessageArg::Number(id), MessageArg::usd_dollars(allowed_net + nonpassive),
                MessageArg::usd_dollars(split[0]), MessageArg::usd_dollars(property_net),
                MessageArg::usd_dollars(split[1]), MessageArg::usd_dollars(nonpassive),
                MessageArg::usd_dollars(parts.nonpassive[1]), MessageArg::usd_dollars(parts.prior_released[0]),
                MessageArg::usd_dollars(parts.prior_released[1]),
            ])));
        }
        // Unit B9-1: the activity items (code E, R and S) are part of the
        // activity's net. When the net is a loss the passive rules hold back,
        // the unallowed part is shared among the characters that have a net
        // loss in proportion to it (Form 8582 Parts VII to IX), and the
        // allowed part of each loss reaches its own form: code R Form 4797,
        // code S Schedule D. Nothing of them is Schedule E income.
        if let Some((parts, in_pool)) = items {
            let mut exact_item_cuts = None;
            if in_pool && let Some(source) = source_indices[i] {
                let slot = characters.map_or(net_cents[i], |parts| parts.passive[0]);
                let character_nets =
                    [slot - parts.net(), parts.ordinary, parts.short, parts.long, parts.debt]
                        .into_iter()
                        .enumerate()
                        .collect::<alloc::vec::Vec<_>>();
                let SourceFact::Reported(exact) = crate::kernel::loss_limits::passive_characters(
                    SourceFact::Reported(crate::kernel::loss_limits::CharacterInput {
                        nets: &character_nets,
                        suspended:
                            crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                                source_slot_held_cents,
                            )
                            .expect("native suspension is nonnegative"),
                    }),
                )
                .expect("native activity items retain their common cent allowance") else {
                    unreachable!("reported characters remain reported")
                };
                exact_item_cuts =
                    Some([exact[0].held(), exact[1].held(), exact[2].held(), exact[3].held()]);
                source_slot_held_cents = exact[0].held();
                let mut classes: alloc::vec::Vec<_> = [
                    PostPalCharacter::ScheduleE,
                    PostPalCharacter::OrdinaryProperty,
                    PostPalCharacter::ShortCapital,
                    PostPalCharacter::LongCapital,
                    PostPalCharacter::DebtCancellation,
                ]
                .into_iter()
                .zip(&exact)
                .map(|(character, amount)| post_pal_class_amounts(character, amount))
                .collect();
                classes.extend(
                    post_pal.sources[source]
                        .characters
                        .iter()
                        .filter(|amount| amount.character != PostPalCharacter::ScheduleE)
                        .cloned(),
                );
                post_pal.sources[source].characters = classes;
            }
            let exact_cuts = if !in_pool {
                [0; 4]
            } else if let Some(cuts) = exact_item_cuts {
                cuts
            } else {
                // A PTP is intentionally absent from the common source packet.
                // Its own owner result remains the source of its character cut.
                let slot = characters.map_or(
                    net_cents[i] + source_cents.as_ref().map_or(ptp * 100, |source| source.ptp),
                    |c| c.passive[0],
                );
                parts
                    .cuts(slot - parts.net(), allocation.suspended()[i] + ptp_ordinary_held_cents)
                    .expect("the activity items reconcile with their owner passive result")
            };
            let exact_cut_sum = exact_cuts[1] + exact_cuts[2] + exact_cuts[3];
            let exact_items_allowed = parts.net() + exact_cut_sum;
            if in_pool {
                exact_allowed_net_cents -= exact_items_allowed;
                source_slot_held_cents = exact_cuts[0];
            } else {
                exact_nonpassive_cents -= parts.net();
            }
            let cuts = exact_cuts.map(dollars);
            let cut_sum = cuts[1] + cuts[2] + cuts[3];
            let items_allowed = dollars(exact_items_allowed);
            result.k1_cuts.ordinary =
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    result.k1_cuts.ordinary.get() + exact_cuts[1],
                )
                .expect("nonnegative source cut");
            result.k1_cuts.ordinary_by_id.push((
                id,
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(exact_cuts[1])
                    .expect("nonnegative source cut"),
            ));
            result.k1_cuts.income_by_id.push((
                id,
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(exact_cut_sum)
                    .expect("nonnegative source cut"),
            ));
            result.k1_cuts.short =
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    result.k1_cuts.short.get() + exact_cuts[2],
                )
                .expect("nonnegative source cut");
            result.k1_cuts.long = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                result.k1_cuts.long.get() + exact_cuts[3],
            )
            .expect("nonnegative source cut");
            result.k1_cuts.capital_by_id.push((
                id,
                [exact_cuts[2], exact_cuts[3]].map(|value| {
                    crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(value)
                        .expect("nonnegative source cut")
                }),
            ));
            result.k1_cuts.passive =
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    result.k1_cuts.passive.get() + exact_cut_sum,
                )
                .expect("nonnegative source cut");
            result.income -= items_allowed;
            if in_pool {
                allowed_net = dollars(exact_allowed_net_cents);
                carried = cuts[0];
                ordinary_suspended =
                    dollars(phase::floor(exact_cuts[0] - ptp_ordinary_held_cents, 0));
            } else {
                nonpassive = reported_nonpassive(exact_nonpassive_cents);
            }
            if characters.is_some()
                && in_pool
                && let Some(last) =
                    result.scorp_qbi_ordinary_suspensions.iter_mut().rfind(|(k, _)| *k == id)
            {
                last.1 = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    source_slot_held_cents,
                )
                .expect("nonnegative ordinary source hold");
            }
            if !amt
                && exact_cut_sum != 0
                && let Some((_, ledger)) = result.k1_ledgers.iter_mut().find(|(k, _)| *k == id)
            {
                let class = crate::k1_loss::K1LossClass::OtherLoss;
                ledger.passive_suspended[class] = crate::kernel::carry::Amount::new(
                    ledger.passive_suspended[class].get() + exact_cut_sum,
                )
                .expect("nonnegative source suspension");
            }
            if !amt && in_pool && inputs_complete {
                // Part IX: the unallowed and allowed part of each loss of an
                // activity whose losses go to more than one form.
                let loss = positive(-(parts.ordinary + parts.short + parts.long));
                for (suffix, column, cut, explanation) in [
                    (
                        "items_unallowed",
                        "IX (d)",
                        cut_sum,
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ITEMS_UNALLOWED"),
                    ),
                    (
                        "items_allowed",
                        "IX (e)",
                        loss - cut_sum,
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ITEMS_ALLOWED"),
                    ),
                ] {
                    if loss > 0 {
                        form_line(
                            r,
                            FormLine::new(TaxForm::Form8582, column),
                            &alloc::format!("form8582_part_ix_{id}_{suffix}"),
                            cut,
                            &["income.partnership", "income.scorp"],
                            PASSIVE,
                            explanation,
                        );
                    }
                }
            }
        }
        // Schedule K-1 (1065) box 12 / (1120-S) box 11 section 179 allowed on
        // Form 4562 line 12 (T2-29b): Schedule E line 28 column (j) of a
        // nonpassive activity, using this invocation's selected tax system.
        if (partner || scorp.is_some()) && source_group.is_none() && bound_ptp.is_none() {
            let section179 = dollars(result.section179_allowed_cents(d, id));
            nonpassive -= section179;
            exact_nonpassive_cents -= result.section179_allowed_cents(d, id);
            result.income -= section179;
        }
        if partner && source_group.is_none() {
            // Form 2441 line24 is a partner-level nonpassive Schedule E
            // deduction, not a second change to the partnership's box1 or basis facts.
            let deductible = care.source(id);
            nonpassive = crate::kernel::carry::native_balance(nonpassive, deductible);
            exact_nonpassive_cents =
                crate::kernel::carry::native_balance(exact_nonpassive_cents, deductible * 100);
            result.income = crate::kernel::carry::native_balance(result.income, deductible);
            if deductible != 0 && !amt {
                form_line(
                    r,
                    FormLine::new(TaxForm::ScheduleE, "28 (j)"),
                    &alloc::format!("partnership_{id}_care_deduction"),
                    deductible,
                    &["credits.care.own_business", "income.partnership"],
                    "https://www.irs.gov/instructions/i2441",
                    msg!("EXPLAIN_CARE_SOURCE_DEDUCTION"),
                );
            }
        }
        if !personal {
            // Direct rentals' nonpassive result was retained at its original
            // part_one emission above. Only their remaining passive net is here.
            let amount = allowed_net
                + if partner
                    || scorp.is_some()
                    || source_group.is_some()
                    || pass_through.is_some()
                    || estate_trust
                {
                    nonpassive + ptp_gain
                } else {
                    0
                };
            *result.income_by_id.entry(id).or_default() += amount;
        }
        if partner || scorp.is_some() || source_group.is_some() || pass_through.is_some() {
            let amount = allowed_net + nonpassive + ptp_gain;
            let source_net_cents = source_cents.as_ref().and_then(|_| {
                exact_allowed_net_cents
                    .checked_add(exact_nonpassive_cents)
                    .and_then(|net| net.checked_add(ptp_gain_cents))
            });
            if source_cents.is_some() && source_net_cents.is_none() {
                crate::issue_localized!(
                    r,
                    &crate::IssueContext::income_id(id),
                    "K-1 source endpoint",
                    "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                    "Review the document amounts; they must be within the supported input range."
                );
                result.source_issue = Some(crate::activity_risk::SourceIssue::new(
                    id,
                    crate::activity_risk::ProjectionIssue::Source(
                        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                    ),
                ));
                result.inputs_complete = false;
            }

            match result.k1_amounts.iter_mut().find(|v| v.id == id) {
                Some(existing) => {
                    existing.net += amount;
                    existing.source_net_cents = match (existing.source_net_cents, source_net_cents)
                    {
                        (Some(left), Some(right)) => match left.checked_add(right) {
                            Some(total) => Some(total),
                            None => {
                                crate::issue_localized!(
                                    r,
                                    &crate::IssueContext::income_id(id),
                                    "K-1 source endpoint",
                                    "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                                    "Review the document amounts; they must be within the supported input range."
                                );
                                result.source_issue = Some(crate::activity_risk::SourceIssue::new(
                                    id,
                                    crate::activity_risk::ProjectionIssue::Source(
                                        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                                    ),
                                ));
                                result.inputs_complete = false;
                                None
                            }
                        },
                        _ => None,
                    };
                }
                None => result.k1_amounts.push(K1Amount { id, net: amount, source_net_cents }),
            }
            // The passive stage of the K-1's ledger (unit B8): what the
            // passive rules hold back stays with the class that lost it,
            // including a publicly traded partnership's own suspended box 1
            // loss (section 469(k), `carried` adds it to the pool's share).
            if !amt && let Some((_, ledger)) = result.k1_ledgers.iter_mut().find(|(k, _)| *k == id)
            {
                let class = match &pass_through {
                    Some(rental) if rental.real_estate => {
                        crate::k1_loss::K1LossClass::RentalRealEstate
                    }
                    Some(_) => crate::k1_loss::K1LossClass::OtherRental,
                    None => crate::k1_loss::K1LossClass::Ordinary,
                };
                ledger.passive_suspended[class] = crate::kernel::carry::Amount::new(
                    ledger.passive_suspended[class].get() + source_slot_held_cents,
                )
                .expect("nonnegative source suspension");
                if let Some(source) = bound_ptp {
                    let class = crate::k1_loss::K1LossClass::Section179;
                    let held_cents = source
                        .components
                        .iter()
                        .filter(|part| {
                            part.original.character
                                == crate::ptp_passive::PtpPalCharacter::Section179
                        })
                        .map(|part| part.suspended_cents)
                        .sum::<i64>();
                    ledger.passive_suspended[class] = crate::kernel::carry::Amount::new(
                        ledger.passive_suspended[class].get() + held_cents,
                    )
                    .expect("nonnegative source suspension");
                }
            }
        }
        if partner || scorp.is_some() || source_group.is_some() || estate_trust {
            *result.business_losses_by_id.entry(id).or_default() +=
                positive(-(allowed_net + nonpassive + ptp_gain));
        }
        if partner {
            *result.ordinary_business_by_id.entry(id).or_default() +=
                allowed_net + nonpassive + ptp_gain;
            if let Some(value) = result.partners.iter_mut().find(|value| value.id == id) {
                value.net += allowed_net + nonpassive + ptp_gain;
                value.nonpassive += nonpassive;
                value.suspended = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    value.suspended.get() + source_slot_held_cents,
                )
                .expect("nonnegative source hold");
                value.prior_released += prior_released;
            } else {
                result.partners.push(PartnerPassiveResult {
                    id,
                    net: allowed_net + nonpassive + ptp_gain,
                    nonpassive,
                    suspended: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                        source_slot_held_cents,
                    )
                    .expect("nonnegative source hold"),
                    prior_released,
                });
            }
            if source_group.is_some() && !amt {
                result.partner_niit_adjustment -= nonpassive;
                *result.nii_adjustment_by_id.entry(id).or_default() -= nonpassive;
            }
            if inputs_complete {
                let source_id = source_group.map_or_else(
                    || alloc::format!("partnership_{id}"),
                    |group| alloc::format!("partnership_{id}_group_{group}"),
                );
                line(
                    r,
                    &alloc::format!("{prefix}{source_id}_passive_suspended"),
                    carried,
                    &["income.partnership"],
                    PASSIVE,
                    msg!("EXPLAIN_0PARTNERSHIP_N_PASSIVE_SUSPENDED"),
                );
                line(
                    r,
                    &alloc::format!("{prefix}{source_id}_activity_net"),
                    allowed_net + nonpassive + ptp_gain,
                    &["income.partnership"],
                    PASSIVE,
                    msg!("EXPLAIN_0PARTNERSHIP_N_ACTIVITY_NET"),
                );
            }
        }
        if !partner && let Some(group) = source_group {
            if inputs_complete {
                for (stage, amount, explanation) in [
                    (
                        "passive_suspended",
                        carried,
                        msg!("EXPLAIN_SCORP_ACTIVITY_PASSIVE_SUSPENDED"),
                    ),
                    ("activity_net", allowed_net + nonpassive, msg!("EXPLAIN_SCORP_ACTIVITY_NET")),
                ] {
                    line(
                        r,
                        &alloc::format!("{prefix}scorp_{id}_group_{group}_{stage}"),
                        amount,
                        &["income.scorp.activities"],
                        PASSIVE,
                        explanation,
                    );
                }
            }

            result.scorp_business += allowed_net + nonpassive;
            result.scorp_nonpassive += nonpassive;
            *result.ordinary_business_by_id.entry(id).or_default() += allowed_net + nonpassive;
            if !amt {
                result.scorp_niit_adjustment -= nonpassive;
                *result.nii_adjustment_by_id.entry(id).or_default() -= nonpassive;
            }
        }
        if let Some(source) = scorp {
            result.scorp_business += allowed_net + nonpassive;
            *result.ordinary_business_by_id.entry(id).or_default() += allowed_net + nonpassive;
            result.scorp_nonpassive += nonpassive;
            if !amt {
                // Form 8960 line 4b reverses nonpassive ordinary net, including
                // the own-income release of a former passive loss. Losses released
                // against OTHER passive income remain in allowed_net.
                let adjustment = match source.trade_kind {
                    crate::ScorpTradeKind::Ordinary => -nonpassive,
                    crate::ScorpTradeKind::FinancialTrading => 0,
                };
                result.scorp_niit_adjustment += adjustment;
                *result.nii_adjustment_by_id.entry(id).or_default() += adjustment;
                if pool_complete {
                    let nii = allowed_net + nonpassive + adjustment;
                    r.review_notes.push(crate::federal_model::ReviewNote::localized(
                        #[cfg(test)] alloc::format!("S-corporation document {id}, investment income after activity limits: ${nii}; Form 8960 adjustment: ${adjustment}. Ordinary nonpassive business net is excluded; financial-trading net remains included. Additional return-level limits remain separate."),
                        Message::new("NOTE_SCORP_INVESTMENT_INCOME", alloc::vec![
                            MessageArg::Number(id), MessageArg::usd_dollars(nii), MessageArg::usd_dollars(adjustment),
                        ]),
                    ));
                }
            }
        }
        // Carry resolved source inputs independently of unrelated return issues.
        if inputs_complete
            && characters.is_none()
            && scorp.is_some_and(|source| source.loss_tracking())
        {
            result.scorp_suspensions.push((
                id,
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    source_slot_held_cents,
                )
                .expect("nonnegative ordinary source hold"),
            ));
        }
        if has_scorp && pool_complete && !personal && characters.is_none() && pass_through.is_none()
        {
            if let Some(source) = scorp {
                #[cfg(test)]
                let tax_label = if amt { "AMT S-corporation" } else { "S-corporation" };
                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                    #[cfg(test)] alloc::format!("{tax_label} document {id}, nonpassive net after own prior passive loss: ${nonpassive}; prior passive loss absorbed by own income: ${prior_released}. Nonpassive amounts do not offset other passive losses; the allowance uses regular-tax MAGI."),
                    Message::new(if amt { "NOTE_SCORP_AMT_NONPASSIVE_NET" } else { "NOTE_SCORP_REGULAR_NONPASSIVE_NET" }, alloc::vec![
                        MessageArg::Number(id), MessageArg::usd_dollars(nonpassive), MessageArg::usd_dollars(prior_released),
                    ]),
                ));
                if source.loss_reported_with_basis() {
                    let source = source.qbi_projection();
                    source.loss_qbi.report_passive(
                        &source,
                        id,
                        crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                            source_slot_held_cents,
                        )
                        .expect("nonnegative ordinary source hold"),
                        r,
                    );
                }
            }
            #[cfg(test)]
            let tax_label = if amt { "AMT passive activity" } else { "Passive activity" };
            r.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] alloc::format!("{tax_label} {id}, net after passive limits: ${allowed_net}; remaining passive loss: ${}. Amount after activity limits; return-level limits apply separately.", suspended[i]),
                Message::new(if amt { "NOTE_AMT_PASSIVE_ACTIVITY_NET" } else { "NOTE_REGULAR_PASSIVE_ACTIVITY_NET" }, alloc::vec![
                    MessageArg::Number(id), MessageArg::usd_dollars(allowed_net), MessageArg::usd_dollars(suspended[i]),
                ]),
            ));
        }
        if let Some(business) = uniform_business {
            // Cooperative distributions above their box 8 qualified items are
            // not QBI, on a farm rental as on a farm (Form 1099-PATR box 8).
            let nonqualified = if farm_rental {
                match crate::farm_rental::nonqualified_cooperative(d, id) {
                    Ok(amount) => amount,
                    Err(issue) => {
                        let key = match &issue {
                            crate::Section179PreparedIssue::SourceBoundaryUnavailable(_, key)
                            | crate::Section179PreparedIssue::InvalidSource(_, key) => *key,
                            _ => "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE",
                        };
                        crate::section179_return::Issue::Prepared(issue).report(r);
                        source_issue.get_or_insert(crate::activity_risk::SourceIssue::new(
                            id,
                            crate::activity_risk::ProjectionIssue::Source(key),
                        ));
                        continue;
                    }
                }
            } else {
                0
            };
            let income = allowed_net - nonqualified;
            result.qbi += income;
            *result.business_losses_by_id.entry(id).or_default() += positive(-income);
            if income < 0 {
                result.qbi_loss_sources.push(id);
            }
            if let Some(mut activity) = business.regular_qbi {
                activity.income = income;
                activity.owner = d.income.iter().find(|e| e.id == id).map(|e| e.owner);
                result.qbi_source_ids.push(id);
                result.activities.push(activity);
            }
        }
        if let Some((parent, activity)) = estate_activity {
            let current = dollars(activity.net(amt));
            let source_prior = dollars(activity.prior(amt));
            let gross_income = positive(dollars(activity.income));
            // Box 9 deductions are separate from the fiduciary's signed
            // source income. Keep both Schedule E columns visible; the
            // net already used by the common owner fixes rounded footing.
            let current_deductions = gross_income - current;
            let allowed_loss = current_deductions + source_prior - suspended[i];
            if activity.kind == crate::EstateActivityKind::Business {
                result.estate_trust_business += allowed_net + nonpassive;
                *result.ordinary_business_by_id.entry(id).or_default() += allowed_net + nonpassive;
            }
            result.estate_activities.push(crate::estate_trust::EstateActivityResult {
                source: parent,
                activity: activity.id,
                current,
                prior: source_prior,
                passive: allowed_net,
                nonpassive,
                suspended: suspended[i],
                prior_released,
            });
            if !amt && inputs_complete {
                let material = activity.material();
                let nonpassive_loss =
                    if material { current_deductions + prior_released } else { 0 };
                result.estate_columns[0] += allowed_loss - nonpassive_loss;
                result.estate_columns[1] += if material { 0 } else { gross_income };
                result.estate_columns[2] += nonpassive_loss;
                result.estate_columns[3] += if material { gross_income } else { 0 };
                // Former-passive losses released against this activity's own
                // nonpassive profit are nonpassive under section 469(f).
                // Any further loss allowed by the common pool stays passive.
                for (column, amount) in [
                    (if material { "33f" } else { "33d" }, gross_income),
                    ("33e", nonpassive_loss),
                    ("33c", allowed_loss - nonpassive_loss),
                ] {
                    if amount != 0 {
                        form_line(
                            r,
                            FormLine::new(TaxForm::ScheduleE, column),
                            &alloc::format!(
                                "estate_trust_{parent}_activity_{}_{}",
                                activity.id,
                                column
                            ),
                            amount,
                            &["income.estate_trust_portfolio.activities"],
                            PASSIVE,
                            crate::Message::new(
                                "EXPLAIN_ESTATE_ACTIVITY_COLUMN",
                                alloc::vec![
                                    crate::MessageArg::Number(parent),
                                    crate::MessageArg::Text(activity.name.clone()),
                                    crate::MessageArg::Text(column.into()),
                                ],
                            ),
                        );
                    }
                }
            }
            line(
                r,
                &alloc::format!("{prefix}estate_trust_{parent}_activity_{}_suspended", activity.id),
                suspended[i],
                &["income.estate_trust_portfolio.activities"],
                PASSIVE,
                crate::Message::new(
                    "EXPLAIN_ESTATE_ACTIVITY_CARRY",
                    alloc::vec![
                        crate::MessageArg::Number(parent),
                        crate::MessageArg::Text(activity.name.clone())
                    ],
                ),
            );
        }
        if let Some(rental) = pass_through {
            // Schedule E line 28 of the K-1: column (g)/(h) for the passive
            // part, (i)/(k) for a professional's nonpassive part; the
            // suspended loss carries to 2027 (Form 8582 Parts IV/V).
            let amount = allowed_net + nonpassive;
            result.pass_through_rental_nonpassive += nonpassive;
            if rental.real_estate && rental.material {
                professional_pass_through_rentals += amount
                    + result
                        .k1_section1231
                        .iter()
                        .find(|property| property.id == id)
                        .and_then(|property| {
                            property.rentals.iter().find(|property| property.real_estate)
                        })
                        .map_or(0, |property| property.net);
            }
            let (activity, explanation) = match (rental.real_estate, amt) {
                (true, false) => (
                    "rental_real_estate",
                    [
                        msg!("EXPLAIN_K1_RENTAL_REAL_ESTATE_NET"),
                        msg!("EXPLAIN_K1_RENTAL_REAL_ESTATE_SUSPENDED"),
                    ],
                ),
                (true, true) => (
                    "rental_real_estate",
                    [
                        msg!("EXPLAIN_AMT_K1_RENTAL_REAL_ESTATE_NET"),
                        msg!("EXPLAIN_AMT_K1_RENTAL_REAL_ESTATE_SUSPENDED"),
                    ],
                ),
                (false, false) => (
                    "other_rental",
                    [
                        msg!("EXPLAIN_K1_OTHER_RENTAL_NET"),
                        msg!("EXPLAIN_K1_OTHER_RENTAL_SUSPENDED"),
                    ],
                ),
                (false, true) => (
                    "other_rental",
                    [
                        msg!("EXPLAIN_AMT_K1_OTHER_RENTAL_NET"),
                        msg!("EXPLAIN_AMT_K1_OTHER_RENTAL_SUSPENDED"),
                    ],
                ),
            };
            let [net_explanation, suspended_explanation] = explanation;
            let input = if rental.kind == "scorp" { "income.scorp" } else { "income.partnership" };
            line(
                r,
                &alloc::format!("{prefix}{}_{id}_{activity}_net", rental.kind),
                amount,
                &[input, "rental_magi"],
                PASSIVE,
                net_explanation,
            );
            line(
                r,
                &alloc::format!("{prefix}{}_{id}_{activity}_suspended", rental.kind),
                carried,
                &[input, "rental_magi"],
                PASSIVE,
                suspended_explanation,
            );
            if let Some(property) = result
                .k1_section1231
                .iter()
                .find(|v| v.id == id)
                .and_then(|v| v.rentals.iter().find(|v| v.real_estate == rental.real_estate))
            {
                line(
                    r,
                    &alloc::format!(
                        "{prefix}{}_{id}_{activity}_section1231_suspended",
                        rental.kind
                    ),
                    property.suspended,
                    &[input, "rental_magi"],
                    PASSIVE,
                    msg!("EXPLAIN_K1_RENTAL_SECTION1231_SUSPENDED"),
                );
            }
            // Section 199A: the allowed amount of a rental activity the
            // statement lists as a trade or business (i8995; the
            // convention of Schedule E and Form 4835 rentals).
            if let Some(qbi) = rental.qbi {
                result.qbi += amount;
                *result.business_losses_by_id.entry(id).or_default() += positive(-amount);
                if let Some(property) =
                    result.k1_section1231.iter_mut().find(|v| v.id == id).and_then(|v| {
                        v.rentals.iter_mut().find(|v| v.real_estate == rental.real_estate)
                    })
                {
                    property.qbi_index = Some(result.activities.len());
                }
                if amount < 0 {
                    result.qbi_loss_sources.push(id);
                }
                result.qbi_source_ids.push(id);
                let source_income_cents =
                    exact_allowed_net_cents.checked_add(exact_nonpassive_cents);
                if source_income_cents.is_none() {
                    crate::issue_localized!(
                        r,
                        &crate::IssueContext::income_id(id),
                        "K-1 rental source endpoint",
                        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                        "Review the document amounts; they must be within the supported input range."
                    );
                    result.source_issue = Some(crate::activity_risk::SourceIssue::new(
                        id,
                        crate::activity_risk::ProjectionIssue::Source(
                            "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                        ),
                    ));
                    result.inputs_complete = false;
                }
                result.activities.push(crate::business_rules::QbiActivity {
                    source_wages_cents: Some(qbi.wages),
                    source_property_cents: Some(qbi.property),
                    source_income_cents,
                    owner: d.income.iter().find(|e| e.id == id).map(|e| e.owner),
                    income: amount,
                    wages: dollars(qbi.wages),
                    property: dollars(qbi.property),
                    active: rental.material,
                    service: Some(false),
                    group: None,
                });
            }
        } else if farm_rental {
            result.farm_rental += allowed_net;
            if !amt && inputs_complete {
                crate::farm_rental::report_pool(
                    d,
                    r,
                    id,
                    net,
                    losses[i] - suspended[i],
                    suspended[i],
                    d.any_professional(),
                );
            }
            line(
                r,
                &alloc::format!("{prefix}farm_rental_{id}_suspended_loss"),
                suspended[i],
                &["income.farm_rental", "rental_magi"],
                PASSIVE,
                if amt {
                    msg!("EXPLAIN_0FARM_RENTAL_N_SUSPENDED_LOSS")
                } else {
                    msg!("EXPLAIN_FARM_RENTAL_N_SUSPENDED_LOSS")
                },
            );
        } else if !personal
            && scorp.is_none()
            && !partner
            && source_group.is_none()
            && !estate_trust
        {
            result.part_one += allowed_net;
            // Form 8582 Part VIII column (c): the loss allowed is the net loss
            // plus the prior-year unallowed loss, less what stays suspended.
            // The part of it that the activity's own profit absorbs is on
            // line 22 too ("will include the loss allowed to the extent of the
            // net profit", 2025 i8582, How To Report Allowed Losses). A sale
            // gain is Form 4797 income, not part of the Schedule E net.
            let current = if nonpassive != 0 || prior_released > 0 {
                nonpassive + prior_released
            } else {
                net - disposition_gain + prior
            };
            let allowed_loss = positive(-current) + prior - ordinary_suspended;
            if !amt && (net < 0 || prior > 0 || (disposition_gain != 0 && allowed_net < 0)) {
                form_line(
                    r,
                    FormLine::new(TaxForm::ScheduleE, "22"),
                    &alloc::format!("rental_{id}_line22_allowed_loss"),
                    -allowed_loss,
                    &["income.rental", "rental_magi"],
                    PASSIVE,
                    msg!("EXPLAIN_RENTAL_N_LINE22_ALLOWED_LOSS_2"),
                );
            }
            line(
                r,
                &alloc::format!("{prefix}rental_{id}_suspended_loss"),
                suspended[i],
                &["income.rental", "rental_magi"],
                PASSIVE,
                msg!("EXPLAIN_0RENTAL_N_SUSPENDED_LOSS_2"),
            );
        }
    }
    if !amt {
        crate::asset_sale::report_rental_partial_limits(d, &result, r);
    }
    for (id, business) in home_activities {
        *result.business_losses_by_id.entry(id).or_default() += positive(-business.income);
        if business.income < 0 {
            result.qbi_loss_sources.push(id);
        }
        result.qbi += business.income;
        if let Some(activity) = business.regular_qbi {
            result.qbi_source_ids.push(id);
            result.activities.push(activity);
        }
    }
    // Schedule E totals sum its separately reported activity rows and columns.
    // Exact source stages remain in the selected owner packets and ledgers.
    result.income = result.income_by_id.values().sum();
    if !amt && d.any_professional() {
        let mut total = 0;
        for entry in &allocated_entries {
            // A short-stay business variant is not rental real estate and a
            // section 280A home year is outside section 469 (section 469(j)(10)).
            if let FederalIncome::Rental { data } = &entry.income
                && data.participation.professional()
                && data.property_type.business_participation().is_none()
                && !used_as_home(data)
                && d.professional(crate::community::professional_owner(
                    d.setup.as_ref(),
                    entry.owner,
                ))
                .is_some()
            {
                let current = r
                    .lines
                    .iter()
                    .find(|l| l.id == alloc::format!("rental_{}_line21_income", entry.id))
                    .map_or(0, |l| crate::kernel::ratio::trunc_i64(l.amount, 100));
                let allowed = r
                    .lines
                    .iter()
                    .find(|l| l.id == alloc::format!("rental_{}_line22_allowed_loss", entry.id))
                    .map_or(0, |l| crate::kernel::ratio::trunc_i64(l.amount, 100));
                total += positive(current) + allowed;
                total += rental_source!(crate::asset_sale::rental_source(d, entry.id, false))
                    .map_or(0, |sale| sale.gain);
            }
        }
        // K-1 professional rental amounts are already source-tagged by their
        // owning module; this consumer does not change its pool mechanics.
        total += professional_pass_through_rentals;
        form_line(
            r,
            FormLine::new(TaxForm::ScheduleE, "43"),
            "schedule_e_line43_professional_rentals",
            total,
            &["income.rental", "income.partnership", "income.scorp"],
            PASSIVE,
            msg!("EXPLAIN_SCHEDULE_E_PROFESSIONAL_RENTALS"),
        );
    }
    if result.inputs_complete {
        debug_assert_eq!(result.income_by_id.values().sum::<i64>(), result.income);
        debug_assert_eq!(
            result.ordinary_business_by_id.values().sum::<i64>(),
            result.ordinary_pass_through_business()
        );
    }
    line(
        r,
        &alloc::format!("{prefix}rental_income"),
        result.income,
        &["income.rental", "rental_magi"],
        PASSIVE,
        msg!("EXPLAIN_0RENTAL_INCOME"),
    );
    line(
        r,
        &alloc::format!("{prefix}rental_suspended_loss"),
        result.passive_loss,
        &["income.rental", "rental_magi"],
        PASSIVE,
        msg!("EXPLAIN_0RENTAL_SUSPENDED_LOSS"),
    );
    if !amt && result.inputs_complete {
        for entry in &allocated_entries {
            if let FederalIncome::Partnership { data } = &entry.income
                && let Some(partner) = result.partners.iter().find(|v| v.id == entry.id)
            {
                let counterpart = d.setup.as_ref().is_some_and(crate::community::applies)
                    && entry.owner == Owner::Spouse;
                match crate::community::partner_se(
                    d,
                    entry,
                    data,
                    partner.suspended,
                    crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                        result.section179_allowed_cents(d, entry.id),
                    )
                    .expect("selected section179 source allowance is nonnegative"),
                ) {
                    Ok(mut se) => {
                        // General partners subtract partner-level business expenses
                        // from the K-1 14A source; a limited partner's excluded
                        // distributive share never becomes SE income.
                        if !data.limited_for_se {
                            let deduction = care.source(entry.id);
                            se.earnings =
                                crate::kernel::carry::native_balance(se.earnings, deduction);
                            se.earnings_cents = crate::kernel::carry::native_balance(
                                se.earnings_cents,
                                deduction * 100,
                            );
                        }
                        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
                            "Partnership document {}, SE source including taxable guaranteed payments after regular activity limits: ${}; SE-character suspended losses: basis ${}, at-risk ${}, passive ${}. Intermediate attribution before Schedule SE and other return-level coordination, not additional Schedule E income.",
                            entry.id, se.earnings, se.basis_suspended, se.risk_suspended, se.passive_suspended,
                        ), Message::new("NOTE_PARTNERSHIP_SE_SOURCE", alloc::vec![
                            MessageArg::Number(entry.id), MessageArg::usd_dollars(se.earnings),
                            MessageArg::usd_dollars(se.basis_suspended), MessageArg::usd_dollars(se.risk_suspended),
                            MessageArg::usd_dollars(se.passive_suspended),
                        ])));
                        // Form 8960 line 4b excludes ordinary nonpassive income
                        // and passive SE income once. Trading remains investment
                        // income regardless of participation; remove only its SE
                        // items, if present, using the independent source record.
                        let adjustment = match data.trade_kind {
                            crate::ScorpTradeKind::Ordinary => {
                                -partner.nonpassive
                                    - if counterpart {
                                        0
                                    } else {
                                        crate::community::income_amount(
                                            d,
                                            entry.id,
                                            se.passive_earnings,
                                        )
                                    }
                                    + if data.limited_for_se {
                                        // Capital guarantees remain NII even though
                                        // section 469 treats them as nonpassive.
                                        dollars(data.guaranteed_capital)
                                    } else {
                                        0
                                    }
                            }
                            crate::ScorpTradeKind::FinancialTrading => {
                                if counterpart {
                                    0
                                } else {
                                    -crate::community::income_amount(d, entry.id, se.earnings)
                                }
                            }
                        };
                        result.partner_niit_adjustment += adjustment;
                        *result.nii_adjustment_by_id.entry(entry.id).or_default() += adjustment;
                        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
                            "Partnership document {}, investment income after activity limits: ${}; Form 8960 adjustment: ${adjustment}. Additional return-level limits remain separate.",
                            entry.id, partner.net + adjustment,
                        ), Message::new("NOTE_PARTNERSHIP_INVESTMENT_INCOME", alloc::vec![
                            MessageArg::Number(entry.id), MessageArg::usd_dollars(partner.net + adjustment), MessageArg::usd_dollars(adjustment),
                        ])));
                        if counterpart {
                            result.counterpart_partner_se.push((entry.id, se));
                        } else {
                            result.partner_se.push((entry.id, se));
                        }
                    }
                    Err((key, _message)) => {
                        #[cfg(test)]
                        let message = _message;

                        crate::issue_localized!(
                            r,
                            &crate::interview::income_issue_context(entry),
                            "Partnership self-employment loss attribution",
                            if counterpart
                                && key
                                    == "ISSUE_PARTNERSHIP_RECONCILE_BOX_14_LESS_GUARANTEED_PAYMENTS_F10E8824"
                            {
                                "ISSUE_COMMUNITY_COUNTERPART_PARTNER_SE_UNRESOLVED"
                            } else {
                                key
                            },
                            if counterpart
                                && key
                                    == "ISSUE_PARTNERSHIP_RECONCILE_BOX_14_LESS_GUARANTEED_PAYMENTS_F10E8824"
                            {
                                "Your spouse's partnership self-employment loss is held back by activity limits. This App cannot reconcile that partner's Schedule SE or the required Form 8958 allocation; this separate return is not calculated."
                            } else {
                                message
                            },
                        );
                        result.inputs_complete = false;
                    }
                }
            }
        }
    }
    // Real stage operands remain available for diagnosis, but an incomplete
    // native context cannot supply a final annual source amount. Keep None;
    // the existing owner issues still govern the return's unsupported state.
    if result.inputs_complete && r.issues.is_empty() {
        for (id, source) in &mut result.operating_sources {
            source.filed_ordinary_dollars = result.income_by_id.get(id).copied();
        }
    }
    if result.inputs_complete {
        debug_assert!(post_pal.sources.iter().all(|source| {
            source.characters.iter().all(|amount| {
                amount.suspended_cents >= 0 && amount.suspended_cents <= positive(-amount.net_cents)
            }) && source.characters.iter().map(|amount| amount.net_cents).sum::<i64>()
                == source.net_cents
                && source.characters.iter().map(|amount| amount.suspended_cents).sum::<i64>()
                    == source.suspended_cents
                && source.characters.iter().map(|amount| amount.allowed_net_cents).sum::<i64>()
                    == source.allowed_net_cents
        }));
        result.post_pal = Some(post_pal);
    }
    if let Some(allowed) = result.personal_expenses_cents() {
        // Preserve independent annual categories: round the combined regular
        // current costs and combined final allowed costs, then subtract. A
        // rounded held adjustment subtracted from raw cents fails across zero.
        result.personal_expense_adjustment = dollars(
            result.personal_inputs.values().map(|input| input.regular_expenses_cents).sum(),
        ) - dollars(allowed.values().sum());
        // Show a complete expense reconciliation even when all history is used
        // or net income is recharacterized and never enters the loss allocation.
        for e in &allocated_entries {
            if let FederalIncome::NonbusinessProperty {
                kind: NonbusinessPropertyKind::PersonalRental { .. },
                ..
            } = &e.income
            {
                let input = &result.personal_inputs[&e.id];
                let prior = dollars(input.prior_cents);
                let closing_cents = result
                    .post_pal
                    .as_ref()
                    .expect("complete personal costs")
                    .sources
                    .iter()
                    .find(|source| {
                        source.income == e.id
                            && source.component == PostPalComponent::PersonalProperty
                    })
                    .expect("same-invocation personal source")
                    .suspended_cents;
                let closing = match personal_display_balances.get(&e.id) {
                    Some(balance) => *balance,
                    // Positive and recharacterized sources have a complete
                    // zero hold from the native packet, outside loss rows.
                    None => dollars(closing_cents),
                };
                emit_lines(
                    r,
                    &mut (),
                    [
                        (
                            "prior_suspended_expenses",
                            prior,
                            msg!("EXPLAIN_0PERSONAL_RENTAL_N_N_PRIOR_SUSPENDED_EXPENSES"),
                        ),
                        (
                            "allowed_expenses",
                            dollars(allowed[&e.id]),
                            msg!("EXPLAIN_0PERSONAL_RENTAL_N_N_ALLOWED_EXPENSES"),
                        ),
                        (
                            "suspended_loss",
                            closing,
                            msg!("EXPLAIN_0PERSONAL_RENTAL_N_N_SUSPENDED_LOSS"),
                        ),
                    ],
                    |(suffix, amount, explanation), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!("{prefix}personal_rental_{}_{suffix}", e.id),
                            LineValue::MoneyDollars(amount),
                            &["income.nonbusiness_property", "rental_magi"],
                            PASSIVE,
                            explanation,
                        ))
                    },
                );
            }
        }
    }
    if inputs_complete {
        for entry in &allocated_entries {
            let (kind, source_path) = match entry.income {
                FederalIncome::PartnershipActivities { .. } => {
                    ("partnership", "income.partnership.activities")
                }
                FederalIncome::ScorpActivities { .. } => ("scorp", "income.scorp.activities"),
                _ => continue,
            };
            if let Some(value) = result.k1_amounts.iter().find(|v| v.id == entry.id) {
                line(
                    r,
                    &alloc::format!("{prefix}{kind}_{}_schedule_e", entry.id),
                    value.net,
                    &[source_path],
                    PASSIVE,
                    if kind == "partnership" {
                        msg!("EXPLAIN_0PARTNERSHIP_N_ACTIVITY_NET")
                    } else {
                        msg!("EXPLAIN_SCORP_ACTIVITY_NET")
                    },
                );
            }
        }
    }
    // Settle the existing property rows once from this invocation's actual
    // basis/risk ledger and final PAL output, before any Form4797 lookback.
    if result.inputs_complete {
        for entry in &allocated_entries {
            let settled = crate::k1_sales::facts(entry, amt,
                result.k1_ledgers.iter().find(|(id, _)| *id == entry.id).map(|(_, ledger)| ledger))
                .and_then(|facts| facts.map(|facts| {
                    let sales = facts.settle_property(&result, entry.id)?;
                    let range = (crate::k1_activity::ordinary::RANGE_KEY,
                        crate::fixture_english!("Review the document amounts; they must be within the supported input range."));
                    let reported_net = result.k1_section1231.iter().find(|value| value.id == entry.id)
                        .map(|value| value.reported_net(&sales).ok_or(range)).transpose()?;
                    Ok((sales, reported_net))
                }).transpose());
            match settled {
                Ok(Some((sales, reported_net))) => {
                    if let Some(value) =
                        result.k1_section1231.iter_mut().find(|value| value.id == entry.id)
                    {
                        value.net = reported_net.expect("selected property row");
                        value.section179_sales_cents = sales;
                    }
                }
                Ok(None) => {}
                Err((key, _message)) => {
                    crate::issue_localized!(
                        r,
                        &crate::IssueContext::income_id(entry.id),
                        "K-1 section 179 sale",
                        key,
                        _message
                    );
                    result.inputs_complete = false;
                    source_issue = Some(crate::activity_risk::SourceIssue::new(
                        entry.id,
                        crate::activity_risk::ProjectionIssue::Source(key),
                    ));
                }
            }
        }
    }
    result.source_issue = source_issue;
    result.inputs_complete &= result.source_issue.is_none();
    result
}

// The prior unallowed loss of an activity with active participation this
// year that the special allowance does not admit.
fn ineligible_prior(allowance: bool, history: &crate::RentalHistory, amt: bool) -> i64 {
    if allowance && !history.active_in_loss_year { dollars(history.balance(amt)) } else { 0 }
}

// Form 2555 lines 31 and 38: the qualifying days in 2026 of a bona fide
// residence (its dates clipped to the year) or of the 12-month physical
// presence period, which ends starts_on + 364 days later.
pub(crate) fn exclusion_days(
    q: &ForeignQualification,
) -> Result<i64, (&'static str, &'static str)> {
    match q {
        ForeignQualification::PriorYearOnly => Ok(0),
        // The 330-day test and the residence confirmation are answered before
        // the rules run (unknowns::foreign_claims leaves a failed or unconfirmed
        // claim out with its notice), so only an invalid stored period is
        // reported here.
        ForeignQualification::Residence { began, ended, .. } => {
            if !valid_date(*began) || ended.is_some_and(|e| !valid_date(e) || e <= *began) {
                return Err((
                    "ISSUE_FOREIGN_RESIDENCE_PERIOD_INVALID",
                    crate::fixture_english!(
                        "Enter valid dates for the bona fide residence, ending after it began."
                    ),
                ));
            }
            let (start, last) = period_ordinals(q);
            Ok(i64::from(max(0, min(last + 1, ordinal(20270101)) - start)))
        }
        ForeignQualification::Presence { starts_on, foreign_days } => {
            if !valid_birth(*starts_on) || *starts_on < 20250102 {
                return Err((
                    "ISSUE_FOREIGN_PRESENCE_PERIOD_INVALID",
                    crate::fixture_english!(
                        "Choose a valid 12-month presence period overlapping 2026."
                    ),
                ));
            }
            let start = ordinal(*starts_on);
            // Every period a 2026 return can name has 365 days: starts run
            // from 2025-01-02 to 2026-12-31 and none contains a February 29.
            let end = start + 365;
            if *foreign_days > end - start {
                return Err((
                    "ISSUE_FOREIGN_PRESENCE_PERIOD_INVALID",
                    crate::fixture_english!(
                        "Choose a valid 12-month presence period overlapping 2026."
                    ),
                ));
            }
            Ok(i64::from(max(0, min(end, ordinal(20270101)) - max(start, ordinal(20260101)))))
        }
    }
}

// The first and last day (ordinals) of a qualifying period, the first day
// clipped up to January 1, 2026: the bounds a housing stay must stay inside.
pub(crate) fn period_ordinals(q: &ForeignQualification) -> (i32, i32) {
    let year_start = ordinal(20260101);
    match q {
        ForeignQualification::PriorYearOnly => (ordinal(20260101), ordinal(20260101) - 1),
        ForeignQualification::Residence { began, ended, .. } => {
            (max(ordinal(*began), year_start), ended.map_or(ordinal(20261231), ordinal))
        }
        ForeignQualification::Presence { starts_on, .. } => {
            let start = ordinal(*starts_on);
            (max(start, year_start), start + 364)
        }
    }
}

#[derive(Default)]
pub(crate) struct ForeignResult {
    // Form 1040 line 1h: pay from a foreign employer with no W-2.
    pub wages: [i64; 2],
    // Form 2555 Part IV line 19: W-2 box 1 pay for work abroad plus the
    // foreign-employer pay. A W-2's box 1 is already in the wages total.
    pub earned_wages: [i64; 2],
    pub excluded: [i64; 2],
    pub gross_excluded: [i64; 2],
    pub prior_excluded: [i64; 2],
    pub housing_deduction: [i64; 2],
    pub housing_exclusion: [i64; 2],
    pub elected: [bool; 2],
}

impl ForeignResult {
    pub(crate) fn election(&self) -> bool {
        self.elected.iter().any(|elected| *elected)
    }
}

// The person's Form 2555 record, index 0 the taxpayer and 1 the spouse.
pub(crate) fn foreign_claim(d: &FederalDraft, owner: usize) -> Option<&crate::ForeignEarnedClaim> {
    d.screening
        .as_ref()?
        .foreign_earned
        .iter()
        .find(|claim| usize::from(claim.owner == Owner::Spouse) == owner)
}

pub(crate) fn foreign(d: &FederalDraft, r: &mut FederalResult) -> ForeignResult {
    let mut result = ForeignResult::default();
    let mut abroad = [0; 2];
    for e in &d.income {
        let i = usize::from(e.owner == Owner::Spouse);
        match &e.income {
            FederalIncome::ForeignEmployment { data: v } => result.wages[i] += v.wages,
            FederalIncome::Wage { data: v } => abroad[i] += v.abroad,
            _ => {}
        }
    }
    let claims = [foreign_claim(d, 0), foreign_claim(d, 1)];
    // W-2 box 1 pay for work abroad is a Form 2555 fact: it is read only for a
    // person who has a Form 2555 record (F3, 2026-10-09).
    for (((pay, earned), abroad), claim) in
        result.wages.iter_mut().zip(&mut result.earned_wages).zip(abroad).zip(claims)
    {
        let gross = *pay + if claim.is_some() { abroad } else { 0 };
        *earned = claim.map_or_else(
            || dollars(gross),
            |c| {
                if matches!(c.qualification, ForeignQualification::PriorYearOnly) {
                    0
                } else {
                    c.employee_pay(gross).0
                }
            },
        );
        *pay = dollars(*pay);
    }
    crate::foreign_housing::check_households(
        claims.map(|claim| claim.map(|c| &c.housing)),
        d.setup.as_ref().unwrap().filing_status() == FederalStatus::Separate,
        r,
    );
    let mut housing = [None, None];
    let mut housing_excluded = [0; 2];
    for i in 0..2 {
        let Some(claim) = claims[i] else { continue };
        let barred = claim.revocation.barred();
        if claim.exclusion && barred[0] {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_FOREIGN_ELECTION_BARRED"),
                crate::IssueContext::owner(crate::TaxSection::WorkAbroad, claim.owner),
            );
        }
        if matches!(claim.housing, crate::ForeignHousing::Claim { .. }) && barred[1] {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_FOREIGN_HOUSING_ELECTION_BARRED"),
                crate::IssueContext::owner(crate::TaxSection::WorkAbroad, claim.owner),
            );
        }
        let context = crate::IssueContext::owner(crate::TaxSection::WorkAbroad, claim.owner);
        let business_funding = d.income.iter().any(|entry| usize::from(entry.owner == Owner::Spouse) == i && matches!(&entry.income, FederalIncome::Business { data } if matches!(data.source, crate::BusinessSource::ForeignServices { .. })));
        result.elected[i] = (claim.exclusion && !barred[0])
            || (matches!(claim.housing, crate::ForeignHousing::Claim { .. })
                && (!barred[1] || business_funding));
        let wages = result.earned_wages[i];
        // A self-employed person's housing is figured with the business
        // receipts in foreign_business::exclude.
        if !business_funding && matches!(claim.housing, crate::ForeignHousing::Claim { .. }) {
            housing[i] = Some(&claim.housing);
            housing_excluded[i] =
                claim.housing.calculate(&claim.qualification, wages, wages, i == 1, r).exclusion;
            if barred[1] {
                housing_excluded[i] = 0;
            }
        }
        let receipts: i64 = d
            .income
            .iter()
            .filter(|entry| usize::from(entry.owner == Owner::Spouse) == i)
            .filter_map(|entry| match &entry.income {
                FederalIncome::Business { data }
                    if matches!(data.source, crate::BusinessSource::ForeignServices { .. }) =>
                {
                    Some(data.receipts())
                }
                _ => None,
            })
            .sum();
        // Part IV line 26: foreign earned income.
        line(
            r,
            &alloc::format!("foreign_earned_{}_income", if i == 1 { "spouse" } else { "taxpayer" }),
            wages + dollars(receipts),
            &["screening.foreign_earned", "income.foreign_employment"],
            FOREIGN,
            msg!("EXPLAIN_FOREIGN_EARNED_N_N_INCOME"),
        );
        if claim.exclusion && !barred[0] {
            let gross: i64 = d
                .income
                .iter()
                .filter(|e| e.owner == claim.owner)
                .filter_map(|e| match &e.income {
                    FederalIncome::Wage { data } => Some(data.abroad),
                    FederalIncome::ForeignEmployment { data } => Some(data.wages),
                    _ => None,
                })
                .sum();
            result.prior_excluded[i] = claim.employee_pay(gross).1;
            if claim.prior_year_history.is_some() {
                line(
                    r,
                    &alloc::format!(
                        "foreign_prior_year_{}_exclusion",
                        if i == 1 { "spouse" } else { "taxpayer" }
                    ),
                    result.prior_excluded[i],
                    &[
                        "screening.foreign_earned.prior_year_pay",
                        "screening.foreign_earned.prior_year_history",
                        "screening.foreign_earned.multi_year_pay",
                    ],
                    FOREIGN,
                    msg!("EXPLAIN_FOREIGN_PRIOR_YEAR_EXCLUSION"),
                );
            }
            match exclusion_days(&claim.qualification) {
                Ok(days) => {
                    result.excluded[i] = phase::cap(
                        positive(wages - housing_excluded[i]),
                        [fraction(
                            crate::year_constants::FOREIGN_EARNED_INCOME_EXCLUSION_DOLLARS,
                            days,
                            365,
                        )],
                    );
                    let person = if i == 1 { "spouse" } else { "taxpayer" };
                    line(
                        r,
                        &alloc::format!("foreign_earned_{person}_days"),
                        days,
                        &["screening.foreign_earned"],
                        FOREIGN,
                        msg!("EXPLAIN_FOREIGN_EARNED_N_N_DAYS"),
                    );
                }
                Err((key, _message)) => {
                    #[cfg(test)]
                    let message = _message;
                    crate::issue_localized!(r, &context, "Foreign exclusion", key, message,)
                }
            }
        }
    }
    line(
        r,
        "foreign_wages",
        result.wages.iter().sum(),
        &["income.foreign_employment"],
        FOREIGN,
        msg!("EXPLAIN_FOREIGN_WAGES"),
    );
    result.gross_excluded = result.excluded;
    result.housing_exclusion = housing_excluded;
    for (i, amount) in housing_excluded.iter().enumerate() {
        result.excluded[i] += amount;
        if let Some(crate::ForeignHousing::Claim { prior: Some(prior), .. }) = housing[i] {
            // Section 1.911-4(e)(2): prior carry needs remaining foreign income,
            // not a current self-employment housing amount.
            result.housing_deduction[i] = crate::foreign_housing::deduction(
                0,
                result.earned_wages[i] - result.excluded[i],
                Some(prior),
                if i == 1 { "spouse" } else { "taxpayer" },
                r,
            );
        }
    }
    result
}

#[cfg(test)]
pub(crate) const FTC_NO_SOURCE: &str = "Form 1116 Part II: enter at least one foreign income source, a carryover history or a foreign loss account.";

pub(crate) use crate::form_1116::CreditBasis;

// Retained payee-document taxes in cents on the centrally supported working
// copy: Form1099-INT/DIV components and qualified estate/trust statements.
// The common consumer serves the statement election and the deduction.
pub(crate) fn payee_document_taxes(d: &FederalDraft) -> i64 {
    payee_document_tax_sources(d).iter().map(|(_, _, tax)| tax).sum()
}

// One native extraction owns both the joint total and actual payee sources.
// Values retain the centrally supported statement's actual cents.
fn payee_document_tax_sources(d: &FederalDraft) -> alloc::vec::Vec<(i64, Owner, i64)> {
    crate::community::entries(d)
        .iter()
        .map(|entry| {
            let tax = match &entry.income {
                FederalIncome::Interest { .. } | FederalIncome::Dividend { .. } => {
                    crate::investment_statements::components(entry).unwrap().foreign_tax
                }
                FederalIncome::EstateTrustPortfolio { data } => data
                    .foreign_statement
                    .as_ref()
                    .filter(|s| {
                        s.treatment == crate::EstateForeignTreatment::QualifiedPassivePayeeStatement
                    })
                    .map_or(0, |s| s.tax),
                _ => 0,
            };
            (entry.id, entry.owner, tax)
        })
        .collect()
}

// The section 904(j) election additionally uses its existing eligible K-3
// projection, alongside the statements typed on More credits.
pub(crate) fn payee_statement_taxes(d: &FederalDraft) -> i64 {
    let k3: i64 =
        crate::schedule_k3::credit(&crate::schedule_k3::attached(d), false, &[]).tax.iter().sum();
    payee_document_taxes(d) + k3
}

// Schedule A line 6, before rounding or the itemized-deduction choice. The
// independent K-3 traversal includes every category, not the 904(j) filter.
#[derive(Default)]
pub(crate) struct ForeignTaxDeduction {
    pub tax: i64,
    pub investment: i64,
    // Actual source cents for the same native investment-tax deduction.
    // Aggregate tax rows without a payee remain unavailable, never zero.
    pub investment_owners: Option<[i64; 2]>,
    pub investment_capacities: Option<crate::kernel::attribution::ExpenseSourceCents>,
    pub portfolio: i64,
    pub unallocated_tax: i64,
    pub unallocated_k3: alloc::vec::Vec<i64>,
    pub missing_k1: alloc::vec::Vec<i64>,
    pub accrued_sanctioned: bool,
    pub sanctioned: alloc::collections::BTreeMap<alloc::string::String, i64>,
    earned: [i64; 2],
    earned_countries: [alloc::vec::Vec<(alloc::string::String, i64)>; 2],
}

// Source-only tax projection shared with early royalty interest routing.
// No AGI-dependent tax, exclusion or basis result is inferred here.
pub(crate) fn direct_foreign_tax_deduction(d: &FederalDraft) -> ForeignTaxDeduction {
    let claim = &d.credits.as_ref().unwrap().foreign_tax;
    let chosen = matches!(claim, ForeignTaxClaim::Deduction { .. });
    let sanctioned = matches!(claim, ForeignTaxClaim::Ordinary { .. });
    let mut out = ForeignTaxDeduction {
        investment_owners: Some([0; 2]),
        investment_capacities: Some(crate::kernel::attribution::ExpenseSourceCents::known([0; 2])),
        ..Default::default()
    };
    if !chosen && !sanctioned {
        return out;
    }
    if chosen {
        for (_, owner, tax) in payee_document_tax_sources(d) {
            out.investment += tax;
            out.investment_owners.as_mut().unwrap()[usize::from(owner == Owner::Spouse)] += tax;
            out.investment_capacities.as_mut().unwrap().add(
                tax,
                Some(if owner == Owner::Spouse {
                    crate::DeductionOwners::Spouse
                } else {
                    crate::DeductionOwners::Taxpayer
                }),
            );
        }
        out.tax = out.investment;
        out.portfolio = out.investment;
        if let ForeignTaxClaim::Deduction { taxes } = claim {
            for row in taxes {
                match row.income {
                    crate::ForeignTaxIncome::Earned { owner } => {
                        out.earned[usize::from(owner == Owner::Spouse)] += row.tax
                    }
                    crate::ForeignTaxIncome::Investment => {
                        out.investment_capacities.as_mut().unwrap().add(row.tax, row.owners);
                        match (row.owners, &mut out.investment_owners) {
                            (Some(crate::DeductionOwners::Taxpayer), Some(owners)) => {
                                owners[0] += row.tax
                            }
                            (Some(crate::DeductionOwners::Spouse), Some(owners)) => {
                                owners[1] += row.tax
                            }
                            _ => out.investment_owners = None,
                        }
                        out.tax += row.tax;
                        out.investment += row.tax;
                        out.portfolio += row.tax;
                    }
                    crate::ForeignTaxIncome::Passive => {
                        out.investment_capacities.as_mut().unwrap().add(row.tax, row.owners);
                        match (row.owners, &mut out.investment_owners) {
                            (Some(crate::DeductionOwners::Taxpayer), Some(owners)) => {
                                owners[0] += row.tax
                            }
                            (Some(crate::DeductionOwners::Spouse), Some(owners)) => {
                                owners[1] += row.tax
                            }
                            _ => out.investment_owners = None,
                        }
                        out.tax += row.tax;
                        out.investment += row.tax;
                    }
                    crate::ForeignTaxIncome::Other => out.tax += row.tax,
                }
            }
        }
    } else if let ForeignTaxClaim::Ordinary { basis, sources, .. } = claim {
        // Section 901(j)(3) expressly removes the section 275 bar. Credit
        // source rows already hold the tax, so never add the 1099 boxes again.
        for row in sources
            .iter()
            .filter(|row| crate::year_constants::is_section_901j_country(&row.country))
        {
            // An accrual credit election does not establish cash payment for
            // the separate section 901(j)(3) deduction. Entity K-3 taxes below
            // retain the entity's paid/accrued treatment under 1.702-1(a)(6).
            if *basis == crate::ForeignTaxBasis::Accrued {
                out.accrued_sanctioned |= row.tax > 0;
                if matches!(
                    row.category,
                    crate::ForeignTaxCategory::Passive { .. }
                        | crate::ForeignTaxCategory::PriorPassiveTax { .. }
                ) {
                    out.investment_owners = None;
                    out.unallocated_tax += row.tax;
                    out.investment_capacities = None;
                }
                continue;
            }
            match row.category {
                crate::ForeignTaxCategory::Employment
                | crate::ForeignTaxCategory::Business { .. } => {
                    let owner = usize::from(row.owner == Owner::Spouse);
                    out.earned[owner] += row.tax;
                    out.earned_countries[owner].push((row.country.to_ascii_uppercase(), row.tax));
                }
                crate::ForeignTaxCategory::Passive { .. }
                | crate::ForeignTaxCategory::PriorPassiveTax { .. } => {
                    out.tax += row.tax;
                    out.investment += row.tax;
                    out.portfolio += row.tax;
                    if let Some(owners) = &mut out.investment_owners {
                        owners[usize::from(row.owner == Owner::Spouse)] += row.tax;
                    }
                    if let Some(capacities) = &mut out.investment_capacities {
                        capacities.add(
                            row.tax,
                            Some(if row.owner == Owner::Spouse {
                                crate::DeductionOwners::Spouse
                            } else {
                                crate::DeductionOwners::Taxpayer
                            }),
                        );
                    }
                    *out.sanctioned.entry(row.country.to_ascii_uppercase()).or_default() += row.tax;
                }
                crate::ForeignTaxCategory::PriorEmployeeWageTax { .. } => {
                    out.tax += row.tax;
                    *out.sanctioned.entry(row.country.to_ascii_uppercase()).or_default() += row.tax;
                }
            }
        }
    }
    out
}

pub(crate) fn foreign_tax_deduction(
    d: &FederalDraft,
    foreign: &ForeignResult,
    businesses: &[crate::foreign_business::BusinessExclusion; 2],
    limits: &[crate::schedule_k3::K3Limits],
    pool: &RentalResult,
) -> ForeignTaxDeduction {
    let claim = &d.credits.as_ref().unwrap().foreign_tax;
    let chosen = matches!(claim, ForeignTaxClaim::Deduction { .. });
    let sanctioned = matches!(claim, ForeignTaxClaim::Ordinary { .. });
    let mut out = direct_foreign_tax_deduction(d);
    if !chosen && !sanctioned {
        return out;
    }
    for (owner, business) in businesses.iter().enumerate() {
        let income = foreign.earned_wages[owner] + positive(business.earned_net)
            - foreign.housing_deduction[owner]
            - business.housing_deduction;
        let retained = out.earned[owner]
            - crate::form_1116::excluded_earned_tax(
                out.earned[owner],
                foreign.excluded[owner],
                income,
            );
        out.tax += retained;
        let weights: alloc::vec::Vec<i64> =
            out.earned_countries[owner].iter().map(|(_, tax)| *tax).collect();
        for ((country, _), share) in out.earned_countries[owner]
            .iter()
            .zip(crate::kernel::ratio::allocate_prefix(retained, &weights))
        {
            *out.sanctioned.entry(country.clone()).or_default() += share;
        }
    }
    let mut position = 0;
    for entry in &crate::community::entries(d) {
        let k3 = match &entry.income {
            FederalIncome::Partnership { data } => data.schedule_k3.as_ref(),
            FederalIncome::Scorp { data } => data.schedule_k3.as_ref(),
            _ => None,
        };
        let allowed =
            pool.k1_ledgers.iter().find(|(id, _)| *id == entry.id).map_or(0, |(_, ledger)| {
                ledger.basis_allowed[crate::k1_loss::K1LossClass::ForeignTaxes].get()
            });
        let Some(k3) = k3 else {
            if chosen && allowed > 0 {
                out.investment_owners = None;
                out.missing_k1.push(entry.id);
                out.investment_capacities = None;
                out.unallocated_tax += allowed;
            }
            continue;
        };
        let limit = limits.get(position).copied().unwrap_or(crate::schedule_k3::K3Limits::FULL);
        position += 1;
        let tax: i64 = k3
            .countries
            .iter()
            .filter(|row| chosen || crate::year_constants::is_section_901j_country(&row.country))
            .map(|row| fraction(row.tax, limit.tax.0, limit.tax.1))
            .sum();
        out.tax += tax;
        if sanctioned {
            for row in k3
                .countries
                .iter()
                .filter(|row| crate::year_constants::is_section_901j_country(&row.country))
            {
                *out.sanctioned.entry(row.country.to_ascii_uppercase()).or_default() +=
                    fraction(row.tax, limit.tax.0, limit.tax.1);
            }
        }
        // Released basis carries have no row in this year's K-3. Likewise a
        // K-1 tax not covered by the K-3 has no source character to invent.
        let all_current: i64 =
            k3.countries.iter().map(|row| fraction(row.tax, limit.tax.0, limit.tax.1)).sum();
        // The K-1 ledger owns whole-dollar allowances, while K-3 rows retain
        // cents. Compare in that owner's units; rounding a complete 99-cent
        // source to a dollar must not invent a missing one-cent carry.
        let missing = positive(allowed - dollars(all_current) * 100);
        if chosen && missing > 0 {
            out.investment_owners = None;
            out.missing_k1.push(entry.id);
            out.investment_capacities = None;
            out.unallocated_tax += missing;
        }
        // A Form 1116 category does not establish section 1411 activity
        // attribution. Keep the Schedule A deduction; leave only the uncertain
        // NIIT benefit out with a source-specific notice when it can matter.
        if tax > 0 {
            out.investment_owners = None;
            out.unallocated_k3.push(entry.id);
            out.investment_capacities = None;
            out.unallocated_tax += tax;
        }
    }
    out
}

pub(crate) fn foreign_credit(
    d: &FederalDraft,
    basis: CreditBasis,
    r: &mut FederalResult,
) -> (i64, i64, crate::injured_spouse::income_categories::NativeForeignCreditSources) {
    use crate::injured_spouse::income_categories::{
        NativeForeignCreditCapacity, NativeForeignCreditSources,
    };
    match &d.credits.as_ref().unwrap().foreign_tax {
        ForeignTaxClaim::No | ForeignTaxClaim::Deduction { .. } => (
            0,
            0,
            NativeForeignCreditSources {
                joint: 0,
                owners: Some([0; 2]),
                capacity: Some(NativeForeignCreditCapacity::PayeeCents([0; 2])),
            },
        ),
        ForeignTaxClaim::Statements { eligible } => {
            // Payee-statement taxes: the 1099 boxes on the Interest and
            // dividends entries and the Schedule K-3 taxes
            // (a K-3 with only passive rows is a payee statement). A K-1 tax
            // without a K-3 counts toward the $300/$600 test but is never
            // credited (its income category is unknown).
            let k3_entries = crate::schedule_k3::attached_sources(d);
            // One actual source traversal supplies both the election total
            // and payees. K-3 context stays bound to its own native rows.
            let mut document_sources = payee_document_tax_sources(d);
            let document_tax: i64 = document_sources.iter().map(|(_, _, tax)| tax).sum();
            let k3 = crate::schedule_k3::owned_credit(&k3_entries, false, &[]);
            let k3_tax: i64 = k3.tax.iter().sum();
            document_sources.extend(k3.rows.iter().filter_map(|row| {
                let (id, owner) = row.source?;
                Some((id, owner, row.tax - row.reduction))
            }));
            let documents = document_tax + k3_tax;
            let taxes = documents + crate::schedule_k3::k1_taxes_without_k3(d);
            let limit = if d.setup.as_ref().unwrap().filing_status() == FederalStatus::Joint {
                crate::year_constants::FOREIGN_DE_MINIMIS_JOINT_DOLLARS * 100
            } else {
                crate::year_constants::FOREIGN_DE_MINIMIS_DOLLARS * 100
            };
            crate::require_localized!(
                r,
                *eligible == Some(true)
                    && taxes <= limit
                    && (documents > 0 || !k3_entries.is_empty())
                    && k3_entries.iter().all(|k3| k3.schedule.passive_only())
                    && (taxes == 0
                        || basis.portfolio > 0
                        || k3_entries.iter().any(|k3| k3.schedule.taxes() > 0))
                    && !d.income.iter().any(|e| match &e.income {
                        FederalIncome::ForeignEmployment { .. } => true,
                        FederalIncome::Wage { data } => data.abroad > 0,
                        _ => false,
                    }),
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                "Foreign tax credit",
                "ISSUE_THE_PAYEE_STATEMENT_ELECTION_REQUIRES_ONLY_STATEMENT_REPORTED_PASSI_61D2975F",
                "The payee-statement election requires only statement-reported passive foreign income and taxes at or below $300 ($600 jointly); other claims require Form 1116.",
            );
            let credit = d
                .credits
                .as_ref()
                .unwrap()
                .foreign_tax
                .statement_credit(basis.regular, documents)
                .expect("payee-statement credit branch");
            let source_cents = document_sources
                .iter()
                .try_fold([0_i64; 2], |mut sums, (_, owner, tax)| {
                    if *tax == 0 {
                        return Some(sums);
                    }
                    if *tax < 0 {
                        return None;
                    }
                    let index = match owner {
                        Owner::Taxpayer => 0,
                        Owner::Spouse => 1,
                    };
                    sums[index] = sums[index].checked_add(*tax)?;
                    Some(sums)
                })
                .filter(|owners| owners[0].checked_add(owners[1]) == Some(documents));
            let owners = if credit == 0 {
                Some([0; 2])
            } else {
                source_cents.and_then(|cents| match cents {
                    [first, 0] if first > 0 => Some([credit, 0]),
                    [0, second] if second > 0 => Some([0, credit]),
                    _ => {
                        // Only a fully allowed source pool has a conserving
                        // rounded-pair projection. A binding joint cap cannot
                        // invent post-limit owners from gross source ratios.
                        let first = usize::from(
                            d.injured_spouse
                                .as_ref()
                                .is_some_and(|answer| answer.injured == Owner::Spouse),
                        );
                        crate::kernel::ratio::project_rounded_pair(credit, cents, first)
                    }
                })
            };
            (
                credit,
                credit,
                NativeForeignCreditSources {
                    joint: credit,
                    owners,
                    capacity: source_cents.map(NativeForeignCreditCapacity::PayeeCents),
                },
            )
        }
        ForeignTaxClaim::Ordinary {
            basis: tax_basis,
            sources,
            situations: _,
            history,
            carryback,
            amt_limitation,
            interest_allocation,
            investment_assets,
            loss_history,
            state_taxes,
            rate_adjustment,
        } => crate::form_1116::ordinary(
            d,
            &crate::form_1116::Claim {
                basis: tax_basis,
                sources,
                history,
                carryback,
                amt_limitation,
                interest_allocation,
                investment_assets,
                loss_history,
                state_taxes,
                rate_adjustment,
            },
            &basis,
            r,
        ),
    }
}

pub(crate) fn iso_adjustment(d: &FederalDraft, r: &mut FederalResult) -> i64 {
    let mut cents: i128 = 0;
    for (position, v) in d.screening.as_ref().unwrap().iso_exercises.iter().enumerate() {
        let price = v.recognition_price();
        crate::require_localized!(
            r,
            v.valid_timing()
                && v.held_units().is_some()
                && (0..=100_000_000_000).contains(&v.option_cost)
                && (0..=100_000_000_000).contains(&v.exercise_price)
                && (0..=100_000_000_000).contains(&v.market_price)
                && (0..=100_000_000_000).contains(&price)
                && (v.owner != Owner::Spouse
                    || d.setup.as_ref().unwrap().filing_status() == FederalStatus::Joint),
            &crate::IssueContext::owner(crate::TaxSection::InvestmentSales, v.owner)
                .item("iso_exercises", position)
                .named(&v.label),
            "Incentive stock options",
            "ISSUE_REVIEW_THE_OWNER_SHARE_COUNT_EXERCISE_PRICES_AND_SPECIAL_STOCK_REST_EE383591",
            "Review the owner, share count, exercise prices and special stock restrictions.",
        );
        // Form 6251 line 2i: the shares still held at year-end, at their
        // value when they entered AMT income, less their exercise price and
        // the option cost allocated to them. Shares disposed of in 2026, the
        // exercise year, have no adjustment (i6251).
        if let Some(held) = v.held_units()
            && (0..=100_000_000_000).contains(&price)
            && (0..=100_000_000_000).contains(&v.exercise_price)
            && (0..=100_000_000_000).contains(&v.option_cost)
        {
            let transferred = crate::employee_stock::share_units(v.shares).unwrap_or(1);
            let spread =
                crate::kernel::ratio::fraction(positive(price - v.exercise_price), held, 1_000_000);
            // The option cost covers every share of the exercise; the sold
            // shares take their part of it.
            let option_cost = crate::kernel::ratio::fraction(v.option_cost, held, transferred);
            cents += i128::from(phase::floor(spread.saturating_sub(option_cost), 0));
        }
    }
    crate::require_localized!(
        r,
        cents <= 100_000_000_000,
        &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
        "Incentive stock options",
        "ISSUE_THE_TOTAL_EXERCISE_SPREAD_EXCEEDS_THIS_PROTOTYPE_S_SUPPORTED_AMOUNT_173AE371",
        "The total exercise spread exceeds this prototype's supported amount range.",
    );
    dollars(min(cents, 100_000_000_000) as i64)
}

#[cfg(test)]
pub(crate) fn amt_tax(
    base: i64,
    preferential: i64,
    regular_ordinary: i64,
    status: FederalStatus,
    excluded: i64,
) -> i64 {
    amt_tax_with_collectibles(base, preferential, regular_ordinary, status, excluded, 0)
}
#[cfg(test)]
pub(crate) fn amt_tax_with_collectibles(
    base: i64,
    preferential: i64,
    regular_ordinary: i64,
    status: FederalStatus,
    excluded: i64,
    collectibles: i64,
) -> i64 {
    amt_tax_with_special_gains(
        base,
        preferential,
        regular_ordinary,
        status,
        excluded,
        [collectibles, 0],
    )
}
#[cfg(test)]
pub(crate) fn amt_tax_with_special_gains(
    base: i64,
    preferential: i64,
    regular_ordinary: i64,
    status: FederalStatus,
    excluded: i64,
    special: [i64; 2],
) -> i64 {
    amt_tax_with_foreign_gains(
        base,
        preferential,
        regular_ordinary,
        status,
        excluded,
        special,
        None,
    )
}
#[cfg(test)]
pub(crate) fn amt_tax_with_foreign_gains(
    base: i64,
    preferential: i64,
    regular_ordinary: i64,
    status: FederalStatus,
    excluded: i64,
    special: [i64; 2],
    foreign: Option<crate::income_tax::Preference>,
) -> i64 {
    crate::income_tax::AmtTaxWorksheet {
        year: 2026,
        status,
        base,
        preferential,
        regular_ordinary,
        excluded,
        special,
        foreign,
    }
    .tax()
}

pub(crate) fn record_amt(
    r: &mut FederalResult,
    base: i64,
    amt: i64,
    status: FederalStatus,
    tentative: i64,
    foreign_credit: i64,
    regular_offset: i64,
) {
    let [income, _, _, _, _, exemption, taxable] =
        crate::income_tax::AmtRates::for_year(2026, status).unwrap().exemption_lines(base);
    let required = amt > 0;
    // A positive line11 establishes a Form6251 filing condition. Other filing
    // conditions and PartI/III projection remain separate PRINT work; never
    // substitute positive AMTI for a filing requirement. These rows reuse the
    // actual exemption and comparison owners, including MFS income additions,
    // foreign-credit subtraction and unaveraged ScheduleJ regular tax.
    crate::kernel::lines::push_line(
        r,
        required.then(|| FormLine::new(TaxForm::Form6251, "4")),
        "alternative_minimum_taxable_income",
        income * 100,
        &["agi", "deductions", "income", "screening.iso_exercises"],
        AMT,
        msg!("EXPLAIN_ALTERNATIVE_MINIMUM_TAXABLE_INCOME"),
    );
    crate::kernel::lines::push_line(
        r,
        required.then(|| FormLine::new(TaxForm::Form6251, "11")),
        "alternative_minimum_tax",
        amt * 100,
        &["alternative_minimum_taxable_income", "regular_income_tax", "foreign_tax_credit"],
        AMT,
        msg!("EXPLAIN_ALTERNATIVE_MINIMUM_TAX"),
    );
    if required {
        emit_lines(
            r,
            &mut (),
            [
                ("5", "amt_exemption", exemption),
                ("6", "amt_taxable_base", taxable),
                ("7", "amt_tax_before_foreign_credit", tentative),
                ("8", "amt_foreign_credit", foreign_credit),
                ("9", "amt_tentative_minimum_tax", positive(tentative - foreign_credit)),
                ("10", "amt_regular_tax_offset", regular_offset),
            ],
            |(number, id, value), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form6251, number)),
                    id,
                    LineValue::MoneyDollars(value),
                    &[
                        "alternative_minimum_taxable_income",
                        "regular_income_tax",
                        "foreign_tax_credit",
                    ],
                    AMT,
                    msg!("EXPLAIN_AMT_COMPARISON"),
                ))
            },
        );
    }
}

pub(crate) fn valid_credit(claim: &ForeignTaxClaim) -> bool {
    let amount = |v: i64| (0..=100_000_000_000).contains(&v);
    match claim {
        ForeignTaxClaim::No => true,
        ForeignTaxClaim::Deduction { taxes } => {
            taxes.len() <= 50
                && taxes.iter().all(|row| {
                    !row.label.trim().is_empty() && (1..=100_000_000_000).contains(&row.tax)
                })
        }
        ForeignTaxClaim::Statements { .. } => true,
        ForeignTaxClaim::Ordinary {
            sources,
            amt_limitation,
            history,
            carryback,
            state_taxes,
            loss_history,
            investment_assets,
            ..
        } => {
            history.valid()
                && state_taxes.valid()
                && loss_history.valid()
                && investment_assets.valid()
                && carryback.valid()
                && amt_limitation.valid()
                && sources.len() <= 100
                && sources.iter().all(|s| {
                    !s.label.trim().is_empty()
                        && crate::form_1116::country_size_valid(&s.country)
                        && amount(s.income)
                        && amount(s.tax)
                        && amount(s.qualified())
                        && s.qualified() <= s.income
                        && match s.category {
                            ForeignTaxCategory::PriorEmployeeWageTax { income_year, .. }
                            | ForeignTaxCategory::PriorPassiveTax { income_year, .. } => {
                                (1900..=2025).contains(&income_year) && s.income == 0 && s.tax > 0
                            }
                            _ => true,
                        }
                })
        }
    }
}

pub(crate) fn valid_exercises(values: &[IsoExercise]) -> bool {
    values.iter().all(|v| {
        !v.label.trim().is_empty()
            && v.corporation.as_ref().is_none_or(|c| !c.trim().is_empty())
            && v.valid_timing()
            && v.held_units().is_some()
            && (0..=100_000_000_000).contains(&v.option_cost)
            && (0..=100_000_000_000).contains(&v.exercise_price)
            && (0..=100_000_000_000).contains(&v.market_price)
            && (0..=100_000_000_000).contains(&v.recognition_price())
    })
}

// Schedule D carryover worksheet: loss use depends on unfloored taxable income.
// Current and prior years share this engine; short-term losses are used first.
// Section 1212(b)(2): taxable is unfloored taxable income before section
// 151 deductions (including the enhanced senior deduction), or AMT exemptions.
pub(crate) fn capital_carryover(short: i64, long: i64, deducted: i64, taxable: i64) -> [i64; 2] {
    let used = phase::cap(deducted, [positive(taxable + deducted)]);
    let short_loss = positive(-short);
    [
        crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            short_loss - used,
            positive(long),
        )
        .get(),
        crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            -long - positive(short),
            positive(used - short_loss),
        )
        .get(),
    ]
}

pub(crate) fn regular_capital_carry(v: &PriorCapitalReturn) -> [i64; 2] {
    capital_carryover(
        dollars(v.short),
        dollars(v.long),
        dollars(v.deducted_loss),
        dollars(v.agi) - dollars(v.deductions) + dollars(v.senior_deduction),
    )
}

pub(crate) fn capital_caps(v: &PriorCapitalReturn) -> crate::InvestmentCapitalCap {
    let regular = regular_capital_carry(v);
    let amt = v.amt.as_ref().map_or(regular, |a| {
        capital_carryover(
            dollars(a.short),
            dollars(a.long),
            dollars(a.deducted_loss),
            dollars(a.taxable_income),
        )
    });
    crate::InvestmentCapitalCap {
        short: regular[0] * 100,
        long: regular[1] * 100,
        amt_short: amt[0] * 100,
        amt_long: amt[1] * 100,
    }
}
// No loss-source engine is duplicated: these are portions of the carryover
// computed by the existing Schedule D owner. Missing portions have endpoints,
// never an affirmative zero classification.
pub(crate) fn investment_capital_parts(v: &PriorCapitalReturn) -> Option<[[i64; 2]; 2]> {
    let caps = capital_caps(v);
    let parts = v.investment_parts.as_ref()?;
    let regular = [dollars(parts.short), dollars(parts.long)];
    let amt = if let Some(a) = &parts.amt {
        [dollars(a.short), dollars(a.long)]
    } else if caps.short == caps.amt_short && caps.long == caps.amt_long {
        regular
    } else {
        return None;
    };
    Some([regular, amt])
}
// Relative to PriorCapitalReturn and its editable PriorCapitalReturnPage:
// investment_parts is copied field-for-field, without a Union variant segment.
pub(crate) fn investment_parts_issue(
    v: &PriorCapitalReturn,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    let cap = capital_caps(v);
    let p = v.investment_parts.as_ref()?;
    if (cap.short != cap.amt_short || cap.long != cap.amt_long) && p.amt.is_none() {
        return Some(
            ValidationIssue::new("INVESTMENT_AMT_PARTS_REQUIRED")
                .at_field("amt")
                .in_field("investment_parts"),
        );
    }
    if !(0..=cap.short).contains(&p.short) {
        return Some(
            ValidationIssue::new("INVESTMENT_ST_ABOVE_CARRY")
                .at_field("short")
                .in_field("investment_parts"),
        );
    }
    if !(0..=cap.long).contains(&p.long) {
        return Some(
            ValidationIssue::new("INVESTMENT_LT_ABOVE_CARRY")
                .at_field("long")
                .in_field("investment_parts"),
        );
    }
    if let Some(a) = &p.amt {
        if !(0..=cap.amt_short).contains(&a.short) {
            return Some(
                ValidationIssue::new("INVESTMENT_AMT_ST_ABOVE_CARRY")
                    .at_field("short")
                    .in_field("amt")
                    .in_field("investment_parts"),
            );
        }
        if !(0..=cap.amt_long).contains(&a.long) {
            return Some(
                ValidationIssue::new("INVESTMENT_AMT_LT_ABOVE_CARRY")
                    .at_field("long")
                    .in_field("amt")
                    .in_field("investment_parts"),
            );
        }
    }
    None
}
pub(crate) fn investment_parts_valid(v: &PriorCapitalReturn) -> bool {
    investment_parts_issue(v).is_none()
}
pub(crate) fn all_capital_is_investment(v: &PriorCapitalReturn) -> bool {
    let c = capital_caps(v);
    investment_capital_parts(v)
        == Some([
            [
                crate::kernel::ratio::trunc_i64(c.short, 100),
                crate::kernel::ratio::trunc_i64(c.long, 100),
            ],
            [
                crate::kernel::ratio::trunc_i64(c.amt_short, 100),
                crate::kernel::ratio::trunc_i64(c.amt_long, 100),
            ],
        ])
}
// Retained repayment comparisons may carry source portions only when the
// actual calculated balances are unchanged, or all source loss was established
// as investment property. A changed mixed pool needs its actual source records.
pub(crate) fn revised_investment_parts(
    old: &PriorCapitalReturn,
    new: &PriorCapitalReturn,
) -> Option<crate::InvestmentCapitalParts> {
    let n = capital_caps(new);
    if all_capital_is_investment(old) {
        Some(crate::InvestmentCapitalParts {
            short: n.short,
            long: n.long,
            amt: Some(crate::InvestmentAmtCapitalParts { short: n.amt_short, long: n.amt_long }),
        })
    } else if capital_caps(old) == n {
        old.investment_parts.clone()
    } else {
        None
    }
}

pub(crate) fn prior_niit_difference(d: &FederalDraft, r: &mut FederalResult) -> i64 {
    let mut total = 0;
    for prior in &d.screening.as_ref().unwrap().capital_history {
        let context = crate::IssueContext::Record {
            section: crate::TaxSection::InvestmentSales,
            identity: crate::federal_model::NoteIdentity::FilingYear {
                section: crate::TaxSection::InvestmentSales,
                year: 2025,
                filers: match prior.owners {
                    PriorReturnOwners::Taxpayer => crate::RepaymentFilers::Taxpayer,
                    PriorReturnOwners::Spouse => crate::RepaymentFilers::Spouse,
                    PriorReturnOwners::Joint => crate::RepaymentFilers::Both,
                },
            },
            label: crate::Message::new(
                "ISSUE_REPAYMENT_SOURCE_YEAR",
                alloc::vec![
                    crate::MessageArg::Number(2025),
                    crate::MessageArg::Resource(
                        match prior.owners {
                            PriorReturnOwners::Taxpayer => "PERSON_YOU",
                            PriorReturnOwners::Spouse => "PERSON_SPOUSE",
                            PriorReturnOwners::Joint => "PERSON_YOU_AND_SPOUSE",
                        }
                        .into()
                    ),
                ],
            ),
        };
        let carry = regular_capital_carry(prior).iter().sum::<i64>();
        let difference = match prior.niit_difference {
            Some(value) => dollars(value),
            None if carry == 0 => 0,
            None => {
                crate::issue_localized!(
                    r,
                    &context,
                    "Investment-tax capital history",
                    "ISSUE_ENTER_THE_RETAINED_FORM_8960_EXCLUDED_LOSS_DIFFERENCE_INCLUDING_CON_A89352C6",
                    "Enter the retained Form 8960 excluded-loss difference, including confirmed zero; Schedule D alone does not establish the investment-tax balance.",
                );
                continue;
            }
        };
        crate::require_localized!(
            r,
            difference <= carry,
            &context,
            "Investment-tax capital history",
            "ISSUE_THE_EXCLUDED_LOSS_DIFFERENCE_MUST_FIT_THE_REGULAR_CAPITAL_CARRYOVER_C50F91A2",
            "The Form 8960 excluded-loss difference must fit the regular capital carryover; it is independent of Form 4952 investment-property attribution.",
        );
        total += difference;
    }
    total
}

// The part of the 2025 capital loss carryover that has section 1256 character
// (Pub. 550: treated in the carryover year as a loss from section 1256
// contracts), whole dollars, never more than the regular carryover itself.
pub(crate) fn prior_section_1256(d: &FederalDraft) -> i64 {
    d.screening.as_ref().map_or(0, |s| {
        s.capital_history
            .iter()
            .filter(|v| v.ordinary_history)
            .map(|v| {
                let carry = regular_capital_carry(v);
                phase::cap(dollars(v.section_1256), [carry[0] + carry[1]])
            })
            .sum()
    })
}

pub(crate) fn prior_capital(d: &FederalDraft, r: &mut FederalResult) -> [[i64; 2]; 2] {
    let entries = &d.screening.as_ref().unwrap().capital_history;
    let joint = d.setup.as_ref().unwrap().filing_status() == FederalStatus::Joint;
    let mut seen = [false; 2];
    let mut result = [[0; 2]; 2];
    for v in entries {
        let owners = match v.owners {
            PriorReturnOwners::Taxpayer => [true, false],
            PriorReturnOwners::Spouse => [false, true],
            PriorReturnOwners::Joint => [true, true],
        };
        crate::require_localized!(
            r,
            !(owners[0] && seen[0] || owners[1] && seen[1])
                && (!owners[1] || joint)
                && (matches!(v.owners, PriorReturnOwners::Joint)
                    == (v.status == FederalStatus::Joint))
                // An unsupported history was left out before the rules ran.
                && v.ordinary_history,
            &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            "Capital loss history",
            "ISSUE_REVIEW_PRIOR_RETURN_OWNERSHIP_FILING_STATUS_AND_SPECIAL_LOSS_ADJUST_5C27A4B6",
            "Review prior-return ownership, filing status and special loss adjustments; do not duplicate a return or allocate a former spouse's loss here.",
        );
        seen[0] |= owners[0];
        seen[1] |= owners[1];
        let short = dollars(v.short);
        let long = dollars(v.long);
        let deducted = dollars(v.deducted_loss);
        crate::require_localized!(
            r,
            deducted
                == phase::cap(
                    positive(-short - long),
                    [if v.status == FederalStatus::Separate {
                        crate::year_constants::CAPITAL_LOSS_LIMIT_SEPARATE_DOLLARS
                    } else {
                        crate::year_constants::CAPITAL_LOSS_LIMIT_DOLLARS
                    }],
                ),
            &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            "Capital loss history",
            "ISSUE_THE_PRIOR_SCHEDULE_D_NET_AMOUNTS_AND_LINE_21_LOSS_MUST_RECONCILE_11918C45",
            "The prior Schedule D net amounts and line 21 loss must reconcile.",
        );
        crate::require_localized!(
            r,
            v.senior_deduction <= v.deductions
                && v.senior_deduction
                    <= match v.status {
                        FederalStatus::Joint => {
                            2 * crate::year_constants::SENIOR_DEDUCTION_PER_PERSON_CENTS
                        }
                        FederalStatus::Separate => 0,
                        _ => crate::year_constants::SENIOR_DEDUCTION_PER_PERSON_CENTS,
                    },
            &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            "Capital loss history",
            "ISSUE_THE_PRIOR_ENHANCED_SENIOR_DEDUCTION_MUST_BE_INCLUDED_IN_TOTAL_DEDUC_A4F69A67",
            "The prior enhanced senior deduction must be included in total deductions and respect that return's filing-status limit. Do not enter the additional standard deduction here.",
        );
        let carry = regular_capital_carry(v);
        // The section 1256 part is a part of the regular carryover, not more.
        crate::require_localized!(
            r,
            dollars(v.section_1256) <= carry[0] + carry[1],
            &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            "Capital loss history",
            "ISSUE_THE_SECTION_1256_PART_OF_THE_PRIOR_CARRYOVER_EXCEEDS_THE_CARRYOVER",
            "The part of the 2025 carryover from section 1256 contracts cannot be more than the short- and long-term carryovers calculated from that return.",
        );
        if !v.origins.is_empty() {
            let totals = v.origins.iter().fold([0i64; 2], |mut total, origin| {
                total[0] += origin.short;
                total[1] += origin.long;
                total
            });
            crate::require_localized!(
                r,
                [dollars(totals[0]), dollars(totals[1])] == carry,
                &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                "Capital loss history",
                "ISSUE_LOSS_ORIGIN_WORKSHEET_BALANCES_MUST_RECONCILE_SEPARATELY_TO_THE_CAL_CA991D60",
                "Loss-origin worksheet balances must reconcile separately to the calculated short- and long-term carryovers after 2025 use. Supply all surviving years or leave the optional history empty when unavailable.",
            );
        }
        let alternative = if let Some(amt) = &v.amt {
            let short = dollars(amt.short);
            let long = dollars(amt.long);
            let deducted = dollars(amt.deducted_loss);
            crate::require_localized!(
                r,
                deducted
                    == phase::cap(
                        positive(-short - long),
                        [if v.status == FederalStatus::Separate {
                            crate::year_constants::CAPITAL_LOSS_LIMIT_SEPARATE_DOLLARS
                        } else {
                            crate::year_constants::CAPITAL_LOSS_LIMIT_DOLLARS
                        }],
                    ),
                &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                "Capital loss history",
                "ISSUE_THE_PRIOR_AMT_SCHEDULE_D_NET_GAINS_LOSSES_AND_LINE_21_DEDUCTION_MUS_AE0A8857",
                "The prior AMT Schedule D net gains/losses and line 21 deduction must reconcile independently from regular tax.",
            );
            capital_carryover(short, long, deducted, dollars(amt.taxable_income))
        } else {
            carry
        };
        for (sum, amounts) in result.iter_mut().zip([carry, alternative]) {
            sum[0] += amounts[0];
            sum[1] += amounts[1];
        }
    }
    result
}

pub(crate) fn valid_capital_history(entries: &[PriorCapitalReturn]) -> bool {
    // Tax structure: one prior-return capital record per taxpayer or spouse.
    entries.len() <= 2
        && entries.iter().all(|v| {
            [v.agi, v.short, v.long]
                .iter()
                .all(|n| (-100_000_000_000..=100_000_000_000).contains(n))
                && (0..=100_000_000_000).contains(&v.deductions)
                && (0..=300_000).contains(&v.deducted_loss)
                && (0..=100_000_000_000).contains(&v.section_1256)
                && (0..=1_200_000).contains(&v.senior_deduction)
                && v.niit_difference.is_none_or(|amount| (0..=100_000_000_000).contains(&amount))
                && v.amt.as_ref().is_none_or(|a| {
                    [a.taxable_income, a.short, a.long]
                        .iter()
                        .all(|n| (-100_000_000_000..=100_000_000_000).contains(n))
                        && (0..=300_000).contains(&a.deducted_loss)
                })
                && investment_parts_valid(v)
                && v.injured_owners.as_ref().is_none_or(|owners| {
                    matches!(v.owners, PriorReturnOwners::Joint)
                        && owners.valid(regular_capital_carry(v).map(|amount| amount * 100))
                })
                && v.origins.iter().enumerate().all(|(i, origin)| {
                    (1900..=2025).contains(&origin.year)
                        // Arithmetic safety: the unique finite source years and
                        // these scalar bounds protect unchecked origin totals.
                        && (0..=100_000_000_000).contains(&origin.short)
                        && (0..=100_000_000_000).contains(&origin.long)
                        && !v.origins[..i].iter().any(|prior| prior.year == origin.year)
                })
        })
}

// Schedule R: age qualification and taxable disability-income ceilings.
pub(crate) fn elderly_credit(
    status: FederalStatus,
    apart: bool,
    seniors: usize,
    disability: [i64; 2],
    agi: i64,
    exempt: i64,
) -> i64 {
    let disabled = disability.iter().filter(|v| **v > 0).count();
    if seniors + disabled == 0 || (status == FederalStatus::Separate && !apart) {
        return 0;
    }
    let (base, threshold) = match status {
        FederalStatus::Joint => (if seniors + disabled == 2 { 7500 } else { 5000 }, 10000),
        FederalStatus::Separate => (3750, 5000),
        _ => (5000, 7500),
    };
    let base = if disabled > 0 {
        phase::cap(base, [disability.iter().sum::<i64>() + if seniors > 0 { 5000 } else { 0 }])
    } else {
        base
    };
    fraction(phase::phase_out(base - exempt, fraction(positive(agi - threshold), 1, 2)), 15, 100)
}

// Schedule 1-A Part V, 2025–2028. Eligibility and MAGI sources belong to the caller.
pub(crate) fn senior_deduction(status: FederalStatus, eligible: i64, magi: i64) -> i64 {
    if status == FederalStatus::Separate {
        return 0;
    }
    eligible
        * phase::phase_out(
            6000,
            fraction(
                positive(magi - if status == FederalStatus::Joint { 150000 } else { 75000 }),
                6,
                100,
            ),
        )
}

// Reg. 1.469-2T(f)(3): this basis test applies to every rental type.
fn nondepreciable(v: &FederalRental) -> bool {
    let total: i128 = v.buildings.iter().map(|b| i128::from(b.cost)).sum::<i128>()
        + v.assets.iter().map(|a| i128::from(a.cost)).sum::<i128>();
    let depreciable: i128 = v.buildings.iter().map(|b| i128::from(b.cost - b.land)).sum::<i128>()
        + v.assets.iter().map(|a| i128::from(a.cost)).sum::<i128>();
    total > 0 && depreciable * 10 < total * 3
}

// At-risk loss character stays attached to its source form. Amounts are cents.
#[derive(Clone, Debug)]
pub(crate) struct RentalRisk {
    pub ordinary: i64,
    pub sale: i64,
    pub recapture: Result<i64, crate::activity_risk::ProjectionIssue>,
}
impl RentalRisk {
    fn ordinary_income(&self) -> Result<i64, crate::activity_risk::ProjectionIssue> {
        crate::kernel::ratio::checked_sum([self.ordinary, self.recapture.clone()?]).ok_or(
            crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Overflow,
            ),
        )
    }
    fn from_activity(activity: &crate::activity_risk::ActivityProjection) -> Self {
        Self {
            ordinary: activity.components.allowed[0],
            sale: activity.components.allowed[1],
            recapture: activity.recapture.clone(),
        }
    }
}
pub(crate) fn rental_risk(
    v: &FederalRental,
    ordinary: i64,
    sale: i64,
    amt: bool,
) -> Result<RentalRisk, crate::activity_risk::ProjectionIssue> {
    match &v.at_risk {
        Some(crate::RentalAtRisk::All) => Ok(RentalRisk { ordinary, sale, recapture: Ok(0) }),
        Some(crate::RentalAtRisk::Limited { record, .. }) => {
            let record = record.as_deref().ok_or(crate::activity_risk::ProjectionIssue::Source(
                "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
            ))?;
            let activity = crate::activity_risk::project_record(
                &[
                    (crate::activity_risk::Part1Line::Operating, ordinary),
                    (crate::activity_risk::Part1Line::Property, sale),
                ],
                record,
                &crate::RiskEffectiveCategory::HoldingRealProperty,
                amt,
            )?;
            Ok(RentalRisk::from_activity(&activity))
        }
        None => Err(crate::activity_risk::ProjectionIssue::Source(
            "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
        )),
    }
}
pub(crate) fn rental_source_risk(
    d: &FederalDraft,
    id: i64,
    _property: &FederalRental,
    amt: bool,
    sale: i64,
) -> Result<Option<RentalRisk>, crate::activity_risk::ProjectionIssue> {
    let Some(whole) = d.income.iter().find(|entry| entry.id == id) else {
        return Ok(None);
    };
    let allocated = crate::community::record(d, whole);
    let FederalIncome::Rental { data: v } = allocated.as_ref() else {
        return Ok(None);
    };
    let projected = v.clone();
    let usage = rental_use_core(
        &projected,
        amt,
        crate::depreciation::mid_quarter(d),
        true,
        None,
        &rental_expense_rows(d, whole),
        Some((d, id, false)),
    )
    .map_err(|(key, _)| crate::activity_risk::ProjectionIssue::Source(key))?;
    Ok(Some(rental_risk(v, usage.net, sale, amt)?))
}
fn rental_at_risk(
    v: &FederalRental,
    id: i64,
    raw_sale: i64,
    amt: bool,
    usage: &mut RentalUse,
    r: &mut FederalResult,
) -> Result<Option<i64>, crate::activity_risk::ProjectionIssue> {
    let Some(crate::RentalAtRisk::Limited { record, .. }) = &v.at_risk else {
        // The source admission above distinguishes a missing answer from All.
        let projection = rental_risk(v, usage.net, raw_sale, amt)?;
        usage.net = projection.ordinary_income()?;
        projection.recapture?;
        return Ok(Some(0));
    };
    let record = record.as_deref().ok_or(crate::activity_risk::ProjectionIssue::Source(
        "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
    ))?;
    let activity = crate::activity_risk::project_record(
        &[
            (crate::activity_risk::Part1Line::Operating, usage.net),
            (crate::activity_risk::Part1Line::Property, raw_sale),
        ],
        record,
        &crate::RiskEffectiveCategory::HoldingRealProperty,
        amt,
    )?;
    let prior_allowed = crate::activity_risk::prior_operating_allowed(
        match &usage.current_operating_for_risk {
            Some(amount) => Some(exact::rounded(amount).ok_or(
                crate::activity_risk::ProjectionIssue::Source("RENTAL_EXPENSE_AMOUNT_UNAVAILABLE"),
            )?),
            None => None,
        },
        usage.prior_at_risk_cents,
        activity.components.suspended[0],
    )?;
    let projection = RentalRisk::from_activity(&activity);
    usage.net = projection.ordinary_income()?;
    projection.recapture?;
    report_risk_projection(id, amt, "income.rental.at_risk", activity, r)?;
    Ok(prior_allowed)
}

pub(crate) fn report_risk_projection(
    id: i64,
    amt: bool,
    input: &str,
    activity: crate::activity_risk::ActivityProjection,
    r: &mut FederalResult,
) -> Result<i64, crate::activity_risk::ProjectionIssue> {
    let recapture = activity.recapture.clone()?;
    let sum = |values: &[i64]| {
        crate::kernel::ratio::checked_sum(values.iter().copied()).ok_or(
            crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Overflow,
            ),
        )
    };
    let property_carry = *activity.components.suspended.last().expect("activity component");
    let property_allowed = *activity.components.allowed.last().expect("activity component");
    let kind = if input == "income.business.at_risk" { "business" } else { "rental" };
    let total_carry = sum(&activity.components.suspended)?;
    let prefix = if amt { "amt_" } else { "" };
    emit_lines(
        r,
        &mut (),
        [
            ("at_risk_carryover", total_carry),
            ("at_risk_ordinary_carryover", activity.components.suspended[0]),
            ("at_risk_section1231_carryover", property_carry),
            ("at_risk_allowed_ordinary", activity.components.allowed[0]),
            ("at_risk_allowed_section1231", property_allowed),
        ],
        |(suffix, amount), _, _| {
            if activity.components.allowed.len() == 1 && suffix.contains("section1231") {
                return None;
            }
            Some(calculation_row(
                None,
                &alloc::format!("{prefix}{kind}_{id}_{suffix}"),
                LineValue::MoneyCents(amount),
                &[input],
                "https://www.irs.gov/instructions/i6198",
                msg!("EXPLAIN_RENTAL_AT_RISK"),
            ))
        },
    );
    if activity.components.allowed.len() == 3 {
        emit_lines(
            r,
            &mut (),
            [
                ("at_risk_ordinary_property_carryover", activity.components.suspended[1]),
                ("at_risk_allowed_ordinary_property", activity.components.allowed[1]),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("{prefix}{kind}_{id}_{suffix}"),
                    LineValue::MoneyCents(amount),
                    &[input],
                    "https://www.irs.gov/instructions/i6198",
                    msg!("EXPLAIN_RENTAL_AT_RISK"),
                ))
            },
        );
    }
    emit_lines(
        r,
        &mut (),
        ["at_risk_recapture", "at_risk_recapture_deduction_next_year"],
        |suffix, _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("{prefix}{kind}_{id}_{suffix}"),
                LineValue::MoneyCents(recapture),
                &[input],
                "https://www.irs.gov/instructions/i6198",
                msg!("EXPLAIN_RENTAL_AT_RISK_RECAPTURE"),
            ))
        },
    );
    if !amt {
        emit_lines(r, &mut (), activity.lines, |(printed, amount), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form6198, printed)),
                &alloc::format!("{kind}_{id}_form6198_line{printed}"),
                LineValue::MoneyCents(amount),
                &[input],
                "https://www.irs.gov/instructions/i6198",
                msg!("EXPLAIN_RENTAL_AT_RISK"),
            ))
        });
    }
    Ok(recapture)
}

// Canonical identities are durable source facts, never list positions or echoed catalogs.
pub(crate) fn rental_mortgage_roots(d: &FederalDraft) -> alloc::vec::Vec<i64> {
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Rental { data } => Some(&data.mortgage_loans),
            _ => None,
        })
        .flatten()
        .copied()
        .collect()
}
pub(crate) fn rental_mortgage_rule(d: &FederalDraft) -> Option<&'static str> {
    rental_mortgage_issue(d).map(|(_, issue)| issue.key)
}
pub(crate) fn rental_mortgage_issue(
    d: &FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    use crate::validation::ValidationIssue;
    if let Some(failure) = crate::estate_trust::mortgage_graph_issue(d) {
        return Some(failure);
    }
    let loans = d.deductions.as_ref().map_or(&[][..], |v| v.mortgage_loans());
    let mut seen = alloc::vec::Vec::new();
    for e in &d.income {
        let FederalIncome::Rental { data } = &e.income else {
            continue;
        };
        if (!matches!(data.personal_use, RentalPersonalUse::No)
            && data.expenses.iter().any(|v| {
                matches!(v.category, RentalExpenseCategory::MortgageInterest) && v.amount != 0
            }))
            || (matches!(data.personal_use, RentalPersonalUse::No)
                && !data.mortgage_loans.is_empty())
        {
            return Some((
                e.id,
                ValidationIssue::new("RENTAL_MORTGAGE_SINGLE_SOURCE").at_field("mortgage_loans"),
            ));
        }
        for (index, id) in data.mortgage_loans.iter().enumerate() {
            if *id <= 0
                || !loans.iter().any(|l| {
                    l.id == *id && matches!(l.details, crate::MortgageLoanDetails::FirstLender(_))
                })
            {
                return Some((
                    e.id,
                    ValidationIssue::new("MORTGAGE_RENTAL_ROOT_NEEDED")
                        .at_list_item("mortgage_loans", index),
                ));
            }
            if seen.contains(id) {
                return Some((
                    e.id,
                    ValidationIssue::new("RENTAL_MORTGAGE_ROOT_TWICE")
                        .at_list_item("mortgage_loans", index),
                ));
            }
            seen.push(*id);
        }
    }
    None
}
// The first-lender loan a current Form 8396 certificate names, and the
// credit on it in cents: section 163(g) reduces the deduction for interest
// on that loan, the rental share of it on Schedule E.
fn certified_loan(d: &FederalDraft) -> Option<(i64, i64)> {
    let c = d.credits.as_ref()?;
    let credit = c.mortgage_credit.as_ref()?;
    credit.current.as_ref()?;
    let loan = credit.loan?;
    d.deductions
        .as_ref()?
        .mortgage_loans()
        .iter()
        .any(|l| l.id == loan && matches!(l.details, crate::MortgageLoanDetails::FirstLender(_)))
        .then(|| {
            (
                loan,
                crate::federal_rules::mortgage_credit_line3(c, &mut FederalResult::empty(0)) * 100,
            )
        })
}
fn loan_credit(certified: Option<(i64, i64)>, canonical_id: i64) -> i64 {
    certified.filter(|(loan, _)| *loan == canonical_id).map_or(0, |(_, credit)| credit)
}
// The rental share of the certified loan's credit; the rest reduces Schedule A.
pub(crate) fn rental_credit_share(d: &FederalDraft, credit_cents: i64) -> i64 {
    let Some((loan, _)) = certified_loan(d) else {
        return 0;
    };
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Rental { data } if data.mortgage_loans.contains(&loan) => {
                Some(rental_fraction(data, credit_cents))
            }
            _ => None,
        })
        .sum()
}
// Section 280A(d)(1): the dwelling was used as a residence when personal
// use exceeded the greater of 14 days or 10% of the days rented at a fair
// rental; an overlap day is personal use and not a fair-rental day here.
pub(crate) fn used_as_home(v: &FederalRental) -> bool {
    let RentalPersonalUse::Yes { fair_rental_days, personal_days, overlap_days, .. } =
        v.personal_use
    else {
        return false;
    };
    let personal = i64::from(personal_days) + i64::from(overlap_days);
    personal > 14 && personal * 10 > i64::from(fair_rental_days) - i64::from(overlap_days)
}
fn rental_fraction(v: &FederalRental, cents: i64) -> i64 {
    let RentalPersonalUse::Yes { fair_rental_days, personal_days, .. } = v.personal_use else {
        return 0;
    };
    let fair = i64::from(fair_rental_days);
    let personal = i64::from(personal_days);
    if fair + personal == 0 || (used_as_home(v) && fair < 15) {
        0
    } else {
        fraction(cents, fair, fair + personal)
    }
}
// The part of a linked loan amount that leaves Schedule A: the rental share,
// or the whole amount when the loan's second home is not used as a residence
// under section 280A(d)(1), so it is not a qualified residence (section
// 163(h)(4)(A)(i)(II); Pub. 936 "Second home rented out") and its personal
// share is nondeductible personal interest.
fn linked_loan_share(
    d: &FederalDraft,
    data: &FederalRental,
    l: &crate::mortgage_limit::LoanAllowance,
    cents: i64,
) -> i64 {
    let second = d.deductions.as_ref().map_or(&[][..], |v| v.mortgage_loans()).iter().any(|loan| {
        loan.id == l.canonical_id
            && matches!(
                loan.home(),
                crate::MortgageHome::Second | crate::MortgageHome::SecondVehicle
            )
    });
    if second && !used_as_home(data) { cents } else { rental_fraction(data, cents) }
}
fn rental_loan_allowances(
    d: &FederalDraft,
) -> alloc::vec::Vec<crate::mortgage_limit::LoanAllowance> {
    crate::mortgage_limit::loan_allowances(
        d.deductions.as_ref().map_or(&[][..], |v| v.mortgage_loans()),
        d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Separate),
    )
}
// Whether a linked loan's points reach this rental spread over the term.
fn straight_lined_points(d: &FederalDraft, v: &FederalRental) -> bool {
    rental_fraction(v, 1_000_000) > 0
        && d.deductions.as_ref().map_or(&[][..], |ded| ded.mortgage_loans()).iter().any(|l| {
            v.mortgage_loans.contains(&l.id)
                && matches!(
                    l.points,
                    crate::MortgagePoints::Amortized { .. }
                        | crate::MortgagePoints::Improvement { .. }
                )
        })
}
fn whole_rental_expense_rows(
    d: &FederalDraft,
    v: &FederalRental,
) -> Vec<crate::extended_model::RentalExpenseAmount> {
    let mut rows = RentalExpense::exact_rows(&v.expenses);
    if v.mortgage_loans.is_empty() {
        return rows;
    }
    rows.retain(|row| row.category != RentalExpenseCategory::MortgageInterest);
    let mut interest = exact::whole(0);
    let mut insurance = exact::whole(0);
    let certified = certified_loan(d);
    for loan in rental_loan_allowances(d)
        .iter()
        .filter(|loan| v.mortgage_loans.contains(&loan.canonical_id))
    {
        interest += exact::whole(positive(
            loan.qualified_interest_cents - loan_credit(certified, loan.canonical_id),
        )) + exact::whole(loan.rental_points_cents);
        insurance += exact::whole(loan.qualified_insurance_cents);
    }
    for (category, amount) in [
        (RentalExpenseCategory::MortgageInterest, interest),
        (RentalExpenseCategory::Insurance, insurance),
    ] {
        if let Some(row) = rows.iter_mut().find(|row| {
            row.category == category && row.treatment == crate::BusinessExpenseKind::Ordinary
        }) {
            row.amount += amount;
        } else {
            rows.push(crate::extended_model::RentalExpenseAmount {
                category,
                amount,
                treatment: crate::BusinessExpenseKind::Ordinary,
            });
        }
    }
    rows
}
fn rental_expense_rows(
    d: &FederalDraft,
    entry: &FederalIncomeEntry,
) -> Vec<crate::extended_model::RentalExpenseAmount> {
    let FederalIncome::Rental { data } = &entry.income else { unreachable!("rental source") };
    let mut rows = whole_rental_expense_rows(d, data);
    if let Some(share) = d.setup.as_ref().and_then(|setup| {
        crate::community::share(setup, entry.owner, entry.community, &entry.income)
    }) {
        for row in &mut rows {
            row.amount = share.filer_exact(&row.amount);
        }
    }
    rows
}
pub(crate) fn whole_rental_net(
    d: &FederalDraft,
    entry: &FederalIncomeEntry,
) -> Result<i64, (&'static str, &'static str)> {
    let FederalIncome::Rental { data } = &entry.income else { unreachable!("rental source") };
    rental_use_core(
        data,
        false,
        false,
        true,
        None,
        &whole_rental_expense_rows(d, data),
        Some((d, entry.id, true)),
    )
    .map(|usage| usage.net)
}
// Only taxes are entered on Rental. Linked loan facts already belong to Schedule A;
// its rental components are removed by home_mortgage_shares, never added again.
pub(crate) fn home_personal_shares(d: &FederalDraft) -> [Exact; 2] {
    let mut shares = core::array::from_fn(|_| exact::whole(0));
    for entry in &d.income {
        let FederalIncome::Rental { data } = &entry.income else {
            continue;
        };
        if matches!(data.personal_use, RentalPersonalUse::No) {
            continue;
        }
        for e in &data.expenses {
            if matches!(e.category, RentalExpenseCategory::TaxesAndLicenses) {
                shares[1] += exact::whole(e.amount) - exact::whole(rental_fraction(data, e.amount));
            }
        }
    }
    shares
}
// 8a interest,8b interest,8a points,8c points,pre-phaseout premiums,AMT vehicle adjustment.
pub(crate) fn home_mortgage_shares(d: &FederalDraft) -> [i64; 6] {
    let mut shares = [0; 6];
    let allowances = rental_loan_allowances(d);
    for e in &d.income {
        let FederalIncome::Rental { data } = &e.income else {
            continue;
        };
        for l in allowances.iter().filter(|l| data.mortgage_loans.contains(&l.canonical_id)) {
            shares[usize::from(!l.reported_on_1098)] +=
                linked_loan_share(d, data, l, l.regular_interest_cents);
            shares[2 + usize::from(!l.points_reported_on_1098)] +=
                linked_loan_share(d, data, l, l.regular_points_cents);
            shares[4] += linked_loan_share(d, data, l, l.regular_insurance_cents);
            shares[5] += linked_loan_share(
                d,
                data,
                l,
                l.regular_interest_cents - l.amt_interest_cents + l.regular_points_cents
                    - l.amt_points_cents,
            );
        }
    }
    shares
}
pub(crate) fn home_vehicle_insurance_share(d: &FederalDraft) -> i64 {
    let allowances = rental_loan_allowances(d);
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Rental { data } => Some(data),
            _ => None,
        })
        .map(|data| {
            allowances
                .iter()
                .filter(|l| data.mortgage_loans.contains(&l.canonical_id))
                .map(|l| {
                    linked_loan_share(d, data, l, l.regular_insurance_cents - l.amt_insurance_cents)
                })
                .sum::<i64>()
        })
        .sum()
}
fn home_fixed_allowed(d: &FederalDraft, id: i64, agi: [i64; 2], amt: bool) -> Option<[Exact; 3]> {
    let ded = d.deductions.as_ref()?;
    let mut total_rental_taxes = exact::whole(0);
    let mut target = core::array::from_fn(|_| exact::whole(0));
    let allowances = rental_loan_allowances(d);
    let certified = certified_loan(d);
    let separate = d.setup.as_ref()?.filing_status() == FederalStatus::Separate;
    for entry in &d.income {
        let FederalIncome::Rental { data } = &entry.income else {
            continue;
        };
        if matches!(data.personal_use, RentalPersonalUse::No) {
            continue;
        }
        for expense in &data.expenses {
            if matches!(expense.category, RentalExpenseCategory::TaxesAndLicenses) {
                let part = rental_fraction(data, expense.amount);
                total_rental_taxes += exact::whole(part);
                if entry.id == id {
                    target[1] += exact::whole(part);
                }
            }
        }
        if entry.id == id {
            for l in allowances.iter().filter(|l| data.mortgage_loans.contains(&l.canonical_id)) {
                let personal_points = if amt { l.amt_points_cents } else { l.regular_points_cents };
                let eligible_points = phase::cap(l.rental_points_cents, [personal_points]);
                target[0] += exact::whole(rental_fraction(
                    data,
                    positive(
                        (if amt { l.amt_interest_cents } else { l.regular_interest_cents })
                            - loan_credit(certified, l.canonical_id),
                    ) + eligible_points,
                ));
                target[2] += exact::whole(rental_fraction(
                    data,
                    crate::mortgage_limit::insurance_phaseout_cents(
                        if amt { l.amt_insurance_cents } else { l.regular_insurance_cents },
                        agi[0],
                        separate,
                    ),
                ));
            }
        }
    }
    let cap = exact::whole(crate::federal_rules::salt_cap_current_year(
        d.setup.as_ref()?.filing_status(),
        agi[1],
    )) * exact::whole(100);
    let personal_taxes = exact::whole(ded.state_local_taxes()) + &home_personal_shares(d)[1];
    let allowed = phase::cap(
        phase::floor(cap - personal_taxes, exact::whole(0)),
        [total_rental_taxes.clone()],
    );
    // Section 56(b)(1)(A)(ii): AMT keeps the rental tax share under
    // Worksheet 5-1 line 3, rather than treating it as line 2b.
    target[1] = if total_rental_taxes > exact::whole(0) && !amt {
        exact::rounded_exact(&(&target[1] * allowed / total_rental_taxes))
    } else {
        exact::whole(0)
    };
    Some(target)
}

// Section 469(c)(7)(A): with the election, a member sale is not a
// disposition of the entire elected activity. Read the actual direct and
// K-1 owners; other rentals and retained home years are not invented members.
#[derive(Clone)]
struct ProfessionalSource {
    id: i64,
    parts: crate::scorp::ActivityCharacters,
    income_release: [i64; 2],
}

#[derive(Clone, Default)]
struct ProfessionalGroups {
    sources: alloc::vec::Vec<ProfessionalSource>,
    magi: i64,
}

impl ProfessionalGroups {
    fn source(&self, id: i64) -> Option<&ProfessionalSource> {
        self.sources.iter().find(|source| source.id == id)
    }

    fn apply(&self, id: i64, activities: &mut [crate::k1_rental::RentalActivity], active: bool) {
        if let Some(source) = self.source(id) {
            for activity in activities.iter_mut().filter(|activity| activity.real_estate) {
                activity.grouped_former_passive(source.parts, source.income_release, active);
            }
        }
    }
}

// A wholly separate spouse source contributes no activity to this return.
// Income and property can use different fractions; Share owns that distinction.
fn professional_group_member(
    d: &FederalDraft,
    entry: &crate::FederalIncomeEntry,
    owner: Owner,
) -> bool {
    d.professional_group_owner(entry.owner) == d.professional_group_owner(owner)
        && !d
            .setup
            .as_ref()
            .and_then(|setup| {
                crate::community::share(setup, entry.owner, entry.community, &entry.income)
            })
            .is_some_and(|share| share.none())
}

// Pure source projections use the same depreciation, mortgage, basis and
// at-risk owners as the emitting pass. The elected group is transient: each
// accepted rental/K-1 history remains attached to its own source identity.
// Reg. 1.469-9(e)(1), (4) applies the former-passive limit to this combined
// activity, including current income/loss on property forms.
fn professional_former_passive(
    d: &FederalDraft,
    amt: bool,
    home_agi: [i64; 2],
) -> Result<ProfessionalGroups, crate::activity_risk::SourceIssue> {
    let mut result = ProfessionalGroups::default();
    if professional_election_rule(d).is_some() {
        return Ok(result);
    }
    let itemizing = d.deductions.as_ref().is_none_or(|ded| {
        !matches!(
            ded.choice,
            crate::DeductionSelection::Standard
                | crate::DeductionSelection::Automatic { expenses: crate::ItemizedChoice::No }
        )
    });
    let mid_quarter = crate::depreciation::mid_quarter(d);
    let allocated_entries = crate::community::entries(d);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        if d.professional_group_owner(owner) != owner {
            continue;
        }
        if !d.professional(owner).is_some_and(|status| status.single_activity) {
            continue;
        }
        let disposed = professional_group_disposed(d, owner);
        let mut rental_issue = None;
        let sources = (|| {
            let mut sources = alloc::vec::Vec::new();
            for entry in
                allocated_entries.iter().filter(|entry| professional_group_member(d, entry, owner))
            {
                let projected = match &entry.income {
                    FederalIncome::Rental { data }
                        if data.participation.professional()
                            && data.property_type.business_participation().is_none()
                            && !used_as_home(data) =>
                    {
                        let data = data.clone();
                        let whole_entry = d.income.iter().find(|source| source.id == entry.id)?;
                        let property_entry = crate::community::property_entry(d, whole_entry);
                        let FederalIncome::Rental { data: property_rental } =
                            &property_entry.income
                        else {
                            unreachable!("rental source")
                        };
                        let usage = match rental_use_core(
                            &data,
                            amt,
                            mid_quarter,
                            itemizing,
                            home_fixed_allowed(d, entry.id, home_agi, amt),
                            &rental_expense_rows(d, whole_entry),
                            Some((d, entry.id, false)),
                        ) {
                            Ok(value) => value,
                            Err((key, _)) => {
                                rental_issue = Some(crate::activity_risk::SourceIssue::new(
                                    entry.id,
                                    crate::activity_risk::ProjectionIssue::Source(key),
                                ));
                                return None;
                            }
                        };
                        let projected = crate::asset_sale::rental_sale_amount(
                            property_rental,
                            mid_quarter,
                            amt,
                        )
                        .and_then(|sale| rental_risk(&data, usage.net, sale, amt))
                        .and_then(|risk| risk.ordinary_income());
                        let ordinary = match projected {
                            Ok(value) => value,
                            Err(issue) => {
                                rental_issue =
                                    Some(crate::activity_risk::SourceIssue::new(entry.id, issue));
                                return None;
                            }
                        };
                        let prior = data.history.balance(amt);
                        let sale = match crate::asset_sale::rental_source(d, entry.id, amt) {
                            Ok(value) => value,
                            Err(issue) => {
                                rental_issue = Some(issue);
                                return None;
                            }
                        };
                        Some((
                            [
                                (dollars(ordinary) + sale.as_ref().map_or(0, |sale| sale.ordinary))
                                    * 100,
                                sale.as_ref().map_or(0, |sale| sale.section1231) * 100,
                            ],
                            [dollars(prior) * 100, 0],
                        ))
                    }
                    FederalIncome::Scorp { data }
                        if data.rental_facts.participation.professional() =>
                    {
                        let source: crate::scorp_amt::ScorpSource<'_> =
                            if amt { data.amt.as_ref()?.source(data).ok()? } else { data.into() };
                        let activity =
                            crate::community::scorp_pool(d, entry.id, &source, amt).ok()?;
                        source
                            .rental_activities(&activity.ledger, true, disposed)
                            .ok()?
                            .iter()
                            .find(|rental| rental.real_estate)
                            .map(|rental| rental.current_and_prior())
                    }
                    FederalIncome::Partnership { data }
                        if data.rental_facts.participation.professional() =>
                    {
                        let source = if amt { data.alternative.as_ref()? } else { &data.regular };
                        let activity =
                            crate::community::partner_pool(d, entry.id, data, source).ok()?;
                        data.rental_activities(source, &activity.activity.ledger, true, disposed)
                            .ok()?
                            .iter()
                            .find(|rental| rental.real_estate)
                            .map(|rental| rental.current_and_prior())
                    }
                    _ => None,
                };
                if let Some(projected) = projected {
                    sources.push((entry.id, projected));
                }
            }
            // Stable source identities, rather than labels or interview order,
            // determine the shared allocator's source-cent remainder order.
            sources.sort_by_key(|(id, _)| *id);
            Some(sources)
        })();
        if let Some(issue) = rental_issue {
            return Err(issue);
        }
        let Some(sources) = sources else { continue };
        let amounts: alloc::vec::Vec<_> = sources.iter().map(|(_, amounts)| *amounts).collect();
        let Ok(parts) =
            crate::scorp::ActivityCharacters::grouped_former_passive(&amounts, disposed)
        else {
            continue;
        };
        let Ok(former) = crate::scorp::ActivityCharacters::grouped_former_passive(&amounts, false)
        else {
            continue;
        };
        let net = parts.iter().flat_map(|parts| parts.nonpassive).sum::<i64>();
        result.magi += dollars(if disposed { net } else { positive(net) });
        result.sources.extend(sources.iter().zip(parts).zip(former).map(
            |(((id, _), parts), former)| ProfessionalSource {
                id: *id,
                parts,
                income_release: former.prior_released,
            },
        ));
    }
    Ok(result)
}

pub(crate) fn professional_group_disposed(d: &FederalDraft, owner: Owner) -> bool {
    let owner = d.professional_group_owner(owner);
    if !d.professional(owner).is_some_and(|p| p.single_activity) {
        return true;
    }
    d.income.iter().filter(|entry| professional_group_member(d, entry, owner)).all(|entry| {
        match &entry.income {
            FederalIncome::Rental { data }
                if data.property_type.business_participation().is_none() && !used_as_home(data) =>
            {
                match &data.sale {
                    crate::RentalSale::Sold { buyer, payment, .. } => {
                        *buyer == crate::Buyer::Unrelated && !payment.installment()
                    }
                    crate::RentalSale::Condemned { replacement_cost, .. } => *replacement_cost == 0,
                    crate::RentalSale::Exchanged(exchange) => exchange.releases_losses(),
                    crate::RentalSale::No => false,
                }
            }
            FederalIncome::Partnership { data } if data.regular.rentals.real_estate_present() => {
                data.disposition.ended()
                    || data.rental_facts.disposed.contains(&crate::K1RentalActivity::RealEstate)
            }
            FederalIncome::Scorp { data } if data.rentals.real_estate_present() => {
                data.disposition.ended()
                    || data.rental_facts.disposed.contains(&crate::K1RentalActivity::RealEstate)
            }
            _ => true,
        }
    })
}

pub(crate) fn professional_election_rule(d: &FederalDraft) -> Option<&'static str> {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        if !d.professional(owner).is_some_and(|p| p.single_activity) {
            continue;
        }
        let mut answers = alloc::vec::Vec::new();
        for entry in d.income.iter().filter(|e| professional_group_member(d, e, owner)) {
            match &entry.income {
                // A section 280A home year is outside section 469 (section
                // 469(j)(10)), so the election does not reach it.
                FederalIncome::Rental { data }
                    if data.property_type.business_participation().is_none()
                        && !used_as_home(data) =>
                {
                    answers.push(data.participation.professional())
                }
                FederalIncome::Partnership { data }
                    if data.regular.rentals.real_estate_present() =>
                {
                    answers.push(data.rental_facts.participation.professional())
                }
                FederalIncome::Scorp { data } if data.rentals.real_estate_present() => {
                    answers.push(data.rental_facts.participation.professional())
                }
                _ => {}
            }
        }
        if answers.iter().any(|v| *v) && answers.iter().any(|v| !*v) {
            return Some("ISSUE_PROFESSIONAL_ELECTION_ALL_RENTALS");
        }
    }
    None
}

fn rental_expense_lines(
    v: &FederalRental,
    usage: &RentalUse,
    itemizing: bool,
    fixed_allowed: Option<[Exact; 3]>,
) -> Result<[i64; RentalExpenseCategory::COUNT], (&'static str, &'static str)> {
    let mut rows = RentalExpense::category_amounts(&usage.expense_rows);
    rows[14] += exact::whole(method_amounts(v).1) + exact::whole(usage.prior_at_risk_cents);
    let project = |rows: [Exact; RentalExpenseCategory::COUNT]| {
        let mut native = [0; RentalExpenseCategory::COUNT];
        for (output, amount) in native.iter_mut().zip(rows) {
            *output = exact::rounded(&amount).ok_or_else(rental_expense_amount_issue)?;
        }
        Ok(native)
    };
    let RentalPersonalUse::Yes { fair_rental_days, personal_days, direct_expenses, .. } =
        v.personal_use
    else {
        rows[14] += exact::whole(v.home_operating_carryover) - exact::whole(usage.carry[0]);
        return project(rows);
    };
    let days = i64::from(fair_rental_days + personal_days);
    if days > 0 {
        for amount in &mut rows {
            *amount = exact::rounded_exact(
                &(&*amount * exact::ratio(i64::from(fair_rental_days), days).expect("rental days")),
            );
        }
    }
    if !usage.home {
        rows[14] += exact::whole(direct_expenses) + exact::whole(v.home_operating_carryover)
            - exact::whole(usage.carry[0]);
        return project(rows);
    }
    let fixed = if itemizing {
        fixed_allowed.unwrap_or_else(|| [rows[7].clone(), rows[12].clone(), exact::whole(0)])
    } else {
        core::array::from_fn(|_| exact::whole(0))
    };
    rows[4] -= &fixed[2];
    rows[7] -= &fixed[0];
    rows[12] -= &fixed[1];
    rows[14] += exact::whole(v.home_operating_carryover);
    let mut room = phase::floor(
        exact::whole(v.rent) - fixed.iter().cloned().sum::<Exact>() - exact::whole(direct_expenses),
        exact::whole(0),
    );
    for amount in &mut rows {
        *amount = phase::cap(amount.clone(), [room.clone()]);
        room -= &*amount;
    }
    rows[4] += &fixed[2];
    rows[7] += &fixed[0];
    rows[12] += &fixed[1];
    rows[14] += exact::whole(direct_expenses);
    project(rows)
}
