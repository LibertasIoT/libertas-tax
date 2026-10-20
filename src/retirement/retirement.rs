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
//   set is per row (R1099.allowed_methods, written by Box7Options and DistributionCodes::methods
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
//   Unit B5 also owns the B6a 1099-R branches (map health-and-able-accounts, decisions 3 and 4):
//   box 7c (R1099.trump) with code 4 writes FederalIncome::TrumpDeath and with code G
//   FederalIncome::TrumpAbleRollover (section 530A(d)(6), (d)(4); the 2026 Form 1099-R
//   instructions), a returned Trump excess (code 8) is refused on the page and goes on Other
//   taxes (R_TRUMP_EXCESS_ON_OTHER_TAXES, the stored row has no box 7c field to name it), and
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
//   amount not rolled; a code B early distribution adds box 10 (FederalRetirement.box_10). New
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
//   ConstrainedBy "^.allowed_treatments"; Box7Options (inputs ^.box7, ^.ira, declared on both)
//   replies Box7Treatments with DistributionCodes::treatments (one discriminant: 7 Normal, 1/2
//   Early, 3 Disability, 4 without box 7b Death, G DirectRollover, S EarlySimple, D
//   Nonqualified, Q/J/T RothIra, else Other), CopyTo ^.allowed_treatments; `code` has no
//   Default, so the singleton selects it. The page builders fill the set on reopen; the save
//   refuses a branch outside it (R_BOX7_TREATMENT, server only). STORAGE_FORMAT 74 kept (1099-R
//   entries cleared before testing). Tests: tests/retirement/interview.rs box_7a_*,
//   reopened_1099_r_rows_carry_their_own_treatment_sets; tests/retirement/roth_pages.rs
//   rare_roth_situations_are_refused_by_name.
// Coverage: Form RRB-1099-R — Supported (unit B5, 2026-10-09). Its own income document
//   (IncomeDocument::Rrb1099R, owned like the 1099-R) onto the pension record of the Railroad
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
use crate::federal_rules::valid_birth;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact};
use crate::kernel::ratio::fraction;
use crate::{
    DistributionCode, DistributionCodes, FederalDraft, FederalIncome, FederalIncomeEntry,
    FederalPerson, FederalResult, Owner, RetirementKind, RetirementTreatment,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// The actual 1099-R editor's native admission pipeline, in the former
// interview document-rule order. Each delegated issue is R1099-relative.
pub(crate) fn document_issue(v: &crate::R1099) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    if let Some(issue) = DistributionCodes::issue(v.box7, v.ira) {
        return Some(issue);
    }
    if !DistributionCodes::leads_to(v.box7, v.ira, v.trump, &v.code) {
        return Some(ValidationIssue::new("R_BOX7_TREATMENT").at_field("code"));
    }
    if let Some(issue) = DistributionCodes::treatment_issue(v) {
        return Some(issue);
    }
    if let DistributionCode::RothIra { roth, .. } = &v.code
        && !DistributionCodes::leads_to_roth(v.box7, roth)
    {
        return Some(
            ValidationIssue::new("R_ROTH_CODE_NOT_BOX7").at_field("roth").in_field("code"),
        );
    }
    if let DistributionCode::Disability { wages, .. } = &v.code {
        if v.ira && !matches!(wages, crate::DisabilityWages::None) {
            return Some(
                ValidationIssue::new("R_IRA_DISABILITY_NOT_WAGES")
                    .at_field("wages")
                    .in_field("code"),
            );
        }
        if matches!(wages, crate::DisabilityWages::Part { amount } if *amount > v.taxable) {
            return Some(
                ValidationIssue::new("R_DISABILITY_WAGES_ABOVE_TAXABLE")
                    .at_field("amount")
                    .in_field("wages")
                    .in_field("code"),
            );
        }
    }
    if v.taxable > v.gross {
        return Some(ValidationIssue::new("R_TAXABLE_ABOVE_GROSS").at_field("taxable"));
    }
    if let Some(p) = &v.after_death {
        let field = if p.gross > v.gross {
            Some("gross")
        } else if p.taxable > v.taxable || p.taxable > p.gross {
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
    if v.withholding > v.gross {
        return Some(ValidationIssue::new("R_WITHHOLDING_ABOVE_GROSS").at_field("withholding"));
    }
    if let Some(issue) = v.disaster.issue(v.gross) {
        return Some(issue.in_field("disaster"));
    }
    if let Some(issue) = v.disaster.code_issue(v) {
        return Some(issue);
    }
    if let Some(issue) = crate::lump_sum::page_issue(v) {
        return Some(issue);
    }
    if let Some(issue) = crate::public_safety::paid_issue(
        &v.public_safety,
        v.ira || matches!(v.code, DistributionCode::RothIra { .. }),
        v.gross,
    ) {
        return Some(issue);
    }
    if let DistributionCode::RothIra { roth, .. } = &v.code {
        if let Some(issue) = roth_other_boxes_issue(&v.more) {
            return Some(issue);
        }
        return roth.issue().map(|issue| issue.in_field("roth").in_field("code"));
    }
    None
}

fn roth_other_boxes_issue(more: &crate::RMore) -> Option<crate::validation::ValidationIssue> {
    // Informational box 9a is accepted with every treatment. Keep the original
    // whole-record predicate; the branches below identify its actual input.
    let other = crate::RMore { box_9a_percent: None, ..more.clone() };
    if other == crate::RMore::default() {
        return None;
    }
    let field = if !matches!(more.taxable_method, crate::TaxableMethod::BoxTwoA) {
        "taxable_method"
    } else if more.box_3 != 0 {
        "box_3"
    } else if more.box_6 != 0 {
        "box_6"
    } else if more.box_8a != 0 {
        "box_8a"
    } else if more.box_8b_percent.is_some() {
        "box_8b_percent"
    } else if more.box_9b.is_some() {
        "box_9b"
    } else if more.box_10 != 0 {
        "box_10"
    } else {
        "disability"
    };
    Some(
        crate::validation::ValidationIssue::new("R_ROTH_OTHER_BOXES")
            .at_field(field)
            .in_field("more"),
    )
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
    // The R1099's allowed treatments (Box7Options and the page builder): the
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
    // The box 2b methods the row's box 7a codes and box 7b allow, written by
    // Box7Options (and the page builder): the discriminants of
    // `TaxableMethod`. Box 2a as printed is always one; an employer plan's
    // annuity adds the plan methods, a death adds the CSRS/FERS ones, and a
    // nonqualified annuity (code D) adds the commercial annuity methods. The
    // corrective, not-income, no-additional-tax and prohibited-transaction
    // branches and every designated Roth account row (code B) are box 2a only.
    pub(crate) fn methods(box7: u32, ira: bool, trump: bool) -> Vec<i32> {
        let nonqualified = box7 & Self::of('D') != 0;
        let designated_roth = box7 & Self::of('B') != 0;
        let mut out: Vec<i32> = Vec::new();
        for branch in Self::treatments(box7, ira, trump) {
            let methods: &[i32] = match branch {
                _ if designated_roth => &[0],
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
    // The Roth withdrawal's printed code (a Roth record holds one of Q, J, T).
    pub(crate) fn of_roth(code: crate::RothCode) -> u32 {
        Self::of(match code {
            crate::RothCode::Qualified => 'Q',
            crate::RothCode::Early => 'J',
            crate::RothCode::ExceptionApplies => 'T',
        })
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
    pub(crate) fn treatment_rule(v: &crate::R1099) -> Option<&'static str> {
        Self::treatment_issue(v).map(|issue| issue.key)
    }
    pub(crate) fn treatment_issue(v: &crate::R1099) -> Option<crate::validation::ValidationIssue> {
        use crate::DisasterDistribution;
        let d = Self::of('D');
        let has_d = v.box7 & d != 0;
        let method = &v.more.taxable_method;
        if !Self::methods(v.box7, v.ira, v.trump).contains(&method.index()) {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX2B_METHOD")
                    .at_field("taxable_method")
                    .in_field("more"),
            );
        }
        // Code W is the care-charge branch's own rule, not an exchange.
        let exchange = Self::CODE_6.0 | Self::CODE_N.0 | Self::CODE_R.0;
        if v.box7 & exchange != 0 && v.taxable != 0 {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX7_EXCHANGE_TAXABLE")
                    .at_field("taxable"),
            );
        }
        let other_boxes = matches!(v.disaster, DisasterDistribution::No)
            && matches!(v.more.disability, crate::DisabilityIncome::None)
            && v.public_safety.is_empty();
        if let Some(issue) = Self::code_issue(v) {
            return Some(issue);
        }
        if v.box7 & Self::CODE_W.0 != 0 || matches!(v.code, DistributionCode::QualifiedCareCharge) {
            let issue = crate::validation::ValidationIssue::new("R_CARE_CHARGE_SOURCE_INVALID");
            let failure =
                if !(v.box7 == Self::CODE_W.0 || v.box7 == Self::CODE_W.0 | Self::CODE_6.0) {
                    Some(issue.at_field("box7"))
                } else if !matches!(v.code, DistributionCode::QualifiedCareCharge) {
                    Some(issue.at_field("code"))
                } else if v.ira {
                    Some(issue.at_field("ira"))
                } else if v.trump {
                    Some(issue.at_field("trump"))
                } else if v.taxable != 0 {
                    Some(issue.at_field("taxable"))
                } else if !matches!(method, crate::TaxableMethod::BoxTwoA) {
                    Some(issue.at_field("taxable_method").in_field("more"))
                } else if !(0..=v.gross).contains(&v.more.box_8a) {
                    Some(issue.at_field("box_8a").in_field("more"))
                } else if v.more.box_3 != 0 {
                    Some(issue.at_field("box_3").in_field("more"))
                } else if v.more.box_6 != 0 {
                    Some(issue.at_field("box_6").in_field("more"))
                } else if v.more.box_8b_percent.is_some() {
                    Some(issue.at_field("box_8b_percent").in_field("more"))
                } else if v.more.box_9a_percent.is_some() {
                    Some(issue.at_field("box_9a_percent").in_field("more"))
                } else if v.more.box_10 != 0 {
                    Some(issue.at_field("box_10").in_field("more"))
                } else if !other_boxes {
                    Some(other_boxes_issue(v, "R_CARE_CHARGE_SOURCE_INVALID"))
                } else {
                    None
                };
            if failure.is_some() {
                return failure;
            }
        }
        match &v.code {
            DistributionCode::Exchange if !other_boxes => {
                return Some(other_boxes_issue(v, "R_EXCHANGE_OTHER_BOXES"));
            }
            DistributionCode::Inherited { .. }
            | DistributionCode::TrumpDeath { .. }
            | DistributionCode::TrumpToAble { .. }
                if !other_boxes =>
            {
                return Some(other_boxes_issue(v, "R_INHERITED_OTHER_BOXES"));
            }
            _ => {}
        }
        if has_d
            && matches!(v.code, DistributionCode::Normal { .. } | DistributionCode::Early { .. })
            && disposition_of(&v.code) != Some(&crate::Disposition::Kept)
        {
            return Some(disposition_selection_issue(v, "R_NONQUALIFIED_KEPT"));
        }
        match disposition_of(&v.code) {
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
        if matches!(&v.code, DistributionCode::EarlySimple {
            what_happened: crate::SimpleDisposition::RolledOver { amount }
        } if *amount > v.gross)
        {
            return Some(disposition_issue(
                v,
                crate::validation::ValidationIssue::new("R_ROLLOVER_ABOVE_GROSS")
                    .at_field("amount"),
            ));
        }
        if let Some(property) = disposition_of(&v.code).and_then(crate::Disposition::property) {
            let sold = disposition_of(&v.code).and_then(crate::Disposition::deposited).unwrap_or(0);
            if (v.ira || has_d)
                && matches!(
                    v.code,
                    DistributionCode::Normal { .. } | DistributionCode::Early { .. }
                )
            {
                return Some(
                    crate::validation::ValidationIssue::new("PROPERTY_ROLLOVER_NOT_FROM_A_PLAN")
                        .at_field("ira"),
                );
            }
            if !(matches!(method, crate::TaxableMethod::BoxTwoA) && v.taxable == v.gross) {
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
                        .in_field("taxable_method")
                        .in_field("more"),
                );
            }
            crate::TaxableMethod::AnnuityWithdrawal { data } if v.gross > data.value => {
                return Some(
                    crate::validation::ValidationIssue::new("R_WITHDRAWAL_ABOVE_BALANCE")
                        .at_field("value")
                        .in_field("data")
                        .in_field("taxable_method")
                        .in_field("more"),
                );
            }
            crate::TaxableMethod::CivilServiceLumpSum { data } if data.recovered > data.cost => {
                return Some(
                    crate::validation::ValidationIssue::new("CSRS_RECOVERED_ABOVE_COST")
                        .at_field("recovered")
                        .in_field("data")
                        .in_field("taxable_method")
                        .in_field("more"),
                );
            }
            _ => {}
        }
        if v.trump && v.ira {
            return Some(
                crate::validation::ValidationIssue::new("R_TRUMP_AND_IRA").at_field("trump"),
            );
        }
        if v.trump && v.box7 == Self::CODE_8.0 {
            return Some(
                crate::validation::ValidationIssue::new("R_TRUMP_EXCESS_ON_OTHER_TAXES")
                    .at_field("box7"),
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
            DistributionCode::TrumpDeath { .. } | DistributionCode::TrumpToAble { .. }
                if !v.trump =>
            {
                Some(
                    crate::validation::ValidationIssue::new("R_TRUMP_BRANCH_NEEDS_BOX_7C")
                        .at_field("trump"),
                )
            }
            DistributionCode::TrumpToAble { .. } if v.taxable != 0 => Some(
                crate::validation::ValidationIssue::new("R_TRUMP_ABLE_TAXABLE").at_field("taxable"),
            ),
            _ => None,
        }
    }
    // The server twins of the R1099 rules about the special codes (retirement-special-codes map,
    // R3 to R13), first failure first; the keys are the client rules' keys.
    fn code_issue(v: &crate::R1099) -> Option<crate::validation::ValidationIssue> {
        use crate::{
            DisasterDistribution, Disposition, EarlyException, RolloverDestination as Into,
            RolloverTarget,
        };
        let has = |code: char| v.box7 & Self::of(code) != 0;
        let disposition = disposition_of(&v.code);
        // A 60-day rollover into a Roth account, and what else happened to the money.
        let rolled_to_roth =
            matches!(disposition, Some(Disposition::RolledOver { into: Into::Roth, .. }));
        let kept = disposition == Some(&Disposition::Kept);
        let plain =
            matches!(v.code, DistributionCode::Normal { .. } | DistributionCode::Early { .. });
        let exceptions = match &v.code {
            DistributionCode::Early {
                exception: crate::EarlyException::Applies { facts, second, .. },
                ..
            }
            | DistributionCode::ProhibitedTransaction {
                exception: crate::EarlyException::Applies { facts, second, .. },
            } => [Some(facts), second.as_ref()],
            DistributionCode::RothIra {
                roth: crate::RothIraCode::Early { exception, .. }, ..
            } => [exception.as_ref(), None],
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
        if has('H') && v.taxable != 0 {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX7_H_TAXABLE").at_field("taxable"),
            );
        }
        if has('H')
            && matches!(v.code, DistributionCode::DirectRollover { to } if to != RolloverTarget::Roth)
        {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX7_H_TO_ROTH")
                    .at_field("to")
                    .in_field("code"),
            );
        }
        if has('B')
            && matches!(v.code, DistributionCode::DirectRollover { to } if to == RolloverTarget::Ira)
        {
            return Some(
                crate::validation::ValidationIssue::new("R_DRAC_DIRECT_ROLLOVER")
                    .at_field("to")
                    .in_field("code"),
            );
        }
        if v.ira
            && (matches!(v.code, DistributionCode::DirectRollover { to: RolloverTarget::Roth })
                || (plain && rolled_to_roth))
        {
            return Some(if matches!(v.code, DistributionCode::DirectRollover { .. }) {
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
        if let DistributionCode::Corrective { excess_deferral } = v.code {
            if excess_deferral > v.taxable {
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
            DistributionCode::NotIncome
                | DistributionCode::NoAdditionalTax
                | DistributionCode::Corrective { .. }
        ) && !(matches!(v.more.taxable_method, crate::TaxableMethod::BoxTwoA)
            && matches!(v.disaster, DisasterDistribution::No)
            && v.public_safety.is_empty())
        {
            return Some(special_code_issue(v));
        }
        // A Roth IRA row refuses every other box under its own key (R_ROTH_OTHER_BOXES).
        let roth = matches!(v.code, DistributionCode::RothIra { .. });
        if !roth && v.more.box_10 != 0 && !has('B') {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX10_WITHOUT_B")
                    .at_field("box_10")
                    .in_field("more"),
            );
        }
        if !roth && v.more.box_10 > v.gross {
            return Some(
                crate::validation::ValidationIssue::new("R_BOX10_ABOVE_GROSS")
                    .at_field("box_10")
                    .in_field("more"),
            );
        }
        if let DistributionCode::Early { exception, .. }
        | DistributionCode::ProhibitedTransaction { exception } = &v.code
            && let EarlyException::Applies { distributed, facts, second, repaid } = exception
        {
            if matches!(
                second,
                Some(
                    RetirementException::PeriodicPayments { .. } | RetirementException::ReachedAge
                )
            ) {
                return Some(
                    crate::validation::ValidationIssue::new("R_SECOND_EXCEPTION_KIND")
                        .at_field("second")
                        .in_field("exception")
                        .in_field("code"),
                );
            }
            if let Some(repaid) = repaid {
                if !(v.ira && crate::ira_repayment::special(facts)) {
                    return Some(repayment_issue(v, "R_REPAYMENT_NOT_SPECIAL"));
                }
                if repaid.amount_exact() > exact::whole(v.gross) {
                    return Some(repayment_issue(v, "R_REPAYMENT_ABOVE_GROSS"));
                }
            }
            if matches!(facts, RetirementException::PeriodicPayments { first_payment, .. } if first_payment > distributed)
            {
                return Some(
                    crate::validation::ValidationIssue::new(
                        "SEPP_FIRST_PAYMENT_AFTER_DISTRIBUTION",
                    )
                    .at_field("first_payment")
                    .in_field("facts")
                    .in_field("exception")
                    .in_field("code"),
                );
            }
        }
        if let DistributionCode::DeathLumpSum {
            died,
            tax: crate::InheritedLumpSumTax::Form4972 { death_benefit, .. },
        } = &v.code
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
        if has('D') && !matches!(v.disaster, DisasterDistribution::No) {
            return Some(
                crate::validation::ValidationIssue::new("R_DISASTER_NONQUALIFIED")
                    .at_field("disaster"),
            );
        }
        if let DistributionCode::RothIra { roth, rolled_over, sequence } = &v.code {
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
                    if !exception.as_ref().is_some_and(crate::ira_repayment::special) {
                        return Some(repayment_issue(v, "R_REPAYMENT_NOT_SPECIAL"));
                    }
                    if repaid.amount_exact() > exact::whole(v.gross - rolled_over) {
                        return Some(repayment_issue(v, "R_REPAYMENT_ABOVE_GROSS"));
                    }
                }
            }
            if !matches!(
                v.disaster,
                DisasterDistribution::No | DisasterDistribution::Qualified { .. }
            ) {
                return Some(
                    crate::validation::ValidationIssue::new("R_ROTH_DISASTER_KIND")
                        .at_field("disaster"),
                );
            }
        }
        if matches!(v.code, DistributionCode::EmployerRothContribution)
            && !(v.ira
                && v.taxable == v.gross
                && matches!(v.more.taxable_method, crate::TaxableMethod::BoxTwoA)
                && matches!(v.disaster, DisasterDistribution::No)
                && v.public_safety.is_empty())
        {
            return Some(
                crate::validation::ValidationIssue::new("R_EMPLOYER_ROTH_CONTRIBUTION")
                    .at_field("code"),
            );
        }
        if let crate::TaxableMethod::Simplified { data } = &v.more.taxable_method
            && data.cost.is_none()
            && !(v.more.box_9b.is_some() && data.started / 10000 == 2026)
        {
            return Some(
                crate::validation::ValidationIssue::new("R_COST_NEEDS_BOX_9B")
                    .at_field("cost")
                    .in_field("data")
                    .in_field("taxable_method")
                    .in_field("more"),
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
        /// Completed years of service under this plan when employment ended
        /// Use the plan's service record, not years worked for unrelated employers or in unrelated plans.
        #[libertas_number(min = 0)]
        service_years: u32,
        /// This employment and plan qualify for the public-safety exception
        /// Governmental plans: eligible state/local police, fire, emergency medical, corrections or forensic-patient security work; or specified federal law-enforcement, border, fire, air-traffic, nuclear-courier, Capitol/Supreme Court police or diplomatic-security work. Private-sector firefighters need a qualified employer plan, section 403(a) annuity or 403(b) plan. An IRA does not qualify. No does not apply the exception, so the additional tax applies.
        eligible: bool,
    },
}
impl SeparationWork {
    fn qualifies(&self, birth: u32, separated: u32) -> bool {
        // IRC 72(t)(2)(A)(v), (t)(10): separation in or after the year the age
        // is attained, tested at the close of the separation year. The age is
        // day-exact (Pub. 501): a person born January 1, 1971 attained 55 on
        // December 31, 2025.
        let year_end = separated / 10000 * 10000 + 1231;
        let attained = |years| crate::federal_model::attained_on(birth, years, true, year_end);
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

/// Duration specified by the reserve call-up orders
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ReserveTerm {
    /// Indefinite period
    Indefinite,
    /// Specified number of days
    Days {
        /// Days ordered, not days already served
        #[libertas_number(min = 1)]
        days: u32,
    },
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
    #[libertas_number(min = 0)]
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
    fn permits(&self) -> bool {
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
        #[libertas_number(min = 0)]
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
        /// Withdrawal-election window allowed by this plan, in days
        /// Obtain the plan's actual window, which must be between 30 and 90 days.
        // IRC 414(w): the plan’s permissible withdrawal election window is 30 through 90 days.
        #[libertas_number(min = 30, max = 90)]
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
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        expenses: i64,
    },
    /// IRA withdrawal for health insurance during unemployment
    UnemploymentInsurance {
        /// Medical insurance premiums paid in 2026 for you, your spouse and dependents
        /// Enter only unreimbursed premiums allocated to this distribution, excluding amounts used for another distribution's exception. Tax-free subsidies and reimbursements are not your premiums. Any Schedule A or self-employed deduction is entered separately.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        premiums: i64,
        /// Actual or deemed unemployment qualification
        qualification: UnemploymentQualification,
        /// Calendar year of actual or deemed qualifying unemployment compensation
        // IRC 72(t)(2)(D): unemployment benefits must be for the distribution year or preceding year.
        #[libertas_number(min = 2025, max = 2026)]
        benefit_year: u32,
        /// Consecutive weeks for which compensation was received or would have been received but for self-employed status
        #[libertas_number(min = 0)]
        weeks: u32,
        /// Reemployment before this withdrawal
        reemployment: Reemployment,
    },
    /// IRA withdrawal for college or other eligible postsecondary education
    Education {
        /// Tuition, required fees, books, equipment and eligible special-needs services paid in 2026
        /// Include only expenses allocated to this distribution. The student must be you, your spouse, or a child or grandchild of either; no dependency claim is required. Education-credit expenses may also support this exception.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        education: i64,
        /// Room and board actually paid for this student's academic period
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        room_board: i64,
        /// School's room-and-board cost-of-attendance allowance for this period and living arrangement
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        allowance: i64,
        /// Actual charge for school-owned or school-operated housing
        /// Enter zero for private housing. The App uses the larger of this charge and the school's allowance, capped by actual room-and-board expenses.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        school_housing: i64,
        /// Student attended at least half-time during this period
        half_time: bool,
        /// Tax-free assistance or Coverdell-funded expenses included above
        /// Include tax-free scholarships, grants, employer/veterans assistance and expenses used for tax-free Coverdell withdrawals. Do not include wages, loans, gifts, inheritances or personal savings, including QTP savings withdrawals.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
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
    pub(crate) fn valid(&self, distributed: u32, gross: i64) -> bool {
        if !valid_birth(distributed) || distributed / 10000 != 2026 {
            return false;
        }
        match self {
            Self::LongTermCare { data } => data.valid(),
            Self::Governmental457 { rollover_taxable, .. } => {
                (0..=gross).contains(rollover_taxable)
            }
            Self::DomesticAbuse { plan, event, balance, prior_used, prior_plan_used, .. } => {
                !plan.trim().is_empty()
                    && plan.trim() == plan
                    && valid_birth(*event)
                    && (2025..=2026).contains(&(event / 10000))
                    && (gross..=100000000000).contains(balance)
                    && (0..=crate::year_constants::DOMESTIC_ABUSE_DISTRIBUTION_LIMIT_CENTS)
                        .contains(prior_used)
                    && (0..=*prior_used).contains(prior_plan_used)
            }
            Self::Emergency { balance, history, .. } => {
                (gross..=100000000000).contains(balance) && history.valid()
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
                    && crate::federal_rules::valid_date(*paid_on)
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
// The rows the early-distribution stage works on (Form 5329 Part I): every Form 1099-R record,
// and each Roth IRA row whose Form 5329 line 1 amount has an exception of its own, as the IRA
// distribution it is for that stage (Form 8606 line 25c earnings plus the conversion recapture
// are its taxable amount; the early-distribution exceptions apply to a Roth IRA as to any IRA).
pub(crate) fn retirement_rows<'a>(
    d: &'a FederalDraft,
    basis: &BasisResult,
) -> alloc::vec::Vec<(
    &'a crate::FederalIncomeEntry,
    alloc::borrow::Cow<'a, crate::FederalRetirement>,
)> {
    let mut rows = alloc::vec::Vec::new();
    for e in &d.income {
        match &e.income {
            FederalIncome::Retirement { data } => {
                rows.push((e, alloc::borrow::Cow::Borrowed(data)));
            }
            FederalIncome::Roth { data: w } => {
                let (Some(staged), Some(exception)) = (basis.roth.staged.get(&e.id), &w.exception)
                else {
                    continue;
                };
                // The funding of an HSA is subtracted again by BasisResult::taxable.
                let funding = basis.hsa_funding.get(&e.id).copied().unwrap_or(0);
                let record = crate::FederalRetirement {
                    disability: crate::DisabilityIncome::None,
                    kind: RetirementKind::Ira,
                    disability_wages: crate::DisabilityWages::None,
                    gross: phase::floor(w.gross - w.rolled_over - funding, 0),
                    taxable: RetirementTaxable::Payer { amount: staged + funding },
                    withholding: 0,
                    box7: 0,
                    ordinary_distribution: RetirementTreatment::Exception {
                        distributed: w.distributed,
                        facts: exception.clone(),
                        second: None,
                        early_simple: w.simple_early,
                    },
                    disaster: crate::DisasterDistribution::No,
                    after_death: None,
                    box_10: 0,
                    box_9b: None,
                };
                rows.push((e, alloc::borrow::Cow::Owned(record)));
            }
            _ => {}
        }
    }
    rows
}

#[derive(Default)]
pub(crate) struct GrossExceptions {
    pub amounts: alloc::collections::BTreeMap<i64, (i64, i64)>,
}

// Limits on qualified distributions consume gross dollars, while Form 5329
// relief covers only their taxable part. Expense-based exceptions are different.
#[derive(Clone)]
struct GrossExceptionClaim {
    id: i64,
    gross: i64,
    taxable: i64,
    eligible: i64,
    simple: bool,
    proportional: bool,
}
impl GrossExceptionClaim {
    fn relief(&self, amount: i64, r: &mut FederalResult, out: &mut GrossExceptions) -> i64 {
        let amount = phase::floor(phase::cap(amount, [self.gross]), 0);
        crate::require_localized!(
            r,
            amount == 0
                || amount == self.gross
                || self.taxable == 0
                || self.taxable == self.gross
                || self.proportional,
            &crate::IssueContext::income_id(self.id).manual(),
            "Retirement exception allocation",
            "ISSUE_A_PARTIAL_EXCEPTION_NEEDS_A_TRADITIONAL_IRA_ALLOCATION_THE_QUALIFIE_C205A7E3",
            "A partial exception needs a traditional IRA allocation, the qualified pre-annuity withdrawal worksheet, or a complete qualified employer-plan discharge. Other non-IRA basis methods and overlapping public-safety premium exclusions need source-specific allocation.",
        );
        let taxable = if self.gross == 0 { 0 } else { fraction(self.taxable, amount, self.gross) };
        out.amounts.insert(self.id, (amount, taxable));
        emit_lines(
            r,
            &mut (),
            [
                ("gross", amount, msg!("EXPLAIN_RETIREMENT_N_QUALIFIED_N_GROSS")),
                ("taxable", taxable, msg!("EXPLAIN_RETIREMENT_N_QUALIFIED_N_TAXABLE")),
            ],
            |(suffix, value, note), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("retirement_{}_qualified_{suffix}", self.id),
                    LineValue::MoneyCents(value),
                    &["income.retirement"],
                    "https://www.irs.gov/instructions/i5329",
                    note,
                ))
            },
        );
        taxable
    }
}
// Preserve the existing choice to apply a shared limit where it saves the most
// additional tax. Compare exact tax relief per gross dollar; stable IDs break ties.
fn gross_claim_order(a: &GrossExceptionClaim, b: &GrossExceptionClaim) -> core::cmp::Ordering {
    let benefit = |v: &GrossExceptionClaim| i128::from(v.taxable) * if v.simple { 25 } else { 10 };
    (benefit(b) * i128::from(a.gross.max(1)))
        .cmp(&(benefit(a) * i128::from(b.gross.max(1))))
        .then_with(|| a.id.cmp(&b.id))
}

fn gross_exception_limit(
    mut claims: alloc::vec::Vec<GrossExceptionClaim>,
    limit: i64,
    r: &mut FederalResult,
    out: &mut GrossExceptions,
) -> i64 {
    claims.sort_by(gross_claim_order);
    let mut remaining = limit;
    for claim in claims {
        let amount = crate::kernel::carry::native_consume_step(claim.eligible, &mut remaining);
        claim.relief(amount, r, out);
    }
    crate::kernel::carry::native_balance(limit, remaining)
}

fn domestic_abuse_limit(
    mut claims: alloc::vec::Vec<(alloc::string::String, GrossExceptionClaim)>,
    plans: &alloc::collections::BTreeMap<alloc::string::String, (i64, i64)>,
    limit: i64,
    r: &mut FederalResult,
    out: &mut GrossExceptions,
) -> i64 {
    claims.sort_by(|a, b| gross_claim_order(&a.1, &b.1));
    let mut remaining = limit;
    let mut plan_remaining: alloc::collections::BTreeMap<_, _> = plans
        .iter()
        .map(|(name, (balance, prior))| {
            (name.as_str(), phase::floor(crate::kernel::ratio::trunc_i64(*balance, 2) - prior, 0))
        })
        .collect();
    for (plan, claim) in claims {
        let available = plan_remaining.get_mut(plan.as_str()).expect("recorded plan");
        let amount = phase::cap(phase::cap(remaining, [*available]), [claim.eligible]);
        remaining = crate::kernel::carry::native_balance(remaining, amount);
        *available = crate::kernel::carry::native_balance(*available, amount);
        claim.relief(amount, r, out);
    }
    crate::kernel::carry::native_balance(limit, remaining)
}

pub(crate) fn gross_exceptions(
    d: &FederalDraft,
    basis: &BasisResult,
    qcd: &QcdResult,
    r: &mut FederalResult,
) -> GrossExceptions {
    let mut out = GrossExceptions::default();
    let mut birth_claims: alloc::collections::BTreeMap<
        (usize, alloc::string::String),
        (u32, i64, alloc::vec::Vec<GrossExceptionClaim>),
    > = alloc::collections::BTreeMap::new();
    let mut abuse_prior = [None, None];
    let mut abuse_plans: [alloc::collections::BTreeMap<alloc::string::String, (i64, i64)>; 2] =
        Default::default();
    let mut abuse_claims: [alloc::vec::Vec<(alloc::string::String, GrossExceptionClaim)>; 2] =
        Default::default();
    let mut home_prior = [None, None];
    let mut home_claims: [alloc::vec::Vec<GrossExceptionClaim>; 2] = Default::default();
    let rows = retirement_rows(d, basis);
    for (e, v) in &rows {
        let v: &crate::FederalRetirement = v;
        let RetirementTreatment::Exception { facts, second, early_simple, .. } =
            &v.ordinary_distribution
        else {
            continue;
        };
        #[cfg(test)]
        let source = alloc::format!("Income: {}", e.label);
        let taxable = basis.taxable(e.id, v) - basis.converted(e.id, v) - qcd.for_entry(e.id);
        let claim = GrossExceptionClaim {
            id: e.id,
            gross: v.gross,
            taxable,
            eligible: v.gross,
            simple: *early_simple,
            proportional: v.kind == RetirementKind::Ira
                || ((matches!(v.taxable, RetirementTaxable::QualifiedWithdrawal { .. })
                    || (v.kind == RetirementKind::Pension
                        && matches!(v.taxable, RetirementTaxable::FullDischarge { .. })))
                    && basis.public_safety.excluded(e.id) == 0),
        };
        for fact in core::iter::once(facts).chain(second.iter()) {
            match fact {
                RetirementException::DomesticAbuse {
                    plan,
                    balance,
                    prior_used,
                    prior_plan_used,
                    ..
                } => {
                    let owner = usize::from(e.owner == Owner::Spouse);
                    crate::require_localized!(
                        r,
                        abuse_prior[owner].is_none_or(|v| v == *prior_used),
                        &crate::IssueContext::Section(crate::TaxSection::Retirement),
                        &source,
                        "ISSUE_DOMESTIC_ABUSE_CLAIMS_FOR_ONE_OWNER_MUST_USE_THE_SAME_PRE_2026_QUAL_77D1C43F",
                        "Domestic-abuse claims for one owner must use the same pre-2026 qualified gross total.",
                    );
                    abuse_prior[owner] = Some(*prior_used);
                    let opening = abuse_plans[owner]
                        .entry(plan.clone())
                        .or_insert((*balance, *prior_plan_used));
                    crate::require_localized!(
                        r,
                        *opening == (*balance, *prior_plan_used),
                        &crate::IssueContext::Section(crate::TaxSection::Retirement),
                        &source,
                        "ISSUE_DOMESTIC_ABUSE_CLAIMS_FROM_ONE_PLAN_MUST_USE_THE_SAME_OPENING_BALAN_956F1ABF",
                        "Domestic-abuse claims from one plan must use the same opening balance and pre-2026 plan usage.",
                    );
                    abuse_claims[owner].push((plan.clone(), claim.clone()));
                }
                RetirementException::Emergency { balance, .. } => {
                    claim.relief(phase::bound(balance - 100000, 0, 100000), r, &mut out);
                }
                RetirementException::Levy { amount } => {
                    claim.relief(*amount, r, &mut out);
                }
                RetirementException::TerminalIllness { .. } => {
                    claim.relief(v.gross, r, &mut out);
                }
                RetirementException::BirthAdoption { child, event, prior_used, .. } => {
                    let entry = birth_claims
                        .entry((usize::from(e.owner == Owner::Spouse), child.clone()))
                        .or_insert((*event, *prior_used, alloc::vec::Vec::new()));
                    crate::require_localized!(
                        r,
                        entry.0 == *event && entry.1 == *prior_used,
                        &crate::IssueContext::Section(crate::TaxSection::Retirement),
                        &source,
                        "ISSUE_CLAIMS_FOR_ONE_PARENT_AND_CHILD_MUST_USE_THE_SAME_BIRTH_ADOPTION_DA_F05B51E0",
                        "Claims for one parent and child must use the same birth/adoption date and opening prior-distribution total.",
                    );
                    entry.2.push(claim.clone());
                }
                RetirementException::FirstHome { costs, prior_used, .. } => {
                    let owner = usize::from(e.owner == Owner::Spouse);
                    crate::require_localized!(
                        r,
                        home_prior[owner].is_none_or(|v| v == *prior_used),
                        &crate::IssueContext::Section(crate::TaxSection::Retirement),
                        &source,
                        "ISSUE_FIRST_HOME_CLAIMS_FOR_ONE_IRA_OWNER_MUST_USE_THE_SAME_PRE_2026_LIFE_BBE3640F",
                        "First-home claims for one IRA owner must use the same pre-2026 lifetime total.",
                    );
                    home_prior[owner] = Some(*prior_used);
                    home_claims[owner].push(GrossExceptionClaim {
                        eligible: phase::cap(*costs, [v.gross]),
                        ..claim.clone()
                    });
                }
                _ => (),
            }
        }
    }
    for ((owner, child), (_, prior, claims)) in birth_claims {
        let used = gross_exception_limit(claims, 500000 - prior, r, &mut out);
        crate::kernel::lines::line_cents(
            r,
            &alloc::format!("birth_adoption_used_{owner}_{child}"),
            prior + used,
            &["income.retirement"],
            "https://www.irs.gov/irb/2020-38_IRB",
            msg!("EXPLAIN_BIRTH_ADOPTION_USED_N_N"),
        );
    }
    for owner in 0..2 {
        if let Some(prior) = abuse_prior[owner] {
            crate::require_localized!(
                r,
                abuse_plans[owner].values().map(|(_, used)| used).sum::<i64>() <= prior,
                &crate::IssueContext::owner(
                    crate::TaxSection::Retirement,
                    if owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
                ),
                "Domestic-abuse claim history",
                "ISSUE_EARLIER_QUALIFIED_AMOUNTS_ASSIGNED_TO_CURRENT_PLANS_CANNOT_EXCEED_T_7D6D3EC7",
                "Earlier qualified amounts assigned to current plans cannot exceed the owner's total earlier qualified amount.",
            );
            let used = domestic_abuse_limit(
                core::mem::take(&mut abuse_claims[owner]),
                &abuse_plans[owner],
                crate::year_constants::DOMESTIC_ABUSE_DISTRIBUTION_LIMIT_CENTS - prior,
                r,
                &mut out,
            );
            crate::kernel::lines::line_cents(
                r,
                if owner == 0 {
                    "domestic_abuse_used_taxpayer"
                } else {
                    "domestic_abuse_used_spouse"
                },
                prior + used,
                &["income.retirement"],
                "https://www.irs.gov/irb/2024-28_IRB",
                msg!("EXPLAIN_DOMESTIC_ABUSE_USED_TAXPAYER"),
            );
        }
    }
    for owner in 0..2 {
        if let Some(prior) = home_prior[owner] {
            let used = gross_exception_limit(
                core::mem::take(&mut home_claims[owner]),
                phase::floor(
                    crate::year_constants::FIRST_HOME_DISTRIBUTION_LIMIT_CENTS
                        - prior
                        - basis.roth.home_used[owner],
                    0,
                ),
                r,
                &mut out,
            );

            crate::kernel::lines::line_cents(
                r,
                if owner == 0 { "first_home_used_taxpayer" } else { "first_home_used_spouse" },
                prior + used,
                &["income.retirement"],
                "https://www.irs.gov/publications/p590b",
                msg!("EXPLAIN_FIRST_HOME_USED_TAXPAYER"),
            );
        }
    }
    out
}

// Native PartI total keeps its existing cents API; owner form taxes are dollars.
pub(crate) struct EarlyDistributionTaxes {
    pub total_cents: i64,
    pub owners: [i64; 2],
}

pub(crate) fn early_tax(
    d: &FederalDraft,
    r: &mut FederalResult,
    basis: &BasisResult,
    qcd: &QcdResult,
    agi: i64,
) -> EarlyDistributionTaxes {
    let care = crate::care_distribution::calculate(d, r, basis);
    // Roth rows: Form 5329 line 1 takes the Form 8606 line 25c earnings, the recapture of
    // 2022-2026 conversions and the line 20 first-home slice (which line 2 excepts as 09); the
    // 25% rate is a Roth SIMPLE IRA's inside its first two years. A row with an exception of its
    // own goes through the stage below (retirement_rows).
    let mut taxable_before = [0, 0];
    let mut after_exception = [[0, 0], [0, 0]];
    for owner in 0..2 {
        taxable_before[owner] =
            basis.roth.penalized[owner] + basis.roth.simple[owner] + basis.roth.slice[owner];
        after_exception[owner] = [basis.roth.penalized[owner], basis.roth.simple[owner]];
    }
    let mut emergency_seen = [false, false];
    let mut medical_expenses = 0;
    let mut medical_taxable = [[0, 0]; 2];
    let mut insurance_exception = false;
    let rows = retirement_rows(d, basis);
    for (e, v) in &rows {
        let v: &crate::FederalRetirement = v;
        // A qualified disaster distribution's taxable part is not an early
        // distribution on Form 5329 line 1 (Form 8915-F line 6).
        let taxable = basis.taxable(e.id, v)
            - basis.converted(e.id, v)
            - qcd.for_entry(e.id)
            - basis.disaster.get(&e.id).copied().unwrap_or(0)
            - if basis.home_repaid.contains(&e.id) {
                // Form 8606 line 7 already left the repayment out of this row's taxable amount.
                0
            } else {
                crate::disaster::home_repaid(v, basis.taxable(e.id, v))
            };
        let taxable = if let RetirementTaxable::LegacyAnnuity { ref data } = v.taxable {
            // Section 72(q)(2)(F): earnings allocated to pre-1982 investment.
            phase::floor(taxable - data.parts(v.gross)[1], 0)
        } else {
            taxable
        };
        // Pretax money rolled into a Roth account is taxed whole, but only the taxable amount
        // kept is an early distribution (sections 402A(c)(4)(B), 408A(d)(3)(A)(ii)); a
        // designated Roth account row already takes the rollover out of its earnings first.
        let taxable = match v.ordinary_distribution {
            RetirementTreatment::Rollover {
                destination: crate::RolloverDestination::Roth,
                amount,
                ..
            } if v.box7 & DistributionCodes::CODE_B.0 == 0 => phase::floor(taxable - amount, 0),
            _ => taxable,
        };
        // Form 5329 line 1 adds the box 10 amount allocable to an in-plan Roth rollover within
        // five years to a designated Roth account distribution (i1099r box 10; i5329 line 1).
        let taxable = if v.box7 & DistributionCodes::CODE_B.0 != 0 {
            taxable + phase::cap(v.box_10, [v.gross])
        } else {
            taxable
        };
        let mut exempt = 0;
        let rate = match &v.ordinary_distribution {
            // A code S distribution kept, in a SIMPLE IRA's first two years, is taxed at 25%
            // (section 72(t)(6)).
            RetirementTreatment::Rollover { early: true, .. }
                if v.box7 & DistributionCodes::CODE_S.0 != 0 =>
            {
                25
            }
            RetirementTreatment::Early
            | RetirementTreatment::Rollover { early: true, .. }
            | RetirementTreatment::Conversion { early: true, .. } => 10,
            RetirementTreatment::EarlySimple => 25,
            RetirementTreatment::Exception { distributed, facts, second, early_simple } => {
                // A prohibited transaction (code 5) is deemed a distribution on January 1 (IRC
                // 408(e)(2)(B)): every date-tested exception is tested on that date, whatever
                // date the page holds.
                let deemed = 20260101;
                let distributed =
                    if v.box7 & DistributionCodes::CODE_5.0 != 0 { &deemed } else { distributed };
                let person: Option<&FederalPerson> = d.people.as_ref().and_then(|p| {
                    if e.owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
                });
                #[cfg(test)]
                let source = alloc::format!("Income: {}", e.label);
                crate::require_localized!(
                    r,
                    !early_simple || v.kind == RetirementKind::Ira,
                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                    &source,
                    "ISSUE_CONFIRM_THE_ELIGIBLE_ACCOUNT_OR_ANNUITY_SOURCE_DATES_AND_EXCEPTION_1865F21C",
                    "Confirm the eligible account or annuity, source dates and exception requirements; Roth recapture and unentered repayment adjustments need another worksheet.",
                );
                // No on the exception's own confirmation: the
                // exception is not applied, so the additional tax applies.
                // The exception, and a second one on the same distribution (Form 5329 line 2
                // enters 99 for two or more): each is tested and counted once.
                for fact in core::iter::once(facts).chain(second.iter()) {
                    let answer = fact.answer();
                    let applied = answer;
                    let valid = !applied
                        || match fact {
                            RetirementException::LongTermCare { .. } => {
                                v.kind == RetirementKind::Pension
                            }
                            RetirementException::Governmental457 { rollover_taxable, .. } => {
                                v.kind == RetirementKind::Pension && *rollover_taxable <= taxable
                            }
                            RetirementException::EsopDividends
                            | RetirementException::FederalPhasedRetirement => {
                                v.kind == RetirementKind::Pension
                            }
                            RetirementException::AutomaticEnrollment {
                                first_contribution,
                                elected,
                                window_days,
                                ..
                            } => {
                                *first_contribution <= *elected
                                    && *elected <= *distributed
                                    && crate::dates::days_between(*first_contribution, *elected)
                                        <= *window_days as i32
                            }
                            RetirementException::DomesticAbuse { event, .. } => {
                                v.kind != RetirementKind::Nonqualified
                                    && *event <= *distributed
                                    && *distributed < *event + 10000
                            }
                            RetirementException::Emergency { history, .. } => {
                                let owner = usize::from(e.owner == Owner::Spouse);
                                crate::require_localized!(
                                    r,
                                    !emergency_seen[owner],
                                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                                    &source,
                                    "ISSUE_ONLY_ONE_ANNUAL_EMERGENCY_DISTRIBUTION_PER_PERSON_IS_PERMITTED_ENTE_4A862D65",
                                    "Only one annual emergency distribution per person is permitted; enter traditional IRA repayments in the owner's basis record.",
                                );
                                emergency_seen[owner] = true;
                                v.kind != RetirementKind::Nonqualified && history.permits()
                            }
                            RetirementException::BirthAdoption { event, .. } => {
                                v.kind != RetirementKind::Nonqualified
                                    && *event <= *distributed
                                    && *distributed < *event + 10000
                            }
                            RetirementException::DomesticRelations => {
                                v.kind == RetirementKind::Pension
                            }
                            RetirementException::Reservist { ordered, through, term, .. } => {
                                *ordered > 20010911
                                    && *ordered <= *distributed
                                    && *distributed <= *through
                                    && match term {
                                        ReserveTerm::Indefinite => true,
                                        ReserveTerm::Days { days } => *days >= 180,
                                    }
                            }
                            RetirementException::FirstHome { paid_on, .. } => {
                                v.kind == RetirementKind::Ira
                                    && *paid_on >= *distributed
                                    && crate::dates::days_between(*distributed, *paid_on) <= 120
                            }
                            RetirementException::Medical { .. } => true,
                            RetirementException::UnemploymentInsurance {
                                benefit_year,
                                weeks,
                                reemployment,
                                ..
                            } => {
                                v.kind == RetirementKind::Ira
                                    && *weeks >= 12
                                    && (*benefit_year == distributed / 10000
                                        || *benefit_year + 1 == distributed / 10000)
                                    && match reemployment {
                                        Reemployment::None => true,
                                        Reemployment::Started { date } => {
                                            crate::dates::days_between(*date, *distributed) <= 60
                                        }
                                    }
                            }
                            RetirementException::Education { .. } => v.kind == RetirementKind::Ira,
                            RetirementException::Disability { began, .. } => *began <= *distributed,
                            RetirementException::Death { died } => *died <= *distributed,
                            RetirementException::TerminalIllness { certified } => {
                                *certified <= *distributed
                            }
                            RetirementException::ImmediateAnnuity {
                                purchased, started, ..
                            } => {
                                v.kind == RetirementKind::Nonqualified
                                    && *purchased <= *started
                                    && *started <= *purchased + 10000
                                    && *started <= *distributed
                                    && match &v.taxable {
                                        RetirementTaxable::GeneralAnnuity { data } => {
                                            data.started == *started
                                        }
                                        RetirementTaxable::VariableAnnuity { data } => {
                                            data.started == *started
                                        }
                                        RetirementTaxable::DeferredWithdrawal { .. }
                                        | RetirementTaxable::LegacyAnnuity { .. } => false,
                                        _ => true,
                                    }
                            }
                            RetirementException::Separation { separated, work } => {
                                v.kind == RetirementKind::Pension
                                    && *separated <= *distributed
                                    && person
                                        .is_some_and(|p| work.qualifies(p.birth_date, *separated))
                            }
                            RetirementException::Levy { .. } => {
                                v.kind != RetirementKind::Nonqualified
                            }
                            RetirementException::PeriodicPayments { first_payment, .. } => {
                                *first_payment <= *distributed
                            }
                            // The payer's code 1 was wrong: the owner had turned 59½ by the payment
                            // (Form 5329 line 2, exception 12).
                            // Code 5 is deemed paid on January 1 whatever the page holds: an owner
                            // not yet 59½ that day simply gets no exception.
                            RetirementException::ReachedAge => {
                                v.box7 & DistributionCodes::CODE_5.0 != 0
                                    || person.is_some_and(|p| {
                                        *distributed >= crate::roth::age_half(p.birth_date)
                                    })
                            }
                        };
                    // Pub. 590-B, Recapture tax for changes in distribution method: a change in a
                    // series of substantially equal periodic payments adds the earlier years' 10%
                    // with interest, which is not calculated.
                    crate::require_localized!(
                        r,
                        !(applied
                            && matches!(
                                fact,
                                RetirementException::PeriodicPayments { modified: true, .. }
                            )),
                        &crate::interview::income_issue_context(e),
                        &source,
                        "ISSUE_SEPP_RECAPTURE",
                        "A change in a series of substantially equal periodic payments before the later of five years after the first payment and age 59½ adds the earlier years' 10% additional tax with interest, which is not calculated.",
                    );
                    crate::require_localized!(
                        r,
                        valid,
                        &crate::IssueContext::Section(crate::TaxSection::Retirement),
                        &source,
                        "ISSUE_THE_EXCEPTION_S_DATE_ACCOUNT_TYPE_EMPLOYMENT_ELIGIBILITY_OR_SEPARAT_9E1325F3",
                        "The exception's date, account type, employment eligibility or separation age/service does not qualify for this distribution.",
                    );
                    let part = if !applied {
                        0
                    } else {
                        match fact {
                            RetirementException::LongTermCare { .. } => {
                                care.get(&e.id).copied().unwrap_or(0)
                            }
                            RetirementException::Governmental457 { rollover_taxable, .. } => {
                                phase::floor(taxable - rollover_taxable, 0)
                            }
                            RetirementException::DomesticAbuse { .. }
                            | RetirementException::Emergency { .. } => {
                                basis.gross.amounts.get(&e.id).map_or(0, |v| v.1)
                            }
                            RetirementException::BirthAdoption { .. }
                            | RetirementException::FirstHome { .. } => {
                                basis.gross.amounts.get(&e.id).map_or(0, |v| v.1)
                            }
                            RetirementException::Medical { expenses } => {
                                medical_expenses += expenses;
                                0
                            }
                            RetirementException::UnemploymentInsurance {
                                premiums,
                                qualification,
                                ..
                            } => {
                                let amount = phase::cap(*premiums, [taxable]);
                                crate::kernel::lines::line_cents(
                                    r,
                                    &alloc::format!("insurance_exception_{}", e.id),
                                    amount,
                                    &["income.retirement.ordinary_distribution"],
                                    "https://www.irs.gov/irb/2026-06_IRB",
                                    match qualification {
                                        UnemploymentQualification::Received { .. } => {
                                            msg!("EXPLAIN_INSURANCE_EXCEPTION_N")
                                        }
                                        UnemploymentQualification::SelfEmployed { .. } => {
                                            msg!("EXPLAIN_INSURANCE_EXCEPTION_N_2")
                                        }
                                    },
                                );
                                amount
                            }
                            RetirementException::Education {
                                education,
                                room_board,
                                allowance,
                                school_housing,
                                half_time,
                                assistance,
                                ..
                            } => {
                                let housing = if *half_time {
                                    phase::cap(
                                        *room_board,
                                        [phase::floor(*allowance, *school_housing)],
                                    )
                                } else {
                                    0
                                };
                                phase::cap(
                                    phase::floor(education + housing - assistance, 0),
                                    [taxable],
                                )
                            }
                            RetirementException::Levy { .. } => {
                                basis.gross.amounts.get(&e.id).map_or(0, |v| v.1)
                            }
                            RetirementException::ReachedAge
                                if !person.is_some_and(|p| {
                                    *distributed >= crate::roth::age_half(p.birth_date)
                                }) =>
                            {
                                0
                            }
                            _ => taxable,
                        }
                    };
                    exempt += part;
                }
                exempt = phase::cap(exempt, [taxable]);
                if *early_simple { 25 } else { 10 }
            }
            _ => 0,
        };
        // Exception 12 (i5329 line 2; IRC 72(t)(2)(A)(i)): a distribution a payer coded 1 or S (or
        // J) that the owner received at 59½ or older is never early, whatever other exception
        // was entered. The page asks no date for a plain early distribution, so it applies
        // when the owner was 59½ before 2026 began; an entered exception carries its own date
        // (January 1 for code 5).
        let received = match &v.ordinary_distribution {
            RetirementTreatment::Exception { distributed, .. } => {
                if v.box7 & DistributionCodes::CODE_5.0 != 0 { 20260101 } else { *distributed }
            }
            _ => 20260101,
        };
        let age_exempt = rate > 0
            && d.people.as_ref().is_some_and(|p| {
                let person =
                    if e.owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() };
                person.is_some_and(|p| crate::roth::age_half(p.birth_date) <= received)
            });
        if age_exempt {
            exempt = taxable;
        }
        let owner = usize::from(e.owner == Owner::Spouse);
        if rate > 0 {
            taxable_before[owner] += taxable;
        }
        // Medical expenses above 7.5% of AGI are allocated once over the return (below); what a
        // second exception on the same distribution already covers is not theirs.
        let medical = match &v.ordinary_distribution {
            RetirementTreatment::Exception { facts, second, .. } => core::iter::once(facts)
                .chain(second.iter())
                .any(|f| matches!(f, RetirementException::Medical { .. })),
            _ => false,
        };
        if !age_exempt && medical {
            if rate > 0 {
                medical_taxable[owner][usize::from(rate == 25)] += taxable - exempt;
            }
        } else {
            insurance_exception |= match &v.ordinary_distribution {
                RetirementTreatment::Exception { facts, second, .. } => core::iter::once(facts)
                    .chain(second.iter())
                    .any(|f| matches!(f, RetirementException::UnemploymentInsurance { .. })),
                _ => false,
            };
            if rate > 0 {
                after_exception[owner][usize::from(rate == 25)] += taxable - exempt;
            }
        }
    }
    crate::require_localized!(
        r,
        medical_expenses == 0 || !insurance_exception,
        &crate::IssueContext::Section(crate::TaxSection::Retirement).manual(),
        "Retirement medical expenses",
        "ISSUE_COMBINED_MEDICAL_EXPENSE_AND_UNEMPLOYMENT_INSURANCE_EXCEPTIONS_NEED_05A37B66",
        "Combined medical-expense and unemployment-insurance exceptions need allocation of overlapping premiums before calculation.",
    );
    let medical_remaining = medical_exception_allocation(medical_taxable, medical_expenses, agi);
    for owner in 0..2 {
        for rate in 0..2 {
            after_exception[owner][rate] += medical_remaining[owner][rate];
        }
    }
    for (owner, amount) in basis.repayment_excluded.iter().enumerate() {
        taxable_before[owner] = phase::floor(taxable_before[owner] - amount, 0);
    }
    // Each spouse files a separate Form 5329. Aggregate source cents before
    // rounding each form line, then add the separately rounded taxes.
    let mut total = 0;
    let mut owners = [0; 2];
    for owner in 0..2 {
        if taxable_before[owner] == 0 {
            continue;
        }
        let [taxable, exceptions, subject, tax] =
            early_tax_lines(taxable_before[owner], after_exception[owner]);
        total += tax;
        owners[owner] = tax;
        emit_lines(
            r,
            &mut (),
            [("taxable", taxable), ("exceptions", exceptions), ("subject", subject), ("tax", tax)],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!(
                        "early_{}_{suffix}",
                        if owner == 0 { "taxpayer" } else { "spouse" }
                    ),
                    LineValue::MoneyDollars(amount),
                    &["income.retirement", "income.roth"],
                    "https://www.irs.gov/instructions/i5329",
                    msg!("EXPLAIN_EARLY_FILER"),
                ))
            },
        );
    }
    EarlyDistributionTaxes { total_cents: total * 100, owners }
}

/// Trustee payment directly to an eligible public charity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QcdPayment {
    /// Inherited traditional IRA income entry ID, if applicable
    /// Leave absent for personal traditional IRAs. Otherwise use the income entry ID for this beneficiary and deceased owner. The gift must be included in that entry's gross distributions; never use another filer's entry or another deceased owner's basis.
    #[libertas_number(min = 1)]
    pub inherited: Option<i64>,
    /// Date paid in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Cash amount the IRA trustee paid directly
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
    /// Paid to a split-interest entity under the one-time election
    /// Counts within the annual limit and within the $55,000 one-time limit.
    pub split_interest: bool,
}
/// Qualified charitable distributions for one filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QualifiedCharity {
    /// Filer receiving the IRA distributions
    pub owner: Owner,
    /// All direct trustee payments for this filer, including inherited IRAs
    #[libertas_size(min = 1)]
    pub payments: alloc::vec::Vec<QcdPayment>,
    /// Unused post-age-70½ deductible contributions from the prior QCD adjustment worksheet
    /// Use the carryforward from the latest worksheet. For a first worksheet, use the aggregate deductions on prior filed returns beginning with the year you were 70½ at year-end, but not before 2020. Enter zero if none.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_deductions: i64,
}
impl QualifiedCharity {
    pub(crate) fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.prior_deductions)
            && !self.payments.is_empty()
            && self.payments.len() <= 100
            && self.payments.iter().all(|p| {
                p.inherited.is_none_or(|id| id > 0)
                    && valid_birth(p.date)
                    && p.date / 10000 == 2026
                    && (0..=100000000000).contains(&p.amount)
            })
    }
}
fn age_seventy_half(birth: u32) -> u32 {
    let mut year = birth / 10000 + 70;
    let mut month = birth / 100 % 100 + 6;
    if month > 12 {
        month -= 12;
        year += 1;
    }
    let mut day = birth % 100;
    while day > 28 && !valid_birth(year * 10000 + month * 100 + day) {
        day -= 1;
    }
    year * 10000 + month * 100 + day
}
// Returns excluded IRA distributions and nonexcluded cash gifts, in source cents.
#[derive(Default)]
pub(crate) struct QcdResult {
    pub excluded: i64,
    pub excluded_by_owner: [i64; 2],
    pub cash_gifts: i64,
    qualified: [i64; 2],
    by_entry: alloc::collections::BTreeMap<i64, i64>,
    exclusions: alloc::collections::BTreeMap<i64, Option<i64>>,
}
impl QcdResult {
    pub(crate) fn exclusion_for_entry(&self, id: i64) -> Option<i64> {
        self.exclusions.get(&id).copied().unwrap_or(Some(0))
    }
    pub(crate) fn for_entry(&self, id: i64) -> i64 {
        self.by_entry.get(&id).copied().unwrap_or(0)
    }
}
pub(crate) fn qualified_charity(
    d: &FederalDraft,
    r: &mut FederalResult,
    contributions: Option<&crate::ira::IraResult>,
    funding: &crate::hsa_funding::FundingResult,
) -> QcdResult {
    use crate::kernel::lines::line;
    use crate::kernel::ratio::{dollars, positive};
    let Some(a) = &d.adjustments else {
        return QcdResult::default();
    };
    let mut excluded = 0;
    let mut cash = 0;
    let mut carry = 0;
    let mut result = QcdResult::default();
    for (index, q) in a.qualified_charity.iter().enumerate() {
        let owner = usize::from(q.owner == Owner::Spouse);
        let person = d
            .people
            .as_ref()
            .and_then(|p| if owner == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() });
        let mut ira = exact::whole(0);
        let mut pretax = true;
        for e in &d.income {
            if e.owner == q.owner
                && let FederalIncome::Retirement { data: v } = &e.income
                && v.kind == crate::RetirementKind::Ira
                && !v.outside_form_8606()
            {
                ira += (exact::whole(v.taxable_income())
                    - exact::whole(v.converted_amount())
                    - exact::whole(funding.for_entry(e.id)))
                .max(exact::whole(0));
                pretax &= v.taxable == RetirementTaxable::Payer { amount: v.gross }
                    || matches!(v.ordinary_distribution, RetirementTreatment::Rollover { amount, .. } if amount == v.gross && v.taxable == RetirementTaxable::Payer { amount: 0 });
            }
        }
        let current_contribution = a.entered_iras().map_or(0, |f| {
            if owner == 0 {
                f.taxpayer.traditional
            } else {
                f.spouse.iter().map(|v| v.traditional).sum()
            }
        });
        // Start with full deductibility; a second candidate may use the IRA
        // worksheet result. The caller independently verifies either candidate.
        let nondeductible = contributions.map_or(0, |v| v.nondeductible[owner]);
        let current_deduction = positive(dollars(current_contribution) - nondeductible) * 100;
        let payments: i64 = q.payments.iter().map(|p| p.amount).sum();
        let basis = a.ira_basis.iter().find(|b| b.owner == q.owner);
        // Section 408(d)(8)(D): charitable transfers use the aggregate pretax
        // portion first, before applying ordinary section 72 basis allocation.
        let pretax_capacity = basis.map_or_else(
            || ira.clone(),
            |b| {
                (b.adjusted_year_end_exact(d) - b.line_2_exact()
                    + exact::whole(funding.basis_used[owner])
                    - incoming_plan_basis(d, q.owner)
                    - b.reservist_basis_exact()
                    - exact::whole(available_contribution_basis(
                        b,
                        current_contribution,
                        contributions.map_or(0, |v| v.absorbed[owner]),
                        nondeductible,
                    )) * exact::whole(100)
                    + d.income
                        .iter()
                        .filter_map(|e| {
                            if e.owner == q.owner
                                && let FederalIncome::Retirement { data } = &e.income
                                && data.kind == crate::RetirementKind::Ira
                            {
                                Some(
                                    exact::whole(data.gross) - exact::whole(data.rolled_amount())
                                        + exact::whole(data.outstanding_rollover())
                                        - exact::whole(funding.for_entry(e.id)),
                                )
                            } else {
                                None
                            }
                        })
                        .sum::<Exact>())
                .max(exact::whole(0))
            },
        );
        let personal_payments: i64 =
            q.payments.iter().filter(|p| p.inherited.is_none()).map(|p| p.amount).sum();
        let qualified = exact::rounded(&exact::whole(personal_payments).min(pretax_capacity))
            .expect("QCD exclusion bounded by actual native payments");
        result.qualified[owner] += qualified;
        let mut entries: alloc::vec::Vec<_> =
            d.income.iter().filter(|e| e.owner == q.owner).collect();
        entries.sort_by_key(|e| e.id);
        let mut remaining_qualified = qualified;
        for e in entries {
            if let FederalIncome::Retirement { data } = &e.income
                && data.kind == crate::RetirementKind::Ira
            {
                let assigned = crate::kernel::carry::native_consume_step(
                    positive(
                        data.taxable_income() - data.converted_amount() - funding.for_entry(e.id),
                    ),
                    &mut remaining_qualified,
                );
                result.by_entry.insert(e.id, assigned);
            }
        }
        crate::require_localized!(
            r,
            (pretax || basis.is_some())
                && exact::whole(personal_payments) <= ira
                && person.is_some()
                && !a.qualified_charity[..index].iter().any(|v| v.owner == q.owner),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "QCD",
            "ISSUE_USE_ONE_COMPLETE_QCD_RECORD_PER_FILER_RECONCILE_GIFTS_TO_ENTERED_IR_167E808F",
            "Use one QCD record per filer and reconcile gifts to entered IRA distributions. Personal after-tax sources need the owner-level basis worksheet; inherited basis stays in its linked income entry.",
        );
        crate::require_localized!(
            r,
            person.is_some_and(|p| {
                q.payments.iter().all(|v| v.date >= age_seventy_half(p.birth_date))
            }),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "QCD",
            "ISSUE_EACH_TRUSTEE_PAYMENT_MUST_BE_MADE_ON_OR_AFTER_THE_RECEIVING_FILER_S_17F2FD62",
            "Each trustee payment must be made on or after the receiving filer's actual age-70½ date, including gifts from inherited IRAs.",
        );
        // Inherited basis never joins personal IRAs or another decedent's group.
        // Notice 2007-7 Q&A-37 permits QCDs at the beneficiary's age 70½.
        let mut inherited_payments = alloc::collections::BTreeMap::<i64, i64>::new();
        for payment in &q.payments {
            if let Some(id) = payment.inherited {
                *inherited_payments.entry(id).or_default() += payment.amount;
            }
        }
        let mut inherited_qualified = 0;
        for (id, amount) in inherited_payments {
            let source = d.income.iter().find(|e| e.id == id && e.owner == q.owner);
            if let Some(crate::FederalIncomeEntry {
                income: FederalIncome::InheritedIra { data },
                ..
            }) = source
                && !data.roth()
            {
                let group = crate::inherited_ira::gross(d, id);
                crate::require_localized!(
                    r,
                    amount <= phase::floor(group - funding.for_inherited(id), 0)
                        && q.payments
                            .iter()
                            .filter(|p| p.inherited == Some(id))
                            .all(|p| p.date >= data.died),
                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                    "QCD",
                    "ISSUE_INHERITED_GIFTS_MUST_FOLLOW_THE_ORIGINAL_OWNER_S_DEATH_AND_BE_INCLU_434B63F8",
                    "Inherited gifts must follow the original owner's death and be included in the linked group's gross beneficiary distributions.",
                );
                let available = phase::floor(group - funding.for_inherited(id), 0);
                let capacity = data.basis().map_or(available, |b| {
                    positive(
                        available + b.year_end - b.prior
                            + funding.inherited_basis_used.get(&id).copied().unwrap_or(0),
                    )
                });
                let assigned = phase::cap(phase::cap(amount, [available]), [capacity]);
                result.by_entry.insert(id, assigned);
                inherited_qualified += assigned;
            } else {
                crate::require_localized!(
                    r,
                    false,
                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                    "QCD",
                    "ISSUE_LINK_EACH_INHERITED_GIFT_TO_AN_EXISTING_INHERITED_TRADITIONAL_IRA_I_4140777A",
                    "Link each inherited gift to an existing inherited traditional IRA income entry belonging to this filer.",
                );
            }
        }
        let capped =
            phase::cap(qualified + inherited_qualified, [crate::year_constants::QCD_LIMIT_CENTS]);
        let adjustment = q.prior_deductions + current_deduction;
        let allowed = positive(capped - adjustment);
        let sources: alloc::vec::Vec<_> =
            d.income.iter().filter(|e| e.owner == q.owner && result.for_entry(e.id) > 0).collect();
        for entry in &sources {
            let source_exclusion = if allowed == 0 {
                Some(0)
            } else if allowed == qualified + inherited_qualified {
                Some(result.for_entry(entry.id))
            } else if sources.len() == 1 {
                Some(allowed)
            } else {
                None
            };
            result.exclusions.insert(entry.id, source_exclusion);
        }
        let remaining = crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
            adjustment, capped,
        )
        .get();
        excluded += allowed;
        result.excluded_by_owner[owner] += allowed;
        cash += payments - allowed;
        carry += remaining;
        line(
            r,
            if owner == 0 { "qcd_taxpayer_exclusion" } else { "qcd_spouse_exclusion" },
            dollars(allowed),
            &["adjustments.qualified_charity", "income.retirement"],
            "https://www.irs.gov/publications/p590b",
            msg!("EXPLAIN_QCD_TAXPAYER_EXCLUSION"),
        );
        line(
            r,
            if owner == 0 {
                "qcd_taxpayer_adjustment_carry"
            } else {
                "qcd_spouse_adjustment_carry"
            },
            dollars(remaining),
            &["adjustments.qualified_charity"],
            "https://www.irs.gov/publications/p590b",
            msg!("EXPLAIN_QCD_TAXPAYER_ADJUSTMENT_CARRY"),
        );
    }
    emit_lines(
        r,
        &mut (),
        [
            ("qcd_exclusion", excluded, msg!("EXPLAIN_QCD_EXCLUSION")),
            ("qcd_cash_gifts", cash, msg!("EXPLAIN_QCD_CASH_GIFTS")),
            ("qcd_adjustment_carryforward", carry, msg!("EXPLAIN_QCD_ADJUSTMENT_CARRYFORWARD")),
        ],
        |(id, value, explanation), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(dollars(value)),
                &["adjustments.qualified_charity"],
                "https://www.irs.gov/publications/p590b",
                explanation,
            ))
        },
    );
    result.excluded = excluded;
    result.cash_gifts = cash;
    result
}

