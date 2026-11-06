pub use crate::reservist_sources::ReserveTerm;
pub use crate::retirement_adjustment_sources::{IraBasis, QcdPayment, QualifiedCharity};
// Coverage: Form 1099-R conditional follow-ups use Nullable constrained choices:
//   disaster relief, public-safety insurance, Schedule R disability and box 10 basis.
//   RCalls only selects which answers are offered; existing tax and validation
//   owners retain the accepted answers. Tests: retirement/interview follow-up calls
//   and save/reopen; existing retirement_special_codes and unasked_pages amounts.
//   Sources: https://www.irs.gov/pub/irs-pdf/f1099r.pdf (2026 recipient instructions),
//   https://www.irs.gov/publications/p575 (disability, public-safety premiums,
//   recapture allocation), https://www.irs.gov/publications/p590b (disaster relief).
// Coverage: Form 1099-R — Supported (audited 2026-09-24).
//   Lines: Box 1 gross, box 4 withholding , payer or computed taxable amount (Simplified Method,
//   Form 8606 IRA basis, Roth), rollovers/conversions, 10%/25% early tax with Form 5329 exceptions, RMD shortfall,
//   Schedule R disability. Exclusions: RetirementTreatment::Unsupported (box 7a codes named) and undetermined
//   nonqualified-annuity amounts refused ("needs a payer-determined amount or supported
//   worksheet": the box 2b methods of unit B5 below are the supported worksheets).
//   Stored computation with no page (audit of 2026-10-08, tax-unreachable-stored-kinds) is now
//   written by the 1099-R page, unit B5 (2026-10-09): see the unit B5 paragraph below. NuaSale has
//   its page on Investment sales (unit B7a).
//   Refused explicitly (unit A10, from the 1099-R map, narrowed by unit B9-4 below): a Roth IRA
//   prohibited transaction (code 5 without box 7b), a reportable death benefit (code C), a
//   charitable gift annuity with a box 3 gain (code F) and the pairs left (C with D) by name
//   (a Roth SIMPLE IRA, codes J and S, is calculated since unit B11); every other box 7a code and Table 1 pair routes to a
//   calculated branch (DistributionCodes::treatments).
//   Unit B2 (2026-10-09): codes 7A and 4A (box 7b unchecked) are two calculated treatment
//   branches, NormalLumpSum and DeathLumpSum: With your other income stores what 7 or 4 stores,
//   Separately on Form 4972 stores RetirementTreatment::LumpSum (lump_sum.rs). Boxes 3, 6, 8a and
//   8b are imported (RMore) and stored only on an elected record; the rest reopen at zero, as box
//   2a does under the Simplified Method. Code A alone offers both branches (unit B9-4). Tests:
//   tests/retirement/interview.rs lump_sum_*, tests/retirement/lump_sum.rs.
//   Unit B6b (2026-10-09): box 7a code 3 also asks the plan's minimum retirement age
//   (DisabilityWages): None (reached before 2026, or an IRA), All (not reached by the end of 2026:
//   every taxable payment is wages on Form 1040 line 1h) or Part (the taxable payments through the
//   day it was reached are line 1h wages, the rest line 5b), per Pub. 525 "Disability Pensions"
//   and the 1040 instructions for line 1h. The wages part is earned income (EIC, dependent
//   standard deduction; Pub. 596) and not IRA compensation (Pub. 590-A). An IRA offers only None
//   (R_IRA_DISABILITY_NOT_WAGES) and Part stays within box 2a (R_DISABILITY_WAGES_ABOVE_TAXABLE).
//   The separate FederalIncome::DisabilityPension kind is removed. Tests: tests/income/relief_income.rs
//   fixture K, tests/retirement/retirement.rs disability_pension_*. RRB-1099-R disability amounts stay with
//   the Railroad unit.
//   Unit B5 (2026-10-09, map retirement-treatments): the ten kept RetirementTaxable kinds are reached
//   from the 1099-R page through one box 2b question (RMore.taxable_method, ten choices), whose allowed
//   set is per row (R1099.allowed_methods, supplied by DistributionCodes::methods
//   from box 7a and 7b; the singleton hides the question): Box 2a (Payer), Simplified Method
//   (PensionBasis), General Rule fixed (GeneralAnnuity) and variable (VariableAnnuity), the whole
//   contract ended (FullDischarge, which absorbs a gift of a contract issued after April 22, 1987
//   and carries the section 1041 and pre-1987 exceptions in its confirmation), a withdrawal from an
//   employer plan (QualifiedWithdrawal, section 72(e)(8)), a withdrawal, loan or pledge from a
//   commercial annuity (DeferredWithdrawal, section 72(e)(2)(B), (e)(4)(A), (e)(12)), investment
//   from before August 14, 1982 (LegacyAnnuity), a CSRS or FERS death lump sum (CivilServiceDeath,
//   Pub. 721 Worksheet D) and a CSRS or FERS child survivor annuity (ChildAnnuity). The page and
//   stored kinds merged away: AnnuityGift (into FullDischarge), AnnuityExchange (code 6, new
//   DistributionCode::Exchange: Payer with box 2a required 0 and nothing on lines 5a and 5b,
//   Pub. 575 "You need not report this on your tax return") and EmployerSecurities (NUA: box 2a as
//   the payer prints it already leaves the deferred NUA out; the include-now election stays on
//   Form 4972 only). Box 7a routing: code D printed with 1, 2, 3, 4 or 7 leads to that numeric
//   code's branch and the kind (Nonqualified, Ira, Pension) comes from box 7a and box 7b
//   (retirement_record), not from the branch; R_NONQUALIFIED_KEPT keeps a D row out of rollovers.
//   Disposition::SoldAndRolledOver writes PropertyProceeds with a rollover (Pub. 575 Paul,
//   Examples 3 and 4). Server twins: DistributionCodes::treatment_rule (R_BOX2B_METHOD,
//   R_BOX7_EXCHANGE_TAXABLE, R_EXCHANGE_OTHER_BOXES, R_NONQUALIFIED_KEPT, PROPERTY_*,
//   R_WITHDRAWAL_ABOVE_BALANCE, CSRS_RECOVERED_ABOVE_COST, R_TRUMP_*, R_INHERITED_OTHER_BOXES).
//   Cash rollovers and conversions fit box 1 on both sides; a property sale keeps its own
//   proceeds bound. Tests: validation_f08_* and f10_property_sold_and_rolled_over.
//   Trump-account and inherited-IRA source routing:
//   box 7c (R1099.trump) with code 4 writes FederalIncome::TrumpDeath and with code G
//   FederalIncome::TrumpAbleRollover (section 530A(d)(6), (d)(4); the 2026 Form 1099-R
//   instructions). Code 8 retains box 1 principal and box 7d earnings independently;
//   the existing Other taxes account owner applies the corrective-return taxes, with
//   receipt dates and eligibility answered there. Printed box 13 does not supply those dates.
//   Also,
//   code 4 with box 7b writes RetirementTaxable::Inherited, taxed with the Inherited IRA record
//   it chooses ($.inherited_iras, inherited_ira.rs; Form 8606 Part I per decedent) or, with no
//   record to use, as the payer's box 2a with a note (WARN_INHERITED_IRA_RECORD_MISSING);
//   the kind is RetirementKind::Inherited, so no own-IRA consumer sees it (F3). Tests:
//   tests/retirement/retirement_treatments.rs f1-f13 (Pub. 575, 721 and 939 worked examples), the D routing,
//   the methods per row, the RRB-1099-R, Trump and inherited branches and the catalog round trip.
//   Evidence: retirement.rs, pension.rs, ira.rs, roth.rs, rmd*.rs, annuity_tables.rs (the
//   other kinds' computations are in inherited_ira.rs, legacy_annuity.rs, lump_sum.rs); tests:
//   retirement_rollover_preserves_gross_withholding_and_taxes_only_amount_kept,
//   direct_rollover_is_not_subtracted_twice_and_ineligible_transfer_is_explicit (retirement.rs),
//   historical_simplified_method_uses_start_date_tables (health_accounts.rs),
//   single_life_general_rule_reproduces_irs_exclusion_example (general.rs); Pub 575 Worksheet A,
//   Pub 939, Pub 590-A/B, i5329, i8606
//   Line audit F10 (2026-10-10): exception 12 (a plain early distribution when the owner was 59½
//   before 2026) is a Form 5329 line 2 amount; exception 02 is RetirementException::
//   PeriodicPayments (see unit B9-4 below); a disabled Roth owner's line 25c
//   earnings and conversion recapture are Part I lines 1-2 (roth.rs disabled); a code S rollover or
//   conversion inside the SIMPLE IRA's first two years was a kept 25% distribution (unit B11: the
//   page offers Kept or a rollover into another SIMPLE IRA, SimpleDisposition); a code G box 2a above zero is taxable (the Roth target of unit B9-4
//   below, or ordinary income with another target; interview.rs distribution_treatment);
//   the first required minimum of the year the owner turns 73 is due April 1, 2027 and is no 2026
//   shortfall (rmd_minimum.rs); the $10,500 domestic-abuse and $2,600 long-term-care caps are
//   year_constants.rs constants. Not modeled: exception 14, a plain early distribution in the year
//   the owner turns 59½ keeps the 10%. Tests: tests/cross/line_audit_f10.rs.
//   Unit B9-4 (2026-10-10, map retirement-special-codes; inventor decisions of 2026-10-10: the code 8
//   excess-deferral part of box 2a is asked, SECURE 2.0 repayments cover IRA rows only, Form 4972
//   line 18 divides the person's estate-tax share by the box 9a percentage): every box 7a code and
//   Table 1 pair routes to a branch (DistributionCodes::treatments; source-only codes B, D, K, L, M
//   and Y take the branch of the code printed with them). W and 6W use the qualified care-charge
//   branch under the inventor-approved Unit37 correction (2026-10-12), independently of a plain
//   code6 exchange; the employer Roth contribution keeps its separate branch. Four new
//   branches: Corrective (code 8, RetirementTreatment::Corrective: no 10% tax, no rollover, an
//   employer plan's taxable amount less the asked 2026 excess deferral is Form 1040 line 1h wages,
//   an IRA's is line 4a/4b outside Form 8606), NotIncome (N, P, R: nothing on 2026 lines but box 1 of
//   N on line 4a, withholding credited, a note naming the statement or the earlier year),
//   NoAdditionalTax (9, E, F, U: taxable as printed, no 10%; F with a box 3 gain stays refused) and
//   ProhibitedTransaction (code 5 with box 7b: an early distribution unless the owner was 59 1/2 on
//   2026-01-01). Direct rollovers and 60-day rollovers gain the Roth destination
//   (RolloverDestination::Roth: pretax money rolled into a Roth IRA or designated Roth account is
//   taxed, code G with a taxable box 2a, code H, an in-plan Roth rollover; a designated Roth account
//   row, code B, rolls over earnings first) and the Form 5329 line 1 base keeps only the taxable
//   amount not rolled; a code B early distribution adds box 10 less box_10_basis (FederalRetirement). New
//   exceptions RetirementException::PeriodicPayments (02; a changed series is ISSUE_SEPP_RECAPTURE)
//   and ReachedAge (12). A birth or adoption, emergency, domestic-abuse or terminal-illness IRA
//   distribution carries its repayment on the page (EarlyException::Applies.repaid), kept in the
//   owner's IraBasis.special_repayments (R_REPAYMENT_NEEDS_FORM_8606). Server rules that read
//   several rows: IRA_ROLLOVER_ONCE_A_YEAR, R_EXCESS_DEFERRAL_ABOVE_W2_EXCESS,
//   QCD_SPLIT_INTEREST_ABOVE_LIMIT, DISASTER_ABOVE_QCD_REST. A QCD gift can be the one-time
//   split-interest election (QcdPayment.split_interest). Boxes 9a (every treatment) and 10 (code B)
//   are accepted. Tests: tests/retirement/retirement_special_codes.rs (the map's fifteen fixtures).
//   Unit B11 (2026-10-12, map ira-roth-histories; inventor decisions of 2026-10-11): Roth IRA rows
//   (RothWithdrawal) take a 60-day Roth-to-Roth rollover (rolled_over, outside Form 8606 line 19), any
//   IRA early-distribution exception (first home: Form 8606 line 20 for a first Roth year of 2021 or
//   earlier, else exception 09; the early_tax stage runs on retirement_rows, a Roth row's line 25c
//   plus recapture as an IRA row), a repayment of a birth or adoption, emergency, domestic-abuse or
//   terminal-illness distribution (the 2026 Line 25c Worksheet, roth.rs), a qualified disaster share
//   (line 25b, disaster.rs) and the Roth SIMPLE IRA 25% rate (J printed with S); an employer's Roth SEP
//   or Roth SIMPLE contribution is RetirementTreatment::EmployerRothContribution (taxable, no
//   additional tax, outside Form 8606 Part I, a 2026 conversion). A code S row offers Kept or a
//   rollover into another SIMPLE IRA. Form 8606: line 2 adjustment (IraBasis.adjustment), reservist
//   repayments on the item (ReservistRepayment, no label or confirmation), the date-of-death value on
//   a final return, QCD capped at the pre-tax pool (already in qualified_charity), Social Security
//   IRA MAGI from all benefits under the regular method. Form 5329: a second exception on one
//   distribution (99), line 10 absorbs a carried excess into the deduction and Form 8606 line 1
//   (ira::absorbed_traditional), line 12 returned prior excess (ira::exclude_returned_prior_excess,
//   outside line 4b and Form 8606 line 7), a 529-to-Roth transfer above the year's room is a Roth excess
//   and a taxable 529 withdrawal (ira::qtp_excess_by_entry), the long-term-care cap across plans. A
//   2027 return of a 2026 contribution is ReturnedContribution on the IRA page. Removed refusals and
//   notices: the recharacterization record, returned contributions, Roth history specials, the four
//   RMD confirmations and the Roth, SIMPLE and lump-sum refusals. Tests: tests/retirement/ira_roth_histories.rs
//   (F1-F20).
//   Unit M3-P4 (2026-09-25): the interview stores unsupported box 7 codes as
//   RetirementTreatment::Unsupported (then with an OtherDistributionCode) refused by name;
//   the UnusualRetirementCodes situation is gone (tests/retirement/retirement.rs
//   retirement_unmodeled_codes_are_refused_by_name).
//   Unit u6-roth (2026-09-28): codes J, Q and T leave OtherDistributionCode for their own box 7
//   branch onto the Roth income kind (roth.rs Coverage); the B, N, P, R and 8 refusals now name
//   their form lines.
//   Box 7a as printed (2026-09-28): the 1099-R's printed codes are the Bitflags field
//   R1099.box7 (catalog DistributionCodes, every Table 1 code 1-9, A-H, J-N, P-U, W, Y), stored
//   on FederalRetirement.box7; OtherDistributionCode (with TwoCodes) is gone and
//   RetirementTreatment::Unsupported carries no payload: DistributionCodes::refusal names the
//   record's codes (single-code texts kept; 6 added; 8J, PJ, SJ keep the Roth texts; any other
//   pair "codes X and Y printed together"). Rules on both sides (DistributionCodes::rule and
//   R1099's ValidationRules): R_BOX7_NO_CODE, R_BOX7_TOO_MANY_CODES, R_BOX7_CODES_NOT_COMBINED
//   (Table 1's 47 pairs; Y never alone), R_BOX7B_NOT_IRA (D, N, R), R_BOX7B_NEEDED (K, S) and
//   R_ROTH_CODE_NOT_BOX7. The treatment branch is a per-row calculated allowed set:
//   R1099.allowed_treatments (hidden, read-only) constrains `code` by the parent-relative
//   ConstrainedBy "^.allowed_treatments"; the topic catalog uses DistributionCodes::treatments
//   (one discriminant: 7 Normal, 1/2
//   Early, 3 Disability, 4 without box 7b Death, G DirectRollover, S EarlySimple, D
//   Nonqualified, Q/J/T RothIra, else Other). An unanswered `code` stays None.
//   The page builders fill the set on reopen; the save
//   refuses a branch outside it (R_BOX7_TREATMENT, server only). STORAGE_FORMAT 74 kept (1099-R
//   entries cleared before testing). Tests: tests/retirement/interview.rs box_7a_*,
//   reopened_1099_r_rows_carry_their_own_treatment_sets; tests/retirement/roth_pages.rs
//   rare_roth_situations_are_refused_by_name.
// Coverage: Form RRB-1099-R — Supported (unit B5, 2026-10-09). Its own income document
//   (RetirementDocument::Rrb1099R, owned like the 1099-R) onto the pension record of the Railroad
//   kind: boxes 4, 5, 6, 7, 8 and 9 are imported (box 7 is the gross, box 7a code 7 "if you are
//   asked for a distribution code", Pub. 575 Railroad Retirement Benefits); boxes 10 and 11
//   (nonresident alien) are an import error and box 3 is the Simplified Method cost the filer
//   enters once. Asked: whose annuity it is (RailroadRecipient: own employee annuity, spouse or
//   divorced-spouse annuity, survivor annuity with the date of death), the employee-contribution
//   recovery (RailroadRecovery: none, or the Simplified Method on box 4 only, section 72(r)) and
//   box 8 by the year the benefits were paid (RailroadRepayment, into the claim-of-right
//   comparison). Boxes 5 and 6 are fully taxable. The existing server gate stands: an employee or
//   spouse annuitant under 60 is refused by name, a survivor needs the death exception. The stored
//   Railroad record keeps boxes 5 and 6 apart and whose annuity it is (RailroadHolder, with the
//   survivor's date of death); the computation reads their sum with box 4, so the page reopens
//   exactly as entered. Rules both sides: RRB_BOX7_NOT_SUM,
//   RRB_REPAYMENTS_WITHOUT_BOX_8, RRB_BOX_8_NOT_SPLIT, R_WITHHOLDING_ABOVE_GROSS; server only
//   RRB_REPAYMENTS_NOT_BOX_8. Not calculated: a disability annuity before age 60 (refused by name),
//   tier 1 Social Security equivalent benefits (an SSA-1099). Tests: tests/retirement/retirement_treatments.rs
//   f9_rrb_1099_r_own_spouse_and_survivor_annuities, rrb_1099_r_rules_reject_on_the_server_and_
//   import_reads_boxes_4_to_9; Pub. 575 (2026 draft) Railroad Retirement Benefits and Simplified
//   Method Worksheet A, 26 U.S.C. 72(r).
// Coverage: Pub. 575, 939 and 721 annuity worksheets — Supported through the 1099-R box 2b question
//   (unit B5, 2026-10-09): the Simplified Method (Worksheet A, Tables 1 and 2), the General Rule
//   for fixed and variable payments (Pub. 939 Tables V-VIII; refigure of a shortfall under
//   Treas. Reg. 1.72-4(d)(3)), the whole contract ended (section 72(e)(5)(E), the gift of a
//   contract under section 72(e)(4)(C)), a withdrawal from an employer plan (section 72(e)(8),
//   Pub. 575 Ann Brown and Ryan), a withdrawal, loan or pledge from a commercial annuity
//   (section 72(e)(2)(B), (e)(4)(A), (e)(12); Pub. 575 example), investment from before August 14,
//   1982 (section 72(e)(5)(B), (q)(2)(F)), a CSRS or FERS death lump sum (Pub. 721 Worksheet D)
//   and a CSRS or FERS child survivor annuity (Pub. 721 "Surviving child only"); distributed
//   property sold and rolled over (section 402(c)(6), Pub. 575 Paul, Examples 3 and 4). Not
//   calculated: the sex-based Tables I-IV, an annuity that started before July 2, 1986, exchanges
//   outside code 6 (the insurers' records for later years), the NUA include-now election outside
//   Form 4972. Tests: tests/retirement/retirement_treatments.rs f1 to f13 with their arithmetic.
// Coverage: Form 8606 — Supported (audited 2026-09-24).
//   Lines: Part I lines 1–15c: nondeductible contributions and reservist repayments, incoming
//   after-tax plan basis, line 6 value with outstanding rollovers, pro-rata recovery, carryforward,
//   line 15c worksheet. Part II lines 16–18 conversions. Part III Roth distributions: regular,
//   oldest conversion, then earnings; five-year/qualification; conversion recapture. Stored
//   computation: the separate per-decedent inherited 8606 (inherited_ira.rs; the Inherited IRA
//   record of the Retirement page, traditional or Roth, taxes the 1099-R payments that choose it,
//   unit B5). Refused: unconfirmed conversion sources, missing Roth
//   history/ordering.
//   Unit6 PRINT (2026-10-13): per-filer PartsI/II project the existing
//   basis owner and final disaster/repayment outputs. Reservist repayments
//   stay on line1; the retained Pub590-B worksheet skips6–12; a balance alone
//   and fully repaid-only special payments create no form. Line10's displayed
//   four-place ratio never replaces the owner's exact fraction/rounding.
//   Tests: print_traditional_ira_*, existing IRA controls and corpus footings.
//   Unit6 scoped correction (dispatcher27e0864, 2026-10-13): absorbed prior
//   excess is available current-year contribution basis, never next-year timing.
//   Shared QCD/ordinary helper consumes IraResult.absorbed; original Worksheet1-1
//   preliminary allocation stays owned by its actual contribution inputs.
//   Tests: absorbed_prior_ira_excess_* (both reported payment-year witnesses).
//   u6-roth (2026-09-28): Part III is reached through the interview (1099-R codes J and T and
//   the Retirement topic's Roth IRA history kind); see roth.rs Coverage.
//   Evidence: retirement.rs, roth.rs, inherited_ira.rs, ira_repayment.rs, reservist.rs,
//   federal_rules.rs; tests/retirement/passthrough.rs
//   current_ira_insufficient_basis_uses_form_8606_after_fixing_deduction,
//   partial_roth_conversion_allocates_basis_and_taxes_only_kept_money_as_early,
//   inherited_ira_form8606_rounding_and_empty_balance_preserve_unused_basis; tests/retirement/retirement.rs
//   roth_withdrawal_uses_regular_then_oldest_conversion_then_earnings; sources i8606, Pub 590-B.
//   T2-fx (2026-09-27): line 1 holds only permitted contributions. ira.rs calculate puts
//   min(traditional, the age/compensation limit) less the deduction there (Pub 590-A 2025
//   Worksheet 1-2 line 6 "don't enter more than $7,000 ($8,000 if you are age 50 or older)" and
//   line 8 "Subtract line 7 from line 5 or line 6, whichever is smaller. Enter the result here
//   and on line 1 of your Form 8606"; i8606 2025 line 1 "you can make nondeductible contributions
//   up to your contribution limit"); an excess left in the IRA is Form 5329 line 15, and i5329
//   line 10 moves a later-absorbed excess onto that later year's IRA Deduction Worksheet line 11.
//   Previously traditional less the deduction, which counted the excess as basis. No stored type
//   change. Tests: tests/retirement/retirement.rs form_8606_line_1_leaves_out_a_retained_traditional_excess,
//   form_8606_line_1_all_nondeductible_stops_at_the_limit,
//   form_8606_line_1_with_a_phased_deduction_and_an_excess. Sources i8606 (2025, irs.gov html),
//   p590a.pdf and i5329.pdf (2025), fetched 2026-09-27.
// Coverage: Form 5329 — Supported (audited 2026-09-24; unit T2-42, 2026-09-26).
//   Lines: Part I 10%/25%-SIMPLE early-distribution tax per spouse with exceptions; Part III lines 9–11 and 13–17 and Part IV lines 18–25 (ira.rs excess_tax: the 2026
//   excess is the contribution above the App's own traditional limit or validate_roth's Roth
//   limit, taxed at 6% of the smaller of the total excess or the owner's December 31, 2026 value
//   from the IraExcess record, whose traditional value is the owner's Form 8606 year-end value
//   when a Form 8606 is entered (T2-cx C18, kept in step by the interview); one carried-in 2025 line 16/24 amount per owner is reduced by
//   unused 2026 room, by that owner's taxable traditional IRA distributions (line 11) and by
//   gross Roth withdrawals with the entire-balance floor (line 20)); Part VI Archer MSA; Part VII
//   HSA; Part VIII ABLE excess 6%; Part IX RMD shortfall 25%/10% with correction window and
//   waiver; 2026-draft Trump-account lines 57–63. Part II lines 5–8 per filer (unit B3 Form 1099-Q,
//   2026-10-09; account_additional_tax): line 5 the filer's taxable ABLE earnings plus the taxable
//   earnings of the filer's Forms 1099-Q (education_account.rs), line 6 the ABLE distributions not
//   penalized (post-death payments, corrective returns) plus the education exceptions (death or
//   disability, a timely returned Coverdell excess, expenses used only for an education credit,
//   tax-free scholarships and military academy costs), line 7 their difference and line 8 its 10%,
//   source cents added before each line rounds, as Part I does (it replaces the former
//   able_additional_tax, which rounded the penalized base alone). The ABLE half is the Form
//   1099-QA document (able.rs, unit B6a); both halves are on the Prizes, scholarships, relief
//   payments, account payouts and other income page.
//   Refused by name (T2-42): an excess with no IraExcess record (Part III line 15 / Part IV line
//   23, "enter the December 31, 2026 IRA values"); Part III line 10 absorption of a carried
//   traditional excess (the IRA Deduction Worksheet line 11 deduction and Form 8606 basis change
//   are not calculated); line 12 (2026 return of a pre-2025 excess, not entered, confirmed absent
//   in the record); an excess or carry-in alongside a 529-to-Roth rollover (shared annual room);
//   Part V contribution excess and carry are owned by coverdell.rs; Part VIII by able.rs.
//   Their own coverage blocks own the current boundaries and fixtures. Distribution tax
//   remains Part II above.
//   Unit u5-adjust (2026-09-28): Part IX is asked on the Retirement topic's "Missed required
//   minimum distribution" kind (MissedDistribution onto RequiredDistribution, by owner; spouse
//   only jointly), verified against i5329 (2025) lines 52a–55 and the correction window (earliest
//   of notice, assessment and December 31, 2028). Tests: tests/retirement/unasked_pages.rs
//   missed_required_distribution_through_the_retirement_topic_by_hand (fixture),
//   missed_distribution_correction_window_ends_december_31_2028,
//   missed_distribution_waiver_and_owner_rules.
//   Evidence: retirement.rs (early_tax), ira.rs (excess_tax, validate_roth limits), rmd.rs,
//   hsa.rs, archer_contributions.rs, able.rs, trump_account.rs; tests/retirement/retirement.rs
//   early_distribution_rounds_form_lines_and_each_filer_tax,
//   rmd_uncorrected_shortfall_adds_tax_without_creating_income,
//   trump_account_excess_and_returned_earnings_use_separate_form_5329_taxes,
//   form_5329_parts_iii_and_iv_tax_an_uncorrected_ira_excess_by_hand (fixture),
//   form_5329_line_17_is_capped_by_the_year_end_traditional_value,
//   T2-cy: line 17's traditional value from a Form 8606 adds paid_next_year (2026 contributions
//   made in 2027); tests/retirement/interview.rs form_5329_line_17_value_adds_2026_contributions_made_in_2027,
//   form_5329_excess_follows_the_age_50_contribution_limit,
//   form_5329_roth_excess_follows_the_single_magi_phase_out_end,
//   form_5329_carried_traditional_excess_is_reduced_by_2026_withdrawals,
//   form_5329_carried_roth_excess_counts_a_full_withdrawal_at_least_at_the_carried_amount;
//   tests/retirement/interview.rs ira_page_excess_fold_round_trips_to_the_excess_record;
//   tests/retirement/health_accounts.rs
//   uncorrected_personal_hsa_excess_reduces_deduction_and_creates_f5329_tax; sources i5329,
//   f5329--dft, i5329--dft (fetched from irs.gov 2026-09-26; not on disk). STORAGE_FORMAT 50.
//   C-fixture (2026-09-25, tests/retirement/retirement.rs): pension_and_ira_distributions_match_hand_computation
//   (lines 4a, 4b, 5a, 5b, 25b and the return by hand), early_ira_distribution_adds_the_ten_percent_tax_on_schedule_2,
//   direct_rollover_is_reported_on_5a_only, ira_basis_taxes_only_the_pro_rata_part,
//   qualified_charitable_distribution_reduces_4b_only.

