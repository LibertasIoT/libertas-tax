// Coverage: Form 1099-INT / DIV / OID — partial investment specials (unit B11, 2026-10-10).
//   INT boxes 1–13 (box 7 country, box 5 informational expenses and box 10 current-election
//   market discount); DIV 1a/1b, 2a–2f, 4/5, 7/8, 12/13; OID 1–6 and 8–11 (box 9
//   informational expenses). Section 897 amounts remain subsets; expenses are accepted without
//   an investment-expense deduction. Boxes 3/9/10 of DIV use the capital-return document.
//   Existing premium/accrued-interest netting is payer-level; annual instrument, nominee,
//   prior-inclusion and linked Form 8815 work remains explicit until its owner is implemented.
//   Foreign-tax credit, REIT/QSBS, child election and AMT retain their existing owners.
//   Evidence: src/tests/investment/investment_statements.rs public saves, restored subset guards and restart;
//   src/tests/investment/investment.rs premium, Treasury OID, private-activity and special-treatment boundaries.
//   IRS INT/DIV instructions Rev.January2024; Pub.550(2025), Pub.1212 Rev.December2025.
// Coverage: Form 1099-B / 8949 — Supported (audited 2026-09-24).
//   Lines: Per-lot entry: proceeds net of fees, adjusted basis, holding period, disallowed
//   wash-sale loss (code W), withholding to line 25b, personal-use loss disallowance, AMT
//   basis/wash differences. Unit B7a (2026-10-09): the Investment sales page also writes employee
//   stock, ESPP and NUA share sales (EmployeeShareSale, EsppShareSale, NuaShareSale; Form 8949
//   box A/D with code B, so Schedule D line 1b/8b), QSBS code Q (QsbsSale, line 8b/9/10 by the
//   Form 8949 box), the section 1244 loss (SmallBusinessStockLoss: Form 4797 line 11, the capital
//   part code S on line 3/10), worthless securities (line 3/10) and nonbusiness bad debts (box C,
//   line 3); every short/long contributor has its Schedule D line, so lines 1a-4 and 8a-11 foot
//   (investment_kinds.rs schedule_d_lines_foot_with_the_investment_kinds). Section 1242 (SBIC stock) is
//   not supported. Exclusions: unresolved compensation (an ordinary option, RSU, ESPP, ISO or NUA sale
//   has its own kind), business property, a home sale and another treatment (a contingent payment
//   debt instrument, code R rollover, code X, a trader election, a carried-interest gain, a loss
//   the broker says the proceeds can't support) are one SaleSpecial choice each on
//   FederalSale.special, left out with their own notice; no broker-summary entry.
//   Unit B9-5 (2026-10-10, tax-forms/investment-sale-specials map): the page is the 1099-B or
//   1099-DA sale. FederalSale gains basis Option<i64> (empty counts as zero with
//   WARN_SALE_BASIS_ASSUMED_ZERO), digital_asset (boxes G-L), market_discount (code D, interest)
//   and several_sales (code M); proceeds may be negative for SaleUse::WrittenOption (always
//   short-term, section 1234(b)); SaleAdjustment::Nominee is calculated (code N, column (g) backs
//   out the gain) and OptionPremium is code E; options follow the Pub. 550 outcome table with no
//   option record (an option bought is an Investment lot, an exercised option changes the stock
//   lot); Form 8949 rows per lot with columns (a), (d)-(h) and the box and Schedule D columns
//   (d), (e), (g) (capital.rs Form8949Row, federal_rules.rs collect_sale); SaleSpecial loses
//   UnknownBasis and OptionContract; qualified opportunity funds are the QofInvestment and
//   QofDeferral kinds (opportunity_fund.rs) and the opportunity-fund situation is gone; a
//   fund's section 1202 gain (1099-DIV box 2c, Form 2439 box 1c) is calculated with one
//   FundQsbsExclusion choice (qsbs.rs fund_gain, Form 8949 code Q). Tests: src/tests/investment/sale_specials.rs.
//   Unit B2 (2026-10-09): a Form 1099-B that completes boxes 8-11 (regulated futures, foreign
//   currency and section 1256 options, reported in aggregate) is its own kind, Contracts1099B
//   onto FederalIncome::Section1256 (broker, box 11 aggregate, box 4, the state boxes; boxes 1a,
//   8, 9 and 10 are not read), calculated on Form 6781 Part I (section1256.rs); the sale kind's
//   import reports such a form as an error. Evidence: section1256.rs tests.
//   Evidence: federal_model.rs (FederalSale, SaleUse, CapitalSaleAmt), federal_rules.rs
//   validate_sale, stock_loss.rs, employee_stock.rs, espp_sale.rs, iso_sale.rs, qsbs.rs; tests:
//   personal_sale_loss_cannot_offset_another_items_capital_gain,
//   collectible_sales_net_losses_and_preserve_personal_loss_disallowance (investment.rs),
//   employee_stock_losses_wash_adjustments_and_reporting_boundaries,
//   espp_loss_fees_wash_and_holding_period_boundaries (business.rs); i1040sd (no separate
//   8949/1099-B citation)
//   Unit F4 (2026-10-10, line audit D7, D10): a basis-reported lot that is a collectible (code C) or a
//   personal-use loss (code L) is Schedule D line 1b/8b, never the summary line 1a/8a
//   (federal_rules.rs collect_sale); `personal_loss()` is the loss after code E selling expenses, so
//   expenses on personal-use property never make a deductible loss. Tests: src/tests/investment/investment.rs
//   collectibles_and_personal_losses_leave_the_summary_lines, personal_use_selling_expenses_never_create_a_loss.
//   Unit C12a (2026-09-25): FederalSale.adjustment is the SaleAdjustment union (None, code B
//   BasisCorrection { reported_basis }, code T HoldingPeriodCorrection, code E SellingExpenses
//   { amount } subtracted in gain()/amt_gain(), code N Nominee, refused until B9-5); one code per
//   lot; no column (f)/(g) output line yet. STORAGE_FORMAT 9. Tests: investment.rs
//   form_8949_codes_b_t_e_adjust_gain_per_lot, nominee_sale_is_refused_by_code.
//   Unit C12b (2026-09-25): FederalSale.basis_reporting (ReportedToIrs, NotReported, NoForm1099B)
//   with the term selects box A-F; Schedule D lines 1a/1b/2/3 and 8a/8b/9/10 are emitted as gain
//   totals (capital_gain_1a..capital_gain_10); a covered lot with no code and no wash sale is
//   1a/8a. Lines 7 and 15 stay the existing capital_gain total. STORAGE_FORMAT 10. Test:
//   investment.rs schedule_d_lines_group_lots_by_form_8949_box.
//   C-fixture (2026-09-25, src/tests/investment/investment.rs): brokerage_lots_return_matches_hand_computation (five
//   lots across boxes A-F with a wash sale and collectibles, the Schedule D Tax Worksheet by hand),
//   wash_sale_disallowance_cannot_exceed_the_loss, wash_sale_add_back_never_creates_a_gain,
//   code_e_selling_expenses_match_net_proceeds, holding_period_term_is_recorded_as_the_model_takes_it
//   (the model carries a term flag, not dates).
//   Not-sure audit batch 7 (2026-10-05): FederalSale.amt is the SaleAmt choice (SameAsRegular or
//   Different with the AMT record) instead of a nullable record; the Form 1099-B page writes
//   SameAsRegular. STORAGE_FORMAT 84.
// Coverage: Form 1099-K / NEC / MISC — Supported (unit b9-2, 2026-10-10; see information_returns.rs).
//   Lines: each form is its own document (Form1099Nec, Form1099K on Self-employment, Form1099Misc
//   on Other income); every nonzero box is sent to the business, rental, royalty, equipment or hobby
//   record the person names, box 4 reaches line 25b, and the 1099-K box 1a parts must add to box 1a.
//   Refused by name on the form: 1099-NEC box 3 and 1099-MISC box 15 (the UnusualMisc and
//   GoldenParachute situations are gone).
//   Unit B6b (2026-10-09): hobby 1099-K sales are the Other income page's Sales from a hobby kind
//   (FederalIncome::HobbySales) and misclassified NEC/MISC pay is the Wages page's Form 8919 kind
//   (FederalIncome::EmployeeWages, 1099-NEC box 1 / 1099-MISC box 3 and box 4 imported).
//   Stored computation, no page: the self-employed overtime record (FederalBusiness.overtime,
//   which the pages leave None).
//   Evidence: federal_model.rs (BusinessActivity::Ordinary, FederalBusinessOvertime,
//   FederalIncome::{NonbusinessProperty,Prize,Sale,HobbySales,EmployeeWages}),
//   hobby.rs, prizes.rs, employee_wages.rs; tests:
//   reported_1099_overtime_shares_the_w2_cap_without_duplicating_business_income,
//   hobby_goods_sales_use_inventory_cost_without_business_deductions_or_losses,
//   employee_8919_return_reports_wages_and_employee_tax_once (business.rs),
//   personal_prizes_include_noncash_and_withholding_without_earned_income (deductions.rs); irs.gov
//   1099-K guidance, f8919, Pub 531
//   C-fixture (2026-09-25, src/tests/federal/general.rs): contractor_return_matches_hand_computation (NEC and K
//   receipts, NEC withholding, a MISC prize and royalty by hand), personal_property_rent_reaches_schedule_1_lines_8l_and_24b,
//   unmodeled_1099_misc_nec_boxes_stay_refused, prize_and_business_1099_withholding_combine_once_on_25b.
// Coverage: Form1098 and Schedule A mortgage interest (B10-4,2026-10-10).
//   mortgage_limit.rs owns current supported proceeds, home, average-balance, refinance,
//   points, premium and refund rules and their source/fixture inventory. Mortgage1098 imports
//   boxes1–6, preserving box3 as printed; personal loan/refund facts live outside import targets.
//   Schedule A5b owns box10 real estate taxes; no dead box10 amount or refusal remains.
// Coverage: Form 1099-G — Supported (audited 2026-09-25; 2026-10-08: box 1 is importable-empty and box 7 an
//   AI-import target, so a form with only box 7 imports; box 4 may reach the sum of boxes 1, 5, 6, 7, 9 and 10.
//   Test: src/tests/income/interview.rs unemployment_1099g_with_only_box_7_imports_and_reaches_the_farm).
//   The page is Form1099G. A farm or farm rental claims its box 7 component through
//   related forms; an unclaimed payment uses the owner's only eligible activity,
//   with a correction note when the owner has several: Schedule F line 4a/4b or Form 4835 line 3a/3b,
//   no longer the lowest-id farm. Tests: src/tests/farm/farm_rental.rs
//   fixture_e_documents_route_to_the_only_farm_rental_and_to_the_chosen_farm,
//   the_1099_a_and_1099_g_pages_keep_their_chosen_loan_and_farm.
//   Lines: Box 1 unemployment with box 4 withholding, same-year repayment netting, prior-year
//   repayment $3,000/claim-of-right coordination; box 2 state/local refund recovery via Pub 525
//   worksheet incl. multi-year allocation, SALT-cap benefit limit, Pease, AMT removal. Not
//   calculated: box 5 RTAA, box 6 taxable grants, box 7 agriculture payments as a 1099-G item (farm
//   accepts an aggregate only), box 9 market gain; non-ordinary unemployment (union/private funds)
//   refused via ordinary Answer.
//   Evidence: federal_model.rs FederalUnemployment, FederalIncome::TaxRefund; tax_refund.rs,
//   claim_repayment.rs, federal_rules.rs; tests:
//   unemployment_fully_repaid_benefits_preserve_actual_withholding (taxes.rs),
//   unemployment_same_year_repayments_net_once_on_complete_return,
//   state_refund_is_limited_to_prior_itemization_benefit_and_removed_from_amt,
//   one_state_refund_from_three_prior_payment_years_reconciles_each_year_and_return (repayment.rs);
//   Pub 525, i1040sca
//   Unit C23a (2026-09-25): FederalUnemployment gains boxes 5 (RTAA, refused), 6 (taxable grants to
//   Schedule 1 line 8z), 7 (agriculture payments routed to the owner's lowest-id farm for SE and QBI;
//   refused without a farm; shown in that farm's gross-income and net-profit lines), 8 (refused when
//   Yes), 9 (market gain, refused) and 10 (family leave to line 8z per Rev. Rul. 2025-4; refused
//   when a medical-leave portion is included); TaxRefund gains box 4 withholding to line 25b.
//   STORAGE_FORMAT 21. Tests: src/tests/deductions/repayment.rs taxable_grants_and_agriculture_payments_reach_their_lines,
//   family_leave_benefits_reach_line_8z_and_medical_leave_is_refused, state_refund_withholding_reaches_line_25b_once,
//   rtaa_box_8_and_market_gain_stay_refused.
//   C-fixture (2026-09-25, src/tests/deductions/repayment.rs): unemployment_refund_and_grant_return_matches_hand_computation
//   (boxes 1, 2, 4, 6 by hand), state_refund_is_taxable_up_to_the_prior_year_benefit (Pub 525),
//   agriculture_payment_shows_in_the_farm_lines_once, later_year_unemployment_repayment_follows_the_claim_of_right_path.
// Coverage: Form 2210 — calculation and source limits are owned by underpayment.rs.
//   FederalPayments retains one estimated-payment method with dated or annual funding parts.
//   Explicit EqualTimely projects ordinary/farmer installments; TimingUnknown preserves line 26
//   without establishing a precise penalty. No stored total duplicates those source facts.
//   Tests: src/tests/payments/penalty.rs, src/tests/payments/interview.rs.
// Coverage: Form 1098-E — Supported (audited 2026-09-24).
//   Lines: Student-loan interest deduction: $2,500 cap; 2026 MAGI phaseout $85k–$100k ($175k–$205k
//   joint); MFS = 0; claimed-as-dependent check (unclaimed_filers); eligibility gate ("Student
//   loan"); health-service program-paid interest removed once; NOL and section 1202 refigures.
//   Deduction is inside the aggregate adjustments line (no dedicated line id).
//   Evidence: federal_model.rs StudentLoanClaim, federal_rules.rs; src/tests/family/family.rs
//   dependent_student_loan_uses_actual_claim_not_claimability,
//   prior_nol_refigures_student_loan_deduction_and_each_vintages_absorption; src/tests/retirement/retirement.rs
//   ira_social_security_recomputed_then_student_loan_phaseout; src/tests/health/health_accounts.rs
//   loan_phaseout_and_hsa_limits; source Pub 970, irb/2025-45.
//   T2-33x (2026-09-26): box 2. Each Form1098E is stored in StudentLoanClaim::Yes.forms with
//   its box 1 and a Form1098EBox2 inline fold; Checked carries the additional qualified interest
//   the filer figured under Pub 970 chapter 4 (origination fees amortized over the loan term and
//   capitalized interest paid in 2026). The claim's own student_loan_interest is now the
//   qualified interest paid with no Form 1098-E (a lender under $600 or one that did not file).
//   student_loan_interest() adds the three before the health-service removal, the $2,500 cap and
//   the phase-out. The amortization itself is the filer's figure; nothing else needs the pre-2004
//   schedule, so no refusal is added. STORAGE_FORMAT 56. Sources: f1098e (box 2), Pub 970
//   chapter 4, fetched from irs.gov 2026-09-26. Tests: src/tests/investment/repayment.rs
//   box_2_interest_and_interest_without_a_form_join_box_1, box_2_interest_is_added_before_the_2500_cap,
//   box_2_interest_is_phased_out_with_box_1, box_2_and_no_form_interest_end_at_the_phaseout_end;
//   src/tests/deductions/interview.rs educator_and_1098e_pages_map_onto_their_claims; src/tests/federal/general.rs round trip.
//   T2-fx (2026-09-27): section 127 employer-paid interest. StudentLoanClaim::Yes and the page's
//   StudentLoanInterest carry employer_paid_interest (tax-free interest an employer paid under an
//   educational assistance program, which no Form 1098-E or W-2 box separates); federal_rules.rs
//   subtracts it with the health-service overlap before the $2,500 cap and phase-out (line
//   employer_loan_interest_removed) and refuses "Student loan interest" when the two together
//   exceed the interest entered. Pub 970 (2025) chapter 4: "No double benefit allowed. You can't
//   deduct as interest on a student loan any interest paid by your employer after March 27,
//   2020, under an educational assistance program"; chapter 10 caps tax-free benefits at $5,250.
//   STUDENT_LOAN_EMPLOYER_ABOVE_INTEREST on both sides (the client rule sums the interest paid with
//   no form and every form's box 1 and box 2 amount; the server repeats it on the whole page).
//   STORAGE_FORMAT 67. Tests: src/tests/investment/repayment.rs employer_paid_interest_is_not_deductible,
//   employer_paid_interest_is_removed_before_the_2500_cap,
//   employer_paid_interest_is_removed_before_the_phase_out,
//   employer_paid_interest_equal_to_all_interest_leaves_no_deduction; src/tests/family/interview.rs
//   employer_paid_student_loan_interest_round_trips_and_is_checked; src/tests/federal/general.rs round trip.
// Coverage: Form 5695 — Supported (audited 2026-09-24).
//   Lines: 2026 form is carryforward-only under P.L. 119-21: lines 1–3 (2025 residential clean
//   energy carryforward limited by tax after preceding credits incl. Form 8859) and line 4
//   carryforward to 2027, with a review note that ordering follows the draft credit-limit
//   worksheet. No input exists for new 2026 expenditures (Part I new property or Part II home
//   improvement); code states they do not qualify in 2026, so they are absent, not refused.
//   Evidence: federal_model.rs (residential_energy_carry), federal_rules.rs; src/tests/credits/credits.rs
//   residential_energy_carryforward_uses_form_5695_and_follows_dc_credit; source f5695--dft.pdf.
// Coverage: Schedule K-1 (1041) — the beneficiary's boxes; the source records and their
//   calculation owners are in estate_trust.rs.
//   Lines: Boxes 1, 2a/2b, 3, 4a–4c, 5 (nonbusiness royalties), 6–8 by fiduciary activity
//   statement (Schedule E line 33 columns (c)–(f) with Form 8582 passive limits, Form 8960 and the
//   Form 8995/8995-A QBI statements), 9 codes A–C, 10 (IRD-interest estate tax), 11 codes A–F
//   final year (section 67(e), code B taxes, capital loss and NOL carryovers by original year),
//   12 codes A–J (Form 6251 and the 2027 Form 8801 exclusion items), 13 codes A and B (payments)
//   and C–T and ZZ (Form 3800 Part III, through the passive credit limit of the linked
//   activities), 14 A, B (a qualified passive payee statement), C, D and M (Form 3468 property),
//   E (Form 4952), F (Schedule E line 42), H (Form 8960 line 7) and I (section 199A). Left out
//   with a notice: box 13 code P (Form 8912 is figured only for bonds held directly), box 14
//   code B foreign tax needing a category or country allocation, code G foreign trading gross
//   receipts and codes J–L (Form 3468 Parts II–IV project property). Refused by name: box 13 code R
//   credit recapture, unclassified box 14 information, a box 12 code G–I amount with no
//   statement, and unclassified K-1 income or a foreign beneficiary. Box 12 code J retains its
//   exclusion component for the following year's minimum-tax credit; it is not another
//   current-year AMT adjustment.
//   Evidence: src/tests/passthrough/passthrough.rs estate_trust_*; src/tests/passthrough/interview.rs
//   k1_1041_*; sources f1041sk1--dft and i1041sk1--dft (2026 drafts).
// Coverage: Form W-2G — Supported (audited 2026-09-24).
//   Lines: Box 1 winnings to Schedule 1 gross (no netting); box 4 withholding to line 25c/25d;
//   losses to itemized gambling_loss_deduction = min(winnings, 90% of losses) under the 2026 rule,
//   zero under standard deduction; Form 8615 and prior-NOL paths. Left out with
//   WARN_GAMBLING_LEFT_OUT, which sends a gambling business to Self-employment (unit B6b: the
//   Business page's Gambling as a business record, BusinessActivity::Gambling), a treaty or
//   nonresident position, an installment prize sale or unreconciled session records. Interview: the Gambling winnings topic (W2g,
//   GamblingLosses; unit T2-44p, 2026-09-27, see interview.rs); Schedule 1 line 8b and Schedule A
//   line 16 tagged.
//   Evidence: federal_model.rs (FederalGambling), federal_rules.rs; src/tests/federal/taxes.rs
//   gambling_withholding_and_unresolved_records_are_handled; src/tests/investment/investment.rs
//   casual_gambling_uses_2026_loss_limit_and_preserves_gross_income,
//   gambling_loss_cap_is_winnings_not_ninety_percent_of_winnings; src/tests/family/family.rs
//   child_gambling_losses_reduce_unearned_income_after_ninety_percent_limit; sources Tax Topic 419,
//   Pub 505.
// Coverage: Form 1310 — Deferred (audited 2026-09-24); the decedent's final return itself
//   calculates (unit B9-3, 2026-10-10; Pub. 559, Pub. 501, Pub. 596, i1040gi "Death of a Taxpayer",
//   26 U.S.C. 6013(a)(2), (c), (d)(1)(B), 213(c), 443(a)(2), 691(a), 2(b)(2)(C), 152(c)(1)(B)).
//   Lines: Filing administration stays outside the App (the "Deceased" box and date, the
//   signatures, Form 1310, the court certificate). The About you and About your spouse pages'
//   "Died in 2026" choice writes FederalPerson.died: the date of death, the section 213(c)
//   estate-paid medical amount (zero is no election), the savings-bond record, and the one
//   Yes/No on the death adjustments the App cannot compute (No leaves them out with
//   WARN_FINAL_RETURN_ADJUSTMENTS_LEFT_OUT: released passive losses, suspended S corporation
//   losses, section 692 forgiveness, an earlier decedent's section 691(c) deduction, section
//   1402(f) partnership self-employment, estate-paid deductible payments on a Form 1098, 1098-E
//   or 1098-T). The full standard deduction (age 65 and blindness at death), the residence
//   tests over half the days alive on a return that is not joint, a joint return of the year a
//   spouse died under MaritalState::Married, the per-statement "paid after the death" part of a
//   1099-INT, 1099-DIV or 1099-R left off (after_death.rs), carryovers ending with
//   NOTE_FINAL_RETURN_CARRYOVERS_END, and the rest of every Form 8915-F spread (disaster.rs).
//   Evidence: src/tests/federal/general.rs final_return_f1_*, final_return_f2_*, final_return_f5_*,
//   the_part_paid_after_a_death_*, a_final_return_carries_no_loss_forward_and_says_so;
//   src/tests/retirement/disaster.rs form_8915_f_final_return_includes_the_rest_of_every_spread;
//   src/tests/passthrough/passthrough.rs deceased_shareholder_k1_*; src/tests/deductions/deductions.rs
//   final_filer_standard_deduction_not_prorated_and_no_premature_senior_addition; src/tests/retirement/credits.rs
//   saver_final_return_uses_age_at_death_not_future_birthday; src/tests/cross/interview.rs
//   final_return_page_maps_the_stored_final_return_both_ways, death_date_rules_*.
// Coverage: Form 1040-ES (computation) — Supported; see forms/form_1040_es.rs (T2-49, T2-49b).
// Coverage: Form 3903 — Missing (audited 2026-09-24).
//   Lines: No calculation; refused explicitly via the OtherAdjustments situation (M1-P3;
//   formerly other_adjustments), refused by name.
//   Evidence: M/federal_model.rs, federal_rules.rs; none
// Coverage: Form 2210-F — Missing (audited 2026-09-24).
//   Lines: No underpayment-penalty calculation (also none for Form 2210); only a note that 1041-T
//   allocated estimates count as paid January 15, 2027.
//   Evidence: federal_model.rs note only; no tests
// Coverage: Relief payments and other income — Supported (unit B6b, 2026-10-09; audited against
//   the 2026 drafts of Form 1040, Schedule 1 and Form 8919, i1040gi (2025), Pub. 525, 547, 970 and
//   596 (2025), 26 U.S.C. 123, 139, 139M (P.L. 119-108 section 3, payments received in taxable
//   years beginning after 2025), 108(f)(4), 117 and 183).
//   Lines: the Other income page's seven kinds. Wildfire relief (section 139M: excluded, or
//   Schedule 1 line 8z when not confirmed, with the 1099 withholding on line 25b either way),
//   disaster relief (section 139(a)-(c), (e), (h): the same, with the carrier and government
//   purposes matched to their qualified-disaster bases), living-expense insurance (section 123 and
//   Pub. 547: the excess of the insurer's payments over actual less normal expenses, line 8z, none
//   in a federally declared disaster area, with the prior-return reconciliation), hazard
//   mitigation (section 139(g): excluded, left out with WARN_DISASTER_MITIGATION_LEFT_OUT when not
//   confirmed), scholarships and fellowships (Pub. 970 Worksheet 1-1, line 8r: the award less the
//   part a degree candidate spent on qualified expenses, so services paid by the award are
//   taxable), jury pay (line 8h; pay turned over to the employer, line 24a) and hobby sales
//   (scholarship.rs, hobby.rs). The Wages page's Form 8919 pay (employee_wages.rs), the Canceled
//   debt page's health-service loan repayment (debt.rs, section 108(f)(4)) and the 1099-R code 3
//   minimum retirement age (retirement.rs) are the same unit. A preview of an excluded payment
//   shows 0 and a left-out row shows 0. Review note NOTE_SCHOLARSHIP_MAY_BE_TAXABLE when a filer's
//   1098-T box 5 exceeds the tuition and books and no scholarship is entered for that person.
//   East Palestine train derailment payments (P.L. 118-148 section 3) have no kind: the Other
//   income description says they are not taxable and need not be entered.
//   Exclusions: a payment that only partly qualifies (enter the taxable part under another
//   source), a deduction or credit for a compensated expense, foreign scholarships and treaty
//   exclusions, Form 1098-T allocation elections (the Education page keeps its own box 5 amount),
//   hobby deductions (section 67(h)), Form 4852 as a kind (the W-2 and 1099-R pages take its
//   lines).
//   Evidence: federal_model.rs (FederalWildfireRelief, FederalDisasterRelief,
//   FederalCasualtyLivingInsurance, FederalDisasterMitigation, JuryPay),
//   federal_rules.rs, unknowns.rs; src/tests/income/relief_income.rs fixtures A-L (Form 8919, health-service
//   loans, wildfire, disaster, living-expense insurance, mitigation, scholarship, jury pay, hobby
//   sales, professional gambling, disability pension, the Form 2555 boundary); the stored
//   computations are also covered by src/tests/federal/general.rs, src/tests/health/health_accounts.rs and
//   src/tests/business/business.rs.