/// Annuity value and investment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AnnuityBalance {
    /// Cash value before this withdrawal, ignoring surrender charges
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub value: i64,
    /// Unrecovered investment before this withdrawal
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 1)]
    pub value: i64,
    /// Unrecovered after-tax investment in that same account or contract
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    pub value: i64,
    /// Unrecovered after-tax investment immediately before this event
    /// Use contract tax-basis records, reflecting earlier tax-free withdrawals and other documented adjustments. Do not use market value or the original premium when basis has changed.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Earlier tax-free annuity recovery from this cost
    /// Include every recipient's earlier recovery, including the final annuitant's payments in the current year. Enter zero if no survivor annuity was paid. Do not include taxable annuity payments.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
        /// Prior-year repayments from box 8
        repayments: Vec<crate::RailroadRepayment>,
        /// Box 4 contributory amount paid
        /// Included in box 7 gross entered above. Boxes 5 and 6 are fully taxable and cannot absorb an exclusion from employee contributions.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        contributory: i64,
        /// Box 5 vested dual benefit
        /// Included in box 7 gross entered above; fully taxable.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        box_5: i64,
        /// Box 6 supplemental annuity
        /// Included in box 7 gross entered above; fully taxable.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        box_6: i64,
        /// Whose annuity this is
        holder: RailroadHolder,
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
    Payer {
        /// Form 1099-R box 2a
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
    },
    /// Calculate traditional IRA basis allocation
    /// Choose this for every traditional IRA distribution belonging to this filer. Supply the prior basis and all year-end IRA balances in Adjustments.
    IraBasis,
    /// Inherited traditional IRA, taxed with the other payments from the same deceased owner
    /// Form 1099-R code 4 with box 7b checked. The account is the income identity of the Inherited IRA record that holds the allocated basis and estate-tax facts; absent uses the filer's only inherited traditional IRA. The payer's box 2a is kept for a payment that has no record to use.
    Inherited {
        /// Inherited IRA record
        #[libertas_number(min = 1)]
        account: Option<i64>,
        /// Box 2a taxable amount the payer printed
        /// Taxed as printed when no Inherited IRA record can be used for this payment.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        payer: i64,
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

/// Traditional IRA basis for one filer
/// All this filer's IRA distributions use the traditional IRA basis allocation. Conversions, eligible rollovers, the QCD gifts and prior-basis HSA funding are supported; inherited accounts have their own record. Early-distribution exceptions use the taxable amount after this allocation.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraBasis {
    /// Account owner
    pub owner: Owner,
    /// Unused basis from the most recently filed Form 8606 line 14
    /// Use zero if no nondeductible contributions remain. For a form before 2001 use the Total Basis Chart of the instructions. Do not add after-tax employer-plan rollovers entered in current-year income; the App adds those automatically. The Form 8606 line 2 adjustment is entered with the Form 8606.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior: i64,
    /// Adjustment to that basis, Form 8606 line 2
    /// Line 6 of the last Line 15c Worksheet, basis received or transferred in a divorce, a nontaxable plan rollover never reported, a returned excess contribution. Signed.
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub adjustment: i64,
    /// December 31, 2026 value of all this filer's traditional IRAs
    /// Combine traditional, SEP and SIMPLE accounts, including accounts with no distributions. Do not include Roth or inherited accounts or a spouse's accounts.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub year_end: i64,
    /// Contributions for 2026 paid after December 31, 2026
    /// Included in the traditional contribution total entered below. Include all completed 2027 traditional deposits entered there and any planned timely payments by the filing deadline; none shelter a 2026 distribution.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub paid_next_year: i64,
    /// Qualified reservist repayments deposited in 2026
    /// Adds nondeductible basis outside regular contribution limits. Include the deposits in year-end value, but do not add them to prior basis or regular contributions. Leave empty when none.
    /// ----
    /// Original distribution and its repayments
    pub reservist_repayments: alloc::vec::Vec<crate::ReservistRepayment>,
    /// Repayments of special 2026 IRA distributions
    /// Birth/adoption, emergency, domestic-abuse and terminal-illness repayments use their separate worksheet. Leave empty when none. Keep the original distributions and actual year-end balance unchanged in their source entries.
    /// ----
    /// Original distribution and accepted repayments
    pub special_repayments: alloc::vec::Vec<crate::IraRepayment>,
}
// The calculation and form owners report Dollars as native Cents. A failure
// belongs to the actual reported amount, not an exact source subtotal.
#[derive(Clone, Copy)]
pub(crate) struct IraAmountIssue;
impl IraAmountIssue {
    pub(crate) fn reported(dollars: i64) -> Result<i64, Self> {
        dollars.checked_mul(100).map(|_| dollars).ok_or(Self)
    }
    pub(crate) fn report(self, r: &mut FederalResult) {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "IRA basis",
            "IRA_REPAYMENT_AMOUNT_UNREPRESENTABLE",
            "An IRA basis or repayment amount cannot be represented in the native reported result. Review the original distribution and repayment sources.",
        );
        crate::federal_rules::withhold_required_source_result(r);
    }
}
impl IraBasis {
    // Form 8606 line 2: the last form's line 14 and the adjustment.
    pub(crate) fn opening_basis_exact(prior: i64, adjustment: i64) -> Exact {
        exact::whole(prior) + exact::whole(adjustment)
    }
    pub(crate) fn form_5329_value_exact(&self) -> Exact {
        exact::whole(self.year_end) + exact::whole(self.paid_next_year)
    }
    pub(crate) fn line_2_exact(&self) -> Exact {
        Self::opening_basis_exact(self.prior, self.adjustment)
    }
    pub(crate) fn reservist_basis_exact(&self) -> Exact {
        self.reservist_repayments.iter().map(|p| p.amount_exact()).sum()
    }
    pub(crate) fn valid(&self) -> bool {
        self.prior >= 0
            && self.line_2_exact() >= exact::whole(0)
            && self.year_end >= 0
            && self.paid_next_year >= 0
            && self.reservist_repayments.iter().all(|p| p.valid())
            && self.special_repayments.iter().all(|p| p.valid())
    }
}
#[derive(Default)]
pub(crate) struct BasisResult {
    pub gross: GrossExceptions,
    pub regular_gross: [i64; 2],
    pub regular_taxable: [i64; 2],
    pub repayment_excluded: [i64; 2],
    pub inherited_basis_used: alloc::collections::BTreeMap<i64, i64>,
    pub inherited: crate::inherited_ira::InheritedResult,
    pub public_safety: crate::public_safety::PublicSafetyResult,
    pub hsa_funding: alloc::collections::BTreeMap<i64, i64>,
    pub inherited_hsa_funding: alloc::collections::BTreeMap<i64, i64>,
    pub roth: crate::roth::RothResult,
    // Taxable part of each 2026 qualified disaster distribution (Form 8915-F).
    pub disaster: alloc::collections::BTreeMap<i64, i64>,
    // The Part IV home-purchase rows whose repaid amount Form 8606 line 7 left out: their
    // allocated taxable amount is already net of the repayment.
    pub home_repaid: alloc::collections::BTreeSet<i64>,
    taxable: alloc::collections::BTreeMap<i64, i64>,
    converted: alloc::collections::BTreeMap<i64, i64>,
}