// Form 5329 exception amounts are derived from distribution facts, never entered as tax results.
use crate::dates::{source_valid_date as valid_date, valid_birth};
use crate::kernel::ratio::exact;
use crate::{DistributionCode, DistributionCodes, FederalRetirement};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Printed relationships are shared by paper admission and the retirement topic.
struct DistributionBoxes {
    gross: i64,
    taxable: i64,
    withholding: i64,
    box7: u32,
    ira: bool,
    trump: bool,
    box_3: i64,
    box_6: i64,
    box_8a: i64,
    box_8b_percent: Option<f64>,
    box_9a_percent: Option<f64>,
    box_10: i64,
}

impl From<&crate::R1099> for DistributionBoxes {
    fn from(v: &crate::R1099) -> Self {
        Self {
            gross: v.gross,
            taxable: v.taxable.unwrap_or(0),
            withholding: v.withholding.unwrap_or(0),
            box7: v.box7,
            ira: v.ira,
            trump: v.trump,
            box_3: v.box_3.unwrap_or(0),
            box_6: v.box_6.unwrap_or(0),
            box_8a: v.box_8a.unwrap_or(0),
            box_8b_percent: v.box_8b_percent,
            box_9a_percent: v.box_9a_percent,
            box_10: v.box_10.unwrap_or(0),
        }
    }
}

impl From<&crate::Form1099RPaper> for DistributionBoxes {
    fn from(v: &crate::Form1099RPaper) -> Self {
        Self {
            gross: v.gross,
            taxable: v.taxable.unwrap_or(0),
            withholding: v.withholding.unwrap_or(0),
            box7: v.box7,
            ira: v.ira,
            trump: v.trump,
            box_3: v.box_3.unwrap_or(0),
            box_6: v.box_6.unwrap_or(0),
            box_8a: v.box_8a.unwrap_or(0),
            box_8b_percent: v.box_8b_percent,
            box_9a_percent: v.box_9a_percent,
            box_10: v.box_10.unwrap_or(0),
        }
    }
}