use crate::kernel::phase;
use crate::{CareProvider, Owner};
use crate::{InformationReturnFiling, MealExpense, OtherBusinessIncome, SimpleEmployer};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport, libertas_bitflags};

pub use crate::accepted_income::{FederalIncome, FederalIncomeEntry};
pub(crate) use crate::income_editor::IncomeEdits;
pub use crate::income_editor::{
    CanceledDebtDocument, IncomeDocumentItem, IncomeRecord, IncomeRecordItem, IncomeRecords,
    IncomeRefundManagementDerived, IncomeSource, IncomeSourceChoices, IncomeSourceKind, K1Document,
    OtherIncomeDocument,
};
pub use crate::misc_source::{ForeignAccountOrTrust, JuryPay, QbiLossCarryforward};

/// Foreign tax situation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignSituationChoice {
    /// Situation code
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub code: u32,
    /// Situation
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}

pub use crate::record_choice::RecordChoice;

pub use crate::diagnostics::{FederalIssue, IssueContext, NoteIdentity, ReviewNote};
pub(crate) use crate::return_state::PayrollKnownRows;
#[allow(unused_imports)]
pub use crate::return_state::{
    FederalAmount, FederalEstimate, FederalResult, FederalReviewNoteView,
};

#[allow(unused_imports)]
pub(crate) use crate::return_reporting::IdKeys;
#[allow(unused_imports)]
pub use crate::return_reporting::{draft_return, form_coverage, id_pattern, line_label};