// The funding source is an actual accepted payment, while its inherited
// basis belongs to the existing beneficiary/decedent group. Never substitute
// an annual group for the payment or create a second IRA pool identity.
pub(crate) struct IraFundingSource<'a> {
    pub entry: &'a FederalIncomeEntry,
    pub inherited: Option<(i64, &'a crate::FederalInheritedIra)>,
}

pub(crate) fn ira_funding_source(d: &FederalDraft, id: i64) -> Option<IraFundingSource<'_>> {
    let entry = d.income.iter().find(|entry| entry.id == id)?;
    if !matches!(&entry.income,
        FederalIncome::Retirement { data }
            if matches!(data.kind, RetirementKind::Ira | RetirementKind::Inherited))
        && !matches!(entry.income, FederalIncome::Roth { .. })
    {
        return None;
    }
    let inherited = match crate::inherited_ira::link_of(d, entry) {
        None => None,
        Some(crate::inherited_ira::Link::Record(group)) => {
            let group_entry = d.income.iter().find(|entry| entry.id == group)?;
            let FederalIncome::InheritedIra { data } = &group_entry.income else {
                return None;
            };
            Some((group, data))
        }
        Some(_) => return None,
    };
    // An inherited payment without its canonical inherited link is not an
    // own-account source merely because it has a retirement gross amount.
    if matches!(&entry.income, FederalIncome::Retirement { data }
        if data.kind == RetirementKind::Inherited && inherited.is_none())
    {
        return None;
    }
    Some(IraFundingSource { entry, inherited })
}
impl BasisResult {
    pub(crate) fn converted(&self, id: i64, v: &crate::FederalRetirement) -> i64 {
        self.converted.get(&id).copied().unwrap_or(v.converted_amount())
    }
    pub(crate) fn taxable(&self, id: i64, v: &crate::FederalRetirement) -> i64 {
        self.taxable.get(&id).copied().unwrap_or_else(|| v.taxable_income())
            - self.public_safety.excluded(id)
            - self.hsa_funding.get(&id).copied().unwrap_or(0)
    }
}
// After-tax employer-plan rollovers deposited into a traditional IRA this year
// increase Form 8606 basis without being regular annual contributions.
fn incoming_plan_basis(d: &FederalDraft, owner: Owner) -> Exact {
    d.income
        .iter()
        .filter_map(|entry| {
            if entry.owner != owner {
                return None;
            }
            let FederalIncome::Retirement { data } = &entry.income else {
                return None;
            };
            if data.kind == RetirementKind::Pension
                && matches!(
                    data.taxable,
                    RetirementTaxable::QualifiedWithdrawal {
                        data: PlanDistribution { eligible: true, .. }
                    } | RetirementTaxable::FullDischarge {
                        data: ContractDischarge { eligible: true, .. }
                    }
                )
                && matches!(
                    data.ordinary_distribution,
                    RetirementTreatment::Rollover {
                        destination: crate::RolloverDestination::Ira,
                        ..
                    }
                )
            {
                Some(exact::whole(data.rolled_basis()))
            } else {
                None
            }
        })
        .sum()
}