impl DistributionBoxes {
    fn source_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        DistributionCodes::issue(self.box7, self.ira)
            .or_else(|| self.care_charge_issue())
            .or_else(|| {
                (self.taxable > self.gross)
                    .then(|| ValidationIssue::new("R_TAXABLE_ABOVE_GROSS").at_field("taxable"))
            })
            .or_else(|| {
                (self.withholding > self.gross).then(|| {
                    ValidationIssue::new("R_WITHHOLDING_ABOVE_GROSS").at_field("withholding")
                })
            })
            .or_else(|| {
                (self.box_10 > self.gross)
                    .then(|| ValidationIssue::new("R_BOX10_ABOVE_GROSS").at_field("box_10"))
            })
            .or_else(|| {
                (self.box_3 > self.taxable)
                    .then(|| ValidationIssue::new("R_BOX3_ABOVE_TAXABLE").at_field("box_3"))
            })
            .or_else(|| {
                (self.box_8b_percent.is_some() && self.box_8a <= 0).then(|| {
                    ValidationIssue::new("R_BOX8B_WITHOUT_BOX8A").at_field("box_8b_percent")
                })
            })
    }

    fn care_charge_issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.box7 & DistributionCodes::CODE_W.0 == 0 {
            return None;
        }
        let issue = crate::validation::ValidationIssue::new("R_CARE_CHARGE_SOURCE_INVALID");
        if !(self.box7 == DistributionCodes::CODE_W.0
            || self.box7 == DistributionCodes::CODE_W.0 | DistributionCodes::CODE_6.0)
        {
            Some(issue.at_field("box7"))
        } else if self.ira {
            Some(issue.at_field("ira"))
        } else if self.trump {
            Some(issue.at_field("trump"))
        } else if self.taxable != 0 {
            Some(issue.at_field("taxable"))
        } else if !(0..=self.gross).contains(&self.box_8a) {
            Some(issue.at_field("box_8a"))
        } else if self.box_3 != 0 {
            Some(issue.at_field("box_3"))
        } else if self.box_6 != 0 {
            Some(issue.at_field("box_6"))
        } else if self.box_8b_percent.is_some() {
            Some(issue.at_field("box_8b_percent"))
        } else if self.box_9a_percent.is_some() {
            Some(issue.at_field("box_9a_percent"))
        } else if self.box_10 != 0 {
            Some(issue.at_field("box_10"))
        } else {
            None
        }
    }
}

pub(crate) fn source_issue(v: &crate::R1099) -> Option<crate::validation::ValidationIssue> {
    DistributionBoxes::from(v).source_issue()
}

pub(crate) fn paper_source_issue(
    v: &crate::Form1099RPaper,
) -> Option<crate::validation::ValidationIssue> {
    DistributionBoxes::from(v).source_issue()
}

pub(crate) fn input_issue(
    v: &crate::R1099Answered,
    require_answers: bool,
) -> Option<crate::validation::ValidationIssue> {
    source_issue(v).or_else(|| {
        if v.code.is_none() {
            return require_answers.then(|| {
                crate::validation::ValidationIssue::new("R_TREATMENT_NEEDED").at_field("code")
            });
        }
        if v.taxable_method.is_none() {
            return require_answers.then(|| {
                crate::validation::ValidationIssue::new("R_TAXABLE_METHOD_NEEDED")
                    .at_field("taxable_method")
            });
        }
        if require_answers
            && matches!(v.code, Some(crate::DistributionCode::TrumpDeath { basis: None, .. }))
        {
            return Some(
                crate::validation::ValidationIssue::new("R_TRUMP_BASIS_NEEDED")
                    .at_field("basis")
                    .in_field("code"),
            );
        }
        document_issue(v)
    })
}

pub(crate) fn document_issue(
    v: &crate::R1099Answered,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    if let Some(issue) = DistributionCodes::issue(v.box7, v.ira) {
        return Some(issue);
    }
    if v.code.as_ref().is_none_or(|code| !DistributionCodes::leads_to(v.box7, v.ira, v.trump, code))
    {
        return Some(ValidationIssue::new("R_BOX7_TREATMENT").at_field("code"));
    }
    if let Some(issue) = DistributionCodes::treatment_issue(v) {
        return Some(issue);
    }
    if let Some(DistributionCode::RothIra { roth, .. }) = &v.code
        && !DistributionCodes::leads_to_roth(v.box7, roth)
    {
        return Some(
            ValidationIssue::new("R_ROTH_CODE_NOT_BOX7").at_field("roth").in_field("code"),
        );
    }
    if let Some(DistributionCode::Disability { wages, .. }) = &v.code {
        if v.ira && !matches!(wages, crate::DisabilityWages::None) {
            return Some(
                ValidationIssue::new("R_IRA_DISABILITY_NOT_WAGES")
                    .at_field("wages")
                    .in_field("code"),
            );
        }
        if matches!(wages, crate::DisabilityWages::Part { amount } if *amount > v.taxable.unwrap_or(0))
        {
            return Some(
                ValidationIssue::new("R_DISABILITY_WAGES_ABOVE_TAXABLE")
                    .at_field("amount")
                    .in_field("wages")
                    .in_field("code"),
            );
        }
    }
    if v.taxable.unwrap_or(0) > v.gross {
        return Some(ValidationIssue::new("R_TAXABLE_ABOVE_GROSS").at_field("taxable"));
    }
    if let Some(p) = &v.after_death {
        let field = if p.gross > v.gross {
            Some("gross")
        } else if p.taxable > v.taxable.unwrap_or(0) || p.taxable > p.gross {
            Some("taxable")
        } else {
            None
        };
        if let Some(field) = field {
            return Some(
                ValidationIssue::new("AFTER_DEATH_ABOVE_BOX")
                    .at_field(field)
                    .in_field("after_death"),
            );
        }
    }
    if v.withholding.unwrap_or(0) > v.gross {
        return Some(ValidationIssue::new("R_WITHHOLDING_ABOVE_GROSS").at_field("withholding"));
    }
    if let Some(issue) = v.disaster().issue(v.gross) {
        return Some(issue.in_field("disaster"));
    }
    if let Some(issue) = v.disaster().code_issue(v) {
        return Some(issue);
    }
    if let Some(issue) = crate::lump_sum::page_issue(v) {
        return Some(issue);
    }
    if let Some(issue) = crate::public_safety_source::paid_issue(
        v.public_safety(),
        v.ira || matches!(v.code, Some(DistributionCode::RothIra { .. })),
        v.gross,
    ) {
        return Some(issue);
    }
    if let Some(DistributionCode::RothIra { roth, .. }) = &v.code {
        if let Some(issue) = roth_other_boxes_issue(v) {
            return Some(issue);
        }
        return roth.issue().map(|issue| issue.in_field("roth").in_field("code"));
    }
    None
}

pub(crate) fn roth_other_boxes_issue(
    v: &crate::R1099Source,
) -> Option<crate::validation::ValidationIssue> {
    // Informational box 9a is accepted with every treatment. Keep the original
    // whole-record predicate; the branches below identify its actual input.
    let box_two_a = matches!(v.taxable_method, Some(crate::TaxableMethod::BoxTwoA));
    let other = v.box_3.unwrap_or(0) != 0
        || v.box_6.unwrap_or(0) != 0
        || v.box_8a.unwrap_or(0) != 0
        || v.box_8b_percent.is_some()
        || v.box_9b.is_some()
        || v.box_10.unwrap_or(0) != 0
        || v.box_10_basis.is_some()
        || v.disability.is_some();
    if box_two_a && !other {
        return None;
    }
    let field = if !box_two_a {
        "taxable_method"
    } else if v.box_3.unwrap_or(0) != 0 {
        "box_3"
    } else if v.box_6.unwrap_or(0) != 0 {
        "box_6"
    } else if v.box_8a.unwrap_or(0) != 0 {
        "box_8a"
    } else if v.box_8b_percent.is_some() {
        "box_8b_percent"
    } else if v.box_9b.is_some() {
        "box_9b"
    } else if v.box_10.unwrap_or(0) != 0 {
        "box_10"
    } else {
        "disability"
    };
    Some(crate::validation::ValidationIssue::new("R_ROTH_OTHER_BOXES").at_field(field))
}

// Form 1099-R recipient instructions (2026), box 7a; Pub. 575,
// Disability Pensions, Insurance Premiums for Retired Public Safety Officers,
// and Recapture Allocation Chart; Pub. 590-B, Disaster-Related Relief.
// These are calls for answers, not eligibility decisions. The saved answers
// still pass through their existing validation and tax owners.
impl crate::RCalls {
    pub(crate) fn for_boxes(box7: u32, ira: bool, trump: bool, box_10: i64) -> Self {
        let has = |codes: &str| codes.chars().any(|c| box7 & DistributionCodes::of(c) != 0);
        let open = |called, choices: &[i32]| if called { choices.to_vec() } else { Vec::new() };
        let treatments = DistributionCodes::treatments(box7, ira, trump);
        // These treatments require no disaster or public-safety election in
        // treatment_issue. Keep an answer if any offered treatment permits it,
        // including the ordinary/inherited IRA alternatives for code A.
        let no_other_answers = [
            libertas_macros::variant_index!(crate::DistributionCode::Exchange) as i32,
            libertas_macros::variant_index!(crate::DistributionCode::Inherited) as i32,
            libertas_macros::variant_index!(crate::DistributionCode::Corrective) as i32,
            libertas_macros::variant_index!(crate::DistributionCode::NotIncome) as i32,
            libertas_macros::variant_index!(crate::DistributionCode::NoAdditionalTax) as i32,
            libertas_macros::variant_index!(crate::DistributionCode::QualifiedCareCharge) as i32,
            libertas_macros::variant_index!(crate::DistributionCode::EmployerRothContribution)
                as i32,
        ];
        let permits_other_answers = treatments.iter().any(|t| !no_other_answers.contains(t));
        let roth_only = treatments.iter().all(|t| {
            *t == libertas_macros::variant_index!(crate::DistributionCode::RothIra) as i32
        });
        Self {
            // Nonqualified annuities, exchanges, direct rollovers, corrective
            // payments and insurance charges are not disaster withdrawals.
            // A Roth IRA supports the Qualified disaster answer alone.
            disaster: open(
                !trump && !has("DF6NRWGH98PEU") && permits_other_answers,
                if roth_only { &[1] } else { &[1, 2, 3] },
            ),
            // Box 7b and Roth IRA codes identify IRAs; D/F identify annuities
            // rather than the officer's governmental employer plan.
            public_safety: open(!trump && !ira && !has("DFJQT") && permits_other_answers, &[0]),
            // Code 3 identifies disability. Code 7 can also report disability
            // pensions after minimum retirement age; boxes cannot decide the
            // separate Schedule R mandatory-retirement-age test.
            disability: open(!trump && !ira && has("37"), &[1]),
            // Only a designated Roth account's positive box 10 calls for the
            // recapture allocation. Known zero basis remains a real answer.
            box_10_basis: open(!trump && has("B") && box_10 > 0, &[0]),
        }
    }
}

// Codes that cannot come from an IRA, so box 7b is unchecked with them: the
// insurance and annuity codes (6, 9, C, D, F, W), the plan loan and Roth
// account codes (B, H, L, M, U) and the recharacterizations (N, R).
const PLAN_OR_UNCHECKED: &str = "69BCDFHLMNRUW";

// Form 1099-R box 7a (2026 Instructions for Forms 1099-R and 5498, box 7a
// and Table 1, Guide to Distribution Codes): each printed code with the
// codes Table 1's "Used with code" column lets it share one form (the
// column is symmetric), in the catalog's bit order. The R1099 page's
// ValidationRules spell the same rules over the numeric masks; these are
// their server twins, and the Retirement topic stores the printed codes on
// `FederalRetirement.box7`.
const BOX_7A: [(char, &str); 30] = [
    ('1', "8BDKLMP"),
    ('2', "8BDKLMP"),
    ('3', "D"),
    ('4', "8ABDGHKLMPY"),
    ('5', ""),
    ('6', "W"),
    ('7', "ABDKLMY"),
    ('8', "124BJK"),
    ('9', ""),
    ('A', "47"),
    ('B', "12478GLMPU"),
    ('C', "D"),
    ('D', "12347C"),
    ('E', ""),
    ('F', ""),
    ('G', "4BK"),
    ('H', "4"),
    ('J', "8PS"),
    ('K', "12478GY"),
    ('L', "1247B"),
    ('M', "1247B"),
    ('N', ""),
    ('P', "124BJ"),
    ('Q', ""),
    ('R', ""),
    ('S', "J"),
    ('T', ""),
    ('U', "B"),
    ('W', "6"),
    ('Y', "47K"),
];