pub use crate::accepted_draft::FederalDraft;

pub use crate::form_lines::{FormLine, FormLineKind, TaxForm};

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateTrustNiitAdjustment {
    amounts: [line_7, modified_agi],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(FinalEstateTrustNol {
    amounts: [years],
    gains: [],
    facts: [terminated_on, fiduciary_history_complete, taxable_periods],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(FinalEstateTrustNolYear {
    amounts: [regular_balance, alternative_balance],
    gains: [],
    facts: [year, original_period, remaining_years],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(FederalBusinessExpense {
    amounts: [amount],
    gains: [],
    facts: [category, treatment],
});

crate::community::divide_enum!(BusinessExpenseKind {
    records: [DeMinimis { amounts: [], facts: [largest_cost, financial_statement, eligible] }],
    units: [Ordinary, Meals, TransportationMeals, Nondeductible],
    tuples: [],
});

crate::community::divide_enum!(BusinessExpenseCategory {
    records: [Other { amounts: [], facts: [label] }],
    units: [
        Advertising,
        Commissions,
        ContractLabor,
        EmployeeBenefits,
        Insurance,
        MortgageInterest,
        OtherInterest,
        LegalAndProfessional,
        Office,
        VehicleAndEquipmentRent,
        OtherRent,
        Repairs,
        Supplies,
        TaxesAndLicenses,
        Travel,
        Meals,
        Utilities,
        Wages,
        Chemicals,
        CustomHire,
        Feed,
        Fertilizers,
        Freight,
        Fuel,
        EmployeePension,
        Seeds,
        Storage,
        Veterinary
    ],
    tuples: [],
});

// Per-form modules split out by tools/split-hubs; the tool regenerates this block.
#[path = "../deductions/adjustments_model.rs"]
mod adjustments_model;
#[path = "../passthrough/bba_audit_model.rs"]
mod bba_audit_model;
#[path = "../deductions/charity_model.rs"]
mod charity_model;
#[path = "../attribution/community_property_model.rs"]
mod community_property_model;
#[path = "../credits/credits_model.rs"]
mod credits_model;
#[path = "../family/dependents_model.rs"]
mod dependents_model;
#[path = "../home/disaster_model.rs"]
mod disaster_model;
#[path = "../deductions/educator_model.rs"]
mod educator_model;
#[path = "../passthrough/estate_trust_k1_model.rs"]
mod estate_trust_k1_model;
#[path = "../family/filers_model.rs"]
mod filers_model;
#[path = "../payments/form_1040_es_model.rs"]
mod form_1040_es_model;
#[path = "../deductions/form_1098_e_model.rs"]
mod form_1098_e_model;
#[path = "../home/form_1098_model.rs"]
mod form_1098_model;
#[path = "../investment/form_1099_b_model.rs"]
mod form_1099_b_model;
#[path = "../investment/form_1099_div_model.rs"]
mod form_1099_div_model;
#[path = "../payments/form_1099_g_model.rs"]
mod form_1099_g_model;
#[path = "../investment/form_1099_int_model.rs"]
mod form_1099_int_model;
#[path = "../investment/form_1099_oid_model.rs"]
mod form_1099_oid_model;
#[path = "../farm/form_1099_patr_model.rs"]
mod form_1099_patr_model;
#[path = "../retirement/form_1099_r_model.rs"]
mod form_1099_r_model;
#[path = "../business/form_2106_model.rs"]
mod form_2106_model;
#[path = "../payments/form_2210_model.rs"]
mod form_2210_model;
#[path = "../family/form_2441_model.rs"]
mod form_2441_model;
#[path = "../credits/form_4136_model.rs"]
pub(crate) mod form_4136_model;
#[path = "../home/form_8396_model.rs"]
mod form_8396_model;
#[path = "../family/form_8814_model.rs"]
mod form_8814_model;
#[path = "../investment/form_8815_model.rs"]
mod form_8815_model;
#[path = "../family/form_8863_model.rs"]
mod form_8863_model;
#[path = "../credits/form_8880_model.rs"]
mod form_8880_model;
#[path = "../health/form_8889_model.rs"]
mod form_8889_model;
#[path = "../credits/form_8936_model.rs"]
mod form_8936_model;
#[path = "../health/form_8962_model.rs"]
mod form_8962_model;
#[path = "../income/form_w2g_model.rs"]
mod form_w2g_model;
#[path = "../retirement/ira_contributions_model.rs"]
mod ira_contributions_model;
#[path = "../payments/other_taxes_model.rs"]
mod other_taxes_model;
#[path = "../payments/payments_model.rs"]
mod payments_model;
#[path = "../deductions/schedule_1a_model.rs"]
mod schedule_1a_model;
#[path = "../deductions/schedule_a_model.rs"]
mod schedule_a_model;
#[path = "../business/schedule_c_model.rs"]
mod schedule_c_model;
#[path = "../investment/schedule_e_model.rs"]
mod schedule_e_model;
#[path = "../health/se_benefits_model.rs"]
mod se_benefits_model;
#[path = "../retirement/ssa_1099_model.rs"]
pub(crate) mod ssa_1099_model;
#[path = "../income/w2_model.rs"]
mod w2_model;
#[path = "../income/wages_records_view.rs"]
mod wages_records_view;
pub use crate::accepted_screening::FederalScreening;
#[allow(unused_imports)]
pub use crate::checklist_model::*;
#[allow(unused_imports)]
pub use crate::income_editor::InvestmentSaleDocument;
pub use crate::interview::OftenMissedAnswers;
#[allow(unused_imports)]
pub use adjustments_model::*;
#[allow(unused_imports)]
pub use bba_audit_model::*;
#[allow(unused_imports)]
pub use charity_model::*;
#[allow(unused_imports)]
pub use community_property_model::*;
#[allow(unused_imports)]
pub use credits_model::*;
#[allow(unused_imports)]
pub use dependents_model::*;
#[allow(unused_imports)]
pub use disaster_model::*;
#[allow(unused_imports)]
pub use educator_model::*;
#[allow(unused_imports)]
pub use estate_trust_k1_model::*;
#[allow(unused_imports)]
pub use filers_model::*;
#[allow(unused_imports)]
pub use form_1040_es_model::*;
#[allow(unused_imports)]
pub use form_1098_e_model::*;
#[allow(unused_imports)]
pub use form_1098_model::*;
pub use form_1099_b_model::*;
pub(crate) use form_1099_b_model::{validate_sale, validate_sale_with_mode};
#[allow(unused_imports)]
pub use form_1099_div_model::*;
#[allow(unused_imports)]
pub use form_1099_g_model::*;
#[allow(unused_imports)]
pub use form_1099_int_model::*;
#[allow(unused_imports)]
pub use form_1099_oid_model::*;
#[allow(unused_imports)]
pub use form_1099_patr_model::*;
#[allow(unused_imports)]
pub use form_1099_r_model::*;
#[allow(unused_imports)]
pub use form_2106_model::*;
#[allow(unused_imports)]
pub use form_2210_model::*;
#[allow(unused_imports)]
pub use form_2441_model::*;
#[allow(unused_imports)]
pub use form_4136_model::*;
#[allow(unused_imports)]
pub use form_8396_model::*;
#[allow(unused_imports)]
pub use form_8814_model::*;
#[allow(unused_imports)]
pub use form_8815_model::*;
#[allow(unused_imports)]
pub use form_8863_model::*;
#[allow(unused_imports)]
pub use form_8880_model::*;
#[allow(unused_imports)]
pub use form_8889_model::*;
#[allow(unused_imports)]
pub use form_8936_model::*;
#[allow(unused_imports)]
pub use form_8962_model::*;
#[allow(unused_imports)]
pub use form_w2g_model::*;
#[allow(unused_imports)]
pub use ira_contributions_model::*;
#[allow(unused_imports)]
pub use other_taxes_model::*;
#[allow(unused_imports)]
pub use payments_model::*;
#[allow(unused_imports)]
pub use schedule_1a_model::*;
#[allow(unused_imports)]
pub use schedule_a_model::*;
#[allow(unused_imports)]
pub use schedule_c_model::*;
#[allow(unused_imports)]
pub use schedule_e_model::*;
#[allow(unused_imports)]
pub use se_benefits_model::*;
#[allow(unused_imports)]
pub use ssa_1099_model::*;
#[allow(unused_imports)]
pub use w2_model::*;
pub use wages_records_view::*;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) const HUB_SOURCE: &str = concat!(
    include_str!("../income/espp_return.rs"),
    include_str!("../income/iso_return.rs"),
    include_str!("../family/child_election_source.rs"),
    include_str!("../investment/annual_debt_source.rs"),
    include_str!("../investment/debt_reporting_source.rs"),
    include_str!("../investment/dividend_source.rs"),
    include_str!("../family/after_death_income_source.rs"),
    include_str!("../investment/interest_source.rs"),
    include_str!("../investment/investment_allocation_source.rs"),
    include_str!("../investment/investment_validation_source/mod.rs"),
    include_str!("../investment/oid_source.rs"),
    include_str!("../investment/reit_dividend_source.rs"),
    include_str!("../investment/investment_validation_source/annual.rs"),
    include_str!("../family/able_distribution_source.rs"),
    include_str!("../family/education_account_source.rs"),
    include_str!("../family/scholarship_source.rs"),
    include_str!("../family/accepted_dependents.rs"),
    include_str!("../home/home_site_source.rs"),
    include_str!("../farm/farm_averaging_source.rs"),
    include_str!("../farm/farm_nol_source.rs"),
    include_str!("../business/optional_se_source.rs"),
    include_str!("../income/clergy_source.rs"),
    include_str!("../foreign/foreign_earned_source.rs"),
    include_str!("../business/household_employment_source.rs"),
    include_str!("../family/able_contribution_source.rs"),
    include_str!("../family/trump_account_source.rs"),
    include_str!("../retirement/public_safety_source.rs"),
    include_str!("../income/espp_source.rs"),
    include_str!("../activity/qbi_history_model.rs"),
    include_str!("../attribution/retained_loss_owners_model.rs"),
    include_str!("../credits/retained_research_model.rs"),
    include_str!("atnol_model.rs"),
    include_str!("nol_model.rs"),
    include_str!("capital_history_model.rs"),
    include_str!("../income/iso_source.rs"),
    include_str!("../investment/related_exchange_model.rs"),
    include_str!("../investment/section1231_model.rs"),
    include_str!("../retirement/disaster_carry_model.rs"),
    include_str!("accepted_screening.rs"),
    include_str!("../core/checklist_page.rs"),
    include_str!("../foreign/foreign_loss_source.rs"),
    include_str!("../attribution/prior_care_identity_source.rs"),
    include_str!("../credits/accepted_credits.rs"),
    include_str!("../credits/business_credit_source.rs"),
    include_str!("../credits/clean_vehicle_source.rs"),
    include_str!("../credits/education_recapture_source.rs"),
    include_str!("../credits/personal_refueling_source.rs"),
    include_str!("../credits/saver_source.rs"),
    include_str!("../credits/tax_credit_bond_source.rs"),
    include_str!("../family/adoption_source.rs"),
    include_str!("../family/care_source.rs"),
    include_str!("../family/education_source.rs"),
    include_str!("../federal/minimum_tax_source.rs"),
    include_str!("../foreign/foreign_allocation_source.rs"),
    include_str!("../foreign/foreign_claim_source.rs"),
    include_str!("../foreign/foreign_credit_source.rs"),
    include_str!("../foreign/foreign_investment_source.rs"),
    include_str!("../foreign/foreign_source_basics.rs"),
    include_str!("../foreign/foreign_state_source.rs"),
    include_str!("../forms/disallowance_source.rs"),
    include_str!("../health/form_1095a_source.rs"),
    include_str!("../health/marketplace_claim_source.rs"),
    include_str!("../health/marketplace_record_source.rs"),
    include_str!("../home/mortgage_certificate_source.rs"),
    include_str!("../home/mortgage_subsidy_source.rs"),
    include_str!("../passthrough/bba_audit_source.rs"),
    include_str!("federal_model.rs"),
    include_str!("return_reporting.rs"),
    include_str!("income_adapters.rs"),
    include_str!("../deductions/adjustments_model.rs"),
    include_str!("../passthrough/bba_audit_model.rs"),
    include_str!("../deductions/charity_model.rs"),
    include_str!("../core/checklist_model.rs"),
    include_str!("../core/shared/document_parties.rs"),
    include_str!("../attribution/community_property_model.rs"),
    include_str!("../credits/credits_model.rs"),
    include_str!("../family/dependents_model.rs"),
    include_str!("../home/disaster_model.rs"),
    include_str!("../deductions/educator_model.rs"),
    include_str!("../passthrough/estate_trust_k1_model.rs"),
    include_str!("../family/filers_model.rs"),
    include_str!("../payments/form_1040_es_model.rs"),
    include_str!("../deductions/form_1098_e_model.rs"),
    include_str!("../home/form_1098_model.rs"),
    include_str!("../investment/form_1099_b_model.rs"),
    include_str!("../investment/form_1099_div_model.rs"),
    include_str!("../payments/form_1099_g_model.rs"),
    include_str!("../investment/form_1099_int_model.rs"),
    include_str!("../investment/form_1099_oid_model.rs"),
    include_str!("../farm/form_1099_patr_model.rs"),
    include_str!("../retirement/form_1099_r_model.rs"),
    include_str!("../business/form_2106_model.rs"),
    include_str!("../payments/form_2210_model.rs"),
    include_str!("../family/form_2441_model.rs"),
    include_str!("../credits/form_4136_model.rs"),
    include_str!("../home/form_8396_model.rs"),
    include_str!("../family/form_8814_model.rs"),
    include_str!("../investment/form_8815_model.rs"),
    include_str!("../family/form_8863_model.rs"),
    include_str!("../credits/form_8880_model.rs"),
    include_str!("../health/form_8889_model.rs"),
    include_str!("../credits/form_8936_model.rs"),
    include_str!("../health/form_8962_model.rs"),
    include_str!("../income/form_w2g_model.rs"),
    include_str!("../retirement/ira_contributions_model.rs"),
    include_str!("../payments/other_taxes_model.rs"),
    include_str!("../payments/payments_model.rs"),
    include_str!("../deductions/schedule_1a_model.rs"),
    include_str!("../deductions/schedule_a_model.rs"),
    include_str!("../business/schedule_c_model.rs"),
    include_str!("../business/expense_source.rs"),
    include_str!("../farm/cooperative_source.rs"),
    include_str!("../investment/schedule_e_model.rs"),
    include_str!("../health/se_benefits_model.rs"),
    include_str!("../retirement/ssa_1099_model.rs"),
    include_str!("../income/w2_model.rs"),
    include_str!("../income/wages_records_view.rs"),
    include_str!("../core/shared/record_choice.rs"),
    include_str!("../home/mortgage_point_treatment.rs"),
    include_str!("../investment/bond_premium_source.rs"),
    include_str!("../deductions/accepted_deductions.rs"),
    include_str!("../deductions/charity_sources.rs"),
    include_str!("../home/mortgage_source.rs"),
    include_str!("../income/prior_recovery_source.rs"),
    include_str!("../investment/investment_borrowing_source.rs"),
    include_str!("../health/medical_care_source.rs"),
    include_str!("../deductions/vehicle_loan_source.rs"),
    include_str!("../deductions/accepted_adjustments.rs"),
    include_str!("../health/hsa_source.rs"),
    include_str!("../health/archer_source.rs"),
    include_str!("../health/health_contribution_source.rs"),
    include_str!("../health/health_employer_source.rs"),
    include_str!("../health/health_marketplace_source.rs"),
    include_str!("../retirement/social_sources.rs"),
    include_str!("../retirement/retirement_adjustment_sources.rs"),
    include_str!("../retirement/reservist_sources.rs"),
    include_str!("../retirement/ira_repayment_sources.rs"),
    include_str!("../retirement/rmd_sources.rs"),
    include_str!("../retirement/roth_history.rs"),
    include_str!("../retirement/pension_contribution_source.rs"),
    include_str!("../retirement/ira_contribution_sources.rs"),
    include_str!("../family/alimony_model.rs"),
    include_str!("../business/moving_model.rs"),
    include_str!("../deductions/student_loan_source.rs"),
    include_str!("../deductions/educator_source.rs"),
    include_str!("../business/employee_expense_model.rs"),
    include_str!("../cost_recovery/vehicle_model.rs"),
    include_str!("../credits/investment_credit_source.rs"),
);
// End of the tools/split-hubs block.