// Current-year basis excludes nondeductible contributions paid the following year.
// Both the QCD pretax-first rule and ordinary allocation use this same amount.
fn available_contribution_basis(
    b: &IraBasis,
    contribution: i64,
    absorbed: i64,
    nondeductible: i64,
) -> i64 {
    use crate::kernel::ratio::dollars;
    let later = dollars(b.paid_next_year);
    // Form5329 line10 applies an existing prior-year excess to this year. It
    // is already in the IRA, not a payment made next year. IraResult includes
    // it in permitted contributions and Form8606 line1; retain its source cents
    // until the same whole-dollar rounding as that owner.
    let earlier = exact::dollars(&(exact::whole(contribution) + exact::whole(absorbed)))
        .expect("contributions and absorbed excess retain their native source bounds")
        - later;
    // Favorable order: nondeductible basis uses amounts already in the IRA first.
    let nondeductible_later = phase::floor(nondeductible - earlier, 0);
    nondeductible - nondeductible_later
}

pub(crate) fn basis(
    d: &FederalDraft,
    r: &mut FederalResult,
    contribution_result: Option<&crate::ira::IraResult>,
    qcd: &QcdResult,
    funding: &crate::hsa_funding::FundingResult,
) -> Result<BasisResult, IraAmountIssue> {
    use crate::kernel::ratio::dollars;
    let mut result = BasisResult::default();
    let Some(a) = &d.adjustments else {
        return Ok(result);
    };
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        crate::require_localized!(
            r,
            incoming_plan_basis(d, owner) == exact::whole(0)
                || a.ira_basis.iter().any(|b| b.owner == owner),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "IRA basis",
            "ISSUE_PROVIDE_THIS_RECIPIENT_S_IRA_BASIS_RECORD_AFTER_AN_EMPLOYER_PLAN_RO_E8156654",
            "Provide this recipient's IRA basis record after an employer-plan rollover of after-tax contributions. The App adds the incoming basis automatically; do not include it in prior basis.",
        );
    }
    for (index, b) in a.ira_basis.iter().enumerate() {
        let entries: alloc::vec::Vec<_> = d
            .income
            .iter()
            .filter_map(|e| {
                if e.owner != b.owner {
                    return None;
                }
                match &e.income {
                    FederalIncome::Retirement { data }
                        if data.kind == crate::RetirementKind::Ira && !data.outside_form_8606() =>
                    {
                        Some((e, data))
                    }
                    _ => None,
                }
            })
            .collect();
        let contribution_fact = a.entered_iras().and_then(|f| {
            if b.owner == Owner::Taxpayer { Some(&f.taxpayer) } else { f.spouse.as_ref() }
        });
        let contribution = contribution_fact.map_or(0, |v| v.traditional);
        let actually_deposited_next_year = contribution_fact
            .and_then(|f| f.next_year_deposits.as_ref())
            .map_or(0, |d| d.traditional);
        let valid = !a.ira_basis[..index].iter().any(|v| v.owner == b.owner)
            && (b.owner == Owner::Taxpayer
                || d.people.as_ref().is_some_and(|p| p.spouse.is_some()))
            && b.paid_next_year <= contribution
            && b.paid_next_year >= actually_deposited_next_year
            && entries.iter().all(|(_, v)| {
                v.taxable == RetirementTaxable::IraBasis
                    && match &v.ordinary_distribution {
                        RetirementTreatment::Ordinary
                        | RetirementTreatment::Early
                        | RetirementTreatment::EarlySimple
                        | RetirementTreatment::Conversion { .. }
                        | RetirementTreatment::Rollover { .. } => true,
                        // Form 5329 uses the allocated taxable amount. Its existing
                        // exception stage owns eligibility and gross-limit checks.
                        // A deceased owner's basis never joins personal IRA basis.
                        RetirementTreatment::Exception { facts, .. } => {
                            !matches!(facts, RetirementException::Death { .. })
                        }
                        _ => false,
                    }
            });
        crate::require_localized!(
            r,
            valid,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "IRA basis",
            "ISSUE_USE_ONE_COMPLETE_BASIS_RECORD_PER_FILER_AND_SELECT_THE_IRA_WORKSHEE_F3EC6BF8",
            "Use one complete basis record per filer and select the IRA worksheet for all their traditional IRA distributions. Contributions paid after 2026 must include every actual 2027 traditional deposit designated for 2026 and cannot exceed retained traditional contributions. Inherited accounts use their separate deceased-owner record; changed basis requires additional coordination. Exception eligibility and any gross-distribution limits are checked separately.",
        );
        if !valid {
            continue;
        }
        if b.adjustment != 0 {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("NOTE_FORM_8606_DIVORCE_STATEMENT"),
                crate::IssueContext::owner(crate::TaxSection::Retirement, b.owner),
            );
        }
        let owner_name = if b.owner == Owner::Taxpayer { "taxpayer" } else { "spouse" };
        for (i, payment) in b.reservist_repayments.iter().enumerate() {
            crate::require_localized!(
                r,
                !b.reservist_repayments[..i].iter().any(|p| p.distributed == payment.distributed),
                &crate::IssueContext::Review,
                "Reservist IRA repayment",
                "ISSUE_USE_ONE_RECORD_PER_ORIGINAL_DISTRIBUTION_AND_COMBINE_ITS_2026_PAYME_8B66531C",
                "Use one record per original distribution and combine its 2026 payments; do not duplicate the repayment ceiling.",
            );
            payment.report(r, owner_name, i + 1);
        }
        let gross_cents = entries
            .iter()
            .map(|(e, v)| {
                exact::whole(v.gross)
                    - exact::whole(v.rolled_amount())
                    - exact::whole(funding.for_entry(e.id))
                    - exact::whole(v.home_repaid())
            })
            .sum::<Exact>();
        let outstanding =
            entries.iter().map(|(_, v)| exact::whole(v.outstanding_rollover())).sum::<Exact>();
        let converted_cents =
            entries.iter().map(|(_, v)| exact::whole(v.converted_amount())).sum::<Exact>();
        let qcd_qualified = qcd.qualified[usize::from(b.owner == Owner::Spouse)];
        let regular_cents =
            (gross_cents - &converted_cents - exact::whole(qcd_qualified)).max(exact::whole(0));
        let converted_gross = exact::dollars(&converted_cents).ok_or(IraAmountIssue)?;
        let regular_gross = exact::dollars(&regular_cents).ok_or(IraAmountIssue)?;
        let gross = regular_gross.checked_add(converted_gross).ok_or(IraAmountIssue)?;
        let incoming = incoming_plan_basis(d, b.owner);
        // These are independently reported categories: preserve both existing
        // whole-Dollar boundaries before adding their reported amounts.
        let opening = exact::dollars(
            &(b.line_2_exact()
                - exact::whole(funding.basis_used[usize::from(b.owner == Owner::Spouse)])
                + &incoming),
        )
        .ok_or(IraAmountIssue)?;
        let reservist = exact::dollars(&b.reservist_basis_exact()).ok_or(IraAmountIssue)?;
        let prior = opening.checked_add(reservist).ok_or(IraAmountIssue)?;
        let year_end =
            exact::dollars(&(b.adjusted_year_end_exact(d) + &outstanding)).ok_or(IraAmountIssue)?;
        let denominator = year_end.checked_add(gross).ok_or(IraAmountIssue)?;
        let limited = a
            .entered_iras()
            .zip(d.setup.as_ref())
            .is_some_and(|(f, s)| crate::ira::income_limited(f, s, b.owner));
        let owner = usize::from(b.owner == Owner::Spouse);
        let nondeductible = contribution_result.map_or(0, |v| v.nondeductible[owner]);
        let total_basis = prior.checked_add(nondeductible).ok_or(IraAmountIssue)?;
        let available = prior
            .checked_add(available_contribution_basis(
                b,
                contribution,
                contribution_result.map_or(0, |v| v.absorbed[owner]),
                nondeductible,
            ))
            .ok_or(IraAmountIssue)?;
        IraAmountIssue::reported(reservist)?;
        let current = nondeductible.checked_add(reservist).ok_or(IraAmountIssue)?;
        IraAmountIssue::reported(current)?;
        let to_plan = entries.iter().any(|(_, v)| matches!(v.ordinary_distribution,
            RetirementTreatment::Rollover { destination: crate::RolloverDestination::EmployerPlan, amount, .. } if amount > 0));
        if to_plan && (contribution == 0 || contribution_result.is_some()) {
            crate::require_localized!(
                r,
                available <= denominator,
                &crate::IssueContext::owner(crate::TaxSection::Retirement, b.owner),
                "IRA rollover",
                "ISSUE_AN_IRA_TO_EMPLOYER_PLAN_ROLLOVER_CAN_CONTAIN_ONLY_PRETAX_FUNDS_THE_EA576D4E",
                "An IRA-to-employer-plan rollover can contain only pretax funds. The retained IRA balances and amounts not rolled into that plan must cover this owner's available after-tax basis.",
            );
        }
        let pro_rata = |basis: i64| {
            if denominator == 0 {
                0
            } else {
                phase::cap(
                    phase::cap(
                        fraction(gross, phase::cap(basis, [denominator]), denominator),
                        [basis],
                    ),
                    [gross],
                )
            }
        };
        // Pub. 590-B Worksheet 1-1 first treats current contributions as basis
        // to determine deduction MAGI. After fixing that deduction, its reporting
        // instructions either retain the preliminary exclusion or use ordinary
        // Form 8606 allocation when available basis is insufficient. No iteration.
        let preliminary = pro_rata(
            prior
                .checked_add(if limited { dollars(contribution) } else { 0 })
                .ok_or(IraAmountIssue)?,
        );
        let worksheet = limited
            && contribution > 0
            && (contribution_result.is_none() || available >= preliminary);
        let (regular_taxable, converted_taxable) = if worksheet {
            let taxable = gross - preliminary;
            let converted = if gross == 0 { 0 } else { fraction(taxable, converted_gross, gross) };
            (taxable - converted, converted)
        } else {
            let basis = if contribution_result.is_none() { prior } else { available };
            let fraction_for = |amount| {
                if denominator == 0 {
                    0
                } else {
                    phase::cap(
                        fraction(amount, phase::cap(basis, [denominator]), denominator),
                        [amount],
                    )
                }
            };
            // Form 8606 lines 11 and 12 separately round conversions and withdrawals.
            let converted_exempt = phase::cap(fraction_for(converted_gross), [basis]);
            let regular_exempt = phase::cap(
                fraction_for(regular_gross),
                [phase::floor(basis - converted_exempt, 0)],
            );
            (regular_gross - regular_exempt, converted_gross - converted_exempt)
        };
        result.regular_gross[owner] = regular_gross;
        result.regular_taxable[owner] = regular_taxable;
        let taxable = regular_taxable + converted_taxable;
        IraAmountIssue::reported(taxable)?;
        let exempt = gross - taxable;
        let carried = crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            total_basis,
            exempt,
        )
        .get();
        // Stable IDs keep cent allocation independent of the editor's row order.
        let mut entries = entries;
        entries.sort_by_key(|(e, _)| e.id);
        let mut prefixes = [exact::ExactPrefix::default(), exact::ExactPrefix::default()];
        for (e, v) in entries {
            let amounts = [
                phase::floor(
                    v.gross
                        - v.rolled_amount()
                        - v.converted_amount()
                        - qcd.for_entry(e.id)
                        - funding.for_entry(e.id)
                        - v.home_repaid(),
                    0,
                ),
                v.converted_amount(),
            ];
            if v.home_repaid() > 0 {
                result.home_repaid.insert(e.id);
            }
            let mut portion = [0, 0];
            for (i, (total, gross)) in
                [(regular_taxable, &regular_cents), (converted_taxable, &converted_cents)]
                    .into_iter()
                    .enumerate()
            {
                portion[i] = prefixes[i].next(&exact::whole(amounts[i]), |cumulative| {
                    if gross == &exact::whole(0) {
                        0
                    } else {
                        // The total's actual reported Cents were checked above;
                        // each cumulative share is bounded by that native total.
                        exact::rounded(
                            &(exact::whole(total) * exact::whole(100) * cumulative / gross),
                        )
                        .expect("bounded IRA taxable share")
                    }
                });
            }
            result.taxable.insert(
                e.id,
                portion[0] + portion[1] + qcd.for_entry(e.id) + funding.for_entry(e.id),
            );
            result.converted.insert(e.id, portion[1]);
        }
        let prefix = if b.owner == Owner::Taxpayer { "taxpayer" } else { "spouse" };
        let calculation_rows = [
            (
                "ira_current_nondeductible_basis",
                current,
                msg!("EXPLAIN_N_N_IRA_CURRENT_NONDEDUCTIBLE_BASIS"),
            ),
            ("ira_reservist_basis", reservist, msg!("EXPLAIN_N_N_IRA_RESERVIST_BASIS")),
            (
                "ira_incoming_plan_basis",
                exact::dollars(&incoming).ok_or(IraAmountIssue)?,
                msg!("EXPLAIN_N_N_IRA_INCOMING_PLAN_BASIS"),
            ),
            (
                "ira_outstanding_rollover",
                exact::dollars(&outstanding).ok_or(IraAmountIssue)?,
                msg!("EXPLAIN_N_N_IRA_OUTSTANDING_ROLLOVER"),
            ),
            ("ira_basis_recovered", exempt, msg!("EXPLAIN_N_N_IRA_BASIS_RECOVERED")),
            ("ira_basis_carryforward", carried, msg!("EXPLAIN_N_N_IRA_BASIS_CARRYFORWARD")),
            ("ira_basis_taxable", taxable, msg!("EXPLAIN_N_N_IRA_BASIS_TAXABLE")),
        ];
        for (_, amount, _) in &calculation_rows {
            IraAmountIssue::reported(*amount)?;
        }
        emit_lines(r, &mut (), calculation_rows, |(suffix, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("{prefix}_{suffix}"),
                LineValue::MoneyDollars(amount),
                &["adjustments.ira_basis", "income.retirement"],
                "https://www.irs.gov/instructions/i8606",
                explanation,
            ))
        });
        // A carried balance alone does not require this year's form. Nor do
        // special distributions all repaid in their original year. Current
        // nondeductible contributions and divorce transfers still require it.
        let fully_repaid = regular_cents > exact::whole(0)
            && converted_cents == exact::whole(0)
            && b.repaid_in_year_exact() >= regular_cents;
        let part_i = current > 0
            || b.adjustment != 0
            || (!fully_repaid
                && opening > 0
                && (regular_gross > 0 || (converted_gross > 0 && year_end > 0)));
        if part_i {
            for value in [current, opening, total_basis, carried] {
                IraAmountIssue::reported(value)?;
            }
            if gross > 0 {
                for value in [total_basis - available, available, regular_taxable] {
                    IraAmountIssue::reported(value)?;
                }
                if !worksheet {
                    for value in [year_end, denominator] {
                        IraAmountIssue::reported(value)?;
                    }
                }
            }
            emit_lines(
                r,
                &mut (),
                [
                    ("1", LineValue::MoneyDollars(current)),
                    ("2", LineValue::MoneyDollars(opening)),
                    ("3", LineValue::MoneyDollars(total_basis)),
                    ("4", LineValue::MoneyDollars(total_basis - available)),
                    ("5", LineValue::MoneyDollars(available)),
                    ("6", LineValue::MoneyDollars(year_end)),
                    ("7", LineValue::MoneyDollars(regular_gross)),
                    ("8", LineValue::MoneyDollars(converted_gross)),
                    ("9", LineValue::MoneyDollars(denominator)),
                    (
                        "10",
                        LineValue::RatioTenThousandths(if denominator == 0 {
                            0
                        } else {
                            fraction(10_000, phase::cap(available, [denominator]), denominator)
                        }),
                    ),
                    ("11", LineValue::MoneyDollars(converted_gross - converted_taxable)),
                    ("12", LineValue::MoneyDollars(regular_gross - regular_taxable)),
                    ("13", LineValue::MoneyDollars(exempt)),
                    ("14", LineValue::MoneyDollars(carried)),
                    ("15a", LineValue::MoneyDollars(regular_taxable)),
                ],
                |(number, value), _, _| {
                    let included = match number {
                        "1" | "2" | "3" | "14" => true,
                        "6" | "7" | "8" | "9" | "10" | "11" | "12" => gross > 0 && !worksheet,
                        _ => gross > 0,
                    };
                    included.then(|| {
                        calculation_row(
                            Some(crate::FormLine::new(crate::TaxForm::Form8606, number)),
                            &alloc::format!("{prefix}_ira_form_line{number}"),
                            value,
                            &["adjustments.ira_basis", "adjustments.iras", "income.retirement"],
                            "https://www.irs.gov/pub/irs-dft/f8606--dft.pdf",
                            if worksheet {
                                msg!("EXPLAIN_IRA_FORM_RETAINED_WORKSHEET")
                            } else {
                                msg!("EXPLAIN_IRA_FORM_PART_I")
                            },
                        )
                    })
                },
            );
        }
        if converted_gross > 0 {
            // Without PartI, line17 is the available basis itself. A line18
            // at or below zero contributes no taxable income or deductible loss.
            let line17 = if part_i { converted_gross - converted_taxable } else { available };
            IraAmountIssue::reported(line17)?;
            IraAmountIssue::reported(converted_gross - line17)?;
            emit_lines(
                r,
                &mut (),
                [("16", converted_gross), ("17", line17), ("18", converted_gross - line17)],
                |(number, amount), _, _| {
                    Some(calculation_row(
                        Some(crate::FormLine::new(crate::TaxForm::Form8606, number)),
                        &alloc::format!("{prefix}_ira_form_line{number}"),
                        LineValue::MoneyDollars(amount),
                        &["adjustments.ira_basis", "adjustments.iras", "income.retirement"],
                        "https://www.irs.gov/pub/irs-dft/f8606--dft.pdf",
                        msg!("EXPLAIN_IRA_FORM_PART_II"),
                    ))
                },
            );
        }
    }
    for e in &d.income {
        if let FederalIncome::Retirement { data: v } = &e.income
            && v.taxable == RetirementTaxable::IraBasis
        {
            crate::require_localized!(
                r,
                v.kind == crate::RetirementKind::Ira && result.taxable.contains_key(&e.id),
                &crate::IssueContext::Section(crate::TaxSection::Retirement),
                "IRA basis",
                "ISSUE_EVERY_IRA_BASIS_SELECTION_NEEDS_A_COMPLETE_SUPPORTED_OWNER_LEVEL_WO_C967CBA5",
                "Every IRA-basis selection needs a complete supported owner-level worksheet in Adjustments; it cannot determine pension or annuity taxation.",
            );
        }
    }
    Ok(result)
}