impl DistributionCodes {
    // Every declared code.
    const ALL: u32 = (1 << BOX_7A.len()) - 1;
    // The printed code's catalog bit.
    pub(crate) fn of(code: char) -> u32 {
        BOX_7A.iter().position(|(c, _)| *c == code).map_or(0, |i| 1 << i)
    }
    fn partners(code: char) -> u32 {
        BOX_7A
            .iter()
            .find(|(c, _)| *c == code)
            .map_or(0, |(_, with)| with.chars().map(Self::of).sum())
    }
    // The printed codes, in catalog order.
    fn printed(box7: u32) -> Vec<char> {
        BOX_7A
            .iter()
            .enumerate()
            .filter(|(i, _)| box7 & (1 << i) != 0)
            .map(|(_, (c, _))| *c)
            .collect()
    }
    // The client rules on box 7a and box 7b, first failure first: a code
    // is printed, at most two ("a maximum of two alphanumeric codes"), two
    // only as Table 1 combines them and Y never alone ("you must use code Y
    // first with either code 4, 7, or K"); box 7b is not checked for a
    // recharacterization (N, R), a nonqualified annuity (D, a plan not
    // described in section 408 or 408A) or a code only a plan or a contract
    // can print (PLAN_OR_UNCHECKED), and is checked for traditional IRA
    // assets (K), a SIMPLE IRA (S) and a QCD (Y, "you must use code Y first
    // with either code 4, 7, or K").
    #[cfg(test)]
    pub(crate) fn rule(box7: u32, ira: bool) -> Option<&'static str> {
        Self::issue(box7, ira).map(|issue| issue.key)
    }
    pub(crate) fn issue(box7: u32, ira: bool) -> Option<crate::validation::ValidationIssue> {
        let codes = Self::printed(box7);
        let has = |code: char| box7 & Self::of(code) != 0;
        let (key, field) = if box7 == 0 {
            ("R_BOX7_NO_CODE", "box7")
        } else if box7.count_ones() > 2 {
            ("R_BOX7_TOO_MANY_CODES", "box7")
        } else if box7 & !Self::ALL != 0
            || box7 == Self::CODE_Y.0
            || (codes.len() == 2 && Self::partners(codes[0]) & Self::of(codes[1]) == 0)
        {
            ("R_BOX7_CODES_NOT_COMBINED", "box7")
        } else if ira && PLAN_OR_UNCHECKED.chars().any(&has) {
            ("R_BOX7B_NOT_IRA", "ira")
        } else if !ira && (has('K') || has('S') || has('Y')) {
            ("R_BOX7B_NEEDED", "ira")
        } else {
            return None;
        };
        Some(crate::validation::ValidationIssue::new(key).at_field(field))
    }

    // Whether the printed codes lead to this treatment branch: each branch
    // the App calculates takes the printed codes it names (4 with box 7b is
    // an inherited IRA, 4 with box 7c a Trump account), and Another code
    // takes every other code and every combination. R_BOX7_TREATMENT on
    // both sides.
    pub(crate) fn leads_to(box7: u32, ira: bool, trump: bool, code: &DistributionCode) -> bool {
        Self::treatments(box7, ira, trump).contains(&(Self::branch(code) as i32))
    }
    // The DistributionCode discriminant, in declaration order.
    fn branch(code: &DistributionCode) -> usize {
        match code {
            DistributionCode::Normal { .. } => 0,
            DistributionCode::Early { .. } => 1,
            DistributionCode::Disability { .. } => 2,
            DistributionCode::Death { .. } => 3,
            DistributionCode::DirectRollover { .. } => 4,
            DistributionCode::EarlySimple { .. } => 5,
            DistributionCode::Nonqualified => 6,
            DistributionCode::RothIra { .. } => 7,
            DistributionCode::Other => 8,
            DistributionCode::NormalLumpSum { .. } => 9,
            DistributionCode::DeathLumpSum { .. } => 10,
            DistributionCode::Exchange => 11,
            DistributionCode::Inherited { .. } => 12,
            DistributionCode::TrumpDeath { .. } => 13,
            DistributionCode::TrumpToAble { .. } => 14,
            DistributionCode::Corrective { .. } => 15,
            DistributionCode::NotIncome => 16,
            DistributionCode::NoAdditionalTax => 17,
            DistributionCode::ProhibitedTransaction { .. } => 18,
            DistributionCode::QualifiedCareCharge => 19,
            DistributionCode::EmployerRothContribution => 20,
        }
    }
    // The R1099's allowed treatments, supplied by the topic catalog: the
    // branch the printed codes lead to (Table 1 in the 2026 Instructions for
    // Forms 1099-R and 5498, "Used with code"). Code A alone is the one row
    // with two: the 10-year tax option of a plan the filer owns or of a
    // decedent's (or, with box 7b, an ordinary or an inherited IRA, which
    // cannot use Form 4972). An alpha code that only describes the source
    // (B designated Roth account, D annuity, K assets without a market value,
    // L loan, M loan offset, Y QCD) takes the branch of the numeric or other
    // code it is printed with, and alone takes Normal ("Generally, use code 7
    // if no other code applies"; D alone is the nonqualified annuity); the
    // codes with a branch of their own are routed first. The row's kind comes
    // from box 7a and box 7b (`FederalRetirement.kind`), not from the branch.
    pub(crate) fn treatments(box7: u32, ira: bool, trump: bool) -> Vec<i32> {
        let has = |c: char| box7 & Self::of(c) != 0;
        let is = |c: char| box7 == Self::of(c);
        let one = |branch: i32| alloc::vec![branch];
        if trump && !ira && is('4') {
            return one(13);
        }
        if trump && !ira && is('G') {
            return one(14);
        }
        if trump && !ira && is('8') {
            return one(15);
        }
        if trump {
            return one(8);
        }
        // Refused by name: a reportable death benefit after a policy sale (C) and a
        // prohibited transaction in a Roth IRA. J printed with S is a Roth SIMPLE IRA inside
        // its first two years: the Roth branch.
        if has('C') || (has('5') && !ira) {
            return one(8);
        }
        if has('J') && has('S') {
            return one(7);
        }
        if has('5') {
            return one(18);
        }
        if has('A') {
            return match (has('7'), has('4'), ira) {
                (true, _, false) => one(9),
                (false, true, false) => one(10),
                (true, _, true) => one(0),
                (false, true, true) => one(12),
                (false, false, false) => alloc::vec![9, 10],
                (false, false, true) => alloc::vec![0, 12],
            };
        }
        // Box 7b with 2 or 7: the employer's Roth SEP or Roth SIMPLE contribution reported as
        // converted, beside the ordinary distribution of the traditional IRA.
        if ira && (is('2') || is('7')) {
            return alloc::vec![if is('2') { 1 } else { 0 }, 20];
        }
        one(if has('8') {
            15
        } else if has('N') || has('P') || has('R') {
            16
        } else if has('9') || has('E') || has('F') || has('U') {
            17
        } else if ira && has('W') {
            // A care charge printed for an IRA is outside the supported branches.
            8
        } else if !ira && (is('W') || box7 == Self::of('W') | Self::of('6')) {
            // A qualified long-term-care charge (code W, alone or with the 1035
            // exchange code 6) is the care-charge branch, not a plain exchange.
            19
        } else if has('6') {
            11
        } else if has('G') || has('H') {
            4
        } else if has('S') {
            5
        } else if has('Q') || has('J') || has('T') {
            7
        } else if has('4') {
            if ira { 12 } else { 3 }
        } else if has('3') {
            2
        } else if has('1') || has('2') {
            1
        } else if is('D') {
            6
        } else if is('6') && !ira {
            11
        } else if is('Q') || is('J') || is('T') {
            7
        } else {
            0
        })
    }
    pub(crate) fn designated_roth_qualified(box7: u32, first_roth_year: i32) -> bool {
        box7 & Self::CODE_B.0 != 0
            && box7 & (Self::CODE_7.0 | Self::CODE_3.0 | Self::CODE_4.0) != 0
            && first_roth_year > 0
            && first_roth_year <= 2021
    }

    // The box 2b methods the row's box 7a codes and box 7b allow, written by
    // the topic catalog: the discriminants of
    // `TaxableMethod`. Box 2a as printed is always one; an employer plan's
    // annuity adds the plan methods, a death adds the CSRS/FERS ones, and a
    // nonqualified annuity (code D) adds the commercial annuity methods. The
    // corrective, not-income, no-additional-tax and prohibited-transaction
    // branches use box 2a only. A designated Roth account also offers its
    // existing proportional-basis withdrawal worksheet when needed.
    pub(crate) fn methods(form: &crate::R1099) -> Vec<i32> {
        // Instructions for Form 1099-R (2026), boxes 2a and 2b: the payer has
        // determined this taxable amount. IRA basis is determined separately.
        if form.payer_taxable_is_settled() {
            return alloc::vec![0];
        }
        let (box7, ira, trump) = (form.box7, form.ira, form.trump);
        let nonqualified = box7 & Self::of('D') != 0;
        let designated_roth = box7 & Self::of('B') != 0;
        let mut out: Vec<i32> = Vec::new();
        for branch in Self::treatments(box7, ira, trump) {
            let methods: &[i32] = match branch {
                _ if designated_roth => &[0, 5],
                0 | 1 | 2 | 3 | 6 if nonqualified => &[0, 2, 3, 4, 6, 7],
                0 | 1 | 2 | 9 if !ira => &[0, 1, 2, 4, 5],
                3 | 10 => &[0, 1, 2, 4, 5, 8, 9],
                _ => &[0],
            };
            for m in methods {
                if !out.contains(m) {
                    out.push(*m);
                }
            }
        }
        out.sort_unstable();
        out
    }
    // The Roth IRA code follows the printed Q, J or T (R_ROTH_CODE_NOT_BOX7).
    pub(crate) fn leads_to_roth(box7: u32, roth: &crate::RothIraCode) -> bool {
        let code = Self::of(match roth {
            crate::RothIraCode::Qualified => 'Q',
            crate::RothIraCode::Early { .. } => 'J',
            crate::RothIraCode::ExceptionApplies { .. } => 'T',
        });
        // J may be printed with S (a Roth SIMPLE IRA inside its first two years).
        box7 == code
            || (matches!(roth, crate::RothIraCode::Early { .. }) && box7 == code | Self::of('S'))
    }
    // The refusal of a 1099-R whose box 7a codes the App does not calculate
    // (`RetirementTreatment::Unsupported`), naming the printed codes. Stored,
    // not rejected, like the W-2 box 12 codes (`federal_rules.rs`
    // `w2_box12_refusal`).
    pub(crate) fn refusal(box7: u32) -> (String, crate::Message) {
        let codes = Self::printed(box7);
        let (key, text) = match codes[..] {
            ['5'] => (
                "ISSUE_1099R_CODE_5",
                crate::fixture_english!(
                    "Form 1099-R code 5 from a Roth IRA (prohibited transaction) is not calculated yet."
                ),
            ),
            ['C'] => (
                "ISSUE_1099R_CODE_C",
                crate::fixture_english!(
                    "Form 1099-R code C (reportable death benefits, section 6050Y) is not calculated yet."
                ),
            ),
            ['F'] => (
                "ISSUE_1099R_CODE_F",
                crate::fixture_english!(
                    "Form 1099-R code F (charitable gift annuity) with a box 3 capital gain is not calculated yet."
                ),
            ),
            [first, second] => {
                #[cfg(test)]
                let english = alloc::format!(
                    "Form 1099-R codes {first} and {second} printed together in box 7a are not calculated yet."
                );
                #[cfg(not(test))]
                let english = String::new();
                return (
                    english,
                    crate::Message::new(
                        "ISSUE_1099R_CODES_PAIR",
                        alloc::vec![
                            crate::MessageArg::Text(first.to_string()),
                            crate::MessageArg::Text(second.to_string()),
                        ],
                    ),
                );
            }
            _ => {
                #[cfg(test)]
                let english = alloc::format!(
                    "Form 1099-R box 7a code {} is not calculated yet.",
                    codes.iter().collect::<String>()
                );
                #[cfg(not(test))]
                let english = String::new();
                return (
                    english,
                    crate::Message::new(
                        "ISSUE_1099R_CODE_OTHER",
                        alloc::vec![crate::MessageArg::Text(codes.iter().collect::<String>())],
                    ),
                );
            }
        };
        (text.into(), crate::Message::new(key, alloc::vec![]))
    }
}

impl crate::TaxableMethod {
    // The discriminant, in declaration order: the values `DistributionCodes::methods` lists.
    pub(crate) fn index(&self) -> i32 {
        match self {
            Self::BoxTwoA => 0,
            Self::Simplified { .. } => 1,
            Self::GeneralRule { .. } => 2,
            Self::VariablePayments { .. } => 3,
            Self::WholeContract { .. } => 4,
            Self::PlanWithdrawal { .. } => 5,
            Self::AnnuityWithdrawal { .. } => 6,
            Self::Pre1982Withdrawal { .. } => 7,
            Self::CivilServiceLumpSum { .. } => 8,
            Self::ChildAnnuity { .. } => 9,
        }
    }
}

impl crate::Disposition {
    // The distributed property sold for a rollover, when that is what happened to it.
    pub(crate) fn property(&self) -> Option<&crate::PropertyRollover> {
        match self {
            Self::SoldAndRolledOver { data, .. } => Some(data),
            _ => None,
        }
    }
    // The amount the disposition deposited (a rollover or the proceeds of a sale).
    fn deposited(&self) -> Option<i64> {
        match self {
            Self::RolledOver { amount, .. } | Self::SoldAndRolledOver { amount, .. } => {
                Some(*amount)
            }
            _ => None,
        }
    }
}

// The what-happened answer of the branches that carry one.
pub(crate) fn disposition_of(code: &DistributionCode) -> Option<&crate::Disposition> {
    use crate::LumpSumTax;
    match code {
        DistributionCode::Normal { what_happened }
        | DistributionCode::Early { what_happened, .. }
        | DistributionCode::NormalLumpSum { tax: LumpSumTax::WithOtherIncome { what_happened } } => {
            Some(what_happened)
        }
        _ => None,
    }
}

