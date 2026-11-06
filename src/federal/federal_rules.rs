// Coverage: Form 1040 — Supported (audited 2026-10-12).
//   Lines: 1a–1z, 2a–7a and 8–37; all filing statuses, dependent/age/blind and
//   eligible MFS spouse standard deduction boxes, senior deduction, draft 2026
//   Tax Table below $100k and QDCG/Schedule D/8615/Schedule J worksheets, AMT.
//   Unit19 operating output correction: Schedule1 lines3/6 reuse collected
//   source net, including ministerial expense restoration and routed farm
//   receipts. Verified C/F cents share the native annual amount and C-first
//   rounding through kernel::ratio::project_rounded_pair; tests/operating_form_sources
//   and abroad_clergy's no-farm minister preserve AGI/tax and source lifecycle.
//   Unit B10-2 adds household 1b, Medicaid 1d/8s, combat EIC election 1i, strike
//   benefits and Form 8919 coverage-No pay on 1h, and an ordinary EIC-decline
//   choice with a line 27c filing reminder. Printed checkboxes remain deferred.
//   Source evidence: src/tests/income/wage_specials.rs A–F, interview save/import tests,
//   statutory_wages.rs, forms/w2.rs, other_income.rs, IRA difficulty-of-care room.
//   Form 4972 is included in line 16; an elected distribution leaves 5a/5b.
//   Exclusions: line 38 penalty; Schedule 3-A is a review note. Year-end release
//   review remains required: final 2026 W-2 instructions, draft 2026 return forms
//   and prior-year final instructions where 2026 instructions are unavailable.
//   Form 8814 (2026-10-09, child_election.rs): a parent's election for a child's interest and
//   dividends is calculated; its line 9 is in lines 3a and 3b, line 10 in line 7a (Schedule D line
//   13), line 12 in line 8 (Schedule 1 line 8z) and line 15 in line 16 (box 1 printed only, and
//   line 3c, line 7b checkboxes: the return has no checked-box line kind). The tax is no credit
//   base and no Schedule J income. ChildInvestmentIncome is removed.
// Coverage: "Anything else?" situations list (redesign 3, unit M1-P3, 2026-09-24).
//   Unprepared is the one owner of what the estimate refuses (RareSituation::refusal_info for the
//   less-common wage/income and itemized situations, RareAdditionalTax::refusal_info, and the
//   payload-less items in Unprepared::refused): each names its form lines, its issue and its
//   review name, without a schedule. refuse_situations reports every selected item as an
//   issue on the Situations people often miss page, so the estimate is Unsupported, never zero. It replaces the Batch A
//   Answer fields (W-2 church/minister and statutory employee, 1099-R unmodeled boxes, Schedule C
//   depletion/pension/7205, Schedule E nonresidential, 4562 asset disposition and special class,
//   Schedule A other lines, Schedule 3 13c/13d, screening Schedule 2 lines, 8960 modifications,
//   5329 education taxes, 1040 elections, Schedule D other forms, 1099-MISC/NEC boxes) and the
//   catch-all answers (screening foreign/property/carryovers/special_taxes/uncertain,
//   adjustments other_adjustments, deductions special_itemizing/special_work_deductions,
//   credits special_credits), plus the M1 exclusions (other income
//   kinds, itemizing, dependents' credits, education, HSA, foreign, AMT, other credits, home
//   kerosene). Evidence: federal_rules.rs (RareSituation::refusal, refuse_situations),
//   federal_model.rs (RareSituation, Situations), interview.rs; tests:
//   every_rare_situation_refuses_with_its_name,
//   whole_return_situations_refuse_and_notice_on_the_filers_pages (interview.rs) and each Batch A
//   test.
//   M3-P15 (2026-09-25): the table is regrouped by destination page;
//   refuse_situations names each issue by its page. Unit B9-3
//   (2026-10-10) removed refuse_person_facts: a filer's final return and the section 152(c)(4)
//   competing claim are calculated (family(), household(), after_death.rs, community.rs).
//   The whole-return variants left the table for the filers' pages (YourSituations on About
//   you, SpouseSituations on the spouse's page, FederalPerson.whole_return); whole_return()
//   refuses dual-status/territory returns and Form 8958 community property by name under
//   About you, gives notices for the
//   section 6013(g)/(h) nonresident spouse choice (an ordinary joint return of both spouses'
//   worldwide income plus the signed election statement, Pub. 519) and Form 8857 innocent
//   spouse relief (a separate request that leaves the joint return's tax unchanged), and leaves
//   Form 8379 to its page. Test: whole_return_situations_refuse_and_notice_on_the_filers_pages.
//   Evidence: federal_rules.rs (estimate_with_ira_pass, standard_deduction), income_tax.rs (Rates),
//   extended_rules.rs (senior_deduction), interview.rs; tests:
//   filing_tables_match_every_published_irs_interval (health_accounts.rs),
//   published_ordinary_bracket_edges_all_statuses (taxes.rs),
//   qualifying_survivor_uses_joint_rates_but_own_credit_thresholds (family.rs); Rev. Proc. 2025-32,
//   draft p1040 table, P.L. 119-21, i1040gi
//   Unit family (2026-10-08), family()/household(): head of household takes a single qualifying child
//   whether or not the child is a citizen or resident (IRC section 2(b)(1)(A)(i); Pub. 501 (2025)
//   Table 4), except a child married at the close of the year who is not a citizen or resident
//   (section 2(b)(1)(A)(i)(I)-(II)); that marital status is now the stored fact
//   PersonFacts::MARRIED_AT_YEAR_END (Forms 8332/2120 unit, 2026-10-09), which replaced the
//   conservative unknown path (unknowns::supported's Single status and
//   WARN_HEAD_CHILD_MARITAL_STATUS_UNKNOWN are gone); qualifying surviving spouse takes a son or
//   daughter who is a
//   qualifying relative as well as a qualifying child (section 2(a)(1)(B); Pub. 501 (2025)
//   "Qualifying Surviving Spouse" test 3), waiving the gross-income, joint-return and
//   claimed-by-another tests and keeping support, citizenship and the all-year home. Tests:
//   head_of_household_takes_a_non_citizen_child_unless_the_child_is_married,
//   head_of_household_still_needs_a_dependent_for_anyone_but_a_qualifying_child,
//   qualifying_survivor_counts_a_son_who_is_a_qualifying_relative,
//   qualifying_survivor_keeps_the_support_citizenship_and_relationship_tests (family.rs).
//   Forms 8332 and 2120 (2026-10-09, family()): a custodial parent's release (PersonFacts
//   YOU_RELEASED / RELEASED_TO_YOU, FederalDependent::custody_release) and a multiple support
//   agreement (multiple_support) change who has the child: the parent holding the release has the
//   dependent (a qualifying child or a qualifying relative) but never head of household, the
//   earned income credit or the care credit; the parent who released keeps head of household (IRC
//   section 2(b)(1)(A)(i) without section 152(e)), the earned income credit and the Form 2441
//   qualifying individual but not the dependent or a qualifying surviving spouse; a married
//   parent living apart for the last six months is considered unmarried through the child
//   (section 7703(b)(1), "would be so entitled but for section 152(e)"), now with the section
//   152(b)(3) citizen or resident test; a multiple support relative is a dependent but never gives
//   head of household (section 2(b)(3)(B)(ii)). The Form 1040 filing-status child name and the
//   Dependents columns (5)(a) and (7) are printed-document items. Evidence: lib.rs Coverage Forms
//   8332 and 2120.
// Coverage: Schedule 1 — Partial (audited 2026-09-24).
//   Lines: Income lines 1, 2a, 3, 4, 5, 6, 7, 8a–8e, 8h–8l, 8q, 8r, 8z; adjustments 11, 12, 13, 14, 15–18,
//   19a, 20, 21, 23, 24a/b/j/k, 501(c)(D). Line 14 moving expenses is calculated from
//   Form 3903 (moving.rs, unit form-3903, 2026-10-09; the "Moving expenses" kind on More
//   adjustments; Armed Forces and intelligence-community moves only; the line 14 storage-only
//   checkbox is a printed-document matter). Exclusions: adjustment lines 24c–24z through the
//   OtherAdjustments item of More adjustments, refused by name. Income lines without a kind of
//   their own are entered as Other income items (line 8z); the RareSituation::OtherIncome refusal
//   is gone.
//   Line 12 (2026-10-09, Form 2106 unit, employee_expenses.rs): the sum of the Forms 2106 line 10 of
//   Armed Forces reservists, qualified performing artists and fee-basis state or local officials
//   (return line schedule_1_employee_business_expenses). The reservist and fee-basis amounts are
//   subtracted with the other early adjustments (so the phase-outs that read AGI see them); a
//   performing artist's amount is subtracted last, after the section 62(b)(1)(C) AGI test, and is
//   left out with one note per failed test. The line 8 excess reimbursement is wage income on line
//   1a, and the Form 1116 line 3e restore counts the deduction with the U.S.-related adjustments.
//   Impairment-related work expenses (Schedule A line 17j) are not line 12. Evidence:
//   employee_expenses.rs; tests: src/tests/business/employee_expenses.rs, src/tests/cross/returns.rs fixtures
//   reservist-drill-travel, performing-artist-agi-boundary, fee-basis-official-excess-reimbursement.
//   Correction (Schedule 1 map, 2026-09-24): line 8f (Form 8889 income) and line 8p (excess
//   business loss) are calculated (hsa_income; excess_business_loss add-back); line 8v (digital
//   assets, new on the 2026 draft) is refused by unit A7 (now the DigitalAssetPay situation).
//   Unit B6b (2026-10-09): lines 8h (jury pay), 8j (hobby sales), 8r (taxable scholarships) and
//   24a (jury pay turned over to the employer) are their own return lines, written by the Other
//   income page's Jury duty pay, Sales from a hobby and Scholarship or fellowship kinds (test
//   jury_pay_and_employer_remittance_preserve_both_return_lines, general.rs; src/tests/income/relief_income.rs
//   fixtures G, H, I). The OtherIncome refusal situation is gone: line 8z is the Other income page's
//   items. Stored computation, no page: the other FederalIncome kinds no page writes (see Form
//   2555, Form 8853).
//   Evidence: federal_rules.rs (adjustments, line-8 text), hsa.rs, sep.rs, health.rs, alimony.rs,
//   tax_refund.rs, debt.rs; tests: alimony_paid_and_received_flow_through_return_without_double_counting
//   (family.rs), educator_excess_is_itemized_once_and_only_when_itemizing (deductions.rs);
//   C-fixture (2026-09-25, repayment.rs): schedule_1_typical_return_lines_match_hand_computation
//   (lines 7, 10, 11, 18, 21, 26 by hand; 11 and 21 only through the adjustments aggregate),
//   educator_expenses_cap_each_educator_separately_on_a_joint_return,
//   student_loan_interest_phases_out_by_2026_magi,
//   same_year_unemployment_repayment_nets_on_schedule_1_line_7,
//   early_withdrawal_penalty_deducts_on_line_18_without_reducing_interest; draft
//   f1040s1, f1040s1a, i1040gi
//   Unit u5-adjust (2026-09-28): line 19a is asked on More adjustments (the "Alimony paid" kind,
//   one row per pre-2019 agreement, by owner; ALIMONY_2024/2025_EXCLUDED_ABOVE_PAID on both
//   sides); line 23 (Archer MSA, Form 8853 Part I) is the HSA page's since unit B6a (the ArcherMsa
//   refusal is gone) and line 24f (501(c)(18)(D)) is refused by name through OtherAdjustments,
//   whose text now names it; the stored pension_contributions answer is erased by the walk.
//   Verified against Pub. 504 (2025): child support first, pre-2019 instrument, third-year
//   recapture ($15,000 thresholds). Tests: src/tests/federal/unasked_pages.rs
//   alimony_paid_through_more_adjustments_by_hand (fixture), alimony_agreement_date_boundary_is_january_1_2019,
//   alimony_paid_no_more_than_child_support_due_deducts_nothing,
//   alimony_page_rules_refuse_on_the_server, more_adjustments_keeps_its_refusal_kinds_beside_alimony.
//   Unit u6-alimony (2026-09-28): line 2a is asked on the prizes, royalties, equipment rent and
//   alimony topic (the "Alimony received" kind, one entry per pre-2019 agreement, by owner);
//   line 2b's date is the stored agreement date. Line 2a left the former OtherIncome refusal.
//   Verified against the 2025 Form 1040 instructions (line 2a: on or before 2018-12-31, unless
//   changed after 2018 to expressly exclude it) and Pub. 504 (2025) Worksheet 1; correction: a
//   change adopting the post-2018 rules during 2026 splits the year (Pub. 504 Example 2) and is
//   now refused by name instead of zeroing the whole year (alimony.rs). Tests:
//   src/tests/family/alimony_received.rs.
//   Unit form-3903 (2026-10-09): line 14 comes from Form 3903 line 5 (moving_expense_deduction),
//   Form 3903 line 5 "No" adds the excess reimbursement to Form 1040 line 1h; per-owner lines 3 to 5
//   against the owner's W-2 box 12 code P. Tests: src/tests/business/moving.rs fixture_a to fixture_e.
// Coverage: Schedule B — retained-source listings and totals (Unit14).
//   schedule_b.rs presents payer amounts, nominee/estate/prior/accrued and
//   ABP/OID source adjustments without a second tax calculation; seller-home
//   mortgage interest is first. InvestmentComponents owns retained characters.
//   Form8815 canonical accepted-source links reduce current interest once and
//   leave redemption proceeds unchanged. Exempt income stays off ScheduleB.
//   PartIII stores distinct required account/FBAR/countries/trust answers from
//   the actual account and trust records. No FBAR filing
//   or export is claimed. Latest2026 ScheduleB draft instructions pp.B-1–B-3.
//   Annual debt accepts documented regular/stripped annual amounts and strict
//   original fixed-principal de-minimis tests. Missing contingent computations,
//   required net-income facts, terminal premium relief and K1-origin8912 credit
//   interfaces retain their named source/benefit limits; no whole-weight closure.
//   Tests: investment_statements.rs; existing investment/family/returns fixtures.
// Coverage: Schedule 3 — Supported (audited 2026-09-24).
//   Lines: Lines 1 (1116/election), 2, 3, 4, 5 (2025 Form 5695 carry only), 6a (3800: 8846 carry
//   only), 6b, 6c, 6d, 6f/6m, 6g, 6h, 6i, 6j, 6k, 6l, 9, 10, 11, 12, 13 (2439, §1341). Exclusions:
//   other 3800 credits, 6e, 6z refused explicitly via the OtherCredits situation (formerly
//   special_credits); draft Schedule 3-A not applied, only a review note.
//   Refused explicitly (unit A9, from the Schedule 3 map): 2026 draft lines 13c (net elective
//   payment election) and 13d (deferred section 965 liability) through the
//   ElectivePaymentOrSection965 situation (M1-P3; formerly elective_payment_or_section_965).
//   Evidence: federal_rules.rs (prior_other_credits, return lines), extended_rules.rs, care.rs,
//   minimum_tax.rs, adoption.rs, clean_vehicle.rs, tax_credit_bonds.rs, bba_audit.rs,
//   marketplace.rs, fuel_credit.rs; tests:
//   personal_refueling_credit_caps_each_unit_and_reduces_schedule_three,
//   earlier_schedule_3_credit_reduces_form_8912_line_11_before_bond_credit (credits.rs),
//   ordinary_foreign_credit_limits_both_regular_and_amt_taxes (foreign.rs),
//   saver_follows_education_care_and_precedes_child_credit (family.rs); i8962/Rev. Proc. 2025-25,
//   Notice 2025-67, f2439, draft f1040s3a
//   C-fixture (2026-09-25, credits.rs): schedule_3_credits_limited_in_worksheet_order_match_hand_computation
//   (care, education, saver and the child credit competing for $1,088 of tax; lines 2, 3, 4, 7, 8
//   and Form 1040 19-22, 27a-29 by hand), exhausted_tax_pushes_the_child_credit_to_the_additional_credit,
//   saver_credit_is_zero_above_the_head_of_household_ceiling, schedule_3_line_8_feeds_form_1040_line_20
//   (line 8 = lines 1-4, 5a and 7 per the 2026 draft).
//   The OtherCredits situation is gone (M3-P12, 2026-09-25, ALL shrinks to 44): its twelve
//   covered credit types are reachable from the credits hub's More credits page (interview.rs)
//   for the first time; the residual "other 3800 credits, 6e, 6z" scope, and Form 8862 (see
//   claim_repayment.rs), had no situation of their own besides OtherCredits and are now
//   unrefused, flagged there rather than fixed here.
//   M3-P15 (2026-09-25) closes that gap: 6e was the AlternativeMotorVehicle situation (removed
//   2026-10-09: 6e is reserved for 2026), 6z OtherCredit, the other 3800 credits BusinessCredits and Form 8862 EicAfterDisallowance, each a
//   refusal kind of More credits; 13c/13d are ElectivePayment and Section965. T2-46 (2026-09-26)
//   replaces EicAfterDisallowance with the calculated Form 8862 (forms/form_8862.rs; the
//   MoreCredit::AfterDisallowance record).
// Coverage: Schedule C — Supported (Unit11 specials updated 2026-10-12).
//   Lines: gross receipts (linked 1099-NEC/K/MISC/G once), returns and allowances, other income,
//   COGS with cost/lower-of-cost-or-market/section471(c) inventory, ordinary/meals
//   50%/DOT80%/fully deductible/annual de minimis expenses, employee plans, working-interest
//   depletion, MACRS/section179/bonus, start-up and section197 amortization, home office,
//   standard-mileage/owned-actual/leased vehicles, owner health/SEP/SIMPLE/qualified/actuarial
//   defined-benefit amount, payroll, tips/overtime, 8846/4136, material participation, at-risk
//   and QBI feed. Refused: LIFO/section263A via inventory eligibility, timber/FormT and Form7205
//   via special_treatment. Foreign
//   services are marked on the business page (BusinessDetails.source, unit B7b; the exclusion is the
//   person's Form 2555 on Work abroad, see foreign_housing.rs).
//   Unit B6b (2026-10-09): professional gambling (BusinessActivity::Gambling) is the optional
//   "Gambling as a business" record of the Business page's More about this business
//   (ProfessionalGambling: losing wagers, W-2G withholding, eligible), with the activity's
//   accounting method stored so the page reopens exactly (src/tests/income/relief_income.rs fixture J).
//   The self-employed overtime record (FederalBusiness.overtime) comes from the business page's
//   explicit overtime choice; the §197 intangibles list (FederalBusiness.intangibles) is written by the
//   Business and Farm pages' "More" folds. The qualified tips record is on both pages (More about this
//   business/farm, F12) and the tips allowed under §224 are removed from the business's QBI.
//   Unit11 replaces A11's depletion/employee-plan refusal with supported lines12/19; Form7205
//   line27a and timber/FormT remain named refusals through the Schedule C special answer.
//   Header A-J and vehicle PartIV records are stored; print rows remain PRINT-owned.
//   Sources: i1040sc (2026 draft), Pub.463/538/560 (2025), Reg.1.263(a)-1(f),
//   Rev.Procs.2018-25 Table4 and 2019-26/2020-37/2021-31/2022-17/2023-14/2024-13/2025-16/2026-15
//   Table3. Tests: schedule_c_specials A-G, line_audit_c_se extended F6, information_returns
//   schedule_c_receipt_bound_*; exact nine-year inclusion tables live in year_constants.rs.
//   Form 8829 actual expenses: calculated since unit T2-39 (2026-09-26),
//   see home_office.rs; line 30 is its line 36.
//   Evidence: federal_rules.rs, business_rules.rs, depreciation.rs, section179.rs, kernel/cost_recovery/macrs_tables.rs,
//   startup.rs, amortization.rs, health.rs, sep.rs, extended_model.rs (BusinessInventory),
//   home/sites.rs (canonical home/owner/period sources); tests:
//   business_inventory_cost_flows_through_profit_se_tax_qbi_and_restore,
//   business_expense_limits_apply_to_meals_and_exclude_entertainment,
//   section179_precedes_bonus_macrs_and_preserves_ubia,
//   startup_initial_deduction_phaseout_and_opening_month_match_irs_example (business.rs); i1040sc,
//   Pub 946
//   Unit 481a (2026-10-09): a section 481(a) adjustment of Form 3115 (FederalBusiness.method_changes,
//   method_change.rs) lands on line 6 other income when positive (the 2026 part, inside receipts())
//   and on Part V line 48 to line 27b when negative (inside the operating costs and the
//   business_{id}_expense_line27b_other row); rows business_{id}_method_change_{n} and ..._remaining
//   and NOTE_METHOD_CHANGE_FORM_3115 per record; QBI, SE, Form 8995 and Form 6251 follow the profit.
//   Tests: src/tests/cost_recovery/method_change.rs.
//   Unit C9a (2026-09-25): FederalBusinessExpense.category is the BusinessExpenseCategory enum
//   (Part II lines 8, 10, 11, 14-18, 20a-26, 27b with a label), one return line per category
//   (business_{id}_expense_line{n}_{name}); 9, 13 and 24b stay computed; Unit11 adds12 and19,
//   while27a remains refused. No C9a total changed. Historical STORAGE_FORMAT7. Test: business.rs
//   schedule_c_expense_lines_are_reported_by_category. T2-45y: the ten farm-only categories
//   (Schedule F lines 11, 13, 16-19, 23, 26, 27, 31) are refused by name on a non-farm business
//   ("Farm-only expense line"); test src/tests/farm/farm.rs
//   schedule_c_business_with_a_farm_only_expense_line_is_refused_by_name.
//   C-fixture (2026-09-25, business.rs): sole_proprietor_return_matches_hand_computation (lines 1-31,
//   Schedule SE, Schedule 1 and Form 8995 by hand), net_profit_under_400_owes_no_self_employment_tax,
//   dot_meals_deduct_eighty_percent, simplified_home_office_is_capped_by_area_and_income,
//   business_loss_reduces_income_without_qbi.
//   Unit C16a (2026-09-25): FederalBusiness.withholding (1099-NEC/MISC/K federal withholding) joins
//   the 1099 withholding total on Form 1040 line 25b once, never an expense. STORAGE_FORMAT 13.
//   Test: general.rs business_1099_withholding_reaches_line_25b_once.
//   Unit C19a (2026-09-25): line 9 car and truck is now standard mileage plus the business-mile share of
//   an actual-expense vehicle's operating costs (business_{id}_car_and_truck); vehicle depreciation
//   reaches line 13 through the asset.
//   Unit form-8941 (2026-10-09): line 14 (employee benefit programs) is reduced by the Form 8941
//   line 12 credit (section 280C(h); the 2026 draft i1040sc line 14), the way line 26 is reduced by
//   the Form 5884 credit (business_{id}_expense_line14_employee_benefit_programs). Test: tests/
//   credits.rs form_8941_full_credit_with_a_seasonal_worker_and_a_partial_year.
//   Unit form-8994 (2026-10-09): line 26 (wages) is also reduced by the Form 8994 line 1a credit and
//   line 14 by the line 1b premium credit (section 280C(a); the premium line is the inventor's
//   decision of 2026-10-09: the 2026 draft i1040sc does not name it), beside Form 8941's. Tests:
//   src/tests/credits/credits.rs form_8994_wage_method_follows_the_instructions_example,
//   form_8994_insurance_premiums_and_k1_code_bb.
//   Line audit D1-D4 (unit F6, 2026-10-10): the emitted lines 8-27b foot to line 28. A standard-mileage
//   vehicle's business share of personal property tax is line 23, not line 9; line 23 also holds the
//   employer FICA on reported tips that Form 8846 line 4 does not take (FederalBusiness::line23_additions);
//   Part V / line 27b holds the start-up costs' initial deduction and amortization and the section 197
//   amortization (amortization_cents) beside the Other categories and a negative section 481(a)
//   adjustment; line 17 is reduced by the Form 8881 start-up credit (plan_startup_credit_cents).
//   Listed-property recapture (D10): a continuing vehicle used 50% or less in 2026 and more than 50% earlier
//   (ListedProperty::Vehicle.earlier_business_percent) adds the depreciation allowed beyond straight-line
//   ADS at that use to line 6 (BusinessAsset::recapture, FederalBusiness::receipts), with Form 4797 Part IV
//   lines 35-37 column (b) and the basis adjusted by it; the earlier-year percentage is one value, and prior
//   depreciation above the basis at this year's use is rejected as before. Tests: src/tests/cross/line_audit_c_se.rs
//   d1_*, d2_*, d3_*, d4_*, d10_*.
// Coverage: Schedule SE — Supported (audited 2026-09-25).
//   Lines: 92.35% net earnings, $184,500 wage-base coordination with W-2/4137 wages (Form 8919
//   wages are the Wages page's Form 8919 kind: FederalIncome::EmployeeWages, src/tests/income/relief_income.rs),
//   12.4%+2.9%, half-SE deduction, farm and nonfarm optional methods (Part II), partnership SE,
//   per-spouse computation. Unit41 adds ministerial wages/housing/fees, approved
//   Form4361/Form4029, notary and totalization exclusions, with optional nonfarm
//   eligibility using actual gross/net and lifetime history (optional_se.rs).
//   Evidence: federal_rules.rs self_employment/se_from_net, optional_se.rs, farm.rs,
//   partnership.rs, employee_tips.rs; tests:
//   optional_se_regular_losses_offset_optional_earnings_without_percentage_reduction,
//   employee_8919_social_base_follows_tips_and_precedes_schedule_se (business.rs),
//   farm_and_schedule_c_share_owner_se_tax_but_keep_separate_profit_records (farm.rs),
//   employee_tip_social_base_precedes_self_employment_and_is_per_spouse (family.rs); i1040sse
//   (federal_rules.rs)
//   Unit C10a (2026-09-25), now W2Coverage::ChurchEmployee, puts box1 on line5a; 5b is
//   92.35% with the $100 floor; line 6 = 4c + 5b enters the ordinary computation (wage base,
//   half-SE deduction; the church share of the half-SE deduction is kept out of QBI). Boxes 3/5
//   on a church W-2 are refused naming line5a. Unit41 replaces the old minister
//   refusal with actual clergy records and retains the named Chapter11 refusal.
//   The partnership actual-method path carries church earnings since line auditD7. Tests: taxes.rs
//   church_employee_income_enters_schedule_se_line_5, church_employee_income_under_the_floor_is_zero,
//   church_w2_with_social_security_wages_is_refused.
//   C-fixture (2026-09-25, taxes.rs): self_employment_with_the_wage_base_binding_matches_hand_computation
//   (lines 2-13 with the wage base binding, Schedule 2 line 11, Form 8995 and the return by hand),
//   wage_base_exactly_met_leaves_only_the_medicare_part, nonfarm_optional_method_caps_at_7560,
//   joint_owners_each_have_their_own_schedule_se, church_income_and_business_profit_share_line_6.
//   Line audit D5-D9 (unit F6, 2026-10-10): line 4c under $400 is -0- beside church employee income, on the
//   actual and optional paths (se_from_net); the partnership actual-method path (partnership.rs
//   owner_actual_se) keeps church employee income on line 6 and includes a farm's routed 1099-G box 7 and
//   1099-PATR income; the farm optional method's gross and profit include that routed income
//   (optional_se.rs); with an optional method the printed lines follow the instructions: the farm method skips
//   1a and 1b, the nonfarm method skips line 2, line 3 and 4a hold the method not elected, line 4b is the total of
//   lines 15 and 17 (optional_se.rs OptionalLines, schedule_se_lines). The emitted lines foot (tests
//   assert_schedule_se_foots). Tests: src/tests/cross/line_audit_c_se.rs d5_* to d9_*.
//   B9-7 R8 (2026-10-12): a farming/fishing partnership's post-limit box 14A
//   source joins line 1a, with other partnership sources on line 2. The same
//   regular source drives the owner's one wage base, half-SE/QBI allocation,
//   and optional election; spouses keep separate computations.
//   Evidence: src/tests/passthrough/k1_items.rs and src/tests/cross/interview.rs k1_box14_*;
//   i1065sk1 (2025 final), Schedule SE and instructions (2026 drafts).
// Coverage: Schedule 2 — Supported (audited 2026-09-25).
//   Native K-1 box20AI deferred-compensation tax appears on2026 line13h,
//   in14/15/21 once; actual settled cents also supply Form8379 owner parts.
//   Negative8978 residual offsets the same native Chapter1 PartII subtotal used
//   by the section1341 comparison; payroll/excise remain due, education recapture
//   enters line18 once. Evidence: src/tests/credits/bba_chapter_one.rs (approved D4).
//   Lines: Line 1 AMT, 1z/13z Form 8978, 2 excess APTC, 4 SE tax, 5, 6, 8 (5329 early distribution,
//   HSA/MSA/ABLE/IRA excess), 9 Schedule H, 11 Additional Medicare, 12 NIIT, 8828 recapture,
//   education-credit recapture, Trump-account taxes. 2026 draft numbering (unit A17): refused
//   explicitly through the payload-less items of the Other taxes page (OtherTax::refused):
//   HousingCreditRecapture (line 9 Form 8611, 13n 8697/8866), OtherAdditionalTaxes (lines 1d–1f,
//   1y, 7, 8, 10, 12, 13a, 13g–13m, 19a–19c), Section965 (Schedule 3 line 13d),
//   AlternativeMinimumTax adjustments and EducationAccountTaxes (Form 5329 Parts V and VIII); the
//   RareSituation variants and the screening answers they replaced are gone; Additional
//   Medicare is emitted as line 11 (self-employment) and 17b (wages); Form 8828 recapture is 13b.
//   Evidence: federal_rules.rs (return lines 17/23), minimum_tax.rs, marketplace.rs,
//   household_employment.rs, employee_tips.rs, employee_wages.rs, retirement.rs,
//   mortgage_recapture.rs; tests: additional_medicare_and_niit_are_not_silently_zero,
//   marketplace_repayment_enters_amt_comparison_before_total_tax (health_accounts.rs),
//   household_return_adds_employer_tax_and_employee_withholding_only_once (family.rs),
//   early_distribution_rounds_form_lines_and_each_filer_tax (retirement.rs); i6251, i8962, i5329,
//   i8959, i8960, Pub 926, draft f1040s2
//   C-fixture (2026-09-25, src/tests/federal/taxes.rs): schedule_2_part_ii_taxes_match_hand_computation (lines 4, 8,
//   11, 13c, 17b, 21 by hand), amt_from_private_activity_interest_reaches_line_2 (Form 6251 lines 1-11),
//   additional_medicare_splits_at_the_threshold, schedule_2_unmodeled_lines_stay_refused.
// Coverage: Schedule A (B10-4,2026-10-10): medical floor, separate5b/5c taxes, SALT
//   phase-down, GST income-distribution tax6, mortgage8a–8e (mortgage_limit.rs), investment
//   interest9, charitable limits/floor/carryovers11–15, ordinary casualty16, qualified disaster
//   loss17b and increased-standard17c, gambling17a, IRD estate tax17e, bond premium17f,
//   debt-instrument loss17g, claim-of-right17h, annuity loss17i, disabled employee expenses17j,
//   educator/coach expenses17k, totals17z and18 after§68. Every numeric line carries FormLine.
//   Foreign-tax deduction6 is implemented by original Unit1 follow-up (extended_rules.rs;
//   src/tests/foreign/foreign_deduction.rs fixtureE and footings);17d remains with the casualty unit. Checkbox rows8/19
//   await B10-8's text row kind. Eligibility includes P.L.119-111 early-childhood educators.
//   Sources: IRS2026 Schedule A draft (Created5/12/26), Pub.9362026 draftSep30, Pub.529,
//   Pub.525 and IRC62,67,163,164,171. Tests: schedule_a_specials A/B/C/D/F/G/H fixtures and existing
//   itemized/charity/mortgage suites. Stored shapes are owned by federal_model.rs.
// Coverage: Form 8863 / 1098-T — Supported (audited 2026-09-24).
//   Lines: AOTC per student (100% of first $2,000 + 25% of next $2,000), LLC per return (20% of
//   $10,000), MAGI phaseout ($80k–$90k/$160k–$180k) with foreign add-back, MFS refusal, one credit
//   per student, filer/dependent student check, 40% refundable AOTC with under-24 restriction,
//   ordering before 8812, 2025 recapture (Schedule 2). Expenses entered from records (tuition,
//   materials, scholarships, refunds, 529), not 1098-T boxes; AOTC eligibility (4 years, half-time,
//   drug conviction) is a confirmed Answer. Unit B11 KIDS (2026-10-11): the scholarship-income
//   election is calculated (ScholarshipInclusion on the Education page: none, Automatic within
//   the living costs, or an amount; the credit's assistance is box 5 less the included part,
//   a filer student's part joins Schedule 1 line 8r, a dependent student's part is a note to
//   the student's own return, and Automatic is resolved by running the whole estimate for each
//   candidate and keeping the lowest tax less refundable credits: resolved_scholarship_choices;
//   src/tests/family/kids.rs f1_*, f2_*, f2c_*). Unit B1 Form 8815 (2026-10-09): FederalCredits::education_expenses_used is the one
//   definition of the expenses a credit takes into account ($4,000 per American opportunity entry,
//   the return's $10,000 for lifetime learning in list order, none when no credit is allowed); the
//   credits are figured from it, and the savings bond exclusion reads it so tuition never earns both
//   benefits (credit first; savings_bonds.rs). Unit B3 Form 1099-Q (2026-10-09): the 529 and Coverdell
//   distributions read the same helper (education_account.rs), a credit phased out to zero still
//   consumes its expenses (NOTE_EDUCATION_ACCOUNT_CREDIT_EXPENSES), and the expenses the tax-free
//   earnings used replace the entry's own "paid with tax-free 529 or Coverdell distributions" amount
//   in Form 8815 line 3 for a person with a Form 1099-Q. Tests: credits.rs
//   education_expenses_used_follows_*, savings_bond_tuition_*, education_accounts.rs
//   form_8815_counts_the_expenses_the_529_exclusion_used_once.
//   Identification requirement (2026 i8863 "Identification requirement" and "Who cannot claim a
//   credit" item 5; units A18 and family credits F8, 2026-10-10): the credit is allowed only when
//   you, or on a joint return either spouse, have a valid SSN (education_allowed; the education
//   account and savings bond coordination read the same answer through education_expenses_used),
//   and when the student is a filer, or a dependent student with a valid SSN. A spouse-student
//   without an SSN keeps the credit when the other spouse has one (the i8863 Tip); the dependent
//   student's SSN answer is carried into the student catalog from the person's own page, and a
//   dependent without one takes none. Tests: line_audit_family_credits.rs
//   d10_a_spouse_student_without_an_ssn_keeps_the_credit_when_the_other_spouse_has_one,
//   d11_the_filers_own_ssn_requirement_applies_to_the_education_credit.
//   Evidence: federal_rules.rs, income_tax.rs, education_recapture.rs, federal_model.rs
//   FederalEducation/FederalEducationExpenses; tests:
//   education_expenses_derive_the_selected_credit_base,
//   education_credits_and_claimant_refund_restriction,
//   prior_education_credit_refund_recapture_reaches_2026_tax_and_survives_restart (family.rs),
//   lifetime_learning_is_capped_per_return (general.rs); i8863, Pub 970 ch. 2–3
//   C-fixture (2026-09-25, src/tests/family/credits.rs): education_credits_return_matches_hand_computation (Parts I-III
//   for an AOTC student and a lifetime-learning filer by hand), education_credits_phase_out_at_mid_range_magi,
//   lifetime_learning_expenses_cap_at_10000, refundable_aotc_is_denied_under_the_kiddie_conditions.
// Coverage: Form 8880 — Supported (audited 2026-09-24).
//   Lines: Lines 1–12: per-person plan, IRA (from Adjustments) and own ABLE contributions;
//   2024–2027 distribution lookback with joint cross-spouse offsets and 414(w) withdrawals; $2,000
//   cap; age-18/student/dependent tests; 2026 AGI bands 50/20/10%; foreign-income add-back;
//   tax-after-preceding-credits limit; an undecided 2027 joint status is assumed joint, with a
//   note. Refused: spouse items on non-joint returns.
//   Evidence: federal_rules.rs (saver_rate, saver_potential, line saver_credit),
//   pension_contributions.rs, federal_model.rs (FederalSaver); src/tests/retirement/credits.rs
//   saver_2026_rate_boundaries_for_every_status,
//   saver_spousal_distribution_exception_and_ineligible_recipient; src/tests/retirement/family.rs
//   saver_follows_education_care_and_precedes_child_credit; source f8880.pdf, Notice 2025-67 for
//   2026 bands.
//   T2-fx (2026-09-27): line 1 keeps a retained excess IRA contribution. f8880 (2025) line 1 is
//   "Traditional and Roth IRA contributions ... Do not include rollover contributions" and its
//   instructions (and Pub 590-A chapter 3 "Eligible contributions") exclude only rollovers; the
//   excess appears only in line 4's list of distributions not counted ("Distributions of excess
//   contributions or deferrals"). IraResult.contributions (traditional + Roth) is therefore
//   right and unchanged; line 6's $2,000 cap bounds it. Test: src/tests/retirement/retirement.rs
//   form_8880_line_1_keeps_a_retained_roth_excess. Source f8880.pdf (2025) fetched 2026-09-27.
// Coverage: Form 8960 — Supported (T2-28, 2026-09-26; audited 2026-09-24).
//   Lines: Lines 1–5c (interest, dividends, nonqualified annuities, rental/royalty/K-1 with 4b
//   nonpassive adjustment, gains with 5b/5c ordinary-property and excluded-loss carry difference),
//   7 (1041 K-1 box 14H), 8, 9a state tax source-allocated after SALT cap, 9b investment interest,
//   691(c) IRD, aggregate section 68 cap, MAGI with 2555 add-back, 3.8% tax. Refused: unresolved
//   state-tax allocation, missing prior excluded-loss difference. Refused explicitly (unit A4): line 6
//   (CFC/PFIC) and line 10 additional modifications through the ForeignCorporation situation
//   (M1-P3; formerly the screening answer net_investment_modifications).
//   Evidence: federal_rules.rs (net_investment_modified_agi, net_investment_income_tax), capital.rs
//   (niit_adjustment), section1231.rs (property_niit), personal_taxes.rs, extended_rules.rs
//   (prior_niit_difference); src/tests/federal/taxes.rs
//   niit_section_68_uses_aggregate_cap_instead_of_proportional_tax_reduction,
//   niit_income_tax_allocation_uses_current_income_before_expenses; src/tests/activity/investment.rs
//   rental_profit_flows_to_schedule_e_agi_niit_and_qbi_once,
//   property_niit_excluded_capital_losses_keep_carryforward_difference; sources i8960, irb/2013-51.
//   Unit T2-28 (2026-09-26): every individual line is printed under Form 8960 (form_8960_lines),
//   whenever modified AGI exceeds the threshold: 1, 2, 3; 4a = Schedule 1 lines 3+5+6 with 4b
//   reversing nonpassive business and K-1 amounts; 5a = Form 1040 line 7 plus Schedule 1 line 4,
//   5b the excluded property/NUA/carry difference; 7 = Schedule 1 line 8 investment items
//   (personal-property rent, hobby sales), the 1041 box 14H amount and Form 8814 line 12 less its
//   Alaska Permanent Fund share (child_election.rs, i8960 "Form 8814 election"); 9a investment interest,
//   9b allocated state income tax plus investment real-property tax, both spread with the 691(c)
//   deduction over the aggregate cap in proportion (worksheet Part IV); 9c zero by law (P.L.
//   119-21 section 70110); 10 early-withdrawal penalty plus 691(c); 11-17. Calculated: the
//   real-estate professional's Reg. 1.1411-4(g)(7) 500-hour safe harbor (RealEstateProfessional
//   .niit_safe_harbor, asked on the Rentals item) moves the professional's rental income or loss
//   to line 4b (RentalResult.rental_niit_adjustment); without it the rentals stay in line 4c.
//   Fixed: a qualifying surviving spouse's threshold is $250,000 (section 1411(b)), not Form
//   8959's $200,000. Refused by name: line 5c (it adjusts only a nonpassive owner's sale, and
//   that owner's S-corporation disposition above the threshold is refused by name in scorp.rs
//   (T2-31); line 5c prints zero), line 6 CFC/PFIC and
//   Reg. 1.1411-10(g) elections and line 10 traders' worksheet (ForeignCorporation situation),
//   line 7 section 1411 NOL (zero: the earlier-year loss kind's Business source excludes
//   investment and trading components, Reg. 1.1411-4(h); u5-losses 2026-09-28),
//   casualty losses (Casualty), recoveries of a prior 9b deduction (tax_refund.rs ordinary
//   answer), unresolved state-tax allocation; Part III estates and trusts is not an individual
//   return. STORAGE_FORMAT 44. Tests: src/tests/federal/taxes.rs form_8960_hand_worked_return_prints_every_line,
//   form_8960_threshold_boundary_for_every_status,
//   form_8960_line_16_takes_the_smaller_of_income_and_excess,
//   form_8960_real_estate_professional_safe_harbor_moves_rentals_to_line_4b; src/tests/cross/interview.rs
//   professional_rental_hours_answer_is_the_investment_tax_safe_harbor.
// Coverage: Form 8959 — Supported (audited 2026-09-24).
//   Lines: Parts I, III, IV, V (2026 numbering): 0.9% on W-2 box 5 wages plus unreported/allocated tips and SE net earnings above $200k/$250k/$125k by status; withholding reconciliation (box 6 plus
//   uncollected Medicare minus 1.45% of box 5; lines 19-22 whole dollars, 21-22 from the printed 19-20,
//   D-2026-10-19-12) credited on line 25c; feeds three-child ACTC
//   alternative. Form 8919 wages (Wages page, employee_8919_* tests), RRTA Part II
//   separate threshold and actual box 14a withholding (src/tests/income/wage_specials.rs D).
//   Evidence: federal_rules.rs (additional_medicare, medicare_withheld, 13237–13242),
//   employee_tips.rs; src/tests/health/health_accounts.rs additional_medicare_and_niit_are_not_silently_zero,
//   employee_tip_additional_medicare_does_not_invent_withholding,
//   employee_8919_additional_medicare_preserves_actual_withholding,
//   uncollected_medicare_reconciliation_preserves_real_additional_withholding; source i8959.
// Coverage: Form 6251 — Partial (audited 2026-09-24).
//   Lines: Lines 2b (refund), 2c (investment interest), 2f (ATNOL), 2g (private-activity), 2h, 2i
//   (ISO), 2j (1041 K-1 A–F), 2k (dispositions), 2l (depreciation), 2m (passive/rental), Part III
//   gains, exemption, AMT FTC via 1116, 8978/1341 ordering. Refused via the AlternativeMinimumTax
//   situation (M1-P3; formerly the special_taxes flag): other
//   preferences (depletion, IDC, long-term contracts, mining, R&E, circulation, tax-shelter farm);
//   K-1 adjustments with 4952 election or 2555.
//   Evidence: M/federal_rules.rs, extended_rules.rs, depreciation.rs, atnol.rs, qsbs.rs,
//   scorp_amt.rs; T/business.rs iso_spread_creates_amt_without_regular_income_and_persists;
//   line 2i entered through the Form 3921 kind on Investment sales (T2-iso: held shares only,
//   same-year dispositions excluded, option cost prorated; T/interview.rs form_3921_*);
//   T/passthrough.rs bba_audit_credit_reduces_form_6251_comparison_before_amt; T/health_accounts.rs
//   minimum_tax_prior_gain_pools_are_clipped_to_exclusion_taxable_income; IRS i6251, draft f6251.
//   Unit B2 (2026-10-09): line 10 subtracts any Form 4972 tax from Form 1040 line 16 (draft f6251
//   line 10): the comparison stays regular tax + Form 8978 + Schedule 2 Part I repayments and
//   never adds lump_sum_tax. T/lump_sum.rs lump_sum_tax_stays_out_of_the_alternative_minimum_tax_comparison.
//   PRINT (2026-10-13): income_tax::AmtTaxWorksheet retains its actual PartIII
//   12–40 components and foreign stacking worksheet. Rate cents round once
//   at line40 before the existing foreign subtraction; skipped lines stay
//   absent. src/tests/federal/amt_projection.rs covers all gain bands,1250,rounding,
//   foreign excess and an actual return/restart; supported() checks footings.
//   This adds no tax input or figure change. Full PartI classification and
//   filing conditions beyond positive AMT remain separate PRINT obligations.
// Coverage: Form 8396 — Supported (audited 2026-09-24).
//   Lines: Line 1 certified-interest allocation, line 3 (rate, $2,000 high-rate cap, reissue
//   split-period and original-schedule ceiling), line 9 limit, lines 14/16/17 carryforwards with
//   2023 expiry, itemized-interest reduction. Refused: multiple current certificates, unreviewed
//   reissue facts. Form 8828 recapture is separate.
//   Evidence: federal_rules.rs, federal_model.rs;
//   credits.rs::mortgage_certificate_credit_reduces_itemized_interest_before_tax,
//   health_accounts.rs::mortgage_credit_precedes_homeowner_carries_and_preserves_refundable_child_credit,
//   family.rs::clean_vehicle_credit_precedes_mortgage_and_child_credits; draft f8396
// Coverage: Form 8958 — see community.rs (unit B9-3, 2026-10-10): every income kind that enters the
//   return totals is divided by its Community or separate answer; rentals, farm rentals and
//   pass-through K-1s enter whole as the filer's own, or are refused by name when shared.
// Coverage: Form 2439 — Supported (audited 2026-09-24; page B7a 2026-10-09).
//   Lines: Interest and dividends kind Form2439 (boxes 1a-1d and 2 imported, the fund the label)
//   writes FederalIncome::UndistributedGain. Box 1a is Schedule D line 11 and long-term gain;
//   box 1b joins the unrecaptured section 1250 worksheet and box 1d the 28% rate gain worksheet;
//   box 2 tax paid is a refundable credit on Schedule 3 line 13a; the share-basis increase
//   (box 1a less box 2) is reported. Unit B9-5 (2026-10-10): box 1c section 1202 gain (and 1099-DIV
//   box 2c) is calculated with one FundQsbsExclusion choice: the whole gain stays on its line and
//   a Form 8949 Part II box F code Q row excludes the chosen percentage (qsbs.rs fund_gain: 50 or
//   75 percent leave the rest in the 28 percent pool with the 7 percent AMT preference). Left out
//   with a notice: IRA, nominee, pass-through ownership or a non-2026 fund year
//   (WARN_UNDISTRIBUTED_GAIN_LEFT_OUT).
//   Evidence: federal_rules.rs, federal_model.rs;
//   investment_kinds.rs::form_2439_reaches_schedule_d_line_11_and_schedule_3_line_13a,
//   passthrough.rs::undistributed_fund_gain_reports_income_credit_and_share_basis_once,
//   health_accounts.rs::undistributed_fund_credit_is_refundable_without_regular_tax,
//   general.rs::undistributed_fund_special_components_and_invalid_statements_are_explicit; f2439,
//   Pub. 550
// Coverage: Form 8911 — Partial (audited 2026-09-24).
//   Lines: Personal-use Schedule A: 30% per unit capped at $1,000, eligible census tract, placed in
//   service Jan 1–Jun 30 2026, line 10 tax-limit after earlier personal credits and tentative
//   minimum tax, non-carryable unused credit. Refused: business/investment use (Part II / Form
//   3800), combination with Form 1062 election.
//   Evidence: federal_rules.rs, federal_model.rs;
//   credits.rs::personal_refueling_credit_caps_each_unit_and_reduces_schedule_three,
//   credits.rs::personal_refueling_unused_credit_expires_and_does_not_displace_earlier_credits,
//   credits.rs::personal_refueling_credit_respects_tentative_minimum_tax_and_defers_form_1062_overlap;
//   i8911
// Coverage: Form 8834 — Supported (audited 2026-09-24).
//   Lines: Line 1 is the entered 2026 Form 8582-CR release (the electric vehicle credit's own
//   8582-CR release is not computed; the Form 8582-CR of the Form 3800 passive credits is, in
//   general_business_credit.rs); lines
//   2–7 regular-tax and tentative-minimum-tax limit after listed credits; unusable credit expires.
//   Refused: same return with a Form 8936 clean-vehicle credit (ordering unresolved), Form 1062
//   election overlap.
//   Evidence: federal_rules.rs, federal_model.rs;
//   credits.rs::legacy_ev_passive_credit_uses_form_8834_regular_tax_limit_and_expires,
//   family.rs::legacy_ev_credit_follows_child_but_precedes_refueling_and_minimum_tax_credit; f8834
// Coverage: Form 8859 — Supported (audited 2026-09-24).
//   Lines: 2026 form is carryforward-only: lines 1–3 prior-carry credit limited by tax after
//   preceding homeowner credits and child-credit reserve, line 4 carryforward to 2027. Refused:
//   Form 1062 election combination.
//   Evidence: federal_rules.rs, federal_model.rs;
//   interview.rs::dc_homebuyer_carryforward_uses_2026_form_8859_limit_and_survives_restart,
//   family.rs::dc_homebuyer_credit_follows_child_credit_and_defers_form_1062_combination,
//   repayment.rs::dc_homebuyer_credit_precedes_minimum_tax_credit_without_double_claim; draft f8859

// 2026 estimate rules. Current-year ordinary-tax lookups use the Sep. 16 draft
// IRS Tax Table below $100,000; integer cents round at tax-line boundaries.
// Annual parameters: IRS Rev. Proc. 2025-32, https://www.irs.gov/irb/2025-45_IRB
// Estimated tax method: https://www.irs.gov/publications/p505
// Enacted 2026 changes: https://www.govinfo.gov/content/pkg/PLAW-119publ21/html/PLAW-119publ21.htm
// Special worksheets and EIC remain estimates pending final 2026 instructions.
use crate::federal_model::ReviewNote;
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact};
use crate::line_emission::emit as emit_lines;
use crate::*;
use alloc::{string::String, vec, vec::Vec};
use core::cmp::min;

const ANNUAL: &str = "https://www.irs.gov/irb/2025-45_IRB";
const TAX_TABLE: &str = "https://www.irs.gov/pub/irs-dft/p1040--dft.pdf";
pub(crate) const LAW: &str =
    "https://www.govinfo.gov/content/pkg/PLAW-119publ21/html/PLAW-119publ21.htm";
use crate::validation::MAX;

use crate::kernel::ratio::{dollars, fraction, positive};

#[cfg(test)]
pub(crate) use crate::issue_helpers::issue_localized;
pub(crate) use crate::issue_helpers::{
    issue_message, note_once, note_once_scoped, require_localized, require_message,
};

/// Business credits that aggregate all of one owner's businesses as one employer
/// (Forms 8941 and 8994 and the next owner-wide credit): the credit may sit on
/// one of the owner's Schedule C businesses only.
fn require_one_per_owner(
    result: &mut FederalResult,
    d: &FederalDraft,
    owner: Owner,
    context: &crate::IssueContext,
    #[cfg(test)] section: &str,
    holds_credit: impl Fn(&crate::FederalBusiness) -> bool,
) {
    crate::require_localized!(
        result,
        d.income
            .iter()
            .filter(|other| {
                other.owner == crate::DocumentOwner::from(owner)
                    && matches!(&other.income, FederalIncome::Business { data } if holds_credit(data))
            })
            .count()
            == 1,
        &context.clone().manual(),
        section,
        "CREDIT_ONE_BUSINESS_PER_OWNER",
        "All of one owner's businesses are one employer; enter the credit on one business only.",
    );
}

// Answers and unknowns: an answer the App cannot use never
// refuses the estimate. A kept Not sure takes the conservative path and
// states its assumption; a scope confirmation answered No leaves its item out
// with a notice in the item's own words. Each note appears once.

fn note_number(
    #[cfg(test)] english: impl Into<alloc::string::String>,
    key: &str,
    value: i64,
) -> ReviewNote {
    ReviewNote::localized(
        #[cfg(test)]
        english,
        crate::Message::new(key, alloc::vec![crate::MessageArg::Number(value)]),
    )
}

// Refused situations retain their names and affected form lines. A supported
// situation leaves this table rather than remaining an unsupported choice.
impl RareSituation {
    // Every situation, in declaration (Avro index) order.
    #[cfg(test)]
    pub(crate) const ALL: [RareSituation; 10] = [
        Self::RailroadRepresentative,
        Self::RemicResidual,
        Self::Straddles,
        Self::SpecialDepreciation,
        Self::AssetDisposed,
        Self::LikeKindExchange,
        Self::TaxTreaty,
        Self::Chapter11Debtor,
        Self::ForeignCorporation,
        Self::Casualty,
    ];

    fn refusal_info(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::RemicResidual => (
                crate::fixture_english!(
                    "REMIC residual interest (Schedule Q; Schedule E lines 38 and 39)"
                ),
                "ISSUE_SITUATION_REMIC_RESIDUAL",
                "SITUATION_NAME_REMIC_RESIDUAL",
            ),
            Self::RailroadRepresentative => (
                crate::fixture_english!(
                    "Railroad employee representative compensation and taxes (Form CT-2; Form 8959 line 8; Schedule 8812 lines 12–15)"
                ),
                "ISSUE_SITUATION_RAILROAD_REPRESENTATIVE",
                "SITUATION_NAME_RAILROAD_REPRESENTATIVE",
            ),
            Self::Straddles => (
                crate::fixture_english!(
                    "Straddles and offsetting positions (Form 6781 Part II, boxes A–C, Schedule D lines 4 and 11)"
                ),
                "ISSUE_SITUATION_STRADDLES",
                "SITUATION_NAME_STRADDLES",
            ),
            Self::SpecialDepreciation => (
                crate::fixture_english!(
                    "Form 4562 lines 14b, 14c, 14d, 19g, 19h and 19k (specified plants, reuse and recycling, qualified production, water utility and railroad property)"
                ),
                "ISSUE_SITUATION_SPECIAL_DEPRECIATION",
                "SITUATION_NAME_SPECIAL_DEPRECIATION",
            ),
            Self::AssetDisposed => (
                crate::fixture_english!(
                    "Sales of purchased goodwill or another section 197 intangible, and of section 1254 or 1255 property (Form 4797)"
                ),
                "ISSUE_SITUATION_ASSET_DISPOSED",
                "SITUATION_NAME_ASSET_DISPOSED",
            ),
            Self::LikeKindExchange => (
                crate::fixture_english!(
                    "Like-kind exchanges of business, farm or investment real property that is not a rental, and conflict-of-interest sales (Form 8824)"
                ),
                "ISSUE_SITUATION_LIKE_KIND_EXCHANGE",
                "SITUATION_NAME_LIKE_KIND_EXCHANGE",
            ),
            Self::TaxTreaty => (
                crate::fixture_english!("Tax treaty positions (Form 8833)"),
                "ISSUE_SITUATION_TAX_TREATY",
                "SITUATION_NAME_TAX_TREATY",
            ),
            Self::Chapter11Debtor => (
                crate::fixture_english!("Chapter 11 debtor with self-employment income"),
                "ISSUE_SITUATION_CHAPTER_11_DEBTOR",
                "SITUATION_NAME_CHAPTER_11_DEBTOR",
            ),
            Self::ForeignCorporation => (
                crate::fixture_english!(
                    "Schedule 1 lines 8n and 8o (Forms 5471 and 8992), Form 8960 line 6 (controlled foreign corporation and PFIC adjustments) and line 10 (additional modifications)"
                ),
                "ISSUE_SITUATION_FOREIGN_CORPORATION",
                "SITUATION_NAME_FOREIGN_CORPORATION",
            ),
            Self::Casualty => (
                crate::fixture_english!(
                    "Casualty and theft losses of business, rental or income-producing property and Ponzi-type or other investment theft losses (Form 4684 Sections B, C and D, Schedule A line 17d)"
                ),
                "ISSUE_SITUATION_CASUALTY",
                "SITUATION_NAME_CASUALTY",
            ),
        }
    }
}

// A chosen item the estimate does not calculate: the form lines refused, the
// issue that refuses it by name and the name the review gives it. Every item
// is a variant of Unprepared, the one owner: RareSituation, the payload-less
// kinds and RareAdditionalTax each declare theirs once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Refusal {
    #[cfg(test)]
    pub(crate) topic: &'static str,
    #[cfg(test)]
    pub(crate) lines: &'static str,
    pub(crate) issue_key: &'static str,
    pub(crate) name_key: &'static str,
}

impl Refusal {
    #[cfg_attr(not(test), allow(unused_variables))]
    fn new(
        (lines, issue_key, name_key): (&'static str, &'static str, &'static str),
        topic: &'static str,
    ) -> Self {
        Self {
            #[cfg(test)]
            topic,
            #[cfg(test)]
            lines,
            issue_key,
            name_key,
        }
    }
}

impl crate::Unprepared {
    pub(crate) fn refused(&self) -> Refusal {
        let mut topic = "";
        let info = match self {
            Self::Situation { situation } => situation.refusal_info(),
            Self::OtherAdjustments => (
                crate::fixture_english!(
                    "Other adjustments to income, including section 501(c)(18)(D) pension contributions (Schedule 1 lines 24a–24z)"
                ),
                "ISSUE_SITUATION_OTHER_ADJUSTMENTS",
                "SITUATION_NAME_OTHER_ADJUSTMENTS",
            ),
            Self::AlternativeMinimumTax => (
                crate::fixture_english!(
                    "Alternative minimum tax adjustments and preferences (Form 6251)"
                ),
                "ISSUE_SITUATION_ALTERNATIVE_MINIMUM_TAX",
                "SITUATION_NAME_ALTERNATIVE_MINIMUM_TAX",
            ),
            Self::HousingCreditRecapture => (
                crate::fixture_english!(
                    "Schedule 2 line 9 (Form 8611 recapture) and line 13n (Forms 8697/8866 look-back interest)"
                ),
                "ISSUE_SITUATION_HOUSING_CREDIT_RECAPTURE",
                "SITUATION_NAME_HOUSING_CREDIT_RECAPTURE",
            ),
            Self::Section965 => (
                crate::fixture_english!("Schedule 3 line 13d (deferred section 965 tax liability)"),
                "ISSUE_SITUATION_SECTION965",
                "SITUATION_NAME_SECTION965",
            ),
            Self::OtherAdditionalTaxes { kind } => {
                let (name, _, lines, issue_key, name_key) = kind.refusal_info();
                topic = name;
                (lines, issue_key, name_key)
            }
        };
        Refusal::new(info, topic)
    }
    // Every refused kind, built from the definitions of its owners: the
    // coverage test and the page's choices cannot disagree.
    #[cfg(test)]
    pub(crate) fn all() -> Vec<Self> {
        crate::RareSituation::ALL
            .into_iter()
            .map(|situation| Self::Situation { situation })
            .chain([
                Self::OtherAdjustments,
                Self::AlternativeMinimumTax,
                Self::HousingCreditRecapture,
                Self::Section965,
            ])
            .chain(
                crate::RareAdditionalTax::ALL
                    .into_iter()
                    .map(|kind| Self::OtherAdditionalTaxes { kind }),
            )
            .collect()
    }
}

impl FederalDraft {
    // Every chosen item the estimate refuses, in the order chosen.
    pub(crate) fn refusals(&self) -> impl Iterator<Item = Refusal> + '_ {
        self.often_missed.unprepared.iter().map(crate::Unprepared::refused)
    }
}

// Refuse every chosen item by name, under the page it is chosen on.
pub(crate) fn refuse_situations(d: &FederalDraft, r: &mut FederalResult) {
    for refusal in d.refusals() {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::OftenMissed),
            crate::interview::section_name(crate::TaxSection::OftenMissed),
            refusal.issue_key,
            &alloc::format!("{} is not calculated yet.", refusal.lines),
        );
    }
}

// The whole-return situations of the filers' pages: the two returns outside
// the App's scope, with the issue and the name the review gives them (a
// refusal of the whole return, decision rule "Answers and unknowns").
pub(crate) const WHOLE_RETURN_REFUSALS: [(u32, &str, &str); 2] = [
    (
        crate::YourSituations::DUAL_STATUS.0,
        "ISSUE_SITUATION_DUAL_STATUS",
        "SITUATION_NAME_DUAL_STATUS",
    ),
    (
        crate::YourSituations::COMMUNITY_PROPERTY.0,
        "ISSUE_SITUATION_COMMUNITY_PROPERTY",
        "SITUATION_NAME_COMMUNITY_PROPERTY",
    ),
];
// The whole-return answers on the estimate. A dual-status or territory
// return and community property with a registered domestic partner are
// refused by name on About you. The nonresident spouse choice (section
// 6013(g) or (h)) is an ordinary joint return of both spouses' worldwide
// income with an attached statement (Pub. 519), and innocent spouse relief is
// a separate Form 8857 request that leaves this joint return's tax unchanged:
// each is a notice from the spouse's page. The injured spouse item opens Form
// 8379's page, which never changes the joint return's tax either.
pub(crate) fn whole_return(d: &FederalDraft, r: &mut FederalResult) {
    let ticks = d.whole_return();
    let you = crate::IssueContext::Section(crate::TaxSection::People);
    for (bit, key, _) in WHOLE_RETURN_REFUSALS {
        if ticks & bit != 0 {
            let message = crate::Message::new(key, alloc::vec![]);
            issue_message(
                r,
                &you,
                #[cfg(test)]
                crate::interview::section_name(crate::TaxSection::People),
                #[cfg(test)]
                message.english(),
                message,
            );
        }
    }
    let spouse = crate::IssueContext::Section(crate::TaxSection::Spouse);
    for (bit, key) in [
        (
            crate::SpouseSituations::NONRESIDENT_SPOUSE.0,
            "NOTE_NONRESIDENT_SPOUSE_TREATED_AS_A_RESIDENT",
        ),
        (
            crate::SpouseSituations::INNOCENT_SPOUSE.0,
            "NOTE_INNOCENT_SPOUSE_RELIEF_IS_A_SEPARATE_REQUEST",
        ),
        (crate::SpouseSituations::NIIT_JOINT_ELECTION.0, "NOTE_NIIT_NONRESIDENT_SPOUSE_ELECTED"),
    ] {
        if ticks & bit != 0 {
            note_once_scoped(r, crate::Message::new(key, alloc::vec![]), spouse.clone());
        }
    }
}
pub(crate) use crate::dates::{source_valid_date as valid_date, valid_birth, valid_service_date};
pub(crate) use crate::form_lines::printed_line;
use crate::line_emission::{form_line, line, line_cents, push_line};

pub(crate) fn joint(s: FederalStatus) -> bool {
    s == FederalStatus::Joint
}

pub(crate) fn brackets(s: FederalStatus) -> [i64; 6] {
    crate::income_tax::Rates::for_year(2026, s).unwrap().ordinary
}

impl FederalDraft {
    pub(crate) fn first_incomplete(&self) -> i32 {
        if self.setup.is_none() {
            0
        } else if self.people.is_none() {
            1
        } else if !self.finished_sections.contains(&crate::TaxSection::Income) {
            3
        } else if self.adjustments.is_none() {
            4
        } else if self.deductions.is_none() {
            5
        } else if self.credits.is_none() {
            6
        } else if self.screening.is_none() {
            7
        } else {
            9
        }
    }
    // The whole-return situations both filers' pages answer (YourSituations
    // and SpouseSituations); the spouse's only while the return has one.
    pub(crate) fn whole_return(&self) -> u32 {
        self.people.as_ref().map_or(0, |p| {
            p.taxpayer.whole_return | p.spouse.as_ref().map_or(0, |s| s.whole_return)
        })
    }
    // The people you support.
    pub(crate) fn dependents(&self) -> &[FederalDependent] {
        &self.dependents
    }
    // Whether a separately filing spouse itemizes, which forces itemizing
    // (the deductions page's route).
    pub(crate) fn deduction_choice(&self) -> DeductionChoice {
        if self.spouse_itemizes() {
            DeductionChoice::Itemized
        } else {
            self.deductions.as_ref().map_or(DeductionChoice::Automatic, |ded| ded.choice())
        }
    }
    pub(crate) fn spouse_itemizes(&self) -> bool {
        self.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Separate)
            && self.deductions.as_ref().is_some_and(|d| d.spouse_itemizes == Some(true))
    }
    pub(crate) fn valid_stored(&self) -> bool {
        self.stored_issue().is_none()
    }
    pub(crate) fn stored_issue(&self) -> Option<crate::validation::ValidationIssue> {
        for entry in &self.income {
            if let FederalIncome::Scorp { data } = &entry.income
                && let Some(issue) = data.admission_issue()
            {
                return Some(issue);
            }
            if let FederalIncome::EstateTrustPortfolio { data } = &entry.income
                && let Some(issue) = data.admission_issue()
            {
                return Some(issue);
            }
        }
        if let Some((_, issue)) = crate::related_forms::source_issue(self) {
            return Some(issue);
        }
        if let Some((_, issue)) = crate::employee_wages_admission::return_issue(self) {
            return Some(issue);
        }
        if let Some((id, issue)) = crate::savings_bonds::redemption_link_issue(&self.income) {
            let index = self.income.iter().position(|entry| entry.id == id).unwrap();
            return Some(issue.in_list("income", index));
        }
        if let Some((_, issue)) = crate::cohorts::source_issue(self) {
            return Some(issue);
        }
        if let Some(issue) = crate::amortization::timber_return_issue(self) {
            return Some(issue);
        }
        if let Some(issue) = crate::home_sites::graph_issue(self) {
            return Some(issue);
        }
        let mut issue = None;
        if self.valid_stored_with_issue(&mut issue) {
            None
        } else {
            Some(issue.unwrap_or_else(|| {
                crate::validation::ValidationIssue::new(
                    "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7",
                )
            }))
        }
    }
    fn valid_stored_with_issue(
        &self,
        issue: &mut Option<crate::validation::ValidationIssue>,
    ) -> bool {
        if crate::retained_research::rule(self).is_some() {
            return false;
        }
        if crate::farm_specials::external_source_rule(self).is_some()
            || crate::medical_rollover_return::correction_reference_rule(self).is_some()
            || crate::hsa_funding::source_reference_rule(self).is_some()
            || crate::injured_spouse_sources::reference_rule(self).is_some()
            || crate::qbi_aggregation::accepted_rule(self).is_some()
        {
            return false;
        }
        if self.tax_year != 2026
            || self.cookie.is_empty()
            || self.revision < 0
            // Accepted revision must have a representable next i64 revision.
            || self.revision == i64::MAX
            || self.next_id < 1
            // The managed i64 allocator must have a representable successor.
            || self.next_id == i64::MAX
            || self.screening.as_ref().is_some_and(|screening| {
                crate::foreign_accounts::rule(
                    screening.foreign_account,
                    screening.fbar_required,
                    &screening.fbar_countries,
                )
                .is_some()
            })
            || crate::coverdell::accepted_rule(&self.coverdell_excess, self).is_some()
            // Each situation is chosen once.
            || self
                .often_missed
                .unprepared
                .iter()
                .enumerate()
                .any(|(i, item)| self.often_missed.unprepared[..i].contains(item))
            // Notice kinds remain unique; their set has no independent count cap.
            || self.credit_refusals.iter().enumerate().any(|(index, kind)| {
                self.credit_refusals[..index].contains(kind)
            })
        {
            return false;
        }
        let ids: Vec<i64> = self
            .income
            .iter()
            .map(|v| v.id)
            .chain(self.dependents().iter().map(|v| v.id))
            .chain(self.income.iter().flat_map(|entry| {
                crate::macrs_records::improvement_buildings(&entry.income).iter().map(|row| row.id)
            }))
            .chain(
                self.adjustments
                    .iter()
                    .flat_map(|a| &a.employee_expenses)
                    .flat_map(|claim| claim.improvement_buildings.iter().map(|row| row.id)),
            )
            .chain(
                self.income
                    .iter()
                    .flat_map(|entry| crate::rental_energy_records::ids(&entry.income))
                    .filter(|id| *id != 0),
            )
            .chain(self.deductions.iter().flat_map(|d| d.appraisals.iter().map(|a| a.id)))
            .chain(
                self.income
                    .iter()
                    .flat_map(|v| crate::investment_statements::annual_ids(&v.income)),
            )
            .chain(self.income.iter().flat_map(|e| {
                match &e.income {
                    FederalIncome::EstateTrustPortfolio { data } => data
                        .statements
                        .activities
                        .iter()
                        .map(|a| a.id)
                        .chain(
                            data.statements
                                .qbi
                                .statements
                                .iter()
                                .map(|row| row.id)
                                .filter(|id| *id != 0),
                        )
                        .chain(
                            data.statements
                                .amt_reconciliation
                                .iter()
                                .flat_map(|amt| amt.adjustments.iter().map(|row| row.id))
                                .filter(|id| *id != 0),
                        )
                        .collect::<Vec<_>>(),
                    _ => Vec::new(),
                }
            }))
            .chain(
                self.credits
                    .as_ref()
                    .and_then(|c| crate::marketplace::facts(&c.marketplace))
                    .into_iter()
                    .flat_map(|f| {
                        f.forms
                            .iter()
                            .map(|r| r.id)
                            .chain(f.other_families.iter().map(|r| r.id))
                            .chain(f.covered.iter().map(|r| r.id))
                            .chain(f.qsehra.iter().map(|r| r.id))
                    }),
            )
            .chain(
                self.income
                    .iter()
                    .flat_map(|entry| crate::cooperative::other_ids(&entry.income))
                    .filter(|id| *id != 0),
            )
            // Accepted property identities share the normal record counter.
            // Zero belongs only to a transient unsaved source.
            .chain(crate::depreciation::assets(self).map(|asset| asset.id).filter(|id| *id != 0))
            .chain(
                self.income
                    .iter()
                    .flat_map(|entry| {
                        crate::BusinessMileage::records(&entry.income).iter().map(|row| row.id)
                    })
                    .filter(|id| *id != 0),
            )
            // A farm's CCC loans carry record identities from the same counter
            // (a loan no Form 1099-A can name yet has none).
            .chain(
                self.income
                    .iter()
                    .flat_map(|v| crate::farm::ccc_lot_ids(&v.income))
                    .filter(|id| *id != 0),
            )
            .chain(
                self.income.iter().flat_map(|entry| crate::farm_specials::land_ids(&entry.income)),
            )
            .chain(self.income.iter().flat_map(|entry| {
                crate::method_change::records(&entry.income)
                    .into_iter()
                    .flat_map(|records| records.iter().map(|record| record.id))
            }))
            .collect();
        if ids
            .iter()
            .enumerate()
            .any(|(i, &id)| id <= 0 || id >= self.next_id || ids[..i].contains(&id))
        {
            return false;
        }
        if crate::savings_bonds::source_rule(&self.income).is_some()
            || crate::estate_trust::graph_issue_with_mode(self, false).is_some()
            || crate::rental_energy::graph_issue(self).is_some()
        {
            return false;
        }
        for entry in &self.income {
            if matches!(entry.income, FederalIncome::InstallmentSale { .. })
                && ((!crate::interview::joint(self)
                    && !self.setup.as_ref().is_some_and(crate::community::applies)
                    && entry.owner == crate::DocumentOwner::Spouses)
                    || crate::injured_spouse::community_row_issue(self, entry.community).is_some())
            {
                return false;
            }
            if let Err((failure, _)) = validate_income_issue_for_year_with_mode(
                entry,
                u32::try_from(self.tax_year).unwrap_or(0),
                false,
            ) {
                // Preserve the actual native range path through this same gate;
                // other stored validation failures retain their existing fallback.
                if failure.key == crate::k1_activity::ordinary::RANGE_KEY && failure.field.is_some()
                {
                    *issue = Some(failure);
                }
                return false;
            }
        }
        if self.dependents().iter().any(|v| validate_dependent(v).is_err()) {
            return false;
        }
        // A filer's 529-to-Roth transfers (Form 1099-Q box 4b) with the
        // earlier use on the IRA page: account age, seasoned balance and the
        // lifetime ceiling (ira.rs).
        let transfers = crate::ira::QtpTransfers::of(&self.income);
        let earlier = self.adjustments.as_ref().map_or([0, 0], |a| {
            [
                a.iras.taxpayer.qtp_roth_lifetime_prior,
                a.iras.spouse.as_ref().map_or(0, |f| f.qtp_roth_lifetime_prior),
            ]
        });
        if (0..2).any(|i| !crate::ira::valid_qtp_rollovers(transfers.owner(i), earlier[i])) {
            return false;
        }
        for (index, entry) in self.income.iter().enumerate() {
            if let FederalIncome::CasualtyLivingInsurance { data } = &entry.income {
                for prior in &self.income[..index] {
                    if let FederalIncome::CasualtyLivingInsurance { data: other } = &prior.income
                        && (other.event_reference == data.event_reference
                            || data.payments.iter().any(|payment| {
                                other.payments.iter().any(|earlier| {
                                    earlier.insurer == payment.insurer
                                        && earlier.payment_reference == payment.payment_reference
                                })
                            }))
                    {
                        return false;
                    }
                }
            }
        }
        if self.setup.as_ref().is_some_and(|s| matches!(s.status, FilingChoice::Separate { section179, .. } if !section179.valid())) {
            return false;
        }
        if self.people.as_ref().is_some_and(|v| {
            v.parent_tax.as_ref().is_some_and(|p| !p.valid())
                || !v.taxpayer.named()
                || v.spouse.as_ref().is_some_and(|p| !p.named())
                || !v.taxpayer.valid_dates()
                || v.spouse.as_ref().is_some_and(|p| !p.valid_dates())
        }) {
            return false;
        }
        if self.screening.as_ref().is_some_and(|v| {
            v.farm_averaging.as_ref().is_some_and(|a| !a.valid() || !a.sources_valid(self))
                || !crate::household_employment::valid(&v.household_employment)
                || v.section1231.as_ref().is_some_and(|h| !h.valid())
                || !v.qbi_history.valid()
                || !v.atnol.valid()
                || !v.nol.valid()
                || v.foreign_earned.iter().enumerate().any(|(i, c)| {
                    !c.housing.valid()
                        || !c.revocation.valid()
                        || !c.pay_valid()
                        || v.foreign_earned[..i].iter().any(|p| p.owner == c.owner)
                })
                || v.clergy.iter().enumerate().any(|(i, c)| {
                    !c.role.valid() || v.clergy[..i].iter().any(|p| p.owner == c.owner)
                })
                || !crate::able::valid_contributions(&v.able_contributions)
                || !crate::trump_account::valid_accounts(&v.trump_accounts)
                || v.public_safety.iter().any(|p| !p.valid())
                || !crate::extended_rules::valid_exercise_sources(&v.iso_exercises)
                || v.espp_acquisitions
                    .iter()
                    .any(|row| crate::Form3922::from(&row.form).issue().is_some())
                || v.espp_sale_answers.iter().enumerate().any(|(i, row)| {
                    v.espp_sale_answers[..i]
                        .iter()
                        .any(|before| before.document_id == row.document_id)
                })
                || v.iso_sale_answers.iter().enumerate().any(|(i, row)| {
                    v.iso_sale_answers[..i]
                        .iter()
                        .any(|before| before.document_id == row.document_id)
                })
                || v.nua_sale_answers.iter().enumerate().any(|(i, row)| {
                    v.nua_sale_answers[..i]
                        .iter()
                        .any(|before| before.document_id == row.document_id)
                })
                || !crate::extended_rules::valid_capital_history(&v.capital_history)
        }) {
            return false;
        }
        if !crate::health::plans_valid(self) {
            return false;
        }
        // Restored data is checked with the same amount invariants as accepted pages.
        if self.adjustments.as_ref().is_some_and(|v| {
            !v.health_marketplace.valid(self)
                || crate::health_employer::binding_rule(self, &v.health_employer_w2).is_some()
                || crate::health_employer::derived_rule(self).is_some()
                || crate::contribution_documents::derived_rule(self).is_some()
                || v.pension_contributions.iter().any(|v| !v.valid())
                || !v.social_election.valid()
                || v.alimony_paid.iter().any(|p| !p.payment.valid())
                || v.moving.iter().any(|m| !m.record.valid())
                || !v.student_loan.forms.iter().all(|source| source.valid())
                || !amounts(&[
                    v.student_loan.student_loan_interest,
                    v.student_loan.employer_paid_interest,
                ])
                || !amounts(&[
                    v.educator_taxpayer(),
                    v.educator_spouse(),
                ])
                || v.educators.taxpayer.as_ref().is_some_and(|r| !r.valid())
                || v.educators.spouse.as_ref().is_some_and(|r| !r.valid())
                || v.employee_expenses.iter().any(|claim| !claim.valid())
                || v.entered_iras().is_some_and(|f| {
                    core::iter::once(&f.taxpayer)
                        .chain(f.spouse.iter()).enumerate()
                        .any(|(index, v)| {
                            !amounts(&[v.traditional, v.roth, v.nondeductible_election])
                                || v.nondeductible_election > v.traditional
                                || v.next_year_deposits.as_ref().is_some_and(|d| !d.valid())
                                || !(0..=crate::year_constants::QTP_ROTH_LIFETIME_LIMIT_CENTS)
                                    .contains(&v.qtp_roth_lifetime_prior)
                                || v.returned_next_year.iter().any(|r| !r.valid())
                                || v.excess.as_ref().is_some_and(|e| !e.valid(
                                    self.adjustments.as_ref().and_then(|a| a.ira_basis.iter().find(|b|
                                        usize::from(b.owner == Owner::Spouse) == index)).map(crate::IraBasis::form_5329_value_exact)))
                        })
                })
                || v.required_distributions.iter().any(|v| !v.valid())
                || v.roth_history.iter().any(|v| !v.valid())
                || v.ira_basis.iter().any(|b| !b.valid())
                // One QCD record per tax owner: taxpayer and spouse.
                || v.qualified_charity.len() > 2
                || v.qualified_charity.iter().any(|q| !q.valid())
                || !v.archer.valid()
                || v.hsa.iter().any(|h| !h.valid())
        }) || self.deductions.as_ref().is_some_and(|v| {
            !v.repayment.valid()
                || !v.valid_taxes()
                || crate::health::medical_expenses_issue(v.medical()).is_some()
                || crate::health::medical_source_amount(v.medical(), v.long_term_care()).is_err()
                || !amounts(&[
                    v.state_local_taxes(),
                    v.investment_property_taxes(),
                    v.cash_charity, v.fund_cash_charity, v.other_cash_charity,
                ])
                // Printed sources survive missing personal answers. The estimate
                // stops at the converter's named boundary until they are supplied.
                || v.mortgage_loans().is_ok_and(|loans| {
                    loans.iter().any(|loan| !loan.valid())
                        || crate::mortgage_limit::graph_rule(&loans).is_some()
                })
                || v.annuity_losses.iter().any(|loss| !loss.valid())
                || v.personal_casualties.iter().any(|event| !event.valid())
                || !crate::PersonalCasualty::sources_representable(&v.personal_casualties)
                || crate::charity::page_problem(&v.property_gifts,&v.vehicle_gifts,&v.vehicle_gift_documents,&v.appraisals,&v.charity_carryovers,v.basis_election).is_some()
                || v.volunteer_expenses.iter().any(|expense| !expense.valid())
                || v.hosted_students.iter().any(|student| !student.valid())
                || v.whaling_captains.iter().any(|claim| !claim.valid())
                || v.whaling_captains.iter().enumerate().any(|(i, claim)| {
                    v.whaling_captains[..i]
                        .iter()
                        .any(|prior| prior.owner == claim.owner)
                })
                || v.hosted_students.iter().enumerate().any(|(i, student)| {
                    v.hosted_students[..i]
                        .iter()
                        .any(|prior| prior.label == student.label)
                })
                || v.annuity_losses.iter().enumerate().any(|(i, loss)| {
                    v.annuity_losses[..i]
                        .iter()
                        .any(|prior| prior.income_id == loss.income_id)
                })
                || v.vehicle_loans.iter().any(|v| {
                    v.model().lender.name.trim().is_empty()
                        || !amounts(&[v.model().interest, v.business])
                })
        }) || self.credits.as_ref().is_some_and(|v| {
            !crate::extended_rules::valid_credit(&v.foreign_tax)
                || (!joint(self.setup.as_ref().map_or(FederalStatus::Single, |s| s.filing_status()))
                    && matches!(&v.foreign_tax, ForeignTaxClaim::Deduction { taxes } if taxes.iter().any(|row| matches!(row.income, crate::ForeignTaxIncome::Earned { owner: Owner::Spouse }))))
                || v.tip_fica_carryforward
                    .as_ref()
                    .is_some_and(|history| !history.valid())
                || v.business_credits
                    .as_ref()
                    .is_some_and(|history| !history.valid() || history.rule().is_some())
                || v.minimum_tax.as_ref().is_some_and(|v| !v.valid() || self.setup.as_ref().is_some_and(|s| v.rule(s.filing_status()).is_some()))
                || v.mortgage_subsidies.iter().enumerate().any(|(i, entry)| {
                    !entry.valid()
                        || v.mortgage_subsidies[..i].iter().any(|prior| {
                            prior.label == entry.label
                                || entry.home_sale_income_id().is_some()
                                    && entry.home_sale_income_id() == prior.home_sale_income_id()
                        })
                })
                || v.mortgage_credit.as_ref().is_some_and(|credit| {
                    !amounts(&[credit.carry_three_years_prior, credit.carry_two_years_prior, credit.carry_prior_year])
                        || credit.current.as_ref().is_some_and(|current| {
                            current.issuer.trim().is_empty()
                                || current.number.trim().is_empty()
                                || !valid_date(current.issue_date)
                                || current.issue_date > 20261231
                                || !current.interest.valid()
                                || !crate::kernel::natural::exact(current.rate, 100).is_some_and(|rate| (1000..=5000).contains(&rate))
                                || !crate::kernel::natural::exact(current.ownership_share, 100).is_some_and(|share| (1..=10000).contains(&share))
                                || match &current.history {
                                    crate::MortgageCertificateHistory::ReissuedBefore2026 {
                                        facts,
                                    } => {
                                        current.issue_date > 20251231
                                            || facts.refinanced_on > 20251231
                                            || !facts.valid_for(current.rate, false)
                                            || current.interest.certified_debt().is_some_and(|debt| debt != facts.reissued_certified_debt)
                                    }
                                    crate::MortgageCertificateHistory::ReissuedDuring2026SameRate {
                                        facts,
                                        original_period_interest,
                                    } => {
                                        current.issue_date < 20260101
                                            || facts.refinanced_on < 20260101
                                            || !facts.valid_for(current.rate, true)
                                            || current.interest.certified_debt().is_some_and(|debt| debt != facts.reissued_certified_debt)
                                            || !original_period_interest.valid()
                                            || current.interest.certified_interest()
                                                > 100_000_000_000
                                                    - original_period_interest.certified_interest()
                                    }
                                    crate::MortgageCertificateHistory::ReissuedDuring2026DifferentRates {
                                        facts,
                                        original_period_interest,
                                    } => {
                                        current.issue_date < 20260101
                                            || facts.refinanced_on < 20260101
                                            || !facts.valid_for(current.rate, false)
                                            || current.interest.certified_debt().is_some_and(|debt| debt != facts.reissued_certified_debt)
                                            || crate::kernel::natural::units(current.rate, 100)
                                                == crate::kernel::natural::units(facts.original_rate, 100)
                                            || !original_period_interest.valid()
                                            || current.interest.certified_interest()
                                                > 100_000_000_000
                                                    - original_period_interest.certified_interest()
                                    }
                                    _ => false,
                                }
                        })
                })
                || v.dc_homebuyer_carry.is_some_and(|carry| !amounts(&[carry]))
                || v.residential_energy_carry
                    .is_some_and(|carry| !amounts(&[carry]))
                || v.personal_refueling.iter().any(|property| !property.valid())
                || v.clean_vehicles
                    .as_ref()
                    .is_some_and(|claim| !crate::clean_vehicle::valid(claim) || crate::clean_vehicle::source_issue(self, claim).is_some())
                || v.legacy_ev_passive_allowed
                    .is_some_and(|amount| !amounts(&[amount]))
                || !crate::adoption::valid(&v.adoptions, &v.adoption_carry)
                || !crate::marketplace::valid_with_issue(&v.marketplace, self, issue)
                || v.exempt_pensions.iter().any(|b| {
                    b.label.trim().is_empty() || !amounts(&[b.amount])
                })
                || !crate::tax_credit_bonds::valid(&v.tax_credit_bonds)
                || !amounts(&[v.care_expenses()])
                || !v.care.valid(self)
                || v.saver.as_ref().is_some_and(|f| {
                    !amounts(&[
                        f.taxpayer_contributions,
                        f.spouse_contributions,
                        f.taxpayer_able,
                        f.spouse_able,
                    ])
                        || f.distributions.iter().any(|d| !amounts(&[d.amount]))
                })
                || v.education.iter().map(crate::FederalEducation::from).any(|e| {
                    // A removed person's physical source stays loadable for
                    // explicit correction; its identity must still be allocated.
                    !amounts(&[e.tuition, e.scholarships])
                        || e.prior_tuition_adjustment.source_fact().is_err()
                        || e.prior_scholarship_adjustment.source_fact().is_err()
                        || !(matches!(e.student, -2..=0)
                            || (1..self.next_id).contains(&e.student))
                })
                || v.students.iter().enumerate().any(|(index, row)| {
                    v.students[..index].iter().any(|prior| prior.student == row.student)
                        || !v.education.iter().any(|source| source.student == row.student)
                        || !amounts(&[row.more.books_from_school, row.more.other_books, row.more.refunds, row.more.tax_free_529,
                            match row.more.scholarship_in_income {
                                crate::ScholarshipInclusion::None => 0,
                                crate::ScholarshipInclusion::Automatic { living_costs } => living_costs,
                                crate::ScholarshipInclusion::Amount { included } => included,
                            }])
                })
                || v.students.iter().any(|row| {
                    let sources = || v.education.iter().filter(|source| source.student == row.student);
                    sources().try_fold(0_i64, |sum, source| sum.checked_add(source.form.box_1)).is_none_or(|total| !amounts(&[total]))
                        || sources().try_fold(0_i64, |sum, source| sum.checked_add(source.form.box_5)).is_none_or(|total| !amounts(&[total]))
                })
                || v.education_recapture.as_ref().is_some_and(|entry| !entry.valid())
        }) || self.payments.as_ref().is_some_and(|v| {
            v.extension < 0
                || v.credit_applied_next_year < 0
                || (v.credit_applied_next_year > 0 && v.credit_recipient.is_none())
                || v.home_kerosene.as_ref().is_some_and(|fuel| !fuel.valid())
                || v.estimated.issue(
                    &crate::injured_spouse_payments::allowed_management(crate::injured_spouse::texas(self)),
                ).is_some_and(|failure| {
                    *issue = Some(failure.in_field("estimated").in_field("payments"));
                    true
                })
                || v.prior_year.as_ref().is_some_and(|p| !p.valid())
                || v.withholding_periods.as_ref().is_some_and(|p| !p.valid())
                || v.annualized_income.as_ref().is_some_and(|p| !p.valid())
                || v.waiver.as_ref().is_some_and(|w| !w.valid())
        }) {
            return false;
        }
        crate::tax_credit_bonds::source_rule(self).is_none()
    }
}
// At most 100 expense records, each labelled where it must be and within range.
pub(crate) fn expenses_valid(expenses: &[crate::FederalBusinessExpense]) -> bool {
    expenses.len() <= 100
        && expenses.iter().all(|e| {
            (match &e.category {
                crate::BusinessExpenseCategory::Other { label } => !label.trim().is_empty(),
                _ => true,
            }) && amounts(&[e.amount])
                && match e.treatment {
                    crate::BusinessExpenseKind::DeMinimis { largest_cost, .. } => {
                        largest_cost > 0 && amounts(&[largest_cost])
                    }
                    _ => true,
                }
        })
}
pub(crate) use crate::validation::amounts;
pub(crate) fn retirement_printed_amounts_valid(
    values: [i64; 6],
    box_8b_percent: Option<f64>,
    box_9a_percent: Option<f64>,
    gross: i64,
    box_10: i64,
) -> bool {
    amounts(&values)
        && crate::lump_sum::percent_valid(box_8b_percent)
        && crate::lump_sum::percent_valid(box_9a_percent)
        && (0..=gross).contains(&box_10)
}

pub(crate) fn social_printed_amounts_valid(benefits: i64, withholding: i64) -> bool {
    (-100_000_000_000..=100_000_000_000).contains(&benefits) && amounts(&[withholding])
}
pub(crate) fn validate_income(e: &FederalIncomeEntry) -> Result<(), (&'static str, &'static str)> {
    validate_income_issue(e).map_err(|(issue, message)| (issue.key, message))
}
pub(crate) fn validate_income_amount(
    income: &FederalIncome,
) -> Result<(), (&'static str, &'static str)> {
    validate_income_amount_issue(income).map_err(|(issue, message)| (issue.key, message))
}
pub(crate) fn validate_income_issue(
    e: &FederalIncomeEntry,
) -> Result<(), (crate::validation::ValidationIssue, &'static str)> {
    validate_income_issue_for_year(e, crate::year_constants::TAX_YEAR)
}
pub(crate) fn validate_income_issue_for_year(
    e: &FederalIncomeEntry,
    year: u32,
) -> Result<(), (crate::validation::ValidationIssue, &'static str)> {
    validate_income_issue_for_year_with_mode(e, year, true)
}

pub(crate) fn validate_income_issue_for_year_with_mode(
    e: &FederalIncomeEntry,
    year: u32,
    source_alignment: bool,
) -> Result<(), (crate::validation::ValidationIssue, &'static str)> {
    if let Some(issue) = crate::method_change::farm_link_issue(&e.income) {
        return Err((
            issue,
            crate::fixture_english!(
                "Select this farm's actual accepted accounting-change source and its consent evidence."
            ),
        ));
    }
    if crate::investment_statements::unresolved_contingent(&e.income) {
        return Err((
            crate::validation::ValidationIssue::new("ISSUE_DEBT_CONTINGENT_INCOME"),
            crate::fixture_english!(
                "A contingent-payment instrument needs its comparable-yield and projected-payment computation."
            ),
        ));
    }
    validate_income_amount_issue_with_mode(&e.income, year, source_alignment)
}

pub(crate) fn validate_income_amount_issue(
    income: &FederalIncome,
) -> Result<(), (crate::validation::ValidationIssue, &'static str)> {
    validate_income_amount_issue_for_year(income, crate::year_constants::TAX_YEAR)
}
pub(crate) fn validate_income_amount_issue_for_year(
    income: &FederalIncome,
    year: u32,
) -> Result<(), (crate::validation::ValidationIssue, &'static str)> {
    validate_income_amount_issue_with_mode(income, year, true)
}

pub(crate) fn validate_income_amount_issue_with_mode(
    income: &FederalIncome,
    year: u32,
    source_alignment: bool,
) -> Result<(), (crate::validation::ValidationIssue, &'static str)> {
    let mut native_issue = None;
    validate_income_amount_details(income, &mut native_issue, year, source_alignment).map_err(
        |(key, message)| {
            let mut issue =
                native_issue.unwrap_or_else(|| crate::validation::ValidationIssue::new(key));
            issue.key = key;
            (issue, message)
        },
    )
}
// Preserve legacy localized/fixture diagnostics while carrying metadata from
// the same native amount pass. This helper is the sole predicate owner.
fn validate_income_amount_details(
    income: &FederalIncome,
    native_issue: &mut Option<crate::validation::ValidationIssue>,
    year: u32,
    source_alignment: bool,
) -> Result<(), (&'static str, &'static str)> {
    if let FederalIncome::Scorp { data } = income
        && let Some(issue) = data.admission_issue()
    {
        let key = issue.key;
        *native_issue = Some(issue);
        return Err((
            key,
            crate::fixture_english!(
                "Review the document amounts; they must be within the supported input range."
            ),
        ));
    }
    if let FederalIncome::EstateTrustPortfolio { data } = income
        && let Some(issue) = data.admission_issue()
    {
        let key = issue.key;
        *native_issue = Some(issue);
        return Err((
            key,
            crate::fixture_english!(
                "Review the document amounts; they must be within the supported input range."
            ),
        ));
    }
    if let Some(issue) = crate::interest_limit::issue(income, year) {
        *native_issue = Some(issue);
        return Err((
            "INTEREST_LIMIT_ELECTION_SOURCE",
            crate::fixture_english!(
                "Correct the actual activity election kind and effective year."
            ),
        ));
    }
    if let Some(issue) = crate::macrs_records::building_facts_issue(
        crate::macrs_records::improvement_buildings(income),
    ) {
        let key = issue.key;
        *native_issue = Some(issue);
        return Err((
            key,
            crate::fixture_english!(
                "Review the physical building's description and original service evidence."
            ),
        ));
    }
    if let Some(issue) = crate::k1_activity::income_issue(income) {
        let key = issue.key;
        *native_issue = Some(issue);
        return Err((
            key,
            crate::fixture_english!(if key == crate::k1_activity::ordinary::RANGE_KEY {
                "Review the document amounts; they must be within the supported input range."
            } else {
                "Reconcile this K-1's actual component source records."
            }),
        ));
    }
    if let FederalIncome::Business { data } = income
        && !data
            .material_participation
            .valid(year, matches!(data.activity, crate::BusinessActivity::Farming { .. }))
    {
        *native_issue = Some(
            crate::validation::ValidationIssue::new("DIRECT_PARTICIPATION_FACTS_INVALID")
                .at_field("material_participation")
                .in_field("more"),
        );
        return Err((
            "DIRECT_PARTICIPATION_FACTS_INVALID",
            crate::fixture_english!(
                "Review this activity's qualifying hours, selected participation test and actual historical years."
            ),
        ));
    }
    let valid = match income {
        // The complete new-source rule above owns this source's validation once.
        FederalIncome::PartnershipActivities { .. } | FederalIncome::ScorpActivities { .. } => true,
        FederalIncome::Royalty { data } => data.valid(),
        FederalIncome::ChildElection { data } => data.valid(),
        FederalIncome::Cooperative { data } => {
            *native_issue = crate::FederalCooperative::from(data).issue();
            native_issue.is_none()
        }
        FederalIncome::Partnership { data } => {
            *native_issue = data.validation_issue_with_mode(
                "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                source_alignment,
            );
            native_issue.is_none()
        }
        FederalIncome::Scorp { data } => {
            let data = &crate::ScorpK1::from(data);
            if source_alignment {
                data.valid()
            } else {
                data.valid_with_sale_carry(false, crate::scorp_amt::ScorpSourceCapacity::regular())
            }
        }
        FederalIncome::EstateTrustPortfolio { data } => {
            crate::FederalEstateTrustPortfolioK1::from(data).valid_with_mode(source_alignment)
        }
        FederalIncome::NonbusinessProperty { kind, receipts, expenses, .. } => {
            amounts(&[*receipts, *expenses])
                && match kind {
                    NonbusinessPropertyKind::PersonalRental {
                        basis,
                        depreciable_basis,
                        assets,
                        history,
                        ..
                    } => {
                        (0..=100000000000).contains(basis)
                            && (0..=*basis).contains(depreciable_basis)
                            && amounts(&[history.balance(false), history.balance(true)])
                            && assets.len() <= 100
                            && assets.iter().all(crate::BusinessAsset::valid)
                            && crate::kernel::ratio::checked_sum(assets.iter().map(|a| a.cost))
                                .is_some_and(|cost| cost <= *depreciable_basis)
                    }
                }
        }
        FederalIncome::SmallBusinessStock { data } => data.valid(),
        FederalIncome::QualifiedSmallBusinessSale { data } => data.valid(),
        FederalIncome::WorthlessSecurity { data } => {
            amounts(&[data.basis]) && data.acquisition.valid()
        }
        FederalIncome::NonbusinessBadDebt { data } => data.source.valid() && data.rule().is_none(),
        FederalIncome::HomeSale { data } => {
            crate::HomeSale::from(data).valid()
                && match &data.answers {
                    Some(crate::RealEstateSaleUse::Other(other)) => {
                        amounts(&[other.basis, other.expenses])
                    }
                    _ => true,
                }
        }
        FederalIncome::ReturnOfCapital { amount, basis, property, .. } => {
            crate::CapitalReturn1099Div::valid_amounts(*amount, *basis, property)
        }
        FederalIncome::UndistributedGain { data } => {
            let v = &crate::UndistributedGain::from(data);
            if source_alignment && v.small_business != 0 && v.section_1202_exclusion.is_none() {
                *native_issue = Some(
                    crate::validation::ValidationIssue::new(
                        "FORM_2439_SECTION_1202_NEEDS_EXCLUSION",
                    )
                    .at_field("section_1202_exclusion"),
                );
                return Err((
                    "FORM_2439_SECTION_1202_NEEDS_EXCLUSION",
                    crate::fixture_english!(
                        "Box 1c section 1202 gain is part of box 1a, and its exclusion must be chosen."
                    ),
                ));
            }
            amounts(&[v.gain, v.unrecaptured, v.small_business, v.collectibles, v.tax_paid])
                && v.unrecaptured + v.small_business + v.collectibles <= v.gain
                && v.tax_paid <= v.gain
        }
        FederalIncome::OpportunityFund { data } => data.valid(),
        FederalIncome::OpportunityFundDeferral { data } => data.valid(),
        FederalIncome::Able { data } => crate::AbleDistribution::from(data).valid(),
        FederalIncome::TrumpDeath { data: stored } => {
            let data = &crate::TrumpDeathAcquisition::from(stored);
            data.valid()
        }
        FederalIncome::TrumpAbleRollover { data: stored } => {
            let data = &crate::TrumpAbleRollover::from(stored);
            data.valid()
        }
        FederalIncome::LtcBenefits { data } => data.record().valid(),
        FederalIncome::LtcInsured { data } => data.valid(),
        FederalIncome::MedicalDistribution { data } => {
            crate::FederalMedicalDistribution::from(data).valid()
        }
        FederalIncome::HobbySales { data } => data.valid(),
        FederalIncome::PersonalDebt { data } => {
            let data = &crate::PersonalDebtCancellation::from(data);
            if source_alignment { data.valid() } else { data.valid_with_mode(false) }
        }
        FederalIncome::PropertyAcquisition { data } => {
            let data = &crate::PropertyAcquisition::from(data);
            if source_alignment { data.valid() } else { data.valid_with_mode(false) }
        }
        FederalIncome::HealthLoanAssistance { data } => data.valid(),
        FederalIncome::WildfireRelief { data } => data.valid(),
        FederalIncome::DisasterRelief { data } => data.valid(),
        FederalIncome::CasualtyLivingInsurance { data } => data.valid(),
        FederalIncome::DisasterMitigation { data } => data.valid(),
        FederalIncome::OtherIncome { data } => data.valid(),
        FederalIncome::Nec1099 { data } => crate::FederalForm1099Nec::from(data).valid(),
        FederalIncome::Misc1099 { data } => crate::FederalForm1099Misc::from(data).valid(),
        FederalIncome::K1099 { data } => crate::FederalForm1099K::from(data).valid(),
        FederalIncome::Scholarship { data } => data.valid(),
        FederalIncome::JuryDuty { data } => data.valid(),
        FederalIncome::Alimony { data } => data.valid(),
        FederalIncome::BondRedemption { data } => data.form.line8 <= data.form.line6,
        FederalIncome::SavingsBonds { data } => {
            data.record().map_or(!source_alignment, |record| record.valid())
        }
        FederalIncome::Section1256 { data } => crate::FederalSection1256::from(data).valid(),
        FederalIncome::EducationDistribution { data } => {
            crate::FederalEducationDistribution::from(data).valid()
        }
        FederalIncome::EducationAccountExpenses { data } => data.valid(),
        FederalIncome::ForeignEmployment { data: v } => amounts(&[v.wages]),
        FederalIncome::EmployeeWages { data } => data.valid(),
        FederalIncome::Rental { data: v } => crate::extended_rules::rental_source_valid(v),
        FederalIncome::FarmRental { data } => data.valid(),
        FederalIncome::Wage { data: w } => {
            let w = &crate::FederalWage::from(w);
            amounts(&[
                w.wages,
                w.withholding,
                w.social_wages,
                w.social_tips,
                w.social_tax,
                w.medicare_wages,
                w.medicare_tax,
                w.medicaid_waiver,
                w.deferred_pension,
                w.inmate_pay,
                w.disability.amount(),
                w.abroad,
            ]) && (!source_alignment
                || w.deferred_pension + w.inmate_pay + w.medicaid_waiver <= w.wages)
                && w.payroll.valid()
                && (!source_alignment || w.disability.amount() <= w.wages)
                && (!source_alignment || w.abroad <= w.wages)
                && w.employee_tips.as_ref().is_none_or(|v| v.valid())
                && w.uncollected_tax.as_ref().is_none_or(|v| v.valid())
                && w.work_deductions.tips_valid()
                && amounts(&[w.work_deductions.overtime()])
        }
        FederalIncome::Interest { data: v } => {
            let v = &crate::FederalInterest::from(v);
            if v.private_activity > v.exempt {
                return Err((
                    "INT_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                    crate::fixture_english!(
                        "Private-activity interest must be included in exempt income."
                    ),
                ));
            }
            if let Some(issue) = if source_alignment {
                crate::investment_statements::int_issue(v)
            } else {
                crate::investment_statements::int_stored_issue(v)
            } {
                return Err((
                    issue.key,
                    crate::fixture_english!(
                        "Investment ownership or reporting facts are inconsistent."
                    ),
                ));
            }
            v.private_activity <= v.exempt
                && amounts(&[
                    v.taxable,
                    v.us_obligations,
                    v.exempt,
                    v.private_activity,
                    v.withholding,
                    v.early_penalty,
                    v.bond_premium,
                    v.treasury_bond_premium,
                    v.tax_exempt_bond_premium,
                    v.accrued_interest,
                    v.foreign_tax,
                    v.investment_expenses,
                    v.market_discount,
                ])
        }
        FederalIncome::Oid { data: v } => {
            let v = &crate::FederalOid::from(v);
            if source_alignment && v.private_activity > v.exempt {
                return Err((
                    "OID_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                    crate::fixture_english!(
                        "Private-activity interest must be included in exempt income."
                    ),
                ));
            }
            if let Some(issue) = if source_alignment {
                crate::investment_statements::oid_issue(v)
            } else {
                crate::investment_statements::oid_stored_issue(v)
            } {
                return Err((
                    issue.key,
                    crate::fixture_english!(
                        "Investment ownership or reporting facts are inconsistent."
                    ),
                ));
            }
            (!source_alignment || v.private_activity <= v.exempt)
                && (-MAX..=MAX).contains(&v.treasury_oid)
                && amounts(&[
                    v.oid,
                    v.other_periodic_interest,
                    v.early_penalty,
                    v.withholding,
                    v.market_discount,
                    v.acquisition_premium,
                    v.exempt,
                    v.bond_premium,
                    v.private_activity,
                    v.investment_expenses,
                    v.tax_exempt_bond_premium,
                ])
        }
        FederalIncome::Dividend { data } => {
            let v = &crate::FederalDividend::from(data);
            if v.collectible_distributions + v.unrecaptured_distributions > v.capital_distributions
            {
                return Err((
                    "ISSUE_FEDERAL_RULES_SECTION_1250_AND_COLLECTIBLE_COMPONENTS_IN_AF75329A",
                    crate::fixture_english!(
                        "Section 1250 and collectible components in boxes 2b and 2d cannot exceed total capital gain distributions in box 2a."
                    ),
                ));
            }
            if v.qualified + v.section199a.amount() > v.ordinary {
                return Err((
                    "ISSUE_FEDERAL_RULES_QUALIFIED_AND_SECTION_199A_DIVIDENDS_MUST_94CFB1A7",
                    crate::fixture_english!(
                        "Qualified and section 199A dividends must be separate parts of ordinary dividends; their sum cannot exceed box 1a."
                    ),
                ));
            }
            if v.private_activity_dividends > v.exempt_dividends {
                return Err((
                    "ISSUE_FEDERAL_RULES_SPECIFIED_PRIVATE_ACTIVITY_BOND_INTEREST_DIVIDENDS_IN_AAA9238B",
                    crate::fixture_english!(
                        "Specified private-activity bond interest dividends in box 13 cannot exceed exempt-interest dividends in box 12."
                    ),
                ));
            }
            if v.section_1202_gain > v.capital_distributions
                || source_alignment
                    && v.section_1202_gain != 0
                    && v.section_1202_exclusion.is_none()
            {
                return Err((
                    if v.section_1202_gain > v.capital_distributions {
                        "DIV_SECTION_1202_ABOVE_2A"
                    } else {
                        "DIV_SECTION_1202_NEEDS_EXCLUSION"
                    },
                    crate::fixture_english!(
                        "Box 2c section 1202 gain is part of box 2a, and its exclusion must be chosen."
                    ),
                ));
            }
            if v.section_897_ordinary > v.ordinary {
                return Err((
                    "DIV_897_ORDINARY_ABOVE_BOX",
                    crate::fixture_english!(
                        "Box 2e section 897 ordinary dividends cannot exceed box 1a."
                    ),
                ));
            }
            if v.section_897_capital > v.capital_distributions {
                return Err((
                    "DIV_897_CAPITAL_ABOVE_BOX",
                    crate::fixture_english!(
                        "Box 2f section 897 capital gain cannot exceed box 2a."
                    ),
                ));
            }
            if let Some(issue) = if source_alignment {
                crate::investment_statements::div_issue(v)
            } else {
                crate::investment_statements::div_stored_issue(v)
            } {
                return Err((
                    issue.key,
                    crate::fixture_english!(
                        "Investment ownership or reporting facts are inconsistent."
                    ),
                ));
            }
            amounts(&[
                v.ordinary,
                v.qualified,
                v.section199a.amount(),
                v.capital_distributions,
                v.collectible_distributions,
                v.unrecaptured_distributions,
                v.section_1202_gain,
                v.withholding,
                v.exempt_dividends,
                v.private_activity_dividends,
                v.foreign_tax,
                v.section_897_ordinary,
                v.section_897_capital,
            ])
        }
        FederalIncome::NuaSale { data } => data.valid(),
        FederalIncome::EmployeeStockSale { data } => data.valid(),
        FederalIncome::EsppSale { data } => data.valid(),
        FederalIncome::EsppSaleSource { data } => data.issue().is_none(),
        // The printed values alone must fit the calculation's domain.
        FederalIncome::IsoSaleSource { data } => {
            data.issue().is_none()
                && data
                    .lot(&crate::IsoShareSaleMore {
                        selling_fees: 0,
                        acquisition_costs: 0,
                        option_cost: 0,
                        special: false,
                    })
                    .valid()
        }
        FederalIncome::NuaSaleSource { data } => {
            data.issue().is_none()
                && data.lot(&crate::NuaShareSaleMore { wash_sale: 0, special: false }, None).valid()
        }
        FederalIncome::IsoSale { data } => data.valid(),
        FederalIncome::InstallmentSale { data } => {
            *native_issue = data.issue();
            native_issue.is_none()
        }
        FederalIncome::Sale { data } => {
            validate_sale(&crate::FederalSale::from(data))?;
            true
        }
        FederalIncome::Broker { data } => {
            *native_issue = data.year_issue(year).or_else(|| data.admission_issue());
            if let Some(issue) = native_issue {
                return Err((
                    issue.key,
                    crate::fixture_english!("Review the actual broker form's printed facts."),
                ));
            }
            if let Some(sale) = crate::broker_calculation::sale(income) {
                validate_sale_with_mode(&sale, false)?;
            }
            true
        }
        FederalIncome::Roth { data: stored } => {
            let data = &crate::RothWithdrawal::from(stored);
            data.valid_with_source_alignment(source_alignment)
        }
        FederalIncome::InheritedIra { data } => data.valid(),
        FederalIncome::Retirement { data: stored } => {
            let v = &crate::FederalRetirement::from(stored);
            let care_issue = if source_alignment {
                v.care_charge_rule()
            } else {
                match stored {
                    crate::RetirementSource::R1099(source) => {
                        crate::DistributionCodes::care_charge_source_issue(&source.form)
                            .map(|issue| issue.key)
                    }
                    crate::RetirementSource::Rrb1099R(_) => None,
                }
            };
            if let Some(key) = care_issue {
                return Err((
                    key,
                    crate::fixture_english!(
                        "A qualified codeW care charge needs its actual nontaxable source and reported investment reduction."
                    ),
                ));
            }
            if matches!(v.taxable, Some(crate::RetirementTaxable::Payer)) && v.box_2a > v.gross {
                return Err((
                    "ISSUE_FEDERAL_RULES_TAXABLE_RETIREMENT_INCOME_CANNOT_EXCEED_ITS_D4BD4F9F",
                    crate::fixture_english!(
                        "Taxable retirement income cannot exceed its gross distribution in this supported path."
                    ),
                ));
            }
            retirement_printed_amounts_valid(
                [v.gross, v.withholding, v.box_2a, v.box_3, v.box_6, v.box_8a],
                v.box_8b_percent,
                v.box_9a_percent,
                v.gross,
                v.box_10,
            ) && amounts(&[v.disability.amount()])
                && (!source_alignment || v.disability.amount() <= v.gross)
                && v.disability_wages.valid()
                && (!source_alignment
                    || v.disability_wages == crate::DisabilityWages::None
                    || v.reports_disability_wages())
                && v.disaster.valid_with_source_alignment(v.gross, source_alignment)
                && v.box_10_basis.is_none_or(|basis| {
                    amounts(&[basis]) && (!source_alignment || basis <= v.box_10)
                })
                && match v.ordinary_distribution {
                    Some(RetirementTreatment::Corrective { excess_deferral }) => {
                        (0..=100000000000).contains(&excess_deferral)
                    }
                    _ => true,
                }
                && match v.taxable {
                    None => true,
                    Some(crate::RetirementTaxable::Payer) => amounts(&[v.box_2a]),
                    Some(crate::RetirementTaxable::CivilServiceDeath {
                        data: crate::SurvivorLumpSum { cost, recovered, .. },
                    }) => amounts(&[cost, recovered]) && recovered <= cost,
                    Some(crate::RetirementTaxable::PropertyProceeds { ref data }) => data.valid(),
                    Some(crate::RetirementTaxable::Inherited { account }) => {
                        account.is_none_or(|id| id > 0)
                            && amounts(&[v.box_2a])
                            && (!source_alignment || v.box_2a <= v.gross)
                            && (!source_alignment || v.kind == RetirementKind::Inherited)
                    }
                    Some(crate::RetirementTaxable::LegacyAnnuity { ref data }) => {
                        data.valid_with_source_alignment(v.gross, source_alignment)
                    }
                    Some(crate::RetirementTaxable::IraBasis) => true,
                    Some(crate::RetirementTaxable::FullDischarge {
                        data: crate::ContractDischarge { investment, .. },
                    }) => amounts(&[investment]),
                    Some(crate::RetirementTaxable::QualifiedWithdrawal {
                        data: crate::PlanDistribution { value, investment, .. },
                    }) => {
                        amounts(&[value, investment])
                            && value > 0
                            && (!source_alignment || v.gross <= value)
                    }
                    Some(crate::RetirementTaxable::DeferredWithdrawal { ref data }) => {
                        data.value >= 0
                            && data.investment >= 0
                            && (!source_alignment || v.gross <= data.value)
                            && data.related.iter().all(|v| v.value >= 0 && v.investment >= 0)
                    }
                    Some(crate::RetirementTaxable::GeneralAnnuity { data }) => {
                        data.valid_with_source_alignment(v.gross, source_alignment)
                    }
                    Some(crate::RetirementTaxable::VariableAnnuity { data }) => data.valid(),
                    Some(crate::RetirementTaxable::Railroad {
                        repaid,
                        contributory,
                        box_5,
                        box_6,
                        ref basis,
                        ref repayments,
                        ..
                    }) => {
                        (0..=v.gross).contains(&contributory)
                            && repaid >= 0
                            && amounts(&[box_5, box_6])
                            && contributory + box_5 + box_6 == v.gross
                            && basis.as_ref().is_none_or(|b| b.valid())
                            && repayments.iter().all(|p| {
                                (1900..=2025).contains(&p.received_year) && amounts(&[p.amount])
                            })
                    }
                    Some(crate::RetirementTaxable::ChildAnnuity { ref data }) => {
                        data.valid_with_source_alignment(v.gross, source_alignment)
                    }
                    Some(crate::RetirementTaxable::Simplified { ref data }) => data.valid(),
                }
                && match &v.ordinary_distribution {
                    Some(RetirementTreatment::Rollover { amount, .. }) => {
                        let ceiling = match &v.taxable {
                            Some(crate::RetirementTaxable::PropertyProceeds { data }) => {
                                data.proceeds
                            }
                            _ => v.gross,
                        };
                        amounts(&[*amount])
                            && (if matches!(
                                v.taxable,
                                Some(crate::RetirementTaxable::PropertyProceeds { .. })
                            ) {
                                *amount <= ceiling
                            } else {
                                !source_alignment || *amount <= ceiling
                            })
                    }
                    Some(RetirementTreatment::Conversion { amount, .. }) => {
                        amounts(&[*amount]) && (!source_alignment || *amount <= v.gross)
                    }
                    Some(RetirementTreatment::Exception { distributed, facts, .. }) => {
                        facts.valid_with_source_alignment(*distributed, v.gross, source_alignment)
                    }
                    // Form 4972 (lump_sum.rs): a qualified-plan record with the payer's box 2a
                    // and exactly code 7A (the filer's own plan) or 4A (a decedent's).
                    Some(RetirementTreatment::LumpSum { data }) => {
                        let a = crate::DistributionCodes::CODE_A.0;
                        (!source_alignment || v.kind == RetirementKind::Pension)
                            && data.valid_with_source_alignment(v.box_2a, v, source_alignment)
                            && (!source_alignment
                                || matches!(v.taxable, Some(crate::RetirementTaxable::Payer)
                                if data.valid(v.box_2a, v)))
                            && (!source_alignment
                                || match data.participant {
                                    crate::Form4972Participant::Own { .. } => {
                                        v.box7 == crate::DistributionCodes::CODE_7.0 | a
                                    }
                                    crate::Form4972Participant::Decedent { .. } => {
                                        v.box7 == crate::DistributionCodes::CODE_4.0 | a
                                    }
                                })
                    }
                    _ => true,
                }
        }
        FederalIncome::SocialSecurity { data } => {
            let v = crate::FederalSocial::from(data);
            social_printed_amounts_valid(v.benefits, v.withholding)
                && match data {
                    crate::SocialBenefitStatement::Rrb1099(paper) => {
                        crate::Rrb1099::from(paper).issue().is_none()
                    }
                    crate::SocialBenefitStatement::Ssa1099(_) => true,
                }
        }
        FederalIncome::GovernmentPayment { data } => {
            let v = crate::FederalUnemployment::from(data);
            amounts(&[
                v.amount,
                v.withholding,
                v.taxable_grants,
                v.agriculture_payment,
                v.rtaa,
                v.market_gain,
                v.family_leave,
            ]) && (data.form.state_or_local_income_tax_refunds == 0 || data.history.valid())
                && v.repayments.len() <= 100
                && v.repayments
                    .iter()
                    .all(|p| (1900..=2026).contains(&p.received_year) && amounts(&[p.amount]))
        }
        FederalIncome::Gambling { data } => {
            let v = FederalGambling::from(data);
            amounts(&[v.winnings, v.losses, v.withholding])
        }
        FederalIncome::Business { data: v } => {
            v.activity.valid()
                && v.at_risk
                    .issue_for_method(
                        &if matches!(v.activity, crate::BusinessActivity::Farming { .. }) {
                            crate::RiskEffectiveCategory::Farming
                        } else {
                            crate::RiskEffectiveCategory::OtherTradeBusiness
                        },
                        true,
                    )
                    .is_none()
                && (0..=100000000000).contains(&v.notary_profit)
                && v.notary_gross
                    .is_none_or(|gross| (v.notary_profit..=100000000000).contains(&gross))
                && v.income_source.valid()
                && v.payroll.valid()
                && v.inventory.valid()
                && v.other_income.len() <= 20
                && v.other_income.iter().all(|i| {
                    !i.description.trim().is_empty() && (1..=100000000000).contains(&i.amount)
                })
                && v.business_code.is_none_or(|c| (100000..=999999).contains(&c))
                && (v.ein.is_empty()
                    || (v.ein.len() == 10
                        && v.ein.as_bytes()[2] == b'-'
                        && v.ein.bytes().enumerate().all(|(i, b)| i == 2 || b.is_ascii_digit())))
                && v.de_minimis.as_ref().is_none_or(crate::DeMinimisElection::valid)
                && v.leased_vehicles.len() <= 100
                && v.leased_vehicles.iter().all(crate::BusinessLeasedVehicle::valid)
                && (!matches!(v.activity, crate::BusinessActivity::Farming { .. })
                    || v.leased_vehicles.is_empty())
                && v.depletion.as_ref().is_none_or(crate::BusinessDepletion::valid)
                && v.startup.valid()
                && crate::method_change::valid(&v.method_changes)
                && v.intangibles.iter().all(crate::BusinessIntangible::valid)
                && v.timber.iter().all(crate::BusinessTimberCost::valid)
                && v.other_amortization.iter().all(crate::OtherAmortizableCost::valid)
                && (v.timber.is_empty()
                    || (matches!(v.activity, crate::BusinessActivity::Ordinary { .. })
                        && v.source == crate::BusinessSource::UnitedStates))
                && v.health_insurance.valid()
                && v.retirement.valid()
                && v.assets.len() <= 100
                && v.assets.iter().all(crate::BusinessAsset::valid)
                && v.vehicles.len() <= 100
                && v.vehicles.iter().all(|vehicle| vehicle.valid())
                && expenses_valid(&v.expenses)
                && v.tips.as_ref().is_none_or(|tips| amounts(&[tips.amount]))
                && v.overtime.as_ref().is_none_or(|overtime| amounts(&[overtime.amount]))
                && v.fuel_tax_credit.as_ref().is_none_or(|claim| claim.valid())
                && v.employer_tip_fica_credit
                    .as_ref()
                    .is_none_or(crate::EmployerTipFicaCredit::valid)
                && v.work_opportunity_credit
                    .as_ref()
                    .is_none_or(crate::WorkOpportunityCredit::valid)
                && v.retirement_plan_credit.as_ref().is_none_or(crate::RetirementPlanCredit::valid)
                && v.small_employer_health_credit
                    .as_ref()
                    .is_none_or(crate::SmallEmployerHealthCredit::valid)
                && v.paid_leave_credit.as_ref().is_none_or(crate::PaidLeaveCredit::valid)
                && v.cash_fuel_tax_recovery
                    .as_ref()
                    .is_none_or(|recovery| recovery.valid() && v.cash_accounting())
                && v.withholding >= 0
                && amounts(&[v.withholding])
                && {
                    if (!v.intangibles.is_empty()
                        || !v.timber.is_empty()
                        || !v.other_amortization.is_empty())
                        && (v.checked_amortization_cents().is_none()
                            || [false, true].into_iter().any(|mid_quarter| {
                                v.checked_operating_cost_before_supplies(mid_quarter).is_none()
                            }))
                    {
                        let (key, field) = if !v.intangibles.is_empty() {
                            ("AMORTIZATION_TOTAL_UNREPRESENTABLE", "intangibles")
                        } else if v.other_amortization.is_empty() {
                            ("TIMBER_TOTAL_UNREPRESENTABLE", "timber")
                        } else {
                            ("AMORTIZATION_TOTAL_UNREPRESENTABLE", "other_amortization")
                        };
                        *native_issue = Some(
                            crate::validation::ValidationIssue::new(key)
                                .at_field(field)
                                .in_field("more"),
                        );
                        return Err((
                            key,
                            crate::fixture_english!(
                                "Correct the original amortization sources before the combined native amount can be calculated."
                            ),
                        ));
                    }
                    true
                }
        }
    };
    if valid {
        Ok(())
    } else {
        Err((
            "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
            crate::fixture_english!(
                "Review the document amounts; they must be within the supported input range."
            ),
        ))
    }
}

fn expense_elections(
    r: &mut FederalResult,
    expenses: &[crate::FederalBusinessExpense],
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
                msg!("EXPLAIN_N_EXPENSE_N_DE_MINIMIS_2"),
            );
        }
    }
}

// What the Form 8814 elections report on this return, in whole dollars
// (child_election.rs): the amounts already in the dividend and capital
// totals are not repeated here.
#[derive(Default)]
pub(crate) struct ChildTotals {
    // Same-invocation source parents, in native whole dollars. The totals
    // below remain unchanged; these projections never rerun Form8814.
    pub(crate) qualified_by_owner: [i64; 2],
    pub(crate) other_by_owner: [i64; 2],
    pub(crate) investment_by_owner: [i64; 2],
    pub(crate) tax_by_owner: [i64; 2],
    // Line 9, the qualified dividends the elections report.
    pub(crate) qualified: i64,
    // Line 10, the capital gain distributions the elections report.
    pub(crate) distributions: i64,
    // Line 12: Schedule 1 line 8z.
    pub(crate) other: i64,
    // Line 12 less the Alaska Permanent Fund share: Form 8960 line 7, Form
    // 4952 investment income and Pub 596 Worksheet 1 line 4.
    pub(crate) investment: i64,
    // Line 1b, the children's tax-exempt interest (Pub 596 Worksheet 1 line 2).
    pub(crate) exempt: i64,
    // Line 15: Form 1040 line 16.
    pub(crate) tax: i64,
    // Form 8962 household income (i8962 line 2b, "Form 8814"): line 1b plus the lesser of
    // line 4 and line 5, for each election over the $1,350 not taxed.
    pub(crate) household_income: i64,
}

// Native earned income before foreign, clergy and community adjustments.
// Form2441 uses this same source expression in its before-section179 projection.
pub(crate) fn earned_before_special_adjustments(
    t: &Totals,
    i: usize,
    profit: i64,
    half: i64,
) -> i64 {
    dollars(
        t.wages[i]
            + t.iso_unreported[i]
            + t.disability_pension[i]
            + t.moving_excess[i]
            + t.deferral_excess[i]
            + t.household_wages[i]
            + t.strike_benefits[i]
            + t.medicaid_taxable[i]
            + t.statutory_gross[i]
            - t.medicaid_in_box1[i],
    ) + profit
        - half
}

#[derive(Default)]
pub(crate) struct Totals {
    pub(crate) research_items:
        alloc::collections::BTreeMap<i64, crate::general_business_credit::ResearchItems>,
    // Whole counterpart payroll facts retained only for Form8958's partner
    // SE projection; never included in this separate return's payroll tax.
    community_counterpart_payroll: [i64; 2],
    pub(crate) household_wages: [i64; 2],
    pub(crate) strike_benefits: [i64; 2],
    pub(crate) medicaid_taxable: [i64; 2],
    pub(crate) medicaid_excluded: [i64; 2],
    pub(crate) medicaid_in_box1: [i64; 2],
    pub(crate) combat_pay: [i64; 2],
    pub(crate) statutory_gross: [i64; 2],
    pub(crate) statutory_profit: [i64; 2],
    pub(crate) employee_wages_not_covered: [i64; 2],
    pub(crate) rrta_compensation: [i64; 2],
    pub(crate) rrta_medicare: [i64; 2],
    pub(crate) rrta_additional_withheld: [i64; 2],
    pub(crate) rrta_tier_2_forms: [i64; 2],
    // Form 8814 elections reported on this return (whole dollars).
    child: ChildTotals,
    inherited_estate_tax: i64,
    able: [i64; 2],
    able_penalty_base: [i64; 2],
    able_corrections: i64,
    trump_death: i64,
    pub(crate) wages: [i64; 2],
    pub(crate) employee_tips: [crate::employee_tips::TipAmounts; 2],
    employee_wages: [i64; 2],
    // Physical payroll source bases: never part of shared() income allocation.
    pub(crate) employee_8919: [i64; 2],
    employee_payroll_social: [i64; 2],
    employee_payroll_medicare: [i64; 2],
    employee_other_social: [i64; 2],
    employee_other_medicare: [i64; 2],
    employee_unknown_social: [i64; 2],
    employee_unknown_medicare: [i64; 2],
    employee_rrta_compensation_unknown: [bool; 2],
    employee_rrta_withholding_unknown: [bool; 2],
    pub(crate) uncollected_social: [i64; 2],
    pub(crate) uncollected_medicare: i64,
    // Same native collection; keep the actual payee for source attribution.
    pub(crate) uncollected_medicare_owners: [i64; 2],
    pub(crate) ordinary_uncollected_medicare: i64,
    pub(crate) ordinary_uncollected_medicare_owners: [i64; 2],
    // Section 4999 tax, cents by actual payee. W-2 K is already tax;
    // Form 1099-NEC box 3 is the excess-payment base, already in box 1a.
    pub(crate) golden_parachute_tax: [i64; 2],
    pub(crate) iso_unreported: [i64; 2],
    iso_amt_wage_adjustment: [i64; 2],
    nua_excluded_gain: i64,
    // Actual source IDs and same-call realized NUA. A missing confirmation
    // remains unknown; signed sums cannot prove each current effect absent.
    nua_exclusion_sources: Vec<(i64, Option<i64>)>,
    disability_pension: [i64; 2],
    pub(crate) moving_excess: [i64; 2],
    // Excess elective deferrals, in cents per owner (Form 1040 line 1h).
    pub(crate) deferral_excess: [i64; 2],
    scholarship: i64,
    pub(crate) personal_debt: i64,
    // Unconfirmed health-service loan repayment assistance (Pub. 970 ch. 5):
    // other income, Schedule 1 line 8z, not canceled debt (line 8c).
    health_assistance: i64,
    health_loan_interest: i64,
    pub(crate) prizes: i64,
    // Schedule 1 lines 8g, 8m, 8v, 8z and 8j beyond hobby goods (unit b9-2), cents.
    pub(crate) alaska: i64,
    pub(crate) olympic: i64,
    pub(crate) digital: i64,
    pub(crate) other_8z: i64,
    // The 1099-MISC box 8 part of line 8z, which is net investment income.
    pub(crate) substitute: i64,
    pub(crate) not_for_profit: i64,
    // Form 1099-K parts that were not income (the top of Schedule 1).
    pub(crate) not_income_1099k: i64,
    // Lines 8t and 8u by owner: W-2 box 1 pay and Other income's inmate pay.
    pub(crate) deferred_pension: [i64; 2],
    pub(crate) inmate_pay: [i64; 2],
    // Line 8z share of the prior-year recovery worksheets.
    pub(crate) recovery_8z: i64,
    // The part of `recovery_8z` that refunds real estate or personal property tax (cents).
    pub(crate) recovery_8z_taxes: i64,
    casualty_living_insurance: i64,
    // Relief payments not confirmed as excluded (section 139 or 139M).
    taxed_relief: i64,
    care_benefits: i64,
    injured_ltc: Vec<crate::care_benefit_return::InjuredLtcPart>,
    // Actual accepted LTC receipt interest, in cents, by physical source ID.
    // Periods-only records have a known zero; an absent ID is not a zero.
    injured_ltc_nii: alloc::collections::BTreeMap<i64, i64>,
    medicare_forms: Vec<crate::medical_account::MedicareForm>,
    hsa_distributions: [crate::medical_account::OwnedHsaAmounts; 2],
    archer_msa: Exact,
    archer_msa_owner: [Exact; 2],
    archer_penalized: Exact,
    inherited_medical: [Exact; 3],
    // Schedule 1 line 8z cents: earnings after an inherited account holder's death and a
    // returned Archer MSA excess.
    inherited_medical_earnings: Exact,
    archer_excess_earnings: Exact,
    pub(crate) tax_refund: i64,
    pub(crate) royalties: Exact,
    pub(crate) k1_royalty_gross: i64,
    pub(crate) k1_royalty_deductions: i64,
    pub(crate) k1_royalty_losses: i64,
    pub(crate) amt_royalty_adjustment: i64,
    // The part of `royalties` from Schedule K-1 (Form 1041) box 5, net of its
    // box 9 royalty deductions: Schedule E line 33 column (f), not Part I.
    estate_royalties: i64,
    estate_royalty_columns: [i64; 2],
    personal_rent: i64,
    investment_rent: [i64; 2],
    personal_rent_expenses: i64,
    personal_rent_depreciation: i64,
    business_depreciation_adjustment: i64,
    // Source cents before community allocation. Deduction compensation is
    // the worker's own income, independently of the joint/8958 AGI split.
    business_amt_adjustments: alloc::collections::BTreeMap<i64, i64>,
    scorp_loan_gain: [i64; 2],
    // Regular-tax cents of box 16 code D gain above stock basis from a
    // non-trading S corporation (the section 1411(c)(4) question).
    pub(crate) k1_business_gain: i64,
    stock_loss: i64,
    fund_tax_paid: i64,
    hobby: i64,
    personal_sale_loss: i64,
    amt_personal_sale_loss: i64,
    pub(crate) amt_sale_adjustment: [i64; 2],
    // K-1-owned capital net/gross AMT-minus-regular cents. Gross preserves
    // code versus own-interest event contributions and excludes K-3 codes.
    // Form 1116 replaces this net adjustment with the actual gross change.
    pub(crate) amt_k1_capital_net_adjustment: i64,
    pub(crate) amt_k1_capital_gross_adjustment: i64,
    pub(crate) amt_investment_gain_adjustment: [i64; 2],
    pub(crate) amt_collectible_adjustment: i64,
    pub(crate) collectible_long: i64,
    qsbs_excluded: i64,
    qsbs_amt_preference: i64,
    pub(crate) unrecaptured_long: i64,
    pub(crate) amt_unrecaptured_adjustment: i64,
    graduate_compensation: [i64; 2],
    jury: i64,
    jury_remitted: i64,
    taxable_grants: i64,
    family_leave: i64,
    pub(crate) gambling: i64,
    pub(crate) gambling_losses: i64,
    alimony: [i64; 2],
    alimony_compensation: [i64; 2],
    alimony_recapture_deduction: i64,
    // Same collector result, in its returned cents after native worksheet
    // rounding. The recipient owns this deduction; no payment preview or
    // second recapture invocation supplies Form8379's adjustment weights.
    injured_alimony_recapture: [i64; 2],
    pub(crate) disability: [i64; 2],
    pub(crate) church_wages: [i64; 2],
    pub(crate) social_wages: [i64; 2],
    pub(crate) social_tax: [i64; 2],
    pub(crate) tips: [i64; 2],
    pub(crate) overtime: [i64; 2],
    pub(crate) medicare_wages: [i64; 2],
    pub(crate) medicare_tax: [i64; 2],
    business: [i64; 2],
    // Actual operator net in source cents, before community allocation.
    // Schedule C/F rows and final source classification reuse this amount.
    business_profits: alloc::collections::BTreeMap<i64, i64>,
    business_sources:
        alloc::collections::BTreeMap<i64, [crate::community::BusinessSourceProjection; 2]>,
    business_se_exclusion: [i64; 2],
    // Actual non-wage amounts excluded by the source's SE owner, retained
    // only for admitted passive C. Section1411-9(a) does not exclude them
    // merely because they were omitted from1401 income.
    passive_business_niit: alloc::collections::BTreeMap<i64, i64>,
    interest: i64,
    pub(crate) early_penalty: i64,
    exempt: i64,
    pub(crate) dividends: i64,
    pub(crate) reit_dividends: i64,
    pub(crate) injured_reit_dividends: [Exact; 2],
    injured_reit_owner_unavailable: bool,
    qualified: i64,
    // Native activity K-1 source differences, before annual category rounding.
    // A wider exact difference prevents intermediate overflow before the
    // two annual nonnegative interest endpoints are checked (IRS i6251).
    k1_amt_interest: i128,
    k1_amt_interest_source: Option<crate::IssueContext>,
    k1_amt_dividends: i64,
    k1_amt_qualified: i64,
    pub(crate) short: i64,
    pub(crate) long: i64,
    injured_capital_sources: crate::injured_spouse::CapitalSources,
    injured_investment_sources: crate::injured_spouse::InvestmentSources,
    // Gains before losses of every Schedule D contributor, in cents: the
    // positive part of each amount `add_short`/`add_long` took (Form 1116
    // line 3e counts gains, not nets).
    pub(crate) capital_gross: i64,
    // Form 8949/Schedule D per-box gain totals (i8949 boxes A-F; i1040sd
    // lines 1a/1b/2/3 and 8a/8b/9/10): `FederalSale` lots, reported main-home
    // sales, QSBS (code Q), the section 1244 capital part (code S), worthless
    // securities, bad debts (box C) and return-of-capital gains. With the
    // lines 4 and 11 below, they foot to `short`/`long` before carryovers.
    sale_line_1a: i64,
    sale_line_1b: i64,
    sale_line_2: i64,
    pub(crate) sale_line_3: i64,
    // Farmer-only replacement of aggregate capital contributors by their
    // existing positive recognized source amounts; never changes Schedule D.
    farmer_capital_correction: Option<i64>,
    // Schedule D lines 4 and 11 in cents: Form 6252 line 26 of earlier-year
    // installment sales of investment property plus Form 6781 lines 8 and 9.
    schedule_d_line_4: i64,
    schedule_d_line_11: i64,
    // PRINT source traces only: admitted K-1 portfolio character and actual
    // fund/child-election distributions, retaining the same community shares.
    pub(crate) schedule_d_k1: [i64; 2],
    schedule_d_distributions: i64,
    sale_line_8a: i64,
    sale_line_8b: i64,
    sale_line_9: i64,
    pub(crate) sale_line_10: i64,
    // Schedule D columns (d), (e) and (g) of the eight lines above (`capital::SALE_LINES`), three
    // to a line: the totals of the Form 8949 rows and the summary lines of Exception 1.
    sale_columns: [i64; 24],
    // The positive Form 8949 row gains by holding period: the eligible gain a 2026 deferral
    // (code Z) can come from. Not a community-share amount.
    positive_rows: [i64; 2],
    // Gross loss items for research deduction proration and capital-gross test
    // footing: the losses `add_short`/`add_long` netted (each
    // contribution's negative part) and the gains that bypass line 3e on purpose (a K-1 with
    // a Schedule K-3), so `capital_gross == short + long + netted_losses - bypassed_gross`
    // holds after every exclusion and community share.
    netted_losses: i64,
    #[cfg(test)]
    bypassed_gross: i64,
    // The entry whose Form 8949 rows are being numbered and the last row number used.
    row_entry: (i64, u32),
    pub(crate) investment_gains: [i64; 2],
    retirement: i64,
    injured_retirement: [[i64; 2]; 2],
    nonqualified_annuity: i64,
    // Actual collector contributions to line13b for routed information
    // returns and Other income. None is a source still decided elsewhere.
    pub(crate) injured_other_sources: alloc::collections::BTreeMap<i64, Option<Exact>>,
    injured_income_components: Vec<crate::injured_spouse::income_categories::NativeIncomeComponent>,
    injured_unemployment: [i64; 2], // native net after same-year repayments, cents
    injured_accounts: crate::injured_spouse::AccountSources,
    injured_medical: Vec<crate::medical_account::MedicalSource>,
    pub(crate) injured_olympic: [i64; 2],
    injured_substitute: [i64; 2],
    // Actual K-1 interest/ordinary dividends/qualified dividends by owner,
    // retained after the ordinary community settlement, source cents.
    injured_k1_portfolio: [[i64; 2]; 3],
    injured_k1_nii: alloc::boxed::Box<crate::injured_spouse::K1NiiSources>,
    // Settled native Schedule2 line13h contributions by original K-1 owner, cents.
    pub(crate) injured_k1_deferred_tax: [i64; 2],
    injured_nonqualified_annuity: [i64; 2],
    injured_early_penalty: [i64; 2],
    retirement_gross: [i64; 2],
    retirement_taxable: [i64; 2],
    social: i64,
    unemployment: i64,
    pub(crate) withholding_w2: i64,
    withholding_1099: i64,
    withholding_other: i64,
    // Withholding of every kind by document owner (Form 8379 line 19).
    pub(crate) withholding_owner: [i64; 2],
    withholding_categories: [[i64; 2]; 3],
    native_withholding_sources: Option<
        alloc::boxed::Box<crate::injured_spouse::income_categories::NativeWithholdingSources>,
    >,
    private_activity: i64,
    estate_67e: i64,
    // Final-year box 11 code B investment interest, part of k1_interest.
    estate_investment_interest: i64,
    estate_final_personal_property_tax: i64,
    estate_final_real_estate_tax: i64,
    estate_final_state_income_tax: i64,
    estate_final_state_sales_tax: i64,
    estate_interest_ird_tax: i64,
    pub(crate) injured_niit_deduction_cents:
        alloc::boxed::Box<crate::injured_spouse::niit_deductions::NiitDeductionSourceCents>,
    estate_niit_line_7: i64,
    // Final-year section 642(h)(2) deductions allocable to NII, not already
    // included in the fiduciary's box 14 H adjustment (Form 8960 line 10).
    estate_niit_excess_deduction: i64,
    estate_niit_magi: i64,
    estate_amt_box12a: i64,
    // Raw A amounts routed through activity/independent owner lines instead.
    estate_amt_reallocated: i64,
    estate_amt_independent: i64,
    estate_amt_box12b: i64,
    estate_amt_box12c: i64,
    estate_amt_box12d: i64,
    // A community-property separate return (Form 8958): the cents the filer's
    // own wages exceed their community share, which earned income and IRA
    // compensation keep (Pub. 555 "Earned income credit", Pub. 590-A); and the
    // cents the community share of business profit differs from the profit
    // the operator's self-employment tax is figured on.
    community: bool,
    pub(crate) community_earned: i64,
    pub(crate) community_business: i64,
    // Unit B9-1 (k1_totals.rs), cents. Form 4797 Part II line 11 gains of the
    // Schedules K-1 (box 11 code R, the section 751 gain), regular and AMT:
    // Schedule 1 line 4, with the loan gains.
    pub(crate) k1_ordinary: [i64; 2],
    // Form 1116 line 3e counts gains before losses (i1116): the sum of the positive part of
    // each ordinary gain that reaches Schedule 1 line 4 (a Form 4797 Part II line 10 or 11
    // gain), whole, regular tax; and the net of the Schedule K-3 partners' box 11 (10) items
    // that sit in AGI outside Schedule D and Form 4797 (codes A, J, E, K), which their K-3
    // gross income replaces. Neither is a tax amount.
    pub(crate) ordinary_gain_gross: i64,
    pub(crate) k3_agi_items: i64,
    // The part of them that is investment income for Form 4952.
    pub(crate) k1_investment_ordinary: [i64; 2],
    // Schedule 1 line 8z items that are net investment income (Form 8960
    // line 7) and the other portfolio items that are Form 4952 line 4a income.
    pub(crate) k1_line_7: i64,
    pub(crate) k1_investment_income: i64,
    // Gains of a nonpassive business that is not trading: not net investment
    // income (Form 8960 line 5b), a negative amount.
    pub(crate) k1_nii_excluded: i64,
    // Schedule A contributions of the K-1 classes (B), (A) and (C) and Form
    // 4952 line 1 interest.
    pub(crate) k1_charity: [i64; 3],
    pub(crate) k1_charity_cash30: i64,
    pub(crate) k1_interest: [i64; 2],
    // Schedule 2 line 13h: interest and additional tax on deferred
    // compensation (box 20 code AI).
    pub(crate) k1_deferred_tax: i64,
}

// The Totals that carry income-tax amounts: a community-property return
// reports its share of each (Form 8958). Everything else is the person's own
// and is never divided: the employee's Social Security and Medicare wages, the
// operator's self-employment amounts, tips and overtime, and the accounts
// that belong to their owner (HSA, Archer MSA, ABLE, Trump account).
macro_rules! shared_totals {
    ($t:expr; ones: [$($one:ident),* $(,)?]; arrays: [$($array:ident),* $(,)?]; tests: [$($test:ident),* $(,)?]; lists: [$($list:ident),* $(,)?]; twos: [$($two:ident),* $(,)?]) => {{
        let t = $t;
        #[allow(unused_mut)]
        let mut ones: Vec<&mut i64> = alloc::vec![$(&mut t.$one),*];
        $(ones.extend(t.$array.iter_mut());)*
        $(#[cfg(test)] ones.push(&mut t.$test);)*
        (
            ones,
            alloc::vec![$(&mut t.$list),*],
            alloc::vec![$(&mut t.$two),*],
        )
    }};
}
type SharedFields<'a> = (Vec<&'a mut i64>, Vec<&'a mut [i64; 2]>, Vec<&'a mut [i64; 2]>);
// Source deltas and their final totals are distinct required native carriers.
fn checked_native_add(target: &mut i64, delta: i128) -> Option<()> {
    let delta = i64::try_from(delta).ok()?;
    *target = target.checked_add(delta)?;
    Some(())
}

impl Totals {
    // One Form 8949 row on its Schedule D line: columns (d), (e) and (g) and the line's
    // total of column (h), which feeds the gain totals the callers add on their own.
    fn add_row(&mut self, row: &crate::capital::Form8949Row) {
        self.try_add_row(row).expect("admitted native capital row totals");
    }
    fn try_add_row(&mut self, row: &crate::capital::Form8949Row) -> Option<()> {
        self.try_add_printed_row(row)?;
        let gain = row.checked_gain()?;
        if gain > 0 {
            checked_native_add(
                &mut self.positive_rows[usize::from(row.line >= 4)],
                i128::from(gain),
            )?;
        }
        Some(())
    }
    // Projection-only rows cannot admit new sources to the existing QOF
    // deferral pool. Financial admission stays with add_row's original callers.
    fn add_printed_row(&mut self, row: &crate::capital::Form8949Row) {
        self.try_add_printed_row(row).expect("admitted printed capital row totals");
    }
    fn try_add_printed_row(&mut self, row: &crate::capital::Form8949Row) -> Option<()> {
        checked_native_add(
            match row.line {
                0 => &mut self.sale_line_1a,
                1 => &mut self.sale_line_1b,
                2 => &mut self.sale_line_2,
                3 => &mut self.sale_line_3,
                4 => &mut self.sale_line_8a,
                5 => &mut self.sale_line_8b,
                6 => &mut self.sale_line_9,
                _ => &mut self.sale_line_10,
            },
            i128::from(row.checked_gain()?),
        )?;
        for (column, amount) in [row.proceeds, row.basis, row.adjustment].into_iter().enumerate() {
            checked_native_add(&mut self.sale_columns[row.line * 3 + column], i128::from(amount))?;
        }
        Some(())
    }
    // Schedule D columns (d) and (e) of a K-1 disposition's Form 8949 box C or F row (its line
    // total is added by the caller): `line` is the index in `capital::SALE_LINES`.
    pub(crate) fn add_columns(&mut self, line: usize, proceeds: i64, basis: i64) {
        self.sale_columns[line * 3] += proceeds;
        self.sale_columns[line * 3 + 1] += basis;
    }
    // The next Form 8949 row number of an entry (numbered from 1 for each entry).
    fn next_row(&mut self, entry: i64) -> u32 {
        self.row_entry =
            if self.row_entry.0 == entry { (entry, self.row_entry.1 + 1) } else { (entry, 1) };
        self.row_entry.1
    }
    // A row of an entry whose amounts are not a lot of FederalSale: numbered and added.
    fn row(
        &mut self,
        r: &mut FederalResult,
        entry: &FederalIncomeEntry,
        line: usize,
        box_letter: char,
        amounts: [i64; 3],
        codes: &str,
    ) {
        let row = crate::capital::Form8949Row {
            entry: entry.id,
            row: self.next_row(entry.id),
            line,
            box_letter,
            label: crate::interview::income_display_arg(entry),
            proceeds: amounts[0],
            basis: amounts[1],
            adjustment: amounts[2],
            codes: codes.into(),
            summary: false,
        };
        row.emit(r);
        self.add_row(&row);
    }
    // The personal-use part of a business asset's sale: a gain is a capital
    // gain (Form 8949 box C or F by the holding period, and investment
    // income), a loss is not deductible (i4797 "Sale of Home Used for
    // Business"; Pub 544, "Property Used Partly for Business or Rental").
    fn personal_part(
        &mut self,
        r: &mut FederalResult,
        entry: i64,
        label: crate::MessageArg,
        sale: Option<crate::asset_sale::AssetGain>,
    ) {
        let Some(sale) = sale.filter(|sale| sale.personal > 0) else {
            return;
        };
        self.property_row(
            r,
            entry,
            label,
            sale.long,
            [sale.personal_proceeds, sale.personal_basis, 0],
            true,
        );
    }
    // One Form 8949 row for the gain on property outside a trade or business:
    // proceeds and cost in whole dollars, box F (long term) or C, no code.
    fn property_row(
        &mut self,
        r: &mut FederalResult,
        entry: i64,
        label: crate::MessageArg,
        long: bool,
        amounts: [i64; 3],
        investment: bool,
    ) {
        let gain = (amounts[0] - amounts[1] + amounts[2]) * 100;
        let (line, box_letter) = if long { (7, 'F') } else { (3, 'C') };
        if long {
            self.add_long(gain);
        } else {
            self.add_short(gain);
        }
        if investment {
            self.investment_gains[usize::from(long)] += gain;
        }
        let row = crate::capital::Form8949Row {
            entry,
            row: self.next_row(entry),
            line,
            box_letter,
            label,
            proceeds: amounts[0] * 100,
            basis: amounts[1] * 100,
            adjustment: amounts[2] * 100,
            codes: String::new(),
            summary: false,
        };
        row.emit(r);
        self.add_row(&row);
    }
    fn try_add_capital(&mut self, amount: i64, long: bool) -> Option<()> {
        self.try_add_k1_capital(amount, long, false)
    }
    pub(crate) fn try_add_k1_capital(
        &mut self,
        amount: i64,
        long: bool,
        outside_line_3e: bool,
    ) -> Option<()> {
        checked_native_add(
            if long { &mut self.long } else { &mut self.short },
            i128::from(amount),
        )?;
        if !outside_line_3e {
            checked_native_add(&mut self.capital_gross, i128::from(amount).max(0))?;
        }
        #[cfg(test)]
        if outside_line_3e {
            checked_native_add(&mut self.bypassed_gross, i128::from(amount).max(0))?;
        }
        checked_native_add(&mut self.netted_losses, (-i128::from(amount)).max(0))?;
        i64::try_from(i128::from(self.short) + i128::from(self.long)).ok()?;
        Some(())
    }
    fn capital_endpoints_fit(&self) -> Option<()> {
        let mut amt_net = 0_i128;
        for term in 0..2 {
            let amount = i128::from(if term == 0 { self.short } else { self.long })
                + i128::from(self.amt_sale_adjustment[term]);
            i64::try_from(amount).ok()?;
            amt_net += amount;
            i64::try_from(
                i128::from(self.investment_gains[term])
                    + i128::from(self.amt_investment_gain_adjustment[term]),
            )
            .ok()?;
        }
        i64::try_from(amt_net).ok()?;
        i64::try_from(
            i128::from(self.collectible_long) + i128::from(self.amt_collectible_adjustment),
        )
        .ok()?;
        i64::try_from(
            i128::from(self.unrecaptured_long) + i128::from(self.amt_unrecaptured_adjustment),
        )
        .ok()?;
        Some(())
    }
    // Check every signed native source before changing any annual capital owner.
    fn native_entity_portfolio_fits(
        &self,
        regular: &crate::k1_activity::ordinary::Output,
        alternative: &crate::k1_activity::ordinary::Output,
    ) -> Result<(), crate::validation::ValidationIssue> {
        let interest = [regular.entity_interest, alternative.entity_interest];
        let regular_interest = self.interest.checked_add(interest[0]);
        let interest_fits = regular_interest.and_then(|regular_interest| {
            i64::try_from(
                i128::from(regular_interest) + self.k1_amt_interest + i128::from(interest[1])
                    - i128::from(interest[0]),
            )
            .ok()
        });
        if interest_fits.is_none() {
            return Err(crate::validation::ValidationIssue::new(
                crate::k1_activity::ordinary::RANGE_KEY,
            )
            .at_field("interest")
            .in_field("portfolio")
            .in_field("entity_only")
            .in_field("activities"));
        }
        let mut net = [self.short, self.long];
        let mut gross = self.capital_gross;
        let mut losses = self.netted_losses;
        let mut schedule = self.schedule_d_k1;
        let mut investment = self.investment_gains;
        let mut amt = self.amt_sale_adjustment;
        let mut amt_investment = self.amt_investment_gain_adjustment;
        let mut amt_net = self.amt_k1_capital_net_adjustment;
        let mut amt_gross = self.amt_k1_capital_gross_adjustment;
        for (index, ((regular, regular_income), (alternative, alternative_income))) in regular
            .capital_sources()
            .zip(regular.capital_income())
            .zip(alternative.capital_sources().zip(alternative.capital_income()))
            .enumerate()
        {
            // An absent capital contribution must not gate another source's
            // existing totals. Equal gain and released loss still has gross income.
            if regular == [0; 2]
                && alternative == [0; 2]
                && regular_income == [0; 2]
                && alternative_income == [0; 2]
            {
                continue;
            }
            let fits = (|| {
                for term in 0..2 {
                    let gain = regular[term];
                    let adjustment = alternative[term].checked_sub(gain)?;
                    let loss = regular_income[term].checked_sub(gain)?;
                    net[term] = net[term].checked_add(regular_income[term])?.checked_sub(loss)?;
                    gross = gross.checked_add(regular_income[term])?;
                    losses = losses.checked_add(loss)?;
                    schedule[term] = schedule[term].checked_add(gain)?;
                    investment[term] = investment[term].checked_add(gain)?;
                    amt[term] = amt[term].checked_add(adjustment)?;
                    amt_investment[term] = amt_investment[term].checked_add(adjustment)?;
                    amt_net = amt_net.checked_add(adjustment)?;
                    amt_gross = amt_gross
                        .checked_add(alternative_income[term].checked_sub(regular_income[term])?)?;
                }
                net[0].checked_add(net[1])?.checked_neg()?;
                investment[0].checked_add(investment[1])?;
                amt[0].checked_add(amt[1])?;
                amt_investment[0].checked_add(amt_investment[1])?;
                gross.checked_add(amt_gross)?;
                i64::try_from(
                    i128::from(net[0])
                        + i128::from(net[1])
                        + i128::from(amt[0])
                        + i128::from(amt[1]),
                )
                .ok()?;
                Some(())
            })();
            if fits.is_none() {
                let issue = crate::validation::ValidationIssue::new(
                    crate::k1_activity::ordinary::RANGE_KEY,
                );
                return Err(if index == 0 {
                    issue.at_field("portfolio").in_field("entity_only").in_field("activities")
                } else {
                    issue
                        .in_list("components", index - 1)
                        .in_field("source")
                        .in_field("statement")
                        .in_field("activities")
                });
            }
        }
        Ok(())
    }
    fn try_bypass_line_3e(&mut self, gross: i64) -> Option<()> {
        checked_native_add(&mut self.capital_gross, -i128::from(gross))?;
        #[cfg(test)]
        checked_native_add(&mut self.bypassed_gross, i128::from(gross))?;
        Some(())
    }
    // Called by the existing income owner, never by a document preview.
    // Only an explicitly registered source participates, so an unrelated
    // collect_8z call cannot create an incomplete source projection.
    pub(crate) fn injured_other_amount(&mut self, id: i64, amount: Option<i64>) {
        if let Some(total) = self.injured_other_sources.get_mut(&id) {
            *total = total.take().zip(amount).map(|(before, added)| before + exact::whole(added));
        }
    }
    // Retain an actual accepted entry's native source and its independent
    // legacy Rest contribution. Late non-entry owners use no fabricated ID.
    // Attribution only: the native and legacy scalar owners are unchanged.
    pub(crate) fn record_income_component(
        &mut self,
        id: i64,
        owner: usize,
        category: crate::injured_spouse::income_categories::IncomeCategory,
        source_cents: Option<i64>,
        rest_cents: Option<i64>,
    ) {
        self.record_income_component_exact(
            id,
            owner,
            category,
            source_cents.map(exact::whole),
            rest_cents.map(exact::whole),
        );
    }
    pub(crate) fn record_income_component_exact(
        &mut self,
        id: i64,
        owner: usize,
        category: crate::injured_spouse::income_categories::IncomeCategory,
        source_cents: Option<Exact>,
        rest_cents: Option<Exact>,
    ) {
        if let Some(component) = self.injured_income_components.iter_mut().find(|component| {
            component.id == id && component.owner == owner && component.category == category
        }) {
            component.source_cents =
                component.source_cents.take().zip(source_cents).map(|(a, b)| a + b);
            component.rest_cents = component.rest_cents.take().zip(rest_cents).map(|(a, b)| a + b);
        } else {
            self.injured_income_components.push(
                crate::injured_spouse::income_categories::NativeIncomeComponent {
                    id,
                    owner,
                    category,
                    source_cents,
                    rest_cents,
                },
            );
        }
    }

    // Heap-backed frame: adding native boundaries does not put an array of
    // category records in the already large collector/estimate stack frames.
    #[inline(never)]
    fn income_category_snapshot(
        &self,
    ) -> Vec<(crate::injured_spouse::income_categories::IncomeCategory, Exact)> {
        use crate::injured_spouse::income_categories::IncomeCategory as IC;
        alloc::vec![
            (IC::HouseholdWages, self.household_wages.iter().sum()),
            (
                IC::MedicaidWaiverWages,
                self.medicaid_taxable.iter().sum::<i64>()
                    + self.medicaid_excluded.iter().sum::<i64>()
            ),
            (IC::StrikeBenefits, self.strike_benefits.iter().sum()),
            (IC::TaxableRefunds, self.tax_refund),
            (IC::AlaskaDividends, self.alaska),
            (IC::Prizes, self.prizes),
            (IC::NotForProfit, self.not_for_profit),
            (IC::OlympicPrizes, self.olympic),
            (
                IC::MedicaidWaiverExclusion,
                -self.medicaid_excluded.iter().sum::<i64>()
                    - self.medicaid_in_box1.iter().sum::<i64>()
            ),
            (IC::InmatePay, self.inmate_pay.iter().sum()),
            (IC::DigitalAssets, self.digital),
            (IC::Other8z, self.other_8z + self.recovery_8z),
            (IC::EmployeeTips, self.employee_tips.iter().map(|v| v.income).sum()),
            (IC::EmployeeWages, self.employee_wages.iter().sum()),
            (IC::UncoveredEmployeeWages, self.employee_wages_not_covered.iter().sum()),
            (IC::DisabilityPension, self.disability_pension.iter().sum()),
            (IC::IsoCompensation, self.iso_unreported.iter().sum()),
            (IC::Alimony, self.alimony.iter().sum()),
            (IC::Gambling, self.gambling),
            (IC::PersonalDebt, self.personal_debt),
            (IC::Hobby, self.hobby),
            (IC::Scholarship, self.scholarship),
            (IC::Grants, self.taxable_grants),
            (IC::FamilyLeave, self.family_leave),
            (IC::CasualtyLivingInsurance, self.casualty_living_insurance),
            (IC::TaxedRelief, self.taxed_relief),
            (IC::HealthLoanAssistance, self.health_assistance),
            (IC::TrumpDeath, self.trump_death),
            (IC::Royalties, 0),
            (IC::ScorpLoanGain, self.scorp_loan_gain[0]),
            (IC::K1OrdinaryGain, self.k1_ordinary[0]),
            (IC::Interest, self.interest),
            (IC::OrdinaryDividends, self.dividends),
        ]
        .into_iter()
        .map(|(category, amount)| {
            let amount = if category == IC::Royalties {
                self.royalties.clone()
            } else {
                exact::whole(amount)
            };
            (category, amount)
        })
        .collect()
    }

    #[inline(never)]
    fn retain_income_categories(&mut self, previous: Option<NativeIncomeFrame>) {
        let Some(previous) = previous else { return };
        // A shared/unresolved source is resolved only by its owning later
        // worksheet. A zero native increment is not an answer to that source.
        if self.injured_other_sources.get(&previous.id) == Some(&None) {
            return;
        }
        for ((category, before), (_, after)) in
            previous.before.into_iter().zip(self.income_category_snapshot())
        {
            let amount = after - before;
            if amount != exact::whole(0) {
                self.record_income_component_exact(
                    previous.id,
                    previous.owner,
                    category,
                    Some(amount.clone()),
                    Some(if previous.rest.includes(category) { amount } else { exact::whole(0) }),
                );
            }
        }
    }

    // Single amounts, then amounts kept by something other than the owner (a
    // holding term, an IRA or pension), then amounts kept by owner.
    fn shared(&mut self) -> SharedFields<'_> {
        shared_totals!(self;
            ones: [
                scholarship, personal_debt, health_loan_interest, prizes,
                casualty_living_insurance, taxed_relief, tax_refund,
                k1_royalty_gross, k1_royalty_deductions, k1_royalty_losses, amt_royalty_adjustment,
                personal_rent, personal_rent_expenses, personal_rent_depreciation,
                business_depreciation_adjustment, k1_business_gain,
                hobby, jury, jury_remitted, taxable_grants,
                family_leave, gambling, gambling_losses, alimony_recapture_deduction,
                interest, early_penalty, exempt, dividends, reit_dividends, qualified,
                retirement, nonqualified_annuity, social,
                unemployment, withholding_w2, withholding_1099, withholding_other,
                private_activity, health_assistance, alaska, olympic,
                digital, other_8z, substitute, not_for_profit, not_income_1099k,
            ];
            arrays: [];
            tests: [];
            lists: [investment_rent, retirement_gross, retirement_taxable];
            twos: [
                wages, employee_wages, iso_unreported, iso_amt_wage_adjustment,
                disability_pension, moving_excess, graduate_compensation, alimony,
                alimony_compensation, injured_alimony_recapture, disability, deferral_excess, deferred_pension,
                inmate_pay, household_wages, strike_benefits, medicaid_taxable, medicaid_excluded, medicaid_in_box1, statutory_profit, employee_wages_not_covered,
                injured_nonqualified_annuity, injured_early_penalty, injured_olympic, injured_substitute,
            ]
        )
    }
    // Pub. 555: property gains follow ownership even when the property's
    // income follows a different community-property fraction.
    fn shared_capital(&mut self) -> SharedFields<'_> {
        shared_totals!(self;
            ones: [
                stock_loss, fund_tax_paid, personal_sale_loss, amt_personal_sale_loss,
                amt_k1_capital_net_adjustment, amt_k1_capital_gross_adjustment,
                amt_collectible_adjustment, collectible_long, qsbs_excluded,
                qsbs_amt_preference, unrecaptured_long, amt_unrecaptured_adjustment,
                short, long, capital_gross, netted_losses,
                sale_line_1a, sale_line_1b, sale_line_2, sale_line_3,
                schedule_d_line_4, schedule_d_line_11, schedule_d_distributions,
                sale_line_8a, sale_line_8b, sale_line_9, sale_line_10, nua_excluded_gain,
            ];
            arrays: [sale_columns, schedule_d_k1];
            tests: [bypassed_gross];
            lists: [scorp_loan_gain, amt_sale_adjustment, amt_investment_gain_adjustment, investment_gains];
            twos: []
        )
    }
    fn shared_values(&mut self) -> (Vec<i64>, Vec<[i64; 2]>, Vec<[i64; 2]>) {
        let (ones, lists, twos) = self.shared();
        (
            ones.into_iter().map(|v| *v).collect(),
            lists.into_iter().map(|v| *v).collect(),
            twos.into_iter().map(|v| *v).collect(),
        )
    }
}

// Whether an existing legacy Rest source actually contains this category.
// This classifies participation only; no monetary source is recomputed here.
#[derive(Clone, Copy)]
enum NativeRestSource {
    All,
    WageTips,
    EmployeeWages,
    None,
}
impl NativeRestSource {
    fn for_entry(entry: &FederalIncomeEntry) -> Self {
        match &entry.income {
            FederalIncome::Wage { .. } => Self::WageTips,
            FederalIncome::EmployeeWages { .. } => Self::EmployeeWages,
            FederalIncome::OtherIncome { .. }
            | FederalIncome::Nec1099 { .. }
            | FederalIncome::Misc1099 { .. }
            | FederalIncome::GovernmentPayment { .. }
            | FederalIncome::Partnership { .. }
            | FederalIncome::Scorp { .. }
            | FederalIncome::EstateTrustPortfolio { .. }
            | FederalIncome::Royalty { .. }
            | FederalIncome::Gambling { .. }
            | FederalIncome::PersonalDebt { .. }
            | FederalIncome::HobbySales { .. }
            | FederalIncome::Scholarship { .. }
            | FederalIncome::HealthLoanAssistance { .. }
            | FederalIncome::WildfireRelief { .. }
            | FederalIncome::DisasterRelief { .. }
            | FederalIncome::CasualtyLivingInsurance { .. }
            | FederalIncome::TrumpDeath { .. }
            | FederalIncome::Alimony { .. } => Self::All,
            _ => Self::None,
        }
    }
    fn includes(self, category: crate::injured_spouse::income_categories::IncomeCategory) -> bool {
        use crate::injured_spouse::income_categories::IncomeCategory as IC;
        match self {
            Self::All => true,
            Self::WageTips => category == IC::EmployeeTips,
            Self::EmployeeWages => {
                matches!(category, IC::EmployeeWages | IC::UncoveredEmployeeWages)
            }
            Self::None => false,
        }
    }
}
struct NativeIncomeFrame {
    id: i64,
    owner: usize,
    rest: NativeRestSource,
    before: Vec<(crate::injured_spouse::income_categories::IncomeCategory, Exact)>,
}

type EmployeeOffsets = ([i64; 2], [i64; 2], [i64; 2], [i64; 3]);

// One income item whose community share is applied when its amounts have
// entered the totals (`settle`): the values before it, and who it belongs to.
struct PendingShare {
    entry: usize,
    owner: usize,
    royalties: Exact,
    share: crate::community::Share,
    // The share the item's withholding divides by (community::withholding_share).
    withholding_share: crate::community::Share,
    ones: Vec<i64>,
    lists: Vec<[i64; 2]>,
    twos: Vec<[i64; 2]>,
    gains: Vec<i64>,
    gain_lists: Vec<[i64; 2]>,
    fund: Option<(i64, Option<crate::FundQsbsExclusion>)>,
    business: [i64; 2],
    uncollected_medicare: i64,
    ordinary_uncollected_medicare: i64,
    care_benefits: i64,
    withheld: i64,
    // The three withholding totals before the item.
    withholding: [i64; 3],
    employee_offsets: Option<EmployeeOffsets>,
}
impl PendingShare {
    fn new(
        t: &mut Totals,
        entry: usize,
        owner: usize,
        share: crate::community::Share,
        withholding_share: crate::community::Share,
    ) -> Self {
        let (ones, lists, twos) = t.shared_values();
        let (gains, gain_lists, _) = t.shared_capital();
        let gains = gains.into_iter().map(|value| *value).collect();
        let gain_lists = gain_lists.into_iter().map(|value| *value).collect();
        Self {
            entry,
            owner,
            royalties: t.royalties.clone(),
            share,
            withholding_share,
            ones,
            lists,
            twos,
            gains,
            gain_lists,
            fund: None,
            business: t.business,
            uncollected_medicare: t.uncollected_medicare,
            ordinary_uncollected_medicare: t.ordinary_uncollected_medicare,
            care_benefits: t.care_benefits,
            withheld: t.withholding_w2 + t.withholding_1099 + t.withholding_other,
            withholding: [t.withholding_w2, t.withholding_1099, t.withholding_other],
            employee_offsets: None,
        }
    }
    // Keep one employee source's unallocated income separate from prior settled
    // income. A valid filer share can fit even when raw household sums do not.
    fn stage_employee(&mut self, t: &mut Totals) {
        self.employee_offsets = Some((
            t.wages,
            t.employee_wages,
            t.employee_wages_not_covered,
            [t.withholding_w2, t.withholding_1099, t.withholding_other],
        ));
        t.wages = [0; 2];
        t.employee_wages = [0; 2];
        t.employee_wages_not_covered = [0; 2];
        t.withholding_w2 = 0;
        t.withholding_1099 = 0;
        t.withholding_other = 0;
        (self.ones, self.lists, self.twos) = t.shared_values();
        self.withheld = 0;
        self.withholding = [0; 3];
    }
    // Apply the share to what the item added: this return reports the filer's
    // part of every income amount and of its withholding; a spouse's own
    // payroll and self-employment amounts never join this return.

    fn settle_with_claims(
        self,
        t: &mut Totals,
        d: &FederalDraft,
        e: &FederalIncomeEntry,
        r: &mut FederalResult,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
    ) {
        let Self {
            owner,
            royalties,
            share,
            withholding_share,
            ones,
            lists,
            twos,
            gains,
            gain_lists,
            fund,
            business,
            uncollected_medicare,
            ordinary_uncollected_medicare,
            care_benefits,
            withheld,
            withholding,
            employee_offsets,
            ..
        } = self;
        let spouse = owner == 1;
        let full_withheld = t.withholding_w2 + t.withholding_1099 + t.withholding_other - withheld;
        let full_wages = t.wages[0] - twos[0][0];
        let full_withholding = [t.withholding_w2, t.withholding_1099, t.withholding_other];
        {
            let (fields, kept, pairs) = t.shared();
            for (field, before) in fields.into_iter().zip(&ones) {
                *field = before + share.filer(*field - before);
            }
            for (pair, before) in kept.into_iter().zip(&lists) {
                *pair = [
                    before[0] + share.filer(pair[0] - before[0]),
                    before[1] + share.filer(pair[1] - before[1]),
                ];
            }
            for (pair, before) in pairs.into_iter().zip(&twos) {
                let (own, spouses) = (pair[0] - before[0], pair[1] - before[1]);
                *pair = if spouse {
                    [before[0] + own + share.filer(spouses), before[1]]
                } else {
                    [before[0] + share.filer(own), before[1] + spouses]
                };
            }
        }
        {
            let (fields, lists, _) = t.shared_capital();
            for (field, before) in fields.into_iter().zip(gains) {
                *field = before + share.gain(*field - before);
            }
            for (pair, before) in lists.into_iter().zip(gain_lists) {
                for (value, old) in pair.iter_mut().zip(before) {
                    *value = old + share.gain(*value - old);
                }
            }
        }
        if let Some((gain, exclusion)) = fund {
            fund_section_1202(t, r, e, gain, exclusion, Some(share), separate_return(d));
        }
        t.royalties = &royalties + share.filer_exact(&(&t.royalties - &royalties));
        // The tax withheld follows the income it was withheld from.
        if withholding_share != share {
            let fields = [&mut t.withholding_w2, &mut t.withholding_1099, &mut t.withholding_other];
            for ((field, before), full) in fields.into_iter().zip(withholding).zip(full_withholding)
            {
                *field = before + withholding_share.filer(full - before);
            }
        }
        let this_withheld = t.withholding_w2 + t.withholding_1099 + t.withholding_other - withheld;
        // Wages: the filer's own pay stays earned income and IRA
        // compensation whatever the share; a spouse's pay is neither.
        let scaled_wages = t.wages[0] - twos[0][0];
        t.community_earned += if spouse { 0 } else { full_wages } - scaled_wages;
        // Business profit: the operator's self-employment tax is on the whole
        // profit, income tax on the share.
        let profit = [t.business[0] - business[0], t.business[1] - business[1]];
        if let Some(source) = t.business_sources.get(&e.id) {
            t.community_business +=
                source[0].income.filer - if spouse { 0 } else { source[0].operator_earnings };
            t.business_depreciation_adjustment += source[1].income.filer
                - source[0].income.filer
                - share.filer(t.business_amt_adjustments.get(&e.id).copied().unwrap_or(0));
        } else {
            t.community_business +=
                if spouse { share.filer(profit[1]) } else { share.filer(profit[0]) - profit[0] };
        }
        if spouse {
            t.community_counterpart_payroll[0] += t.social_wages[1];
            t.community_counterpart_payroll[1] += t.church_wages[1];
            t.business = business;
            t.business_se_exclusion[1] = 0;
            t.social_wages[1] = 0;
            t.social_tax[1] = 0;
            t.medicare_wages[1] = 0;
            t.medicare_tax[1] = 0;
            t.church_wages[1] = 0;
            t.tips[1] = 0;
            t.overtime[1] = 0;
            t.uncollected_social[1] = 0;
            t.uncollected_medicare_owners[1] = 0;
            t.employee_tips[1] = Default::default();
            t.employee_8919[1] = 0;
            t.employee_payroll_social[1] = 0;
            t.employee_payroll_medicare[1] = 0;
            t.employee_other_social[1] = 0;
            t.employee_other_medicare[1] = 0;
            t.employee_unknown_social[1] = 0;
            t.employee_unknown_medicare[1] = 0;
            t.employee_rrta_compensation_unknown[1] = false;
            t.employee_rrta_withholding_unknown[1] = false;
            t.uncollected_medicare = uncollected_medicare;
            t.ordinary_uncollected_medicare = ordinary_uncollected_medicare;
            t.ordinary_uncollected_medicare_owners[1] = 0;
            t.rrta_compensation[1] = 0;
            t.rrta_medicare[1] = 0;
            t.rrta_additional_withheld[1] = 0;
            t.rrta_tier_2_forms[1] = 0;
            t.combat_pay[1] = 0;
            t.statutory_gross[1] = 0;
            t.care_benefits = care_benefits;
        }
        if d.setup.as_ref().is_some_and(crate::community::applies) {
            t.community = true;
            note_once(r, msg!("NOTE_FORM_8958_ALLOCATION"));
            // Form 8958: this item's total and this return's share.
            let label = crate::interview::income_display_arg(e);
            // An amount a Form 1099 sent to a business, rental, royalty, equipment
            // or hobby record is that record's row, not a second item (Form 8958
            // lists each item of income once).
            let routed = if matches!(
                e.income,
                FederalIncome::Nec1099 { .. }
                    | FederalIncome::Misc1099 { .. }
                    | FederalIncome::K1099 { .. }
            ) {
                crate::information_returns::routed_from_with_claims(d, e, source_claims)
            } else {
                0
            };
            // A community share exists only on a separate return (community::share).
            let business_source = t.business_sources.get(&e.id).map(|rows| rows[0]);
            let native_amount = match business_source {
                Some(source) => source.income.whole,
                None => {
                    crate::interview::document_amount(&e.income, true).map(|amount| amount - routed)
                }
            };
            // Per-person exclusions can make a whole-document cap differ from
            // the income the two spouses actually report. Form 8958 columns foot
            // to those same allocated source amounts (instructions, page 4).
            let financial = matches!(
                &e.income,
                FederalIncome::Dividend { .. } | FederalIncome::UndistributedGain { .. }
            );
            let financial_amounts = if financial {
                native_amount.and_then(|amount| {
                    crate::interview::allocated_preview_amounts(e, amount, share, true)
                })
            } else {
                None
            };
            let native_amount = if financial {
                financial_amounts.and_then(crate::kernel::ratio::checked_sum)
            } else {
                native_amount
            };
            if let Some(source) = business_source.filter(|source| source.income.whole.is_none()) {
                line_cents(
                    r,
                    &alloc::format!("form8958_{}_this_return", e.id),
                    source.income.filer,
                    &["income", "setup.community_property"],
                    "https://www.irs.gov/pub/irs-pdf/f8958.pdf",
                    Message::new("EXPLAIN_FORM8958_N_THIS_RETURN", alloc::vec![label.clone()]),
                );
                note_once_scoped(
                    r,
                    msg!("WARN_FARM_CONSERVATION_CARRY_SOURCE"),
                    crate::interview::income_issue_context(e),
                );
            }
            if matches!(e.income, FederalIncome::Royalty { .. }) && native_amount.is_none() {
                crate::extended_rules::expense_output_unavailable(r, Some(e.id));
                return;
            }
            if let Some(amount) = native_amount
                && amount != 0
            {
                let id = e.id;
                line_cents(
                    r,
                    &alloc::format!("form8958_{id}_total"),
                    amount,
                    &["income", "setup.community_property"],
                    "https://www.irs.gov/pub/irs-pdf/f8958.pdf",
                    Message::new("EXPLAIN_FORM8958_N_TOTAL", alloc::vec![label.clone()]),
                );
                let selected = if financial {
                    financial_amounts.map(|parts| parts[0])
                } else {
                    business_source.map_or_else(
                        || crate::interview::community_amount(d, e, Some(amount)),
                        |source| Some(source.income.filer),
                    )
                };
                if let Some(selected) = selected {
                    line_cents(
                        r,
                        &alloc::format!("form8958_{id}_this_return"),
                        selected,
                        &["income", "setup.community_property"],
                        "https://www.irs.gov/publications/p555",
                        Message::new("EXPLAIN_FORM8958_N_THIS_RETURN", alloc::vec![label.clone()]),
                    );
                }
            }
            if full_withheld != 0 {
                let id = e.id;
                line_cents(
                    r,
                    &alloc::format!("form8958_withholding_{id}_total"),
                    full_withheld,
                    &["income", "setup.community_property"],
                    "https://www.irs.gov/pub/irs-pdf/f8958.pdf",
                    Message::new(
                        "EXPLAIN_FORM8958_WITHHOLDING_N_TOTAL",
                        alloc::vec![label.clone()],
                    ),
                );
                line_cents(
                    r,
                    &alloc::format!("form8958_withholding_{id}_this_return"),
                    this_withheld,
                    &["income", "setup.community_property"],
                    "https://www.irs.gov/publications/p555",
                    Message::new("EXPLAIN_FORM8958_WITHHOLDING_N_THIS_RETURN", alloc::vec![label]),
                );
            }
        }
        if let Some((wages, employee, other, withholding)) = employee_offsets {
            let fields = [&mut t.wages, &mut t.employee_wages, &mut t.employee_wages_not_covered];
            for (field, prior) in fields.into_iter().zip([wages, employee, other]) {
                for (amount, old) in field.iter_mut().zip(prior) {
                    let Some(merged) = amount.checked_add(old) else {
                        crate::issue_localized!(
                            r,
                            &crate::interview::income_issue_context(e),
                            "Employee payroll",
                            "EMPLOYEE_PAYROLL_RANGE",
                            "The employee income aggregate exceeds its native range."
                        );
                        return;
                    };
                    *amount = merged;
                }
            }
            for (amount, old) in
                [&mut t.withholding_w2, &mut t.withholding_1099, &mut t.withholding_other]
                    .into_iter()
                    .zip(withholding)
            {
                let Some(merged) = amount.checked_add(old) else {
                    crate::issue_localized!(
                        r,
                        &crate::interview::income_issue_context(e),
                        "Employee payroll",
                        "EMPLOYEE_PAYROLL_RANGE",
                        "The employee withholding aggregate exceeds its native range."
                    );
                    return;
                };
                *amount = merged;
            }
        }
    }
}

// Keep construction and clone return slots out of the long-lived estimate
// frames, including debug builds where Box::new(value) leaves value on stack.
#[inline(never)]
pub(crate) fn boxed<T>(build: impl FnOnce() -> T) -> alloc::boxed::Box<T> {
    alloc::boxed::Box::new(build())
}

#[inline(never)]
fn try_boxed<T, E>(build: impl FnOnce() -> Result<T, E>) -> Result<alloc::boxed::Box<T>, E> {
    build().map(alloc::boxed::Box::new)
}

enum CollectFailure {
    OpportunityFundEvent(crate::IssueContext),
    Section179(crate::Section179PreparedIssue),
    Medical(crate::medical_account::OutputFailure),
    Broker(crate::IssueContext),
}
impl From<crate::Section179PreparedIssue> for CollectFailure {
    fn from(issue: crate::Section179PreparedIssue) -> Self {
        Self::Section179(issue)
    }
}
impl From<crate::medical_account::OutputFailure> for CollectFailure {
    fn from(issue: crate::medical_account::OutputFailure) -> Self {
        Self::Medical(issue)
    }
}
impl CollectFailure {
    fn report(self, r: &mut FederalResult) {
        match self {
            Self::OpportunityFundEvent(context) => {
                crate::issue_localized!(
                    r,
                    &context,
                    "Opportunity fund investment",
                    "QOF_EVENT_NEEDED",
                    "Tell us what happened to this opportunity fund investment during the return year."
                );
                r.state = ResultState::Unsupported;
                r.financial_source_scope_complete = false;
            }
            Self::Section179(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
            }
            Self::Medical(issue) => issue.report(r),
            Self::Broker(context) => {
                crate::issue_localized!(
                    r,
                    &context,
                    "Broker financial sources",
                    "BROKER_SALE_TOTALS_ARITHMETIC_UNAVAILABLE",
                    "The actual broker sources cannot fit a required native financial total. Retain the source records; this estimate is unavailable."
                );
                r.state = ResultState::Unsupported;
                r.financial_source_scope_complete = false;
            }
        }
    }
}

// Form 6781 remains the single contract owner. This adapter adds its actual
// regular/AMT outputs to the existing native return totals with checked carriers.
fn collect_contracts(d: &FederalDraft, t: &mut Totals) -> Result<(), CollectFailure> {
    let values = (|| -> Option<()> {
        let contracts = crate::section1256::form_6781(d);
        let amt = crate::section1256::form_6781_amt(d);
        let outside = crate::section1256::form_6781_without_k3(d);
        let amt_outside = crate::section1256::form_6781_amt_without_k3(d);
        for (term, (regular, alternative)) in
            [contracts.short, contracts.long].into_iter().zip([amt.short, amt.long]).enumerate()
        {
            let delta = (i128::from(alternative) - i128::from(regular)) * 100;
            checked_native_add(&mut t.amt_sale_adjustment[term], delta)?;
            checked_native_add(&mut t.amt_investment_gain_adjustment[term], delta)?;
        }
        let short = contracts.short.checked_mul(100)?;
        let long = contracts.long.checked_mul(100)?;
        t.try_add_capital(short, false)?;
        t.try_add_capital(long, true)?;
        checked_native_add(
            &mut t.amt_k1_capital_net_adjustment,
            (i128::from(amt.short) + i128::from(amt.long)
                - i128::from(contracts.short)
                - i128::from(contracts.long))
                * 100,
        )?;
        checked_native_add(
            &mut t.amt_k1_capital_gross_adjustment,
            (i128::from(positive(amt_outside.short)) + i128::from(positive(amt_outside.long))
                - i128::from(positive(outside.short))
                - i128::from(positive(outside.long)))
                * 100,
        )?;
        t.farmer_capital_correction = t
            .farmer_capital_correction
            .zip(outside.gains.checked_mul(100))
            .zip(outside.short.checked_mul(100))
            .zip(outside.long.checked_mul(100))
            .and_then(|(((before, gains), short), long)| {
                crate::kernel::ratio::checked_sum([
                    before,
                    gains,
                    -positive(short),
                    -positive(long),
                ])
            });
        let bypass = i64::try_from(
            i128::from(positive(contracts.short)) * 100 - i128::from(positive(outside.short)) * 100
                + i128::from(positive(contracts.long)) * 100
                - i128::from(positive(outside.long)) * 100,
        )
        .ok()?;
        t.try_bypass_line_3e(bypass)?;
        checked_native_add(&mut t.schedule_d_line_4, i128::from(short))?;
        checked_native_add(&mut t.schedule_d_line_11, i128::from(long))?;
        checked_native_add(&mut t.investment_gains[0], i128::from(short))?;
        checked_native_add(&mut t.investment_gains[1], i128::from(long))?;
        t.capital_endpoints_fit()?;
        t.injured_capital_sources.contracts(d, &contracts, &mut t.injured_investment_sources);
        Some(())
    })();
    values.ok_or_else(|| {
        let context = d
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::Broker { .. }))
            .map_or(
                crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                crate::interview::income_issue_context,
            );
        CollectFailure::Broker(context)
    })
}

// collect_with_claims's inputs and the state its per-document arms carry from one document
// to the next. Each arm moved out of the loop takes it with the document's `Source`.
pub(super) struct Collect<'a> {
    d: &'a FederalDraft,
    form_context: crate::related_forms::RelatedFormContext<'a>,
    source_claims: &'a alloc::collections::BTreeMap<i64, i64>,
    qcd: &'a crate::retirement_return::QcdResult,
    basis: &'a crate::retirement_return::BasisResult,
    selected_pair: Option<&'a [crate::section179_return::Selected; 2]>,
    medical_projection: &'a crate::medical_rollover_return::Prepared,
    home: Option<&'a crate::home_office::HomeContext>,
    excluded_homes: i32,
    itemized_refund_years: Vec<i32>,
    current_year_state_refund_seen: bool,
    previous_k1: Option<(i64, usize, ([i64; 13], Exact))>,
    deferrals: Vec<&'a FederalIncomeEntry>,
}

// The document one collect_with_claims arm reads: `e` is its community-allocated view,
// `accepted_entry` the saved entry, `owner` 0 for the filer and 1 for the spouse.
pub(super) struct Source<'a, 'e> {
    e: &'e FederalIncomeEntry,
    #[allow(dead_code)] // Read by arms that later collect slices move out of the loop.
    accepted_entry: &'a FederalIncomeEntry,
    owner: usize,
    context: &'e crate::IssueContext,
    #[cfg(test)]
    source: &'e str,
}

fn collect_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    qcd: &crate::retirement_return::QcdResult,
    r: &mut FederalResult,
    basis: &crate::retirement_return::BasisResult,
    selected_pair: Option<&[crate::section179_return::Selected; 2]>,
    medical_projection: &crate::medical_rollover_return::Prepared,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<alloc::boxed::Box<Totals>, CollectFailure> {
    let direct_participation = form_context.participation();
    let crate::related_forms::RelatedFormContext { draft: d, claims: source_claims, .. } =
        form_context;
    if let Some((id, _)) = crate::care_benefit_return::native_receipts_issue(d) {
        let context = d
            .income
            .iter()
            .find(|entry| entry.id == id)
            .map(crate::interview::income_issue_context)
            .unwrap_or_else(|| crate::IssueContext::income_id(id));
        return Err(crate::medical_account::OutputFailure { context }.into());
    }
    // Reference admission is return-wide. Reuse its result for each receipt
    // rather than revalidating every insured period for every source form.
    let mid_quarter =
        crate::depreciation::mid_quarter_with_macrs(d, Some(form_context.macrs().as_ref()));
    let ltc_reference_rule = crate::care_benefit_return::reference_rule(d);
    let selected = selected_pair.map(|pair| &pair[0].allocation);
    let mut t = boxed(Totals::default);
    let qsbs = crate::qsbs_return::calculate(d, r);
    t.add_long(qsbs.taxable_gain);
    // Farmer gross retains each admitted source before other sources' losses.
    t.farmer_capital_correction =
        crate::kernel::ratio::checked_sum(qsbs.sources.iter().map(|source| {
            positive(crate::community::income_amount(d, source.id, source.taxable_gain))
        }))
        .and_then(|gross| gross.checked_sub(positive(qsbs.taxable_gain)));
    t.investment_gains[1] += qsbs.taxable_gain;
    // Form 8949 code Q in the box the page names: lines 8b, 9 and 10.
    for row in &qsbs.rows {
        row.emit(r);
        t.add_row(row);
    }
    t.native_withholding_sources = Some(alloc::boxed::Box::new(
        crate::injured_spouse::income_categories::NativeWithholdingSources::collecting(),
    ));
    for source in &qsbs.sources {
        t.native_withholding_sources.as_mut().unwrap().retain(
            source.id,
            source.owner,
            [0, source.withholding, 0],
        );
        t.injured_capital_sources.add(Some(source.owner), [0, source.taxable_gain]);
        t.injured_investment_sources.add(Some(source.owner), [0, source.taxable_gain, 0, 0, 0]);
        if let Some(rate28) = source.rate_28_gain {
            // The native admitted28% source is a subset of its taxable long
            // gain, never another income amount or an issuer-wide allocation.
            t.injured_capital_sources.add_special(Some(source.owner), [rate28, 0]);
        } else {
            t.injured_capital_sources.invalidate();
        }
    }
    t.withholding_1099 += qsbs.withholding;
    t.qsbs_excluded = qsbs.excluded_gain;
    t.collectible_long += qsbs.rate_28_gain;
    t.qsbs_amt_preference = qsbs.amt_preference;
    // Form 6252 line 26 of earlier-year sales of investment property and
    // Form 8824 line 24 of earlier related-party exchanges: Schedule D lines
    // 4 and 11, net investment income.
    let (installment, installment_owners) = crate::installment_return::earlier_capital_sources(d);
    // Current-year raw sales retain their native identity and property classification.
    // The common installment engine owns recognized profit; full recognition uses collect_sale.
    for record in &d.income {
        let FederalIncome::InstallmentSale { data: sale } = &record.income else { continue };
        if sale.issue().is_some() {
            continue;
        }
        let context = crate::interview::income_issue_context(record);
        if let Some((key, _text)) = sale.refusal() {
            crate::issue_localized!(r, &context, "Installment sale", key, _text);
            continue;
        }
        let full = sale.full_sale();
        let entry = FederalIncomeEntry {
            printed: None,
            id: record.id,
            label: sale.property.clone(),
            owner: record.owner,
            community: record.community,
            income: FederalIncome::Sale { data: crate::SaleSource::from_sale(&full, "") },
            state_source: record.state_source.clone(),
            refund_management: record.refund_management,
        };
        let share = d.setup.as_ref().and_then(|setup| {
            crate::community::share(setup, record.owner.owned(), record.community, &entry.income)
        });
        let Ok(form) = sale.calculation() else { continue };
        let full_gain = form.map_or_else(
            || {
                let gain = full.gain();
                if sale.buyer != crate::Buyer::Unrelated { gain.max(0) } else { gain }
            },
            |current| current.gain() * 100,
        );
        let gain = if let Some(current) = form {
            let form = current.form;
            let recognized = current.gain() * 100;
            let gain = share.map_or(recognized, |share| share.gain(recognized));
            if sale.term() == GainTerm::Long {
                t.add_long(gain);
                t.schedule_d_line_11 += gain;
                if full.collectible {
                    t.collectible_long += gain;
                }
            } else {
                t.add_short(gain);
                t.schedule_d_line_4 += gain;
            }
            if matches!(sale.use_kind, crate::InstallmentUse::Investment) {
                t.investment_gains[usize::from(sale.term() == GainTerm::Long)] += gain;
            }
            crate::installment_return::report_sale(
                r,
                &alloc::format!("installment_sale_{}", record.id),
                "income.installment_sale",
                &form,
                0,
                false,
            );
            crate::installment_return::report_current_adjustments(r, record.id, sale, &current);
            gain
        } else {
            let divided = crate::community::record(d, &entry);
            let FederalIncome::Sale { data } = divided.as_ref() else { unreachable!() };
            let data = &crate::FederalSale::from(data);
            if sale.buyer != crate::Buyer::Unrelated && data.gain() < 0 {
                let row = crate::capital::Form8949Row {
                    entry: record.id,
                    row: t.next_row(record.id),
                    line: crate::capital::sale_line(data.term, data.basis_reporting, false),
                    box_letter: data.box_letter(),
                    label: crate::MessageArg::Text(sale.property.clone()),
                    proceeds: data.proceeds,
                    basis: data.column_e(),
                    adjustment: data.column_g() - data.gain(),
                    codes: "EL".into(),
                    summary: false,
                };
                row.emit(r);
                t.add_row(&row);
                0
            } else {
                collect_sale(
                    &mut t,
                    data,
                    &entry,
                    r,
                    &context,
                    #[cfg(test)]
                    "Installment sale",
                );
                data.gain()
            }
        };
        if share.is_some() {
            let label = crate::MessageArg::Text(sale.property.clone());
            line_cents(
                r,
                &alloc::format!("form8958_{}_total", record.id),
                full_gain,
                &["income.installment_sale", "setup.community_property"],
                "https://www.irs.gov/pub/irs-pdf/f8958.pdf",
                Message::new("EXPLAIN_FORM8958_N_TOTAL", alloc::vec![label.clone()]),
            );
            line_cents(
                r,
                &alloc::format!("form8958_{}_this_return", record.id),
                gain,
                &["income.installment_sale", "setup.community_property"],
                "https://www.irs.gov/publications/p555",
                Message::new("EXPLAIN_FORM8958_N_THIS_RETURN", alloc::vec![label]),
            );
        }
        let term = usize::from(sale.term() == GainTerm::Long);
        let mut amounts = [0; 2];
        amounts[term] = gain;
        t.injured_capital_sources.add(Some(record.owner.owned()), amounts);
        if term == 1 && full.collectible {
            t.injured_capital_sources.add_special(Some(record.owner.owned()), [gain, 0]);
        }
        if matches!(sale.use_kind, crate::InstallmentUse::Investment) {
            t.injured_investment_sources
                .add(Some(record.owner.owned()), [amounts[0], amounts[1], 0, 0, 0]);
        }
    }
    let (related, related_owners) = crate::exchange::related_capital_sources(d);
    let installment = [installment[0] + related[0], installment[1] + related[1]];
    for (owner, parts) in
        [Owner::Taxpayer, Owner::Spouse].into_iter().zip(core::array::from_fn::<_, 2, _>(|i| {
            core::array::from_fn::<_, 2, _>(|term| {
                installment_owners[i][term] + related_owners[i][term]
            })
        }))
    {
        t.injured_capital_sources.add(Some(owner), parts);
        t.injured_investment_sources.add(Some(owner), [parts[0], parts[1], 0, 0, 0]);
    }
    t.farmer_capital_correction = t
        .farmer_capital_correction
        .zip(crate::installment_return::earlier_capital_gross(d))
        .zip(crate::exchange::related_capital_gross(d))
        .and_then(|((before, earlier), related)| {
            crate::kernel::ratio::checked_sum([
                before,
                earlier,
                related,
                -positive(installment[0]),
                -positive(installment[1]),
            ])
        });
    t.add_short(installment[0]);
    t.add_long(installment[1]);
    t.schedule_d_line_4 += installment[0];
    t.schedule_d_line_11 += installment[1];
    t.investment_gains[0] += installment[0];
    t.investment_gains[1] += installment[1];
    let broker_sources =
        d.income.iter().any(|entry| matches!(entry.income, FederalIncome::Broker { .. }));
    if !broker_sources {
        collect_contracts(d, &mut t)?;
    }
    let mut stock_losses = [0i64; 2];
    for e in &d.income {
        if let FederalIncome::SmallBusinessStock { data } = &e.income {
            let ordinary = data.ordinary_loss();
            stock_losses[usize::from(data.term() == GainTerm::Long)] += ordinary;
            let capital = data.loss() - ordinary;
            let parts = if data.term() == GainTerm::Long { [0, -capital] } else { [-capital, 0] };
            t.injured_investment_sources
                .add(Some(e.owner.owned()), [parts[0], parts[1], 0, 0, ordinary]);
            t.injured_capital_sources.add(
                Some(e.owner.owned()),
                if data.term() == GainTerm::Long { [0, -capital] } else { [-capital, 0] },
            );
            t.investment_gains[usize::from(data.term() == GainTerm::Long)] -= capital;
            // The capital part is Form 8949 box C or F with code S: the sale's proceeds and
            // basis, the ordinary part as column (g), the capital loss as column (h).
            let (line, box_letter) = if data.term() == GainTerm::Long {
                t.add_long(-capital);
                (7, 'F')
            } else {
                t.add_short(-capital);
                (3, 'C')
            };
            let row = crate::capital::Form8949Row {
                entry: e.id,
                row: 1,
                line,
                box_letter,
                label: crate::interview::income_display_arg(e),
                proceeds: data.proceeds(),
                basis: data.basis.total(),
                adjustment: ordinary,
                codes: if ordinary == 0 { String::new() } else { "S".into() },
                summary: false,
            };
            row.emit(r);
            t.add_row(&row);
            line_cents(
                r,
                &alloc::format!("stock_{}_ordinary_before_limit", e.id),
                ordinary,
                &["income.small_business_stock.basis"],
                "https://www.irs.gov/publications/p550",
                msg!("EXPLAIN_STOCK_N_ORDINARY_BEFORE_LIMIT"),
            );
            line_cents(
                r,
                &alloc::format!("stock_{}_capital_basis_loss", e.id),
                capital,
                &["income.small_business_stock.basis"],
                "https://www.irs.gov/publications/p550",
                msg!("EXPLAIN_STOCK_N_CAPITAL_BASIS_LOSS"),
            );
        }
    }
    let stock_limit = if d.setup.as_ref().is_some_and(|s| joint(s.filing_status())) {
        crate::year_constants::SECTION_1244_ORDINARY_LOSS_LIMIT_JOINT_CENTS
    } else {
        crate::year_constants::SECTION_1244_ORDINARY_LOSS_LIMIT_CENTS
    };
    t.stock_loss = phase::cap(stock_losses.iter().sum(), [stock_limit]);
    crate::require_localized!(
        r,
        stock_losses.iter().sum::<i64>() <= stock_limit || stock_losses.contains(&0),
        &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
        "Section 1244",
        "ISSUE_LOSSES_EXCEEDING_THE_ORDINARY_ALLOWANCE_WITH_BOTH_SHORT_AND_LONG_TE_82324EBC",
        "Losses exceeding the ordinary allowance with both short- and long-term lots need a verified allocation of the annual allowance.",
    );
    // Under the limit there is no capital excess. Above it, the currently
    // supported lots share one holding term, independent of entry order.
    let excess = positive(stock_losses.iter().sum::<i64>() - stock_limit);
    // Source-owned ordinary-loss caps need their own final adapter. Do not
    // let opposite-source changes cancel into apparently complete metadata.
    t.injured_capital_sources.add(None, [0, -excess]);
    // The excess over the limit is also Form 8949 box C or F with code S: it takes back that
    // much of the entries' column (g), so the ordinary losses of the rows total the allowance.
    if stock_losses[0] == 0 {
        t.investment_gains[1] -= excess;
        t.add_long(-excess);
        t.sale_line_10 -= excess;
        t.sale_columns[7 * 3 + 2] -= excess;
    } else {
        t.investment_gains[0] -= excess;
        t.add_short(-excess);
        t.sale_line_3 -= excess;
        t.sale_columns[3 * 3 + 2] -= excess;
    }
    crate::iso_return::reconcile_wages(d, r);
    // Each document's withholding is the change in the three withholding
    // totals while it is collected, credited to its owner (Form 8379 line 19).
    let withheld = |t: &Totals| t.withholding_w2 + t.withholding_1099 + t.withholding_other;
    let categories = |t: &Totals| [t.withholding_w2, t.withholding_1099, t.withholding_other];
    let mut categories_before = categories(&t);
    let mut withheld_before = withheld(&t);
    let mut withheld_owner = 0;
    let mut withheld_source: Option<(i64, Owner)> = None;
    let mut capital_before = [t.short, t.long];
    let mut special_before = [t.collectible_long, t.unrecaptured_long];
    let investment_components = |t: &Totals| {
        [
            t.investment_gains[0],
            t.investment_gains[1],
            t.scorp_loan_gain[0],
            t.k1_investment_ordinary[0],
            0,
        ]
    };
    let mut investment_before = investment_components(&t);
    let mut capital_weights = None;
    let k1_components = |t: &Totals| {
        (
            [
                t.interest,
                t.dividends,
                t.qualified,
                t.scorp_loan_gain[0] + t.k1_ordinary[0],
                t.k1_deferred_tax,
                t.other_8z + t.personal_debt + t.gambling + t.recovery_8z,
                t.k1_line_7,
                t.k1_nii_excluded,
                t.estate_niit_line_7,
                t.estate_niit_excess_deduction,
                0,
                t.scorp_loan_gain[0],
                t.k1_ordinary[0],
            ],
            t.royalties.clone(),
        )
    };
    let retain_k1 = |t: &mut Totals, previous: Option<(i64, usize, ([i64; 13], Exact))>| {
        if let Some((id, owner, before)) = previous {
            let after = k1_components(t);
            let delta = core::array::from_fn::<_, 13, _>(|i| after.0[i] - before.0[i]);
            let royalty_delta = after.1 - before.1;
            t.injured_k1_deferred_tax[owner] += delta[4];
            t.injured_other_sources.insert(
                id,
                Some(exact::whole(delta[0] + delta[1] + delta[3] + delta[5]) + &royalty_delta),
            );
            for (component, amount) in delta[..3].iter().enumerate() {
                t.injured_k1_portfolio[component][owner] += amount;
            }
            for (component, amount) in delta[6..10].iter().enumerate() {
                t.injured_k1_nii.categories[component][owner] += amount;
            }
            t.injured_k1_nii.royalties[owner] +=
                exact::rounded(&royalty_delta).expect("native K-1 royalty source operand");
            t.injured_k1_nii.loans[owner] += delta[11];
            t.injured_k1_nii.ordinary[owner] += delta[12];
        }
    };
    let mut previous_income = None;
    // A community-property return reports a share of each item: the share is
    // applied when the item's amounts have entered the totals (Form 8958).
    let mut pending: Option<PendingShare> = None;
    let mut cx = Collect {
        d,
        form_context,
        source_claims,
        qcd,
        basis,
        selected_pair,
        medical_projection,
        home,
        excluded_homes: 0,
        itemized_refund_years: Vec::new(),
        current_year_state_refund_seen: false,
        previous_k1: None,
        deferrals: Vec::new(),
    };
    // Other sources retain their order. Employee sources use this same collector
    // afterwards, so checked admission includes every existing wage/withholding.
    for (entry_index, e) in
        d.income
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                !matches!(
                    entry.income,
                    FederalIncome::EmployeeWages { .. } | FederalIncome::Broker { .. }
                )
            })
            .chain(
                d.income.iter().enumerate().filter(|(_, entry)| {
                    matches!(entry.income, FederalIncome::EmployeeWages { .. })
                }),
            )
            .chain(
                d.income
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| matches!(entry.income, FederalIncome::Broker { .. })),
            )
    {
        if let Some(item) = pending.take() {
            let entry = &d.income[item.entry];
            item.settle_with_claims(&mut t, d, entry, r, source_claims);
        }
        retain_k1(&mut t, cx.previous_k1.take());
        t.retain_income_categories(previous_income.take());
        previous_income = Some(NativeIncomeFrame {
            id: e.id,
            owner: usize::from(e.owner == crate::DocumentOwner::Spouses),
            rest: NativeRestSource::for_entry(e),
            before: t.income_category_snapshot(),
        });
        let accepted_entry = e;
        cx.previous_k1 = matches!(
            e.income,
            FederalIncome::Partnership { .. }
                | FederalIncome::Scorp { .. }
                | FederalIncome::EstateTrustPortfolio { .. }
        )
        .then(|| (e.id, usize::from(e.owner == crate::DocumentOwner::Spouses), k1_components(&t)));
        t.injured_capital_sources.add_weighted(
            capital_weights,
            [t.short - capital_before[0], t.long - capital_before[1]],
        );
        t.injured_capital_sources.add_special_weighted(
            capital_weights,
            [t.collectible_long - special_before[0], t.unrecaptured_long - special_before[1]],
        );
        let investment_now = investment_components(&t);
        t.injured_investment_sources.add_weighted(
            capital_weights,
            core::array::from_fn(|i| investment_now[i] - investment_before[i]),
        );
        investment_before = investment_now;
        capital_before = [t.short, t.long];
        special_before = [t.collectible_long, t.unrecaptured_long];
        let allocated_entry = boxed(|| crate::community::entry(d, e));
        let e = allocated_entry.as_ref().as_ref();
        capital_weights = e.accepted_assignment().owner_weights();
        let now = withheld(&t);
        t.withholding_owner[withheld_owner] += now - withheld_before;
        let withholding_delta = core::array::from_fn(|i| categories(&t)[i] - categories_before[i]);
        for (category, amount) in withholding_delta.iter().enumerate() {
            t.withholding_categories[category][withheld_owner] += amount;
        }
        if let Some((id, owner)) = withheld_source {
            t.native_withholding_sources.as_mut().unwrap().retain(id, owner, withholding_delta);
        } else if withholding_delta != [0; 3] {
            t.native_withholding_sources.as_mut().unwrap().complete = false;
        }
        categories_before = categories(&t);
        withheld_before = now;
        let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
        withheld_owner = owner;
        withheld_source = Some((e.id, e.owner.owned()));
        pending =
            d.setup.as_ref().filter(|_| !crate::community::self_divided(&e.income)).and_then(|s| {
                let share = crate::community::entry_share(s, e)?;
                let withholding = crate::community::withholding_share(d, e, source_claims)?;
                Some(PendingShare::new(&mut t, entry_index, owner, share, withholding))
            });
        if matches!(e.income, FederalIncome::EmployeeWages { .. })
            && let Some(frame) = &mut pending
        {
            frame.stage_employee(&mut t);
        }
        let context = crate::interview::income_issue_context(e);
        #[cfg(test)]
        let source = alloc::format!("Income: {}", e.label);
        // unknowns::supported left off the after-death part of a person who
        // died; a part that remains belongs to a living owner's statement.
        if crate::after_death::recorded(&e.income) {
            let message = crate::Message::new(
                "ISSUE_AFTER_DEATH_WITHOUT_A_DEATH",
                alloc::vec![crate::interview::income_display_arg(e)],
            );
            let page =
                if owner == 0 { crate::TaxSection::People } else { crate::TaxSection::Spouse };
            for place in [context.clone(), crate::IssueContext::owner(page, e.owner.owned())] {
                issue_message(
                    r,
                    &place,
                    #[cfg(test)]
                    &source,
                    #[cfg(test)]
                    message.english(),
                    message.clone(),
                );
            }
        }
        // A community-property separate return divides every item by its own
        // answer; a row saved before the return became one has none yet.
        if e.community == crate::CommunityShare::NotNeeded
            && d.setup.as_ref().is_some_and(crate::community::applies)
        {
            let message = crate::Message::new(
                "ISSUE_COMMUNITY_ANSWER_NEEDED",
                alloc::vec![crate::interview::income_display_arg(e)],
            );
            issue_message(
                r,
                &context,
                #[cfg(test)]
                &source,
                #[cfg(test)]
                message.english(),
                message,
            );
        }
        crate::require_localized!(
            r,
            owner == 0
                || d.setup.as_ref().is_some_and(|s| joint(s.filing_status()))
                || d.setup.as_ref().is_some_and(crate::community::applies)
                || pending.as_ref().is_some_and(|frame| frame.share.none()),
            &context,
            &source,
            "ISSUE_SPOUSE_OWNED_INCOME_NEEDS_A_JOINT_OR_COMMUNITY_PROPERTY_RETURN",
            "Spouse-owned income requires a joint return or a community-property separate return.",
        );
        let src = Source {
            e,
            accepted_entry,
            owner,
            context: &context,
            #[cfg(test)]
            source: &source,
        };
        match &e.income {
            FederalIncome::PartnershipActivities { .. } | FederalIncome::ScorpActivities { .. } => {
                collect_k1_activities(&mut cx, &src, &mut t, r, &k1_components)?;
            }
            FederalIncome::Partnership { data } => {
                collect_partnership(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::Scorp { data: stored } => {
                collect_scorp(&mut cx, &src, &mut t, r, stored)?;
            }
            FederalIncome::EstateTrustPortfolio { data } => {
                collect_estate_trust_portfolio(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::InheritedIra { data } => {
                collect_inherited_ira(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::Roth { data: stored } => {
                collect_roth(&mut cx, &src, &mut t, r, stored)?;
            }
            FederalIncome::Royalty { data } => collect_royalty(&mut cx, &src, &mut t, r, data)?,
            FederalIncome::NonbusinessProperty { kind, receipts, expenses, .. } => {
                collect_nonbusiness_property(&mut cx, &src, &mut t, r, (kind, receipts, expenses))?
            }
            FederalIncome::SmallBusinessStock { .. } => {
                // Its loss is collected above; a negative qualification answer leaves it capital.
            }
            FederalIncome::QualifiedSmallBusinessSale { .. } => {
                // Issuer-wide limits are calculated before the per-document pass.
            }

            FederalIncome::WorthlessSecurity { data } => {
                collect_worthless_security(&mut cx, &src, &mut t, r, data)?
            }

            FederalIncome::NonbusinessBadDebt { data } => {
                collect_nonbusiness_bad_debt(&mut cx, &src, &mut t, r, data)?
            }

            FederalIncome::HomeSale { data } => collect_home_sale(&mut cx, &src, &mut t, r, data)?,

            FederalIncome::ReturnOfCapital { kind, amount, property, basis, term, .. } => {
                collect_return_of_capital(
                    &mut cx,
                    &src,
                    &mut t,
                    r,
                    (kind, amount, property, basis, term),
                )?
            }

            FederalIncome::UndistributedGain { data } => {
                collect_undistributed_gain(&mut cx, &src, &mut t, r, data, pending.as_mut())?
            }
            FederalIncome::OpportunityFund { data } => {
                collect_opportunity_fund(&mut cx, &src, &mut t, r, data)?
            }
            // A deferral is checked against the year's gains once every row is in (below).
            // The deferral is an ordinary settled entry: its amounts leave the totals here and
            // take its own community share; the cap check waits for every row (below).
            FederalIncome::OpportunityFundDeferral { data } => {
                collect_opportunity_fund_deferral(&mut cx, &src, &mut t, r, data)?
            }
            FederalIncome::HobbySales { data } => {
                collect_hobby_sales(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::Able { data } => {
                collect_able(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::TrumpDeath { data: stored } => {
                collect_trump_death(&mut cx, &src, &mut t, r, stored)?;
            }
            FederalIncome::TrumpAbleRollover { data: stored } => {
                collect_trump_able_rollover(&mut cx, &src, &mut t, r, stored)?;
            }
            FederalIncome::MedicalDistribution { data } => {
                collect_medical_distribution(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::LtcBenefits { data } => {
                collect_ltc_benefits(&mut cx, &src, &mut t, r, data, ltc_reference_rule)?;
            }
            FederalIncome::LtcInsured { data } => {
                collect_ltc_insured(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::HealthLoanAssistance { data } => {
                collect_health_loan_assistance(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::WildfireRelief { data } => {
                collect_wildfire_relief(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::DisasterRelief { data } => {
                collect_disaster_relief(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::CasualtyLivingInsurance { data } => {
                collect_casualty_living_insurance(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::DisasterMitigation { data } => {
                collect_disaster_mitigation(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::PersonalDebt { data } => {
                collect_personal_debt(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::PropertyAcquisition { data } => {
                collect_property_acquisition(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::OtherIncome { data } => {
                collect_other_income(&mut cx, &src, &mut t, r, data)?
            }
            FederalIncome::Nec1099 { data } => collect_nec1099(&mut cx, &src, &mut t, r, data)?,
            FederalIncome::Misc1099 { data } => collect_misc1099(&mut cx, &src, &mut t, r, data)?,
            FederalIncome::K1099 { data } => collect_k1099(&mut cx, &src, &mut t, r, data)?,
            FederalIncome::Scholarship { data } => {
                collect_scholarship(&mut cx, &src, &mut t, r, data)?
            }
            FederalIncome::JuryDuty { data } => collect_jury_duty(&mut cx, &src, &mut t, r, data)?,
            FederalIncome::Alimony { data } => collect_alimony(&mut cx, &src, &mut t, r, data)?,
            FederalIncome::EmployeeWages { data } => {
                collect_employee_wages(&mut cx, &src, &mut t, r, data)?
            }
            FederalIncome::ForeignEmployment { .. } => {}
            FederalIncome::Rental { data } => {
                collect_rental(&mut cx, &src, &mut t, r, data, mid_quarter)?
            }
            FederalIncome::Wage { data: w } => collect_wage(&mut cx, &src, &mut t, r, w)?,
            FederalIncome::ChildElection { data } => {
                collect_child_election(&mut cx, &src, &mut t, r, data)?
            }
            FederalIncome::Interest { data: v } => collect_interest(&mut cx, &src, &mut t, r, v)?,
            FederalIncome::Oid { data: v } => collect_oid(&mut cx, &src, &mut t, r, v)?,
            FederalIncome::Dividend { data: v } => {
                collect_dividend(&mut cx, &src, &mut t, r, v, pending.as_mut())?
            }
            FederalIncome::NuaSale { data } => collect_nua_sale(&mut cx, &src, &mut t, r, data)?,
            FederalIncome::EmployeeStockSale { data } => {
                collect_employee_stock_sale(&mut cx, &src, &mut t, r, data)?
            }
            FederalIncome::EsppSaleSource { .. }
            | FederalIncome::IsoSaleSource { .. }
            | FederalIncome::NuaSaleSource { .. } => {}
            FederalIncome::EsppSale { data } => collect_espp_sale(&mut cx, &src, &mut t, r, data)?,
            // A sale with special treatment never reaches here (unknowns.rs).
            FederalIncome::IsoSale { data } => collect_iso_sale(&mut cx, &src, &mut t, r, data)?,
            FederalIncome::InstallmentSale { .. } => {}
            FederalIncome::Sale { data } => collect_sale_source(&mut cx, &src, &mut t, r, data)?,
            FederalIncome::Broker { data } => collect_broker(&mut cx, &src, &mut t, r, data)?,
            FederalIncome::Retirement { data: stored } => {
                collect_retirement(&mut cx, &src, &mut t, r, stored)?;
            }
            FederalIncome::SocialSecurity { data } => {
                collect_social_security(&mut cx, &src, &mut t, r, data)?;
            }
            FederalIncome::Gambling { data } => {
                collect_gambling(&mut cx, &src, &mut t, r, data)?;
            }
            // Form 8815 adds no income of its own: the savings bond interest
            // is on a 1099-INT, and the exclusion is figured with the return
            // (savings_bonds::calculate).
            FederalIncome::SavingsBonds { .. } | FederalIncome::BondRedemption { .. } => {}
            // Form 6781 Part I: the amounts join Schedule D lines 4 and 11
            // below; the statement's own withholding is credited here.
            FederalIncome::Section1256 { data } => {
                collect_section_1256(&mut cx, &src, &mut t, r, data)?
            }
            // A Form 1099-Q adds income of its own only through the expenses
            // the return reports (education_account::calculate).
            FederalIncome::EducationDistribution { .. }
            | FederalIncome::EducationAccountExpenses { .. } => {}
            FederalIncome::GovernmentPayment { data } => {
                t.injured_other_sources.insert(e.id, Some(exact::whole(0)));
                // Box 4 belongs to the physical paper, independently of the
                // components its payment and refund owners can calculate.
                t.withholding_1099 += data.form.federal_income_tax_withheld;
                if data.form.has_payments() || !data.repayments.is_empty() {
                    collect_unemployment(&mut cx, &src, &mut t, r, data)?;
                }
                if data.form.state_or_local_income_tax_refunds != 0 {
                    collect_tax_refund(&mut cx, &src, &mut t, r, data)?;
                }
            }
            FederalIncome::Cooperative { data: v } => {
                collect_cooperative(&mut cx, &src, &mut t, r, v)?;
            }
            FederalIncome::Business { data: v } => collect_business(
                &mut cx,
                &src,
                &mut t,
                r,
                v,
                BusinessCollectIn {
                    direct_participation: &direct_participation,
                    selected,
                    #[cfg(test)]
                    source: &source,
                },
            )?,
            // Form 4835: the rental's lines and notes; its income,
            // passive limits and Schedule E line 40 come from the passive pool
            // (extended_rules.rs).
            FederalIncome::FarmRental { data: v } => {
                collect_farm_rental(&mut cx, &src, &mut t, r, v, &direct_participation)?;
            }
        }
    }
    if let Some(item) = pending.take() {
        let entry = &d.income[item.entry];
        item.settle_with_claims(&mut t, d, entry, r, source_claims);
    }
    retain_k1(&mut t, cx.previous_k1.take());
    t.retain_income_categories(previous_income.take());
    t.injured_capital_sources
        .add_weighted(capital_weights, [t.short - capital_before[0], t.long - capital_before[1]]);
    t.injured_capital_sources.add_special_weighted(
        capital_weights,
        [t.collectible_long - special_before[0], t.unrecaptured_long - special_before[1]],
    );
    let investment_now = investment_components(&t);
    t.injured_investment_sources.add_weighted(
        capital_weights,
        core::array::from_fn(|i| investment_now[i] - investment_before[i]),
    );
    t.withholding_owner[withheld_owner] += withheld(&t) - withheld_before;
    let withholding_delta = core::array::from_fn(|i| categories(&t)[i] - categories_before[i]);
    for (category, amount) in withholding_delta.iter().enumerate() {
        t.withholding_categories[category][withheld_owner] += amount;
    }
    if let Some((id, owner)) = withheld_source {
        t.native_withholding_sources.as_mut().unwrap().retain(id, owner, withholding_delta);
    } else if withholding_delta != [0; 3] {
        t.native_withholding_sources.as_mut().unwrap().complete = false;
    }
    if broker_sources {
        collect_contracts(d, &mut t)?;
    }
    // Code Z: a 2026 gain invested in an opportunity fund in 2027 (IRC 1400Z-2(a)). The deferred
    // gain of each holding period cannot exceed that period's positive Form 8949 rows.
    let mut deferred = [0i64; 2];
    for e in cx.deferrals {
        let FederalIncome::OpportunityFundDeferral { data } = &e.income else {
            continue;
        };
        let mut over = None;
        for (term, amount, word) in
            [(0, data.short_term, "TERM_SHORT_TERM"), (1, data.long_term, "TERM_LONG_TERM")]
        {
            deferred[term] += amount;
            if deferred[term] > t.positive_rows[term] {
                over.get_or_insert(word);
            }
        }
        if let Some(word) = over {
            let message = Message::new(
                "ISSUE_QOF_DEFERRAL_ABOVE_GAINS",
                alloc::vec![
                    crate::interview::income_display_arg(e),
                    MessageArg::Resource(word.into()),
                ],
            );
            issue_message(
                r,
                &crate::interview::income_issue_context(e),
                #[cfg(test)]
                "Opportunity fund deferral",
                #[cfg(test)]
                message.english(),
                message,
            );
        }
    }
    collect_income_without_form(d, &mut t, r);
    // Pub. 525 Worksheet 2 for each prior year the return recovers something in.
    crate::other_income::collect_years(d, &mut t, r);
    // Section 408(d)(4) treats timely returned traditional or Roth principal
    // as never contributed. Report the actual distribution and any positive
    // earnings on the 2026 return even when the custodian pays it in 2027.
    let transfers = crate::ira::QtpTransfers::of(&d.income);
    if let Some(iras) = crate::ira::facts(d, &transfers).as_deref() {
        for (index, (owner, deposits_key, rollover_key, fact)) in [
            (
                "taxpayer",
                "NOTE_NEXT_YEAR_IRA_DEPOSITS_TAXPAYER",
                "NOTE_QTP_ROTH_ROLLOVERS_TAXPAYER",
                Some(&iras.taxpayer),
            ),
            (
                "spouse",
                "NOTE_NEXT_YEAR_IRA_DEPOSITS_SPOUSE",
                "NOTE_QTP_ROTH_ROLLOVERS_SPOUSE",
                iras.spouse.as_ref(),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            if let Some(fact) = fact
                && let Some(deposits) = fact.next_year_deposits.as_ref()
            {
                crate::require_localized!(
                    r,
                    deposits.eligible(fact),
                    &crate::IssueContext::Section(crate::TaxSection::IraContributions),
                    "IRA 2027 deposit for 2026",
                    "ISSUE_CONFIRM_ALL_INCLUDED_REGULAR_IRA_DEPOSITS_WERE_RECEIVED_BY_APRIL_15_3676602F",
                    "Confirm all included regular IRA deposits were received by April 15, 2027 and designated for 2026 with their custodians. Their principal must be included once in the retained 2026 traditional or Roth contribution totals.",
                );
                if deposits.eligible(fact) {
                    for (box_number, amount) in [(1, deposits.traditional), (10, deposits.roth)] {
                        if amount > 0 {
                            line_cents(
                                r,
                                &alloc::format!(
                                    "{owner}_2026_form_5498_box{box_number}_2027_deposit"
                                ),
                                amount,
                                &["adjustments.iras"],
                                "https://www.irs.gov/instructions/i1099r",
                                msg!("EXPLAIN_N_N_FORM_N_BOX0_N_DEPOSIT"),
                            );
                        }
                    }
                    r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
                        "2026-designated IRA deposits paid in 2027 ({owner}): reconcile the custodian's 2026 Form 5498, box 1 for traditional and box 10 for Roth, with the actual deposits. These amounts enter the 2026 contribution limit, deduction or basis, and any 2026 Saver's Credit once. Do not reuse them for a 2027 Saver's Match. The custodian, not this App, files Form 5498."
                    ), Message::new(deposits_key, alloc::vec![])));
                }
            }
            if let Some(fact) = fact
                && !transfers.owner(index).is_empty()
            {
                let rollovers = transfers.owner(index);
                let qualified =
                    crate::ira::valid_qtp_rollovers(rollovers, fact.qtp_roth_lifetime_prior)
                        && rollovers.iter().all(crate::ira::QtpTransfer::eligible);
                crate::require_localized!(
                    r,
                    qualified,
                    &crate::IssueContext::Section(crate::TaxSection::OtherIncome),
                    "529-to-Roth IRA rollover",
                    "ISSUE_CONFIRM_EVERY_DIRECT_2026_TRUSTEE_TRANSFER_FOR_THE_SAME_BENEFICIARY_E404803A",
                    "Confirm every direct 2026 trustee transfer for the same beneficiary, each 529 account's 15-year history and administrator-reconciled five-year seasoned source amount after earlier distributions, and $35,000 lifetime room across all 529 accounts.",
                );
                if qualified {
                    let source = &["adjustments.iras", "income.education_distribution"];
                    let qtp_source = "https://www.irs.gov/taxtopics/tc313";
                    let total = crate::ira::qtp_rollover_amount(rollovers);
                    let contribution_basis: i64 =
                        rollovers.iter().map(|entry| entry.contributions).sum();
                    line_cents(
                        r,
                        &alloc::format!("{owner}_529_roth_form_5498_box10"),
                        total,
                        source,
                        "https://www.irs.gov/instructions/i1099r",
                        msg!("EXPLAIN_N_N_ROTH_FORM_N_BOX10"),
                    );
                    line_cents(
                        r,
                        &alloc::format!("{owner}_529_roth_source_contribution_basis"),
                        contribution_basis,
                        source,
                        qtp_source,
                        msg!("EXPLAIN_N_N_ROTH_SOURCE_CONTRIBUTION_BASIS"),
                    );
                    emit_lines(
                        r,
                        &mut (),
                        rollovers.iter().enumerate(),
                        |(position, rollover), _, _| {
                            Some(calculation_row(
                                None,
                                &alloc::format!(
                                    "{owner}_529_roth_transfer_{}_amount",
                                    position + 1
                                ),
                                LineValue::MoneyCents(rollover.amount()),
                                source,
                                qtp_source,
                                Message::new(
                                    "EXPLAIN_N_N_ROTH_TRANSFER_N_AMOUNT",
                                    alloc::vec![
                                        MessageArg::Number((position + 1) as i64),
                                        MessageArg::Text(
                                            rollover.rollover.account_reference.clone()
                                        ),
                                        MessageArg::DateOnly(i64::from(
                                            rollover.rollover.transferred_on
                                        ))
                                    ],
                                ),
                            ))
                        },
                    );
                    r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
                        "529-to-Roth IRA rollovers ({owner}): reconcile every direct transfer on its 2026 Form 1099-Q (including box 4b and its contribution/earnings split) and the sum of all receiving custodians' 2026 Form 5498 box 10 entries. These qualified transfers are excluded from federal income and the Saver's Credit base. Their source 529 contribution portions add Roth basis, but they are not regular IRA deposits and must not also appear in Income. The custodians, not this App, file the information returns."
                    ), Message::new(rollover_key, alloc::vec![])));
                }
            }
            // A contribution returned to the person in 2027 with its earnings or loss, by the
            // return's due date (section 408(d)(4)): the cash paid back is Form 1040 line 4a,
            // positive earnings line 4b, with no additional tax (Form 5329 exception 21) and
            // nothing on Form 8606 (i8606 Return of IRA Contributions).
            if let Some(fact) = fact {
                for (kind, key) in
                    [(crate::IraKind::Traditional, "ira"), (crate::IraKind::Roth, "roth_ira")]
                {
                    let returned: alloc::vec::Vec<_> =
                        fact.returned_next_year.iter().filter(|entry| entry.kind == kind).collect();
                    if returned.is_empty() {
                        continue;
                    }
                    let gross: i64 = returned.iter().map(|entry| entry.gross()).sum();
                    let taxable_earnings: i64 = returned.iter().map(|entry| entry.taxable()).sum();
                    t.retirement_gross[0] += gross;
                    t.retirement_taxable[0] += taxable_earnings;
                    t.retirement += taxable_earnings;
                    t.injured_retirement[0][index] += taxable_earnings;
                    line_cents(
                        r,
                        &alloc::format!("{owner}_{key}_timely_return_gross"),
                        gross,
                        &["adjustments.iras"],
                        "https://www.irs.gov/instructions/i8606",
                        msg!("EXPLAIN_N_N_TIMELY_RETURN_GROSS"),
                    );
                    line_cents(
                        r,
                        &alloc::format!("{owner}_{key}_timely_return_earnings"),
                        taxable_earnings,
                        &["adjustments.iras"],
                        "https://www.irs.gov/publications/p590a",
                        msg!("EXPLAIN_N_N_TIMELY_RETURN_EARNINGS"),
                    );
                    let message_key = match (owner, kind) {
                        ("taxpayer", crate::IraKind::Traditional) => {
                            "NOTE_RETURNED_TRADITIONAL_IRA_TAXPAYER"
                        }
                        ("taxpayer", _) => "NOTE_RETURNED_ROTH_IRA_TAXPAYER",
                        (_, crate::IraKind::Traditional) => "NOTE_RETURNED_TRADITIONAL_IRA_SPOUSE",
                        (_, _) => "NOTE_RETURNED_ROTH_IRA_SPOUSE",
                    };
                    r.review_notes.push(ReviewNote::localized(
                        #[cfg(test)] alloc::format!("Timely returned {} contribution ({owner}): attach a statement to the 2026 return explaining the original contribution, allocated gain or loss, actual distribution, and receipt date. This App does not prepare or file the statement.", if kind == crate::IraKind::Traditional { "traditional IRA" } else { "Roth IRA" }),
                        Message::new(message_key, alloc::vec![]),
                    ));
                    if returned
                        .iter()
                        .any(|entry| entry.late == Some(crate::LateCorrection::TimelyFiledReturn))
                    {
                        note_once(r, msg!("NOTE_IRA_RETURN_AMENDED"));
                    }
                }
            }
        }
    }
    // Pub. 525 applies the $3,000 test to repayments on the return, not
    // independently to each source or spouse. Same-year unemployment netting
    // is not a prior-income deduction. Net Social Security repayments have
    // their own prior-taxability calculation, so combined amounts above the
    // boundary require that history rather than assuming a taxable fraction.
    let prior_unemployment = d
        .income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::GovernmentPayment { data } => Some(
                crate::FederalUnemployment::from(data)
                    .repayments
                    .iter()
                    .filter(|p| p.received_year < 2026)
                    .map(|p| p.amount)
                    .sum::<i64>(),
            ),
            _ => None,
        })
        .sum::<i64>();
    let prior_railroad: i64 = d
        .income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Retirement { data: stored } => {
                let data = &crate::FederalRetirement::from(stored);
                match &data.taxable {
                    Some(RetirementTaxable::Railroad { repayments, .. }) => {
                        Some(repayments.iter().map(|p| p.amount).sum::<i64>())
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .sum();
    let review_total = prior_unemployment + prior_railroad + positive(-t.social);
    if prior_railroad > 0 {
        line_cents(
            r,
            "railroad_prior_year_repayments",
            prior_railroad,
            &["income.retirement.taxable.repayments"],
            "https://www.irs.gov/publications/p575",
            msg!("EXPLAIN_RAILROAD_PRIOR_YEAR_REPAYMENTS"),
        );
    }
    crate::require_localized!(
        r,
        review_total <= 300_000
            || (t.social >= 0
                && d.deductions.as_ref().is_some_and(|v| v.repayment.method().is_some())),
        &crate::IssueContext::Section(crate::TaxSection::Deductions),
        if prior_unemployment + prior_railroad > 0 {
            "Earlier-income repayments"
        } else {
            "Social Security repayments"
        },
        "ISSUE_COMBINED_EARLIER_YEAR_UNEMPLOYMENT_RAILROAD_AND_NET_SOCIAL_SECURITY_4FCE0321",
        "Combined earlier-year unemployment, railroad and net Social Security repayments exceed $3,000. Prior taxable-income records and deduction-versus-credit eligibility must be evaluated; do not subtract them from current unemployment benefits.",
    );
    if prior_unemployment + prior_railroad > 0 {
        line_cents(
            r,
            "unemployment_prior_year_repayments",
            prior_unemployment,
            &["income.unemployment.repayments"],
            "https://www.irs.gov/publications/p525",
            msg!("EXPLAIN_UNEMPLOYMENT_PRIOR_YEAR_REPAYMENTS"),
        );
        line_cents(
            r,
            "prior_income_repayment_review_total",
            review_total,
            &["income.unemployment.repayments", "income.social_security.benefits"],
            "https://www.irs.gov/publications/p525",
            msg!("EXPLAIN_PRIOR_INCOME_REPAYMENT_REVIEW_TOTAL"),
        );
    }

    line_cents(
        r,
        "social_security_net_repayments",
        positive(-t.social),
        &["income.social_security.benefits"],
        "https://www.irs.gov/publications/p915",
        msg!("EXPLAIN_SOCIAL_SECURITY_NET_REPAYMENTS"),
    );
    t.social = positive(t.social);
    Ok(t)
}
// Test-only record of the regular-tax base each credit limit starts from, with the part of
// regular_liability the limit leaves out by rule: the corpus check (tests::supported) asserts every
// base plus its exclusions is Form 1040 line 18 less the alternative minimum tax, one base.
#[cfg(test)]
extern crate std;
#[cfg(test)]
std::thread_local! {
    pub(crate) static CREDIT_BASES: core::cell::RefCell<Vec<(&'static str, i64)>> =
        const { core::cell::RefCell::new(Vec::new()) };
}
#[cfg(test)]
fn record_credit_base(name: &'static str, base_with_exclusions: i64) {
    CREDIT_BASES.with(|bases| bases.borrow_mut().push((name, base_with_exclusions)));
}

pub(crate) fn estimate(d: &FederalDraft) -> FederalResult {
    // A document reaches the calculation only through an owned value: while
    // any document's owner is Unknown, the estimate waits for it.
    let unknown = crate::interview::unknown_owner_issues(d);
    if !unknown.is_empty() {
        let mut result = FederalResult::empty(d.revision);
        for context in &unknown {
            issue_message(
                &mut result,
                context,
                #[cfg(test)]
                "Documents",
                #[cfg(test)]
                "Whose document is this? Add it to its owner in this section.",
                Message::new(
                    "ISSUE_WHOSE_DOCUMENT_IS_THIS",
                    vec![crate::MessageArg::Nested(alloc::boxed::Box::new(context.label()))],
                ),
            );
        }
        return result;
    }
    for entry in &d.income {
        if let FederalIncome::EstateTrustPortfolio { data } = &entry.income
            && let Some(issue) = data.admission_issue()
        {
            let mut result = FederalResult::empty(d.revision);
            crate::issue_localized!(
                &mut result,
                &crate::interview::income_issue_context(entry),
                "Income",
                issue.key,
                "Review the document amounts; they must be within the supported input range.",
            );
            withhold_required_source_result(&mut result);
            result.financial_source_scope_complete = false;
            return result;
        }
        if let FederalIncome::Scorp { data } = &entry.income
            && let Some(issue) = data.admission_issue()
        {
            let mut result = FederalResult::empty(d.revision);
            crate::issue_localized!(
                &mut result,
                &crate::interview::income_issue_context(entry),
                "Income",
                issue.key,
                "Review the document amounts; they must be within the supported input range.",
            );
            withhold_required_source_result(&mut result);
            result.financial_source_scope_complete = false;
            return result;
        }
    }
    if let Some(issue) = d.deductions.as_ref().and_then(|deductions| {
        deductions.mortgage.iter().find_map(crate::Mortgage1098Source::conversion_issue)
    }) {
        let mut result = FederalResult::empty(d.revision);
        crate::issue_localized!(
            &mut result,
            &crate::IssueContext::Section(crate::TaxSection::Mortgage).manual(),
            "Mortgage interest",
            issue.key,
            "Complete this mortgage's personal answers before using the estimate.",
        );
        withhold_required_source_result(&mut result);
        result.financial_source_scope_complete = false;
        return result;
    }
    let source_claims = crate::related_forms::effective_claimants(d);
    let mut result = estimate_inner(d, &source_claims);
    // Required broker source facts and unclassified actual receipts cannot
    // become a completed return by leaving out that component. The collector
    // still owns the document's single withholding contribution.
    for entry in &d.income {
        let FederalIncome::Broker { data } = &entry.income else { continue };
        let view = data.projection();
        let receipt =
            view.receipt.filter(|receipt| receipt.amount != 0).map(|receipt| receipt.boundary());
        for key in [view.sale_boundary, view.contracts_boundary, receipt].into_iter().flatten() {
            let context = crate::interview::income_issue_context(entry);
            if !result
                .issues
                .iter()
                .any(|issue| issue.context == context && issue.display.key == key)
            {
                issue_message(
                    &mut result,
                    &context,
                    #[cfg(test)]
                    "Broker document",
                    #[cfg(test)]
                    "Complete the actual broker source and its required personal records before using this estimate.",
                    Message::new(key, alloc::vec![]),
                );
            }
            withhold_required_source_result(&mut result);
            result.financial_source_scope_complete = false;
        }
    }
    if result.state != ResultState::Incomplete
        && let Some((None, issue)) = crate::interview::retirement_aggregate_issue(d)
    {
        let context = crate::IssueContext::Section(crate::TaxSection::Retirement);
        crate::issue_localized!(
            &mut result,
            &context,
            "Retirement",
            issue.key,
            "Complete the retirement records before using this estimate."
        );
        withhold_required_source_result(&mut result);
        result.financial_source_scope_complete = false;
    }
    if result.state != ResultState::Incomplete
        && let Some((id, issue)) = crate::information_returns::answer_issue(d)
    {
        let mut context = d
            .income
            .iter()
            .find(|entry| entry.id == id)
            .map_or(crate::IssueContext::income_id(id), crate::interview::income_issue_context);
        // Estate personal records are answered on the K-1 topic.
        if d.income.iter().any(|entry| {
            entry.id == id && matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. })
        }) {
            context = context.corrective(crate::TaxSection::PassThrough);
        }
        if !result
            .issues
            .iter()
            .any(|existing| existing.context == context && existing.display.key == issue.key)
        {
            crate::issue_localized!(
                &mut result,
                &context,
                &alloc::format!(
                    "Income: {}",
                    d.income
                        .iter()
                        .find(|entry| entry.id == id)
                        .map_or("", |entry| entry.label.as_str())
                ),
                issue.key,
                "Complete the form's personal records on its income topic before using this estimate."
            );
        }
        withhold_required_source_result(&mut result);
        result.financial_source_scope_complete = false;
    }
    if let Some((id, issue)) = crate::social_documents::issue(d, true) {
        let context = d
            .income
            .iter()
            .find(|entry| entry.id == id)
            .map_or(crate::IssueContext::income_id(id), crate::interview::income_issue_context);
        crate::issue_localized!(
            &mut result,
            &context,
            "Medicare premiums",
            issue.key,
            "Identify where these printed Medicare premiums have already been included or reimbursed."
        );
        withhold_required_source_result(&mut result);
        result.financial_source_scope_complete = false;
    }
    if crate::related_forms::destination_issues_with_claims(d, &source_claims, &mut result) {
        withhold_required_source_result(&mut result);
        result.financial_source_scope_complete = false;
    }
    if d.credits.is_some() {
        let issues = result.issues.len();
        crate::estate_foreign::check(d, &mut result);
        crate::estate_foreign::warn(d, &mut result);
        if result.issues.len() != issues {
            withhold_required_source_result(&mut result);
        }
    }
    #[cfg(test)]
    crate::tests::cross::oracle::record_estimate(d, &result);
    result
}

fn estimate_inner(
    d: &FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> FederalResult {
    if let Some((id, key)) = crate::broker_calculation::financial_issue(d) {
        let mut result = FederalResult::empty(d.revision);
        let context = d
            .income
            .iter()
            .find(|entry| entry.id == id)
            .map_or(crate::IssueContext::income_id(id), crate::interview::income_issue_context);
        crate::issue_localized!(
            &mut result,
            &context,
            "Broker financial sources",
            key,
            "The actual statement totals cannot be represented in this estimate. Keep the source records for review.",
        );
        result.state = ResultState::Unsupported;
        result.financial_source_scope_complete = false;
        return result;
    }
    if let Some((id, key)) = crate::nonmacrs::financial_issue(d) {
        let mut result = FederalResult::empty(d.revision);
        crate::issue_localized!(
            &mut result,
            &crate::IssueContext::income_id(id).manual(),
            "Property depreciation",
            key,
            "The actual property sources cannot be projected into the financial owner's signed-cent carrier. Retain the source records; this estimate is unavailable.",
        );
        result.state = ResultState::Unsupported;
        return result;
    }
    // Check associations against accepted sources before benefit omission. The
    // calculation view may omit an ancestor's benefit without changing a
    // retained renewal's actual predecessor identity.
    if let Some((id, issue)) = crate::amortization::identity_issue(d) {
        let mut result = FederalResult::empty(d.revision);
        let context = d
            .income
            .iter()
            .find(|entry| entry.id == id)
            .map_or(crate::IssueContext::income_id(id), crate::interview::income_issue_context);
        crate::issue_localized!(
            &mut result,
            &context,
            "Intangible sources",
            issue.key,
            "The recorded predecessor must identify an actual earlier intangible in this activity.",
        );
        result.state = ResultState::Unsupported;
        return result;
    }
    if let Some((id, issue)) = crate::related_forms::source_issue(d) {
        let mut result = FederalResult::empty(d.revision);
        let context = d
            .income
            .iter()
            .find(|entry| entry.id == id)
            .map_or(crate::IssueContext::income_id(id), crate::interview::income_issue_context);
        crate::issue_localized!(
            &mut result,
            &context,
            "Related forms",
            issue.key,
            "This form is no longer available. Reopen the activity to refresh its related forms.",
        );
        result.state = ResultState::Unsupported;
        return result;
    }
    if let Some(key) = crate::qbi_aggregation::computation_issue(d) {
        let mut result = FederalResult::empty(d.revision);
        crate::issue_localized!(
            &mut result,
            &crate::IssueContext::Section(crate::TaxSection::PassThrough),
            "QBI aggregation",
            key,
            "Selected QBI aggregation needs supported eligibility, election history and complete annual disclosures.",
        );
        return result;
    }

    if let Some(key) = crate::information_returns::receipt_rule_with_claims(d, source_claims) {
        let mut result = FederalResult::empty(d.revision);
        crate::issue_localized!(
            &mut result,
            &crate::IssueContext::Section(crate::TaxSection::SelfEmployment),
            "Schedule C",
            key,
            "Returns and allowances cannot exceed all gross receipts, including linked forms.",
        );
        return result;
    }
    // Items a No or a kept Not sure leaves out are removed from a working
    // copy first, so every rule below sees only what it calculates.
    let participation = d.direct_participation_view();
    let macrs_elections = alloc::sync::Arc::new(crate::macrs_source::MacrsElectionView::new(d));
    let mut notes = Vec::new();
    let crate::unknowns::SupportedScope { mut working, financial_source_scope_complete } =
        crate::unknowns::supported_with_scope(
            crate::related_forms::RelatedFormContext::bound(d, source_claims, &participation),
            &mut notes,
        );
    // Recompute current readonly coverage before worksheet selection. Historical
    // reconstructed returns retain their own source-year participation facts.
    let covered =
        [Owner::Taxpayer, Owner::Spouse].map(|owner| crate::interview::ira_covered(d, owner));
    if working
        .as_ref()
        .unwrap_or(d)
        .adjustments
        .as_ref()
        .and_then(|a| a.entered_iras())
        .is_some_and(|iras| {
            iras.taxpayer.covered_at_work != covered[0]
                || iras.spouse.as_ref().is_some_and(|spouse| spouse.covered_at_work != covered[1])
        })
    {
        let iras = &mut working.get_or_insert_with(|| d.clone()).adjustments.as_mut().unwrap().iras;
        iras.taxpayer.covered_at_work = covered[0];
        if let Some(spouse) = iras.spouse.as_mut() {
            spouse.covered_at_work = covered[1];
        }
    }
    // Resolve actual indirect rollover history and failed ordinary deposits once,
    // before HSA funding, retirement and all deduction-method candidates.
    let prepared = match crate::medical_rollover_return::prepare(working.as_ref().unwrap_or(d)) {
        Ok(prepared) => prepared,
        Err((source, key)) => {
            let mut result = FederalResult::empty(d.revision);
            let context = d
                .income
                .iter()
                .find(|e| e.id == source)
                .map(crate::interview::income_issue_context)
                .unwrap_or(crate::IssueContext::Section(crate::TaxSection::OtherIncome))
                .alternatives(&[crate::TaxSection::OtherIncome, crate::TaxSection::Hsa]);
            crate::issue_localized!(
                &mut result,
                &context,
                "Income",
                key,
                "Reconcile the actual medical-account rollover source, ordinary deposit designation and destination contribution facts before calculating required income or excess tax.",
            );
            result.state = ResultState::Unsupported;
            return result;
        }
    };
    // Source validation and correction links keep the original graph. Only
    // the distribution collector consumes the prepared medical amounts.
    let calculation_draft = working.as_ref().unwrap_or(d);
    let mut result = estimate_choices_with_claims(
        calculation_draft,
        financial_source_scope_complete,
        &prepared,
        source_claims,
        &participation,
        &macrs_elections,
        d,
    );
    for source in crate::farm::sources(calculation_draft) {
        if crate::farm_insurance_paper::missing(calculation_draft, source.entry.id, source_claims) {
            let context = crate::interview::income_issue_context(source.entry)
                .corrective(crate::TaxSection::Income);
            crate::issue_localized!(
                &mut result,
                &context,
                "Crop insurance",
                "CROP_PAPER_FACTS_NEEDED",
                "Crop-insurance proceeds are included in current income. Open this activity's crop-insurance details to answer its missing timing or physical-loss facts; only an established election can defer income."
            );
        }
    }
    // A completed payroll pass may retain independently known rows while
    // incomplete. Its unrelated mandatory income sources still need validation.
    let completed = result.state != ResultState::Incomplete || result.payroll_known_rows.is_some();
    for entry in &d.income {
        if completed
            && crate::investment_statements::net_income_unresolved(
                &entry.income,
                d.screening.as_ref().and_then(|s| s.taxable_premium_election),
            )
        {
            crate::issue_localized!(
                &mut result,
                &crate::interview::income_issue_context(entry),
                "Net bond interest",
                "ISSUE_DEBT_NET_GROSS_INCOME_UNKNOWN",
                "The net report does not establish gross taxable income without the documented retained taxable-premium offset.",
            );
            result.state = ResultState::Unsupported;
        }
    }
    if completed
        && d.credits.as_ref().is_some_and(|c| {
            c.tax_credit_bonds.iter().any(crate::FederalTaxCreditBond1097::income_unknown)
        })
    {
        crate::issue_localized!(
            &mut result,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Tax-credit bond income",
            "ISSUE_TAX_CREDIT_BOND_INCOME_UNKNOWN",
            "The direct bond's required income source or tax character is unresolved; credit omission alone does not establish ordinary federal tax.",
        );
        result.state = ResultState::Unsupported;
    }
    for note in notes {
        if !result.review_notes.iter().any(|existing| existing.same_notice(&note)) {
            result.review_notes.push(note);
        }
    }
    // A post-projection required-income limit has the same result contract as
    // an ordinary unsupported calculation: no apparent tax or return breakdown.
    if result.state == ResultState::Unsupported {
        withhold_required_source_result(&mut result);
    }
    result.financial_source_scope_complete &= result.state
        == ResultState::EstimatedForSupportedScope
        && result.tax.is_some()
        && result.payments.is_some()
        && result.refund.is_some()
        && result.balance.is_some();
    crate::schedule_b::report(d, calculation_draft, &mut result);
    crate::community::explain_with_claims(calculation_draft, &mut result, source_claims);
    crate::casualty::rental::report(
        crate::related_forms::RelatedFormContext::bound(
            calculation_draft,
            source_claims,
            &participation,
        )
        .with_macrs(&macrs_elections),
        &mut result,
    );
    for issue in &mut result.issues {
        issue.context = crate::interview::resolved_issue_context(d, &issue.context);
    }
    for note in &mut result.review_notes {
        if let Some(context) = &mut note.context {
            *context = crate::interview::resolved_issue_context(d, context);
        }
    }
    result
}

fn estimate_choices_with_claims(
    d: &FederalDraft,
    financial_source_scope_complete: bool,
    medical: &crate::medical_rollover_return::Prepared,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,

    participation: &alloc::sync::Arc<crate::direct_participation::DirectParticipationView>,

    macrs_elections: &alloc::sync::Arc<crate::macrs_source::MacrsElectionView>,
    source_draft: &FederalDraft,
) -> FederalResult {
    if let Some(resolved) = resolved_scholarship_choices_with_claims(
        d,
        financial_source_scope_complete,
        medical,
        source_claims,
        participation,
        macrs_elections,
        source_draft,
    ) {
        // Missing payroll tax prevents the automatic comparison from choosing
        // a final scholarship inclusion; its trial AGI is not certified.
        return unsettled_payroll_agi(estimate_choices_with_claims(
            &resolved,
            financial_source_scope_complete,
            medical,
            source_claims,
            participation,
            macrs_elections,
            source_draft,
        ));
    }
    if d.deductions
        .as_ref()
        .is_some_and(|v| v.repayment.method() == Some(crate::RepaymentMethod::Automatic))
    {
        let mut working = d.clone();
        working.deductions.as_mut().unwrap().repayment.select(crate::RepaymentMethod::Deduction);
        let deduction = estimate_choices_with_claims(
            &working,
            financial_source_scope_complete,
            medical,
            source_claims,
            participation,
            macrs_elections,
            source_draft,
        );
        working.deductions.as_mut().unwrap().repayment.select(crate::RepaymentMethod::Credit);
        let credit = estimate_choices_with_claims(
            &working,
            financial_source_scope_complete,
            medical,
            source_claims,
            participation,
            macrs_elections,
            source_draft,
        );
        if deduction.state != ResultState::EstimatedForSupportedScope {
            return unsettled_payroll_agi(deduction);
        }
        if credit.state != ResultState::EstimatedForSupportedScope {
            return unsettled_payroll_agi(credit);
        }
        let deduction_cost = deduction.tax.unwrap() - deduction.refundable_credits.unwrap();
        let credit_cost = credit.tax.unwrap() - credit.refundable_credits.unwrap();
        let mut result = if deduction_cost <= credit_cost { deduction } else { credit };
        result.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("Repayment comparison: deduction-method tax less refundable credits ${}; credit-method tax less refundable credits ${}. Selected {}. Withholding and estimated payments are unchanged between methods.", dollars(deduction_cost), dollars(credit_cost), if deduction_cost <= credit_cost {"deduction"} else {"credit"}),
            Message::new(if deduction_cost <= credit_cost { "NOTE_REPAYMENT_DEDUCTION_SELECTED" } else { "NOTE_REPAYMENT_CREDIT_SELECTED" }, alloc::vec![
                MessageArg::usd_dollars(dollars(deduction_cost)), MessageArg::usd_dollars(dollars(credit_cost)),
            ]),
        ));
        return result;
    }
    let home = d.income.iter().any(|entry| matches!(&entry.income,
        FederalIncome::Rental { data } if matches!(data.personal_use, crate::RentalPersonalUse::Yes { .. })));
    let first = estimate_route_with_claims(
        d,
        None,
        financial_source_scope_complete,
        medical,
        source_claims,
        participation,
        macrs_elections,
        source_draft,
    );
    if d.deduction_choice() != DeductionChoice::Automatic {
        return first;
    }
    if (first.state == ResultState::Incomplete || first.payroll_known_rows.is_some()) && !home {
        let changes_income = crate::home_office::has_actual(d)
            || d.income.iter().any(|e| {
                matches!(&e.income, FederalIncome::Rental { data }
                if matches!(data.personal_use, crate::RentalPersonalUse::Yes { .. }))
            });
        return if changes_income { unsettled_payroll_agi(first) } else { first };
    }
    // Compare final liability, including AMT and refundable credits, with the same
    // calculation engine. Explicit user choices are never overridden.
    // An actual-expense home office uses different Form 8829 columns on each
    // route; the transient invocation context supplies that route to the home
    // owner. The unforced first run is not a candidate for an actual home office.
    let office = crate::home_office::has_actual(d);
    let (itemized, standard) = (
        estimate_route_with_claims(
            d,
            Some(true),
            financial_source_scope_complete,
            medical,
            source_claims,
            participation,
            macrs_elections,
            source_draft,
        ),
        estimate_route_with_claims(
            d,
            Some(false),
            financial_source_scope_complete,
            medical,
            source_claims,
            participation,
            macrs_elections,
            source_draft,
        ),
    );
    let unavailable = itemized.state != ResultState::EstimatedForSupportedScope
        || standard.state != ResultState::EstimatedForSupportedScope;
    if unavailable && d.deductions.as_ref().is_some_and(|v| v.repayment.method().is_some()) {
        return if itemized.state != ResultState::EstimatedForSupportedScope {
            itemized
        } else {
            standard
        };
    }
    let seed = if office || home {
        let mut seed = first;
        seed.tax = None;
        seed
    } else {
        first
    };
    let fallback = (office || home).then(|| itemized.clone());
    // Equal final tax retains the ordinary deduction owner's preference.
    // A forced empty itemized route must not displace an eligible standard
    // deduction merely because that candidate was evaluated first.
    let (mut best, _) = [(true, itemized), (false, standard)].into_iter().fold(
        (seed, true),
        |(best, best_matches_default), (itemizes, candidate)| {
            let matches_default = itemizes == candidate.automatic_itemizes;
            if let (Some(tax), Some(refundable)) = (candidate.tax, candidate.refundable_credits) {
                let current = best.tax.zip(best.refundable_credits).map(|(t, c)| t - c);
                if current.is_none_or(|v| {
                    tax - refundable < v
                        || (tax - refundable == v && matches_default && !best_matches_default)
                }) {
                    return (candidate, matches_default);
                }
            }
            (best, best_matches_default)
        },
    );
    // Neither route produced a tax: report the itemizing route's issues.
    if best.tax.is_none()
        && let Some(itemized) = fallback
    {
        best = itemized;
    }
    if unavailable && best.state == ResultState::EstimatedForSupportedScope {
        best.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Automatic deduction selection compared supported routes only. One route was unavailable; review its inputs and coverage before assuming this is the lowest possible tax.", crate::Message::new("NOTE_AUTOMATIC_DEDUCTION_SELECTION_COMPARED_SUPPORTED_ROUTES_ONLY_ONE_D549B510", alloc::vec![])));
    }
    best
}

// Call-scoped calculation context: metadata follows every native candidate,
// including existing recursive routes, without adding another estimate pass.
#[derive(Clone, Copy)]
struct EstimateContext<'a> {
    source_claims: &'a alloc::collections::BTreeMap<i64, i64>,
    medical: &'a crate::medical_rollover_return::Prepared,
    force_itemized: Option<bool>,
    financial_source_scope_complete: bool,
    participation: &'a alloc::sync::Arc<crate::direct_participation::DirectParticipationView>,
    macrs_elections: &'a alloc::sync::Arc<crate::macrs_source::MacrsElectionView>,
    // Complete accepted or submitted sources, before conservative benefit omission.
    source_draft: &'a FederalDraft,
}
impl EstimateContext<'_> {
    fn forms<'a>(
        &'a self,
        draft: &'a FederalDraft,
    ) -> crate::related_forms::RelatedFormContext<'a> {
        crate::related_forms::RelatedFormContext::bound(
            draft,
            self.source_claims,
            self.participation,
        )
        .with_macrs(self.macrs_elections)
    }
}

#[allow(clippy::too_many_arguments)]
fn estimate_route_with_claims(
    d: &FederalDraft,
    force_itemized: Option<bool>,
    financial_source_scope_complete: bool,
    medical: &crate::medical_rollover_return::Prepared,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,

    participation: &alloc::sync::Arc<crate::direct_participation::DirectParticipationView>,

    macrs_elections: &alloc::sync::Arc<crate::macrs_source::MacrsElectionView>,
    source_draft: &FederalDraft,
) -> FederalResult {
    let projection = crate::estate_trust::losses(d, false);
    let has_nol = !projection.vintages.is_empty();
    let route_dependent_refund = d.income.iter().any(|entry| {
        matches!(&entry.income, FederalIncome::GovernmentPayment { data }
            if matches!(&data.history, TaxRefundHistory::IncomeTaxPriorYearsAndCurrent { .. }))
    });
    // These calculations need a resolved route before choosing a deduction.
    // Seed automatic comparison with a complete standard candidate; the
    // existing comparison also evaluates the complete itemized candidate.
    let force_itemized = force_itemized.or_else(|| {
        ((has_nol || route_dependent_refund) && d.deduction_choice() == DeductionChoice::Automatic)
            .then_some(false)
    });
    let context = EstimateContext {
        source_claims,
        medical,
        force_itemized,
        financial_source_scope_complete,
        participation,
        macrs_elections,
        source_draft,
    };
    let mut run = crate::nol::NolRun { exclude_qbi: has_nol, ..Default::default() };
    let mut first = estimate_with_ira_with_claims(d, context, None, None, None, &mut run);
    if !has_nol {
        return first;
    }
    let Some(base) = run.taxable_base else {
        return first;
    };
    let mut unresolved = None;
    let allocation = NolHistory::allocate(base, &projection.vintages, |earlier| {
        nol_modified_income_with_claims(d, context, earlier, None, None, &mut unresolved)
    });
    if let Some(result) = unresolved {
        return result;
    }
    let mut allocation = match allocation {
        Ok(allocation) => allocation,
        Err(issue) => {
            issue.report(&mut first);
            return first;
        }
    };
    allocation.set_sources(projection.single_estate, projection.has_estate);
    run.exclude_qbi = false;
    run.deduction = allocation.deduction;
    run.allocation = Some(allocation);
    // Section 613A(d)(1) disregards only NOL carrybacks, so the oil and gas
    // 65% limit reads taxable income after this year's NOL deduction, while
    // the section 172(a)(2) 80% limit reads taxable income after that
    // depletion: the projection reruns with the deduction and the deduction
    // with the resulting base until both settle.
    let mut base = base;
    let mut actual = estimate_with_ira_with_claims(d, context, None, None, None, &mut run);
    for _ in 0..16 {
        let Some(next) =
            run.taxable_base.filter(|next| run.royalty_limits.is_some() && *next != base)
        else {
            break;
        };
        base = next;
        let allocation = NolHistory::allocate(base, &projection.vintages, |earlier| {
            nol_modified_income_with_claims(d, context, earlier, None, None, &mut unresolved)
        });
        if let Some(result) = unresolved {
            return result;
        }
        let mut allocation = match allocation {
            Ok(allocation) => allocation,
            Err(issue) => {
                issue.report(&mut actual);
                return actual;
            }
        };
        allocation.set_sources(projection.single_estate, projection.has_estate);
        run.deduction = allocation.deduction;
        run.allocation = Some(allocation);
        run.royalty_limits = None;
        actual = estimate_with_ira_with_claims(d, context, None, None, None, &mut run);
    }
    if actual.state != ResultState::EstimatedForSupportedScope || run.charity_carry == 0 {
        return actual;
    }
    // Section 170(d)(1)(B): remove only the charitable excess that has
    // increased the surviving NOL carryover. Keep all other modifications,
    // percentage limits and original NOL deduction limits unchanged.
    let without_excess = NolHistory::allocate(base, &projection.vintages, |earlier| {
        nol_modified_income_with_claims(
            d,
            context,
            earlier,
            Some(run.charity_deduction),
            None,
            &mut unresolved,
        )
    });
    if let Some(result) = unresolved {
        return result;
    }
    let without_excess = match without_excess {
        Ok(allocation) => allocation,
        Err(issue) => {
            issue.report(&mut actual);
            return actual;
        }
    };
    run.charity_nol_reduction = crate::kernel::ratio::exact::rounded(&phase::cap(
        phase::floor(
            &run.allocation.as_ref().unwrap().carryforward - &without_excess.carryforward,
            crate::kernel::ratio::exact::whole(0),
        ),
        [crate::kernel::ratio::exact::whole(run.charity_carry)],
    ))
    .expect("charitable debit is bounded by its native carry");
    if run.charity_nol_reduction == 0 {
        return actual;
    }
    // Reg1.170A-10(d)(1): section170(b)(1)(A) organizations precede
    // other organizations in NOL absorption. Within those groups retain the
    // existing class worksheet order. These finite source-owner projections
    // attribute the already-established aggregate endpoint, never a ratio of
    // gross carryovers or a readback of presentation rows.
    let mut mask = 0;
    let mut previous = without_excess.carryforward;
    let mut remaining = run.charity_nol_reduction;
    for i in [0, 1, 3, 5, 6, 2, 4, 7] {
        mask |= 1 << i;
        if run.charity_class_carry[i] == 0 {
            continue;
        }
        let next = NolHistory::allocate(base, &projection.vintages, |earlier| {
            nol_modified_income_with_claims(
                d,
                context,
                earlier,
                Some(run.charity_deduction),
                Some((run.charity_class_deductions, mask)),
                &mut unresolved,
            )
        });
        if let Some(result) = unresolved {
            return result;
        }
        let next = match next {
            Ok(allocation) => allocation,
            Err(issue) => {
                issue.report(&mut actual);
                return actual;
            }
        };
        let debit = crate::kernel::ratio::exact::rounded(&phase::cap(
            phase::floor(&next.carryforward - &previous, crate::kernel::ratio::exact::whole(0)),
            [
                crate::kernel::ratio::exact::whole(remaining),
                crate::kernel::ratio::exact::whole(run.charity_class_carry[i]),
            ],
        ))
        .expect("class debit is bounded by its native remaining carry");
        run.charity_class_reduction[i] = debit;
        remaining -= debit;
        previous = next.carryforward;
    }
    assert_eq!(remaining, 0, "charitable NOL attribution must conserve its established endpoint");
    estimate_with_ira_with_claims(d, context, None, None, None, &mut run)
}

// Preserve the selected Step4 calculation, including its own household-income
// provenance. The final pass applies Step5 health deductions without replacing
// Step4 PTC by an unrequested additional iteration.
#[derive(Clone)]
struct OwnerSnapshot<T> {
    outcome: T,
    output: FederalResult,
}
impl<T> OwnerSnapshot<T> {
    fn append(&self, result: &mut FederalResult) {
        result.lines.extend(self.output.lines.iter().cloned());
        result.issues.extend(self.output.issues.iter().cloned());
        result.review_notes.extend(self.output.review_notes.iter().cloned());
    }
}

// Native earnings sources retained by the existing return coordinator. Direct
// operating allowances remain separate for regular tax and AMT; partnership
// SE packets keep their own established source shape.
#[derive(Clone, Default, PartialEq, Eq)]
struct EarningsInputs {
    partners: Vec<(i64, crate::partnership::PartnerSeResult)>,
    direct: [alloc::collections::BTreeMap<i64, i64>; 2],
}

#[derive(Default)]
struct PartnerPass {
    // Heap-owned with the existing pass: retained rental snapshots and trials
    // consume the same selected allocations. A changed care projection clears
    // this pair along with the corresponding source snapshots below.
    section179: Option<alloc::boxed::Box<[crate::section179_return::Selected; 2]>>,
    // Retain the original source class before resolved() materializes it as cost.
    percentage_depletion_source: Option<i64>,
    care: crate::care::CareDeductions,
    care_next: Option<crate::care::CareDeductions>,
    // Reuse inferred repayment history after the large income frame returns.
    repayment_next: Option<alloc::boxed::Box<FederalDraft>>,
    // Coordinate the same adoption pass after the large income frame returns.
    adoption_next: Option<crate::adoption::Benefits>,
    contribution_next: Option<crate::ira::IraResult>,
    inputs: EarningsInputs,
    next: Option<EarningsInputs>,
    // Pub. 974 Simplified Calculation Method: None runs Steps 1 and 2 with the
    // largest self-employed deduction for Marketplace premiums; Step 3's
    // deduction then runs Step 4, the final pass.
    specified: Option<alloc::collections::BTreeMap<i64, crate::kernel::ratio::exact::Exact>>,
    specified_next: Option<alloc::collections::BTreeMap<i64, crate::kernel::ratio::exact::Exact>>,
    iterative: bool,
    iteration: crate::health_ptc_iteration::Iteration,
    // Boxed: the pass nests inside the royalty projection's recursive call.
    marketplace_trial: Option<alloc::boxed::Box<MarketplaceSnapshot>>,
    marketplace_final: Option<alloc::boxed::Box<MarketplaceSnapshot>>,
    simplified_fallback: bool,
    specials: alloc::boxed::Box<HealthSpecials>,
    health_method_complete: bool,
    health_usage_valid: bool,
    // The Form 8815 exclusion that reduces educator expenses (IRC 62(d)(2)). The exclusion reads
    // an AGI that contains the educator deduction, so pass one deducts the expenses before it
    // and a second pass reduces them by that exclusion.
    bond_exclusion: i64,
    bond_next: Option<i64>,
    // One no-depletion taxable-income projection supplies each 65% limit.
    royalty_limits: Option<[i64; 2]>,
    royalty_next: Option<[i64; 2]>,
    depletion_regular: Vec<crate::royalties::Depletion>,
    depletion_amt: Vec<crate::royalties::Depletion>,
    // The home-use worksheet shares final AGI floors with the normal return pass.
    home_agi: Option<crate::extended_rules::HomeUseContext>,
    home_next: Option<crate::extended_rules::HomeUseContext>,
    // Runtime-only Form 8829 projection; accepted home source facts never
    // store the AGI or SALT cap produced by this return's calculation.
    office: crate::home_office::HomeContext,
    office_next: Option<crate::home_office::HomeContext>,
}

fn estimate_with_ira_with_claims(
    d: &FederalDraft,
    context: EstimateContext<'_>,
    contribution_result: Option<&crate::ira::IraResult>,
    qcd_contributions: Option<&crate::ira::IraResult>,
    adoption_result: Option<&crate::adoption::Benefits>,
    nol_run: &mut crate::nol::NolRun,
) -> FederalResult {
    let percentage_depletion_source = d.income.iter().find_map(|entry| {
        matches!(&entry.income, FederalIncome::Business { data }
            if data.depletion.as_ref().is_some_and(|source| matches!(source.depletion, RoyaltyDepletion::Percentage { .. })))
            .then_some(entry.id)
    });
    estimate_with_ira_from_source_with_claims(
        d,
        context,
        contribution_result,
        qcd_contributions,
        adoption_result,
        nol_run,
        percentage_depletion_source,
    )
}

fn estimate_with_ira_from_source_with_claims(
    d: &FederalDraft,
    context: EstimateContext<'_>,
    contribution_result: Option<&crate::ira::IraResult>,
    qcd_contributions: Option<&crate::ira::IraResult>,
    adoption_result: Option<&crate::adoption::Benefits>,
    nol_run: &mut crate::nol::NolRun,
    percentage_depletion_source: Option<i64>,
) -> FederalResult {
    if !nol_run.royalty_projection && nol_run.royalty_limits.is_none()
        && d.income.iter().any(|entry| matches!(&entry.income, FederalIncome::Royalty { data } if matches!(data.kind, RoyaltyKind::OilGas)) || matches!(&entry.income,FederalIncome::Business {data} if data.depletion.as_ref().is_some_and(|v|matches!(v.kind,crate::RoyaltyKind::OilGas))))
    {
        // Run the same complete income/deduction worksheets without oil/gas
        // depletion, QBI or NOL. Adding depletion/NOL back after the fact would
        // retain the wrong AGI floors, phaseouts and section 68 reduction.
        let mut projection = crate::nol::NolRun {
            royalty_projection: true,
            exclude_qbi: true,
            deduction: nol_run.deduction,
            ..Default::default()
        };
        let projected = estimate_with_ira_from_source_with_claims(d, context, None, None, None, &mut projection, percentage_depletion_source);
        let Some(limits) = projection.royalty_limits else { return projected; };
        nol_run.royalty_limits = Some(limits);
    }
    let (Some(regular), Some(amt)) = (
        crate::royalties::calculate_with_claims(
            context.forms(d),
            nol_run.royalty_limits.map(|v| v[0]),
            false,
        ),
        crate::royalties::calculate_with_claims(
            context.forms(d),
            nol_run.royalty_limits.map(|v| v[1]),
            true,
        ),
    ) else {
        if d.stored_issue().is_some() {
            // Preserve the existing source-validation refusal before classifying
            // a failed provisional depletion as a native-output failure.
            return estimate_with_ira_pass_with_claims(
                d,
                context,
                contribution_result,
                qcd_contributions,
                adoption_result,
                nol_run,
                &mut PartnerPass::default(),
            );
        }
        let mut result = FederalResult::empty(d.revision);
        crate::extended_rules::expense_output_unavailable(&mut result, None);
        return result;
    };
    let resolved = crate::schedule_c_specials::resolved(d, &regular);
    let d = &resolved;
    let mut pass = PartnerPass {
        percentage_depletion_source,
        // Form 8829 Line 11 Worksheet starts its complete-return SALT
        // iteration at the statutory floor, before calculating a new cap.
        office: crate::home_office::HomeContext {
            itemizes: context.force_itemized,
            salt_cap_cents: if d
                .setup
                .as_ref()
                .is_some_and(|setup| setup.filing_status() == FederalStatus::Separate)
            {
                500_000
            } else {
                1_000_000
            },
            ..Default::default()
        },
        depletion_regular: regular,
        depletion_amt: amt,
        royalty_limits: nol_run.royalty_limits,
        iterative: d
            .adjustments
            .as_ref()
            .is_some_and(|a| a.health_marketplace.method == crate::HealthPtcMethod::Iterative),
        ..Default::default()
    };
    let mut seen = vec![(pass.inputs.clone(), pass.care.clone())];
    let mut care_attempts = 0;
    let mut partner_attempts = 0;
    let mut office_seen = vec![pass.office.clone()];
    loop {
        pass.next = None;
        pass.specified_next = None;
        pass.marketplace_trial = None;
        pass.bond_next = None;
        pass.health_method_complete = false;
        pass.health_usage_valid = true;
        pass.office_next = None;
        let mut result = estimate_with_ira_pass_with_claims(
            d,
            context,
            contribution_result,
            qcd_contributions,
            adoption_result,
            nol_run,
            &mut pass,
        );
        if let Some(next) = pass.repayment_next.take() {
            let mut result = estimate_with_ira_from_source_with_claims(
                &next,
                context,
                contribution_result,
                qcd_contributions,
                adoption_result,
                nol_run,
                percentage_depletion_source,
            );
            result.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Current carryover inputs reuse complete original 2025 repayment worksheets; duplicate entry is unnecessary. Additional investment-loss source details may still be required.", crate::Message::new("NOTE_CURRENT_CARRYOVER_INPUTS_REUSE_COMPLETE_ORIGINAL_2025_REPAYMENT_2F25B188", alloc::vec![])));
            return result;
        }
        if let Some(next) = pass.contribution_next.take() {
            return estimate_with_ira_from_source_with_claims(
                d,
                context,
                Some(&next),
                qcd_contributions,
                adoption_result,
                nol_run,
                percentage_depletion_source,
            );
        }
        if let Some(next) = pass.adoption_next.take() {
            return estimate_with_ira_from_source_with_claims(
                d,
                context,
                None,
                None,
                Some(&next),
                nol_run,
                percentage_depletion_source,
            );
        }
        if let Some(next) = pass.care_next.take() {
            care_attempts += 1;
            if seen.contains(&(pass.inputs.clone(), next.clone())) || care_attempts == 64 {
                crate::issue_localized!(
                    &mut result,
                    &crate::IssueContext::Section(crate::TaxSection::Care),
                    "Care deduction coordination",
                    "ISSUE_CARE_DEDUCTION_COORDINATION",
                    "The source deductions and Form 2441 earned-income limit did not stabilize; no final tax can be reported.",
                );
                result.state = ResultState::Unsupported;
                result.lines.clear();
                result.return_lines.clear();
                return result;
            }
            seen.push((pass.inputs.clone(), next.clone()));
            pass.care = next;
            pass.section179 = None;
            // Source expenses changed. Reuse the normal coordinator, with no
            // retained health or passive snapshot from the preceding context.
            pass.specified = None;
            pass.marketplace_final = None;
            pass.iteration = Default::default();
            *pass.specials = Default::default();
            continue;
        }
        let Some(next) = pass.next.take() else {
            if let Some(office) = pass.office_next.take() {
                // Bound the calculation's convergence work, never the number
                // of accepted homes. A repeated trial cannot establish a
                // consistent final return and must not contribute a tax.
                if office_seen.contains(&office) || office_seen.len() == 64 {
                    crate::issue_localized!(
                        &mut result,
                        &crate::IssueContext::Section(crate::TaxSection::HomeOffices),
                        "Home-office income coordination",
                        "HOME_OFFICE_AGI_UNRESOLVED",
                        "The home-office worksheets and final adjusted gross income did not stabilize. No final tax can be reported.",
                    );
                    result.state = ResultState::Unsupported;
                    result.lines.clear();
                    result.return_lines.clear();
                    return result;
                }
                office_seen.push(office.clone());
                pass.office = office;
                pass.section179 = None;
                // Every subsequent worksheet must use this same home-use
                // context. Retained health trials and cycle observations
                // from the preceding income projection are not reusable.
                pass.specified = None;
                pass.marketplace_final = None;
                pass.iteration = Default::default();
                *pass.specials = Default::default();
                seen.clear();
                seen.push((pass.inputs.clone(), pass.care.clone()));
                care_attempts = 0;
                partner_attempts = 0;
                continue;
            }
            if let Some(home_agi) = pass.home_next.take() {
                if partner_attempts == 63 {
                    crate::issue_localized!(
                        &mut result,
                        &crate::IssueContext::Section(crate::TaxSection::Rentals),
                        "Home-use AGI coordination",
                        "RENTAL_HOME_AGI_UNRESOLVED",
                        "The home-use worksheet and final AGI did not stabilize. No final tax can be reported.",
                    );
                    result.state = ResultState::Unsupported;
                    result.lines.clear();
                    result.return_lines.clear();
                    return result;
                }
                pass.home_agi = Some(home_agi);
                pass.specials.rental = None;
                pass.specials.rental_trial = None;
                continue;
            }
            if let Some(specified) = pass.specified_next.take() {
                if pass.iterative {
                    let current = pass
                        .marketplace_trial
                        .take()
                        .expect("specified calculation has PTC snapshot");
                    match pass.iteration.observe(current.outcome.calculation_credit, &specified) {
                        crate::health_ptc_iteration::Decision::Continue => {
                            pass.specified = Some(specified);
                            continue;
                        }
                        crate::health_ptc_iteration::Decision::Accept => {
                            pass.specified = Some(specified);
                            pass.marketplace_final = Some(current);
                            continue;
                        }
                        crate::health_ptc_iteration::Decision::Simplified => {
                            pass.iterative = false;
                            pass.simplified_fallback = true;
                            pass.specified = None;
                            pass.iteration = Default::default();
                            // Start Simplified Step1, preserving only the same
                            // independently stabilized partnership inputs.
                            pass.bond_exclusion = 0;
                            *pass.specials = Default::default();
                            continue;
                        }
                    }
                } else if pass.specified.is_none() {
                    pass.specified = Some(specified);
                    continue;
                }
            }
            if let Some(excluded) = pass.bond_next.take() {
                pass.bond_exclusion = excluded;
                pass.specified = None;
                pass.marketplace_final = None;
                pass.iteration = Default::default();
                *pass.specials = Default::default();
                continue;
            }
            if pass.health_method_complete && pass.specials.advance() {
                pass.specified = None;
                pass.marketplace_final = None;
                pass.iteration = Default::default();
                continue;
            }
            if pass.simplified_fallback {
                note_once_scoped(
                    &mut result,
                    Message::new("NOTE_HEALTH_PTC_SIMPLIFIED_FALLBACK", alloc::vec![]),
                    crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits),
                );
            }
            if !pass.health_usage_valid {
                crate::issue_localized!(
                    &mut result,
                    &crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits)
                        .corrective(crate::TaxSection::Marketplace),
                    "Marketplace source expenses",
                    "ISSUE_HEALTH_MARKETPLACE_SOURCE_OVERALLOCATION",
                    "The selected health deduction and retained premium subsidy exceed a linked plan's available source expenses, or its subsidy allocation exceeds the actual enrollment source. No complete estimate can be reported from an overallocated source.",
                );
                result.state = ResultState::Unsupported;
                result.lines.clear();
                result.return_lines.clear();
            }
            if let Some(limits) = pass.royalty_next.take() {
                nol_run.royalty_limits = Some(limits);
                return result;
            }
            return result;
        };
        partner_attempts += 1;
        if seen.contains(&(next.clone(), pass.care.clone())) || partner_attempts == 64 {
            crate::issue_localized!(
                &mut result,
                &crate::IssueContext::Section(crate::TaxSection::PassThrough),
                "Partnership deduction coordination",
                "ISSUE_PARTNERSHIP_EARNINGS_AND_INCOME_DEPENDENT_DEDUCTIONS_DID_NOT_STABIL_F1B9D70E",
                "Partnership earnings and income-dependent deductions did not stabilize to one consistent calculation. No final tax can be reported from these intermediate amounts.",
            );
            result.state = ResultState::Unsupported;
            result.lines.clear();
            result.return_lines.clear();
            return result;
        }
        seen.push((next.clone(), pass.care.clone()));
        pass.inputs = next;
        // A changed earnings owner invalidates both the previous method pair
        // and its source limits. Never mix PTC from one earnings context with
        // deductions from a different partnership trial.
        pass.specified = None;
        pass.marketplace_final = None;
        pass.iteration = Default::default();
        *pass.specials = Default::default();
    }
}

// Build the mandatory current-source context only on the changed-filer path.
// Keeping its scratch map in a separate call also avoids adding that stack
// frame to ordinary interview estimates which do not need this projection.
fn changed_filer_credit_ratios(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    r: &FederalResult,
    facts: &crate::injured_spouse::JointFacts<'_>,
    current_totals: impl FnOnce() -> crate::injured_spouse::owners::LiabilityTotals,
) -> Option<[i64; 2]> {
    let d = form_context.draft;
    let totals = current_totals();
    crate::injured_spouse::owners::project_sources(
        form_context,
        r,
        d.injured_spouse.as_ref(),
        facts,
        Some(&totals),
    )
    .and_then(|projection| crate::injured_spouse::separate_taxes(d, &projection))
    .map(|taxes| {
        let first = fraction(10_000, taxes[0], taxes.iter().sum());
        [first, 10_000 - first]
    })
}

// The read-only context of one estimate pass, built once after its setup and before
// collect: per-form phase functions read it beside the mutable result, totals and
// partner-pass state. A field a phase does not read yet keeps a temporary allow.
pub(super) struct Pass<'a> {
    d: &'a FederalDraft,
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    care_original: &'a FederalDraft,
    context: EstimateContext<'a>,
    allocated_entries: &'a [alloc::borrow::Cow<'a, crate::FederalIncomeEntry>],
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    office_context: &'a crate::home_office::HomeContext,
    s: &'a FederalSetup,
    p: &'a FederalPeople,
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    a: &'a FederalAdjustments,
    ded: &'a FederalDeductions,
    c: &'a FederalCredits,
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    screen: &'a FederalScreening,
    pay: &'a FederalPayments,
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    social_election: &'a crate::SocialElection,
    student_catalog: &'a [PersonChoice],
    pension: &'a crate::pension_contributions::Contributions,
    repayment: &'a crate::claim_repayment::RepaymentResult,
    j: bool,
    separate: bool,
    dependent_filer: bool,
    family: &'a Family,
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    qcd: &'a crate::retirement_return::QcdResult,
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    basis: &'a crate::retirement_return::BasisResult,
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    selected_regular: Option<&'a crate::Section179PreparedAllocation>,
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    selected_alternative: Option<&'a crate::Section179PreparedAllocation>,
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    contribution_result: Option<&'a crate::ira::IraResult>,
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    qcd_contributions: Option<&'a crate::ira::IraResult>,
    #[allow(dead_code)] // Read once its phase moves out of the pass.
    adoption_result: Option<&'a crate::adoption::Benefits>,
}

fn estimate_with_ira_pass_with_claims(
    d: &FederalDraft,
    context: EstimateContext<'_>,
    contribution_result: Option<&crate::ira::IraResult>,
    qcd_contributions: Option<&crate::ira::IraResult>,
    adoption_result: Option<&crate::adoption::Benefits>,
    nol_run: &mut crate::nol::NolRun,
    partner_pass: &mut PartnerPass,
) -> FederalResult {
    let source_claims = context.source_claims;
    // All direct and prepared consumers read one immutable home projection
    // during this pass; the outer coordinator alone advances it.
    partner_pass.office.bind_macrs(context.macrs_elections);
    let mut office_context = partner_pass.office.clone();
    office_context.bind_participation(context.participation);
    office_context.bind_macrs(context.macrs_elections);
    let allocated_entries = crate::community::entries(d);
    let mut r = FederalResult::empty(d.revision);
    macro_rules! medical_native {
        ($amount:expr) => {{
            match crate::medical_account::required_output(
                &$amount,
                crate::IssueContext::Section(crate::TaxSection::OtherIncome)
                    .alternatives(&[crate::TaxSection::OtherIncome, crate::TaxSection::Hsa]),
            ) {
                Ok(amount) => amount,
                Err(issue) => {
                    issue.report(&mut r);
                    return r;
                }
            }
        }};
    }
    macro_rules! medical_dollars {
        ($amount:expr) => {{
            let dollars = exact::dollars_exact(&$amount);
            medical_native!(&dollars * exact::whole(100));
            medical_native!(dollars)
        }};
    }
    if let Some((id, issue)) =
        crate::employee_wages_admission::return_issue_with_claims(d, source_claims)
    {
        let entry = d.income.iter().find(|entry| entry.id == id).expect("source range owner");
        crate::issue_localized!(
            &mut r,
            &crate::interview::income_issue_context(entry),
            "Employee payroll",
            issue.key,
            "The employee payroll source aggregate exceeds the supported monetary range."
        );
        return r;
    }
    if let Err((_source, key)) = crate::contribution_documents::project(d) {
        crate::issue_localized!(
            &mut r,
            &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
            "Form 5498-SA contribution records",
            key,
            "Use the original statement and actual custodian, employer and transfer records to resolve this contribution source before calculating its tax effects.",
        );
        return r;
    }
    if let Some(failure) = d.stored_issue() {
        if failure.has_field_prefix(&["payments"]) {
            crate::issue_localized!(
                &mut r,
                &crate::IssueContext::Section(crate::TaxSection::Payments),
                "Estimated payments",
                failure.key,
                "Retain an honest native projection of the actual payment sources.",
            );
            r.state = ResultState::Unsupported;
            return r;
        }
        // A record whose stored values fall outside their ranges names its own
        // page (F3, 2026-10-09); other invalid data stays a manual setup issue.
        let context =
            allocated_entries.iter().find(|entry| validate_income(entry).is_err()).map_or_else(
                || (crate::IssueContext::Section(crate::TaxSection::Setup)).manual(),
                |entry| crate::interview::income_issue_context(entry),
            );
        if failure.key == crate::k1_activity::ordinary::RANGE_KEY && failure.field.is_some() {
            crate::issue_localized!(
                &mut r,
                &context,
                "Saved return",
                crate::k1_activity::ordinary::RANGE_KEY,
                "Review the document amounts; they must be within the supported input range.",
            );
            return r;
        }
        crate::issue_localized!(
            &mut r,
            &context,
            "Saved return",
            "ISSUE_ACCEPTED_DATA_IS_INVALID_NO_CALCULATION_WAS_PERFORMED_7A345C87",
            "Accepted data is invalid; no calculation was performed.",
        );
        return r;
    }
    if let Some(key) =
        crate::health_employer::mandatory_rule(d, crate::health_employer::bindings(d))
    {
        crate::issue_localized!(
            &mut r,
            &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
            "Employer medical-account contribution",
            key,
            "Resolve the actual W-2 employer contribution year and matching current account before completing this estimate.",
        );
    }
    let mut afs = None;
    let mut consistent_afs = true;
    for entry in &allocated_entries {
        let mut facts = Vec::new();
        match &entry.income {
            FederalIncome::Business { data } => {
                if let Some(v) = &data.de_minimis {
                    facts.push(v.financial_statement);
                }
                facts.extend(data.expenses.iter().filter_map(|v| match v.treatment {
                    crate::BusinessExpenseKind::DeMinimis { financial_statement, .. } => {
                        Some(financial_statement)
                    }
                    _ => None,
                }));
            }
            FederalIncome::Rental { data } => {
                facts.extend(data.expenses.iter().filter_map(|v| match v.treatment {
                    crate::BusinessExpenseKind::DeMinimis { financial_statement, .. } => {
                        Some(financial_statement)
                    }
                    _ => None,
                }))
            }
            _ => {}
        }
        for fact in facts {
            if afs.is_some_and(|prior| prior != fact) {
                consistent_afs = false;
            }
            afs = Some(fact);
        }
    }
    crate::require_localized!(
        &mut r,
        consistent_afs,
        &crate::IssueContext::Section(crate::TaxSection::SelfEmployment),
        "De minimis election",
        "ISSUE_USE_CONSISTENT_APPLICABLE_FINANCIAL_STATEMENT_FACTS_FOR_THIS_ACTIVI_5F5D61F3",
        "Use the same applicable-financial-statement facts for all de minimis purchases on this return.",
    );
    if d.first_incomplete() != 9 || d.payments.is_none() {
        crate::issue_localized!(
            &mut r,
            &crate::IssueContext::Interview,
            "Interview",
            "ISSUE_COMPLETE_THE_REMAINING_APPLICABLE_PAGES_TO_CALCULATE_THE_FEDERAL_ES_CAC8B62D",
            "Complete the remaining applicable pages to calculate the federal estimate.",
        );
        return r;
    }
    // The saved facts were checked above. Only this pass's working business
    // expenses receive the calculated deduction; the saved return is unchanged.
    let care_original = d;
    let care_projection = boxed(|| partner_pass.care.business_projection(d));
    let d = care_projection.as_ref();
    // Later source consumers read the actual expense projection used by this
    // pass, rather than retaining a pre-care allocation snapshot.
    let allocated_entries = crate::community::entries(d);
    let (s, p, a, ded, c, screen, pay) = (
        d.setup.as_ref().unwrap(),
        d.people.as_ref().unwrap(),
        d.adjustments.as_ref().unwrap(),
        d.deductions.as_ref().unwrap(),
        d.credits.as_ref().unwrap(),
        d.screening.as_ref().unwrap(),
        d.payments.as_ref().unwrap(),
    );
    let social_election = a.social_election.bound_to_printed(d);
    crate::require_localized!(
        &mut r,
        social_election.printed_years_complete(d),
        &crate::IssueContext::Section(crate::TaxSection::Retirement),
        "Social Security lump-sum election",
        "DOCUMENT_SOCIAL_PRIOR_YEAR_FACTS",
        "To compare the lump-sum election, provide prior-return facts for each year identified in the benefit statements. The current receipt amounts come from those printed descriptions.",
    );
    // Cross-page completeness must not invalidate the saved People draft.
    // The ordinary Care editor owns the same completeness checks on Save.
    if let Some(key) = c.care.rule(d) {
        crate::issue_localized!(
            &mut r,
            &crate::IssueContext::Section(crate::TaxSection::Care),
            "Child and dependent care",
            key,
            "Complete the care provider and plan facts on Child and dependent care.",
        );
        return r;
    }
    for (student, key) in c.education_pending() {
        let position = c.students.iter().position(|row| row.student == student).unwrap_or(0);
        crate::issue_localized!(
            &mut r,
            &crate::IssueContext::Section(crate::TaxSection::Education).item("students", position),
            "Education",
            key,
            "Complete the annual education credit and eligibility answers for this student.",
        );
    }
    // The people a student can be, from the people page itself.
    let student_catalog = people_students(d);
    if let Some(index) = c
        .education
        .iter()
        .position(|entry| !student_catalog.iter().any(|student| student.id == entry.student))
    {
        crate::issue_localized!(
            &mut r,
            &crate::IssueContext::Section(crate::TaxSection::Education)
                .item("students", index)
                .corrective(crate::TaxSection::Documents),
            "Education",
            "MESSAGE_A_STUDENT_IS_NOT_ON_THIS_RETURN_CHOOSE_THE_STUDENT_AGAIN",
            "A student is not on this return; choose the student again.",
        );
        return r;
    }
    crate::require_localized!(
        &mut r,
        s.filing_status() == FederalStatus::Separate
            || !matches!(s.community_property, crate::CommunityProperty::Yes { .. }),
        &crate::IssueContext::Section(crate::TaxSection::Setup),
        "Community-property allocation",
        "ISSUE_COMMUNITY_FILING_SCOPE",
        "A community-property answer applies only to a married-filing-separately return.",
    );
    crate::require_localized!(
        &mut r,
        s.filing_status() != FederalStatus::Separate
            || s.community_property != crate::CommunityProperty::NotAnswered,
        &crate::IssueContext::Section(crate::TaxSection::People),
        "Community-property allocation",
        "ISSUE_ANSWER_WHETHER_COMMUNITY_PROPERTY_APPLIES",
        "Answer whether community property applies to your separate return: a community-property state divides your income between your separate returns.",
    );
    // Section 911(b)(2)(C): the exclusion is figured as if the income were not
    // community income, which the Form 8958 division does not do.
    crate::require_localized!(
        &mut r,
        !crate::community::applies(s) || screen.foreign_earned.is_empty(),
        &crate::IssueContext::Section(crate::TaxSection::WorkAbroad),
        "Community-property allocation",
        "ISSUE_FORM_2555_COMMUNITY_SPLIT",
        "A separate return that splits community wages cannot also claim the foreign earned income exclusion: Form 2555 figures the exclusion as if the income were not community income.",
    );
    let repayment = crate::claim_repayment::calculate(d, &mut r);
    if !repayment.inferred_capital.is_empty() || repayment.inferred_minimum.is_some() {
        let mut working = boxed(|| d.clone());
        working.screening.as_mut().unwrap().capital_history.extend(repayment.inferred_capital);
        if let Some(history) = repayment.inferred_minimum {
            working.credits.as_mut().unwrap().minimum_tax =
                Some(crate::MinimumTaxHistories::from_current_unit(history));
        }
        partner_pass.repayment_next = Some(working);
        return r;
    }
    let pension = crate::pension_contributions::calculate(
        &a.pension_contributions,
        s.filing_status(),
        &mut r,
    );
    let j = joint(s.filing_status());
    let separate = s.filing_status() == FederalStatus::Separate;
    let dependent_filer = core::iter::once(&p.taxpayer)
        .chain(p.spouse.iter())
        .any(|person| person.dependency.claimable());
    if dependent_filer
        && d.dependents().iter().any(|person| person.use_for == crate::PersonUse::TaxDependent)
    {
        note_once(&mut r, msg!("NOTE_DEPENDENT_FILER_HAS_NO_DEPENDENTS"));
    }
    let section179_needed = match crate::section179_return::needed(context.forms(d)) {
        Ok(needed) => needed,
        Err(issue) => {
            issue.report(&mut r);
            r.state = ResultState::Unsupported;
            return r;
        }
    };
    if section179_needed {
        // Dispatcher 2026-10-16: this existing owner ordering is phase C work.
        // Cost depletion retains its normal path; no legacy percentage net is
        // accepted as a selected-allocation source.
        if let Some(id) = partner_pass.percentage_depletion_source {
            crate::section179_return::Issue::Activity(
                id,
                "ISSUE_SECTION179_PERCENTAGE_DEPLETION",
                "Percentage depletion with section 179 allocation is not supported yet.",
            )
            .report(&mut r);
            r.state = ResultState::Unsupported;
            return r;
        }
        if partner_pass.section179.is_none() {
            let selected = try_boxed(|| {
                let regular = crate::section179_return::prepare_with_home_and_care_with_claims(
                    d,
                    crate::Section179IncomeSystem::Regular,
                    &partner_pass.office,
                    source_claims,
                    None,
                    None,
                )?;
                let alternative = crate::section179_return::prepare_with_home_and_care_with_claims(
                    d,
                    crate::Section179IncomeSystem::Alternative,
                    &partner_pass.office,
                    source_claims,
                    regular.hypothetical_owner_half,
                    regular.care_compensation,
                )?;
                Ok::<_, crate::section179_return::Issue>([regular, alternative])
            });
            match selected {
                Ok(selected) => partner_pass.section179 = Some(selected),
                Err(issue) => {
                    issue.report(&mut r);
                    r.state = ResultState::Unsupported;
                    return r;
                }
            }
        }
        for (selected, allowed) in
            partner_pass.section179.as_mut().unwrap().iter_mut().zip(&partner_pass.inputs.direct)
        {
            for (&id, source) in &mut selected.allocation.passive_ordinary {
                source.allowed_cents = allowed.get(&id).copied();
            }
        }
        if let Err(issue) = partner_pass.section179.as_ref().unwrap()[0].report(&mut r) {
            issue.report(&mut r);
            r.state = ResultState::Unsupported;
            return r;
        }
    } else {
        if let Err(issue) = crate::section179::report_with_home(d, &mut r, &office_context) {
            crate::section179_return::Issue::from(issue).report(&mut r);
            r.state = ResultState::Unsupported;
            return r;
        }
    }
    let family = family(d, &mut r, p);
    household(s, p, &family, &mut r);
    refuse_situations(d, &mut r);
    whole_return(d, &mut r);
    // Schedule B Part III screening: foreign accounts and trusts
    if screen.foreign_account || screen.foreign_trust {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)]
            "FinCEN 114 / Form 3520 may be required",
            crate::Message::new(
                "NOTE_FINCEN_114_FORM_3520_MAY_BE_REQUIRED_4D35C150",
                alloc::vec![],
            ),
        ));
    }
    crate::foreign_assets::check(d, joint(s.filing_status()), &mut r);
    let mut funding = crate::hsa_funding::calculate(d, &mut r);
    crate::ira::exclude_returned_prior_excess(d, &mut funding);
    let qcd = crate::retirement_return::qualified_charity(d, &mut r, qcd_contributions, &funding);
    let mut basis =
        match crate::retirement_return::basis(d, &mut r, contribution_result, &qcd, &funding) {
            Ok(basis) => basis,
            Err(issue) => {
                issue.report(&mut r);
                return r;
            }
        };
    basis.public_safety = match crate::public_safety::calculate(d, &mut r) {
        Ok(value) => value,
        Err(issue) => {
            issue.report(&mut r);
            return r;
        }
    };
    basis.inherited = crate::inherited_ira::calculate(d, &mut r, &qcd, &funding);
    basis.roth = crate::roth::calculate(d, &mut r, &basis, &funding);
    basis.hsa_funding = funding.excluded;
    basis.inherited_hsa_funding = funding.inherited_excluded;
    basis.inherited_basis_used = funding.inherited_basis_used;
    basis.gross = crate::retirement_return::gross_exceptions(d, &basis, &qcd, &mut r);
    basis.repayment_excluded = match crate::ira_repayment::calculate(d, &mut r, &basis) {
        Ok(excluded) => excluded,
        Err(issue) => {
            issue.report(&mut r);
            return r;
        }
    };
    basis.disaster = crate::disaster::designated(d, &basis, &qcd);
    crate::retirement_return::project_basis_relief(d, &mut r, &basis);
    // Selected reads are fallible: preserve the owner's issue and end this
    // projection before a failed source can contribute a partial amount.
    macro_rules! selected179 {
        ($value:expr) => {
            match $value {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(&mut r);
                    r.state = ResultState::Unsupported;
                    return r;
                }
            }
        };
    }
    let selected_regular = partner_pass.section179.as_ref().map(|pair| &pair[0].allocation);
    let selected_alternative = partner_pass.section179.as_ref().map(|pair| &pair[1].allocation);
    let pass = Pass {
        d,
        care_original,
        context,
        allocated_entries: &allocated_entries,
        office_context: &office_context,
        s,
        p,
        a,
        ded,
        c,
        screen,
        pay,
        social_election: &social_election,
        student_catalog: &student_catalog,
        pension: &pension,
        repayment: &repayment,
        j,
        separate,
        dependent_filer,
        family: &family,
        qcd: &qcd,
        basis: &basis,
        selected_regular,
        selected_alternative,
        contribution_result,
        qcd_contributions,
        adoption_result,
    };
    let mut t = match collect_with_claims(
        context.forms(d),
        &qcd,
        &mut r,
        &basis,
        partner_pass.section179.as_deref(),
        context.medical,
        Some(&office_context),
    ) {
        Ok(totals) => totals,
        Err(issue) => {
            issue.report(&mut r);
            return r;
        }
    };
    if r.issues.iter().any(|issue| issue.display.key == crate::k1_activity::ordinary::RANGE_KEY) {
        r.lines.clear();
        r.return_lines.clear();
        return r;
    }
    macro_rules! required_native {
        ($value:expr) => {
            match $value {
                Some(value) => value,
                None => {
                    CollectFailure::Broker(crate::IssueContext::Section(
                        crate::TaxSection::InvestmentSales,
                    ))
                    .report(&mut r);
                    withhold_required_source_result(&mut r);
                    return r;
                }
            }
        };
    }
    let WithholdingOut { withholding_25a, withholding_25b, withholding_other_dollars } =
        withholding_phase(&mut r, &mut t);
    crate::inherited_ira::project_relief(d, &mut r, &basis);
    let RoyaltySourcesOut {
        royalty_regular,
        royalty_amt,
        royalty_total,
        business_depletion_adjustment,
    } = match royalty_sources_phase(&pass, &mut r, &mut t, partner_pass) {
        core::ops::ControlFlow::Continue(output) => output,
        core::ops::ControlFlow::Break(()) => return r,
    };

    let TaxCreditBondSourceOut { late_investment_sources, investment_sources_known } =
        match tax_credit_bond_source_phase(&pass, &mut r, &mut t) {
            core::ops::ControlFlow::Continue(output) => output,
            core::ops::ControlFlow::Break(()) => return r,
        };
    let EmployeeExpensesOut { employee, employee_recapture, employee_ordinary } =
        employee_expenses_phase(
            &pass,
            &mut r,
            &mut t,
            EmployeeExpensesIn { selected: partner_pass.section179.as_deref() },
        );
    let personal_casualties = selected179!(crate::home_sites::personal_casualties(d));
    let casualty_capital = match crate::PersonalCasualty::capital_gains(&personal_casualties) {
        Ok(gains) => gains,
        Err((key, _message)) => {
            crate::issue_localized!(
                &mut r,
                &crate::IssueContext::Section(crate::TaxSection::Deductions),
                "Personal casualty gain",
                key,
                _message,
            );
            [0, 0]
        }
    };
    // Personal-use conversion gain belongs on Schedule D and Form 8960, but
    // the property was not held for investment under Form 4952.
    let casualty_cents = [
        required_native!(casualty_capital[0].checked_mul(100)),
        required_native!(casualty_capital[1].checked_mul(100)),
    ];
    required_native!(t.try_add_capital(casualty_cents[0], false));
    required_native!(t.try_add_capital(casualty_cents[1], true));
    required_native!(t.capital_endpoints_fit());
    t.injured_capital_sources.add(None, casualty_cents);
    // Form 4684 line 15 gains are Schedule D lines 4 and 11.
    required_native!(checked_native_add(&mut t.schedule_d_line_4, i128::from(casualty_cents[0])));
    required_native!(checked_native_add(&mut t.schedule_d_line_11, i128::from(casualty_cents[1])));
    retirement_adjustments_phase(&pass, &mut r, &mut t);
    let FinalBondInterestOut { late_investment_sources, amt_interest_cents } =
        match final_bond_interest_phase(
            &pass,
            &mut r,
            &mut t,
            FinalBondInterestIn { late_investment_sources },
        ) {
            core::ops::ControlFlow::Continue(output) => output,
            core::ops::ControlFlow::Break(()) => return r,
        };

    t.retirement -= qcd.excluded;
    t.retirement_taxable[0] -= qcd.excluded;
    for owner in 0..2 {
        t.injured_retirement[0][owner] -= qcd.excluded_by_owner[owner];
    }
    let CharitySourcesOut { hosted_student_expenses, whaling_expenses, partner_cash, cash_charity } =
        charity_sources_phase(&pass, &mut r);
    let ForeignValidationOut { mut foreign, foreign_business, foreign_wages } =
        foreign_validation_phase(&pass, &mut r);
    crate::statutory_wages::validate_with_claims(d, &mut r, source_claims);
    for entry in &d.income {
        if let FederalIncome::Wage { data } = &entry.income
            && crate::FederalWage::from(data).paying_scorporation.is_some_and(|id| {
                !d.income.iter().any(|source| {
                    source.id == id
                        && matches!(
                            source.income,
                            FederalIncome::Scorp { .. } | FederalIncome::ScorpActivities { .. }
                        )
                })
            })
        {
            crate::issue_localized!(
                &mut r,
                &crate::interview::income_issue_context(entry),
                "W-2 S-corporation employer",
                "W2_SCORP_EMPLOYER_UNAVAILABLE",
                "Select the actual S-corporation K-1 whose corporation paid this W-2, or choose Not paid by one of my S corporations.",
            );
        }
    }

    for i in 0..2 {
        t.business[i] += t.statutory_profit[i];
        t.business_se_exclusion[i] += t.statutory_profit[i];
    }
    let EmployeePayrollOut {
        tip_income,
        w2_wages,
        uncollected_payroll,
        golden_parachute_tax,
        tip_payroll,
        tip_payroll_owners,
        employee_wage_income,
        mut employee_wage_tax,
        mut employee_wage_taxes,
        form_8919_tax,
    } = employee_payroll_phase(&mut r, &mut t);
    let employee_payroll_complete = true;
    let employee_medicare_complete = true;
    let payroll_agi_known = true;
    let payroll_se_net_known = true;
    let mut other_employee_payroll_tax = 0;
    let mut employee_coverage_taxes = [[0; 2]; 2];
    for owner in 0..2 {
        let taxes = crate::employee_tips::payroll(
            t.social_wages[owner],
            &crate::employee_tips::TipAmounts {
                income: 0,
                social: t.employee_other_social[owner],
                medicare: t.employee_other_medicare[owner],
            },
        );
        employee_coverage_taxes[owner] = taxes;
        let tax = taxes.iter().sum::<i64>();
        other_employee_payroll_tax += tax;
        employee_wage_taxes[owner] += tax;
        // Known SS coverage also consumes this person's Schedule SE wage base.
        t.social_wages[owner] += t.employee_other_social[owner];
        for (suffix, value) in [("social_tax", taxes[0]), ("medicare_tax", taxes[1]), ("tax", tax)]
        {
            line(
                &mut r,
                &alloc::format!("employee_coverage_{owner}_{suffix}"),
                value,
                &["income.employee_wages", "income.social_wages"],
                "https://www.irs.gov/irb/2012-52_IRB",
                msg!("EXPLAIN_EMPLOYEE_COVERAGE_PAYROLL"),
            );
        }
    }
    employee_wage_tax += other_employee_payroll_tax;
    // Compare complete same-owner ordinary payroll endpoints, not the missing
    // operand alone: adding it can cross the native tax's rounding boundary.
    let employee_coverage_possible = [0, 1].map(|owner| {
        crate::employee_tips::payroll(
            t.social_wages[owner] - t.employee_other_social[owner],
            &crate::employee_tips::TipAmounts {
                income: 0,
                social: t.employee_other_social[owner]
                    .checked_add(t.employee_unknown_social[owner])
                    .expect("employee aggregate boundary"),
                medicare: t.employee_other_medicare[owner]
                    .checked_add(t.employee_unknown_medicare[owner])
                    .expect("employee aggregate boundary"),
            },
        )
    });

    let w2_rules::WageAdjustmentsOut {
        moving,
        iso_unreported,
        iso_amt_wage_adjustment,
        moving_excess,
        moving_deduction,
        disability_pension,
        deferral_excess,
    } = w2_rules::wage_adjustments_phase(&pass, &mut r, &mut t);
    let initial_adoption = crate::adoption::benefits(&c.adoptions, &d.income, s, None, &mut r);
    let adoption_benefits = adoption_result.unwrap_or(&initial_adoption);
    let adoption_wages =
        adoption_benefits.wages.iter().sum::<i64>() - adoption_benefits.deemed_excess;
    let adoption_excluded = adoption_benefits.excluded;
    let mut injured_adoption = alloc::boxed::Box::new(
        crate::injured_spouse::income_categories::AdoptionIncomeDollars::from(adoption_benefits),
    );
    let w2_rules::OtherWageIncomeOut {
        household_wages,
        strike_benefits,
        employee_wages_not_covered,
        medicaid_waiver_wages,
        medicaid_waiver_excluded,
        clergy_housing_excess,
        clergy_excess,
        wages,
    } = match w2_rules::other_wage_income_phase(
        &pass,
        &mut r,
        &t,
        w2_rules::OtherWageIncomeIn {
            w2_wages,
            tip_income,
            employee_wage_income,
            disability_pension,
            moving_excess,
            deferral_excess,
        },
    ) {
        core::ops::ControlFlow::Continue(value) => value,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let AlimonyOut {
        alimony_recaptured,
        alimony_recaptured_owners,
        alimony,
        alimony_paid_cents,
        alimony_paid,
    } = alimony_phase(&pass, &mut r, &mut t);
    let other_income_rules::PersonalSalesReportingOut { hobby } =
        other_income_rules::personal_sales_reporting_phase(&mut r, &t);
    let form_4797_rules::OrdinaryDispositionIncomeOut {
        pure_loan_gain,
        pure_amt_loan_gain,
        k1_ordinary,
        loan_gain,
        amt_loan_gain,
        k1_investment_ordinary,
        stock_loss,
    } = form_4797_rules::ordinary_disposition_income_phase(&mut r, &t);
    let form_4952_rules::InvestmentSourcesOut {
        royalty_investment_expenses,
        royalty_investment_income,
        royalty_routing,
        investment_unavailable,
        investment_paid,
        k1_interest_record,
    } = match form_4952_rules::investment_sources_phase(&pass, &mut r, &t) {
        core::ops::ControlFlow::Continue(value) => value,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let investment_facts = if investment_unavailable { None } else { k1_interest_record.as_ref() };
    let schedule_e_rules::RoyaltyIncomeReportingOut {
        royalty_interest,
        royalties,
        personal_rent,
        personal_rent_expenses,
        property_net,
    } = match schedule_e_rules::royalty_income_reporting_phase(
        &pass,
        &mut r,
        &mut t,
        schedule_e_rules::RoyaltyIncomeReportingIn { royalty_routing: &royalty_routing },
    ) {
        core::ops::ControlFlow::Continue(value) => value,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let tax_refund = dollars(t.tax_refund);
    line(
        &mut r,
        "taxable_state_tax_refund",
        tax_refund,
        &["income.tax_refund"],
        "https://www.irs.gov/instructions/i1040gi",
        msg!("EXPLAIN_TAXABLE_STATE_TAX_REFUND"),
    );
    let education_allowed = education_credit_allowed(&pass);
    let qtp_excess_by_entry = crate::ira::qtp_excess_by_entry(d);
    let EducationDistributionsOut { education_context, education, education_income } =
        education_distributions_phase(
            &pass,
            &mut r,
            &mut t,
            EducationDistributionsIn {
                education_allowed,
                qtp_excess_by_entry: &qtp_excess_by_entry,
            },
        );
    let AccountDistributionsOut {
        able_excess_tax,
        trump_excess_tax,
        trump_earnings_tax,
        able_income,
        account_distribution_taxes,
        account_tax,
    } = account_distributions_phase(
        &pass,
        &mut r,
        &mut t,
        AccountDistributionsIn { education: &education },
    );
    let MedicalDistributionsOut {
        archer_msa,
        archer_msa_tax,
        medicare_msa,
        medicare_msa_tax,
        medical_earnings,
    } = match medical_distributions_phase(&mut r, &mut t) {
        core::ops::ControlFlow::Continue(output) => output,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let OtherIncomeOut {
        care_benefits,
        personal_debt,
        prizes,
        alaska,
        olympic_prize,
        digital,
        deferred_pension,
        inmate_pay,
        other_8z,
        not_for_profit,
        other_line_8,
        casualty_living_insurance,
        taxed_relief,
        health_assistance,
        scholarship,
        scholarship_inclusions,
        jury,
        jury_remitted,
        taxable_grants,
        family_leave,
    } = other_income_phase(
        &pass,
        &mut r,
        &mut t,
        OtherIncomeIn { medicaid_waiver_excluded, medical_earnings },
    );
    let gambling = dollars(t.gambling);
    let gambling_loss = phase::cap(gambling, [fraction(dollars(t.gambling_losses), 90, 100)]);
    let interest = dollars(t.interest);
    let savings_penalty = dollars(t.early_penalty);
    let estate_67e = dollars(t.estate_67e);
    line(
        &mut r,
        "estate_trust_67e_deduction",
        estate_67e,
        &["income.estate_trust_portfolio.final_67e_deduction"],
        "https://www.irs.gov/pub/irs-dft/f1040s1--dft.pdf",
        msg!("EXPLAIN_ESTATE_TRUST_67E_DEDUCTION"),
    );
    let exempt = dollars(t.exempt);
    let dividends = dollars(t.dividends);
    let qualified = dollars(t.qualified);
    form_8814_projection_phase(&mut r, &t);
    // Schedule B is filed only above $1,500 of interest or of dividends (or
    // with Part III facts, or to claim the savings bond exclusion); below that
    // the return shows no Schedule B. The interest lines follow Form 8815.
    const SCHEDULE_B_THRESHOLD: i64 = 1_500;
    if dividends > SCHEDULE_B_THRESHOLD {
        form_line(
            &mut r,
            FormLine::new(TaxForm::ScheduleB, "6"),
            "schedule_b_ordinary_dividends",
            dividends,
            &["income.dividends"],
            "https://www.irs.gov/pub/irs-pdf/f1040sb.pdf",
            msg!("EXPLAIN_SCHEDULE_B_ORDINARY_DIVIDENDS"),
        );
    }
    // Replace each native annual category before rounding, not a rounded
    // source difference. Estate qualification keeps its existing projection.
    let k1_amt_interest_adjustment = dollars(amt_interest_cents) - interest;
    let k1_amt_dividend_adjustment = dollars(t.dividends + t.k1_amt_dividends) - dividends;
    let existing_amt_qualified = dollars(t.qualified + t.estate_amt_box12b);
    let amt_qualified = dollars(t.qualified + t.estate_amt_box12b + t.k1_amt_qualified);
    let k1_amt_qualified_adjustment = amt_qualified - existing_amt_qualified;
    let CapitalPriorOut {
        capital_in,
        amt_capital_in,
        estate_carry_cents,
        capital_work_prior,
        filed_prior,
        estate_investment_carry,
        estate_business_carry,
        investment_capital_in,
        regular_capital_amounts,
        portfolio_capital,
        amt_capital_amounts,
    } = capital_prior_phase(
        &pass,
        &mut r,
        &t,
        CapitalPriorIn { modify_income: nol_run.modify_income },
    );
    let ScheduleSeOut {
        se,
        se_profit,
        half,
        net,
        se_sources_complete,
        mut earned,
        mut compensation,
        compensation_earned_profit,
        optional_eic,
        optional_child,
        optional_weights,
        optional_weights_cents,
        church_half,
        se_church_wages,
        business_se_tax,
        business_half_se,
        business_half_se_cents,
        business_exclusions,
        business_excluded,
        income_excluded,
        housing_deduction,
        excluded,
    } = match schedule_se_phase(
        &pass,
        &mut r,
        &t,
        ScheduleSeIn { partners: &partner_pass.inputs.partners, foreign: &mut foreign },
    ) {
        core::ops::ControlFlow::Continue(output) => output,
        core::ops::ControlFlow::Break(()) => return r,
    };
    for i in 0..2 {
        earned[i] += adoption_benefits.wages[i];
        compensation[i] += adoption_benefits.wages[i];
    }
    let CareBenefitsOut { form_2441_context, form_2441, benefit_wages, earned, compensation } =
        match care_benefits_phase(
            &pass,
            &mut r,
            &t,
            &mut partner_pass.care_next,
            CareBenefitsIn { care_current: &partner_pass.care, earned, compensation, optional_eic },
        ) {
            core::ops::ControlFlow::Continue(out) => out,
            core::ops::ControlFlow::Break(()) => return r,
        };
    let se_tax = se.iter().sum::<i64>();
    let half_se = half.iter().sum::<i64>();
    let SelfEmployedRetirementOut {
        partner_half_attribution,
        compensation_nonbusiness,
        sep,
        sep_deduction,
        compensation,
    } = match self_employed_retirement_phase(
        &pass,
        &mut r,
        partner_pass,
        SelfEmployedRetirementIn {
            half,
            compensation,
            compensation_earned_profit,
            business_half_se: &business_half_se,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let HealthInsuranceOut { health, health_earnings } = match health_insurance_phase(
        &pass,
        &mut r,
        HealthInsuranceIn {
            half,
            partner_half_attribution,
            business_half_se_cents: &business_half_se_cents,
            optional_weights: &optional_weights,
            optional_weights_cents,
            sep: &sep,
            partners: &partner_pass.inputs.partners,
            iterative: partner_pass.iterative,
            specified: partner_pass.specified.as_ref(),
        },
    ) {
        core::ops::ControlFlow::Continue(output) => output,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let business = dollars(t.business.iter().sum::<i64>() + t.community_business);
    // IRC 62(d)(2): educator expenses are deductible only above the earnings a
    // 529 plan or Coverdell ESA excludes and the savings bond interest Form 8815 excludes
    // (the savings bond record's owner).
    let bond_owner = crate::savings_bonds::record(&d.income)
        .map(|(entry, _)| usize::from(entry.owner == crate::DocumentOwner::Spouses));
    let bond_reduction = partner_pass.bond_exclusion;
    let educator_net_cents = |amount: i64, owner: usize| {
        positive(
            amount
                - education.excluded_earnings[owner]
                - if bond_owner == Some(owner) { bond_reduction * 100 } else { 0 },
        )
    };
    let educator_net = |amount: i64, owner: usize| dollars(educator_net_cents(amount, owner));
    crate::require_localized!(
        &mut r,
        a.educators.spouse.is_none() || j,
        &crate::IssueContext::Section(crate::TaxSection::Educator),
        "Adjustments",
        "ISSUE_SPOUSE_EDUCATOR_EXPENSES_REQUIRE_A_JOINT_RETURN_71AF9B36",
        "Spouse educator expenses require a joint return.",
    );

    let educator = phase::cap(
        educator_net(a.educator_taxpayer(), 0),
        [crate::year_constants::EDUCATOR_EXPENSE_LIMIT_DOLLARS],
    ) + if j {
        phase::cap(
            educator_net(a.educator_spouse(), 1),
            [crate::year_constants::EDUCATOR_EXPENSE_LIMIT_DOLLARS],
        )
    } else {
        0
    };
    let educator_work_cents = phase::cap(
        educator_net_cents(a.educator_taxpayer(), 0),
        [crate::year_constants::EDUCATOR_EXPENSE_LIMIT_DOLLARS * 100],
    ) + if j {
        phase::cap(
            educator_net_cents(a.educator_spouse(), 1),
            [crate::year_constants::EDUCATOR_EXPENSE_LIMIT_DOLLARS * 100],
        )
    } else {
        0
    };
    let ArcherMsaOut {
        msa_deduction_cents,
        archer_employer_income_cents,
        archer_correction_earnings_cents,
        archer_correction_employer_cents,
        archer_prior_employer_cents,
        archer_excess_tax,
        msa_deduction,
        archer_total_income,
    } = match archer_msa_phase(
        &pass,
        &mut r,
        &mut t,
        ArcherMsaIn { half, optional_weights: &optional_weights, archer_msa },
    ) {
        core::ops::ControlFlow::Continue(output) => output,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let HsaOut {
        hsa,
        hsa_income,
        hsa_tax,
        hsa_distribution_tax,
        hsa_chapter_one_tax,
        hsa_testing_tax,
        hsa_excess_tax,
        hsa_work_income,
        hsa_deduction_by_owner,
    } = match hsa_phase(&pass, &mut r, &mut t) {
        core::ops::ControlFlow::Continue(output) => output,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let RetirementIncomeReportOut { retirement_taxable, retirement, trump_death, conversion } =
        retirement_income_report_phase(&pass, &mut r, &t);
    adjustment_reports_phase(
        &mut r,
        AdjustmentReportsIn {
            savings_penalty,
            alimony,
            alimony_paid,
            moving_deduction,
            moving_excess,
        },
    );
    // Keep this subtotal grouped: Clippy 1.95 otherwise recursively revisits
    // its initializer when checking later method receivers.
    let before_rental = {
        wages + iso_unreported + adoption_wages + benefit_wages + foreign_wages - excluded
            + alimony
            + gambling
            + jury
            + scholarship
            + prizes
            + other_line_8
            + taxable_grants
            + family_leave
            + casualty_living_insurance
            + personal_debt
            + care_benefits
            + medicare_msa
            + archer_total_income
            + able_income
            + education_income
            + tax_refund
            + property_net
            + loan_gain
            - stock_loss
            + hobby
            + interest
            + dividends
            + portfolio_capital
            + t.child.other
            + retirement
            + trump_death
            + dollars(t.unemployment)
            + hsa_income
            + business
            - half_se
            - health.deduction
            - sep_deduction
            - educator
            - employee_ordinary
            - dollars(hsa)
            - msa_deduction
            - savings_penalty
            - alimony_paid
            - moving_deduction
            - jury_remitted
            - estate_67e
            - nol_run.deduction
    };
    let RentalMagiPreflightOut { rental_magi_base, home_route, home_use } =
        rental_magi_preflight_phase(
            &pass,
            &mut r,
            partner_pass,
            RentalMagiPreflightIn {
                regular_capital_amounts,
                capital_in,
                modify_income: nol_run.modify_income,
                before_rental,
                half_se,
                adoption_excluded,
            },
        );
    let mut rental = if let Some(snapshot) = &partner_pass.specials.rental {
        snapshot.calculation.append(&mut r);
        snapshot.calculation.outcome.clone()
    } else {
        let mut output = FederalResult::empty(d.revision);
        let outcome = crate::extended_rules::rentals_for_home_with_selected_section179_with_claims(
            d,
            rental_magi_base,
            &mut output,
            false,
            &home_use,
            &partner_pass.care,
            partner_pass.section179.as_deref(),
            Some(&office_context),
            source_claims,
        );
        let snapshot = HealthRentalSnapshot {
            calculation: OwnerSnapshot { outcome, output },
            magi_input: rental_magi_base,
        };
        snapshot.calculation.append(&mut r);
        let rental = snapshot.calculation.outcome.clone();
        partner_pass.specials.rental_trial = Some(snapshot);
        rental
    };
    if rental.source_issue.is_some() || rental.pool_issue.is_some() {
        withhold_required_source_result(&mut r);
        return r;
    }
    crate::asset_sale::report_farm_rental_property_limits(&rental, &mut r);
    crate::asset_sale::report_direct_business_property_limits(d, &rental, &mut r);
    // Form 172 Part II line 6 restores excluded section 1202 gain before the
    // rental real-estate special allowance. Keep the ordinary result only when
    // the complete passive pool, including suspensions and downstream source
    // attribution, is identical in that hypothetical calculation.
    let passive_unchanged_under_qsbs = if nol_run.modify_income
        && t.qsbs_excluded > 0
        && (screen.nol.present() || estate_final_nol(d).is_some())
    {
        let mut comparison = r.clone();
        let modified_magi = rental_magi_base + dollars(t.qsbs_excluded);
        let mut modified =
            crate::extended_rules::rentals_for_home_with_selected_section179_with_claims(
                d,
                modified_magi,
                &mut comparison,
                false,
                &crate::extended_rules::HomeUseContext::provisional(
                    d,
                    [modified_magi; 2],
                    home_route,
                ),
                &crate::care::CareDeductions::default(),
                partner_pass.section179.as_deref(),
                Some(&office_context),
                source_claims,
            );
        // The modified AGI only sizes a credit allowance (Form 8582-CR line 22).
        modified.modified_agi = rental.modified_agi;
        modified.home_use = rental.home_use.clone();
        modified == rental && comparison.issues == r.issues
    } else {
        true
    };
    // The same return coordinator already owns earnings-dependent deductions.
    // Retain the native AMT pool snapshot only when direct C also supplies its
    // earnings inputs, then append that exact snapshot at the normal AMT stage.
    let amt_direct_trial = if partner_pass
        .section179
        .as_ref()
        .is_some_and(|pair| !pair[0].allocation.passive_ordinary.is_empty())
    {
        let mut output = FederalResult::empty(d.revision);
        let outcome = crate::extended_rules::rentals_for_home_with_selected_section179_with_claims(
            d,
            rental_magi_base,
            &mut output,
            true,
            &home_use,
            &partner_pass.care,
            partner_pass.section179.as_deref(),
            Some(&office_context),
            source_claims,
        );
        if outcome.source_issue.is_some() || outcome.pool_issue.is_some() {
            OwnerSnapshot { outcome, output }.append(&mut r);
            withhold_required_source_result(&mut r);
            return r;
        }
        Some(OwnerSnapshot { outcome, output })
    } else {
        None
    };
    let direct = if let Some(trial) = &amt_direct_trial {
        match (
            rental.direct_business_allowances(false),
            trial.outcome.direct_business_allowances(true),
        ) {
            (Some(regular), Some(alternative))
                if partner_pass.section179.as_ref().is_some_and(|pair| {
                    regular.keys().eq(pair[0].allocation.passive_ordinary.keys())
                        && alternative.keys().eq(pair[1].allocation.passive_ordinary.keys())
                }) =>
            {
                [regular, alternative]
            }
            _ => {
                crate::section179_return::Issue::Activity(
                    partner_pass
                        .section179
                        .as_ref()
                        .and_then(|pair| pair[0].allocation.passive_ordinary.keys().next().copied())
                        .unwrap_or(0),
                    "BUSINESS_PASSIVE_OPERATING_SOURCE_REQUIRED",
                    "The final regular and AMT passive operating sources are unavailable.",
                )
                .report(&mut r);
                withhold_required_source_result(&mut r);
                return r;
            }
        }
    } else {
        Default::default()
    };
    let earnings = EarningsInputs { partners: rental.partner_se.clone(), direct };
    if rental.inputs_complete && earnings != partner_pass.inputs {
        partner_pass.next = Some(earnings);
        return r; // Discard this trial: all downstream consumers must use the same inputs.
    }
    if let core::ops::ControlFlow::Break(()) = schedule_se_rules::partnership_se_reporting_phase(
        &pass,
        &mut r,
        &t,
        partner_pass,
        schedule_se_rules::PartnershipSeReportingIn {
            rental: &rental,
            se_tax,
            half_se,
            health_deduction: health.deduction,
            sep_deduction,
            rental_magi_base,
        },
    ) {
        return r;
    }
    crate::community::SourceProjection::bind_business(&mut rental, &t.business_sources, false);
    let k1_sale_ordinary = crate::k1_sales::report(d, &rental, false, &mut r);
    let property_character =
        crate::section1231::report_with_claims(d, &rental, false, &mut r, source_claims);
    // report(None) alone is ambiguous: worksheet errors also return None.
    // A complete, issue-free same-call report proves its Ok(None) absence
    // branch, whose actual property inventory contains no current1231 source.
    let property_sources_known_absent =
        property_character.is_none() && rental.inputs_complete && r.issues.is_empty();
    crate::installment_return::report_with_claims(
        d,
        &rental,
        property_character.as_ref(),
        &mut r,
        source_claims,
    );
    crate::exchange::report(d, &rental, property_character.as_ref(), &mut r);
    // Resolve both activity pools before capital netting. AMT uses regular
    // rental MAGI but its own activity amounts, histories and depreciation.
    let regular_issue_count = r.issues.len();
    let mut amt_rental = if let Some(snapshot) = amt_direct_trial {
        snapshot.append(&mut r);
        snapshot.outcome
    } else {
        crate::extended_rules::rentals_for_home_with_selected_section179_with_claims(
            d,
            rental_magi_base,
            &mut r,
            true,
            &home_use,
            &partner_pass.care,
            partner_pass.section179.as_deref(),
            Some(&office_context),
            source_claims,
        )
    };
    if amt_rental.source_issue.is_some() || amt_rental.pool_issue.is_some() {
        withhold_required_source_result(&mut r);
        return r;
    }
    let form_4797_rules::OrdinaryPropertySourcesOut {
        k1_ordinary_passive_adjustment,
        amt_k1_ordinary_passive_adjustment,
        loan_gain,
        amt_loan_gain,
        amt_property_character,
        amt_activity_issues,
        direct_property_sources,
        amt_direct_property_sources,
        direct_property_ordinary,
        amt_direct_property_ordinary,
        rental_property_sources,
        injured_property_sources,
        property_ordinary,
        amt_property_ordinary,
    } = match form_4797_rules::ordinary_property_sources_phase(
        &pass,
        &mut r,
        &t,
        form_4797_rules::OrdinaryPropertySourcesIn {
            rental: &rental,
            amt_rental: &mut amt_rental,
            property_character: &property_character,
            regular_issue_count,
            k1_ordinary,
            k1_sale_ordinary,
            loan_gain,
            amt_loan_gain,
        },
    ) {
        core::ops::ControlFlow::Continue(value) => value,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let schedule_d_rules::CapitalNettingOut {
        regular_capital,
        amt_capital_result,
        short,
        long,
        capital,
        amt_short,
        amt_long,
        amt_capital,
        amt_capital_independent_adjustment,
        amt_net_capital_gain,
        amt_schedule_d_18,
        amt_schedule_d_19,
        pref,
        net_capital_gain,
        schedule_d_18,
        schedule_d_19,
        unrecaptured_gain,
        collectible_gain,
    } = match schedule_d_rules::capital_netting_phase(
        &pass,
        &mut r,
        &t,
        schedule_d_rules::CapitalNettingIn {
            rental: &rental,
            amt_rental: &amt_rental,
            property_character: &property_character,
            amt_property_character: &amt_property_character,
            regular_capital_amounts,
            amt_capital_amounts,
            capital_in,
            amt_capital_in,
            filed_prior,
            estate_carry_cents,
            modify_income: nol_run.modify_income,
            qualified,
            loan_gain,
            amt_loan_gain,
            property_ordinary,
            amt_property_ordinary,
        },
    ) {
        core::ops::ControlFlow::Continue(value) => value,
        core::ops::ControlFlow::Break(()) => return r,
    };
    schedule_d_source_reporting_phase(
        &mut r,
        &t,
        ScheduleDSourceReportingIn {
            unrecaptured_gain,
            collectible_gain,
            property_character: &property_character,
        },
    );
    let form_461_rules::BusinessLossClassificationOut {
        before_rental,
        k1_business_ordinary,
        amt_k1_business_ordinary,
        property_business_capital,
        property_business_income,
        amt_property_business_income,
        personal_rent_expenses,
        property_net,
        adjusted_business,
        excess_business_loss_operand,
        property_loss_deferred,
        excess_business_loss,
        before_social,
    } = form_461_rules::business_loss_classification_phase(
        &pass,
        &mut r,
        form_461_rules::BusinessLossClassificationIn {
            rental: &rental,
            property_character: &property_character,
            amt_property_character: &amt_property_character,
            direct_property_sources: &direct_property_sources,
            health: &health,
            k1_ordinary,
            before_rental,
            portfolio_capital,
            capital,
            amt_capital,
            property_ordinary,
            amt_property_ordinary,
            k1_ordinary_passive_adjustment,
            amt_k1_ordinary_passive_adjustment,
            short,
            long,
            amt_short,
            amt_long,
            personal_rent_expenses,
            property_net,
            business,
            half_se,
            sep_deduction,
            business_excluded,
            direct_property_ordinary,
        },
    );
    let ira_contributions_rules::IraDeductionOut {
        ira_magi,
        qtp_transfers,
        carry_distributions,
        ira_facts,
        covered_ira,
        pending_health_ira,
        ira,
    } = ira_contributions_rules::ira_deduction_phase(
        &pass,
        &mut r,
        &t,
        partner_pass,
        ira_contributions_rules::IraDeductionIn {
            health: &health,
            compensation,
            before_social,
            exempt,
            excluded,
            adoption_excluded,
        },
    );
    let form_6251_rules::AmtRelatedDeductionsOut {
        amt_sep_deduction,
        amt_health,
        amt_ira_deduction,
        amt_related_adjustment,
    } = match form_6251_rules::amt_related_deductions_phase(
        &pass,
        &mut r,
        &t,
        partner_pass,
        form_6251_rules::AmtRelatedDeductionsIn {
            health_earnings: &health_earnings,
            amt_rental: &amt_rental,
            business_exclusions: &business_exclusions,
            half,
            partner_half_attribution,
            health: &health,
            sep: &sep,
            compensation,
            compensation_nonbusiness,
            compensation_earned_profit,
            ira_facts: &ira_facts,
            qtp_transfers: &qtp_transfers,
            carry_distributions,
            ira_magi,
            ira: &ira,
            sep_deduction,
        },
    ) {
        core::ops::ControlFlow::Continue(value) => value,
        core::ops::ControlFlow::Break(()) => return r,
    };
    partner_pass.specials.ira_trial = Some(ira.clone());
    // Independently recalculate each candidate's deduction using the resulting
    // QCD exclusion and pretax capacity. Accept exact agreement only.
    let qcd_agrees = a.qualified_charity.iter().all(|q| {
        let owner = usize::from(q.owner == Owner::Spouse);
        ira.nondeductible[owner] == qcd_contributions.map_or(0, |v| v.nondeductible[owner])
    });
    if !pending_health_ira && !qcd_agrees && qcd_contributions.is_none() && r.issues.is_empty() {
        let checked = estimate_with_ira_from_source_with_claims(
            d,
            context,
            None,
            Some(&ira),
            adoption_result,
            nol_run,
            partner_pass.percentage_depletion_source,
        );
        if checked.state == ResultState::EstimatedForSupportedScope {
            return checked;
        }
        if let Some(f) = ira_facts.as_deref() {
            for candidate in crate::ira::deduction_candidates(
                f,
                p,
                s,
                compensation,
                &qtp_transfers,
                carry_distributions,
            ) {
                let result = estimate_with_ira_from_source_with_claims(
                    d,
                    context,
                    None,
                    Some(&candidate),
                    adoption_result,
                    nol_run,
                    partner_pass.percentage_depletion_source,
                );
                if result.state == ResultState::EstimatedForSupportedScope {
                    return result;
                }
            }
        }
        return checked;
    }
    crate::require_localized!(
        &mut r,
        pending_health_ira || qcd_agrees,
        &crate::IssueContext::Section(crate::TaxSection::Retirement),
        "QCD",
        "ISSUE_THE_IRA_DEDUCTION_AND_QCD_EXCLUSION_REQUIRE_FURTHER_COORDINATION_TH_115808A3",
        "The IRA deduction and QCD exclusion require further coordination; the verified worksheet candidate does not agree with the recalculated deduction.",
    );
    // A second ordered pass substitutes final Form 8606 amounts while keeping
    // the deduction determined by the preliminary IRS worksheet. Dependent
    // AGI items (Social Security, rental loss allowance, credits) then use final income.
    if contribution_result.is_none()
        && !pending_health_ira
        && r.issues.is_empty()
        && !a.ira_basis.is_empty()
        && a.entered_iras().is_some_and(|f| {
            f.taxpayer.traditional > 0 || f.spouse.as_ref().is_some_and(|v| v.traditional > 0)
        })
    {
        partner_pass.contribution_next = Some(ira);
        return r;
    }
    for (owner, nondeductible) in ira.nondeductible.iter().enumerate() {
        let has_distribution = allocated_entries.iter().any(|e| {
            usize::from(e.owner == crate::DocumentOwner::Spouses) == owner
                && e.income.retirement().is_some_and(|data| {
                    data.kind == crate::RetirementKind::Ira && data.taxable_income() > 0
                })
        });
        crate::require_localized!(
            &mut r,
            *nondeductible == 0
                || !has_distribution
                || a.ira_basis.iter().any(|b| usize::from(b.owner == Owner::Spouse) == owner),
            &crate::IssueContext::Section(crate::TaxSection::IraContributions),
            "IRA contributions",
            "ISSUE_NONDEDUCTIBLE_CONTRIBUTIONS_AND_IRA_DISTRIBUTIONS_FOR_THE_SAME_OWNE_7AC2F41E",
            "Nondeductible contributions and IRA distributions for the same owner require a complete basis worksheet in Adjustments.",
        );
    }
    let SocialOut { social, qsbs_with_prior_nol, modified_ira } = social_phase(
        &pass,
        &mut r,
        &t,
        nol_run,
        SocialIn {
            ira: &ira,
            ira_facts: &ira_facts,
            qtp_transfers: &qtp_transfers,
            compensation,
            carry_distributions,
            before_social,
            exempt,
            excluded,
            adoption_excluded,
            ira_magi,
        },
    );
    let ira_excess_tax = ira_excess_phase(
        &pass,
        &mut r,
        &mut t,
        partner_pass,
        nol_run,
        IraExcessIn {
            ira_facts: &ira_facts,
            qtp_transfers: &qtp_transfers,
            qtp_excess_by_entry: &qtp_excess_by_entry,
            compensation,
            home_route,
            before_social,
            social,
            exempt,
            excluded,
            adoption_excluded,
            conversion,
            before_rental,
            half_se,
            adjusted_business,
            property_business_income,
        },
    );
    // Form 8615 can apply even when the filer is not another person's dependent.
    // Its "more than half" test differs from the AOTC refund's "less than half" test.
    let unearned = interest
        + dividends
        + capital
        + t.child.other
        + retirement
        + trump_death
        + social
        + dollars(t.unemployment)
        + alimony
        + gambling
        + jury
        + scholarship
        + prizes
        + other_line_8
        - inmate_pay
        + taxable_grants
        + family_leave
        + casualty_living_insurance
        + personal_debt
        + care_benefits
        + medicare_msa
        + archer_total_income
        + able_income
        + education_income
        + tax_refund
        + property_net
        + loan_gain
        + hobby
        + hsa_income
        + rental.income
        + property_ordinary
        // Form 8615 starts with total income, including the Form 461 addback.
        // Disallowed business loss cannot reduce the child's unearned income.
        + excess_business_loss
        - savings_penalty;
    let kiddie_age = kiddie_age(&p.taxpayer);
    let form_8615_rules::PartnershipEarnedSourcesOut {
        partner_young_earned,
        partner_young_loss_addback,
        partner_standard_earned,
        partner_young_complete,
        partner_standard_complete,
        young_8615,
        dependent_standard,
    } = form_8615_rules::partnership_earned_sources_phase(
        &pass,
        &mut r,
        form_8615_rules::PartnershipEarnedSourcesIn {
            rental: &rental,
            business_half_se: &business_half_se,
        },
    );
    let before_loan = before_social - ira.deduction - pension.deduction + social;
    let mut adoption_exclusion_unchanged_under_qsbs = true;
    if crate::adoption::has_benefits(&c.adoptions) {
        // Form 8839 line 25: no section 137 or student-loan reduction;
        // retain the prior Social Security, IRA and passive-loss worksheets.
        let magi = before_loan
            + excluded
            + adoption_excluded
            + jury_remitted
            + dollars(t.alimony_recapture_deduction);
        let calculated = crate::adoption::benefits(&c.adoptions, &d.income, s, Some(magi), &mut r);
        if qsbs_with_prior_nol && nol_run.modify_income {
            // Form 172 Part II line 6 restores section 1202 gain before
            // testing Form 8839's exclusion phaseout. If the resulting
            // benefit stays identical, the existing income pass is exact.
            // A changed benefit also changes wages and dependent worksheets
            // and remains an explicit unsupported combination.
            let modified_magi = magi
                + dollars(t.qsbs_excluded)
                + nol_run.section1202_ira_adjustment
                + nol_run.section1202_social_adjustment;
            let mut comparison = r.clone();
            let modified = crate::adoption::benefits(
                &c.adoptions,
                &d.income,
                s,
                Some(modified_magi),
                &mut comparison,
            );
            adoption_exclusion_unchanged_under_qsbs = modified.wages == adoption_benefits.wages
                && modified.deemed_excess == adoption_benefits.deemed_excess
                && modified.excluded == adoption_excluded;
        }
        if adoption_result.is_none() && r.issues.is_empty() {
            partner_pass.adoption_next = Some(calculated);
            return r;
        }
        crate::require_localized!(
            &mut r,
            calculated.wages == adoption_benefits.wages
                && calculated.deemed_excess == adoption_benefits.deemed_excess,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Adoption benefits",
            "ISSUE_THE_EXCLUSION_AND_INCOME_DEPENDENT_DEDUCTIONS_NEED_FURTHER_COORDINA_1C26D484",
            "The exclusion and income-dependent deductions need further coordinated worksheets; the provisional exclusion does not agree with the final income calculation.",
        );
        // Retain the final same-invocation source packet after the existing
        // financial fixedpoint check. The provisional pass has no final MAGI
        // classification, even if its monetary values happened to match.
        *injured_adoption =
            crate::injured_spouse::income_categories::AdoptionIncomeDollars::from(&calculated);
        emit_lines(
            &mut r,
            &mut (),
            [
                ("adoption_exclusion_magi", magi, msg!("EXPLAIN_ADOPTION_EXCLUSION_MAGI")),
                (
                    "adoption_benefits_excluded",
                    adoption_excluded,
                    msg!("EXPLAIN_ADOPTION_BENEFITS_EXCLUDED"),
                ),
                (
                    "adoption_wage_adjustment",
                    adoption_wages,
                    msg!("EXPLAIN_ADOPTION_WAGE_ADJUSTMENT"),
                ),
                (
                    "adoption_prior_taxed_benefits",
                    adoption_benefits.prior_taxable,
                    msg!("EXPLAIN_ADOPTION_PRIOR_TAXED_BENEFITS"),
                ),
            ],
            |(id, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(amount),
                    &["credits.adoptions.benefits", "income", "adjustments"],
                    "https://www.irs.gov/instructions/i8839",
                    explanation,
                ))
            },
        );
    }
    let eligible_loan_interest =
        eligible_loan_phase(&pass, &mut r, &t, EligibleLoanIn { education: &education });
    // Form 8815 (IRC 135). Its modified AGI is AGI before the student loan
    // deduction and after every other adjustment, plus the foreign earned
    // income and adoption exclusions; the exclusion then lowers the AGI that
    // the student loan phaseout reads (IRC 221(b)(2)(C)). The passive loss,
    // Social Security and IRA worksheets above keep the gross interest, which
    // IRC 86(b)(2)(A), 219(g)(3)(A)(ii) and 469(i)(3)(E)(ii) read without
    // regard to section 135, as the i8815 line 9 worksheet does (Schedule B
    // line 2 is the gross interest). Pinned by
    // passive_allowance_reads_the_savings_bond_interest_gross.
    let savings_bond_context = crate::savings_bonds::SavingsBondContext {
        income: &d.income,
        credits: c,
        students: &student_catalog,
        people: p,
        status: s.filing_status(),
        dependent_ids: &family.dependent_ids,
        credit_allowed: !separate && education_allowed,
        education: &education,
        magi: before_loan + excluded + adoption_excluded,
    };
    let savings_bond_exclusion = if let Some(snapshot) = &partner_pass.specials.bonds {
        snapshot.append(&mut r);
        snapshot.outcome
    } else {
        let mut output = FederalResult::empty(d.revision);
        let outcome = crate::savings_bonds::calculate(&savings_bond_context, &mut output);
        let snapshot = OwnerSnapshot { outcome, output };
        snapshot.append(&mut r);
        partner_pass.specials.bonds_trial = Some(snapshot);
        outcome
    };
    if partner_pass.bond_exclusion == 0
        && savings_bond_exclusion > 0
        && a.educator_taxpayer() + a.educator_spouse() > 0
    {
        partner_pass.bond_next = Some(savings_bond_exclusion);
    }
    if savings_bond_exclusion > 0 {
        form_line(
            &mut r,
            FormLine::new(TaxForm::ScheduleB, "2"),
            "schedule_b_total_interest",
            interest,
            &["income.interest"],
            "https://www.irs.gov/pub/irs-dft/f1040sb--dft.pdf",
            msg!("EXPLAIN_SCHEDULE_B_TOTAL_INTEREST"),
        );
        form_line(
            &mut r,
            FormLine::new(TaxForm::ScheduleB, "3"),
            "schedule_b_savings_bond_exclusion",
            savings_bond_exclusion,
            &["savings_bond_exclusion"],
            "https://www.irs.gov/pub/irs-dft/f1040sb--dft.pdf",
            msg!("EXPLAIN_SCHEDULE_B_SAVINGS_BOND_EXCLUSION"),
        );
    }
    let interest = interest - savings_bond_exclusion;
    let before_loan = before_loan - savings_bond_exclusion;
    if interest > SCHEDULE_B_THRESHOLD || savings_bond_exclusion > 0 {
        form_line(
            &mut r,
            FormLine::new(TaxForm::ScheduleB, "4"),
            "schedule_b_taxable_interest",
            interest,
            &["income.interest", "savings_bond_exclusion"],
            "https://www.irs.gov/pub/irs-pdf/f1040sb.pdf",
            msg!("EXPLAIN_SCHEDULE_B_TAXABLE_INTEREST"),
        );
    }
    let loan_start = if j {
        crate::year_constants::STUDENT_LOAN_PHASEOUT_START_JOINT_DOLLARS
    } else {
        crate::year_constants::STUDENT_LOAN_PHASEOUT_START_OTHER_DOLLARS
    };
    let loan_range = if j {
        crate::year_constants::STUDENT_LOAN_PHASEOUT_RANGE_JOINT_DOLLARS
    } else {
        crate::year_constants::STUDENT_LOAN_PHASEOUT_RANGE_OTHER_DOLLARS
    };
    let loan_claim_allowed = if eligible_loan_interest == 0 { true } else { unclaimed_filers(p) };
    let loan = if let Some(deduction) = partner_pass.specials.student {
        deduction
    } else if separate || !loan_claim_allowed {
        0
    } else {
        phase::remaining_fraction(
            phase::cap(
                dollars(eligible_loan_interest),
                [crate::year_constants::STUDENT_LOAN_INTEREST_MAXIMUM_DOLLARS],
            ),
            phase::cap(positive(loan_start + loan_range - before_loan - excluded), [loan_range]),
            loan_range,
        )
    };
    partner_pass.specials.student_trial = loan;
    nol_run.section1202_loan_adjustment =
        if nol_run.modify_income && t.qsbs_excluded > 0 && !separate && loan_claim_allowed {
            loan - phase::remaining_fraction(
                phase::cap(
                    dollars(eligible_loan_interest),
                    [crate::year_constants::STUDENT_LOAN_INTEREST_MAXIMUM_DOLLARS],
                ),
                phase::cap(
                    positive(
                        loan_start + loan_range
                            - before_loan
                            - excluded
                            - dollars(t.qsbs_excluded)
                            - nol_run.section1202_ira_adjustment
                            - nol_run.section1202_social_adjustment,
                    ),
                    [loan_range],
                ),
                loan_range,
            )
        } else {
            0
        };
    let form_1040_rules::AgiAssemblyOut {
        employee_counted,
        employee_deduction,
        social,
        olympic_taxable,
        agi,
    } = form_1040_rules::agi_assembly_phase(
        &pass,
        &mut r,
        &t,
        form_1040_rules::AgiAssemblyIn {
            employee: &employee,
            ira: &ira,
            before_loan,
            loan,
            employee_ordinary,
            social,
            before_social,
            exempt,
            excluded,
            adoption_excluded,
            olympic_prize,
        },
    );
    // P.L. 119-21 section 70425(b) subtracts A-class noncash from G's
    let foreign_magi = agi + excluded;
    r.rental_home_context = Some(home_use.clone());
    let home_use = crate::extended_rules::HomeUseContext::observed(
        d,
        [agi, foreign_magi],
        home_route,
        &rental,
    );
    if allocated_entries.iter().any(|e| matches!(&e.income, FederalIncome::Rental { data } if !matches!(data.personal_use, crate::RentalPersonalUse::No))) && partner_pass.home_agi.as_ref() != Some(&home_use) {
        partner_pass.home_next = Some(home_use);
    }
    let coverdell_excess_owners =
        crate::coverdell::calculate(d, foreign_magi, s.filing_status(), &mut r);
    let coverdell_excess_tax = coverdell_excess_owners.iter().sum::<i64>();
    t.injured_accounts.fixed_tax(coverdell_excess_owners);
    let mut clean_vehicle = crate::clean_vehicle::calculate(
        context.forms(d),
        c.clean_vehicles.as_ref(),
        p,
        s.filing_status(),
        foreign_magi,
        &mut r,
    );
    let capital_sources = {
        let mut sources = t.injured_capital_sources.clone();
        for (id, cuts) in &rental.k1_cuts.capital_by_id {
            sources.add(
                d.income.iter().find(|e| e.id == *id).map(|e| e.owner.owned()),
                cuts.map(|v| v.get()),
            );
        }
        if let Some(property) = injured_property_sources {
            for (index, owner) in [Owner::Taxpayer, Owner::Spouse].into_iter().enumerate() {
                sources.add(Some(owner), [0, property.long_term[index]]);
                sources.add_special(Some(owner), [0, property.unrecaptured[index]]);
            }
        } else {
            // Final Form4797/1231 owner characters remain an
            // independent source feed, including canceling changes.
            sources.invalidate();
        }
        sources
    };
    let capital_source_owners = capital_sources.owners(
        d,
        [short, long],
        [
            dollars(t.collectible_long),
            dollars(t.unrecaptured_long)
                + property_character.as_ref().map_or(0, |source| source.unrecaptured),
        ],
    );
    let niit_magi = agi + foreign.gross_excluded.iter().sum::<i64>()
        - business_exclusions.iter().map(|v| v.related).sum::<i64>()
        + dollars(t.estate_niit_magi);
    // Form 8960's default section 6013(g)/(h) treatment uses the resident's
    // own NII and MAGI, the MFS capital-loss limit and the $125,000 threshold.
    // Reuse the settled source gains and histories; no document is recalculated.
    // https://www.irs.gov/instructions/i8960
    let nonresident_ticks = d.whole_return();
    let separate_niit = s.filing_status() == FederalStatus::Joint
        && nonresident_ticks & crate::SpouseSituations::NONRESIDENT_SPOUSE.0 != 0
        && nonresident_ticks & crate::SpouseSituations::NIIT_JOINT_ELECTION.0 == 0;
    let nonresident =
        usize::from(nonresident_ticks & crate::SpouseSituations::YOU_WERE_THE_NONRESIDENT.0 == 0);
    let resident = 1 - nonresident;
    // Wages, interest, ordinary dividends and early-withdrawal penalties.
    let mut nonresident_amounts = [0_i64; 4];
    let mut resident_niit_capital = None;
    let mut resident_niit_prior = None;
    if separate_niit {
        let source = (|| {
            let mut nonresident_cents: [Exact; 4] = core::array::from_fn(|_| exact::whole(0));
            let mut long_adjustment = exact::whole(0);
            for entry in &allocated_entries {
                let (weights, denominator) = entry.accepted_assignment().owner_weights()?;
                let fund = match &entry.income {
                    FederalIncome::Dividend { data } => Some((
                        crate::investment_statements::components(entry)?.section_1202_gain,
                        data.section_1202_exclusion,
                    )),
                    FederalIncome::UndistributedGain { data } => {
                        let data = crate::UndistributedGain::from(data);
                        Some((data.small_business, data.section_1202_exclusion))
                    }
                    _ => None,
                };
                if let Some((gain, exclusion)) = fund {
                    let native = crate::investment_statements::allocated_fund_gain(
                        gain, exclusion, None, false,
                    );
                    let selected = crate::investment_statements::allocated_fund_gain(
                        fraction(gain, weights[resident], denominator),
                        exclusion,
                        None,
                        true,
                    );
                    // Replace only the resident's actual native exclusion.
                    // Fractional native source cents survive until the same
                    // annual capital boundary as all other source gains.
                    long_adjustment += exact::whole(native.excluded)
                        * exact::ratio(weights[resident], denominator)?
                        - exact::whole(selected.excluded);
                }
                if weights[nonresident] == 0 {
                    continue;
                }
                let weight = exact::ratio(weights[nonresident], denominator)?;
                let mut penalty = 0;
                let amount = match &entry.income {
                    FederalIncome::Wage { data } => (0, crate::FederalWage::from(data).wages),
                    FederalIncome::Interest { data } => {
                        penalty = crate::FederalInterest::from(data).early_penalty;
                        (1, crate::investment_statements::components(entry)?.interest)
                    }
                    FederalIncome::Oid { data } => {
                        penalty = crate::FederalOid::from(data).early_penalty;
                        (1, crate::investment_statements::components(entry)?.interest)
                    }
                    FederalIncome::Dividend { .. } => {
                        (2, crate::investment_statements::components(entry)?.ordinary_dividends)
                    }
                    // These actual gains already entered CapitalSources through
                    // their native collectors, including lots and fund exclusions.
                    FederalIncome::ReturnOfCapital { .. }
                    | FederalIncome::UndistributedGain { .. }
                    | FederalIncome::Sale { .. }
                    | FederalIncome::WorthlessSecurity { .. }
                    | FederalIncome::NonbusinessBadDebt { .. }
                    | FederalIncome::QualifiedSmallBusinessSale { .. } => (0, 0),
                    _ => return None,
                };
                nonresident_cents[amount.0] += exact::whole(amount.1) * &weight;
                nonresident_cents[3] += exact::whole(penalty) * weight;
            }
            let native = [
                exact::whole(t.wages.iter().sum()),
                exact::whole(t.interest) - exact::whole(savings_bond_exclusion) * exact::whole(100),
                exact::whole(t.dividends),
                exact::whole(t.early_penalty),
            ];
            let targets = [dollars(t.wages.iter().sum()), interest, dividends, savings_penalty];
            let mut excluded = [0; 4];
            for (category, (total, other)) in native.into_iter().zip(nonresident_cents).enumerate()
            {
                // Round the actual resident's annual source once. The other
                // spouse receives the same native category's exact residual.
                excluded[category] = targets[category] - exact::dollars(&(total - other))?;
            }
            let selected = capital_sources.separate_income(d, resident, long_adjustment)?;
            let prior = crate::injured_spouse::niit_difference_owners(d)?[resident];
            Some((excluded, selected, prior))
        })();
        let Some((excluded, selected, prior)) = source else {
            crate::issue_localized!(
                &mut r,
                &crate::IssueContext::Section(crate::TaxSection::Spouse),
                "Net investment income tax",
                "ISSUE_NIIT_NONRESIDENT_SPOUSE_DOCUMENTS_NEED_THE_JOINT_ELECTION",
                "Establish the resident spouse's source income, deductions and capital history for separate net investment income tax figures, or review the joint election for Form 8960.",
            );
            return r;
        };
        nonresident_amounts = excluded;
        resident_niit_capital = Some(selected);
        resident_niit_prior = Some(prior);
        note_once(&mut r, msg!("NOTE_NIIT_NONRESIDENT_SPOUSE_SEPARATE"));
    }
    let niit_capital = resident_niit_capital.as_ref().map_or(capital, |source| source.income);
    let niit_capital_net =
        resident_niit_capital.as_ref().map_or(short + long, |source| source.short + source.long);
    let niit_savings_penalty = savings_penalty - nonresident_amounts[3];
    let nonresident_income: i64 = nonresident_amounts[..3].iter().sum();
    let niit_magi =
        niit_magi - nonresident_income + nonresident_amounts[3] - capital + niit_capital;
    line(
        &mut r,
        "net_investment_modified_agi",
        niit_magi,
        &["agi", "foreign_earned_exclusion", "income.estate_trust_portfolio.niit_adjustment"],
        "https://www.irs.gov/instructions/i8960",
        msg!("EXPLAIN_NET_INVESTMENT_MODIFIED_AGI"),
    );
    line(
        &mut r,
        "wages",
        wages,
        &["income.wages"],
        "https://www.irs.gov/instructions/i1040gi",
        if t.community { msg!("EXPLAIN_WAGES") } else { msg!("EXPLAIN_WAGES_2") },
    );
    line(
        &mut r,
        "capital_gain",
        capital,
        &["income.sales", "income.dividends"],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_CAPITAL_GAIN"),
    );
    form_line(
        &mut r,
        FormLine::new(TaxForm::Schedule2, "4"),
        "self_employment_tax",
        se_tax,
        &["income.business", "income.social_wages"],
        "https://www.irs.gov/instructions/i1040sse",
        {
            let taxpayer = selected179!(crate::clergy::se_exemption_with_section179_with_claims(
                d,
                Owner::Taxpayer,
                net[0],
                selected_regular,
                Some(&office_context),
                source_claims
            ));
            let spouse = selected179!(crate::clergy::se_exemption_with_section179_with_claims(
                d,
                Owner::Spouse,
                net[1],
                selected_regular,
                Some(&office_context),
                source_claims
            ));
            if taxpayer.any() || spouse.any() {
                crate::Message::new(
                    "EXPLAIN_SE_EXEMPTION_BOXES",
                    alloc::vec![
                        crate::MessageArg::Number(taxpayer.box_index),
                        crate::MessageArg::Text(taxpayer.code().into()),
                        crate::MessageArg::Number(i64::from(taxpayer.line_a)),
                        crate::MessageArg::Number(spouse.box_index),
                        crate::MessageArg::Text(spouse.code().into()),
                        crate::MessageArg::Number(i64::from(spouse.line_a)),
                    ],
                )
            } else {
                msg!("EXPLAIN_SELF_EMPLOYMENT_TAX")
            }
        },
    );
    line(
        &mut r,
        "taxable_social_security",
        social,
        &["income.social_security", "income", "adjustments"],
        "https://www.irs.gov/publications/p915",
        msg!("EXPLAIN_TAXABLE_SOCIAL_SECURITY"),
    );
    line(
        &mut r,
        "adjustments",
        half_se
            + sep_deduction
            + health.deduction
            + educator
            + employee_deduction
            + dollars(hsa)
            + msa_deduction
            + ira.deduction
            + pension.deduction
            + loan
            + savings_penalty
            + alimony_paid
            + moving_deduction
            + jury_remitted
            + estate_67e,
        &["self_employment_tax", "adjustments"],
        ANNUAL,
        msg!("EXPLAIN_ADJUSTMENTS"),
    );
    emit_lines(
        &mut r,
        &mut (),
        [
            ("ira_deduction", ira.deduction),
            ("taxpayer_nondeductible_ira", ira.nondeductible[0]),
            ("spouse_nondeductible_ira", ira.nondeductible[1]),
        ],
        |(id, amount), _, _| {
            Some(calculation_row(
                (amount > 0
                    && id.ends_with("_nondeductible_ira")
                    && !a
                        .ira_basis
                        .iter()
                        .any(|basis| (basis.owner == Owner::Spouse) == id.starts_with("spouse_")))
                .then(|| FormLine::new(TaxForm::Form8606, "1")),
                id,
                LineValue::MoneyDollars(amount),
                &["adjustments.iras", "income", "people"],
                "https://www.irs.gov/publications/p590a",
                msg!("EXPLAIN_FEDERAL_RULES_3"),
            ))
        },
    );
    line(
        &mut r,
        "agi",
        medical_dollars!(exact::whole(agi) * exact::whole(100)),
        &["income", "taxable_social_security", "adjustments"],
        "https://www.irs.gov/publications/p505",
        msg!("EXPLAIN_AGI"),
    );
    let standard_deduction_rules::StandardDeductionOut { standard, senior_ded } =
        standard_deduction_rules::standard_deduction_phase(
            &pass,
            &mut r,
            standard_deduction_rules::StandardDeductionIn {
                rental: &rental,
                modify_income: nol_run.modify_income,
                earned,
                dependent_standard,
                partner_standard_complete,
                strike_benefits,
                deferred_pension,
                inmate_pay,
                scholarship,
                partner_standard_earned,
                foreign_magi,
            },
        );
    let ScheduleASourcesOut {
        estate_state_income_tax,
        estate_state_sales_tax,
        salt,
        gst_tax,
        bond_premium,
        debt_instrument_loss,
        impairment_work,
        investment_property_taxes,
        office_schedule_a,
        rental_personal,
        schedule_a_state_local,
        salt_eligible,
    } = match schedule_a_sources_phase(
        &pass,
        &mut r,
        &t,
        &mut partner_pass.office_next,
        ScheduleASourcesIn { agi, foreign_magi, office_current: &partner_pass.office },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let k3_basis = crate::schedule_k3::basis(d, &rental);
    let foreign_tax_deduction = crate::extended_rules::foreign_tax_deduction(
        d,
        &foreign,
        &business_exclusions,
        &k3_basis.limits,
        &rental,
    );
    let foreign_tax_deductible = dollars(foreign_tax_deduction.tax);
    let foreign_investment_tax = dollars(foreign_tax_deduction.investment);
    let foreign_portfolio_tax = dollars(foreign_tax_deduction.portfolio);
    let investment_tax_bounds = ded.investment_income_tax_bounds(&salt_eligible, salt);
    let investment_expenses = royalty_investment_expenses
        + investment_property_taxes
        + investment_tax_bounds[1]
        + foreign_portfolio_tax;
    let MortgageDeductionOut {
        mortgage_credit_current,
        mortgage_8a,
        mortgage_8b,
        mortgage_8c,
        mortgage_8d,
        amt_mortgage,
        mortgage_deductible_interest,
    } = match mortgage_deduction_phase(
        &pass,
        &mut r,
        MortgageDeductionIn {
            agi,
            foreign_magi,
            office_schedule_a: &office_schedule_a,
            rental_personal: &rental_personal,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    if !ded.whaling_captains.is_empty() {
        crate::require_localized!(
            &mut r,
            ded.whaling_captains.iter().all(|claim| claim.owner == crate::Owner::Taxpayer || j),
            &crate::IssueContext::Section(crate::TaxSection::GiftCarryovers),
            "Whaling captain charity",
            "ISSUE_CONFIRM_EACH_CLAIMANT_IS_ON_THIS_RETURN_WITH_CURRENT_AEWC_RECOGNITI_EB3950A0",
            "Whaling captain expenses can be claimed only by a captain on this return: your spouse's expenses need a joint return. A whaling expense can be claimed only as an itemized contribution.",
        );
    }
    // The same qualifying classroom expense is deducted above the line once; only its
    // remainder enters the new 2026 itemized educator deduction (P.L.119-21 section 70110).
    let educator_itemized = positive(
        educator_net(a.educator_taxpayer(), 0)
            - crate::year_constants::EDUCATOR_EXPENSE_LIMIT_DOLLARS,
    ) + if j {
        positive(
            educator_net(a.educator_spouse(), 1)
                - crate::year_constants::EDUCATOR_EXPENSE_LIMIT_DOLLARS,
        )
    } else {
        0
    };
    let educator_itemized = educator_itemized
        + dollars(a.educator_itemized_only(crate::Owner::Taxpayer))
        + if j { dollars(a.educator_itemized_only(crate::Owner::Spouse)) } else { 0 };
    let mut marketplace_income =
        agi + excluded + exempt + positive(dollars(t.social) - social) + t.child.household_income;
    for dependent in
        crate::marketplace::facts(&c.marketplace).map(|f| f.dependents.as_slice()).unwrap_or(&[])
    {
        crate::require_localized!(
            &mut r,
            family.dependent_ids.contains(&dependent.person),
            &crate::IssueContext::Section(crate::TaxSection::Marketplace),
            "Marketplace",
            "ISSUE_EVERY_REQUIRED_TO_FILE_HOUSEHOLD_INCOME_RECORD_MUST_REFER_TO_A_QUAL_EBA6B226",
            "Every required-to-file household income record must refer to a qualifying dependent on this return.",
        );
        marketplace_income += dependent.income();
    }
    let health_specified: crate::kernel::ratio::exact::Exact =
        health.specified.iter().map(|v| &v.deduction).sum();
    if !health.specified.is_empty() {
        let initial_passive = rental.health_ptc.initial_addback();
        crate::require_localized!(
            &mut r,
            initial_passive.is_ok(),
            &crate::IssueContext::Section(crate::TaxSection::Marketplace)
                .corrective(crate::TaxSection::SelfEmployedBenefits)
                .corrective(crate::TaxSection::Income),
            "Marketplace initial passive losses",
            "ISSUE_HEALTH_PTC_INITIAL_PASSIVE_ALLOCATION",
            "The initial health-insurance and premium-credit calculation must omit rental real estate losses before completing Form 8582. The source-supported interim treatment is not verified for competing non-rental passive losses, concurrent nonpassive rental losses, or disposed/former-passive loss releases. No complete estimate can be reported from a guessed allocation.",
        );
        partner_pass.specials.initialize(
            rental.health_ptc.deferred && initial_passive.is_ok(),
            covered_ira,
            crate::savings_bonds::record(&d.income).is_some(),
            eligible_loan_interest > 0 && !separate && loan_claim_allowed,
        );
        // Temporary omission changes the PTC candidate's income only. It
        // does not turn a deductible IRA deposit into nondeductible basis,
        // nor change the accepted contribution or interest facts.
        marketplace_income += partner_pass.specials.addback(
            initial_passive.unwrap_or(0),
            ira.deduction,
            savings_bond_exclusion,
            loan,
        );
    }
    let capital_work_long = if !health.specified.is_empty() && partner_pass.iterative {
        required_native!(
            i64::try_from(
                i128::from(t.long)
                    + i128::from(property_character.as_ref().map_or(0, |v| v.long_term)) * 100
            )
            .ok()
        )
    } else {
        t.long
    };
    let capital_work_unrecaptured = if !health.specified.is_empty() && partner_pass.iterative {
        required_native!(
            i64::try_from(
                i128::from(t.unrecaptured_long)
                    + i128::from(property_character.as_ref().map_or(0, |v| v.unrecaptured)) * 100
            )
            .ok()
        )
    } else {
        t.unrecaptured_long
    };
    let capital_work = if !health.specified.is_empty() && partner_pass.iterative {
        required_native!(
            crate::capital::CapitalAmounts {
                short: t.short,
                long: capital_work_long,
                collectibles: t.collectible_long,
                unrecaptured: capital_work_unrecaptured,
            }
            .checked_calculate_cents(
                capital_work_prior,
                separate,
                nol_run.modify_income
            )
        )
        .income
    } else {
        0
    };
    let calculation_income = (!health.specified.is_empty() && partner_pass.iterative)
        .then(|| {
            // These are the existing admitted owners' cent projections, before
            // their filing-only dollar rows. Preserve each residual once; this
            // does not recompute tax character or an annual allocation. Taxable
            // Social Security cancels in PTC MAGI, leaving its actual gross;
            // the foreign exclusion/addback likewise cancels unchanged.
            let source_residual = exact::whole(
                [
                    (capital_work, capital),
                    (rental.income * 100 + rental.health_work_income_residual_cents, rental.income),
                    (
                        t.wages.iter().sum::<i64>()
                            + t.disability_pension.iter().sum::<i64>()
                            + moving.excess.iter().sum::<i64>()
                            + t.deferral_excess.iter().sum::<i64>(),
                        wages,
                    ),
                    (t.iso_unreported.iter().sum(), iso_unreported),
                    (
                        allocated_entries
                            .iter()
                            .filter_map(|entry| match &entry.income {
                                FederalIncome::ForeignEmployment { data } => Some(data.wages),
                                _ => None,
                            })
                            .sum(),
                        foreign_wages,
                    ),
                    (t.interest, interest + savings_bond_exclusion),
                    (t.dividends, dividends),
                    (t.exempt, exempt),
                    (t.social, dollars(t.social)),
                    (t.retirement_taxable.iter().sum(), retirement),
                    (t.unemployment, dollars(t.unemployment)),
                    (t.business.iter().sum::<i64>() + t.community_business, business),
                    (t.alimony.iter().sum::<i64>() + alimony_recaptured, alimony),
                    (t.hobby, hobby),
                    (t.scorp_loan_gain[0], loan_gain),
                    (-t.stock_loss, -stock_loss),
                    (0, 0),
                    (t.tax_refund, tax_refund),
                    (education.income.iter().sum(), education_income),
                    (t.able.iter().sum(), able_income),
                    (t.care_benefits, care_benefits),
                    (t.personal_debt, personal_debt),
                    (t.prizes, prizes),
                    (
                        t.alaska
                            + t.deferred_pension.iter().sum::<i64>()
                            + t.inmate_pay.iter().sum::<i64>()
                            + t.digital
                            + t.other_8z
                            + t.recovery_8z
                            + t.not_for_profit,
                        other_line_8,
                    ),
                    (t.scholarship, scholarship),
                    (t.jury, jury),
                    (-t.jury_remitted, -jury_remitted),
                    (t.taxable_grants, taxable_grants),
                    (t.family_leave, family_leave),
                    (t.gambling, gambling),
                    (t.trump_death, trump_death),
                    (-t.early_penalty, -savings_penalty),
                    (-alimony_paid_cents, -alimony_paid),
                    (-educator_work_cents, -educator),
                    (-employee.ordinary_deduction_cents(), -employee_ordinary),
                    (-sep.by_business_cents.values().sum::<i64>(), -sep_deduction),
                    (-t.estate_67e, -estate_67e),
                    (-moving.deduction.iter().sum::<i64>(), -moving_deduction),
                    (-hsa, -dollars(hsa)),
                    // This existing pension-owner output is set at exactly the same
                    // confirmed/capped branch as its deduction. Preserve those source
                    // cents; never recover an expense from an unadmitted raw claim.
                    (-pension.saver.iter().sum::<i64>(), -pension.deduction),
                    // A covered-IRA claim is omitted until its ordered owner stage
                    // completes. Thereafter freeze its actual result, including this
                    // source-cent projection, alongside the existing filing amounts.
                    if covered_ira && partner_pass.specials.ira.is_none() {
                        (0, 0)
                    } else {
                        (-ira.work_deduction_cents, -ira.deduction)
                    },
                ]
                .into_iter()
                .map(|(cents, filing)| cents - filing * 100)
                .sum::<i64>(),
            ) + exact::whole(
                t.casualty_living_insurance + t.taxed_relief + t.health_assistance,
            ) + &t.inherited_medical_earnings
                + &t.archer_excess_earnings
                - exact::whole(casualty_living_insurance) * exact::whole(100)
                + &hsa_work_income
                + &t.inherited_medical[0] * exact::whole(100)
                - exact::whole(hsa_income) * exact::whole(100)
                + &t.archer_msa
                + &t.inherited_medical[1] * exact::whole(100)
                + &archer_employer_income_cents
                + &archer_correction_earnings_cents
                + &archer_correction_employer_cents
                + &archer_prior_employer_cents
                - exact::whole(archer_total_income) * exact::whole(100)
                + t.medicare_forms.iter().map(|form| &form.taxable).sum::<Exact>()
                + &t.inherited_medical[2] * exact::whole(100)
                - exact::whole(medicare_msa) * exact::whole(100)
                - &msa_deduction_cents
                + exact::whole(msa_deduction) * exact::whole(100)
                + &t.royalties
                + exact::whole(t.personal_rent)
                - exact::whole(t.personal_rent_expenses)
                + exact::whole(rental.personal_expense_adjustment) * exact::whole(100)
                - exact::whole(property_net) * exact::whole(100);
            let dependent_residual: i64 = crate::marketplace::facts(&c.marketplace)
                .into_iter()
                .flat_map(|facts| &facts.dependents)
                .map(|dependent| dependent.income_cents() - dependent.income() * 100)
                .sum();
            crate::kernel::ratio::exact::rounded(
                &(exact::whole(marketplace_income + health.deduction) * exact::whole(100)
                    + source_residual
                    + exact::whole(dependent_residual)
                    - health.work_deduction()),
            )
        })
        .flatten();
    if !health.specified.is_empty() && partner_pass.iterative && calculation_income.is_none() {
        crate::issue_localized!(
            &mut r,
            &crate::IssueContext::Section(crate::TaxSection::Marketplace).alternatives(&[
                crate::TaxSection::Marketplace,
                crate::TaxSection::SelfEmployedBenefits
            ]),
            "Marketplace calculation income",
            "MARKETPLACE_CALCULATION_INCOME_UNAVAILABLE",
            "The required income for the Marketplace and self-employed health insurance calculation cannot be represented in its native amount. No final tax can be reported.",
        );
        withhold_required_source_result(&mut r);
        return r;
    }

    let marketplace = if let Some(final_calculation) = &partner_pass.marketplace_final {
        final_calculation.append(&mut r);
        emit_lines(
            &mut r,
            &mut (),
            [
                ("marketplace_iterative_credit_work", final_calculation.outcome.calculation_credit),
                (
                    "marketplace_iterative_deduction_work",
                    crate::kernel::ratio::exact::rounded(&health_specified)
                        .expect("bounded health work amount"),
                ),
            ],
            |(id, amount), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyCents(amount),
                    &["credits.marketplace", "income.business.health_insurance"],
                    "https://www.irs.gov/publications/p974",
                    msg!("EXPLAIN_MARKETPLACE_ITERATIVE_PAIR"),
                ))
            },
        );
        final_calculation.outcome.clone()
    } else {
        let mut output = FederalResult::empty(d.revision);
        let outcome = crate::marketplace::calculate(
            d,
            marketplace_income,
            calculation_income,
            &crate::marketplace::family_people(d),
            s.filing_status(),
            dependent_filer,
            &mut output,
        );
        let calculation = MarketplaceSnapshot { outcome, output };
        calculation.append(&mut r);
        let outcome = calculation.outcome.clone();
        partner_pass.marketplace_trial = Some(alloc::boxed::Box::new(calculation));
        outcome
    };
    let (premium_refund, premium_repayment) = (marketplace.refundable, marketplace.repayment);
    // Pub974 Step3: allocate the same retained subsidy using canonical
    // enrollment premiums, then combine all source results once. Established
    // trade limits and actual entity-funded subsets remain source-specific.

    if !health.specified.is_empty() {
        partner_pass.health_method_complete = true;
        crate::require_localized!(
            &mut r,
            !dependent_filer,
            &(crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits)).manual(),
            "Business health insurance",
            "ISSUE_A_FILER_CLAIMED_AS_A_DEPENDENT_DOES_NOT_SHARE_MARKETPLACE_PREMIUMS_8CEE604B",
            "A filer claimed as a dependent does not share Marketplace premiums with the premium tax credit (Pub. 974 specified premiums).",
        );
        if partner_pass.marketplace_final.is_none() {
            partner_pass.specified_next = Some(
                health
                    .specified
                    .iter()
                    .map(|v| (v.activity, v.after_credit(&marketplace)))
                    .collect(),
            );
        }
        emit_lines(
            &mut r,
            &mut (),
            [
                (
                    "marketplace_specified_premiums",
                    health.specified.iter().map(|v| &v.premiums).sum(),
                    msg!("EXPLAIN_MARKETPLACE_SPECIFIED_PREMIUMS"),
                ),
                (
                    "marketplace_specified_maximum",
                    health.specified.iter().map(|v| &v.maximum).sum(),
                    msg!("EXPLAIN_MARKETPLACE_SPECIFIED_MAXIMUM"),
                ),
                (
                    "marketplace_specified_deduction",
                    health.specified.iter().map(|v| &v.deduction).sum(),
                    msg!("EXPLAIN_MARKETPLACE_SPECIFIED_DEDUCTION"),
                ),
            ],
            |(id, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(
                        crate::kernel::ratio::exact::dollars(&amount)
                            .expect("bounded health source"),
                    ),
                    &["income.business.health_insurance", "credits.marketplace"],
                    "https://www.irs.gov/publications/p974",
                    explanation,
                ))
            },
        );
    }
    partner_pass.health_usage_valid = health.marketplace_usage_valid(&marketplace);
    let marketplace_medical = health.marketplace_medical(d, &marketplace, &mut r);
    let annuity_loss = final_annuity_losses(d, &mut r);
    // Section 213(c): medical expenses the estate paid within a year after the
    // death, elected by the executor onto this return; zero is no election.
    let estate_medical = core::iter::once(&p.taxpayer)
        .chain(p.spouse.iter())
        .filter_map(|person| person.died.as_ref())
        .map(|final_return| final_return.estate_medical)
        .sum::<i64>();
    if estate_medical > 0 {
        line_cents(
            &mut r,
            "estate_paid_medical_election",
            estate_medical,
            &["people.died.estate_medical"],
            "https://www.irs.gov/publications/p559",
            msg!("EXPLAIN_ESTATE_PAID_MEDICAL_ELECTION"),
        );
    }

    let form_4952_rules::InvestmentSourcePoolsOut {
        scorp_investment,
        property_investment,
        regular_property_investment_known_zero,
        investment_carry_max,
        investment_carry_known,
        investment_gain,
        investment_net_capital,
        maximum_gain,
        maximum_net_capital,
        amt_investment_gain,
        amt_investment_net_capital,
        maximum_amt_gain,
        maximum_amt_net_capital,
    } = form_4952_rules::investment_source_pools_phase(
        &pass,
        &mut r,
        &t,
        form_4952_rules::InvestmentSourcePoolsIn {
            investment_facts,
            rental: &rental,
            amt_rental: &amt_rental,
            property_character: &property_character,
            amt_property_character: &amt_property_character,
            property_sources_known_absent,
            estate_investment_carry,
            investment_capital_in,
            pure_loan_gain,
            pure_amt_loan_gain,
            k1_investment_ordinary,
            stock_loss,
        },
    );
    let form_4952_rules::InvestmentElectionsOut {
        capital_election_unavailable,
        elected,
        tax_net_capital,
        investment_ordinary_income,
        investment_income,
        amt_elected,
        amt_tax_net_capital,
        amt_tax_pref,
        amt_tax_unrecaptured,
        amt_tax_collectible,
        amt_investment_adjustment,
        investment_interest,
        portfolio_interest,
    } = form_4952_rules::investment_elections_phase(
        &pass,
        &mut r,
        &t,
        form_4952_rules::InvestmentElectionsIn {
            investment_facts,
            royalty_routing: &royalty_routing,
            rental: &rental,
            amt_rental: &amt_rental,
            scorp_investment,
            investment_carry_known,
            investment_gain,
            investment_net_capital,
            maximum_net_capital,
            amt_investment_gain,
            amt_investment_net_capital,
            maximum_amt_net_capital,
            interest,
            dividends,
            qualified,
            amt_qualified,
            net_capital_gain,
            amt_net_capital_gain,
            able_income,
            education_income,
            royalty_investment_income,
            k1_amt_interest_adjustment,
            k1_amt_dividend_adjustment,
            k1_amt_qualified_adjustment,
            amt_schedule_d_18,
            amt_schedule_d_19,
            investment_paid,
            investment_expenses,
            royalty_investment_expenses,
        },
    );
    let CharityWorksheetOut {
        charity_claims,
        gifts,
        charity,
        noncash_claim,
        charity_excess,
        charity_carry,
        charity_deduction,
        standard_cash_allowed,
    } = match charity_worksheet_phase(
        &pass,
        &mut r,
        &t,
        nol_run,
        CharityWorksheetIn {
            agi,
            cash_charity,
            hosted_student_expenses,
            whaling_expenses,
            partner_cash,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let DeductionElectionOut {
        medical_before_limit,
        medical_deduction,
        amt_medical_deduction,
        casualty_net,
        qualified_net_casualty,
        ordinary_casualty_deduction,
        casualty_deduction,
        itemized,
        cash_standard,
        claimed_casualty,
        use_itemized,
    } = match deduction_election_phase(
        &pass,
        &mut r,
        &t,
        DeductionElectionIn {
            health: &health,
            amt_health: &amt_health,
            marketplace_medical: &marketplace_medical,
            personal_casualties: &personal_casualties,
            foreign_tax_deduction: &foreign_tax_deduction,
            portfolio_interest,
            casualty_capital,
            agi,
            estate_medical,
            gst_tax,
            foreign_tax_deductible,
            bond_premium,
            debt_instrument_loss,
            impairment_work,
            educator_itemized,
            annuity_loss,
            gambling_loss,
            salt,
            investment_property_taxes,
            mortgage_deductible_interest,
            charity_deduction,
            standard_cash_allowed,
            standard,
            estate_state_income_tax,
            estate_state_sales_tax,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    charity_settlement_reporting_phase(
        &pass,
        &mut r,
        &t,
        nol_run,
        CharitySettlementReportingIn {
            charity: &charity,
            gifts: &gifts,
            use_itemized,
            charity_deduction,
            charity_carry,
            charity_excess,
            hosted_student_expenses,
            whaling_expenses,
        },
    );
    let form_4952_rules::InvestmentInterestRefigureOut { investment_interest, portfolio_interest } =
        form_4952_rules::investment_interest_refigure_phase(
            &mut r,
            form_4952_rules::InvestmentInterestRefigureIn {
                investment_facts,
                royalty_routing: &royalty_routing,
                foreign_tax_deduction: &foreign_tax_deduction,
                use_itemized,
                capital_election_unavailable,
                investment_carry_known,
                investment_interest,
                portfolio_interest,
                investment_paid,
                elected,
                amt_elected,
                investment_gain,
                investment_net_capital,
                maximum_gain,
                maximum_net_capital,
                amt_investment_gain,
                amt_investment_net_capital,
                maximum_amt_gain,
                maximum_amt_net_capital,
                investment_ordinary_income,
                investment_income,
                amt_investment_adjustment,
                investment_expenses,
                royalty_investment_expenses,
                investment_property_taxes,
                investment_tax_bounds,
                foreign_portfolio_tax,
            },
        );
    let schedule_a_rules::DeductionAssemblyOut { deduction } =
        schedule_a_rules::deduction_assembly_phase(
            &mut r,
            &t,
            schedule_a_rules::DeductionAssemblyIn {
                use_itemized,
                itemized,
                standard,
                qualified_net_casualty,
                cash_standard,
                annuity_loss,
                gambling,
                gambling_loss,
            },
        );
    let schedule_1a_rules::Schedule1AWorkSourcesOut {
        qualified_tip_owners,
        qualified_overtime_owners,
        qualified_tips,
        qualified_overtime,
        qualified_business_tips,
        business_tips,
    } = match schedule_1a_rules::schedule_1a_work_sources_phase(
        &pass,
        &mut r,
        schedule_1a_rules::Schedule1AWorkSourcesIn {
            tips: t.tips,
            overtime: t.overtime,
            business_half_se: &business_half_se,
            health: &health,
            sep: &sep,
        },
    ) {
        core::ops::ControlFlow::Continue(value) => value,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let schedule_1a_rules::Schedule1ADeductionsOut { additional, tips_deducted } =
        schedule_1a_rules::schedule_1a_deductions_phase(
            &pass,
            &mut r,
            schedule_1a_rules::Schedule1ADeductionsIn {
                foreign_magi,
                qualified_tips,
                qualified_overtime,
                qualified_business_tips,
                business_tips: &business_tips,
            },
        );
    let form_8995_rules::QbiBasisHalfSeOut {
        before_qbi_signed,
        before_qbi,
        partner_qbi_complete,
        qbi_gross_unknown,
        qbi_half_se,
    } = form_8995_rules::qbi_basis_half_se_phase(
        &pass,
        &mut r,
        form_8995_rules::QbiBasisHalfSeIn {
            agi,
            deduction,
            senior_ded,
            additional,
            half,
            church_half,
            rental_inputs_complete: rental.inputs_complete,
        },
    );
    let form_8995_rules::QbiSourceSetupOut {
        activities,
        community_qbi_loss_sources,
        research_qbi_ids,
        research_business_income,
        released_qbi,
        partner_qbi_complete,
        ptp_qualified,
        injured_ptp,
        ptp_nol_complete,
        ptp_nol_owners_complete,
        ptp_nol_predebt_carry,
        ptp_loss_sources,
    } = match form_8995_rules::qbi_source_setup_phase(
        &pass,
        &mut r,
        nol_run,
        form_8995_rules::QbiSourceSetupIn {
            rental: &rental,
            partner_qbi_complete,
            excess_business_loss,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let form_8995_rules::PartnershipQbiSourcesOut {
        activities,
        community_qbi_loss_sources,
        research_qbi_ids,
        released_qbi,
        ptp_qualified,
        injured_ptp,
        ptp_loss_sources,
        partner_qbi_complete,
    } = match form_8995_rules::partnership_qbi_sources_phase(
        &pass,
        &mut r,
        form_8995_rules::PartnershipQbiSourcesIn {
            rental: &rental,
            health: &health,
            sep: &sep,
            qbi_half_se: &qbi_half_se,
            care: &partner_pass.care,
            before_qbi,
            excess_business_loss,
            qbi_gross_unknown,
            partner_qbi_complete,
            activities,
            community_qbi_loss_sources,
            research_qbi_ids,
            released_qbi,
            ptp_qualified,
            injured_ptp,
            ptp_loss_sources,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let form_8995_rules::ScorpEstateQbiSourcesOut {
        activities,
        community_qbi_loss_sources,
        research_qbi_ids,
        released_qbi,
        ptp_qualified,
        injured_ptp,
        partner_qbi_complete,
    } = match form_8995_rules::scorp_estate_qbi_sources_phase(
        &pass,
        &mut r,
        form_8995_rules::ScorpEstateQbiSourcesIn {
            rental: &rental,
            health: &health,
            property_character: property_character.as_ref(),
            before_qbi,
            partner_qbi_complete,
            activities,
            community_qbi_loss_sources,
            research_qbi_ids,
            released_qbi,
            ptp_qualified,
            injured_ptp,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let form_8995_rules::DirectQbiActivitiesOut {
        activities,
        research_business_income,
        farm_qbi_indices,
        business_qbi_ids,
        foreign_business_losses,
        foreign_business_profits,
    } = match form_8995_rules::direct_qbi_activities_phase(
        &pass,
        &mut r,
        &mut t,
        form_8995_rules::DirectQbiActivitiesIn {
            rental: &mut rental,
            property_character: property_character.as_ref(),
            business_exclusions: &business_exclusions,
            business_half_se: &business_half_se,
            health: &health,
            sep: &sep,
            tips_deducted: &tips_deducted,
            qbi_half_se: &qbi_half_se,
            before_qbi,
            activities,
            research_business_income,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let form_8995_rules::QbiExcessLossOut { activities, ptp_qualified, injured_ptp } =
        match form_8995_rules::qbi_excess_loss_phase(
            &pass,
            &mut r,
            form_8995_rules::QbiExcessLossIn {
                rental: &rental,
                property_character: property_character.as_ref(),
                business_qbi_ids: &business_qbi_ids,
                research_qbi_ids: &research_qbi_ids,
                ptp_loss_sources: &ptp_loss_sources,
                property_loss_deferred,
                activities,
                ptp_qualified,
                injured_ptp,
                foreign_business,
                qbi_gross_unknown,
                partner_qbi_complete,
                property_business_income,
                excess_business_loss,
                k1_business_ordinary,
                before_qbi,
                foreign_business_losses,
                foreign_business_profits,
            },
        ) {
            core::ops::ControlFlow::Continue(out) => out,
            core::ops::ControlFlow::Break(()) => return r,
        };
    let form_8995_rules::QbiFilingOut {
        activities,
        prior_business,
        prior_reit,
        qbi,
        qbi_ded,
        injured_qbi,
        research_qbi_deductions,
        qbi_carry,
        qbi_form,
    } = match form_8995_rules::qbi_filing_phase(
        &pass,
        &mut r,
        &t,
        nol_run,
        form_8995_rules::QbiFilingIn {
            activities,
            released_qbi,
            research_qbi_ids,
            business_qbi_ids,
            ptp_qualified,
            injured_ptp,
            qbi_gross_unknown,
            ptp_nol_complete,
            ptp_nol_owners_complete,
            before_qbi,
            pref,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let qbi_line = |form_8995: &str, form_8995a: &str| {
        qbi_form.map(|form| {
            FormLine::new(form, if form == TaxForm::Form8995 { form_8995 } else { form_8995a })
        })
    };
    form_8995_rules::qbi_carry_reporting_phase(
        &mut r,
        form_8995_rules::QbiCarryReportingIn {
            qbi: &qbi,
            qbi_carry,
            qbi_gross_unknown,
            ptp_nol_complete,
        },
        qbi_line,
    );
    let form_4952_rules::PostQbiDeductionOut {
        unlimited_deduction,
        itemized_reduction,
        investment_interest_deduction,
        deduction,
        taxable,
    } = form_4952_rules::post_qbi_deduction_phase(
        &pass,
        &mut r,
        form_4952_rules::PostQbiDeductionIn {
            royalty_routing: royalty_routing.as_ref(),
            investment_facts,
            use_itemized,
            investment_unavailable,
            additional,
            elected,
            amt_elected,
            portfolio_interest,
            royalty_interest,
            investment_paid,
            investment_interest,
            agi,
            deduction,
            itemized,
            before_qbi_signed,
            qbi_ded,
            investment_income,
            investment_gain,
            investment_net_capital,
            investment_expenses,
            royalty_investment_expenses,
            qualified,
            amt_investment_adjustment,
            amt_investment_gain,
            amt_investment_net_capital,
            amt_qualified,
        },
    );
    let income_sales = match schedule_a_detail_reporting_phase(
        &pass,
        &mut r,
        &t,
        ScheduleADetailReportingIn {
            use_itemized,
            office_schedule_a: &office_schedule_a,
            charity: &charity,
            salt_eligible: &salt_eligible,
            portfolio_interest,
            investment_property_taxes,
            estate_state_income_tax,
            estate_state_sales_tax,
            charity_deduction,
            qualified_net_casualty,
            gambling_loss,
            educator_itemized,
            annuity_loss,
            bond_premium,
            debt_instrument_loss,
            impairment_work,
            medical_before_limit,
            agi,
            medical_deduction,
            salt,
            gst_tax,
            foreign_tax_deductible,
            mortgage_deductible_interest,
            standard,
            itemized,
            itemized_reduction,
            mortgage_8a,
            mortgage_8b,
            mortgage_8c,
            mortgage_8d,
            ordinary_casualty_deduction,
            amt_mortgage,
            amt_related_adjustment,
            amt_medical_deduction,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let KiddieIncomeOut {
        service_businesses,
        classified_business_earned,
        unearned,
        investment_tax_deduction,
        child_direct_deductions,
        kiddie_deduction,
        parent_rate_required,
    } = match kiddie_income_phase(
        &pass,
        &mut r,
        KiddieIncomeIn {
            rental: &rental,
            business_half_se: &business_half_se,
            kiddie_age,
            young_8615,
            partner_young_complete,
            use_itemized,
            unearned,
            partner_young_loss_addback,
            partner_young_earned,
            savings_bond_exclusion,
            wages,
            iso_unreported,
            foreign_wages,
            benefit_wages,
            adoption_wages,
            agi,
            business,
            investment_property_taxes,
            foreign_portfolio_tax,
            itemized_reduction,
            itemized,
            investment_interest_deduction,
            gambling_loss,
            taxable,
            excluded,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    schedule_d_rules::capital_carry_reporting_phase(
        &pass,
        &mut r,
        schedule_d_rules::CapitalCarryReportingIn {
            additional,
            capital_in,
            itemized_reduction,
            short,
            long,
            capital,
            agi,
            deduction,
            qbi_ded,
        },
    );
    nol_rules::regular_nol_refigure_phase(
        &pass,
        &mut r,
        &t,
        nol_run,
        nol_rules::RegularNolRefigureIn {
            modified_ira: &modified_ira,
            ira: &ira,
            salt_eligible: &salt_eligible,
            charity_claims,
            additional,
            kiddie_age,
            qsbs_with_prior_nol,
            adoption_exclusion_unchanged_under_qsbs,
            passive_unchanged_under_qsbs,
            use_itemized,
            foreign_magi,
            noncash_claim,
            hosted_student_expenses,
            whaling_expenses,
            salt,
            medical_deduction,
            medical_before_limit,
            agi,
            ordinary_casualty_deduction,
            casualty_net,
            qualified_net_casualty,
            charity_deduction,
            itemized,
            unlimited_deduction,
            senior_ded,
            qbi_ded,
            itemized_reduction,
        },
    );
    let nol_rules::CurrentNolOut {
        nonbusiness_capital_net,
        amt_nonbusiness_capital_net,
        amt_business_capital_net,
        nonbusiness_income,
        regular_business_casualty,
        nonbusiness_deductions,
        nol,
        prior_capital_source_known,
        loss_carryovers,
    } = nol_rules::current_nol_phase(
        &pass,
        &mut r,
        &t,
        nol_run,
        nol_rules::CurrentNolIn {
            ira: &ira,
            regular_capital: &regular_capital,
            property_character: property_character.as_ref(),
            amt_property_character: amt_property_character.as_ref(),
            regular_capital_amounts,
            amt_capital_amounts,
            estate_business_carry,
            capital_in,
            amt_capital_in,
            additional,
            use_itemized,
            agi,
            deduction,
            senior_ded,
            interest,
            dividends,
            retirement,
            trump_death,
            social,
            hsa_income,
            alimony,
            gambling,
            jury,
            scholarship,
            prizes,
            other_line_8,
            olympic_taxable,
            taxable_grants,
            family_leave,
            casualty_living_insurance,
            personal_debt,
            care_benefits,
            medicare_msa,
            archer_total_income,
            able_income,
            education_income,
            tax_refund,
            property_net,
            loan_gain,
            k1_business_ordinary,
            hobby,
            claimed_casualty,
            itemized_reduction,
            itemized,
            sep_deduction,
            hsa,
            msa_deduction,
            loan,
            savings_penalty,
            alimony_paid,
            jury_remitted,
            estate_67e,
            capital,
            salt,
            excess_business_loss,
        },
    );
    let form_1040_rules::RegularTaxSetupOut {
        current_preference,
        regular_tax_pref,
        regular_collectible,
        regular_unrecaptured,
        regular_tax_worksheet,
        regular_tax,
    } = form_1040_rules::regular_tax_setup_phase(
        &pass,
        form_1040_rules::RegularTaxSetupIn {
            foreign: &foreign,
            elected,
            qualified,
            tax_net_capital,
            schedule_d_18,
            schedule_d_19,
            taxable,
            excluded,
        },
    );
    let KiddieTaxOut { regular_tax, kiddie_credit_tax } = kiddie_tax_phase(
        &pass,
        &mut r,
        &t,
        KiddieTaxIn {
            foreign: &foreign,
            investment_facts,
            parent_rate_required,
            use_itemized,
            service_businesses,
            classified_business_earned,
            elected,
            regular_tax,
            taxable,
            excluded,
            unearned,
            kiddie_deduction,
            pref,
            investment_interest_deduction,
            itemized_reduction,
            itemized,
            annuity_loss,
            agi,
            deduction,
            standard,
            qualified,
            child_direct_deductions,
            investment_tax_deduction,
            net_capital_gain,
            schedule_d_18,
            schedule_d_19,
        },
    );
    // Form 6251 line 10 disregards Schedule J; credits and Form 1040 use
    // the elected regular tax. Never let averaging create an offsetting AMT.
    let unaveraged_regular_tax = regular_tax;
    let ScheduleJAveragingOut { regular_tax, averaged_tax } = match schedule_j_averaging_phase(
        &pass,
        &mut r,
        &t,
        ScheduleJAveragingIn {
            business_half_se: &business_half_se,
            health: &health,
            sep: &sep,
            rental: &rental,
            property_character: property_character.as_ref(),
            foreign: &foreign,
            activities: &activities,
            qbi: &qbi,
            farm_qbi_indices: &farm_qbi_indices,
            current_preference,
            use_itemized,
            parent_rate_required,
            regular_tax,
            qbi_ded,
            prior_business,
            prior_reit,
            taxable,
            excluded,
            excess_business_loss,
            msa_deduction,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let AmtAdjustmentsOut {
        iso,
        amt_tax_recovery,
        amt_base,
        amt_rental_adjustment,
        amt_excess_business_loss,
        amt_property_loss_deferred,
        k1_depletion_adjustment,
    } = amt_adjustments_phase(
        &pass,
        &mut r,
        &t,
        nol_run,
        AmtAdjustmentsIn {
            rental: &rental,
            amt_rental: &amt_rental,
            amt_health: &amt_health,
            amt_property_character: amt_property_character.as_ref(),
            amt_direct_property_sources: &amt_direct_property_sources,
            amt_activity_issues,
            additional,
            royalty_interest,
            portfolio_interest,
            use_itemized,
            tax_refund,
            salt,
            investment_property_taxes,
            foreign_tax_deductible,
            standard,
            agi,
            k1_amt_interest_adjustment,
            k1_amt_dividend_adjustment,
            amt_mortgage,
            amt_loan_gain,
            loan_gain,
            amt_capital_independent_adjustment,
            unlimited_deduction,
            qbi_ded,
            iso_amt_wage_adjustment,
            amt_related_adjustment,
            medical_deduction,
            amt_medical_deduction,
            amt_property_ordinary,
            property_ordinary,
            business,
            half_se,
            amt_sep_deduction,
            business_excluded,
            amt_property_business_income,
            amt_direct_property_ordinary,
            excess_business_loss,
            business_depletion_adjustment,
        },
    );
    let AmtCurrentNolOut { amt_nol, farm_route } = match amt_current_nol_phase(
        &pass,
        &mut r,
        &t,
        nol_run,
        partner_pass,
        AmtCurrentNolIn {
            rental: &rental,
            amt_rental: &amt_rental,
            ira: &ira,
            amt_capital_result: &amt_capital_result,
            estate_business_carry,
            royalty_interest,
            portfolio_interest,
            amt_capital_in,
            use_itemized,
            prior_capital_source_known,
            unlimited_deduction,
            salt,
            investment_property_taxes,
            foreign_tax_deductible,
            amt_medical_deduction,
            medical_deduction,
            qualified_net_casualty,
            nonbusiness_income,
            k1_amt_interest_adjustment,
            k1_amt_dividend_adjustment,
            amt_loan_gain,
            loan_gain,
            amt_k1_business_ordinary,
            k1_business_ordinary,
            amt_tax_recovery,
            amt_nonbusiness_capital_net,
            nonbusiness_capital_net,
            nonbusiness_deductions,
            sep_deduction,
            amt_sep_deduction,
            amt_ira_deduction,
            deduction,
            regular_business_casualty,
            claimed_casualty,
            amt_business_capital_net,
            amt_base,
            qbi_ded,
            amt_capital,
            nol,
            noncash_claim,
            amt_excess_business_loss,
            amt_rental_adjustment,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let form_6251_rules::AmtAbsorptionOut {
        has_atnol,
        prior_atnol,
        amt_base,
        amt_computation,
        tentative,
    } = match form_6251_rules::amt_absorption_phase(
        &pass,
        &mut r,
        &t,
        nol_run,
        form_6251_rules::AmtAbsorptionIn {
            k1_depletion_adjustment,
            royalty_regular: &royalty_regular,
            royalty_amt: &royalty_amt,
            foreign: &foreign,
            royalty_limits: &partner_pass.royalty_limits,
            royalty_next: &mut partner_pass.royalty_next,
            amt_base,
            business_depletion_adjustment,
            amt_capital,
            qbi_ded,
            before_qbi_signed,
            itemized_reduction,
            amt_short,
            amt_long,
            amt_capital_independent_adjustment,
            amt_tax_collectible,
            amt_tax_unrecaptured,
            amt_tax_pref,
            taxable,
            excluded,
            regular_tax_pref,
            regular_collectible,
            regular_unrecaptured,
            amt_qualified,
            amt_tax_net_capital,
            amt_schedule_d_18,
            amt_schedule_d_19,
            kiddie_age,
            amt_elected,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let form_1040_rules::RegularLiabilityOut {
        bba_audit,
        bba_credit,
        bba_additional_tax,
        child_tax,
        clean_vehicle_repayment,
        education_recapture_tax,
        lump_sum_tax,
        regular_liability,
    } = form_1040_rules::regular_liability_phase(
        &pass,
        &mut r,
        &t,
        form_1040_rules::RegularLiabilityIn {
            clean_vehicle: &clean_vehicle,
            regular_tax,
            premium_repayment,
        },
    );
    let form_1116_rules::ForeignCreditOut {
        foreign_basis,
        foreign_credit,
        amt_credit,
        rental_gross,
        royalty_gross,
        foreign_credit_sources,
    } = match form_1116_rules::foreign_credit_phase(
        &pass,
        &mut r,
        &t,
        nol_run,
        form_1116_rules::ForeignCreditIn {
            property_character: property_character.as_ref(),
            amt_property_character: amt_property_character.as_ref(),
            rental: &rental,
            amt_rental: &amt_rental,
            business_half_se: &business_half_se,
            sep: &sep,
            health: &health,
            royalty_total: &royalty_total,
            salt_eligible: &salt_eligible,
            foreign_tax_deduction: &foreign_tax_deduction,
            ira: &ira,
            foreign: &foreign,
            regular_capital: &regular_capital,
            amt_capital_result: &amt_capital_result,
            business_exclusions: &business_exclusions,
            k3_basis: &k3_basis,
            use_itemized,
            regular_tax_worksheet,
            royalty_interest,
            portfolio_interest,
            elected,
            additional,
            mortgage_deductible_interest,
            charity_deduction,
            cash_standard,
            amt_medical_deduction,
            casualty_deduction,
            qualified_net_casualty,
            itemized_reduction,
            itemized,
            amt_loan_gain,
            loan_gain,
            gambling_loss,
            annuity_loss,
            investment_property_taxes,
            regular_liability,
            lump_sum_tax,
            bba_credit,
            tentative,
            agi,
            deduction,
            qbi_ded,
            amt_base,
            foreign_tax_deductible,
            salt,
            interest,
            dividends,
            taxable,
            qualified,
            regular_tax_pref,
            regular_collectible,
            regular_unrecaptured,
            amt_tax_pref,
            amt_tax_collectible,
            amt_tax_unrecaptured,
            capital,
            property_ordinary,
            amt_tax_recovery,
            iso,
            educator_itemized,
            wages,
            foreign_wages,
            excess_business_loss,
            regular_tax,
            educator,
            housing_deduction,
            hsa,
            msa_deduction,
            alimony_paid,
            estate_67e,
            half_se,
            sep_deduction,
            savings_penalty,
            jury_remitted,
            employee_deduction,
            moving_deduction,
            loan,
            business,
            stock_loss,
        },
    ) {
        core::ops::ControlFlow::Continue(out) => out,
        core::ops::ControlFlow::Break(()) => return r,
    };
    form_8978_rules::bba_audit_reporting_phase(&mut r, &bba_audit);
    let amt = match form_6251_rules::amt_final_tax_phase(
        &pass,
        &mut r,
        form_6251_rules::AmtFinalTaxIn {
            amt_computation: &amt_computation,
            unaveraged_regular_tax,
            bba_additional_tax,
            child_tax,
            education_recapture_tax,
            premium_repayment,
            clean_vehicle_repayment,
            foreign_credit,
            bba_credit,
            tentative,
            amt_credit,
            amt_base,
        },
    ) {
        core::ops::ControlFlow::Continue(amt) => amt,
        core::ops::ControlFlow::Break(()) => return r,
    };
    let form_1040_rules::PrecreditTaxReportingOut {
        income_tax,
        bba_schedule_3_credit,
        bba_after_line_18,
        creditable_tax,
        regular_creditable_tax,
    } = form_1040_rules::precredit_tax_reporting_phase(
        &mut r,
        form_1040_rules::PrecreditTaxReportingIn {
            use_itemized,
            foreign_credit,
            regular_liability,
            amt,
            bba_credit,
            deduction,
            senior_ded,
            qbi_ded,
            taxable,
        },
        qbi_line,
    );
    // Form 8862 bans (section 32(k), 24(g), 25A(b)(4)): each zeroes only its
    // own credit group; the notes state them where the credit would apply.
    let bans = crate::forms::form_8862::bans(c.after_disallowance.as_ref());
    let Form8863Out { aotc, llc, injured_education_sources, education_refund, education_nonrefund } =
        form_8863_phase(
            &pass,
            &mut r,
            Form8863In {
                foreign_magi,
                education_allowed,
                savings_bond_context: &savings_bond_context,
                education_context: &education_context,
            },
        );
    let care_components = form_2441.credit(&form_2441_context, agi, &mut r);
    let care = care_components.total;
    let elderly_potential = schedule_r_phase(&pass, &mut r, ScheduleRIn { t: &t, agi, social });
    let Form8880Out { saver_available, saver_eligible_sources } =
        form_8880_phase(&pass, &mut r, Form8880In { ira: &ira, foreign_magi });
    let PayrollTaxOut {
        payroll_withheld,
        payroll_other,
        excess_social,
        niit_threshold,
        known_additional_parts,
        additional_medicare_self_employment,
        additional_medicare_wages,
        additional_medicare_rrta_withheld,
        possible_additional_parts,
        medicare_withheld,
        injured_medicare_withholding,
        additional_medicare,
        additional_medicare_sources,
        employee_payroll_complete,
        payroll_agi_known,
        payroll_se_net_known,
    } = payroll_tax_phase(
        &pass,
        &mut r,
        &t,
        PayrollTaxIn {
            tip_payroll,
            employee_wage_tax,
            uncollected_payroll,
            separate_niit,
            net,
            payroll_agi_known,
            payroll_se_net_known,
            se_church_wages,
            half,
            employee_coverage_possible,
            employee_coverage_taxes,
            se,
            employee_payroll_complete,
            employee_medicare_complete,
            withholding_25a,
            withholding_25b,
            withholding_other_dollars,
            rental: &rental,
            se_sources_complete,
        },
    );
    let FamilyCreditsOut {
        adoption,
        homeowner_potentials,
        combat_pay_election,
        schedule_eic,
        eic,
        schedule_8812,
    } = family_credits_phase(
        &pass,
        &mut r,
        &t,
        FamilyCreditsIn {
            bans,
            rental: &rental,
            interest,
            exempt,
            dividends,
            short,
            long,
            property_character: &property_character,
            property_net,
            foreign: &foreign,
            earned,
            adoption_benefits,
            optional_eic,
            agi,
            foreign_magi,
            optional_child,
            payroll_withheld,
            medicare_withheld,
            additional_medicare_rrta_withheld,
            additional_medicare,
            payroll_other,
            half_se,
            excess_social,
            care,
            elderly_potential,
            education_nonrefund,
            saver_available,
            creditable_tax,
            clean_vehicle: &clean_vehicle,
            mortgage_credit_current,
        },
    );
    // Form 8862 (T2-46): whether each previously disallowed credit may be
    // claimed for 2026, filled from the people, children and students above.
    if let Some(claim) = c.after_disallowance.as_ref() {
        crate::forms::form_8862::compute(
            &crate::forms::form_8862::Form8862Context {
                claim,
                people: p,
                dependents: d.dependents(),
                eic_children: &family.eic_child_ids,
                ctc_children: &family.ctc_ids,
                other_dependents: &family.odc_ids,
                education: &c.education_inputs(),
                students: &student_catalog,
                joint: j,
                eic_claimed: eic > 0,
                child_credits_claimed: schedule_8812.potential > 0,
                aotc_claimed: aotc > 0,
            },
            &mut r,
        );
    }
    // Schedule 8812 Credit Limit Worksheet B reserves only the part of the
    // child credit that cannot be refundable before limiting Form 8839.
    // Without Form 2555 eligibility, the ACTC limit is zero and all CTC is reserved.
    let credit_potentials = crate::income_tax::PersonalCreditPotentials {
        care,
        elderly: elderly_potential,
        education: education_nonrefund,
        saver: saver_available,
        adoption: adoption.nonrefundable,
        child: schedule_8812.potential,
        actc_limit: schedule_8812.actc_limit,
    };
    let limited_credits = credit_potentials.limit(creditable_tax);
    let care_used = limited_credits.care;
    let elderly = limited_credits.elderly;
    let education_used = limited_credits.education;
    let saver = limited_credits.saver;
    let Form8936Out { used_clean_vehicle_credit, new_clean_vehicle_credit } = form_8936_phase(
        &pass,
        &mut r,
        Form8936In {
            clean_vehicle: &clean_vehicle,
            creditable_tax,
            care_used,
            elderly,
            education_used,
            saver,
        },
    );
    // Schedule 8812 Worksheet B reserves only the child credit that cannot
    // become ACTC while it limits the homeowner carryforwards. The final
    // nonrefundable child amount follows those credits.
    let preceding_mortgage_credits = care_used
        + elderly
        + education_used
        + saver
        + used_clean_vehicle_credit
        + new_clean_vehicle_credit;
    let Form8396Out { homeowner_credits, mortgage_credit_used } = form_8396_phase(
        &pass,
        &mut r,
        Form8396In {
            schedule_8812: &schedule_8812,
            homeowner_potentials,
            mortgage_credit_current,
            creditable_tax,
            preceding_mortgage_credits,
        },
    );
    let adoption_used = homeowner_credits.adoption;
    let preceding_homeowner_credits =
        preceding_mortgage_credits + mortgage_credit_used + adoption_used;
    let dc_homebuyer_credit =
        form_8859_phase(&pass, &mut r, Form8859In { homeowner_credits: &homeowner_credits });
    let residential_energy_credit =
        form_5695_phase(&pass, &mut r, Form5695In { homeowner_credits: &homeowner_credits });
    let child_credits = schedule_8812.limit(
        creditable_tax,
        preceding_homeowner_credits + dc_homebuyer_credit + residential_energy_credit,
    );
    let child_nonrefund = child_credits.nonrefundable;
    let actc = child_credits.additional;
    let legacy_ev_credit = form_8834_phase(
        &pass,
        &mut r,
        Form8834In {
            clean_vehicle: &clean_vehicle,
            #[cfg(test)]
            regular_liability,
            regular_creditable_tax,
            preceding_homeowner_credits,
            dc_homebuyer_credit,
            residential_energy_credit,
            child_nonrefund,
            tentative,
            amt_credit,
        },
    );
    let personal_refueling_credit = form_8911_phase(
        &pass,
        &mut r,
        Form8911In {
            #[cfg(test)]
            regular_liability,
            regular_creditable_tax,
            preceding_homeowner_credits,
            dc_homebuyer_credit,
            residential_energy_credit,
            child_nonrefund,
            legacy_ev_credit,
            tentative,
            amt_credit,
        },
    );
    let prior_credits = elderly + education_used + care_used;
    let Form3800SourcesOut {
        credit_entries,
        credit_carrybacks,
        investment,
        investment_recapture_tax,
        has_business_credit,
    } = form_3800_sources_phase(
        &pass,
        &mut r,
        Form3800SourcesIn { clean_vehicle: &mut clean_vehicle, foreign_magi },
    );
    let other_nonrefundable_before_business = bba_schedule_3_credit
        + prior_credits
        + saver
        + used_clean_vehicle_credit
        + new_clean_vehicle_credit
        + legacy_ev_credit
        + adoption_used
        + child_nonrefund
        + mortgage_credit_used
        + dc_homebuyer_credit
        + residential_energy_credit
        + personal_refueling_credit;
    let passive_capacity = form_8582_credit_capacity_phase(
        &pass,
        Form8582CreditCapacityIn {
            regular_tax_worksheet: &regular_tax_worksheet,
            kiddie_credit_tax: &kiddie_credit_tax,
            averaged_tax: &averaged_tax,
            rental: &rental,
        },
    );
    // 2026 draft i3800 line 7: "Don't include any recapture of an education credit".
    #[cfg(test)]
    record_credit_base(
        "Form 3800 capacity",
        (regular_liability - education_recapture_tax) + education_recapture_tax,
    );
    let research_numerators = match research_numerators_phase(
        &pass,
        &t,
        ResearchNumeratorsIn {
            rental: &rental,
            research_business_income,
            tips_deducted,
            business_half_se: &business_half_se,
            health: &health,
            sep: &sep,
            source_claims,
            property_character: &property_character,
            excess_business_loss,
            capital_in,
            capital,
            short,
            long,
            additional,
            business_tips,
            qualified_tips,
            qualified_business_tips,
            qualified_overtime,
            charity_claims,
            use_itemized,
            charity: &charity,
            nol_run,
            cash_standard,
            investment_interest_deduction,
            itemized,
            itemized_reduction,
            research_qbi_deductions,
            hsa,
            msa_deduction,
            ira: &ira,
            loan,
            alimony_paid,
            agi,
            qbi_ded,
            taxable,
            credit_entries: &credit_entries,
        },
    ) {
        Ok(value) => value,
        Err(issue) => {
            issue.report(&mut r);
            return r;
        }
    };
    let remaining_tax = creditable_tax
        - prior_credits
        - saver
        - used_clean_vehicle_credit
        - new_clean_vehicle_credit
        - legacy_ev_credit;
    line(
        &mut r,
        "adoption_nonrefundable_credit",
        adoption_used,
        &["credits.adoptions", "credits.adoption_carry", "child_credit"],
        "https://www.irs.gov/instructions/i8839",
        msg!("EXPLAIN_ADOPTION_NONREFUNDABLE_CREDIT"),
    );
    line(
        &mut r,
        "adoption_refundable_credit",
        adoption.refundable,
        &["credits.adoptions", "agi"],
        ANNUAL,
        msg!("EXPLAIN_ADOPTION_REFUNDABLE_CREDIT"),
    );
    nonqualified_annuity_output_phase(&mut r, &t);
    // Remaining property-specific gaps must be explicit. Completed gain
    // treatments need no blanket rejection merely for having property records.
    for allocation in [property_loss_deferred, amt_property_loss_deferred] {
        if let Err((key, _message)) = allocation {
            crate::issue_localized!(
                &mut r,
                &crate::IssueContext::Section1231,
                "Property excess business loss",
                key,
                _message,
            );
        }
    }
    let Some(NiitOut {
        prior_niit_difference,
        estate_niit_prior_sources_known,
        self_rental_current_components_zero,
        property_niit,
        niit_casualty_loss,
        passive_business_niit,
        income_tax_expense,
        niit_itemized,
        niit_deduction_components,
        injured_niit_itemized,
        estate_niit_excess_deduction,
        nii,
        niit,
    }) = selected179!(niit_phase(
        &pass,
        &mut r,
        &t,
        NiitIn {
            property_character: &property_character,
            property_sources_known_absent,
            rental: &rental,
            source_claims,
            niit_capital_net,
            niit_capital,
            resident_niit_prior,
            separate_niit,
            resident,
            direct_property_sources: &direct_property_sources,
            niit_magi,
            niit_threshold,
            claimed_casualty,
            interest,
            dividends,
            property_net,
            loan_gain,
            hobby,
            nonresident_amounts,
            use_itemized,
            salt,
            salt_eligible: &salt_eligible,
            agi,
            savings_penalty: niit_savings_penalty,
            foreign_tax_deduction: &foreign_tax_deduction,
            foreign_tax_deductible,
            foreign_investment_tax,
            investment_property_taxes,
            investment_interest_deduction,
            deduction,
            stock_loss,
            itemized_reduction,
            gst_tax,
            bond_premium,
            debt_instrument_loss,
            impairment_work,
            educator_itemized,
            annuity_loss,
            gambling_loss,
            medical_deduction,
            mortgage_deductible_interest,
            charity_deduction,
            casualty_deduction,
            schedule_a_state_local: &schedule_a_state_local,
            rental_personal: &rental_personal,
            office_schedule_a: &office_schedule_a,
            estate_state_income_tax,
            estate_state_sales_tax,
            investment_unavailable,
        },
    ))
    else {
        return r;
    };
    niit_output_phase(
        &pass,
        &mut r,
        &t,
        NiitReportingIn {
            use_itemized,
            deduction,
            investment_property_taxes,
            investment_unavailable,
            investment_interest_deduction,
            income_tax_expense,
            salt,
            income_sales,
            foreign_investment_tax,
            investment_facts,
            property_niit: &property_niit,
            interest,
            dividends,
            nonresident_amounts,
            business,
            rental: &rental,
            royalties,
            passive_business_niit,
            capital: niit_capital,
            property_ordinary,
            loan_gain,
            stock_loss,
            property_net,
            niit_casualty_loss,
            hobby,
            niit_itemized,
            niit_deduction_components,
            savings_penalty: niit_savings_penalty,
            estate_niit_excess_deduction,
            niit_magi,
            niit_threshold,
            nii,
            niit,
        },
    );
    if allocated_entries.iter().any(|e| matches!(e.income, FederalIncome::Scorp { .. }))
        && r.issues.iter().all(|i| i.context == crate::IssueContext::ScorpAlternativeMinimum)
    {
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("K-1 earned-income-credit worksheet: investment income ${}. Nonpassive ordinary K-1 business income and losses are excluded from this investment-income test; other return eligibility checks remain separate.", schedule_eic.investment),
            Message::new("NOTE_K1_EIC_INVESTMENT_INCOME", alloc::vec![MessageArg::usd_dollars(schedule_eic.investment)]),
        ));
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("K-1 investment-tax worksheet: net investment income ${nii}; net investment income tax ${niit}. Other return calculations and missing-source checks remain separate."),
            Message::new("NOTE_K1_INVESTMENT_TAX", alloc::vec![MessageArg::usd_dollars(nii), MessageArg::usd_dollars(niit)]),
        ));
    }
    let early_distribution_taxes =
        crate::retirement_return::early_tax(d, &mut r, &basis, &qcd, agi);
    let early_tax = dollars(early_distribution_taxes.total_cents);
    line(
        &mut r,
        "early_distribution_tax",
        early_tax,
        &["income.retirement"],
        "https://www.irs.gov/instructions/i5329",
        msg!("EXPLAIN_EARLY_DISTRIBUTION_TAX"),
    );
    let rmd_taxes = crate::rmd::calculate(d, &mut r);
    let rmd_tax = rmd_taxes.total;
    let household_tax = crate::household_employment::calculate(d, &mut r);
    let mortgage_recapture =
        crate::mortgage_recapture::calculate_all(&c.mortgage_subsidies, d, agi, t.exempt, &mut r);
    let mortgage_recapture_tax = mortgage_recapture.joint;
    // Form 4255 column (r): the investment credit recapture is a tax outside
    // the Form 3800 limits (section 50(a)(7)(C)), Schedule 2 line 13a.
    // Box 20 code AI (unit B9-1): interest and the additional tax on
    // deferred compensation, Schedule 2 line 13h in 2026.
    let deferred_tax = dollars(t.k1_deferred_tax);
    if deferred_tax != 0 {
        form_line(
            &mut r,
            FormLine::new(TaxForm::Schedule2, "13h"),
            "k1_deferred_compensation_tax",
            deferred_tax,
            &["income.partnership"],
            "https://www.irs.gov/instructions/i1065sk1",
            msg!("EXPLAIN_K1_DEFERRED_COMPENSATION_TAX"),
        );
    }
    if investment_recapture_tax != 0 {
        form_line(
            &mut r,
            FormLine::new(TaxForm::Schedule2, "13a"),
            "investment_credit_recapture_tax",
            investment_recapture_tax,
            &["income.business.assets.investment_credit", "income.rental.buildings.rehabilitation"],
            "https://www.irs.gov/pub/irs-dft/i4255--dft.pdf",
            msg!("EXPLAIN_SCHEDULE2_LINE_13A_INVESTMENT_CREDIT"),
        );
    }
    // Reuse the existing Chapter 1 classification for both the negative-8978
    // worksheet and the section 1341 comparison below. These are the native
    // owners' rounded return amounts, each included once. Education recapture
    // is already in line 18; payroll, SE, NIIT, household and excise are outside
    // this pool. Section 409A(a)(1)(B) treats both its interest and addition as
    // an increase in Chapter 1 tax (the K-1 owner supplies that combined amount).
    let chapter_one_part_ii_tax = early_tax
        + hsa_chapter_one_tax
        + medicare_msa_tax
        + archer_msa_tax
        + account_tax
        + trump_earnings_tax
        + mortgage_recapture_tax
        + investment_recapture_tax
        + deferred_tax;
    let bba_schedule_2_adjustment = phase::cap(bba_after_line_18, [chapter_one_part_ii_tax]);
    let bba_unusable_decrease = bba_after_line_18 - bba_schedule_2_adjustment;
    // Current native owners run once before the credit whose limit needs their
    // pre-business liabilities. Later GBC-dependent MTC/bond credits are excluded.
    let adjustments = personal_rent_expenses
        + housing_deduction
        + half_se
        + sep_deduction
        + health.deduction
        + educator
        + employee_deduction
        + dollars(hsa)
        + msa_deduction
        + ira.deduction
        + pension.deduction
        + loan
        + savings_penalty
        + alimony_paid
        + moving_deduction
        + jury_remitted
        + (olympic_prize - olympic_taxable)
        + estate_67e;
    let additional_taxes = se_tax
        + tip_payroll
        + employee_wage_tax
        + uncollected_payroll
        + golden_parachute_tax
        + household_tax
        + additional_medicare
        + niit
        + early_tax
        + rmd_tax
        + hsa_tax
        + medicare_msa_tax
        + archer_msa_tax
        + archer_excess_tax
        + account_tax
        + able_excess_tax
        + coverdell_excess_tax
        + trump_excess_tax
        + trump_earnings_tax
        + ira_excess_tax
        + mortgage_recapture_tax
        + investment_recapture_tax
        + deferred_tax
        - bba_schedule_2_adjustment;
    let Form8379BusinessOut {
        schedule_c,
        schedule_f,
        total_income,
        business_form_owners,
        statutory_form_owners,
        statutory_sources_known,
    } = form_8379_business_phase(
        &pass,
        &r,
        &mut t,
        Form8379BusinessIn {
            rental: &rental,
            business,
            agi,
            adjustments,
            excess_business_loss,
            estate_niit_excess_deduction,
            pure_loan_gain,
        },
    );
    let injured_boundaries = injured_boundaries_phase(
        &pass,
        &t,
        InjuredBoundariesIn {
            rental: &rental,
            moving: &moving,
            employee_recapture,
            alimony_recaptured_owners,
            scholarship_inclusions,
            royalty_interest,
            clergy_housing_excess,
            household_wages,
            medicaid_waiver_wages,
            strike_benefits,
            tax_refund,
            alaska,
            prizes,
            not_for_profit,
            olympic_prize,
            medicaid_waiver_excluded,
            inmate_pay,
            deferred_pension,
            digital,
            other_8z,
            tip_income,
            employee_wage_income,
            employee_wages_not_covered,
            disability_pension,
            iso_unreported,
            alimony_recaptured,
            alimony,
            gambling,
            personal_debt,
            hobby,
            scholarship,
            taxable_grants,
            family_leave,
            taxed_relief,
            health_assistance,
            trump_death,
            royalties,
            pure_loan_gain,
            clergy_excess,
            moving_excess,
            deferral_excess,
        },
    );
    // Retain the same native line26 collection once, including each actual
    // beneficiary. Ordinary line26 and Form8379 reuse this source.
    let (estate_estimated_cents, estate_estimated_cents_by_owner) = allocated_entries.iter().fold(
        (
            crate::kernel::ratio::exact::whole(0),
            core::array::from_fn(|_| crate::kernel::ratio::exact::whole(0)),
        ),
        |(total, mut owners), entry| {
            if let FederalIncome::EstateTrustPortfolio { data } = &entry.income {
                let data = &crate::FederalEstateTrustPortfolioK1::from(data);
                let amount = crate::kernel::ratio::exact::whole(data.allocated_estimated_tax);
                owners[usize::from(entry.owner == crate::DocumentOwner::Spouses)] += &amount;
                (total + amount, owners)
            } else {
                (total, owners)
            }
        },
    );
    let Form4136FuelOut { fuel_credit_cents, fuel_credit, injured_fuel_credit_sources } =
        form_4136_fuel_phase(&pass);
    let injured_income_categories = injured_income_categories_phase(
        &pass,
        &r,
        &t,
        InjuredIncomeCategoriesIn {
            injured_boundaries: &injured_boundaries,
            rental: &rental,
            business_exclusions: &business_exclusions,
            health: &health,
            employee: &employee,
            form_2441: &form_2441,
            property_niit: &property_niit,
            estate_estimated_cents_by_owner: &estate_estimated_cents_by_owner,
            sep: &sep,
            injured_adoption,
            additional_medicare_sources,
            foreign_credit_sources,
            injured_medicare_withholding,
            injured_fuel_credit_sources,
            injured_niit_itemized,
            injured_property_sources,
            saver_eligible_sources,
            employee_wage_taxes,
            tip_payroll_owners,
            half,
            late_investment_sources,
            statutory_form_owners,
            business_form_owners,
            investment_sources_known,
            employee_payroll_complete,
            self_rental_current_components_zero,
            estate_niit_prior_sources_known,
            statutory_sources_known,
            hobby,
            passive_business_niit,
            tip_payroll,
            uncollected_payroll,
            golden_parachute_tax,
            saver,
            care_benefits,
            prior_niit_difference,
            personal_rent,
            w2_wages,
            schedule_c,
            schedule_f,
            business,
            half_se,
            sep_deduction,
            business_excluded,
            property_business_capital,
            property_ordinary,
            k1_business_ordinary,
            excess_business_loss,
            excess_business_loss_operand,
        },
    );
    let injured_facts = crate::injured_spouse::JointFacts {
        income_categories: alloc::boxed::Box::new(injured_income_categories),
        w2_wages: [0, 1].map(|i| {
            t.wages[i]
                - t.employee_tips[i].income
                - t.employee_wages[i]
                - t.employee_wages_not_covered[i]
        }),
        k1_nii: &t.injured_k1_nii,
        nii_rental: &rental,
        child: &t.child,
        savings_bond_exclusion: crate::savings_bonds::record(&d.income).map_or(
            Some([0; 2]),
            |(entry, _)| {
                // The source rule verifies the actual linked interest ID and
                // its owner, not the student's education beneficiary.
                crate::savings_bonds::source_rule(&d.income).is_none().then(|| {
                    let mut owners = [0; 2];
                    owners[usize::from(entry.owner == crate::DocumentOwner::Spouses)] =
                        savings_bond_exclusion;
                    owners
                })
            },
        ),
        foreign: &foreign,
        foreign_business_housing: business_exclusions.each_ref().map(|v| v.housing_deduction),
        deferred_compensation_tax: crate::injured_spouse::DeferredCompensationTaxSources {
            joint_dollars: deferred_tax,
            cents: t.injured_k1_deferred_tax,
        },
        accounts: t.injured_accounts,
        investment_recapture: investment.recapture_by_owner,
        property_ordinary: injured_property_sources.map(|source| source.ordinary),
        mortgage_recapture,
        other_sources: {
            let mut sources = t.injured_other_sources.clone();
            for entry in &d.income {
                if matches!(entry.income, FederalIncome::SavingsBonds { .. }) {
                    sources.insert(
                        entry.id,
                        crate::savings_bonds::source_rule(&d.income)
                            .is_none()
                            .then(|| exact::whole(0)),
                    );
                }
            }
            for (&id, input) in &rental.personal_inputs {
                sources.insert(
                    id,
                    rental.post_pal.as_ref().map(|_| exact::whole(input.receipts_cents)),
                );
            }
            for (&id, &amount) in &rental.income_by_id {
                let value = sources.entry(id).or_insert_with(|| Some(exact::whole(0)));
                *value = value.take().map(|v| v + exact::whole(amount) * exact::whole(100));
                if !rental.inputs_complete {
                    // The final source owns ScheduleE after earlier limits;
                    //461 remains its separate native8p category below.
                    *value = None;
                }
            }
            for (id, amount) in &rental.k1_cuts.ordinary_by_id {
                if let Some(value) = sources.get_mut(id) {
                    *value = value.take().map(|v| v + exact::whole(amount.get()));
                }
            }
            sources
        },
        investment_election: crate::injured_spouse::InvestmentElectionSources {
            elected,
            capital: (((property_character.is_none()
                && direct_property_sources.is_empty()
                && rental_property_sources.is_empty())
                || (regular_property_investment_known_zero && injured_property_sources.is_some()))
                && property_investment[0] == [0; 2]
                && estate_investment_carry[0] == Some([0; 2]))
            .then(|| {
                t.injured_investment_sources.owners(
                    d,
                    [
                        dollars(t.investment_gains[0]),
                        dollars(t.investment_gains[1]),
                        pure_loan_gain,
                        k1_investment_ordinary[0],
                        stock_loss,
                    ],
                    investment_carry_max[0],
                    investment_net_capital,
                )
            })
            .flatten(),
        },
        unemployment: t.injured_unemployment,
        k1_portfolio: t.injured_k1_portfolio,
        olympic: t.injured_olympic,
        nii_substitute: t.injured_substitute,
        capital: capital_source_owners,
        eic_children: family.eic_children,
        ctc_ids: family.ctc_ids.clone(),
        odc_ids: family.odc_ids.clone(),
        earned: [0, 1].map(|i| earned[i] + optional_eic[i]),
        se,
        half_se: half,
        received_alimony_recapture: t.injured_alimony_recapture,
        employee_expenses: employee.deduction_by_owner(&employee_counted),
        ira: ira.deduction_by_owner,
        hsa: hsa_deduction_by_owner,
        pension: pension.saver.map(|v| dollars(v) * 100),
        vehicle_repayment: clean_vehicle.repayment_by_owner,
        sep: sep.owners,
        health: {
            let mut owners = [0; 2];
            for entry in &allocated_entries {
                owners[usize::from(entry.owner == crate::DocumentOwner::Spouses)] +=
                    health.by_business.get(&entry.id).copied().unwrap_or(0);
            }
            owners
        },
        educator: [0, 1].map(|i| {
            phase::cap(
                educator_net(if i == 0 { a.educator_taxpayer() } else { a.educator_spouse() }, i),
                [crate::year_constants::EDUCATOR_EXPENSE_LIMIT_DOLLARS],
            )
        }),
        ltc: &t.injured_ltc,
        nii_ltc_sources: &t.injured_ltc_nii,
        fund_credit: [Owner::Taxpayer, Owner::Spouse].map(|owner| {
            d.income
                .iter()
                .filter(|e| e.owner == crate::DocumentOwner::from(owner))
                .map(|e| match &e.income {
                    FederalIncome::UndistributedGain { data } => {
                        crate::UndistributedGain::from(data).tax_paid
                    }
                    _ => 0,
                })
                .sum()
        }),
        qbi: injured_qbi,
        care_sources: family
            .care_person_ids
            .iter()
            .filter_map(|id| {
                let person = d.dependents.iter().find(|p| p.id == *id)?;
                Some((*id, person.care_expenses()))
            })
            .collect(),
        care_shared_expenses: c.care.filer_expenses(),
        education_sources: &injured_education_sources,
        education_pools: [aotc - education_refund, llc, education_refund],
        business: t.business,
        regular_nol: nol_run.allocation.as_ref().map_or_else(Vec::new, |n| n.injured_parts(d)),
        withholding_categories: t.withholding_categories,
        retirement: t.injured_retirement,
        nii_annuity: t.injured_nonqualified_annuity,
        nii_savings_penalty: t.injured_early_penalty,
        nii_royalties: royalties,
        personal_expenses: rental.personal_expenses_cents().map(|allowed| {
            let mut owners = [0; 2];
            for (id, amount) in allowed {
                owners[usize::from(rental.personal_inputs[&id].owner == Owner::Spouse)] += amount;
            }
            owners
        }),
        medicare_withheld: [0, 1]
            .map(|i| positive(t.medicare_tax[i] - fraction(t.medicare_wages[i], 145, 10000))),
        excess_social: [0, 1].map(|i| {
            dollars(positive(
                t.social_tax[i] + t.uncollected_social[i]
                    - crate::year_constants::MAX_EMPLOYEE_SOCIAL_SECURITY_TAX_CENTS,
            ))
        }),
        extension: dollars(pay.extension),
        tips: qualified_tip_owners,
        overtime: qualified_overtime_owners,
        // Actual native PartI source operands, not just its W2 subset. Keep
        // the native scalar's separate category rounding/threshold unchanged.
        medicare_wages: [0, 1].map(|i| {
            t.medicare_wages[i] + t.employee_tips[i].medicare + t.employee_payroll_medicare[i]
        }),
        itemized: use_itemized,
    };

    let separate_ratios = injured_credit_ratios_phase(
        &pass,
        &r,
        InjuredCreditRatiosIn {
            nonqualified_annuity: t.nonqualified_annuity,
            unemployment: t.unemployment,
            injured_facts: &injured_facts,
            credit_entries: &credit_entries,
            account_distribution_taxes: &account_distribution_taxes,
            early_distribution_taxes: &early_distribution_taxes,
            care_components: &care_components,
            rmd_taxes: &rmd_taxes,
            property_niit: &property_niit,
            ira: &ira,
            health: &health,
            income_tax_expense,
            retirement_taxable,
            additional,
            use_itemized,
            care_used,
            education_used,
            qbi_ded,
            additional_medicare,
            niit,
            nii,
            jury,
            jury_remitted,
            olympic_prize,
            olympic_taxable,
            schedule_c,
            schedule_f,
            housing_deduction,
            personal_rent,
            personal_rent_expenses,
            alimony_paid,
            msa_deduction,
            savings_penalty,
            educator,
            employee_deduction,
            estate_67e,
            half_se,
            hsa,
            moving_deduction,
            sep_deduction,
            loan,
            premium_repayment,
            clean_vehicle_repayment,
            senior_ded,
            niit_itemized,
            estate_niit_excess_deduction,
            w2_wages,
            interest,
            dividends,
            social,
            capital,
            total_income,
            adjustments,
            itemized,
            itemized_reduction,
            standard,
            qualified_net_casualty,
            cash_standard,
            child_nonrefund,
            other_nonrefundable_before_business,
            additional_taxes,
            taxable,
            qualified,
        },
    );
    let (general_business_credit, elective_payment) = form_3800_phase(
        &pass,
        &mut r,
        Form3800In {
            credit_entries,
            credit_carrybacks: &credit_carrybacks,
            research_numerators,
            taxable,
            separate_ratios,
            regular_liability,
            education_recapture_tax,
            amt,
            tentative,
            amt_credit,
            foreign_credit,
            other_nonrefundable_before_business,
            passive_capacity,
        },
    );
    let minimum_credit = form_8801_phase(
        &pass,
        &mut r,
        Form8801In {
            regular_liability,
            foreign_credit,
            other_nonrefundable_before_business,
            general_business_credit,
            tentative,
            amt_credit,
        },
    );
    let bond_credit = form_8912_phase(
        &pass,
        &mut r,
        Form8912In {
            remaining_tax,
            child_nonrefund,
            adoption_used,
            mortgage_credit_used,
            dc_homebuyer_credit,
            residential_energy_credit,
            personal_refueling_credit,
            minimum_credit,
            general_business_credit,
        },
    );
    crate::adoption::carryforward(&adoption, &c.adoption_carry, adoption_used, &mut r);
    let Form1040LiabilityOut {
        total,
        installments,
        installment_first,
        installment_offset,
        current_tax,
        refundable,
    } = form_1040_liability_phase(
        &pass,
        &mut r,
        Form1040LiabilityIn {
            remaining_tax,
            child_nonrefund,
            adoption_used,
            mortgage_credit_used,
            dc_homebuyer_credit,
            residential_energy_credit,
            personal_refueling_credit,
            general_business_credit,
            minimum_credit,
            bond_credit,
            se_tax,
            tip_payroll,
            employee_wage_tax,
            uncollected_payroll,
            golden_parachute_tax,
            household_tax,
            additional_medicare,
            niit,
            early_tax,
            rmd_tax,
            hsa_tax,
            medicare_msa_tax,
            archer_msa_tax,
            archer_excess_tax,
            account_tax,
            able_excess_tax,
            coverdell_excess_tax,
            trump_excess_tax,
            trump_earnings_tax,
            ira_excess_tax,
            mortgage_recapture_tax,
            investment_recapture_tax,
            deferred_tax,
            bba_schedule_2_adjustment,
            regular_tax,
            foreign_credit,
            premium_repayment,
            eic,
            actc,
            education_refund,
            premium_refund,
            elective_payment,
            chapter_one_part_ii_tax,
            rental: &rental,
            regular_capital_amounts,
            capital_in,
            elected,
            regular_tax_worksheet,
            limited_credits,
            credit_potentials,
            foreign_basis,
            averaged_tax,
            parent_rate_required,
            has_business_credit,
            bba_audit: &bba_audit,
            adoption: &adoption,
            use_itemized,
        },
    );
    let fund_credit = dollars(t.fund_tax_paid);
    // Schedule 3 line 13a: Form 2439 box 2.
    form_line(
        &mut r,
        FormLine::new(TaxForm::Schedule3, "13a"),
        "undistributed_gain_tax_credit",
        fund_credit,
        &["income.undistributed_gain.tax_paid"],
        "https://www.irs.gov/pub/irs-pdf/f2439.pdf",
        msg!("EXPLAIN_UNDISTRIBUTED_GAIN_TAX_CREDIT"),
    );
    // Form 6251 line 2h (2026 draft): the section 1202 exclusion preference,
    // part of alternative minimum taxable income.
    form_line(
        &mut r,
        FormLine::new(TaxForm::Form6251, "2h"),
        "qsbs_amt_preference",
        dollars(t.qsbs_amt_preference),
        &["income.qualified_small_business_sale"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_QSBS_AMT_PREFERENCE"),
    );
    form_4136_output_phase(&pass, &mut r, fuel_credit_cents);
    // Native projection is the only bound: exact/source sums and independently
    // rounded printed categories stay distinct until their actual i64 output.
    macro_rules! payment_native {
        ($value:expr) => {
            match $value {
                Some(value) => value,
                None => {
                    crate::underpayment::amount_unavailable(&mut r);
                    return r;
                }
            }
        };
    }
    let PaymentsOut {
        withholding_25c,
        withholding_25d,
        estimated_dollars,
        estimated_cents,
        payments,
        payment_cents,
    } = payment_native!(payments_phase(
        &pass,
        &r,
        PaymentsIn {
            estate_estimated_cents: &estate_estimated_cents,
            fund_credit,
            fuel_credit,
            excess_social
        },
    ));
    credit_line_reporting_phase(
        &pass,
        &mut r,
        CreditLineReportingIn {
            education_used,
            form_2441_context: &form_2441_context,
            creditable_tax,
            care_used,
            elderly,
            saver,
            schedule_8812: &schedule_8812,
            child_credits: &child_credits,
            schedule_eic: &schedule_eic,
            education_refund,
            additional_medicare,
            additional_medicare_self_employment,
            additional_medicare_wages,
            niit,
            payments,
            total,
        },
    );
    if nol_carryovers_phase(
        &pass,
        &mut r,
        NolCarryoversIn {
            amt_nol,
            amt_excess_business_loss,
            amt,
            loss_carryovers,
            nol_run,
            prior_atnol: &prior_atnol,
            nol,
            excess_business_loss,
            farm_route,
            ptp_nol_predebt_carry,
            has_atnol,
        },
    )
    .is_break()
    {
        return r;
    }
    k1_source_consistency_phase(&pass, &mut r);
    let Schedule3CreditTotalsOut { other_credits, credits } =
        schedule_3_credit_totals_phase(Schedule3CreditTotalsIn {
            elderly,
            education_used,
            care_used,
            saver,
            adoption_used,
            minimum_credit,
            mortgage_credit_used,
            dc_homebuyer_credit,
            residential_energy_credit,
            personal_refueling_credit,
            general_business_credit,
            used_clean_vehicle_credit,
            new_clean_vehicle_credit,
            legacy_ev_credit,
            foreign_credit,
            bba_schedule_3_credit,
            bond_credit,
            child_nonrefund,
        });
    let Form2210Out { line_2_other, farming_return_year, penalty, penalty_explanation } =
        selected179!(form_2210_phase(
            &pass,
            &mut r,
            Form2210In {
                farmer_capital_correction: t.farmer_capital_correction,
                capital_gross: t.capital_gross,
                ordinary_gain_gross: t.ordinary_gain_gross,
                unemployment: t.unemployment,
                additional_medicare,
                niit,
                early_tax,
                account_tax,
                hsa_chapter_one_tax,
                archer_msa_tax,
                medicare_msa_tax,
                investment_recapture_tax,
                trump_earnings_tax,
                bba_schedule_2_adjustment,
                wages,
                adoption_wages,
                benefit_wages,
                foreign_wages,
                interest,
                dividends,
                social,
                iso_unreported,
                hsa_income,
                alimony,
                gambling,
                jury,
                scholarship,
                prizes,
                other_line_8,
                olympic_taxable,
                taxable_grants,
                family_leave,
                casualty_living_insurance,
                personal_debt,
                care_benefits,
                medicare_msa,
                archer_total_income,
                able_income,
                education_income,
                trump_death,
                tax_refund,
                rental_gross,
                royalty_gross,
                personal_rent,
                hobby,
                child_nonrefund,
                actc,
                income_tax,
                credits,
                se_tax,
                household_tax,
                refundable,
                fuel_credit,
                withholding_25d,
                excess_social,
                fund_credit,
                agi,
                taxable,
                standard,
                regular_tax,
                child_tax,
                education_recapture_tax,
                excluded,
                regular_tax_pref,
                amt,
                bba_additional_tax,
                lump_sum_tax,
                premium_repayment,
                clean_vehicle_repayment,
                retirement_taxable,
                se,
                use_itemized,
                parent_rate_required,
                rental: &rental,
                estate_estimated_cents: &estate_estimated_cents,
                installments: &installments,
                source_claims,
            },
        ));
    let SettlementOut {
        refund_net,
        refund_cents,
        balance_cents,
        payment_net,
        final_refund_cents,
        final_balance_cents,
        credit_pending,
        reported_credit_cents,
        cash_refund_cents,
    } = payment_native!(settlement_phase(
        &pass,
        &mut r,
        SettlementIn { payments, refundable, current_tax, penalty },
    ));
    form_1040_es_phase(
        &pass,
        &mut r,
        Form1040EsIn {
            reported_credit_cents,
            agi,
            deduction,
            qbi_ded,
            senior_ded,
            additional,
            regular_tax_pref,
            regular_collectible,
            regular_unrecaptured,
            parent_rate_required,
            excluded,
            regular_tax,
            unaveraged_regular_tax,
            amt,
            bba_additional_tax,
            premium_repayment,
            clean_vehicle_repayment,
            credits,
            residential_energy_credit,
            personal_refueling_credit,
            used_clean_vehicle_credit,
            new_clean_vehicle_credit,
            legacy_ev_credit,
            se_tax,
            line_2_other,
            education_recapture_tax,
            rmd_tax,
            hsa_tax,
            hsa_chapter_one_tax,
            archer_excess_tax,
            able_excess_tax,
            coverdell_excess_tax,
            trump_excess_tax,
            ira_excess_tax,
            household_tax,
            refundable,
            fuel_credit,
            income_tax,
            withholding_25d,
            farming_return_year,
        },
    );
    r.payroll_known_rows = (!r.issues.is_empty()
        && r.issues.iter().all(|issue| payroll_source_issue(&issue.display.key)))
    .then_some(crate::federal_model::PayrollKnownRows {
        // NOL limit/absorption passes carry provisional deductions. Only the
        // final ordinary pass can certify the resulting AGI.
        agi: payroll_agi_known && !nol_run.modify_income && !nol_run.exclude_qbi,
        social: t.employee_rrta_compensation_unknown.map(|unknown| !unknown),
        coverage_parts: [0, 1].map(|owner| {
            [
                !t.employee_rrta_compensation_unknown[owner]
                    && employee_coverage_possible[owner][0] == employee_coverage_taxes[owner][0],
                employee_coverage_possible[owner][1] == employee_coverage_taxes[owner][1],
            ]
        }),
        form8919: t.employee_8919.map(|amount| amount > 0),
        medicare_wages: t.employee_unknown_medicare.iter().all(|amount| *amount == 0),
        railroad_compensation: t.employee_rrta_compensation_unknown.iter().all(|unknown| !unknown),
        railroad_withholding: t.employee_rrta_withholding_unknown.iter().all(|unknown| !unknown),
        additional_parts: [
            possible_additional_parts[0] == known_additional_parts[0],
            t.employee_rrta_compensation_unknown.iter().all(|unknown| !unknown),
            payroll_se_net_known && possible_additional_parts[2] == known_additional_parts[2],
        ],
        se_net: payroll_se_net_known,
    });
    if r.issues.is_empty() {
        let additional_deductions = form_1040_rules::successful_result_phase(
            &pass,
            &mut r,
            form_1040_rules::SuccessfulResultIn {
                property_character: &property_character,
                amt_property_character: &amt_property_character,
                property_loss_deferred,
                amt_property_loss_deferred,
                installments,
                additional,
                se,
                se_profit,
                social_wages: t.social_wages,
                use_itemized,
                bba_audit: &bba_audit,
                total,
                refundable,
                payment_cents,
                refund_cents,
                balance_cents,
                senior_ded,
                itemized,
                itemized_reduction,
                bba_credit,
                bba_schedule_3_credit,
                bba_after_line_18,
                bba_schedule_2_adjustment,
                bba_unusable_decrease,
                bba_additional_tax,
            },
        );
        let additional_income = business
            + iso_unreported
            + dollars(t.unemployment)
            + hsa_income
            + alimony
            + gambling
            + jury
            + scholarship
            + prizes
            + other_line_8
            + olympic_prize
            + taxable_grants
            + family_leave
            + casualty_living_insurance
            + personal_debt
            + care_benefits
            + medicare_msa
            + archer_total_income
            + able_income
            + education_income
            + trump_death
            + tax_refund
            + royalties
            + personal_rent
            + loan_gain
            + property_ordinary
            - stock_loss
            - nol_run.deduction
            + hobby
            + rental.income
            - income_excluded
            + excess_business_loss
            + t.child.other;
        // Schedule 1: the lines this return computes, each only when it is
        // not zero; line 10 is Form 1040 line 8 and line 26 is line 10.
        if schedule_e_rules::schedule_e_output_phase(
            &pass,
            &mut r,
            schedule_e_rules::ScheduleEOutputIn {
                royalty_regular: &royalty_regular,
                rental: &rental,
                royalties,
                estate_royalties: t.estate_royalties,
                k1_royalty_losses: t.k1_royalty_losses,
                estate_royalty_columns: t.estate_royalty_columns,
            },
        )
        .is_none()
        {
            return r;
        }
        if let Err(issue) = schedule_1_rules::schedule_1_output_phase(
            &pass,
            &mut r,
            schedule_1_rules::Schedule1OutputIn {
                taxable_grants,
                family_leave,
                casualty_living_insurance,
                education_income,
                trump_death,
                other_8z,
                medicaid_waiver_excluded,
                gambling,
                personal_debt,
                income_excluded,
                care_benefits,
                medicare_msa,
                archer_total_income,
                hsa_income,
                jury,
                prizes,
                hobby,
                iso_unreported,
                personal_rent,
                excess_business_loss,
                able_income,
                scholarship,
                alaska,
                olympic_prize,
                deferred_pension,
                inmate_pay,
                digital,
                not_for_profit,
                jury_remitted,
                olympic_taxable,
                personal_rent_expenses,
                housing_deduction,
                estate_67e,
                property_ordinary,
                loan_gain,
                stock_loss,
                tax_refund,
                alimony,
                msa_deduction,
                schedule_c,
                royalties,
                schedule_f,
                additional_income,
                educator,
                hsa,
                half_se,
                sep_deduction,
                savings_penalty,
                moving_deduction,
                alimony_paid,
                loan,
                adjustments,
                child_other: t.child.other,
                unemployment: t.unemployment,
                nol_deduction: nol_run.deduction,
                rental_income: rental.income,
                health_deduction: health.deduction,
                ira_deduction: ira.deduction,
            },
        ) {
            issue.report(&mut r);
            return r;
        }

        let FinalReturnScheduleProjectionOut {
            other_payments,
            refundable_subtotal,
            payment_total_dollars,
        } = payment_native!(final_return_schedule_projection_phase(
            &pass,
            &mut r,
            FinalReturnScheduleProjectionIn {
                excess_social,
                premium_refund,
                fund_credit,
                fuel_credit,
                installment_offset,
                elective_payment,
                eic,
                actc,
                education_refund,
                adoption: &adoption,
                payments,
                refundable,
                premium_repayment,
                clean_vehicle: &clean_vehicle,
                clean_vehicle_repayment,
                amt,
                early_tax,
                account_tax,
                trump_earnings_tax,
                mortgage_recapture_tax,
                hsa_distribution_tax,
                hsa_testing_tax,
                archer_msa_tax,
                medicare_msa_tax,
                golden_parachute_tax,
                investment_recapture_tax,
                deferred_tax,
                hsa_chapter_one_tax,
                bba_schedule_2_adjustment,
                se_tax,
                niit,
                additional_medicare_self_employment,
                tip_payroll,
                form_8919_tax,
                household_tax,
                uncollected_payroll,
                additional_medicare_wages,
                rmd_tax,
                hsa_excess_tax,
                archer_excess_tax,
                able_excess_tax,
                coverdell_excess_tax,
                trump_excess_tax,
                ira_excess_tax,
                employee_wage_tax,
                additional_taxes,
                foreign_credit,
                care_used,
                education_used,
                saver,
                residential_energy_credit,
                general_business_credit,
                minimum_credit,
                adoption_used,
                elderly,
                new_clean_vehicle_credit,
                mortgage_credit_used,
                dc_homebuyer_credit,
                legacy_ev_credit,
                personal_refueling_credit,
                bond_credit,
                used_clean_vehicle_credit,
                bba_schedule_3_credit,
                other_credits,
                other_employee_payroll_tax,
            },
        ));

        let return_rows = form_1040_return_rows_phase(Form1040ReturnRowsIn {
            actc,
            additional_deductions,
            additional_income,
            additional_taxes,
            adjustments,
            adoption: &adoption,
            adoption_wages,
            agi,
            amt,
            bba_additional_tax,
            benefit_wages,
            capital,
            cash_standard,
            child_nonrefund,
            child_tax,
            clean_vehicle_repayment,
            clergy_excess,
            combat_pay_election,
            credits,
            deduction,
            deferral_excess,
            disability_pension,
            dividends,
            education_recapture_tax,
            education_refund,
            eic,
            employee_wage_income,
            employee_wages_not_covered,
            estimated_dollars,
            exempt,
            foreign_wages,
            household_wages,
            income_tax,
            installment_first,
            interest,
            itemized,
            itemized_reduction,
            lump_sum_tax,
            medicaid_waiver_wages,
            moving_excess,
            other_credits,
            other_payments,
            payment_total_dollars,
            penalty,
            premium_repayment,
            qbi_ded,
            qualified,
            qualified_net_casualty,
            refund_net,
            refundable_subtotal,
            regular_tax,
            retirement_gross: t.retirement_gross,
            retirement_taxable,
            social,
            social_gross: t.social,
            standard,
            strike_benefits,
            taxable,
            tip_income,
            total,
            total_income,
            use_itemized,
            w2_wages,
            wages,
            withholding_25a,
            withholding_25b,
            withholding_25c,
            withholding_25d,
        });
        if form_1040_rules::return_rows_phase(
            &pass,
            &mut r,
            form_1040_rules::ReturnRowsIn {
                return_rows,
                penalty,
                penalty_explanation,
                credit_pending,
                refund_net,
                payment_net,
                cash_refund_cents,
                reported_credit_cents,
                final_balance_cents,
                refundable,
                total,
                final_refund_cents,
            },
            &injured_facts,
        )
        .is_none()
        {
            return r;
        }
    } else {
        // Payment timing cannot invalidate the established annual line 26.
        // Final settlement and a precise penalty still need the missing timing.
        let known_estimates = context.financial_source_scope_complete && estimated_timing_only(&r);
        // Form 8615 line 1 remains available to its owning walk.
        withhold_required_source_result(&mut r);
        if known_estimates {
            r.return_lines.push(FederalAmount {
                label: msg!("RETURN_1040_26").encode(),
                amount: estimated_cents,
                explanation: msg!("EXPLAIN_1040_26").encode(),
                form_line: Some(FormLine::new(TaxForm::Form1040, "26")),
                money: Some(estimated_cents),
            });
        }
    }
    if r.state == ResultState::EstimatedForSupportedScope {
        selected179!(crate::community::partner_se_lines_with_section179_with_claims(
            d,
            &rental,
            (business_se_tax[0] * 100, &business_half_se_cents),
            t.community_counterpart_payroll,
            &community_qbi_loss_sources,
            selected_regular,
            &mut r,
            Some(&office_context),
            source_claims
        ));
    }
    if d.setup.as_ref().is_some_and(crate::community::applies) {
        let report = crate::community::source_projection(d, &rental, false, source_claims)
            .and_then(|projection| projection.report(d, &mut r));
        if let Err(issue) = report {
            issue.report(&mut r);
            withhold_required_source_result(&mut r);
        }
    }
    r
}

// A married person filing separately: the section 1202 issuer limit is halved.
pub(crate) fn separate_return(d: &FederalDraft) -> bool {
    d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Separate)
}

fn checked_collect_sale(
    t: &mut Totals,
    v: &crate::FederalSale,
    e: &FederalIncomeEntry,
    r: &mut FederalResult,
    context: &crate::IssueContext,
    #[cfg(test)] _source: &str,
) -> Option<()> {
    // The explicit conservative basis answer counts as zero and carries its existing notice.
    if v.basis.is_none() {
        note_once_scoped(
            r,
            Message::new(
                "WARN_SALE_BASIS_ASSUMED_ZERO",
                alloc::vec![crate::interview::income_display_arg(e)],
            ),
            context.clone(),
        );
    }
    let gain = v.gain();
    let amt_gain = v.amt_gain();
    let term = usize::from(v.term == GainTerm::Long);
    let amt_term = usize::from(v.amt_term() == GainTerm::Long);
    for index in 0..2 {
        let delta = if index == amt_term { i128::from(amt_gain) } else { 0 }
            - if index == term { i128::from(gain) } else { 0 };
        checked_native_add(&mut t.amt_sale_adjustment[index], delta)?;
    }
    checked_native_add(&mut t.amt_personal_sale_loss, i128::from(v.amt_personal_loss()))?;
    if v.collectible {
        let delta = if v.amt_term() == GainTerm::Long { i128::from(amt_gain) } else { 0 }
            - if v.term == GainTerm::Long { i128::from(gain) } else { 0 };
        checked_native_add(&mut t.amt_collectible_adjustment, delta)?;
    }
    // An option you bought or wrote is an investment like any other (section 1234).
    if !matches!(v.use_kind, SaleUse::Personal) {
        checked_native_add(&mut t.investment_gains[term], i128::from(gain))?;
        for index in 0..2 {
            let delta = if index == amt_term { i128::from(amt_gain) } else { 0 }
                - if index == term { i128::from(gain) } else { 0 };
            checked_native_add(&mut t.amt_investment_gain_adjustment[index], delta)?;
        }
    }
    checked_native_add(&mut t.withholding_1099, i128::from(v.withholding))?;
    checked_native_add(&mut t.personal_sale_loss, i128::from(v.personal_loss()))?;
    if v.term == GainTerm::Short {
        t.try_add_capital(gain, false)?;
    } else {
        t.try_add_capital(gain, true)?;
        if v.collectible {
            checked_native_add(&mut t.collectible_long, i128::from(gain))?;
        }
    }
    // Accrued market discount (code D) is interest, not capital gain (section 1276): Schedule B
    // line 1, Form 1040 line 2b, investment income and net investment income.
    let discount = v.market_discount_interest();
    if discount != 0 {
        checked_native_add(&mut t.interest, i128::from(discount))?;
        line_cents(
            r,
            &alloc::format!("sale_{}_market_discount", e.id),
            discount,
            &["income.sales"],
            "https://www.irs.gov/instructions/i8949",
            msg!("EXPLAIN_SALE_N_MARKET_DISCOUNT"),
        );
    }
    // A summary lot (Exception 1) is Schedule D line 1a or 8a; any other lot is a Form 8949
    // row in its box A-L and its Schedule D line 1b/2/3 or 8b/9/10.
    let summary = v.summary();
    let row = crate::capital::Form8949Row {
        entry: e.id,
        row: t.next_row(e.id),
        line: crate::capital::sale_line(v.term, v.basis_reporting, summary),
        box_letter: v.box_letter(),
        label: crate::interview::income_display_arg(e),
        proceeds: v.proceeds,
        basis: v.column_e(),
        adjustment: v.column_g(),
        codes: v.codes(),
        summary,
    };
    row.emit(r);
    t.try_add_row(&row)?;
    t.capital_endpoints_fit()?;
    Some(())
}

pub(crate) use crate::federal_model::{validate_sale, validate_sale_with_mode};

// Per-form modules split out by tools/split-hubs; the tool regenerates this block.
#[path = "rules/deductions/charity_rules.rs"]
mod charity_rules;
#[path = "rules/family/dependents_rules.rs"]
mod dependents_rules;
#[path = "rules/deductions/educator_rules.rs"]
mod educator_rules;
#[path = "rules/family/eic_rules.rs"]
mod eic_rules;
#[path = "rules/passthrough/estate_trust_k1_rules.rs"]
mod estate_trust_k1_rules;
#[path = "rules/payments/form_1040_es_rules.rs"]
mod form_1040_es_rules;
#[path = "rules/federal/form_1040_rules.rs"]
mod form_1040_rules;
pub(crate) use form_1040_rules::employee_wages_into;
#[path = "rules/deductions/form_1098_e_rules.rs"]
mod form_1098_e_rules;
#[path = "rules/investment/form_1099_div_rules.rs"]
mod form_1099_div_rules;
#[path = "rules/payments/form_1099_g_rules.rs"]
mod form_1099_g_rules;
#[path = "rules/investment/form_1099_int_rules.rs"]
mod form_1099_int_rules;
#[path = "rules/farm/form_1099_patr_rules.rs"]
mod form_1099_patr_rules;
#[path = "rules/retirement/form_1099_r_rules.rs"]
mod form_1099_r_rules;
#[path = "rules/foreign/form_1116_rules.rs"]
mod form_1116_rules;
#[path = "rules/business/form_2106_rules.rs"]
mod form_2106_rules;
#[path = "rules/payments/form_2210_rules.rs"]
mod form_2210_rules;
#[path = "rules/family/form_2441_rules.rs"]
mod form_2441_rules;
#[path = "rules/foreign/form_2555_rules.rs"]
mod form_2555_rules;
#[path = "rules/credits/form_3800_rules.rs"]
mod form_3800_rules;
#[path = "rules/credits/form_4136_rules.rs"]
mod form_4136_rules;
#[path = "rules/cost_recovery/form_4562_rules.rs"]
mod form_4562_rules;
#[path = "rules/business/form_461_rules.rs"]
mod form_461_rules;
#[path = "rules/home/form_4684_rules.rs"]
mod form_4684_rules;
#[path = "rules/investment/form_4797_rules.rs"]
mod form_4797_rules;
#[path = "rules/farm/form_4835_rules.rs"]
mod form_4835_rules;
#[path = "rules/investment/form_4952_rules.rs"]
mod form_4952_rules;
#[path = "rules/retirement/form_5329_rules.rs"]
mod form_5329_rules;
#[path = "rules/credits/form_5695_rules.rs"]
mod form_5695_rules;
#[path = "rules/federal/form_6251_rules.rs"]
mod form_6251_rules;
#[path = "rules/investment/form_6252_rules.rs"]
mod form_6252_rules;
#[path = "rules/investment/form_6781_rules.rs"]
mod form_6781_rules;
#[path = "rules/attribution/form_8379_rules.rs"]
mod form_8379_rules;
#[path = "rules/home/form_8396_rules.rs"]
mod form_8396_rules;
#[path = "rules/activity/form_8582_rules.rs"]
mod form_8582_rules;
#[path = "rules/family/form_8615_rules.rs"]
mod form_8615_rules;
#[path = "rules/credits/form_8801_rules.rs"]
mod form_8801_rules;
#[path = "rules/family/form_8814_rules.rs"]
mod form_8814_rules;
#[path = "rules/investment/form_8815_rules.rs"]
mod form_8815_rules;
#[path = "rules/credits/form_8834_rules.rs"]
mod form_8834_rules;
#[path = "rules/family/form_8839_rules.rs"]
mod form_8839_rules;
#[path = "rules/health/form_8853_rules.rs"]
mod form_8853_rules;
#[path = "rules/credits/form_8859_rules.rs"]
mod form_8859_rules;
#[path = "rules/family/form_8863_rules.rs"]
mod form_8863_rules;
#[path = "rules/credits/form_8880_rules.rs"]
mod form_8880_rules;
#[path = "rules/health/form_8889_rules.rs"]
mod form_8889_rules;
#[path = "rules/credits/form_8911_rules.rs"]
mod form_8911_rules;
#[path = "rules/credits/form_8912_rules.rs"]
mod form_8912_rules;
#[path = "rules/credits/form_8936_rules.rs"]
mod form_8936_rules;
#[path = "rules/investment/form_8949_rules.rs"]
mod form_8949_rules;
#[path = "rules/income/form_8959_rules.rs"]
mod form_8959_rules;
#[path = "rules/investment/form_8960_rules.rs"]
mod form_8960_rules;
#[path = "rules/health/form_8962_rules.rs"]
mod form_8962_rules;
#[path = "rules/passthrough/form_8978_rules.rs"]
mod form_8978_rules;
#[path = "rules/activity/form_8995_rules.rs"]
mod form_8995_rules;
#[path = "rules/home/home_sale_rules.rs"]
mod home_sale_rules;
#[path = "rules/retirement/ira_contributions_rules.rs"]
mod ira_contributions_rules;
#[path = "rules/passthrough/k1_rules.rs"]
mod k1_rules;
#[path = "rules/federal/nol_rules.rs"]
mod nol_rules;
#[path = "rules/income/other_income_rules.rs"]
mod other_income_rules;
#[path = "rules/payments/payments_rules.rs"]
mod payments_rules;
#[path = "rules/federal/schedule_1_rules.rs"]
mod schedule_1_rules;
#[path = "rules/deductions/schedule_1a_rules.rs"]
mod schedule_1a_rules;
#[path = "rules/federal/schedule_2_rules.rs"]
mod schedule_2_rules;
#[path = "rules/credits/schedule_3_rules.rs"]
mod schedule_3_rules;
#[path = "rules/credits/schedule_3a_rules.rs"]
mod schedule_3a_rules;
#[path = "rules/family/schedule_8812_rules.rs"]
mod schedule_8812_rules;
#[path = "rules/deductions/schedule_a_rules.rs"]
mod schedule_a_rules;
#[path = "rules/business/schedule_c_rules.rs"]
mod schedule_c_rules;
#[path = "rules/investment/schedule_d_rules.rs"]
mod schedule_d_rules;
#[path = "rules/investment/schedule_e_rules.rs"]
mod schedule_e_rules;
#[path = "rules/farm/schedule_f_rules.rs"]
mod schedule_f_rules;
#[path = "rules/farm/schedule_j_rules.rs"]
mod schedule_j_rules;
#[path = "rules/credits/schedule_r_rules.rs"]
mod schedule_r_rules;
#[path = "rules/business/schedule_se_rules.rs"]
mod schedule_se_rules;
#[path = "rules/retirement/ssa_1099_rules.rs"]
mod ssa_1099_rules;
#[path = "rules/deductions/standard_deduction_rules.rs"]
mod standard_deduction_rules;
#[path = "rules/income/w2_rules.rs"]
mod w2_rules;
#[allow(unused_imports)]
pub use charity_rules::*;
#[allow(unused_imports)]
pub use dependents_rules::*;
#[allow(unused_imports)]
pub use educator_rules::*;
#[allow(unused_imports)]
pub use eic_rules::*;
#[allow(unused_imports)]
pub use estate_trust_k1_rules::*;
#[allow(unused_imports)]
pub use form_461_rules::*;
#[allow(unused_imports)]
pub use form_1040_es_rules::*;
#[allow(unused_imports)]
pub use form_1040_rules::*;
#[allow(unused_imports)]
pub use form_1098_e_rules::*;
#[allow(unused_imports)]
pub use form_1099_div_rules::*;
#[allow(unused_imports)]
pub use form_1099_g_rules::*;
#[allow(unused_imports)]
pub use form_1099_int_rules::*;
#[allow(unused_imports)]
pub use form_1099_patr_rules::*;
#[allow(unused_imports)]
pub use form_1099_r_rules::*;
#[allow(unused_imports)]
pub use form_1116_rules::*;
#[allow(unused_imports)]
pub use form_2106_rules::*;
#[allow(unused_imports)]
pub use form_2210_rules::*;
#[allow(unused_imports)]
pub use form_2441_rules::*;
#[allow(unused_imports)]
pub use form_2555_rules::*;
#[allow(unused_imports)]
pub use form_3800_rules::*;
#[allow(unused_imports)]
pub use form_4136_rules::*;
#[allow(unused_imports)]
pub use form_4562_rules::*;
#[allow(unused_imports)]
pub use form_4684_rules::*;
#[allow(unused_imports)]
pub use form_4797_rules::*;
#[allow(unused_imports)]
pub use form_4835_rules::*;
#[allow(unused_imports)]
pub use form_4952_rules::*;
#[allow(unused_imports)]
pub use form_5329_rules::*;
#[allow(unused_imports)]
pub use form_5695_rules::*;
#[allow(unused_imports)]
pub use form_6251_rules::*;
#[allow(unused_imports)]
pub use form_6252_rules::*;
#[allow(unused_imports)]
pub use form_6781_rules::*;
#[allow(unused_imports)]
pub use form_8379_rules::*;
#[allow(unused_imports)]
pub use form_8396_rules::*;
#[allow(unused_imports)]
pub use form_8582_rules::*;
#[allow(unused_imports)]
pub use form_8615_rules::*;
#[allow(unused_imports)]
pub use form_8801_rules::*;
#[allow(unused_imports)]
pub use form_8814_rules::*;
#[allow(unused_imports)]
pub use form_8815_rules::*;
#[allow(unused_imports)]
pub use form_8834_rules::*;
#[allow(unused_imports)]
pub use form_8839_rules::*;
#[allow(unused_imports)]
pub use form_8853_rules::*;
#[allow(unused_imports)]
pub use form_8859_rules::*;
#[allow(unused_imports)]
pub use form_8863_rules::*;
#[allow(unused_imports)]
pub use form_8880_rules::*;
#[allow(unused_imports)]
pub use form_8889_rules::*;
#[allow(unused_imports)]
pub use form_8911_rules::*;
#[allow(unused_imports)]
pub use form_8912_rules::*;
#[allow(unused_imports)]
pub use form_8936_rules::*;
#[allow(unused_imports)]
pub use form_8949_rules::*;
#[allow(unused_imports)]
pub use form_8959_rules::*;
#[allow(unused_imports)]
pub use form_8960_rules::*;
#[allow(unused_imports)]
pub use form_8962_rules::*;
#[allow(unused_imports)]
pub use form_8978_rules::*;
#[allow(unused_imports)]
pub use form_8995_rules::*;
#[allow(unused_imports)]
pub use home_sale_rules::*;
#[allow(unused_imports)]
pub use ira_contributions_rules::*;
#[allow(unused_imports)]
pub use k1_rules::*;
#[allow(unused_imports)]
pub use nol_rules::*;
#[allow(unused_imports)]
pub use other_income_rules::*;
#[allow(unused_imports)]
pub use payments_rules::*;
#[allow(unused_imports)]
pub use schedule_1_rules::*;
#[allow(unused_imports)]
pub use schedule_1a_rules::*;
#[allow(unused_imports)]
pub use schedule_2_rules::*;
#[allow(unused_imports)]
pub use schedule_3_rules::*;
#[allow(unused_imports)]
pub use schedule_3a_rules::*;
#[allow(unused_imports)]
pub use schedule_8812_rules::*;
#[allow(unused_imports)]
pub use schedule_a_rules::*;
#[allow(unused_imports)]
pub use schedule_c_rules::*;
#[allow(unused_imports)]
pub use schedule_d_rules::*;
#[allow(unused_imports)]
pub use schedule_e_rules::*;
#[allow(unused_imports)]
pub use schedule_f_rules::*;
#[allow(unused_imports)]
pub use schedule_j_rules::*;
#[allow(unused_imports)]
pub use schedule_r_rules::*;
#[allow(unused_imports)]
pub use schedule_se_rules::*;
#[allow(unused_imports)]
pub use ssa_1099_rules::*;
#[allow(unused_imports)]
pub use standard_deduction_rules::*;
#[allow(unused_imports)]
pub use w2_rules::*;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) const HUB_SOURCE: &str = concat!(
    include_str!("federal_rules.rs"),
    include_str!("rules/deductions/charity_rules.rs"),
    include_str!("rules/family/dependents_rules.rs"),
    include_str!("rules/deductions/educator_rules.rs"),
    include_str!("rules/family/eic_rules.rs"),
    include_str!("rules/passthrough/estate_trust_k1_rules.rs"),
    include_str!("rules/payments/form_1040_es_rules.rs"),
    include_str!("rules/federal/form_1040_rules.rs"),
    include_str!("rules/deductions/form_1098_e_rules.rs"),
    include_str!("rules/investment/form_1099_div_rules.rs"),
    include_str!("rules/payments/form_1099_g_rules.rs"),
    include_str!("rules/investment/form_1099_int_rules.rs"),
    include_str!("rules/farm/form_1099_patr_rules.rs"),
    include_str!("rules/retirement/form_1099_r_rules.rs"),
    include_str!("rules/foreign/form_1116_rules.rs"),
    include_str!("rules/business/form_2106_rules.rs"),
    include_str!("rules/payments/form_2210_rules.rs"),
    include_str!("rules/family/form_2441_rules.rs"),
    include_str!("rules/foreign/form_2555_rules.rs"),
    include_str!("rules/credits/form_3800_rules.rs"),
    include_str!("rules/credits/form_4136_rules.rs"),
    include_str!("rules/cost_recovery/form_4562_rules.rs"),
    include_str!("rules/business/form_461_rules.rs"),
    include_str!("rules/home/form_4684_rules.rs"),
    include_str!("rules/investment/form_4797_rules.rs"),
    include_str!("rules/farm/form_4835_rules.rs"),
    include_str!("rules/investment/form_4952_rules.rs"),
    include_str!("rules/retirement/form_5329_rules.rs"),
    include_str!("rules/credits/form_5695_rules.rs"),
    include_str!("rules/federal/form_6251_rules.rs"),
    include_str!("rules/investment/form_6252_rules.rs"),
    include_str!("rules/investment/form_6781_rules.rs"),
    include_str!("rules/attribution/form_8379_rules.rs"),
    include_str!("rules/home/form_8396_rules.rs"),
    include_str!("rules/activity/form_8582_rules.rs"),
    include_str!("rules/family/form_8615_rules.rs"),
    include_str!("rules/credits/form_8801_rules.rs"),
    include_str!("rules/family/form_8814_rules.rs"),
    include_str!("rules/investment/form_8815_rules.rs"),
    include_str!("rules/credits/form_8834_rules.rs"),
    include_str!("rules/family/form_8839_rules.rs"),
    include_str!("rules/health/form_8853_rules.rs"),
    include_str!("rules/credits/form_8859_rules.rs"),
    include_str!("rules/family/form_8863_rules.rs"),
    include_str!("rules/credits/form_8880_rules.rs"),
    include_str!("rules/health/form_8889_rules.rs"),
    include_str!("rules/credits/form_8911_rules.rs"),
    include_str!("rules/credits/form_8912_rules.rs"),
    include_str!("rules/credits/form_8936_rules.rs"),
    include_str!("rules/investment/form_8949_rules.rs"),
    include_str!("rules/income/form_8959_rules.rs"),
    include_str!("rules/investment/form_8960_rules.rs"),
    include_str!("rules/health/form_8962_rules.rs"),
    include_str!("rules/passthrough/form_8978_rules.rs"),
    include_str!("rules/activity/form_8995_rules.rs"),
    include_str!("rules/home/home_sale_rules.rs"),
    include_str!("rules/retirement/ira_contributions_rules.rs"),
    include_str!("rules/passthrough/k1_rules.rs"),
    include_str!("rules/federal/nol_rules.rs"),
    include_str!("rules/income/other_income_rules.rs"),
    include_str!("rules/payments/payments_rules.rs"),
    include_str!("rules/federal/schedule_1_rules.rs"),
    include_str!("rules/deductions/schedule_1a_rules.rs"),
    include_str!("rules/federal/schedule_2_rules.rs"),
    include_str!("rules/credits/schedule_3_rules.rs"),
    include_str!("rules/credits/schedule_3a_rules.rs"),
    include_str!("rules/family/schedule_8812_rules.rs"),
    include_str!("rules/deductions/schedule_a_rules.rs"),
    include_str!("rules/business/schedule_c_rules.rs"),
    include_str!("rules/investment/schedule_d_rules.rs"),
    include_str!("rules/investment/schedule_e_rules.rs"),
    include_str!("rules/farm/schedule_f_rules.rs"),
    include_str!("rules/farm/schedule_j_rules.rs"),
    include_str!("rules/credits/schedule_r_rules.rs"),
    include_str!("rules/business/schedule_se_rules.rs"),
    include_str!("rules/retirement/ssa_1099_rules.rs"),
    include_str!("rules/deductions/standard_deduction_rules.rs"),
    include_str!("rules/income/w2_rules.rs"),
);
// End of the tools/split-hubs block.