// Complete PartI from the existing final disaster/repayment owners. This
// presentation step does not calculate another taxable distribution or spread.
pub(crate) fn project_basis_relief(d: &FederalDraft, r: &mut FederalResult, basis: &BasisResult) {
    for (owner, who) in [(Owner::Taxpayer, "taxpayer"), (Owner::Spouse, "spouse")] {
        let Some(line15a) = r
            .lines
            .iter()
            .find(|row| row.id == alloc::format!("{who}_ira_form_line15a"))
            .map(|row| row.amount)
        else {
            continue;
        };
        let designated: i64 = d.income.iter().filter(|entry| {
            entry.owner == owner && matches!(&entry.income, FederalIncome::Retirement { data } if data.kind == RetirementKind::Ira && data.taxable == RetirementTaxable::IraBasis)
        }).map(|entry| basis.disaster.get(&entry.id).copied().unwrap_or(0)).sum();
        let repaid = basis.repayment_excluded[usize::from(owner == Owner::Spouse)];
        emit_lines(
            r,
            &mut (),
            [("15b", designated), ("15c", line15a - designated - repaid)],
            |(number, amount), _, _| {
                Some(calculation_row(
                    Some(crate::FormLine::new(crate::TaxForm::Form8606, number)),
                    &alloc::format!("{who}_ira_form_line{number}"),
                    LineValue::MoneyCents(amount),
                    &["adjustments.ira_basis", "income.retirement"],
                    "https://www.irs.gov/pub/irs-dft/i8606--dft.pdf",
                    msg!("EXPLAIN_IRA_FORM_RELIEF"),
                ))
            },
        );
    }
}