impl DistributionCodes {
    // The server twins of the R1099 rules about the box 2b method, the new treatment
    // branches and the property sold for a rollover (the keys are the client rules'
    // keys), first failure first. R_BOX2B_METHOD has no client twin: the method is a
    // choice constrained by the row's allowed set.
    #[cfg(test)]
    pub(crate) fn treatment_rule(v: &crate::R1099Answered) -> Option<&'static str> {
        Self::treatment_issue(v).map(|issue| issue.key)
    }
    // Printed code-W relationships are independent of the later personal election.
    pub(crate) fn care_charge_source_issue(
        v: &crate::R1099,
    ) -> Option<crate::validation::ValidationIssue> {
        DistributionBoxes::from(v).care_charge_issue()
    }

    pub(crate) fn treatment_issue(
        v: &crate::R1099Answered,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::DisasterDistribution;
        let d = Self::of('D');
        let has_d = v.box7 & d != 0;
        let Some(method) = v.effective_taxable_method() else {
            return Some(
                crate::validation::ValidationIssue::new("R_TAXABLE_METHOD_NEEDED")
                    .at_field("taxable_method"),
            );
        };
        if !Self::methods(v).contains(&method.index()) {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX2B_METHOD")
                    .at_field("taxable_method"),
            );
        }
        if v.taxable_not_determined
            && matches!(method, crate::TaxableMethod::BoxTwoA)
            && !v.ira
            && v.printed_taxable_method().is_none()
        {
            return Some(
                crate::validation::ValidationIssue::new("R_UNDETERMINED_METHOD_NEEDED")
                    .at_field("taxable_method"),
            );
        }
        // Code W is the care-charge branch's own rule, not an exchange.
        let exchange = Self::CODE_6.0 | Self::CODE_N.0 | Self::CODE_R.0;
        if v.box7 & exchange != 0 && v.taxable.unwrap_or(0) != 0 {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX7_EXCHANGE_TAXABLE")
                    .at_field("taxable"),
            );
        }
        let other_boxes = matches!(v.disaster(), DisasterDistribution::No)
            && matches!(v.disability(), crate::DisabilityIncome::None)
            && v.public_safety().is_empty();
        if let Some(issue) = Self::code_issue(v) {
            return Some(issue);
        }
        if v.box7 & Self::CODE_W.0 != 0
            || matches!(v.code, Some(DistributionCode::QualifiedCareCharge))
        {
            let issue = crate::validation::ValidationIssue::new("R_CARE_CHARGE_SOURCE_INVALID");
            let failure = Self::care_charge_source_issue(v).or_else(|| {
                if !(v.box7 == Self::CODE_W.0 || v.box7 == Self::CODE_W.0 | Self::CODE_6.0) {
                    Some(issue.at_field("box7"))
                } else if !matches!(v.code, Some(DistributionCode::QualifiedCareCharge)) {
                    Some(issue.at_field("code"))
                } else if !matches!(method, crate::TaxableMethod::BoxTwoA) {
                    Some(issue.at_field("taxable_method"))
                } else if !other_boxes {
                    Some(other_boxes_issue(v, "R_CARE_CHARGE_SOURCE_INVALID"))
                } else {
                    None
                }
            });
            if failure.is_some() {
                return failure;
            }
        }
        match &v.code {
            Some(DistributionCode::Exchange) if !other_boxes => {
                return Some(other_boxes_issue(v, "R_EXCHANGE_OTHER_BOXES"));
            }
            Some(DistributionCode::Inherited { .. })
            | Some(DistributionCode::TrumpDeath { .. })
            | Some(DistributionCode::TrumpToAble { .. })
                if !other_boxes =>
            {
                return Some(other_boxes_issue(v, "R_INHERITED_OTHER_BOXES"));
            }
            _ => {}
        }
        if has_d
            && matches!(
                v.code,
                Some(DistributionCode::Normal { .. }) | Some(DistributionCode::Early { .. })
            )
            && v.code.as_ref().and_then(disposition_of) != Some(&crate::Disposition::Kept)
        {
            return Some(disposition_selection_issue(v, "R_NONQUALIFIED_KEPT"));
        }
        match v.code.as_ref().and_then(disposition_of) {
            Some(crate::Disposition::RolledOver { amount, .. }) if *amount > v.gross => {
                return Some(disposition_issue(
                    v,
                    crate::validation::ValidationIssue::new("R_ROLLOVER_ABOVE_GROSS")
                        .at_field("amount"),
                ));
            }
            Some(crate::Disposition::Converted { amount }) if *amount > v.gross => {
                return Some(disposition_issue(
                    v,
                    crate::validation::ValidationIssue::new("R_CONVERSION_ABOVE_GROSS")
                        .at_field("amount"),
                ));
            }
            _ => {}
        }
        if matches!(&v.code, Some(DistributionCode::EarlySimple {
            what_happened: crate::SimpleDisposition::RolledOver { amount }
        }) if *amount > v.gross)
        {
            return Some(disposition_issue(
                v,
                crate::validation::ValidationIssue::new("R_ROLLOVER_ABOVE_GROSS")
                    .at_field("amount"),
            ));
        }
        if let Some(property) =
            v.code.as_ref().and_then(disposition_of).and_then(crate::Disposition::property)
        {
            let sold = v
                .code
                .as_ref()
                .and_then(disposition_of)
                .and_then(crate::Disposition::deposited)
                .unwrap_or(0);
            if (v.ira || has_d)
                && matches!(
                    v.code,
                    Some(DistributionCode::Normal { .. }) | Some(DistributionCode::Early { .. })
                )
            {
                return Some(
                    crate::validation::ValidationIssue::new("PROPERTY_ROLLOVER_NOT_FROM_A_PLAN")
                        .at_field("ira"),
                );
            }
            if !(matches!(method, crate::TaxableMethod::BoxTwoA)
                && v.taxable.unwrap_or(0) == v.gross)
            {
                return Some(
                    crate::validation::ValidationIssue::new("PROPERTY_ROLLOVER_AFTER_TAX")
                        .at_field("taxable"),
                );
            }
            if sold > property.proceeds {
                return Some(disposition_issue(
                    v,
                    crate::validation::ValidationIssue::new("PROPERTY_ROLLOVER_ABOVE_PROCEEDS")
                        .at_field("amount"),
                ));
            }
            if property.sold < property.distributed {
                return Some(disposition_issue(
                    v,
                    crate::validation::ValidationIssue::new("PROPERTY_SOLD_BEFORE_DISTRIBUTION")
                        .at_field("sold")
                        .in_field("data"),
                ));
            }
        }
        match method {
            crate::TaxableMethod::PlanWithdrawal { data } if v.gross > data.value => {
                return Some(
                    crate::validation::ValidationIssue::new("R_WITHDRAWAL_ABOVE_BALANCE")
                        .at_field("value")
                        .in_field("data")
                        .in_field("taxable_method"),
                );
            }
            crate::TaxableMethod::AnnuityWithdrawal { data } if v.gross > data.value => {
                return Some(
                    crate::validation::ValidationIssue::new("R_WITHDRAWAL_ABOVE_BALANCE")
                        .at_field("value")
                        .in_field("data")
                        .in_field("taxable_method"),
                );
            }
            crate::TaxableMethod::CivilServiceLumpSum { data } if data.recovered > data.cost => {
                return Some(
                    crate::validation::ValidationIssue::new("CSRS_RECOVERED_ABOVE_COST")
                        .at_field("recovered")
                        .in_field("data")
                        .in_field("taxable_method"),
                );
            }
            _ => {}
        }
        if v.trump && v.ira {
            return Some(
                crate::validation::ValidationIssue::new("R_TRUMP_AND_IRA").at_field("trump"),
            );
        }
        if v.trump
            && v.box7 != Self::CODE_4.0
            && v.box7 != Self::CODE_8.0
            && v.box7 != Self::CODE_G.0
        {
            return Some(crate::validation::ValidationIssue::new("R_TRUMP_CODES").at_field("box7"));
        }
        match v.code {
            Some(DistributionCode::TrumpDeath { .. })
            | Some(DistributionCode::TrumpToAble { .. })
                if !v.trump =>
            {
                Some(
                    crate::validation::ValidationIssue::new("R_TRUMP_BRANCH_NEEDS_BOX_7C")
                        .at_field("trump"),
                )
            }
            Some(DistributionCode::TrumpToAble { .. }) if v.taxable.unwrap_or(0) != 0 => Some(
                crate::validation::ValidationIssue::new("R_TRUMP_ABLE_TAXABLE").at_field("taxable"),
            ),
            _ => None,
        }
    }
    // The server twins of the R1099 rules about the special codes (retirement-special-codes map,
    // R3 to R13), first failure first; the keys are the client rules' keys.
    fn code_issue(v: &crate::R1099Answered) -> Option<crate::validation::ValidationIssue> {
        use crate::{
            DisasterDistribution, Disposition, EarlyException, RolloverDestination as Into,
            RolloverTarget,
        };
        let has = |code: char| v.box7 & Self::of(code) != 0;
        let disposition = v.code.as_ref().and_then(disposition_of);
        // A 60-day rollover into a Roth account, and what else happened to the money.
        let rolled_to_roth =
            matches!(disposition, Some(Disposition::RolledOver { into: Into::Roth, .. }));
        let kept = disposition == Some(&Disposition::Kept);
        let plain = matches!(
            v.code,
            Some(DistributionCode::Normal { .. }) | Some(DistributionCode::Early { .. })
        );
        let exceptions = match &v.code {
            Some(DistributionCode::Early {
                exception: crate::EarlyException::Applies { facts, second, .. },
                ..
            })
            | Some(DistributionCode::EarlySimple {
                what_happened:
                    crate::SimpleDisposition::Kept {
                        exception: crate::EarlyException::Applies { facts, second, .. },
                    },
            })
            | Some(DistributionCode::ProhibitedTransaction {
                exception: crate::EarlyException::Applies { facts, second, .. },
            }) => [Some(facts), second.as_ref()],
            Some(DistributionCode::RothIra {
                roth: crate::RothIraCode::Early { exception, .. },
                ..
            }) => [exception.as_ref(), None],
            _ => [None, None],
        };
        if let Some(issue) = exceptions.into_iter().enumerate().find_map(|(index, exception)| {
            exception.and_then(|exception| match exception {
                RetirementException::Emergency { history, .. } => history
                    .issue()
                    .map(|issue| exception_issue(v, index == 1, issue.in_field("history"))),
                _ => None,
            })
        }) {
            return Some(issue);
        }
        if has('H') && v.taxable.unwrap_or(0) != 0 {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX7_H_TAXABLE").at_field("taxable"),
            );
        }
        if has('H')
            && matches!(v.code, Some(DistributionCode::DirectRollover { to }) if to != RolloverTarget::Roth)
        {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX7_H_TO_ROTH")
                    .at_field("to")
                    .in_field("code"),
            );
        }
        if has('B')
            && matches!(v.code, Some(DistributionCode::DirectRollover { to }) if to == RolloverTarget::Ira)
        {
            return Some(
                crate::validation::ValidationIssue::new("R_DRAC_DIRECT_ROLLOVER")
                    .at_field("to")
                    .in_field("code"),
            );
        }
        if v.ira
            && (matches!(
                v.code,
                Some(DistributionCode::DirectRollover { to: RolloverTarget::Roth })
            ) || (plain && rolled_to_roth))
        {
            return Some(if matches!(v.code, Some(DistributionCode::DirectRollover { .. })) {
                crate::validation::ValidationIssue::new("R_ROTH_ROLLOVER_FROM_A_PLAN")
                    .at_field("to")
                    .in_field("code")
            } else {
                disposition_issue(
                    v,
                    crate::validation::ValidationIssue::new("R_ROTH_ROLLOVER_FROM_A_PLAN")
                        .at_field("into"),
                )
            });
        }
        if has('B') && plain && !kept && !rolled_to_roth {
            return Some(disposition_selection_issue(v, "R_DRAC_ROLLOVER_TO_ROTH"));
        }
        if has('L') && plain && !kept {
            return Some(disposition_selection_issue(v, "R_LOAN_KEPT"));
        }
        if let Some(DistributionCode::Corrective { excess_deferral }) = v.code {
            if excess_deferral > v.taxable.unwrap_or(0) {
                return Some(
                    crate::validation::ValidationIssue::new("R_EXCESS_DEFERRAL_ABOVE_TAXABLE")
                        .at_field("excess_deferral")
                        .in_field("code"),
                );
            }
            if excess_deferral != 0 && (v.ira || has('B') || has('J')) {
                return Some(
                    crate::validation::ValidationIssue::new("R_EXCESS_DEFERRAL_NOT_A_PLAN")
                        .at_field("excess_deferral")
                        .in_field("code"),
                );
            }
        }
        if matches!(
            v.code,
            Some(DistributionCode::NotIncome)
                | Some(DistributionCode::NoAdditionalTax)
                | Some(DistributionCode::Corrective { .. })
        ) && !(matches!(v.taxable_method, Some(crate::TaxableMethod::BoxTwoA))
            && matches!(v.disaster(), DisasterDistribution::No)
            && v.public_safety().is_empty())
        {
            return Some(special_code_issue(v));
        }
        // A Roth IRA row refuses every other box under its own key (R_ROTH_OTHER_BOXES).
        let roth = matches!(v.code, Some(DistributionCode::RothIra { .. }));
        if !roth && !v.trump && v.box_10.unwrap_or(0) != 0 && !has('B') {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX10_WITHOUT_B").at_field("box_10"),
            );
        }
        if !roth && v.box_10.unwrap_or(0) > v.gross {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX10_ABOVE_GROSS").at_field("box_10"),
            );
        }
        if let Some(basis) = v.box_10_basis() {
            if v.box_10.unwrap_or(0) == 0 || !has('B') {
                return Some(
                    crate::validation::ValidationIssue::new("R_BOX10_BASIS_WITHOUT_BOX10")
                        .at_field("box_10_basis"),
                );
            }
            if basis > v.box_10.unwrap_or(0) {
                return Some(
                    crate::validation::ValidationIssue::new("R_BOX10_BASIS_ABOVE_BOX10")
                        .at_field("box_10_basis"),
                );
            }
        }
        if let Some(DistributionCode::Early { exception, .. })
        | Some(DistributionCode::EarlySimple {
            what_happened: crate::SimpleDisposition::Kept { exception },
        })
        | Some(DistributionCode::ProhibitedTransaction { exception }) = &v.code
            && let EarlyException::Applies { distributed, facts, second, repaid } = exception
        {
            if matches!(
                second,
                Some(
                    RetirementException::PeriodicPayments { .. } | RetirementException::ReachedAge
                )
            ) {
                return Some(exception_issue(
                    v,
                    true,
                    crate::validation::ValidationIssue::new("R_SECOND_EXCEPTION_KIND").at_record(),
                ));
            }
            if let Some(repaid) = repaid {
                if !(v.ira && special(facts)) {
                    return Some(repayment_issue(v, "R_REPAYMENT_NOT_SPECIAL"));
                }
                if repaid.amount_exact() > exact::whole(v.gross) {
                    return Some(repayment_issue(v, "R_REPAYMENT_ABOVE_GROSS"));
                }
            }
            if matches!(facts, RetirementException::PeriodicPayments { first_payment, .. } if first_payment > distributed)
            {
                return Some(exception_issue(
                    v,
                    false,
                    crate::validation::ValidationIssue::new(
                        "SEPP_FIRST_PAYMENT_AFTER_DISTRIBUTION",
                    )
                    .at_field("first_payment"),
                ));
            }
        }
        if let Some(DistributionCode::DeathLumpSum {
            died,
            tax: crate::InheritedLumpSumTax::Form4972 { death_benefit, .. },
        }) = &v.code
            && *death_benefit != 0
            && *died >= 19960821
        {
            return Some(
                crate::validation::ValidationIssue::new("LUMP_SUM_DEATH_BENEFIT_AFTER_1996")
                    .at_field("death_benefit")
                    .in_field("tax")
                    .in_field("code"),
            );
        }
        if has('D') && !matches!(v.disaster(), DisasterDistribution::No) {
            return Some(
                crate::validation::ValidationIssue::new("R_DISASTER_NONQUALIFIED")
                    .at_field("disaster"),
            );
        }
        if let Some(DistributionCode::RothIra { roth, rolled_over, sequence }) = &v.code {
            if sequence.is_some_and(|value| !(1..=100).contains(&value)) {
                return Some(
                    crate::validation::ValidationIssue::new("R_ROTH_SEQUENCE_INVALID")
                        .at_field("sequence")
                        .in_field("code"),
                );
            }
            if *rolled_over > v.gross {
                return Some(
                    crate::validation::ValidationIssue::new("R_ROTH_ROLLOVER_ABOVE_GROSS")
                        .at_field("rolled_over")
                        .in_field("code"),
                );
            }
            if let crate::RothIraCode::Early { exception, repaid, .. } = roth {
                if exception.as_ref().is_some_and(|e| {
                    matches!(
                        e,
                        RetirementException::Separation { .. }
                            | RetirementException::Governmental457 { .. }
                            | RetirementException::AutomaticEnrollment { .. }
                            | RetirementException::ImmediateAnnuity { .. }
                            | RetirementException::DomesticRelations
                            | RetirementException::LongTermCare { .. }
                            | RetirementException::EsopDividends
                            | RetirementException::FederalPhasedRetirement
                    )
                }) {
                    return Some(
                        crate::validation::ValidationIssue::new("R_ROTH_EXCEPTION_NOT_IRA")
                            .at_field("exception")
                            .in_field("roth")
                            .in_field("code"),
                    );
                }
                if let Some(repaid) = repaid {
                    if !exception.as_ref().is_some_and(special) {
                        return Some(repayment_issue(v, "R_REPAYMENT_NOT_SPECIAL"));
                    }
                    if repaid.amount_exact() > exact::whole(v.gross - rolled_over) {
                        return Some(repayment_issue(v, "R_REPAYMENT_ABOVE_GROSS"));
                    }
                }
            }
            if !matches!(
                v.disaster(),
                DisasterDistribution::No | DisasterDistribution::Qualified { .. }
            ) {
                return Some(
                    crate::validation::ValidationIssue::new("R_ROTH_DISASTER_KIND")
                        .at_field("disaster"),
                );
            }
        }
        if matches!(v.code, Some(DistributionCode::EmployerRothContribution))
            && !(v.ira
                && v.taxable.unwrap_or(0) == v.gross
                && matches!(v.taxable_method, Some(crate::TaxableMethod::BoxTwoA))
                && matches!(v.disaster(), DisasterDistribution::No)
                && v.public_safety().is_empty())
        {
            return Some(
                crate::validation::ValidationIssue::new("R_EMPLOYER_ROTH_CONTRIBUTION")
                    .at_field("code"),
            );
        }
        if let Some(crate::TaxableMethod::Simplified { data }) = &v.taxable_method
            && data.cost.is_none()
            && !(v.box_9b.is_some() && data.started / 10000 == 2026)
        {
            return Some(
                crate::validation::ValidationIssue::new("R_COST_NEEDS_BOX_9B")
                    .at_field("cost")
                    .in_field("data")
                    .in_field("taxable_method"),
            );
        }
        None
    }
}

/// Work covered by the employer plan
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SeparationWork {
    /// Ordinary employment: separation in or after the year of age 55
    Ordinary,
    /// Qualified public-safety work or private-sector firefighting
    PublicSafety {
        /// Completed service under this plan when employment ended
        /// Use the plan's service record, not years worked for unrelated employers or in unrelated plans.
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("year")]
        service_years: u32,
        /// This employment and plan qualify for the public-safety exception
        /// Governmental plans: eligible state/local police, fire, emergency medical, corrections or forensic-patient security work; or specified federal law-enforcement, border, fire, air-traffic, nuclear-courier, Capitol/Supreme Court police or diplomatic-security work. Private-sector firefighters need a qualified employer plan, section 403(a) annuity or 403(b) plan. An IRA does not qualify. No does not apply the exception, so the additional tax applies.
        eligible: bool,
    },
}
impl SeparationWork {
    pub(crate) fn qualifies(&self, birth: u32, separated: u32) -> bool {
        // IRC 72(t)(2)(A)(v), (t)(10): separation in or after the year the age
        // is attained, tested at the close of the separation year. The age is
        // day-exact (Pub. 501): a person born January 1, 1971 attained 55 on
        // December 31, 2025.
        let year_end = separated / 10000 * 10000 + 1231;
        let attained = |years| crate::accepted_filers::attained_on(birth, years, true, year_end);
        match self {
            Self::Ordinary => attained(55),
            Self::PublicSafety { service_years, eligible } => {
                *eligible && (*service_years >= 25 || attained(50))
            }
        }
    }
}

/// Return to work after the unemployment period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Reemployment {
    /// Not reemployed before this distribution
    None,
    /// Reemployed
    Started {
        /// First day back at work
        #[libertas_date_only]
        date: u32,
    },
}

/// Unemployment qualification for IRA insurance withdrawals
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum UnemploymentQualification {
    /// Federal or state unemployment compensation received
    Received {
        /// Job loss, compensation and consecutive qualifying period confirmed
        /// Keep the benefit records. Enter the compensation year and consecutive weeks below; interrupted periods cannot be added together to reach 12 weeks. No does not apply the exception, so the additional tax applies.
        confirmed: bool,
    },
    /// Compensation unavailable only because of self-employed status
    SelfEmployed {
        /// Would have received compensation under federal or state law but for self-employed status
        /// Keep records establishing the job loss and all other benefit requirements for the entered consecutive period. Being self-employed, losing business income or choosing to stop work alone is insufficient. No does not apply the exception, so the additional tax applies. Enter the year and weeks for which compensation would have been received; do not report hypothetical benefits as taxable unemployment income.
        confirmed: bool,
    },
}
impl UnemploymentQualification {
    fn answer(self) -> bool {
        match self {
            Self::Received { confirmed } | Self::SelfEmployed { confirmed } => confirmed,
        }
    }
}