// Shared by current and historical Forms 5329. Source amounts are cents;
// AGI is whole dollars. Apply the return-wide medical floor once, allocating
// relief to 25% SIMPLE amounts first, then 10% amounts.
pub(crate) fn medical_exception_allocation(
    mut taxable: [[i64; 2]; 2],
    expenses: i64,
    agi: i64,
) -> [[i64; 2]; 2] {
    let mut allowance = phase::floor(
        crate::kernel::ratio::dollars(expenses) - fraction(phase::floor(agi, 0), 75, 1000),
        0,
    ) * 100;
    for rate in [1, 0] {
        for owner in &mut taxable {
            let exempt = crate::kernel::carry::native_consume_step(owner[rate], &mut allowance);
            owner[rate] = crate::kernel::carry::native_balance(owner[rate], exempt);
        }
    }
    taxable
}

// Separate spouse forms aggregate source cents before rounding each line.
// Return taxable distributions, exceptions, subject amount and tax in dollars.
pub(crate) fn early_tax_lines(before: i64, after: [i64; 2]) -> [i64; 4] {
    use crate::kernel::ratio::dollars;
    let taxable = dollars(before);
    let exceptions = dollars(before - after.iter().sum::<i64>());
    let subject = phase::floor(taxable - exceptions, 0);
    let simple = phase::cap(dollars(after[1]), [subject]);
    let tax = dollars((subject - simple) * 10 + simple * 25);
    [taxable, exceptions, subject, tax]
}