/// Earlier emergency withdrawal from this plan or IRA
/// The most recent qualifying emergency withdrawal from this account in 2024 or 2025; leave the amounts at zero when there was none.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmergencyHistory {
    /// Prior emergency distribution amount
    #[libertas_money("USD", 2)]
    // IRC 72(t)(2)(I): each qualifying emergency distribution is limited to $1,000.
    #[libertas_number(min = 0, max = 100000)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Amount repaid to this account before the current withdrawal
    #[libertas_money("USD", 2)]
    // IRC 72(t)(2)(I): each qualifying emergency distribution is limited to $1,000.
    #[libertas_number(min = 0, max = 100000)]
    #[libertas_default(0)]
    pub repaid: i64,
    /// Elective deferrals or employee/IRA contributions after that withdrawal and before this one
    /// Exclude employer contributions, rollovers and repayments already counted above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub contributed: i64,
}
impl EmergencyHistory {
    fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.repaid > self.amount).then(|| {
            crate::validation::ValidationIssue::new("EMERGENCY_REPAID_ABOVE_DISTRIBUTION")
                .at_field("repaid")
        })
    }
    /// No earlier emergency withdrawal.
    #[cfg(test)]
    pub(crate) const NONE: Self = Self { amount: 0, repaid: 0, contributed: 0 };
    fn valid(&self) -> bool {
        (0..=100000).contains(&self.amount)
            && (0..=100000).contains(&self.repaid)
            && (0..=100000000000).contains(&self.contributed)
            && self.rule().is_none()
    }
    // No earlier withdrawal, or one repaid or followed by enough contributions.
    pub(crate) fn permits(&self) -> bool {
        self.amount == 0 || self.repaid + self.contributed >= self.amount
    }
}

/// Facts supporting an early-distribution exception
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RetirementException {
    /// Certified long-term-care insurance distribution from a participating employer plan
    LongTermCare {
        /// Plan and certified insurance records
        data: crate::CareDistribution,
    },
    /// Governmental 457(b) payment, with any qualified-plan rollover source identified
    Governmental457 {
        /// Taxable part attributable to rollovers from an IRA or another kind of qualified plan
        /// Use the administrator's allocation for this payment, including investment returns on those transferred funds. Enter zero for a payment solely from the plan's own deferrals and earnings. This is not the original rollover deposit or the whole account balance. No other early-distribution exception applies to this portion; otherwise split actual source records into the appropriate treatments.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        rollover_taxable: i64,
        /// Governmental 457(b) plan and administrator's taxable source allocation confirmed
        /// Not a tax-exempt organization's nongovernmental 457 plan, IRA withdrawal or designated Roth payment. Use the pension income category and payer-determined taxable amount. No does not apply the exception, so the additional tax applies.
        eligible: bool,
    },
    /// ESOP dividends under section 404(k)
    /// Use the separate Form 1099-R with code U, not ordinary stock dividends or an ESOP stock distribution. Enter the payer's taxable amount as pension income. These dividends do not reduce Saver's Credit contributions.
    EsopDividends,
    /// Federal employee phased-retirement annuity payments
    /// The payment is under the federal phased-retirement program, not a private employer's reduced-hours arrangement. Use the payer's taxable amount; ordinary pension income remains taxable.
    FederalPhasedRetirement,
    /// Permissible automatic-enrollment withdrawal under section 414(w)
    AutomaticEnrollment {
        /// Pay date of the first default elective contribution
        /// Use the date the pay would otherwise have been taxable, including other arrangements the plan must aggregate. Obtain this date from the administrator.
        #[libertas_date_only]
        first_contribution: u32,
        /// Date you elected to withdraw
        /// Use the election date, not the later payment date.
        #[libertas_date_only]
        elected: u32,
        /// Withdrawal-election window allowed by this plan
        /// Obtain the plan's actual window, which must be between 30 and 90.
        // IRC 414(w): the plan’s permissible withdrawal election window is 30 through 90 days.
        #[libertas_number(min = 30, max = 90)]
        #[libertas_physical_unit("day")]
        window_days: u32,
        /// Plan confirms a permissible withdrawal of default contributions and earnings
        /// Eligible automatic contribution arrangement, with a timely effective election and matching-contribution forfeiture where required. No Roth source. The App includes this withdrawal automatically in the Saver’s Credit distribution lookback; do not enter it again on the credit page. Ordinary income is taxable in the payment year. No does not apply the exception, so the additional tax applies.
        eligible: bool,
    },
    /// Eligible distribution to a domestic-abuse survivor
    DomesticAbuse {
        /// Private nickname for the distributing IRA or employer plan
        /// Use the same nickname for every claim from this plan and a different nickname for each other plan. Do not enter an account number.
        #[libertas_size(min = 1)]
        plan: alloc::string::String,
        /// A qualifying date within the one-year withdrawal window
        /// No description of the abuse is requested. The relevant abuse is by a spouse or domestic partner.
        #[libertas_date_only]
        event: u32,
        /// Vested plan balance before its first 2026 domestic-abuse withdrawal
        /// Repeat this opening balance on every 2026 claim from the same plan. Obtain the vested amount from the plan administrator or IRA record.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        balance: i64,
        /// This owner's qualified domestic-abuse distributions before 2026
        /// Include all IRA and eligible employer-plan gross claims from earlier years, regardless of recovered basis. Repeat the same opening amount on every 2026 claim by this owner. Do not include current-year claims; the App shares the remaining indexed limit across them.
        #[libertas_money("USD", 2)]
        // IRC 72(t)(2)(K): prior qualified usage cannot exceed the indexed domestic-abuse distribution limit.
        #[libertas_number(min = 0, max = $DOMESTIC_ABUSE_DISTRIBUTION_LIMIT_CENTS)]
        prior_used: i64,
        /// Earlier qualified distributions from this same plan
        /// This gross amount is included in the owner's prior total above. Repeat it on every current-year claim from this plan.
        #[libertas_money("USD", 2)]
        // IRC 72(t)(2)(K): prior qualified usage cannot exceed the indexed domestic-abuse distribution limit.
        #[libertas_number(min = 0, max = $DOMESTIC_ABUSE_DISTRIBUTION_LIMIT_CENTS)]
        prior_plan_used: i64,
        /// Qualifying domestic abuse, eligible plan and all prior/current claims confirmed
        /// Abuse includes physical, psychological, sexual, emotional or economic abuse and qualifying control, isolation or intimidation. The plan is an IRA or eligible retirement plan excluding defined-benefit and spousal-consent plans. Include all of this person's claims in the opening amount and current Income records; any IRA repayment belongs in the owner's basis record. No does not apply the exception, so the additional tax applies.
        eligible: bool,
    },
    /// Emergency personal or family expenses
    Emergency {
        /// Vested account balance immediately before this withdrawal
        /// Total IRA interest or nonforfeitable employer-plan benefit. The App leaves $1,000 outside the exception and caps the exception at $1,000.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        balance: i64,
        /// Earlier emergency-withdrawal history for this account
        history: EmergencyHistory,
        /// Necessary emergency expense and eligible account confirmed
        /// This withdrawal meets an unforeseeable or immediate personal/family financial need. The account is an IRA or eligible defined-contribution/annuity/governmental 457 plan, not a defined-benefit plan. Only one withdrawal per person may use this exception in 2026. Enter any current traditional IRA repayment in the owner's basis record; prior history is complete and qualifying. Personal details of the emergency are not needed here. No does not apply the exception, so the additional tax applies.
        eligible: bool,
    },
    /// Qualified birth or adoption distribution
    BirthAdoption {
        /// Unique fictional child label
        /// Use exactly the same label for this child on all withdrawal entries, and different labels for different children. This calculation prototype does not collect filing identifiers.
        #[libertas_size(min = 1)]
        child: alloc::string::String,
        /// Child's birth or finalized legal adoption date
        #[libertas_date_only]
        event: u32,
        /// This parent's qualified distributions for this child before 2026
        /// Include all eligible plans. Repeat the same opening amount on this parent's other claims for this child. The App applies the remaining gross limit to claims saving the most additional tax first, and reports gross usage separately from taxable relief.
        #[libertas_money("USD", 2)]
        // IRC 72(t)(2)(H): prior qualified birth/adoption usage is limited to $5,000 per parent and child.
        #[libertas_number(min = 0, max = 500000)]
        prior_used: i64,
        /// Your child or eligible adoptee and eligible retirement plan confirmed
        /// Adoption must be of someone other than your spouse's child who is under 18 or permanently incapable of self-support. The source is an IRA or eligible defined-contribution/annuity/governmental 457 plan, not a defined-benefit plan. Enter any traditional IRA repayment in the owner's basis record; no other unentered claim for this parent and child. Each parent's $5,000 limit is separate; ordinary income remains taxable. No does not apply the exception, so the additional tax applies.
        eligible: bool,
    },
    /// Employer-plan payment to you as spouse or former-spouse alternate payee under a QDRO
    /// This must be a qualified domestic relations order and a payment taxable to you. An IRA withdrawal after a divorce transfer does not qualify. Payments for child/dependent alternate payees need their separate income-ownership rules.
    DomesticRelations,
    /// Qualified military reservist distribution
    Reservist {
        /// Date ordered or called to active duty
        #[libertas_date_only]
        ordered: u32,
        /// Last date in 2026 covered by the active-duty period
        /// Use the actual end date if service ended; otherwise December 31. The withdrawal must fall between the order date and the end of active duty.
        #[libertas_date_only]
        through: u32,
        /// Ordered service duration
        term: ReserveTerm,
        /// Reserve component membership and eligible distribution source confirmed
        /// IRA or amounts attributable to elective deferrals under a 401(k), 403(b) or similar arrangement. Eligible components are Army/Air National Guard, Army/Navy/Marine/Air Force/Coast Guard Reserve and the Public Health Service Reserve Corps. Traditional IRA repayments use the owner's IRA basis record and do not reverse original income. No unresolved mixed-source allocation. No does not apply the exception, so the additional tax applies.
        eligible: bool,
    },
    /// IRA withdrawal for a first home
    FirstHome {
        /// Acquisition costs paid using this withdrawal
        /// Buying, building, rebuilding and usual or reasonable settlement, financing or closing costs. Allocate each payment once across distributions and filers.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        costs: i64,
        /// Date these costs were paid
        /// Use separate entries for payments on different dates. Payment may fall in 2027 within the 120-day deadline.
        #[libertas_date_only]
        paid_on: u32,
        /// This IRA owner's lifetime qualified first-home distributions before 2026
        /// Include all traditional and Roth IRA first-home amounts. Repeat the same opening lifetime total on this owner's other 2026 claims; the App shares the remaining $10,000 gross limit, applying it where it saves the most additional tax. Recovered basis still uses the limit.
        #[libertas_money("USD", 2)]
        // IRC 72(t)(2)(F): lifetime qualified first-home distributions are limited to $10,000.
        #[libertas_number(min = 0, max = 1000000)]
        prior_used: i64,
        /// Eligible main home, first-time buyer and family relationship confirmed
        /// The buyer is you, your spouse, a child, grandchild or ancestor of either. The buyer and their spouse had no present interest in a main home during the two years ending on the binding purchase contract or start of construction. No canceled-purchase repayment or other unentered current-year first-home distribution. A Roth IRA row uses this exception too: its first-home amount comes out before the contributions and conversions when your first Roth IRA year is 2021 or earlier. No does not apply the exception, so the additional tax applies.
        eligible: bool,
    },
    /// Unreimbursed medical expenses above the AGI threshold
    Medical {
        /// Eligible unreimbursed medical expenses paid in 2026 allocated to this claim
        /// Use expenses allowable for Schedule A, even if taking the standard deduction. Exclude reimbursements, HSA/FSA-funded amounts, tax-free subsidies, premiums covered by the public-safety insurance exclusion and amounts deducted elsewhere above the line. Allocate each expense only once across this return's medical-exception entries. The App combines these expenses and applies 7.5% of final AGI once, then allocates relief to higher-rate withdrawals first. Itemized deduction entries remain separate.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        expenses: i64,
    },
    /// IRA withdrawal for health insurance during unemployment
    UnemploymentInsurance {
        /// Medical insurance premiums paid in 2026 for you, your spouse and dependents
        /// Enter only unreimbursed premiums allocated to this distribution, excluding amounts used for another distribution's exception. Tax-free subsidies and reimbursements are not your premiums. Any Schedule A or self-employed deduction is entered separately.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        premiums: i64,
        /// Actual or deemed unemployment qualification
        qualification: UnemploymentQualification,
        /// Calendar year of actual or deemed qualifying unemployment compensation
        // IRC 72(t)(2)(D): unemployment benefits must be for the distribution year or preceding year.
        #[libertas_number(min = 2025, max = 2026)]
        benefit_year: u32,
        /// Consecutive time for which compensation was received or would have been received but for self-employed status
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("week")]
        weeks: u32,
        /// Reemployment before this withdrawal
        reemployment: Reemployment,
    },
    /// IRA withdrawal for college or other eligible postsecondary education
    Education {
        /// Tuition, required fees, books, equipment and eligible special-needs services paid in 2026
        /// Include only expenses allocated to this distribution. The student must be you, your spouse, or a child or grandchild of either; no dependency claim is required. Education-credit expenses may also support this exception.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        education: i64,
        /// Room and board actually paid for this student's academic period
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        room_board: i64,
        /// School's room-and-board cost-of-attendance allowance for this period and living arrangement
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        allowance: i64,
        /// Actual charge for school-owned or school-operated housing
        /// Enter zero for private housing. The App uses the larger of this charge and the school's allowance, capped by actual room-and-board expenses.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        school_housing: i64,
        /// Student attended at least half-time during this period
        half_time: bool,
        /// Tax-free assistance or Coverdell-funded expenses included above
        /// Include tax-free scholarships, grants, employer/veterans assistance and expenses used for tax-free Coverdell withdrawals. Do not include wages, loans, gifts, inheritances or personal savings, including QTP savings withdrawals.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        assistance: i64,
        /// Eligible postsecondary institution, family relationship and expense allocation confirmed
        /// Each expense is used only once across IRA distributions and filers. Use one student's academic period per record; tax-free assistance is allocated to these expenses. Elementary/secondary and standalone credential-program cases need separate eligibility rules. No does not apply the exception, so the additional tax applies.
        eligible: bool,
    },
    /// Permanent and total disability
    Disability {
        /// Date permanent and total disability began
        /// A physician must determine that the condition prevents substantial gainful activity and is expected to result in death or be of long, continued and indefinite duration.
        #[libertas_date_only]
        began: u32,
    },
    /// Distribution as beneficiary after the account owner's death
    Death {
        /// Original account owner or annuity holder's death
        /// For an annuity held by a non-individual, use the primary annuitant's death. This must remain a beneficiary distribution; a surviving spouse who treated the IRA as their own cannot use this exception.
        #[libertas_date_only]
        died: u32,
    },
    /// Physician-certified terminal illness
    TerminalIllness {
        /// Physician certification date
        /// The signed certification must meet Notice 2024-2: expected death within 84 months, supporting evidence and required physician statements. Self-certification is insufficient.
        #[libertas_date_only]
        certified: u32,
    },
    /// Immediate commercial annuity
    ImmediateAnnuity {
        /// Original purchase date
        /// After a tax-free annuity exchange, use the purchase date of the exchanged contract, not the replacement date.
        #[libertas_date_only]
        purchased: u32,
        /// Annuity starting date
        /// The first day of the first annuity payment period, or the date the contract obligations became fixed if later. This is not necessarily the day the first payment arrived.
        #[libertas_date_only]
        started: u32,
        /// Contract purchased with a single premium
        /// The annuity contract shows it. No does not apply the exception, so the additional tax applies.
        single_premium: bool,
        /// Contract provides substantially equal payments at least annually
        /// Confirm the payment schedule in the contract, not merely the frequency of withdrawals you chose. No does not apply the exception, so the additional tax applies.
        periodic: bool,
    },
    /// Employer-plan distribution after qualifying separation from service
    Separation {
        /// Date employment with this plan's employer ended
        /// Applies to this employer plan, not an IRA rollover. The App checks the separation year and the work-specific age or service requirement.
        #[libertas_date_only]
        separated: u32,
        /// Employment covered by this plan
        work: SeparationWork,
    },
    /// Amount taken directly by an IRS levy on the account
    Levy {
        /// Actual amount the plan paid directly under the levy
        /// A voluntary withdrawal to pay an IRS bill does not qualify. IRA basis and the qualified pre-annuity withdrawal worksheet allocate basis automatically. Other partial non-IRA levies require source-specific allocation.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Substantially equal periodic payments
    /// At least yearly, for your life or the joint lives of you and your beneficiary, figured under the required minimum distribution, amortization or annuitization method (Notice 2022-6). From an employer plan they must have begun after you left that employer.
    PeriodicPayments {
        /// Date of the first payment in the series
        #[libertas_date_only]
        first_payment: u32,
        /// The payments were changed before the later of five years after the first payment and age 59½
        /// A one-time switch to the required minimum distribution method is not a change. Yes adds back the 10% tax on the earlier payments with interest, which is not calculated.
        modified: bool,
        /// Payment method and schedule confirmed
        /// The plan or IRA statement and your payment calculation show it. No does not apply the exception, so the additional tax applies.
        eligible: bool,
    },
    /// Paid on or after the day you turned 59½
    /// The payer printed code 1 though you were 59½ or older on the payment date. The App checks your birth date. A changed series of periodic payments still owes its recapture.
    ReachedAge,
}
impl RetirementException {
    // The exception's own confirmation; Yes for an exception without one.
    pub(crate) fn answer(&self) -> bool {
        match self {
            Self::Governmental457 { eligible, .. }
            | Self::AutomaticEnrollment { eligible, .. }
            | Self::DomesticAbuse { eligible, .. }
            | Self::Emergency { eligible, .. }
            | Self::BirthAdoption { eligible, .. }
            | Self::Reservist { eligible, .. }
            | Self::FirstHome { eligible, .. }
            | Self::PeriodicPayments { eligible, .. }
            | Self::Education { eligible, .. } => *eligible,
            Self::UnemploymentInsurance { qualification, .. } => qualification.answer(),
            Self::ImmediateAnnuity { single_premium, periodic, .. } => *single_premium && *periodic,
            Self::Separation { work: SeparationWork::PublicSafety { eligible, .. }, .. } => {
                *eligible
            }
            _ => true,
        }
    }
    pub(crate) fn valid_with_source_alignment(
        &self,
        distributed: u32,
        gross: i64,
        source_alignment: bool,
    ) -> bool {
        if !valid_birth(distributed) || distributed / 10000 != 2026 {
            return false;
        }
        match self {
            Self::LongTermCare { data } => data.valid(),
            Self::Governmental457 { rollover_taxable, .. } => {
                (0..=100000000000).contains(rollover_taxable)
                    && (!source_alignment || *rollover_taxable <= gross)
            }
            Self::DomesticAbuse { plan, event, balance, prior_used, prior_plan_used, .. } => {
                !plan.trim().is_empty()
                    && plan.trim() == plan
                    && valid_birth(*event)
                    && (2025..=2026).contains(&(event / 10000))
                    && (0..=100000000000).contains(balance)
                    && (!source_alignment || *balance >= gross)
                    && (0..=crate::year_constants::DOMESTIC_ABUSE_DISTRIBUTION_LIMIT_CENTS)
                        .contains(prior_used)
                    && (0..=*prior_used).contains(prior_plan_used)
            }
            Self::Emergency { balance, history, .. } => {
                (0..=100000000000).contains(balance)
                    && (!source_alignment || *balance >= gross)
                    && history.valid()
            }
            Self::BirthAdoption { child, event, prior_used, .. } => {
                !child.trim().is_empty()
                    && valid_birth(*event)
                    && (2025..=2026).contains(&(event / 10000))
                    && (0..=500000).contains(prior_used)
                    && (event / 10000 < 2026 || *prior_used == 0)
            }
            Self::DomesticRelations | Self::EsopDividends | Self::FederalPhasedRetirement => true,
            Self::ReachedAge => true,
            Self::PeriodicPayments { first_payment, .. } => {
                valid_birth(*first_payment) && *first_payment <= distributed
            }
            Self::AutomaticEnrollment { first_contribution, elected, window_days, .. } => {
                valid_birth(*first_contribution)
                    && valid_birth(*elected)
                    && (30..=90).contains(window_days)
            }
            Self::Reservist { ordered, through, term, .. } => {
                valid_birth(*ordered)
                    && valid_birth(*through)
                    && *through >= *ordered
                    && match term {
                        ReserveTerm::Indefinite => true,
                        ReserveTerm::Days { days } => (1..=36500).contains(days),
                    }
            }
            Self::FirstHome { costs, paid_on, prior_used, .. } => {
                (0..=100000000000).contains(costs)
                    && (0..=1000000).contains(prior_used)
                    && crate::dates::source_valid_date(*paid_on)
                    && (2026..=2027).contains(&(paid_on / 10000))
            }
            Self::Medical { expenses } => (0..=100000000000).contains(expenses),
            Self::UnemploymentInsurance { premiums, benefit_year, weeks, reemployment, .. } => {
                (0..=100000000000).contains(premiums)
                    && (2025..=2026).contains(benefit_year)
                    && *weeks <= 104
                    && match reemployment {
                        Reemployment::None => true,
                        Reemployment::Started { date } => {
                            valid_birth(*date) && *date <= distributed
                        }
                    }
            }
            Self::Disability { began } => valid_birth(*began),
            Self::Death { died } => valid_birth(*died),
            Self::TerminalIllness { certified } => valid_birth(*certified),
            Self::ImmediateAnnuity { purchased, started, .. } => {
                valid_birth(*purchased) && valid_birth(*started)
            }
            Self::Separation { separated, work } => {
                valid_birth(*separated)
                    && !matches!(work, SeparationWork::PublicSafety { service_years, .. } if *service_years > 80)
            }
            Self::Education {
                education,
                room_board,
                allowance,
                school_housing,
                assistance,
                ..
            } => [*education, *room_board, *allowance, *school_housing, *assistance]
                .iter()
                .all(|v| (0..=100000000000).contains(v)),
            Self::Levy { amount } => (0..=gross).contains(amount),
        }
    }
}

/// Annuity value and investment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AnnuityBalance {
    /// Cash value before this withdrawal, ignoring surrender charges
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// Unrecovered investment before this withdrawal
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub investment: i64,
}

pub(crate) fn annuity_balances(
    value: i64,
    investment: i64,
    related: &[AnnuityBalance],
) -> [crate::kernel::ratio::exact::Exact; 2] {
    // Section 72(e)(12) combines the entire insurer-confirmed group before
    // earnings-first allocation; neither pooled balance is a native output.
    use crate::kernel::ratio::exact::whole;
    related.iter().fold([whole(value), whole(investment)], |[v, b], next| {
        [v + whole(next.value), b + whole(next.investment)]
    })
}

/// Earlier-year railroad benefit principal repaid in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RailroadRepayment {
    /// Original receipt year from RRB records
    /// Unknown years need reconciliation. Same-year repayments already netted in payment boxes must not be entered here.
    // Tax structure: this repayment refers to benefits received before the 2026 tax year.
    #[libertas_number(min = 1900, max = 2025)]
    pub received_year: u32,
    /// Benefit principal repaid, excluding interest and penalties
    /// Enter each box 8 repayment once. The original benefits must have been fully taxable; repayments involving tax-free employee-cost recovery require a separate allocation.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Repaid benefits were contributory benefits
    /// True for original box 4; false for vested dual benefits or supplemental annuities in boxes 5 and 6. Split records when both kinds were repaid.
    pub contributory: bool,
}

/// Whose railroad annuity this is
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RailroadHolder {
    /// The employee's own annuity
    Employee,
    /// A spouse or divorced-spouse annuity
    Spouse,
    /// A survivor annuity after the employee's death
    Survivor {
        /// Date of death
        #[libertas_date_only]
        died: u32,
    },
}

/// Complete cash surrender, redemption or maturity of a pension or annuity contract
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ContractDischarge {
    /// Unrecovered investment immediately before the final payment
    /// Use the contract's remaining after-tax cost after earlier tax-free recoveries and other documented adjustments, not its original premium or account value.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub investment: i64,
    /// This payment completely discharges the contract, or is a gift of it
    /// No remaining payments or value, partial surrender, IRA, exchange or final-annuitant death deduction. Include actual gross cash paid before withholding; do not add back surrender charges. Any unrecovered surrender loss is not deductible in 2026. A gift counts only for a commercial annuity issued after April 22, 1987 given without full payment to someone other than your spouse or a former spouse in a divorce; box 1 is then its cash surrender value at the transfer. A complete qualified employer-plan cash discharge supplies its taxable proportion for eligible gross-limited early-distribution exceptions; commercial annuities and overlapping public-safety insurance exclusions need separate allocation. The insurer or plan statement shows it. No taxes the whole distribution with no basis recovery, with a notice; that situation is not supported.
    pub eligible: bool,
}

/// Withdrawal from an employer plan before annuity payments began
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PlanDistribution {
    /// Nonforfeitable account balance immediately before this distribution
    /// Use only the account or separate employee-contribution contract to which this payment belongs. Include its earnings; exclude unvested employer money and amounts belonging to a different separate contract.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub value: i64,
    /// Unrecovered after-tax investment in that same account or contract
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub investment: i64,
    /// Ordinary qualified employer-plan distribution before annuity starting date
    /// Includes a single sum at the start of a Simplified Method annuity. Confirm source allocation to the same account or separate contract. No IRA, after-tax amount rolled over to a Roth IRA, pre-1987 contribution-first entitlement or full-discharge loss. A rollover of after-tax amounts is supported to a traditional IRA by year-end with its owner-level basis record, in 2027 with separately reported next-year basis, or directly to a qualifying employer plan with separate accounting. Use one actual distribution per entry. Its taxable proportion also applies to eligible gross-limited early-distribution exceptions; overlapping public-safety insurance exclusions require separate allocation. The plan statement shows it. No taxes the whole distribution with no basis recovery, with a notice; that situation is not supported.
    pub eligible: bool,
}

/// Withdrawal, loan or pledge from a commercial annuity before payments began
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AnnuityDistribution {
    /// This is a loan, assignment or pledge treated as a distribution
    /// Enter the loan amount or newly assigned/pledged value as the gross distribution above. Do not repeat an amount already taxed. Taxable income increases investment; the untaxed portion does not recover basis. Later loan repayments do not change investment under these ordinary rules.
    pub loan_pledge: bool,
    /// Cash value immediately before this event, before surrender charges
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// Unrecovered after-tax investment immediately before this event
    /// Use contract tax-basis records, reflecting earlier tax-free withdrawals and other documented adjustments. Do not use market value or the original premium when basis has changed.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub investment: i64,
    /// Other annuities required to be combined with this contract
    /// Enter every other contract in the insurer-confirmed section 72(e)(12) group once, using balances immediately before this event. The ordinary rule combines contracts entered into after October 21, 1988, issued by the same insurer or affiliates to the same policyholder in the same calendar year. Leave empty when none apply. Do not include the paying contract again, another owner's contract, or a separate group.
    /// ----
    /// Related annuity
    pub related: Vec<AnnuityBalance>,
    /// Withdrawal or deemed distribution before the annuity starting date; ordinary earnings-first rules confirmed
    /// One withdrawal or deemed distribution per entry, with values immediately before it. All required related contracts are listed with reconciled pre-withdrawal values and investment. For subsequent events update every affected balance from insurer records; do not reuse the original group totals. No full surrender, exchange, pre-August 14, 1982 investment or unresolved grouping. Loan repayments and previously taxed loan amounts are not new distributions. This is a commercial annuity, not life insurance or an employer plan. Other cases need their separate calculations. The insurer statement shows it. No taxes the whole distribution with no basis recovery, with a notice; that situation is not supported.
    pub eligible: bool,
}

/// CSRS or FERS lump sum after a death
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SurvivorLumpSum {
    /// Employee cost for this complete benefit
    /// Regular employee contributions from the common record, not the gross benefit or interest. This entry covers the entire final benefit paid to one individual beneficiary.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Earlier tax-free annuity recovery from this cost
    /// Include every recipient's earlier recovery, including the final annuitant's payments in the current year. Enter zero if no survivor annuity was paid. Do not include taxable annuity payments.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub recovered: i64,
    /// Complete regular-contribution death payment with ordinary tax treatment confirmed
    /// Individual beneficiary receiving the whole CSRS/FERS balance after employee death with no survivor annuity, or after the last survivor annuity ends. No split beneficiaries, estate/trust return, separately treated voluntary contributions, FERS special death benefit, TSP, rollover, capital-gain/10-year lump-sum election, foreign/treaty adjustment or estate-tax deduction. Earlier tax-free recovery does not exceed cost. Any deduction on the deceased annuitant's final return is separate and is not claimed by this beneficiary. The OPM payment statement shows it. No taxes the whole distribution with no basis recovery, with a notice; that situation is not supported.
    pub eligible: bool,
}

/// Taxable retirement amount source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RetirementTaxable {
    /// CSRS/FERS death-benefit lump sum
    CivilServiceDeath { data: crate::SurvivorLumpSum },
    /// Calculate a partial withdrawal with pre-August 14, 1982 investment
    LegacyAnnuity { data: crate::LegacyAnnuity },
    /// Allocate retained proceeds from distributed stock sold for a rollover
    PropertyProceeds { data: crate::PropertyRollover },
    /// Complete cash surrender, redemption or maturity of a pension or annuity contract
    FullDischarge { data: crate::ContractDischarge },
    /// Calculate employer-plan distribution basis before annuity payments begin
    QualifiedWithdrawal { data: crate::PlanDistribution },
    /// Calculate a commercial annuity withdrawal, loan or pledge before payments begin
    DeferredWithdrawal { data: crate::AnnuityDistribution },
    /// Calculate a variable commercial annuity
    VariableAnnuity { data: crate::VariableAnnuity },
    /// Calculate fixed-period or lifetime General Rule pension or annuity basis
    GeneralAnnuity { data: crate::GeneralAnnuity },
    /// Railroad RRB-1099-R pension components
    Railroad {
        /// Box 8 repayments as printed
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        repaid: i64,
        /// Prior-year repayments from box 8
        repayments: Vec<crate::RailroadRepayment>,
        /// Box 4 contributory amount paid
        /// Included in box 7 gross entered above. Boxes 5 and 6 are fully taxable and cannot absorb an exclusion from employee contributions.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        contributory: i64,
        /// Box 5 vested dual benefit
        /// Included in box 7 gross entered above; fully taxable.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        box_5: i64,
        /// Box 6 supplemental annuity
        /// Included in box 7 gross entered above; fully taxable.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        box_6: i64,
        /// Whose annuity this is
        #[libertas_default($null)]
        holder: Option<RailroadHolder>,
        /// Simplified Method recovery of this annuity's employee contributions
        /// Leave absent when no recoverable employee cost remains or for a spouse/divorced-spouse annuity: those benefits do not recover the employee's contributions. A spouse's own employee annuity is different. For employee or survivor recovery use the documented worksheet; never assign employee cost to a spouse annuity or duplicate it across beneficiaries.
        #[libertas_default($null)]
        basis: Option<crate::PensionBasis>,
    },
    /// Temporary CSRS/FERS child survivor annuity
    ChildAnnuity { data: crate::ChildAnnuity },
    /// Calculate pension basis recovery using the Simplified Method
    Simplified { data: crate::PensionBasis },
    /// Payer determined the taxable amount
    Payer,
    /// Calculate traditional IRA basis allocation
    /// Choose this for every traditional IRA distribution belonging to this filer. Supply the prior basis and all year-end IRA balances in Adjustments.
    IraBasis,
    /// Inherited traditional IRA, taxed with the other payments from the same deceased owner
    /// Form 1099-R code 4 with box 7b checked. The account is the income identity of the Inherited IRA record that holds the allocated basis and estate-tax facts; absent uses the filer's only inherited traditional IRA. The payer's box 2a is kept for a payment that has no record to use.
    Inherited {
        /// Inherited IRA record
        #[libertas_number(min = 1)]
        account: Option<i64>,
    },
}