// Form 5329 Part II, one form for each filer: line 5 the ABLE and education
// account distributions included in income, line 6 those that are not subject
// to the additional tax, line 7 the difference and line 8 its 10%
// (Schedule 2 line 5). The ABLE part keeps the cents its calculation made;
// the education part is education_account::calculate's. Source cents are
// aggregated before each line is rounded, as Part I does. Returns the total
// tax in whole dollars.
pub(crate) struct AccountDistributionTaxes {
    pub total_dollars: i64,
    pub owners: [i64; 2],
}

pub(crate) fn account_additional_tax(
    r: &mut crate::FederalResult,
    able: [i64; 2],
    able_penalized: [i64; 2],
    education: &crate::education_account::EducationTotals,
) -> AccountDistributionTaxes {
    use crate::kernel::lines::line;
    use crate::kernel::ratio::{dollars, fraction, positive};
    let mut total = 0;
    let mut owners = [0; 2];
    for owner in 0..2 {
        let included = able[owner] + education.income[owner];
        if included == 0 {
            continue;
        }
        let exempt = able[owner] - able_penalized[owner] + education.exceptions[owner];
        let (line_5, line_6) = (dollars(included), dollars(exempt));
        let line_7 = positive(line_5 - line_6);
        let tax =
            fraction(line_7, crate::year_constants::EDUCATION_ACCOUNT_ADDITIONAL_TAX_PERCENT, 100);
        total += tax;
        owners[owner] = tax;
        emit_lines(
            r,
            &mut (),
            [
                ("taxable", line_5, msg!("EXPLAIN_ACCOUNT_TAXABLE")),
                ("exceptions", line_6, msg!("EXPLAIN_ACCOUNT_EXCEPTIONS")),
                ("subject", line_7, msg!("EXPLAIN_ACCOUNT_SUBJECT")),
                ("tax", tax, msg!("EXPLAIN_ACCOUNT_TAX")),
            ],
            |(suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!(
                        "account_{}_{suffix}",
                        if owner == 0 { "taxpayer" } else { "spouse" }
                    ),
                    LineValue::MoneyDollars(amount),
                    &["income.able", "income.education_distribution"],
                    "https://www.irs.gov/instructions/i5329",
                    explanation,
                ))
            },
        );
    }
    line(
        r,
        "account_additional_tax",
        total,
        &["income.able", "income.education_distribution"],
        "https://www.irs.gov/instructions/i5329",
        msg!("EXPLAIN_ACCOUNT_ADDITIONAL_TAX"),
    );
    AccountDistributionTaxes { total_dollars: total, owners }
}