impl RetirementTaxable {
    pub(crate) fn pension_basis(&self, gross: i64) -> Option<(crate::PensionBasis, i64)> {
        match self {
            Self::Simplified { data } => Some((data.clone(), gross)),
            Self::Railroad { contributory, basis: Some(data), .. } => {
                Some((data.clone(), *contributory))
            }
            _ => None,
        }
    }
}

fn disposition_issue(
    v: &crate::R1099Answered,
    issue: crate::validation::ValidationIssue,
) -> crate::validation::ValidationIssue {
    let issue = issue.in_field("what_happened");
    let issue = if matches!(
        v.code,
        Some(DistributionCode::NormalLumpSum { tax: crate::LumpSumTax::WithOtherIncome { .. } })
    ) {
        issue.in_field("tax")
    } else {
        issue
    };
    issue.in_field("code")
}
fn exception_issue(
    v: &crate::R1099Answered,
    second: bool,
    issue: crate::validation::ValidationIssue,
) -> crate::validation::ValidationIssue {
    if matches!(v.code, Some(DistributionCode::RothIra { .. })) {
        issue.in_field("exception").in_field("roth").in_field("code")
    } else {
        let issue = issue.in_field(if second { "second" } else { "facts" }).in_field("exception");
        let issue = if matches!(v.code, Some(DistributionCode::EarlySimple { .. })) {
            issue.in_field("what_happened")
        } else {
            issue
        };
        issue.in_field("code")
    }
}
fn other_boxes_issue(
    v: &crate::R1099Answered,
    key: &'static str,
) -> crate::validation::ValidationIssue {
    let issue = crate::validation::ValidationIssue::new(key);
    if !matches!(v.disaster(), crate::DisasterDistribution::No) {
        issue.at_field("disaster")
    } else if !matches!(v.disability(), crate::DisabilityIncome::None) {
        issue.at_field("disability")
    } else {
        issue.at_field("public_safety")
    }
}

fn disposition_selection_issue(
    v: &crate::R1099Answered,
    key: &'static str,
) -> crate::validation::ValidationIssue {
    let issue = crate::validation::ValidationIssue::new(key).at_field("what_happened");
    let issue = if matches!(
        v.code,
        Some(DistributionCode::NormalLumpSum { tax: crate::LumpSumTax::WithOtherIncome { .. } })
    ) {
        issue.in_field("tax")
    } else {
        issue
    };
    issue.in_field("code")
}

fn repayment_issue(
    v: &crate::R1099Answered,
    key: &'static str,
) -> crate::validation::ValidationIssue {
    let issue = crate::validation::ValidationIssue::new(key).at_field("repaid");
    if matches!(v.code, Some(DistributionCode::RothIra { .. })) {
        issue.in_field("roth").in_field("code")
    } else if matches!(v.code, Some(DistributionCode::EarlySimple { .. })) {
        issue.in_field("exception").in_field("what_happened").in_field("code")
    } else {
        issue.in_field("exception").in_field("code")
    }
}
fn special_code_issue(v: &crate::R1099Answered) -> crate::validation::ValidationIssue {
    let issue = crate::validation::ValidationIssue::new("R_SPECIAL_CODE_OTHER_BOXES");
    if !matches!(v.taxable_method, Some(crate::TaxableMethod::BoxTwoA)) {
        issue.at_field("taxable_method")
    } else if !matches!(v.disaster(), crate::DisasterDistribution::No) {
        issue.at_field("disaster")
    } else {
        issue.at_field("public_safety")
    }
}

/// Rollover destination and completion
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RolloverDestination {
    /// Traditional IRA; deposited by December 31, 2026
    /// The receiving account's year-end balance belongs in the combined IRA balance.
    Ira,
    /// Traditional IRA; deposited in 2027 within the eligible rollover period
    /// The rollover must already be completed, not merely intended. An outgoing IRA amount is added to the 2026 basis worksheet's year-end balance automatically; do not add it there yourself. An employer-plan rollover is not an outstanding IRA rollover: its after-tax basis is reported separately for 2027 and does not shelter 2026 IRA withdrawals.
    IraNextYear,
    /// Eligible employer retirement plan
    /// The plan must accept this rollover. Only pretax IRA money can enter an employer plan; after-tax basis must remain outside it. This destination never adds an outstanding amount to the IRA basis worksheet.
    EmployerPlan,
    /// Direct employer-plan rollover with separate after-tax accounting
    DirectEmployerPlan {
        /// Direct plan-to-plan transfer to a qualified employee plan or 403(b), with separate taxable and nontaxable accounting confirmed
        /// The receiving plan accepted the rollover and keeps its after-tax contribution record separately. This choice is for an employer-plan source, not an IRA, a 60-day deposit paid through you, a governmental 457 plan or a Roth destination. Form 1099-R code G and the receiving plan's acceptance letter show it. No still counts the transfer the same way for 2026, because the after-tax part is untaxed either way, and adds a note: ask the receiving plan to account for it, or it may be taxed again when paid out.
        confirmed: bool,
    },
    /// Roth IRA or designated Roth account
    /// From an employer plan only; an IRA uses the conversion. Pretax money rolled into a Roth account is taxed and is not subject to the 10% additional tax; a designated Roth account distribution rolled over within 60 days is taxed only on the earnings not rolled over.
    Roth,
    /// Another SIMPLE IRA
    /// The only tax-free rollover from a SIMPLE IRA in its first two years (section 408(d)(3)(G)).
    SimpleIra,
}

/// Retirement distribution treatment
/// Use the payer's Form 1099-R and your distribution facts. Use the owner-level worksheet for IRA basis and Adjustments for QCDs. Use the dedicated inherited IRA entry for inherited basis allocation and Adjustments for required-distribution shortfalls. Unlisted exceptions require review.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RetirementTreatment {
    /// Ordinary distribution, no additional tax
    Ordinary,
    /// Early distribution, 10% additional tax on the taxable amount, no exception
    Early,
    /// SIMPLE IRA within the first two years, 25% additional tax, no exception
    EarlySimple,
    /// Eligible rollover to a traditional IRA or retirement plan
    /// No Roth conversion of an IRA, required minimum distribution, inherited account, hardship withdrawal, deadline waiver, or SIMPLE IRA in its first two years. A qualified plan loan offset (code M) can be rolled over until the due date of the return, including extensions, and a rollover to a Roth IRA or designated Roth account is taxed on its pretax part. IRA-to-IRA indirect rollovers must satisfy the one-per-12-month rule. Employer plans may use calculated distribution basis. After-tax amounts may enter a traditional IRA by year-end when its basis record is provided; direct transfers to a qualifying employer plan require separate taxable/nontaxable accounting. After-tax IRA deposits completed in 2027 are reported as separate next-year basis, outside the 2026 IRA worksheet. IRA source basis uses the owner-level worksheet.
    Rollover {
        /// Receiving account and completion year
        destination: RolloverDestination,
        /// Amount actually deposited into the receiving retirement account
        /// Include any withheld amount you replaced with other funds. Do not enter an intended future deposit.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// Any taxable amount kept is an early distribution subject to 10% additional tax
        early: bool,
    },
    /// Traditional IRA converted to Roth IRA
    /// Direct trustee conversion or completed eligible 60-day conversion; no RMD converted, inherited IRA, SIMPLE IRA in its first two years, employer-plan conversion or repayment. Enter any subsequent ordinary Roth withdrawal in its own Income entry with complete Roth history. With after-tax basis, use Calculate traditional IRA basis allocation for all this owner's traditional IRA distributions.
    Conversion {
        /// Amount actually deposited into the Roth IRA by December 31, 2026
        /// Include withheld money only if replaced with other funds. Any amount kept remains a withdrawal. Enter regular annual contributions separately in Adjustments.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// Any amount kept is an early withdrawal with no exception
        early: bool,
    },
    /// Early distribution with a source-supported exception
    /// Use the payer-determined taxable amount or a supported basis calculation. Nonqualified annuities currently support death, permanent-disability and immediate-annuity exceptions. No Roth recapture; qualified reservist and supported special traditional IRA repayments use the owner's IRA basis record. Repayments of birth or adoption, emergency, domestic-abuse and terminal-illness IRA distributions are entered with the exception. Required distributions were met or shortfalls are entered separately. Beneficiary distributions must remain inherited rather than treated as the surviving spouse's own IRA.
    Exception {
        /// Date of this distribution in 2026
        /// Use separate entries for distinct distribution dates or treatments, splitting the payer totals only from actual account records.
        #[libertas_date_only]
        distributed: u32,
        /// Exception facts
        facts: crate::RetirementException,
        /// A second exception on the same distribution
        #[libertas_default($null)]
        second: Option<crate::RetirementException>,
        /// SIMPLE IRA within its first two years
        early_simple: bool,
    },
    /// Another treatment or not sure
    /// A box 7a code or combination this App does not calculate; the refusal names the record's box 7a codes.
    Unsupported,
    /// Lump-sum distribution taxed separately on Form 4972
    /// Form 1099-R code A with 7 or 4: a plan participant born before January 2, 1936.
    LumpSum {
        /// Form 4972 election
        data: crate::Form4972Election,
    },
    /// Corrective distribution of excess contributions or deferrals (code 8)
    /// No 10% additional tax and no rollover. From an employer plan it is wages on Form 1040 line 1h, less the excess deferral already counted from the W-2s; from an IRA it is on line 4b, outside Form 8606.
    Corrective {
        /// Part of box 2a that is a 2026 excess deferral already counted from the W-2s
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        excess_deferral: i64,
    },
    /// Qualified long-term-care cash-value charge, Form 1099-R code W
    /// Code 6 may also be printed for an accompanying tax-free section 1035 exchange. Source gross is not an additional deductible premium. Retain the actual reported investment reduction, not the gross charge or exchange value.
    QualifiedCareCharge,
    /// Employer contribution to a Roth SEP or Roth SIMPLE IRA (code 2 or 7, box 7b)
    /// Reported as converted: taxable in full, no additional tax, outside Form 8606 Part I, a 2026 conversion for the Roth ordering.
    EmployerRothContribution,
}

/// Disability retirement pay included in this income entry
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum DisabilityIncome {
    /// None
    None,
    /// Taxable disability retirement income
    Qualifying {
        /// Qualifying disability amount already included in this entry
        /// Enter only employer accident, health or pension-plan payments replacing wages while absent due to permanent and total disability. Exclude accrued leave, Social Security, tax-free benefits and amounts after mandatory retirement age; do not add this amount to income again.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// Retired on permanent and total disability, with the required medical certification
        /// You were disabled when you retired, had not reached mandatory retirement age on January 1, 2026, and retain the Schedule R physician/VA statement or qualify to rely on the prior statement. Wage versus pension reporting follows minimum retirement age separately. Check the physician/VA certification and employer retirement records. No claims no Schedule R credit on this income.
        eligible: bool,
    },
}
impl DisabilityIncome {
    pub(crate) fn amount(&self) -> i64 {
        match self {
            Self::None => 0,
            Self::Qualifying { amount, .. } => *amount,
        }
    }
    pub(crate) fn eligible(&self) -> bool {
        matches!(self, Self::None | Self::Qualifying { eligible: true, .. })
    }
}

/// Payments made after the owner's death
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct RetirementAfterDeath {
    /// Part of box 1
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub gross: i64,
    /// Part of box 2a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxable: i64,
}

impl RetirementAfterDeath {
    pub(crate) fn valid(&self, statement: &FederalRetirement) -> bool {
        let printed = match statement.taxable {
            Some(RetirementTaxable::Payer) => statement.box_2a,
            _ => statement.gross,
        };
        (0..=statement.gross).contains(&self.gross)
            && (0..=printed).contains(&self.taxable)
            && self.taxable <= self.gross
    }
}

/// Certified long-term-care distribution records
/// Reduces only the additional early-distribution tax, not pension income. The plan must offer this distribution and accept the issuer's annual premium statement. No three-year repayment treatment. Enter any separately deductible premiums in the usual medical or business expense records; this exception does not create a deduction.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareDistribution {
    /// Consistent fictional plan label
    /// Use the same label for every payment from this plan and a different label for another plan. Multiple plans are supported when their combined qualifying amount stays within $2,600; larger cross-plan claims require further review.
    #[libertas_size(min = 1)]
    pub plan: String,
    /// Vested benefit value used by the administrator for the annual distribution limit
    /// Repeat the same annual valuation for payments from this plan. Do not enter an unvested balance or a calculated 10% limit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub balance: i64,
    /// Coverage is for this employee's spouse
    /// Otherwise the employee is the covered person. Spouse coverage receives this exception only on a joint return. No other relative qualifies.
    #[libertas_default(false)]
    pub spouse_coverage: bool,
    /// Annual certified premiums paid by or assessed to this employee for this covered person
    /// Use the issuer's annual premium statement, including eligible certified riders. Repeat the same annual total on additional payments for this person; do not repeat the other spouse's separately allocated premiums. Assessed premiums can qualify even before payment, but that does not establish a medical deduction.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub premiums: i64,
    /// Date the issuer's 2026 premium statement was filed with the plan
    #[libertas_date_only]
    pub statement_filed: u32,
    /// Participating plan and certified insurance documentation confirmed
    /// Check the plan administrator’s confirmation and issuer’s annual premium statement.
    /// No leaves this exception out, so the 10% additional tax applies. Qualified defined-contribution 401(a), 403(a) or 403(b) plan; issuer disclosure acknowledged by IRS and annual statement accepted before payment. All this employee's qualifying payments are entered. No IRA, designated Roth source, commercial annuity, defined-benefit pension or governmental 457 source. This is not a retroactive designation of an ordinary withdrawal from a plan that does not offer this option. Source valuations and premium allocations are confirmed by the administrator and issuer. A payer-reported taxable amount must already include the special section 401(a)(39)(D)/402(l)(3) pretax-first allocation. Calculated partial account-basis withdrawals require that separate allocation before use here.
    pub confirmed: bool,
}
impl CareDistribution {
    pub(crate) fn valid(&self) -> bool {
        !self.plan.trim().is_empty()
            && self.plan.trim() == self.plan
            && (0..=100000000000).contains(&self.balance)
            && (0..=100000000000).contains(&self.premiums)
            && valid_date(self.statement_filed)
    }
}

pub(crate) fn special(facts: &RetirementException) -> bool {
    matches!(
        facts,
        RetirementException::BirthAdoption { .. }
            | RetirementException::Emergency { .. }
            | RetirementException::DomesticAbuse { .. }
            | RetirementException::TerminalIllness { .. }
    )
}