fn disposition_issue(
    v: &crate::R1099,
    issue: crate::validation::ValidationIssue,
) -> crate::validation::ValidationIssue {
    let issue = issue.in_field("what_happened");
    let issue = if matches!(
        v.code,
        DistributionCode::NormalLumpSum { tax: crate::LumpSumTax::WithOtherIncome { .. } }
    ) {
        issue.in_field("tax")
    } else {
        issue
    };
    issue.in_field("code")
}
fn exception_issue(
    v: &crate::R1099,
    second: bool,
    issue: crate::validation::ValidationIssue,
) -> crate::validation::ValidationIssue {
    if matches!(v.code, DistributionCode::RothIra { .. }) {
        issue.in_field("exception").in_field("roth").in_field("code")
    } else {
        issue
            .in_field(if second { "second" } else { "facts" })
            .in_field("exception")
            .in_field("code")
    }
}
fn other_boxes_issue(v: &crate::R1099, key: &'static str) -> crate::validation::ValidationIssue {
    let issue = crate::validation::ValidationIssue::new(key);
    if !matches!(v.disaster, crate::DisasterDistribution::No) {
        issue.at_field("disaster")
    } else if !matches!(v.more.disability, crate::DisabilityIncome::None) {
        issue.at_field("disability").in_field("more")
    } else {
        issue.at_field("public_safety")
    }
}

fn disposition_selection_issue(
    v: &crate::R1099,
    key: &'static str,
) -> crate::validation::ValidationIssue {
    let issue = crate::validation::ValidationIssue::new(key).at_field("what_happened");
    let issue = if matches!(
        v.code,
        DistributionCode::NormalLumpSum { tax: crate::LumpSumTax::WithOtherIncome { .. } }
    ) {
        issue.in_field("tax")
    } else {
        issue
    };
    issue.in_field("code")
}

fn repayment_issue(v: &crate::R1099, key: &'static str) -> crate::validation::ValidationIssue {
    let issue = crate::validation::ValidationIssue::new(key).at_field("repaid");
    if matches!(v.code, DistributionCode::RothIra { .. }) {
        issue.in_field("roth").in_field("code")
    } else {
        issue.in_field("exception").in_field("code")
    }
}
fn special_code_issue(v: &crate::R1099) -> crate::validation::ValidationIssue {
    let issue = crate::validation::ValidationIssue::new("R_SPECIAL_CODE_OTHER_BOXES");
    if !matches!(v.more.taxable_method, crate::TaxableMethod::BoxTwoA) {
        issue.at_field("taxable_method").in_field("more")
    } else if !matches!(v.disaster, crate::DisasterDistribution::No) {
        issue.at_field("disaster")
    } else {
        issue.at_field("public_safety")
    }
}
