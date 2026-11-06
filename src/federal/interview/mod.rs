mod crop_insurance;
mod payment_parts_editor;
pub(crate) use crate::diagnostics::contextual_message;
use crate::section_labels::section_info;
pub(crate) use crate::section_labels::section_message;
#[cfg(test)]
pub(crate) use crate::section_labels::section_name;

// Coverage: gate fixes, batch A (2026-10-06; direct-question redesign).
//   Row 1: the whole-return situations are asked on the filers' pages with "None of these":
//   Filer.situations (YourSituations: dual status, community property with a registered
//   domestic partner) on About you, SpouseAnswers.situations (SpouseSituations: the section
//   6013(g)/(h) resident choice and its two net investment income tax items, injured spouse,
//   innocent spouse) on the spouse page, a joint return only; each stored in its filer's
//   FederalPerson.whole_return (FederalDraft::whole_return is the union). The injured spouse
//   item opens Form 8379's page (owned data FederalDraft.injured_spouse); situations_issue
//   refuses the other page's items, the resident choice with a dual-status return and the
//   NIIT items without the resident choice. federal_rules::whole_return refuses dual status
//   and Form 8958 community property under About you and gives notices for the section
//   6013(g)/(h) choice and Form 8857 under the spouse page.
//   Row 2: the checklist's "Something less common" ticks left every list; each hub's less-common
//   page (Less common income, More adjustments, More itemized deductions, More credits) is a
//   derived step of its walk until visited (derived_topics), and the income, adjustments,
//   credits and other taxes sections are shown on every return (sections_with_needs); a
//   less-common row reads "Not started" until asked. Row 3: ReturnSetup.has_dependents and
//   FederalDraft.has_dependents left; FederalDraft.dependents is a plain list, the people page
//   is asked on every return after the filer pages (its visit in finished_sections), and the
//   credits hub's care row and the estimate read the list. Row 4: Dependent.facts (PersonFacts
//   Bitflags) replaced MoreAboutPerson. Row 39: Deductions.cash_gifts (an amount, 0 is none)
//   replaced CashGifts. Unit18 replaces the old two investment-interest amounts
//   with complete source-owned borrowing, separate AMT carry and election facts.
//   Tests: src/tests/cross/interview.rs whole_return_situations_refuse_and_notice_on_the_filers_pages,
//   injured_spouse_item_*, person_fold_maps_citizenship_and_joint_return,
//   cash_gifts_reach_the_non_itemizer_deduction, the sequence, walk and fixture tests amended.
// Coverage: M1 interview pages (unit M1-P1, 2026-09-24; M2-P1 2026-09-24).
//   Pages: 1 "Your 2026 return" (ReturnSetup: filing status union; has_dependents left in gate
//   fixes batch A) and 2 "About
//   you" / "About your spouse" (Filer; the spouse page is sent only for a joint return). They are
//   projections of FederalSetup/FederalPerson; names are stored as entered (first_name, last_name)
//   and joined into `name` for existing readers. M2-P1: head of household (home cost, unmarried or
//   living apart) and qualifying surviving spouse (death year, home cost, could file jointly) are
//   Filing branches; client rules HOH_/QSS_NEEDS_HOME_COST (HOH_/QSS_NEEDS_A_DEPENDENT left with
//   has_dependents in gate fixes batch A; the estimate reads the people page's list)
//   have server twins in setup_rule; Identification on both filer pages supplies valid_ssn and
//   valid_tin. Still refused by name: community-property allocation. Removed from the pages:
//   return label, marital situation, basis and its change confirmation.
//   Evidence: federal_model.rs ReturnSetup/Filing/HeadMarital/Filer/Identification/Claimable;
//   src/tests/cross/interview.rs m1_setup_and_filer_pages_reach_the_draft, single_filer_never_sees_spouse_page,
//   head_of_household_page_reaches_the_draft, surviving_spouse_page_reaches_the_draft,
//   household_status_needs_a_dependent_and_home_cost, identification_supplies_ssn_and_tin_answers.
// Coverage: M3 hubs (unit M3-P1, 2026-09-25).
//   Sequence: setup, you, spouse (joint), people (with dependents), income hub, review; the later
//   sections keep their nothing-applies values through fill_unasked until their M3 units land.
//   The income hub uses the Hub response (NextRequest Navigate,OpenSection, no ConstrainedBy):
//   rows open sections through link-only OpenSection; Go to opens Navigate's popup preselected
//   with suggested. Topic pages (Wages, Interest and dividends) open by EditRequest with page =
//   Income and topic in the draft identity, save only their kinds, and return to the hub or the
//   review they came from; Continue records Income in FederalDraft.finished_sections (a stored
//   TaxSection: renumbering now needs a STORAGE_FORMAT bump); progress "section N of 10" (9
//   without dependents) with the topic name; estimates inside the section count the hub as
//   finished. Legacy Begin/Save Adjustments, Deductions, Credits and Screening stay until P7,
//   P9, P11 and P15 replace them. STORAGE_FORMAT 25. Tests: src/tests/cross/interview.rs
//   income_hub_suggests_wages_then_continue, topic_page_opens_by_edit_request_and_returns_to_its_hub,
//   wages_save_keeps_the_interest_documents, topic_is_part_of_the_draft_identity,
//   page_opened_from_the_review_returns_to_it, leaving_the_income_hub_records_it_finished,
//   m1_fixture_completes_through_the_hubs, m2_fixture_completes_through_the_hubs.
// Coverage: M3 W-2, 1099-INT, 1099-DIV and 1099-OID pages (unit M3-P2, 2026-09-25).
//   The W-2 other boxes (W2More) carry box 8/unreported tips, box 10 care benefits, box 12 as
//   Box12Entry rows (A/B/M/N to uncollected tax, TP/TT to the work deductions, the rest stored in
//   FederalWage.other_box12; H/K/R/T/Z/TA refused by name), box 13 statutory and retirement plan,
//   the church-employee flag; W-2 rules W2_SOCIAL_WAGES_AND_TIPS_ABOVE_BASE and
//   W2_BOX12_CODE_TWICE on both sides. 1099-INT gains box 3 (FederalInterest.us_obligations), boxes 6,
//   11-13 and accrued interest; 1099-DIV boxes 7, 12, 13; the 1099-OID document maps onto
//   FederalOid; box 6/7 foreign tax becomes ForeignTaxClaim::Statements when within the election
//   and is restored on reopen; DIV_2B/2D_ABOVE_2A, DIV_13_ABOVE_12, INT/OID_PRIVATE_ACTIVITY_ABOVE_EXEMPT
//   on both sides. StatutoryEmployee and EmployerCareBenefits situations removed. STORAGE_FORMAT 26.
//   Tests: src/tests/cross/interview.rs w2_fold_maps_tips_box12_and_box13, box12_code_twice_is_rejected,
//   w2_social_wages_and_tips_above_base_is_rejected, statutory_w2_is_stored_and_refused_by_name,
//   us_obligations_interest_reopens_in_box_3, oid_document_maps_onto_federal_oid,
//   small_foreign_tax_becomes_the_statement_election, div_box_rules_reject_on_both_sides.
//   W-2 document import (2026-09-27): the Wages editor supplies hidden tax-year facts for
//   document validation. W2 selects printed boxes 1-8, 10, supported 12 codes, and 13; Box 12
//   import replaces its prior rows, and the Box 8 result permits only None or Allocated.
//   Unprinted taxpayer facts remain in the normal editor. Tests:
//   wages_editor_supplies_hidden_w2_document_year;
//   Studio tax-schema compilation checks the nested result and validation paths.
// Coverage: M3 investment sales, unemployment and refunds, prizes and royalties (unit M3-P3, 2026-09-25).
//   Three more income topics (INCOME_TOPICS = 5): Sale1099B onto FederalSale (term flag, basis
//   reporting, one adjustment, wash sale, collectibles; SALE_WASH_SALE_ABOVE_LOSS both sides);
//   Form1099G with its payment and refund projections and the
//   RefundDeduction union onto TaxRefundHistory (SeveralYears stored as a multi-year record with
//   allocation_complete false, left out with a notice; REFUND_* rules); Prize1099Misc, Royalty1099Misc and
//   EquipmentRent onto PrizeAward and NonbusinessProperty (EquipmentRent.assets waits for M3-P5's
//   Asset). InvestmentSales and UnemploymentOrStateRefund situations removed, OtherIncome narrowed.
//   STORAGE_FORMAT 27. Tests: src/tests/investment/interview.rs sale_maps_onto_federal_sale_with_its_box,
//   wash_sale_above_the_loss_is_rejected, unemployment_fold_boxes_refuse_by_name,
//   refund_year_is_the_deduction_year, prize_royalty_and_equipment_rent_reach_their_lines.
// Coverage: pages for the unasked adjustments (unit u5-adjust, 2026-09-28).
//   More adjustments is a list page like More credits (SaveMoreAdjustments /
//   EstimateMoreAdjustments, MoreAdjustments of MoreAdjustment): the "Alimony paid" kind
//   (AlimonyItem: owner inside allowed_owners, the stored AlimonyPayment) onto
//   FederalAdjustments.alimony_paid, the "Moving expenses" kind (unit form-3903, 2026-10-09:
//   MovingItem onto FederalAdjustments.moving, the owner's code P re-derived into
//   MilitaryMove.reimbursed by derive_reimbursed; moving.rs), and the refusal kinds Archer MSA,
//   Form 2106 and other adjustments stored as their situations (ADJUSTMENT_REFUSALS); its hub
//   row joins a count for each kind (alimony agreements, moves) and the refusal kinds with
//   REVIEW_STATUS_JOIN. The W-2 box 12 code is one enum (W2Code) on Box12Entry and in
//   FederalWage.other_box12; W2_BOX12_CODE_TWICE is one is_unique_set rule. The situations page no longer serves MoreAdjustments;
//   RareSituation::AlimonyPaid and PublicSafetyPremiums are removed. The Retirement topic gains
//   the kinds SocialLumpSums (LumpSumElection onto social_election, once per return) and
//   MissedRmd (MissedDistribution onto required_distributions, by owner), and the 1099-R's last
//   field public_safety (PublicSafetyPaid rows onto screening.public_safety, linked to the saved
//   entry's id and restored by id on reopen). Review items: "Alimony"/"Alimony recapture" open
//   More adjustments; the Social Security election, required distribution and public safety
//   sections open the Retirement topic. STORAGE_FORMAT 73. Tests: src/tests/federal/unasked_pages.rs.
// Coverage: alimony received (unit u6-alimony, 2026-09-28).
//   The OtherIncome topic, renamed "Prizes, royalties, equipment rent and alimony" (its checklist
//   tick "Prizes, royalties, equipment rent (1099-MISC) or alimony received"), offers
//   IncomeRecord::AlimonyReceived carrying the shared AlimonyPayment
//   onto FederalIncome::Alimony, one unnamed entry per agreement, owned by the
//   item's owner inside allowed_owners (checked again on save with the record's `valid`, at most
//   20). Rules on both sides: ALIMONY_2024/2025_EXCLUDED_ABOVE_PAID (declared on AlimonyPayment,
//   server twin alimony_issue through document_issue). RareSituation::OtherIncome narrowed to jury
//   duty, scholarships, hobby and other income (Schedule 1 line 2a leaves its refusal; unit B6b
//   then moved those kinds onto this topic and narrowed it to line 8z). Review
//   items "Alimony received"/"Alimony recapture (received)" open the topic; the payer's are
//   "Alimony paid"/"Alimony recapture (paid)" on More adjustments. STORAGE_FORMAT 74 (the shared
//   record's new_rules answer). Tests: src/tests/family/alimony_received.rs; change_audit.rs
//   leaving_joint_through_page_1_erases_the_spouses_alimony_received.
// Coverage: 1099-R box 7a as printed (2026-09-28). R1099.box7 (Bitflags DistributionCodes) is the
//   printed codes and the only box 7a AI Import reads; DistributionCode is the filer's treatment,
//   constrained per row by allowed_treatments, which the topic catalog and retirement_item /
//   roth_item fill on reopen (document_rule refuses R_BOX7_TREATMENT; see retirement.rs Coverage). retirement_record stores box7 on FederalRetirement; distribution_code
//   picks the reopened branch from box7 and fills it from the treatment, so 1 vs 2 and 3 vs an
//   exception dated December 31 reopen as entered; roth_item prints Q, J or T from RothCode.
//   RothIraCode::Early lost `also` (8J, PJ, SJ lead to Another code). display.json splits R1099
//   on tablet and desktop.
// Coverage: unit B5 page mapping (2026-10-09). retirement_record takes the kind from box 7a (D with
//   1, 2, 3, 4 or 7 is Nonqualified) and box 7b and the taxable kind from the box 2b method
//   (TaxableMethod, ten choices; the row's allowed set R1099.allowed_methods is written by
//   the topic catalog beside allowed_treatments and by retirement_item on reopen); a Trump account row
//   (box 7c) is its own income kind (trump_record / trump_item); an Rrb1099R document is the
//   Railroad pension record (rrb_record / rrb_item); an InheritedIra document is the Inherited
//   IRA record (FederalInheritedIra), named by the SaveIncomes catalog `inherited_iras`
//   (income_inherited_iras, keyed by income identity like the farms catalog) from the code 4
//   branch, an inherited Roth withdrawal and a QCD gift.
// Coverage: unit B6a page mapping (2026-10-09). The Prizes, scholarships, relief payments, account
//   payouts and other income page gains OtherIncomeDocument::Sa1099 (FederalIncome::MedicalDistribution,
//   sa1099 record/page), Ltc1099 (FederalIncome::LtcBenefits) with its LtcInsured periods record
//   (FederalIncome::LtcInsured), Qa1099 (FederalIncome::Able) and the AbleContributions data
//   answer (screening.able_contributions, by owner), all appended in Avro order. A 1099-LTC names
//   its record through the SaveIncomes/EstimateDocuments/PreviewDocument catalog `ltc_insured`
//   (income_ltc_insured, keyed by income identity like the farms and inherited_iras catalogs; an
//   empty choice uses the only record, a joint return either filer's). The HSA page gains the
//   stored HSA fields and Archer MSA contributions (hsa_record / hsa_item, hsa_candidate,
//   catalogs `ira_sources` and `archer_income`: hsa_ira_sources, archer_income_sources). A
//   removed record is not cleared from the forms that name it: the link resolves to Missing and
//   the per diem payments are taxed in full with WARN_LTC_PERIODS_MISSING; a periods record of a
//   person who left the return is removed by the walk. RareSituation::AccountDistributions and
//   MoreAdjustment::ArcherMsa are removed. Tests: src/tests/health/account_documents.rs; src/tests/cross/interview.rs
//   every_situation_has_one_destination.
// Coverage: Roth IRA 1099-R codes and history (unit u6-roth, 2026-09-28).
//   DistributionCode::RothIra { roth: RothIraCode } (Q, J with date and exception,
//   T with date and reason) maps onto FederalIncome::Roth (document_income, roth_item back;
//   topic_of Retirement, the kind "1099-R"); IncomeRecord::RothHistory onto
//   adjustments.roth_history by owner (roth_history_records: ROTH_HISTORY_TWICE, spouse only
//   jointly), rebuilt on reopen; the Retirement tick owns it (income_topic_entered). R1099 rules
//   R_ROTH_OTHER_BOXES, R_ROTH_DISASTER, ROTH_DISABLED_AFTER_DISTRIBUTION and the widened
//   PSO_NOT_FROM_AN_IRA on both sides; "Roth IRA" issues open the Retirement topic.
//   STORAGE_FORMAT 74. Tests: src/tests/retirement/roth_pages.rs.
// Coverage: M3 retirement and Social Security (unit M3-P4, 2026-09-25).
//   The Retirement topic (INCOME_TOPICS = 6): R1099 in box order with the box 7 code as the
//   DistributionCode union onto RetirementTreatment (7, 7 rolled over, 1/2 with an exception, G to
//   a plan, S, D; the other codes stored as RetirementTreatment::Unsupported and refused
//   by name; code 6 is the Exchange branch since unit B5), the IRA box, Ssa1099 onto the Social Security record, Form8606
//   switching that owner's IRAs to the basis worksheet (FORM_8606_TWICE), QcdGift grouped by
//   owner into QualifiedCharity; R_TAXABLE_ABOVE_GROSS and R_WITHHOLDING_ABOVE_GROSS on both
//   sides, box 9a/10 rejected on save only. The hub row counts only income documents (a Form
//   8606 or QCD alone shows the topic empty). RetirementIncome, SocialSecurity and
//   UnusualRetirementCodes situations removed. STORAGE_FORMAT 28. Tests: src/tests/retirement/interview.rs
//   distribution_codes_map_onto_treatments, form_8606_switches_that_owners_iras_to_the_basis_worksheet,
//   form_8606_twice_for_the_same_owner_is_rejected, qcd_gifts_group_by_owner,
//   unsupported_code_is_refused_by_name, r1099_box_relationships_reject_on_both_sides, ssa_1099_reaches_line_6.
// Coverage: M3 self-employment (unit M3-P5, 2026-09-25).
//   The SelfEmployment topic (INCOME_TOPICS = 7): Business onto FederalBusiness (Ordinary activity,
//   UnitedStates source): each nonzero Part II line one FederalBusinessExpense (meals Meals or, with
//   dot_meals, TransportationMeals); StandardMileage onto BusinessMileage (Nothing: total = business
//   miles, owned); Actual and each Asset onto BusinessAsset (Regular method, tables, confirmed;
//   Section179 elects its amount under its description, otherwise Eligible { 0 } above 50% use and
//   not 39-year, else Ineligible; Regular elects out of bonus; an actual vehicle's percent is
//   business miles x 100 / total, rounded down); vehicles stored mileage first, "Vehicle n"/"Asset n"
//   labels; the simplified office as twelve months (the first `months` at 15 days); health insurance
//   and retirement kept from the accepted record. BUSINESS_WITHHOLDING_ABOVE_RECEIPTS,
//   MILEAGE_ABOVE_TOTAL_MILES, VEHICLE_BUSINESS_MILES_ABOVE_TOTAL, ASSET_ACQUIRED_AFTER_PLACED,
//   ASSET_179_ABOVE_BUSINESS_COST on both sides; ASSET_NEW_PLACED_IN_2026,
//   ASSET_OLD_PLACED_BEFORE_2026, ASSET_BONUS_ELECTION_BY_CLASS (consistent_elections on the
//   candidate return) and EQUIPMENT_RENT_NO_SECTION_179 server-only. EquipmentRent.assets onto the
//   personal rental's assets (never section 179 property). Stored and refused by name:
//   The special answer (Schedule C lines 12, 19,
//   27a and the other special treatments). SelfEmployment and BusinessDepletionOrPlans situations
//   removed. STORAGE_FORMAT 29. T2-49b (2026-09-26): BusinessMore::Something.commercial_fishing onto
//   FederalBusiness.commercial_fishing (Nothing: false), the fishing marker for the 2026
//   farmer/fisherman test (underpayment::two_thirds_return_year); test
//   commercial_fishing_round_trips_through_the_business_page. STORAGE_FORMAT 57. Tests: src/tests/business/interview.rs expense_lines_become_one_record_each,
//   mileage_nothing_branch_totals_the_business_miles, actual_vehicle_percent_comes_from_miles,
//   first_year_choices_map_onto_179_and_bonus, bonus_election_must_match_within_a_class,
//   simplified_office_months_map_onto_twelve_entries, actual_expense_office_is_refused_by_name,
//   special_business_treatment_is_refused_by_name, equipment_rent_assets_map_onto_business_assets,
//   sole_proprietor_page_reproduces_the_row_9_fixture.
//   Form 8829 uses canonical HomeOfficeSite sources through SaveHomeOffices;
//   business/farm pages retain only the linked activity, never a duplicate office.
//   Native source and worksheet controls: src/tests/home/interview.rs and business.rs.
// Coverage: M3 rental property (unit M3-P6, 2026-09-25).
//   The Rentals topic (INCOME_TOPICS = 8): Rental onto FederalRental: kind onto property_type; each
//   nonzero line 5-17 one ordinary RentalExpense, line 19 rows as Other; the building then the
//   improvements as RentalBuilding (FormattedText LABEL_BUILDING, LABEL_IMPROVEMENT_N); PersonalUse plus the fold's
//   home-use carryovers onto RentalPersonalUse (carryovers without personal use stored with zero
//   days); the fold's Nothing branch is active, at risk, investment-only, no suspended loss,
//   payroll, group or special treatment; SuspendedLoss onto RentalHistory, RentalBusiness (the
//   variant states eligibility) and BusinessPayroll with the page's answer; special onto
//   ordinary_rental false (left out with WARN_RENTAL_LEFT_OUT). Entry label is the
//   address or "Rental"; the row previews line 21. RENTAL_LAND_ABOVE_COST (improvements checked on
//   the server too) and RENTAL_DAYS_ABOVE_365 on both sides. Stored and refused by name: line 1b
//   types 3 and 5-8 (extended_rules). The
//   RealEstateProfessional situation { who } (You; Spouse/Both only on a joint return through
//   allowed_professionals) is offered only with a rental (RareSituation::allowed(rental)), checked
//   again on save, and writes FederalScreening.real_estate_professionals (qualifies Yes,
//   hours_confirmed true: choosing it is the Pub 925 hours confirmation, so the
//   REAL_ESTATE_PROFESSIONAL_NEEDS_HOURS page rule and screening_rule are removed); it reopens from
//   those records. The prune (walk-owned data, below) removes the records when no rental remains
//   and the spouse's off a joint return. RentalOrRoyalty and NonresidentialRental situations removed.
//   STORAGE_FORMAT 30. Tests: src/tests/cross/interview.rs rental_lines_and_building_map_onto_federal_rental,
//   land_above_cost_is_rejected, vacation_rental_type_is_refused_by_name,
//   professional_situation_is_offered_only_with_a_rental,
//   removing_the_last_rental_drops_the_professional_answer.
// Coverage: T2-35 sale folds (2026-09-26). Asset and VehicleMethod::Actual gain `disposal:
//   AssetDisposal` (No | Disposed { date (2026 range), how: Sold { proceeds, selling_expenses } |
//   Scrapped | PersonalUse }, inline choices) onto BusinessAsset.disposal; Rental gains `sale:
//   RentalSale` (No | Sold { date, price, land_price, selling_expenses }) after the building, onto
//   FederalRental.sale (whole property, so it sits on the rental, not on each building record).
//   ASSET_DISPOSED_BEFORE_PLACED, RENTAL_SOLD_BEFORE_IN_SERVICE (improvements on the server
//   only) and RENTAL_LAND_PRICE_ABOVE_PRICE on both sides. IncomeRecord::BusinessPropertyLosses
//   (Section1231History onto FederalScreening.section1231) is a data answer on both the
//   self-employment and rentals pages, one record for the return (BUSINESS_PROPERTY_LOSSES_TWICE);
//   "Section 1231 lookback" issues point at self-employment (rentals without a business).
//   RentalMore.special no longer mentions a sale. STORAGE_FORMAT 47. Tests: src/tests/cross/interview.rs
//   asset_disposal_fold_maps_onto_the_asset_and_back,
//   rental_sale_fold_maps_and_its_rules_reject_on_both_sides,
//   business_property_losses_kind_writes_the_lookback_from_either_page.
// Coverage: M3 adjustments hub and HSA (unit M3-P7, 2026-09-25).
//   Sequence: setup, you, spouse (joint), people (with dependents), income hub, adjustments hub,
//   review. The adjustments hub is the same Hub response (page Adjustments, previous Income,
//   allowed_sections [Income, Hsa, Review]); it suggests Hsa while a W-2 reports box 12 code W for
//   an owner with no account or an account's special answer is stored, else Continue, which
//   records Adjustments in finished_sections (resume and the review's Go to start there first);
//   progress "section 4 of 9" (5 of 10 with dependents), "· Health savings account" on the page.
//   The review gains the "Adjustments to income" row. SaveHsa (HsaPage: up to two HsaAccount rows,
//   FamilySplit) opens by EditRequest with page = previous = Adjustments and maps onto
//   FederalAdjustments.hsa: coverage SelfOnly/Family onto FullYear, Changed onto Monthly; medical
//   onto distributions; the fold's withdrawals onto nonmedical; every other list and option empty;
//   fully_eligible true, or false for the special answer, which federal_rules refuses by name (Form
//   8889 lines 10, 14b, 18 and 19; Form 5329 Part VII). employer is W-2 box 12 W summed over the
//   owner's W-2s (FederalWage.other_box12), ignored on save and re-derived by reconcile whenever
//   the return changes. HsaPage preserves the actual Monthly/December method and other-filer
//   eligibility when only one actual account is entered, including married separate returns.
//   HsaOptions reads the account coverage, external eligibility and method; it writes only
//   allowed_family_split. Both eligible spouses share if either has family coverage; equal is
//   the default unless there is an actual different agreement. EstimateHsa reads the split
//   downstream. Server recomputes marriage, owners and allowed choices from accepted facts;
//   echoed catalogs and readonly permissions do not authorize a candidate. Normal pruning
//   retains the method and removes only inapplicable spouse context or an empty topic.
//   (HSA_ACCOUNT_TWICE gained its client twin in T2-cx, below.)
//   Removed: RareSituation::HealthSavings (Archer MSA stays under OtherAdjustments), the legacy
//   BeginAdjustments/SaveAdjustments page, and FederalIras.spouse's orphaned ConstrainedBy.
//   STORAGE_FORMAT 31. Tests: src/tests/cross/interview.rs adjustments_hub_suggests_hsa_when_box_12_w_exists,
//   employer_contributions_come_from_box_12_w, family_split_uses_either_family_policy_and_overlapping_eligible_months,
//   hsa_options_does_not_read_the_split, hsa_special_is_refused_by_name.
// Coverage: M3 adjustment topic pages (unit M3-P8, 2026-09-25).
//   Four more adjustments-hub topics (ADJUSTMENT_TOPICS = [Hsa, SelfEmployedBenefits,
//   IraContributions, Educator, StudentLoan]). Self-employed benefits (SaveSelfEmployedBenefits:
//   SelfEmployedBenefits.businesses, one read-only, membership-only BusinessBenefits row per
//   Schedule C business, built and matched by FederalIncomeEntry.id) maps its SeHealth/SeRetirement
//   fold onto FederalBusiness.health_insurance/.retirement: the complete Plan preserves actual
//   establishment, monthly employer eligibility, child premium increases and per-person LTC.
//   Sep maps straight onto BusinessRetirement::Sep. It is offered only when a
//   business exists (adjustments_hub_sections and the review's allowed_sections both gate it on
//   has_business) and is suggested exactly while a business exists and nothing has been entered on
//   any of its rows; a save whose business ids no longer match the return (membership changed since
//   the page was built) is refused, though the revision guard already catches that case first. IRA
//   contributions (SaveIraContributions: up to two IraContributor rows, allowed_owners) maps onto
//   FederalIras (stored as entered; FederalIras::entered, not a stored switch, says whether any
//   IRA fact exists), covered_at_work always
//   re-derived from a W-2 box 13 retirement plan or an owner's own business/partnership SEP
//   (ira_covered, ignored on save like the HSA page's employer amount), the spouse row filling
//   IraSpouse::Joint on a joint return (including a zero row when only the taxpayer contributes) and
//   a separate return's own spouse_covered_if_separate fold filling IraSpouse::SeparateTogether,
//   both keyed off FederalSetup::filing_status/lived_apart_all_year rather than a client choice;
//   IRA_NONDEDUCTIBLE_ABOVE_TRADITIONAL on both sides; IRA_CONTRIBUTOR_TWICE was server only until
//   T2-cx gave it a client twin (below). The adjustments hub
//   suggests it while box 12 elective-deferral codes (D/E/F/G/S) or a prior Form 8606 exist and
//   no IRA fact is entered. T2-43 (2026-09-26): SeRetirement::Other (a SIMPLE IRA, solo 401(k) or
//   other owner plan) maps onto BusinessRetirement::Other, stored and refused by name in sep.rs
//   (Schedule 1 line 16; Form 7206 line 9); STORAGE_FORMAT 51. Test: src/tests/cross/interview.rs
//   other_owner_plan_is_stored_and_refused_by_name. Educator expenses (SaveEducator: up to two Educator rows, allowed_owners)
//   and student loan interest (SaveStudentLoan: up to ten Form1098E rows, no owner) are ordinary
//   topics with no suggestion trigger of their own, list-emptiness (not amount) deciding
//   EducatorClaim/StudentLoanClaim's Yes/No as the design's confirmation-is-a-branch convention
//   requires. T2-33x (2026-09-26): the Forms 1098-E are stored as entered (box 1 and the box 2
//   fold with its figured additional interest) and reopen unchanged; the page's other_interest
//   (interest paid with no Form 1098-E) is StudentLoanClaim's student_loan_interest, and a page
//   with no form and no such interest is No. EDUCATOR_TWICE server only until T2-cx (below). RareSituation::IraContributions is removed (federal_rules.rs ALL/refusal mechanically
//   shrink to 47) and OtherAdjustments' description drops the now-supported self-employed
//   retirement mention. The review's "Adjustments to income" count (adjustments_count) now spans
//   all five topics ("n adjustments", not "n HSAs"). STORAGE_FORMAT 32. Tests: src/tests/business/interview.rs
//   business_benefits_write_the_business_record, covered_at_work_comes_from_box_13_or_a_sep,
//   nondeductible_above_traditional_is_rejected, educator_and_1098e_pages_map_onto_their_claims.
// Coverage: deductions and itemized deduction topics.
//   The deductions page edits cash gifts, casualties, vehicle-loan facts,
//   repayment treatment and the optional election to itemize. With no override,
//   the existing calculation compares standard and itemized routes, including
//   final liability, AMT and refundable credits. The itemized facts remain
//   stored when the standard deduction wins. Applicable document and checklist
//   topics open the itemized hub; corrections rederive the estimate directly.
//   A separate return asks whether the spouse itemizes through a required
//   Boolean request without a Default. Only an accepted answer is sent back in
//   EditRequest. A spouse who itemizes requires itemization and precludes the
//   spouse's additional standard-deduction boxes. Other filing statuses never
//   ask this question. Medical, taxes, mortgage, charity and investment interest
//   continue to use their existing source records and calculation owners.
//   Tests: deductions::interview, deductions::deduction_pages,
//   deductions::deductions and home::business.
//   Sources: https://www.irs.gov/instructions/i1040sca
//   https://www.irs.gov/publications/p501
//
// Coverage: M3 home mortgage and charity carryovers (unit M3-P10, 2026-09-25).
//   Completes the itemized hub P9 began: home mortgage and charity carryovers move from "Not
//   available yet" into OPEN_ITEMIZED_TOPICS, so both follow the same zero/visited/suggested row
//   rule as the other three topics. Home mortgage (SaveMortgages: one Mortgage1098 row per Form
//   1098) maps two ways onto FederalDeductions.mortgage's list of
//   mortgage_limit::MortgageLoan. B10-4 replaces the old gate answers with purpose/refinance,
//   direct box4 and its separate origin, typed points and monthly balances. Box3 stays printed.
//   WithinLimit excludes duplicate lender entries, other-use loans and unqualified homes;
//   page rules have server twins in MortgageLoan::rule plus page-level aggregate checks.
//   FirstAndLast. Unit15 supersedes the former cash-only charity page with one
//   canonical class/year ledger and shared property/appraisal records. The Standard
//   route clears all itemized gift-page records through normal pruning.
//   Tests: src/tests/cross/interview.rs within_limit_default_deducts_all_interest,
//   within_limit_above_the_limit_is_rejected_by_name, points_branches_map_onto_mortgage_points,
//   last_balance_above_box_2_is_rejected, carryovers_map_onto_prior_cash_charity (which also
//   covers the Standard route clearing prior_cash_charity), and the updated
//   compare_and_itemize_map_onto_the_selection (all five topics now open, home mortgage and
//   charity carryovers visited alongside the other three).
// Coverage: M3 credits hub, education and retirement savings (unit M3-P11, 2026-09-25).
//   Sequence: ... deductions page, the itemized hub when the route itemizes, the credits hub
//   (section 7: "section 6 of 9", 7 of 10), review. The credits hub is the one Hub response
//   (page Credits, previous the itemized hub or the deductions page, allowed_sections
//   [previous, Children with dependents, Education, Saver when offered, Review]); topics in
//   Schedule 3's order: child and dependent care (listed only with dependents, status from the
//   people page's care costs, opens M2's people page with previous = Credits so its Back and
//   Next return to the hub), education, the saver's credit, and more credits (always "Not
//   available yet", not in allowed_sections until M3-P12). The child tax credit and the earned
//   income credit have no page. Suggestion: education while a filer or person supported is a
//   student and the topic is not visited; the saver's credit while box 12 deferrals (D, E, F,
//   G, S, AA, BB, EE) or IRA contributions exist, the estimate's AGI is inside Form 8880's
//   bands (saver_rate > 0; an AGI the estimate cannot state does not hide it) and it is not
//   visited; otherwise Continue, which records Credits in finished_sections. The saver's topic
//   is offered only on those same conditions or once a credit is recorded; its page and save
//   are refused otherwise. Topic saves record their topic in finished_sections (visited).
//   The TaxSection numbering puts Credits (6) before Itemized (7) while the sequence runs the
//   other way, so page admission compares sequence ranks (rank/reached), not raw numbers.
//   Education (SaveEducation: EducationCredits, one StudentCredit per student, a hidden people
//   catalog of you, your spouse and the people you support, refused on save when the echoed
//   catalog differs or a student is outside it) maps onto FederalEducation: tuition = box 1,
//   Unique on the student selection and the native save reject repeated students before
//   persistence. Tests: validation_audit_f07_education_* (fresh rows, rejection, reopen).
//   school_materials/other_materials/refunds/other_benefits from the fold, assistance = box 5,
//   the credit is derived: AmericanOpportunity for a
//   half-time (box 8) non-graduate (box 9) student who has not used it up, else
//   LifetimeLearning. Boxes 8/9 and "used up" are not stored, so a lifetime learning row
//   reopens as a half-time graduate student (same record on an unchanged save). The stored
//   FederalCredits.students catalog follows the people on every accepted change
//   (follow_students/set_students): a removed person's education row goes, the rest are
//   re-indexed by person id; a marketplace dependent's missing person is kept at the end as
//   no longer on this return. Saver (SaveSaver: SaverCredit) maps onto FederalSaver:
//   taxpayer_contributions = the taxpayer's box 12 deferrals + other_contributions (a single
//   field, the taxpayer's), spouse_contributions = the spouse's box 12 deferrals, the ABLE
//   amounts and the distributions list; from_w2 is derived and ignored on save; a changed W-2
//   re-derives the box 12 part and keeps the rest (follow_saver). The design's help text
//   "2026 distributions on your 1099-Rs are counted automatically" is not what the calculation
//   does (only permissible automatic-enrollment withdrawals are), so the field's help asks for
//   2026 distributions too. Removed: RareSituation::Education (federal_rules.rs ALL shrinks to
//   45), the legacy BeginCredits/SaveCredits page with its handler and student_present,
//   FederalCredits.students' CopyFrom (Studio: reachable only from a Request), and
//   the two tests' SaveCredits steps (src/tests/credits/credits.rs, src/tests/retirement/retirement.rs). STORAGE_FORMAT
//   35. Tests: src/tests/cross/interview.rs aotc_is_derived_for_an_undergraduate,
//   graduate_or_part_time_student_gets_lifetime_learning, saver_contributions_come_from_box_12,
//   saver_is_suggested_only_inside_its_bands, care_row_opens_the_people_page_and_returns_to_the_hub,
//   removed_person_drops_their_education_row,
//   education_is_suggested_while_a_student_is_on_the_return; the M1/M2 fixtures and sequences
//   now pay one more Next (the credits hub) and no field.
// Coverage: M3 more credits (unit M3-P12, 2026-09-25).
//   The credits hub's last topic (page and hub both CREDITS_PAGE; the sequence, section numbers
//   and hub previous/allowed_sections are unchanged from M3-P11). MoreCredits is one list
//   (credits: Vec<MoreCredit>, display.json promotes "More credits"/"Credit" so items read
//   "Credit #1", #[libertas_unordered], max 12); MoreCredit keeps the chooser (more than four
//   variants). Each variant carries an existing FederalCredits field's type unchanged onto the
//   list (more_credit_items/apply_more_credits, interview.rs): MinimumTax(MinimumTaxHistory,
//   8801), MortgageCredit(FederalMortgageCredit, 8396), PriorCreditCarryovers
//   {residential_energy_carry, dc_homebuyer_carry} (5695/8859 carryovers, one item for both),
//   CleanVehiclesAndRefueling {clean_vehicles, personal_refueling} (8936/8911, one item for
//   both), Adoption {adoptions, adoption_carry} (8839), ElderlyOrDisabled {exempt_pensions}
//   (Schedule R), ForeignTax(ForeignTaxClaim) (Form 1116; carries the whole existing election —
//   No/Statements/Ordinary — unchanged, so the chooser's own "Credit for ordinary foreign income
//   using Form 1116" choice is how a claim above the $300/$600 payee-statement election routes
//   there by name; extended_rules.rs::foreign_credit already refuses an over-the-election
//   Statements claim by name, unchanged here, per the existing
//   small_foreign_tax_becomes_the_statement_election boundary test, which still expects that
//   over-the-cap Statements is stored, not save-time rejected), TaxCreditBonds
//   {tax_credit_bonds} (8912), ElectricVehicleCarryover {legacy_ev_passive_allowed} (8834),
//   EmployerTip(EmployerTipFicaCarryforward) (8846), BbaAuditAdjustment(FederalBbaAuditInterest)
//   (8978), EducationRecapture(PriorYearEducationCreditRecapture) (2025 recapture). The mapping is
//   two-way: opening the page lists every field that is Some/non-empty/not-No; saving clears
//   every one of the twelve fields, then applies each submitted item (a repeated variant of the
//   same kind keeps the last one; the chooser makes that unlikely). The hub's row is always
//   shown ("None" until something is added, "N credits" after; More credits is never suggested,
//   unlike education and the saver's credit) and the review's "Credits" count now includes it.
//   SaveMoreCredits/EstimateMoreCredits added with the shared M1 Save-variant block; every
//   #[libertas_next_response] list gains SaveMoreCredits. Removed: RareSituation::OtherCredits
//   (ALL shrinks to 44; federal_rules.rs). Its own twelve credit types move to this page; the
//   residual "other 3800 credits, 6e, 6z" scope and Form 8862 (claim_repayment.rs) had no
//   situation besides OtherCredits and are now unrefused — a gap outside this card's scope,
//   flagged in federal_rules.rs's Schedule 3 Coverage block and claim_repayment.rs, not fixed
//   here. STORAGE_FORMAT 36. Tests: src/tests/credits/interview.rs each_more_credit_variant_reaches_its_field,
//   foreign_tax_above_the_election_points_to_form_1116.
// Coverage: M3 federal payments (unit M3-P13, 2026-09-25).
//   Sequence: ... the credits hub, the federal payments page (section 8: "section 7 of 9", 8 of
//   10), review; the credits hub's Continue now opens it (after_credits = Payments), resume and
//   the review's Go to stop there until it is saved (first_open), and the review gains a
//   "Federal payments" row (payments_count: dated payments plus an extension payment; "No
//   payments"). SavePayments (Payments: withheld, estimated, extension, more) opens by
//   EditRequest with page Payments, previous the credits hub, complete the first time (no
//   payments, IrsFigures); a first entry now requires an explicit payment method. withheld is
//   Form 1040 lines 25a/25b/25c/25d of the accepted return (whole dollars in cents), derived when
//   the page is built and ignored on save. FederalPayments retains the selected estimated method
//   and its exact source parts without a stored total; extension; IrsFigures clears prior_year and every Form
//   2210 fact (withholding_by_date/periods, annualized/annualized_income, farmer answers, waiver);
//   Figure writes PriorYearTax { full_year, tax, agi, spouse_tax (box E, SeparateReturns only) },
//   WithholdingTiming::ByPeriod onto withholding_by_date + PeriodWithholding (box D), IncomeTiming
//   ByPeriod onto annualized = true keeping annualized_income (Evenly erases it), Farmer::Yes onto
//   farmer_fisherman_prior_year Yes + farmer_paid_by_march (T2-49b: Farmer::ThisReturn onto
//   farmer_paid_by_march alone), Waiver onto PenaltyWaiver (boxes A/B);
//   home_kerosene is kept. The reverse mapping (penalty_item) reopens only shapes this page can
//   write. Rules: WITHHOLDING_PERIODS_NOT_TOTAL on both sides (the Payments client rule against
//   withheld/total; the server against the return's own line 25d, never the echoed value);
//   PAYMENT_DATE_OUTSIDE_2026 (January 1, 2026 through April 15, 2027) as the date field's own
//   range on DatedPayment.date (underpayment.rs; the rule language has no date literal) and on
//   the server with the key; box E only on a joint return (allowed_prior_returns) and the farmer
//   choices by the 2026 two-thirds test (allowed_farmer, T2-49b: ThisReturn alone when this
//   return meets it, else No and Yes, for 2025), and the 2027 Form 1040-ES farmer choices
//   (allowed_farmer_next_year, T2-49c: FarmerNextYear::NotNeeded alone when 2026 qualifies, else Yes, No
//   only; next_year_item reopens a stored answer inside that set, and a NotNeeded answer that no
//   longer applies through required WorkflowRequest construction), all checked again on save; a waiver needs an
//   explanation (PenaltyWaiver::valid). Walk-owned data: payments entered for a joint return
//   (FederalPayments.joint) are erased off a joint return, box E with them; one filer's reopen
//   the page on a joint return (T2-49b removed the business-removal reset of the farmer answer).
//   Form 2210-F (unit 2210-F, 2026-10-08): the farmer question comes before the timing questions
//   and a farmer or fisher (Yes or ThisReturn) must keep withholding and income Evenly, the
//   Payments client rule FARMER_EVEN_TIMING with its server twin beside
//   WITHHOLDING_PERIODS_NOT_TOTAL (checked before allowed_farmer); a stored box C or D answer on
//   a farmer return is not used (NOTE_2210F_PERIOD_ANSWERS_NOT_USED). full_year and both
//   paid_by_march_1 answers carry no Default (an explicit Yes/No). Still refused by name in
//   underpayment.rs on Form 2210 (not Form 2210-F), unchanged: a payment January 16-31, 2027.
//   Box C: IncomeTiming::ByPeriod is stored and next_page(PAYMENTS_PAGE)
//   opens the income by period page (M3-P14 below); until its periods are saved the estimate
//   refuses box C by name (Schedule AI needs the periods).
//   Removed: RareSituation::Payments (federal_rules.rs ALL shrinks to 43; the situations page no
//   longer shows or writes payments, superseding the M1-P3 block's Payments situation).
//   STORAGE_FORMAT 37. Tests: src/tests/payments/interview.rs dated_payments_total_the_estimated_payments,
//   irs_figures_leaves_no_prior_year, box_d_periods_must_total_the_withholding,
//   box_e_is_offered_only_on_joint, farmer_is_offered_only_with_a_business,
//   payment_date_outside_2026_is_rejected,
//   farmer_2027_choices_follow_the_2026_test_and_are_checked_on_save (T2-49c, STORAGE_FORMAT
//   58); payments_situation_reaches_the_draft removed (its
//   assertions moved to dated_payments_total_the_estimated_payments).
// Coverage: M3 income by period (unit M3-P14, 2026-09-25).
//   The page after the payments page when box C is chosen (next_page(PAYMENTS_PAGE) while
//   FederalPayments.annualized); it belongs to the payments section (progress "section 7 of 9",
//   8 of 10; rank = PAYMENTS_PAGE), page TaxSection::IncomeByPeriod (33), previous the payments
//   page, and is refused ("This section is not available yet.") on open, save and Go to without
//   box C. SaveIncomeByPeriod/EstimateIncomeByPeriod (IncomeByPeriod: through_march,
//   through_may, through_august entered; full_year read-only) with the shared M1 Save-variant
//   block; every #[libertas_next_response] list gains SaveIncomeByPeriod. AnnualizedPeriod
//   (underpayment.rs) keeps its labels and gains #[libertas_default(0)] on every field but AGI.
//   Column (d) is derived (full_year_period) from the accepted return without Form 2210
//   (return_before_penalty, so a box C still waiting for its periods cannot hide it): AGI = Form
//   1040 line 11, Schedule 1-A = line 13a, QBI = line 13b, and three calculation lines added to
//   federal_rules.rs for figures the return does not show: schedule_ai_full_year_itemized (line
//   12e when itemizing, else 0), schedule_ai_full_year_self_employment and
//   schedule_ai_full_year_social_wages (Schedule SE net profit before the 92.35% reduction and
//   the Social Security wages of the person with self-employment tax). It is ignored on save
//   and re-derived on every accepted change (follow_full_year in reconcile), so the stored
//   Schedule AI record always reconciles to the return. The first open (nothing stored) starts
//   every column at the full year spread evenly (3/12, 5/12, 8/12, even_period): those
//   annualize back to the full year, so an untouched page never lowers the penalty below the
//   regular method, where zeros would silently understate it; the page's help says to replace
//   them. Resume and the review stop at the page while box C has no periods (first_open), the
//   review gains an "Income by period" row ("Not answered yet"/"Entered") and allowed_sections
//   the page, only with box C. Saving stores FederalPayments.annualized_income (checked with
//   AnnualizedIncome::valid) and continues at the review. Erasure: IrsFigures and box C back to
//   Evenly erase the periods (payments_record, unchanged). The payments page's withheld now
//   also reads the return without Form 2210. No client rule is added; the Schedule AI
//   refusals (itemized periods without itemizing, column (d) mismatch such as a standard
//   deduction with a qualified disaster loss or the non-itemizer cash-gift deduction, which
//   Schedule AI does not subtract) stay by name in underpayment.rs. STORAGE_FORMAT 38.
//   Tests: src/tests/payments/interview.rs income_by_period_opens_only_for_box_c,
//   full_year_column_is_the_return, box_c_back_to_evenly_erases_the_periods;
//   irs_figures_leaves_no_prior_year now expects box C to open the page.
// Coverage: Form 8379 page (2026-09-27).
//   The spouse page's item SpouseSituations::INJURED_SPOUSE (a joint return only) plans the
//   page TaxSection::InjuredSpouse (41; Review is
//   42), a walked section of its own after payments (WALKED_SECTIONS, page_section,
//   first_open), until answered; it is in allowed_sections and "Your answers" only while
//   ticked, and is listed "Not answered yet" until saved. SaveInjuredSpouse/EstimateInjuredSpouse
//   store FederalDraft.injured_spouse; joint_estimated and estimated_share are derived and
//   ignored on save; INJURED_SPOUSE_ESTIMATED_ABOVE_PAYMENTS on both sides. The item is no
//   longer refused. Calculation and refusals:
//   injured_spouse.rs. Tests: src/tests/attribution/interview.rs injured_spouse_item_opens_its_page_and_round_trips,
//   injured_spouse_item_is_offered_only_on_a_joint_return,
//   injured_spouse_agreed_payments_are_checked_on_both_sides.
// Coverage: earlier-year losses and Form 8615's parent (unit u5-losses, 2026-09-28).
//   The kind IncomeRecord::NetOperatingLoss on Self-employment and Farming (loss_answer,
//   DataAnswers::SelfEmployment/Farming.losses, data_answer_items) onto screening.nol/atnol;
//   RareSituation::NetOperatingLoss removed. The page TaxSection::ParentTax (PARENT_TAX_PAGE,
//   rank and section Other taxes, after that page: next_page, first_open parent_missing,
//   allowed_sections, "Your answers", a "Not answered yet" remaining item unless an estimate
//   issue names it) while parent_tax_in_walk; SaveParentTax/EstimateParentTax
//   (parent_tax_candidate) store FederalPeople.parent_tax. Issues "Regular/AMT loss history"
//   open the business topic, "Investment tax" about the parent's return opens the page.
//   STORAGE_FORMAT 73. Tests: src/tests/federal/losses.rs, src/tests/cross/change_audit.rs (young households, changes
//   YouAdult, InterestUp, ParentUnsettled, NoLosses).
// Coverage: M3 every situation finds its hub (unit M3-P15, 2026-09-25).
//   RareSituation carries no data: 60 variants declared by hub and topic in the interview map's
//   order (28 less common income, 5 more adjustments, 1 deductions page, 3 more itemized, 11 more
//   credits, 7 other taxes, 5 "Anything else?"), each with destination() -> (TaxSection, topic);
//   federal_rules.rs ALL/refusal follow (43 -> 60), and refuse_situations files each issue under
//   its page's name (section_name). Renames and splits: PartYearResident -> DualStatus,
//   ForeignIncome -> WorkAbroad, PriorYearCarryovers -> NetOperatingLoss, TipsOvertimeCarLoan ->
//   CarLoanInterest + TipsNotOnW2, ElectivePaymentOrSection965 -> ElectivePayment + Section965;
//   SomethingElse retired; OtherIncome/OtherAdjustments/OtherItemizedLines narrowed. The data
//   answers are IncomeSource kinds onto FederalScreening (data_answer_records/items):
//   RealEstateProfessional (one per item owner; T2-28 2026-09-26: its ProfessionalRentals
//   over_500_hours answer writes niit_safe_harbor, the Form 8960 line 4b safe harbor; offered on
//   Rentals only with a stored rental
//   and refused without a Rental on the page: PROFESSIONAL_NEEDS_A_RENTAL, PROFESSIONAL_TWICE;
//   Professionals removed), ForeignAccountOrTrust (account/country/trust onto Schedule B Part III,
//   FOREIGN_ACCOUNT_TWICE), CapitalLossCarryover (PriorCapitalReturn onto capital_history, at most
//   two: CAPITAL_LOSS_CARRYOVER_LIMIT), QbiLossCarryforward (onto QbiHistory::Prior,
//   QBI_LOSS_CARRYFORWARD_TWICE); each topic page is the whole answer for its kinds. Folds,
//   default "Nothing else": Filer.more (MoreAboutYou::Died { date, estate_medical, bonds,
//   ordinary } onto FederalPerson.died, a final return the estimate calculates since unit B9-3,
//   2026-10-10) and Dependent.facts
//   (PersonFacts Bitflags, asked once since gate fixes batch A: stored as the bits themselves in
//   FederalDependent.facts and read through its accessors: ABROAD (the citizen or resident
//   test), FILED_JOINTLY, the competing claim now FederalDependent.competing (CompetingClaim:
//   No one / You with the tiebreaker basis / The other person, section 152(c)(4)),
//   YOU_RELEASED / RELEASED_TO_YOU (Form 8332), MULTIPLE_SUPPORT (Form 2120),
//   MARRIED_AT_YEAR_END and OWN_SUPPORT (section 152(c)(1)(D)), with the page's situation stored
//   as FederalDependent.situation); FinalReturn, DependentAbroad, DependentFilesJointly and CompetingClaim left
//   the enum. Deductions.car_loan is stored as the CarLoanInterest situation. The situations page
//   gains topic (hidden, read-only, CacheIdentity; allowed_professionals removed) and serves
//   TaxSection::MoreIncome/MoreAdjustments/MoreItemized (topics of their hubs, page = the hub,
//   save returns to the hub, row "Farming, Gambling winnings — not calculated yet" once chosen,
//   in allowed_sections), OtherTaxes (a section page between the credits hub and the payments
//   page: after_credits, first_open, rank, progress "section 7 of 10"/8 of 11, payments now 8,
//   review 10; saving records OtherTaxes; review row "Other taxes") and Situations (the whole-
//   return five). Saving replaces only that page's situations (replace_situations, declaration
//   order). More credits appends eleven payload-less refusal kinds (PriorYearCare .. OtherCredit)
//   stored as situations (CREDIT_REFUSALS), closing the M3-P12 finding: Form 8862 is
//   EicAfterDisallowance and the residual Form 3800 credits BusinessCredits (6e is
//   6z OtherCredit; 6e AlternativeMotorVehicle was removed 2026-10-09); list max 24. STORAGE_FORMAT 39. The M1/M2 fixtures
//   keep 12 and 20 fields and pay one more Next (M1 12 requests, M2 11). Tests: src/tests/cross/interview.rs
//   every_situation_has_one_destination,
//   less_common_income_saves_only_its_hubs_situations,
//   real_estate_professional_is_a_rental_kind_offered_only_with_a_rental,
//   foreign_account_kind_answers_schedule_b_part_iii, capital_loss_carryover_kind_reaches_schedule_d,
//   qbi_loss_carryforward_kind_reaches_form_8995, final_return_page_maps_the_stored_final_return_both_ways,
//   person_fold_maps_citizenship_and_joint_return, car_loan_interest_is_refused_by_name,
//   refusals_name_their_topic, m1_and_m2_fixtures_keep_12_and_20_fields.
// Coverage: M3 review remaining items (unit M3-P16, 2026-09-25).
//   Review gains remaining_items (read-only, SuppressLabel, max 64) above "Your answers" (the
//   former `remaining`, renamed `answers`) and allowed_actions under a response-level
//   ConstrainedBy: Navigate and link-only OpenSection always, OpenInterview only for an
//   unfinished interview step, StartFinish only when the list is empty;
//   StartFinish and Finish are refused while any item remains (as well as for an estimate outside
//   the supported scope). One producer, remaining_items(draft, estimate), lists each item once as
//   a ReviewItem (page name, what is left, hidden section) in SEQUENCE order (each hub followed by
//   its topics): refused situations named by RareSituation::destination().0 ("Farm income
//   (Schedule F) is not calculated yet" on "Less common income"; the estimate's own copy of each
//   situation refusal is skipped); every other estimate issue through the issue-to-page table
//   issue_page ("Income: <label>" opens that document's topic, so the statutory W-2, the
//   special-treatment flags and the unsupported 1099-R codes point at Wages, Interest and
//   dividends and Retirement; "Home office" and "Business" (the specified-service answer above
//   the QBI threshold) at Self-employment; "HSA" at the HSA page; "Filing status" and "Filers",
//   the household check, at page 1 and About you; "Form 2210 underpayment penalty" at Federal
//   payments, or at Income by period for a Schedule AI message under box C; "Interview" at the
//   first unfinished section; a page's own name at that page; anything else on the review
//   itself); and facts a later answer made necessary: payer names on Schedule B above $1,500
//   (an unnamed 1099-INT/OID while the estimate emits schedule_b_taxable_interest, an unnamed
//   1099-DIV with schedule_b_ordinary_dividends), a W-2 box 12 W owner with no HSA
//   (HSA_ACCOUNT_NEEDED), and a stored WithinLimit mortgage above the current limit (the
//   MORTGAGE_AVERAGE_NEEDED text, e.g. after a change to a separate return). Review.suggested is
//   the first item's page that is an allowed section, else the first unfinished section. The
//   hubs (income, adjustments, itemized, credits) show a topic's first item as its row,
//   "Needs attention: <the item's text>", listing the topic even with nothing entered
//   (mark_attention); the HSA row's two fixed texts are replaced by it; less-common rows are
//   unchanged. ReviewItem.status max rises to 4000 (issue texts run to about 2,900
//   characters). "Your answers": a passed section with nothing in it reads "Nothing entered"
//   (was "No documents", "No adjustments", "No payments"). Not implemented, owned by M3-P18
//   and M3-P19 which have not landed: planned topics with nothing entered (no
//   FederalDraft.planned) and the "Your 2026 at a glance" row; "skipped" sections are the
//   sections passed with nothing entered. No stored type changes: STORAGE_FORMAT stays 39.
//   The M1/M2 fixtures keep 12 and 20 fields and 12 and 11 requests; their lists are empty and
//   Finish is offered. Tests: src/tests/cross/interview.rs each_issue_points_at_its_page,
//   refused_situations_name_their_topic, hub_attention_and_review_item_share_one_text,
//   finish_is_offered_only_with_nothing_remaining, review_suggests_the_first_item,
//   skipped_sections_read_nothing_entered, payer_names_are_asked_only_above_1500;
//   m1_and_m2_fixtures_keep_12_and_20_fields, adjustments_hub_suggests_hsa_when_box_12_w_exists
//   and each_more_credit_variant_reaches_its_field amended.
// Coverage: Beyond your documents (TaxSection::Checklist, SaveChecklist/EstimateChecklist).
//   The page after the household pages asks three Bitflags questions with "None of these"
//   (Checklist: ActivityTopics, PaidTopics, CarryoverTopics), only what no document shows.
//   FLAGS gives each box its page, its hub-row words and the data it owns (`entered`, never a
//   document; `clear`, as saving the page without it); gifts to charity are two flags of one box
//   (cash on the deductions page, property on the gifts page). The page opens with every box
//   checked, every topic with entries and every box a document evidences (checklist_value;
//   evidence: a 1099-NEC or 1099-K for a business of your own, a 1099-PATR or a 1099-G box 7
//   agriculture payment for a farm), with a note naming each such document (from_documents).
//   Save stores the answer without the evidenced boxes, so a document is never the person's
//   answer and removing it unchecks its box. A box with entries stays checked on every other
//   page's save (follow_ticks); unchecking one removes its entries (prune_unticked) unless a
//   document keeps its topic on the path; documents are never removed. Planned = checked with
//   nothing entered: the walk opens its page (an itemized topic on the deductions page while
//   not itemizing; the state and local taxes and charity carryover boxes open the deductions
//   page on Compare, ITEMIZABLE). The payments page is walked only with its box (estimated or
//   extension payments, or last year's refund applied); a return without it holds no payments
//   (skip_empty_sections). The refund choices (Form 1040 line 36 and its account) are asked on
//   the review's finish step (Finish.refund: refund_choice, refund_candidate), and the
//   payments page keeps them. Hubs: first row "What you told us" (told_us). The topics whose
//   documents bring them (wages, interest, sales, retirement, unemployment, other income,
//   gambling, K-1s, canceled debt, mortgage, tuition, student loans, Marketplace, stock options)
//   are reached from their hubs and the documents; their former options, and those now asked on
//   Situations people often miss, are mapped in src/tests/cross/interview.rs
//   every_option_of_the_old_checklist_has_its_home.
//   Tests: src/tests/cross/interview.rs beyond_your_documents_* and the sequence tests.
// Coverage: M3 guided walk, summaries and skipped sections (unit M3-P19, 2026-09-25).
//   TaxSection is declared in sequence order, each hub followed by its topics (Setup 0 .. Review
//   38, SEQUENCE's order), so a hub's popup reads the previous section, its topics, then the next
//   section (the checklist row and the credits hub's care row, which reopen earlier pages, sort
//   first); rank() is the discriminant (income by period ranks as payments). STORAGE_FORMAT 41
//   (TaxSection is stored in finished_sections). The walk (design (b) A'): walk_topics = a hub's
//   planned (planned_sections) or derived (derived_topics) topics not yet visited, in hub order;
//   enter(hub) opens its first one directly, else the hub. The sequence enters a hub through
//   next_page (checklist, deductions page), resume and Continue; a topic save returns through
//   enter, so it opens the next walk topic, else the hub; enter never opens a page beyond the
//   first unfinished one (it resumes there instead), so a jump ahead skips no step; Back and
//   Go to show the hub. The hub
//   whose walk has ended is the section summary: progress "... · You're done with income",
//   suggestion attention first, else Continue. Visited rule (the rediscovered gap: income and
//   adjustments topic saves recorded nothing): visit() records a topic in finished_sections when
//   it was planned or derived before the save (income, adjustments, the less-common pages;
//   itemized and credits topics already recorded); other saves record nothing, so an unchanged
//   Next stays unchanged. Derived: HSA only while an owner has box 12 W and no account (a stored
//   special answer is attention, not re-walked); itemized topics are walked only when planned
//   (M3-P9 walked every unvisited one), so hub_suggestion is attention, walk, next. Skipping:
//   sections_with_needs (shared with checklist_path) shows a section with a tick, an entry, a
//   derived topic or a remaining item (items of the return with income continued); the itemized
//   hub whenever the route itemizes; credits entries exclude the care row (answered on the people
//   page). skip_empty_sections runs in commit on every accepted change: from first_open, each
//   section with no need is recorded in finished_sections, in the same revision. Continue and
//   Back use next_shown/previous_shown (hub previous, allowed_sections' first and last entries,
//   the deductions, other taxes and payments pages' previous). The "Anything else?" page and
//   its walk step were retired in gate fixes batch A; the injured spouse page is the walked
//   section after payments while its tick is planned. Progress (progress_line) counts setup, you, people (with dependents), the
//   checklist, the shown sections, the page's own section and the review. Income hub rows carry
//   count and amount, "1 W-2 · $60,000.00" (topic_summary: the documents' previews; a business
//   shows the count only). Not done, reported: adjustments and credits rows keep their counts
//   without amounts; a section already recorded is not reopened when a later change gives it a
//   need (un-finishing it would lock every later page through reached()); the review's remaining
//   items and the hub attention cover that case. Fixtures: M1 6 requests (was 13), M2 6 (was
//   12); navigation taps 6 and 5 on 2 and 2 screens; 12 and 20 fields unchanged. Tests:
//   src/tests/cross/interview.rs m1_path_is_checklist_wages_interest_summary_review,
//   m2_path_skips_the_credits_hub, walk_follows_planned_topics_in_hub_order,
//   derived_hsa_and_education_are_walked_without_a_tick,
//   empty_sections_are_skipped_and_listed_on_the_review, a_section_with_attention_is_never_skipped,
//   revisit_from_the_hub_returns_to_it, progress_counts_only_shown_sections,
//   back_passes_skipped_sections, resume_opens_the_next_step, popup_lists_previous_topics_then_next;
//   section numbers in tests are PAGE_* constants; sequence, suggestion and progress tests amended.
// Coverage: M3 tappable hub and review rows and task-owned hub actions (2026-10-02).
//   ReviewItem declares link-only OpenSection: a row sends the bound section in one tap.
//   Each hub offers its own Continue request until first_open reaches Review, then
//   ReviewAccepted. The four Continue requests use their existing after_* destinations,
//   record the section's completion, and enter the next shown section. They validate
//   cookie/revision and eligibility; they never submit another editor's values.
//   The template's single_action caption follows the offered request's localized Name.
//   Hub.next is removed; topic rows and a leading previous-section row retain navigation.
//   Review keeps Navigate, OpenSection, conditional OpenInterview and conditional Finish.
//   No stored type changes: STORAGE_FORMAT stays unchanged. Fixtures invoke hub actions;
//   tests cover all four section paths, completion/reopening, stale and forged requests,
//   row links, and independent leading-page navigation in the master-detail template.
// Coverage: M3 review lists every form's lines (acceptance fix, 2026-09-26).
//   The M3-P17 phone run showed only Form 1040 in the review's "Calculated draft return". The
//   review now replaces result.return_lines with draft_return (federal_model.rs, beside
//   form_coverage): the typed return lines (every FederalAmount with a form_line, today Form
//   1040's), then every CalculationLine with a form_line not already a typed line, labelled with
//   its calculation id; two calculation lines on one form line (two businesses' Schedule C) are
//   both listed. Rows read "<form short name> <line> — <label>" through FormLine::row_label, the
//   one builder the 1040 rows now use too (the FORM_ name resources: "1040", "Schedule C", "Form
//   8995" ...). Order: Form 1040, Schedules 1, 2, 3, A, B, C, D, E, SE, then the other forms in
//   TaxForm declaration order (review_group), the return's line order within a form. Form 1040
//   keeps its $0.00 lines; the other forms' zero lines are dropped. estimate() and the running
//   estimate on the other pages are unchanged (Form 1040 only); return_lines gains
//   libertas_size(max = 256) and draft_return truncates to it; its help text names the review's
//   other forms. Rows: M1 54, M2 56 (Schedule 8812 27, Form 2441 11), M3-F1 56 (Schedule C 9,
//   Form 4562 19i), M3-F2 60 (56 Form 1040 with the line 35a/38 penalty lines, Schedule A 8a,
//   8c, 8e, Schedule 8812 14). No stored type changes: STORAGE_FORMAT stays 41. Tests:
//   src/tests/cross/interview.rs review_lists_schedule_lines_grouped_by_form,
//   review_hides_zero_lines_outside_form_1040; review_lists_answers_with_actions amended. The
//   M3-F1 and M3-F2 drafts moved into helpers (src/tests/business/business.rs m3_self_employed_rental_draft,
//   src/tests/deductions/deductions.rs m3_itemizing_homeowner_draft) shared with these tests.
//   Line tagging (acceptance fix, 2026-09-26): amounts unchanged; calculation lines now carry
//   form lines. Schedule C per ordinary business (federal_rules.rs schedule_c_lines, the category
//   lines and home office): 1, 4 (COGS), 6, 7, 8-27b by category, 9, 13 (asset bonus + MACRS +
//   allowed section 179, section179::allowed), 28, 29, 30, 31 (new business_<id>_net_profit,
//   the amount business income adds); a professional gambler shows only 31, a farm none.
//   Schedule SE per person with SE income (schedule_se_lines, ids taxpayer_/spouse_): 2 (not with
//   a farm), 3, 4a, 4c, 12, 13; the merged self_employment_tax stays untagged. Form 8995 at or
//   below the threshold, else Form 8995-A (qbi_lines, business_rules.rs QbiLines): 8995 1 per
//   activity (business_<id>_qbi, else qbi_activity_<n>), 2-16, 17 = qbi_deduction, 18/19 the
//   carryforwards; 8995-A 2, 3, 15 per column (none when aggregation merged activities), 27-37,
//   39, 40, 41 = qbi_deduction, 42 and Schedule C 6. Schedule E per property without personal
//   use (extended_rules.rs): 3, 5-19 by category, 18, 20, 21, 22 (allowed loss after Form 8582),
//   and 26 = each property's income or allowed loss plus royalties (RentalResult.part_one).
//   Schedule 1, each only when not zero: 3 (non-farm Schedule C), 5 (rental.income + royalties),
//   6, 8z (grants, family leave, living-expense insurance, 1098 refund), 10 (= Form 1040 line 8,
//   now one additional_income value), 11, 13, 15, 16, 17, 18, 19a, 20, 21, 26 (= 1040 line 10).
//   draft_return lists a form one copy at a time (business_<n>, rental_<n>, taxpayer, spouse, in
//   first-reached order) and in printed line order (FormLine::order). Rows: M1 54, M2 56, M3-F1
//   105, M3-F2 62 (Schedule 1 13 and 26 added). Tests: src/tests/cross/interview.rs
//   review_lists_schedule_lines_grouped_by_form (rewritten), schedule_1_lines_are_tagged;
//   src/tests/business/business.rs joint_return_tags_each_spouses_schedule_se. Not tagged, reported: Schedule
//   E line 4 (royalties are a net amount), per-property lines with personal use, Schedule F,
//   Schedule SE line 2 with a farm, Schedule 2 line 4.
//   Form 4562 placement and remaining tags (acceptance fix, 2026-09-26): amounts unchanged.
//   Form 4562 is tagged only for an activity that must file it (i4562 "Who Must File"):
//   depreciation.rs assets_need_form_4562 (an asset placed in service in 2026, a section 179
//   election or carryforward, a vehicle; amortization has Form 4562-B instead, line audit D11);
//   a rental only with a building placed in service in 2026. Otherwise the depreciation
//   is on Schedule C line 13, Schedule F line 14 or Schedule E line 18 alone. Per asset
//   (depreciation.rs form_4562_line): bonus Part II 14a; MACRS of an asset placed in service
//   before 2026 line 17, of a 2026 asset 19a-19f by class, 19j for 39-year; vehicles stay in
//   Part V. Per business (federal_rules.rs form_4562_totals): 12 business_<id>_section179_deduction
//   and 22 business_<id>_depreciation_total (= Schedule C 13, federal_rules.rs
//   business_depreciation). Per rental: rental_<id>_depreciation is line 22 and, without a
//   personal-use split, rental_<id>_building_<n>_depreciation 19i/19j (2026) or 17 (earlier).
//   Schedule 2: 4 self_employment_tax, 6 net_investment_income_tax, 11
//   additional_medicare_self_employment, 17b additional_medicare_wages. Schedule SE with a farm:
//   1a Schedule F profit, 1b the CRP exclusion (negative), 2 the nonfarm rest of line 3. Schedule
//   F (farm.rs, copy farm_<id>): cash 1a resale proceeds, 1b resale basis, 2 raised sales, 9, 14
//   (new farm_<id>_depreciation), 33, 34. Schedule E with personal use: 3, 18 (the rental share
//   after Worksheet 5-1), 20, 21; lines 5-17 stay untagged (the categories are whole-property
//   amounts; the rental share is divided only in total). Schedule E 4: each royalty property's
//   gross receipts (royalty_<id>_line4_royalties, new line). draft_return's copy_of also knows
//   farm_<n> and royalty_<n>. M3-F1 rows: Schedule 2 4 added; Form 4562 is the computer's 14a
//   and 22 ($2,400), no rental row. Tests: src/tests/cross/interview.rs
//   review_lists_schedule_lines_grouped_by_form (amended); src/tests/business/business.rs
//   prior_year_building_needs_no_form_4562, current_year_asset_lists_its_form_4562_lines,
//   schedule_2_line_4_is_the_se_tax; src/tests/farm/farm.rs
//   farm_profit_is_schedule_se_line_1a_and_schedule_f_lines_are_tagged; src/tests/home/investment.rs
//   personal_use_home_lists_its_schedule_e_rental_share,
//   part_iii_lines_19i_and_19j_are_emitted and
//   commercial_rental_is_no_longer_refused_and_residential_is_unchanged (amended). Not tagged,
//   reported: Schedule F accrual Part III and the combined other-income rows; Form 4562 lines
//   21 and 1-11; royalty expenses and K-1 royalties on Schedule E; Form 8995-A per-column rows
//   under aggregation; Schedule 2 lines 1b, 1c, 5, 13b-13f, 16a-17c, 18.
// Coverage: T2-27p Form 1095-A page (2026-09-27).
//   TaxSection::Marketplace (after Saver, before MoreCredits; renumbers the later sections,
//   STORAGE_FORMAT 59) is a credits hub topic, always offered, between the saver's credit and
//   more credits; SaveMarketplace/EstimateMarketplace with the shared Save block and the hidden
//   allowed_sharing (FormerSpouse off a joint return, Spouse only on a separate one),
//   allowed_separate ([No, Yes] on a separate return, else [NotSeparate]) and
//   dependents list (gate fixes batch B: a list that may be empty, rows only for people
//   supported); the dependents catalog is Marketplace.people (hidden, in the page value), stale persons appended for removal.
//   Marketplace (federal_model.rs) is policy-major; marketplace_claim transposes it onto
//   MarketplaceClaim::Yes (one MarketplaceMonth per covered month, policies in form order, issuer
//   as label, sharing inside from..=to as the MarketplaceAllocation, eligibility from
//   other_coverage: NoOne, SomePeople, else Everyone (MarketplaceEligibility); separate
//   NotSeparate as absent so a later separate return asks line A again; the
//   MarketplaceSituations ticks as special in declaration order); an empty forms list is No and clears marketplace_dependents; dependents
//   map onto the students index (set_students). marketplace_value reverses it: AllYear when a
//   form covers twelve equal months with no partial share; a stored shape it cannot write closes
//   the page by name. C1: column B is the one nonzero amount; FamilyBenchmark::Differ gives the
//   family benchmark for months two forms differ (MARKETPLACE_FAMILY_BENCHMARK_NEEDED /
//   _NOT_NEEDED, server); only that benchmark is stored, so the forms reopen showing it. C4: one
//   shared range per form. Rules both sides: MARKETPLACE_ADVANCE_ABOVE_PREMIUM,
//   _ADVANCE_ON_UNSUBSIDIZED_SHARE, _SHARE_MONTHS_REVERSED, _TAXABLE_SOCIAL_ABOVE_GROSS, and
//   _ISSUER_TWICE, _MONTH_TWICE, _PERSON_TWICE (client #[libertas_unique(1)]); server only
//   _SHARE_OUTSIDE_POLICY, _GAP_MONTH_NOT_COVERED and the allowed sets. Hub and review row
//   "1 Form 1095-A · credit $X" / "· repay $X" (marketplace_status); issue_page "Marketplace";
//   the checklist's Marketplace tick plans this topic. Removed: RareSituation::Marketplace (ALL
//   59), MoreCredit::Marketplace and its CREDIT_REFUSALS entry. C3: SeHealth gains the Form
//   1095-A tick (health.rs counts those premiums once). Tests: src/tests/health/interview.rs
//   marketplace_page_reproduces_the_row_27_fixture, marketplace_forms_transpose_onto_months_and_back,
//   marketplace_family_benchmark_is_asked_only_when_forms_differ,
//   marketplace_rules_reject_on_both_sides_at_their_boundary, marketplace_choices_follow_the_return,
//   marketplace_dependent_returns_reach_household_income,
//   marketplace_special_situations_and_mixed_months_are_refused_by_name,
//   marketplace_months_on_the_self_employed_page_count_the_premiums_once.
// Coverage: M2 people page (unit M2-P2, 2026-09-24).
//   Page 3 "Children and people you support" (Dependents: a list of Dependent rows, each a
//   projection of one FederalDependent: names, birth date, Relationship, LivedWithYou (all year
//   or months), DependentSituation (child under 19, student, disabled, another person you
//   support with support and gross income), Identification, CareCosts). Asked on every return
//   after the filer pages since gate fixes batch A (an empty list is "no one"; it was sent only
//   while page 1's has_dependents answer was true); BeginChildren
//   opens an empty list, SaveChildren saves it, EstimateDependents is the page's running
//   estimate. Client rules SUPPORTED_NEEDS_YOUR_SUPPORT, the five Forms 8332/2120 rules,
//   DEPENDENT_AGE_SITUATION and CARE_NEEDS_UNDER_13_OR_SELF_CARE (DateOnly literals; the Bitflags
//   facts through `has`) each have their server twin in dependent_rule; DEPENDENT_OLDER_THAN_YOU
//   alone is server-only (it compares with the filers' birth dates). Any care cost opens the household
//   CareClaim (earnings Actual, no prior care, no benefits, eligible). Names reopen from the
//   stored first_name/last_name (M2-P3); label stays the joined name for existing readers.
//   Evidence: federal_model.rs Dependents/Dependent/LivedWithYou/DependentSituation/CareCosts,
//   FederalDraft.dependents; src/tests/family/interview.rs dependents_page_maps_onto_dependent_records,
//   care_costs_open_the_household_care_claim, dependent_rules_reject_on_save,
//   household_answer_is_stored_with_the_draft_and_reopened.
// Coverage: M2 care-under-13 cutoff aligned with the calculation (unit T2-30x, 2026-09-26).
//   CARE_NEEDS_UNDER_13_OR_SELF_CARE (dependent_rule) refused care costs for any child born in
//   2013 at all (cutoff BORN_BEFORE_AGE_13 = 20140101), one birth year stricter than the
//   calculation it gates: family() (federal_rules.rs) credits care given before a child's 13th
//   birthday, so a child who turns 13 during 2026 (born after January 1, 2013) still qualifies
//   for the part of the year before that birthday (c.birth_date > 20130101). dependent_rule now
//   shares that cutoff (BORN_AFTER_CARE_UNDER_13 = 20130101; born on or before that date is
//   refused, matching the calculation's exclusive bound), checked identically on save and in the
//   running estimate (dependents_candidate calls the same function for both). No stored type or
//   client rule changes: the rule was server-only then; the Forms 8332/2120 unit made it a client rule with a DateOnly literal, dependent_rule being its twin.
//   Evidence: federal_rules.rs family() c.birth_date > 20130101; src/tests/family/interview.rs
//   care_costs_are_accepted_for_a_child_turning_13_in_2026,
//   care_costs_are_refused_for_a_child_already_13_on_january_1;
//   dependent_rules_reject_on_save's CARE_NEEDS_UNDER_13_OR_SELF_CARE and boundary cases moved to
//   the new 20130101 boundary; src/tests/family/family.rs
//   joint_two_children_one_turning_13_with_employer_care_benefits (the fixture that found this).
// Coverage: M2 dependents' ages are day-exact (unit ages, 2026-10-08). The age tests used the
//   birth year alone (2026 - year, constants 20071231, 20021231, 20130101), so a person born on
//   January 1 was a year younger than the IRS counts: a person attains an age on the day before
//   the birthday (Pub. 501; Rev. Rul. 2003-72), so a child born January 1, 2008 is 19 on
//   December 31, 2026. federal_model.rs attained_on/age_on/age_at_year_end is the one rule for
//   filers and dependents; family() (152(c)(3), 24(c)(1), 21(b)(1)(A)), dependent_rule, the
//   long-term-care premium age (213(d)(10), medical_people) and the Trump-account, IRA and HSA
//   catch-up age tests use it. The birth-year constants are gone. Behavior changes only for a
//   January 1 birth, and for a January 2, 2013 birth under the Form 2441 under-13 test (13 on
//   January 1, 2026). Evidence: src/tests/family/family.rs dependent_age_* and
//   age_13_test_is_a_whole_year_cutoff_at_january_2_2013; src/tests/family/interview.rs
//   dependent_rules_reject_on_save (January 1 and January 2 boundaries).
// Coverage: M2 dependents' ages follow Rev. Rul. 2003-72 (unit family credits F8, 2026-10-10),
//   replacing the day-before rule of unit ages (2026-10-08) for the family provisions: a child
//   attains an age on the birthday for sections 21, 23, 24, 32, 129 and 151, so a child born
//   January 1, 2008 is 18 at the close of 2026 (DEPENDENT_AGE_SITUATION now allows born on or
//   after January 1, 2008, and a student on or after January 1, 2003), a child born January 1,
//   2013 is 13 on January 1, 2026 (CARE_NEEDS_UNDER_13_OR_SELF_CARE, unchanged on the client,
//   whose twin dependent_rule now tests the birthday) and a child born January 1, 2010 is 16 on
//   December 31, 2026. The day-before rule stays where the instructions write it: age 65, the
//   childless EIC age 25, Form 8814 and 8615, Form 8863 line 7, Form 8880's printed date and the
//   age-based catch-ups (federal_model.rs attained_on, age_at_year_end, birthday_age_at_close_or_death).
//   A child born and died in 2026 with no Social Security number counts for the child tax
//   credit, the earned income credit and Form 2441 (2026 i1040s8 "Death or birth of child"):
//   FederalDependent::born_and_died derives it from the 2026 birth date, the date of death and
//   the identification answer, so the page asks no separate fact. The fact ABROAD now reads "Not a U.S. citizen, national or
//   resident alien" (a citizen child who lived abroad keeps the child tax credit).
//   Tests: src/tests/family/interview.rs dependent_rules_reject_on_save; line_audit_family_credits.rs.
//   Unit B11 KIDS (2026-10-11): the person has an optional date of death (Dependent.death_date,
//   stored as FederalDependent.death_date): "All year" maps to the whole-year exception for a
//   person born or who died in 2026, family() tests the residence over the days alive, Form
//   8862 line 8 prints the date, and the born-and-died child is read from it (rule
//   DEATH_BEFORE_BIRTH in dependent_rule). The fact
//   PersonFacts::ANOTHERS_QUALIFYING_CHILD (another person you support who is another
//   taxpayer's qualifying child, section 152(d)(1)(D); rule ANOTHERS_CHILD_NEEDS_SUPPORTED)
//   takes the person out of the qualifying relatives, with NOTE_ANOTHERS_QUALIFYING_CHILD, and
//   ClaimBasis gains LongerResidenceThanTheOtherParent (section 152(c)(4)(B)). The Education
//   page's "Part of the box 5 grant the student includes in income" (StudentMore
//   .scholarship_in_income, stored on FederalEducationExpenses, rule
//   SCHOLARSHIP_INCLUDED_ABOVE_BOX_5 with its server twin in education_candidate) reopens as
//   chosen. Tests: src/tests/family/kids.rs.
// Coverage: M1 payments and situations (unit M1-P3, 2026-09-24).
//   No payments page: page 8 is retired (TaxSection::Payments is never allowed; the screening
//   page's Next goes to the review). The dependents, adjustments, deductions, credits and
//   screening pages left the sequence in M1-P4. Withholding
//   is never asked: Form 1040 lines 25a/25b come from the documents' W-2 box 2 and 1099 box 4.
//   The optional "Anything else?" page (retired in gate fixes batch A; Situations, a list of
//   SituationItem whose RareSituation is constrained by allowed_situations, all variants in M1)
//   is off the common path (SITUATIONS_PAGE, reached through Navigate's TaxSection::Situations
//   from the review's "Nothing else applies" row). Its one answered variant, Payments { estimated, extension },
//   writes the draft's FederalPayments on save (absent means 0 and 0; home_kerosene is kept) and
//   reappears on reopen when either amount is nonzero; every other choice is stored in
//   FederalDraft.situations and refused by name from the one table in
//   federal_rules.rs. Earlier pages never erase payments or situations. A new return starts with
//   zero payments. The item is a union, so the schema cannot declare it unique; the save refuses
//   a situation chosen twice. The W-2 More branch lost its statutory-employee and church/minister
//   answers, now the StatutoryEmployee and ChurchOrMinister situations.
//   Evidence: federal_model.rs Situations/SituationItem/RareSituation; src/tests/cross/interview.rs
//   review_line_25_is_the_documents_withholding, payments_situation_reaches_the_draft,
//   every_rare_situation_refuses_with_its_name,
//   whole_return_situations_refuse_and_notice_on_the_filers_pages.
// Coverage: M1 page order, running estimate and review (unit M1-P4, 2026-09-24).
//   Sequence: setup, you, spouse (joint only), people you support (every return since gate fixes
//   batch A; has_dependents only before, M2-P3),
//   income documents, review; progress reads "2026 federal · section N of 4", or "of 5" with
//   dependents (you and spouse share section 2). The adjustments, deductions, credits, screening
//   and payments sections are never opened
//   (Navigate/Back answer "This section is not available yet."); their draft records hold the
//   "nothing applies" answers (fill_unasked, applied on load and after every accepted change),
//   so the M1/M2 pages complete the return. A save of those sections that still arrives is
//   validated and stored as before and continues at the review. TaxSection::Spouse (11) opens the
//   spouse page from the review and is Back's target from the documents page of a joint return
//   without dependents; with dependents Back from the documents page opens the people page.
//   Erasure follows the walk, not page order (walk-owned data replaced M2-P3's
//   erase_what_depends_on): every save prunes what its walk no longer asks (leaving a joint
//   return, the spouse and every spouse-owned answer; the care claim with nobody on the people
//   page) and reopens a finished page its own save refuses; any change clears
//   finished. An unchanged Next from a page opened by the review returns to the review; a
//   changed one goes to the first page in the walk whose data is absent (the spouse or people
//   page a change made necessary, a reopened page, else the review); otherwise Next follows the
//   page order.
//   Running estimate: every M1 page carries estimate and progress (formatted text, CopyFrom on
//   the Save variants) and one CalculationRequest on its edited root (EstimateSetup,
//   EstimateYou, EstimateSpouse, EstimateDocuments, EstimateSituations, each with the hidden
//   current user); the RunningEstimate reply CopyTo's $.estimate and $.progress. The candidate
//   is built by the page's save code and reconciled like a commit (the prune, fill_unasked)
//   without persisting, and names what saving would remove or reopen on its own line; the text is "Estimated federal refund $X", "Estimated federal balance
//   due $X", or "Estimate not available · section: first issue". The documents and situations
//   calculations also copy the page's allowed sets, because a ConstrainedBy path resolves in the
//   calculation request (Studio: "ConstrainedBy '$.allowed_owners' does not resolve in
//   'EstimateDocuments'"). The calculation does not write allowed_documents: Studio rejects it
//   as "Cyclic calculation writes in 'SaveIncomes' through 'EstimateDocuments'" (a calculation
//   cannot read a field whose allowed set it writes); the set is the fixed three M1 kinds.
//   Review: the estimate text, the FederalResult with its typed Form 1040 return_lines, the
//   "Your answers" rows (one per reachable page; the "Nothing else applies"/"Anything else?" row
//   left with that page in gate fixes batch A), each opening its TaxSection through Link(OpenSection), and
//   StartFinish/Finish unchanged. OpenSection is link-only; Go to opens Navigate's section popup.
//   Evidence: protocol.rs EstimateSetup..EstimateSituations/RunningEstimate/Review/ReviewItem;
//   src/tests/cross/interview.rs m1_sequence_is_four_pages, every_page_calculates_the_running_estimate,
//   review_lists_answers_with_actions.
// Coverage: Unit15 (2026-10-12) supersedes the T2-34 three-list gift mapping.
//   GiftCarryovers writes the canonical PropertyGift, Appraisal and CharityCarryover
//   records directly. Appraisals use normal allocated IDs and a server-supplied catalog;
//   the single basis election applies to all eligible current gifts and prior C balances.
//   The common class worksheet owns limits, original-year balances and the 2026 floor.
// Coverage: T2-40p Schedule K-1 topic and the Form 1041 kind (2026-09-27).
//   New income topic TaxSection::PassThrough "Partnerships, S corporations, estates and trusts"
//   after OtherIncome (INCOME_TOPICS = 9, SEQUENCE 40), listed by the checklist flag
//   IncomeTopics::PASS_THROUGH "Schedule K-1 from a partnership, S corporation, estate or trust"
//   it offers only K1Document::K1EstateTrust for now (the 1065 and
//   1120-S kinds follow in T2-29p and T2-31p). The kind maps onto
//   FederalIncome::EstateTrustPortfolio (k1_page.rs K1EstateTrust::record/from_record); the
//   entry label is the estate's or trust's name; a hub row counts "Form 1041 K-1"; the row's
//   preview is document_amount (None while refused, including any unentered box). Rules on both
//   sides through document_rule -> K1EstateTrust::rule. RareSituation::PassThrough stayed in Less
//   common income, narrowed to partnerships and S corporations, until T2-31p removed it.
//   STORAGE_FORMAT 59. Tests: src/tests/passthrough/interview.rs
//   k1_1041_page_maps_every_branch_onto_the_record_and_reopens_unchanged,
//   k1_1041_rules_reject_on_both_sides_at_their_boundaries, k1_1041_refusals_are_stored_and_named,
//   k1_1041_page_reproduces_the_boxes_6_to_8_fixture,
//   k1_topic_is_listed_on_the_checklist_and_the_income_hub, shared_k1_page_types_map_onto_their_records.
//   Merged with T2-44p: PassThrough follows Gambling (income topics 9 and 11 of the map;
//   INCOME_TOPICS 10, SEQUENCE 42, FLAGS 35), GAMBLING is bit 8, PASS_THROUGH bit 9 and
//   LESS_COMMON bit 10; STORAGE_FORMAT 61.
// Coverage: T2-45p Farming topic (2026-09-27). Income topic TaxSection::Farming after OtherIncome
//   (INCOME_TOPICS = 9, SEQUENCE 40), checklist flag IncomeTopics::FARMING (bit 8, LESS_COMMON bit
//   9), RareSituation::Farm removed (ALL 59; FarmRental stays, F10). Kinds (farm_page.rs types):
//   Farm onto FederalIncome::Business { BusinessActivity::Farming } (Part I receipts one per nonzero
//   line in form order with the section 126 CostSharing receipt from the More fold, or Part III
//   field for field; FarmExpenses one FederalBusinessExpense per nonzero line, the ten Schedule F
//   lines without a Schedule C category on their farm-only categories (T2-45y; F2 wrote
//   labelled Other); the shared
//   Vehicle/Asset/HomeOffice/BusinessStartup/BusinessPayroll through vehicle_and_asset_records and
//   home_office_record; a pre-2018 3/5/7/10-year farm asset written Declining150 (F9); CCC, CRP,
//   prepaid supplies, section 175, land sales with their confirmations written Yes and each
//   Something else applies writing No; income_source NotNeeded writes Services; source
//   UnitedStates, inventory None, specified_service No; health insurance and retirement kept),
//   Patr1099 moved here from Self-employment (F3), FarmAveraging onto FederalScreening.
//   farm_averaging (three fixed base years), FarmLoss onto farm_nol (default Waive, Carryback noted
//   with NOTE_FARMING_LOSS_CARRYBACK_KEPT), plus the shared BusinessPropertyLosses and QbiLossCarryforward data kinds. The
//   farm optional method (F6) writes one SeElection { Farm } per owner whose farms tick it
//   (FARM_OPTIONAL_SE_DIFFERS server); a stored non-farm election keeps the page closed. Reopen is
//   accepted only when saving reproduces the stored record. Rules both sides:
//   FARM_COST_SHARING_NEEDS_CASH, CCC_REPAID_ABOVE_PRINCIPAL, CCC_DEFICIENCY_ABOVE_PRINCIPAL,
//   CONSERVATION_SHARE_ABOVE_TOTAL (on the stored ConservationAssessment),
//   FARM_LAND_SOLD_BEFORE_ACQUIRED, AVERAGING_NEGATIVE_BASE_NEEDS_LOSS,
//   AVERAGING_SOURCE_BEFORE_BASE_YEAR, FARM_AVERAGING_TWICE, FARM_LOSS_TWICE,
//   CROP_WITHHOLDING_ABOVE_PAYMENTS and CONSERVATION_PAYMENTS_ABOVE_SHARE (list sums); server only:
//   FARM_LAND_SOLD_OUTSIDE_CURRENT_YEAR,
//   FARM_WITHHOLDING_ABOVE_RECEIPTS, FARM_OPTIONAL_SE_DIFFERS, and the
//   young-filer income source (F15). Deviations: F7's kinds are filtered when the page is built
//   from the accepted return (a calculation that reads the documents cannot write
//   allowed_documents, which constrains them); F15's income source is not ConstrainedBy (a root
//   ConstrainedBy inside the farm income-source choice does not resolve in PreviewDocument, and a root CopyFrom
//   into it cannot be a PreviewDocument input): every choice is offered and Not needed is refused
//   on save for a filer under 24. PreviewDocument gains the row's id and owner: a farm's preview is
//   line 34 on the accepted return with this farm in place; the hub row totals line 34 ("1 farm ·
//   $X"). Farm issues ("Farm ...", "CCC ...", "CRP ...", "Crop insurance ...") and a farm-only
//   return's Business/Section 1231 issues open the Farming page. STORAGE_FORMAT 59. Tests:
//   src/tests/farm/interview.rs farm_page_maps_every_branch_onto_the_stored_farm_and_back,
//   accrual_farm_and_cost_sharing_round_trip, farm_client_rules_reject_on_the_server_at_their_boundary,
//   farm_server_only_rules_reject_at_their_boundary, farm_refusals_are_named_under_the_farming_page,
//   farming_page_offers_schedule_j_only_with_a_farm_profit,
//   pre_2018_farm_property_uses_150_percent_and_is_not_refused,
//   young_filer_answers_what_produces_the_farms_income,
//   farming_topic_reaches_the_schedule_f_cooperative_fixture_totals,
//   farm_ccc_and_crop_insurance_lines_are_tagged.
//   Merged with T2-44p and T2-40p: Farming sits between Gambling and PassThrough (income topics
//   9, 10 and 11 of the map; INCOME_TOPICS 11, SEQUENCE 43, FLAGS 36); GAMBLING is bit 8, FARMING
//   bit 9, PASS_THROUGH bit 10 and LESS_COMMON bit 11; RareSituation::ALL 57 (Gambling and Farm
//   both removed).
//   The Farming topic's Farm rental kind (IncomeRecord::FarmRental) maps onto FederalIncome::FarmRental both ways (farm_page.rs farm_rental_record
//   and farm_rental_item); its preview is Form 4835 line 32 or 34c (farm_rental_preview,
//   farm_rental_line); RareSituation::FarmRental is gone. The request catalogs `farms` and
//   `ccc_lots` (RecordChoice, keyed by income identity and by each CCC loan's own record identity, in
//   SaveIncomes, EstimateDocuments and PreviewDocument) feed the "which farm" choice of the
//   Form 1099-PATR and 1099-G pages and the loan a Form 1099-A names. Tests: src/tests/farm/farm_rental.rs.
//   Fix unit F2 (2026-10-09): CccLot has a record identity (assigned on the Farming save by
//   assign_record_ids from the one next_id counter, kept wherever the loan moves in the list), and
//   a Form 1099-A names it; STORAGE_FORMAT is the dispatcher's to bump.
// Coverage: T2-31p Schedule K-1 (Form 1120-S) kind (2026-09-27).
//   The PassThrough topic offers K1Document::K1Scorp beside K1EstateTrust; it maps onto
//   FederalIncome::Scorp (k1_page.rs K1ScorpPaper::record/from_record, topic_of Scorp -> PassThrough);
//   the entry label is the corporation's name; a hub row counts "Form 1120-S K-1"; the row's
//   amount is ScorpK1::schedule_e_amount (Schedule E after basis, at-risk and the activity's own
//   passive loss; blank while refused, with section 179, a net passive loss or section 1231
//   items). Rules on both sides through document_rule -> K1ScorpPaper::rule. As the last K-1 unit
//   (C16) RareSituation::PassThrough is removed (ALL 57) and the Less common income help no
//   longer names K-1s. STORAGE_FORMAT 62. Tests: src/tests/passthrough/interview.rs
//   k1_1120s_page_maps_every_branch_onto_the_record_and_reopens_unchanged,
//   k1_1120s_rules_reject_on_both_sides_at_their_boundaries, k1_1120s_refusals_are_stored_and_named,
//   k1_1120s_page_reproduces_the_shareholder_fixtures.
//   K1Document::K1Scorp follows K1EstateTrust;
//   RareSituation::ALL 56 (Gambling, Farm and PassThrough removed); the Less common income help
//   names neither farming, gambling nor any K-1. Partnership K-1s (Form 1065) keep their stored
//   calculation but have no interview entry point until T2-29p. STORAGE_FORMAT 63.
// Coverage: T2-29p Schedule K-1 (Form 1065) kind (2026-09-27).
//   The Schedule K-1 topic offers K1Document::K1Partnership before K1EstateTrust; it maps onto
//   FederalIncome::Partnership (k1_page.rs crate::K1PartnershipDocument::record/from_record); topic_of puts every
//   partnership K-1 on the topic, the entry label is the partnership's name, a hub row counts
//   "Form 1065 K-1", and document_amount (partner_amount) is the Schedule E line 28 amount after
//   basis, at-risk and own passive limits (box 1 part, guaranteed payments, a qualified 2025 fuel
//   credit), None while refused, for a passive loss or with a section 179 amount (return-level).
//   document_entry keeps the partner health insurance and SEP answers of the accepted record (C15:
//   they belong to the self-employed benefits page, which does not list partners yet). Rules on
//   both sides through document_rule -> K1PartnershipPaper::rule; PARTNER_RISK_HISTORY_NEW on save
//   only; a filer under 24 who owns the K-1 is refused "Not needed" on save (the T2-45p fallback:
//   a ConstrainedBy inside an income document cannot resolve in PreviewDocument, so every choice
//   is offered). issue_page opens the topic for every "Partnership ..." issue.
//   RareSituation::PassThrough narrows to S corporations. STORAGE_FORMAT 62. Tests:
//   src/tests/passthrough/interview.rs k1_1065_page_maps_every_branch_onto_the_record_and_reopens_unchanged,
//   k1_1065_rules_reject_on_both_sides_at_their_boundaries, k1_1065_refusals_are_stored_and_named,
//   k1_1065_page_reproduces_the_t2_29_and_t2_29b_fixtures.
//   K1Document::K1Partnership is appended
//   after K1ScorpPaper (Avro order) while the topic lists it first (K1PartnershipPaper, K1EstateTrust,
//   K1ScorpPaper); RareSituation::PassThrough stays removed (ALL 56), so the branch's narrowing to S
//   corporations is dropped. section179.rs keeps one PassThrough.carryover, each K-1 kind
//   supplying its own. STORAGE_FORMAT 64.
// Coverage: T2-scr2 retired screening page (2026-09-27).
//   BeginScreening/SaveScreening are removed from TaxInterviewProtocol, with their entries in
//   every NextResponse list and the SaveScreening save handler; the service never opened them
//   (M3 design). FederalScreening stays the stored record the checklist, topic kinds and
//   situations write; its section 1231 lookback stays reachable as BusinessPropertyLosses. A
//   protocol change only, no stored type: STORAGE_FORMAT unchanged. Tests: src/tests/cross/interview.rs
//   protocol_has_no_screening_page.
// Coverage: T2-iso Form 3921 kind (2026-09-27).
//   Investment sales offers InvestmentSaleDocument::Form3921
//   after CapitalLossCarryover. A data answer onto FederalScreening.iso_exercises (storage stays
//   in FederalScreening: the calculation already reads it there and the retired screening page
//   was its only other writer): DataAnswers::InvestmentSales { capital, iso } replaces
//   CapitalLoss; form3921_records / Form3921::record and data_answer_items / from_record (None
//   for a record with confirmed other than Yes, which only the screening page could write). The
//   item's owner is the exercise's owner. Printed boxes in box order (transferor, boxes 1-5, box
//   6 as Option); filer facts in Form3921More { sold, recognition, option_cost } (defaults: every
//   share held at year-end, exercise-date recognition, no option cost). Rules on
//   both sides: ISO_EXERCISED_BEFORE_GRANTED, ISO_SOLD_ABOVE_TRANSFERRED,
//   ISO_VESTED_BEFORE_EXERCISED; on save also a spouse only on a joint return and the 2026
//   recognition refusal by name (Form 6251 line 2i). Stored changes: IsoExercise gains granted,
//   corporation and sold, shares is box 5, market_price is always box 4, and
//   IsoRecognition::Vesting carries the vesting-date value; line 2i uses the held shares (box 5
//   less sold) and prorates the option cost to them (extended_rules.rs iso_adjustment).
//   STORAGE_FORMAT 66. Tests: src/tests/income/interview.rs form_3921_kind_reaches_form_6251_line_2i,
//   form_3921_nothing_fold_round_trips_and_uses_box_4,
//   form_3921_every_share_sold_in_2026_has_no_adjustment,
//   form_3921_exercise_on_the_grant_date_is_accepted_and_earlier_is_refused,
//   form_3921_recognition_must_fall_in_2026.
// Coverage: T3-small (2026-09-27). (1) The checklist's "You exercised incentive stock options"
//   (OtherTaxTopics::STOCK_OPTIONS) is answered on Investment sales and counts as entered with a
//   Form 3921 (screening.iso_exercises) or ISO shares exercised and sold in 2026 (iso_entered);
//   a tick shows the hub of the page that answers it (sections_with_needs and told_us read
//   hub_of(flag.section), identical to the catalog for every other flag), and a chosen
//   AlternativeMinimumTax situation now ticks Other taxes' "Something less common". (2)
//   InvestmentSaleDocument::IsoShareSale on Investment sales writes
//   FederalIncome::IsoSale (iso_sale.rs IsoShareSale::record / from_record; topic_of maps IsoSale
//   to Investment sales): 1099-B boxes 1a, 1c, 1d, 4 and Form 3921 boxes 1-4, More fold for the
//   W-2 inclusion, fees, commissions, option cost and the special answer. Rules:
//   ISO_EXERCISED_BEFORE_GRANTED and ISO_SOLD_BEFORE_EXERCISED on both sides, ISO_SALE_NOT_IN_RETURN_YEAR
//   on save. (3) ForeignAccountOrTrust.highest_value, FOREIGN_ASSETS_VALUE_NEEDED on both sides,
//   stored as FederalScreening.foreign_asset_max (foreign_assets.rs). STORAGE_FORMAT 71. Tests:
//   src/tests/cross/interview.rs stock_options_tick_is_answered_on_investment_sales,
//   checklist_estimate_lists_the_planned_sections, iso_shares_exercised_earlier_use_their_amt_basis,
//   iso_sale_on_the_two_year_grant_anniversary_is_disqualifying,
//   iso_sale_one_year_after_exercise_is_disqualifying, iso_sale_page_rules_and_special_answer,
//   foreign_account_kind_asks_the_highest_value, foreign_account_kind_answers_schedule_b_part_iii.
// Coverage: T2-src young-filer income source (2026-09-27; screen-optimization approval).
//   Farm choices are Not needed, Capital and Both (FarmIncomeSource indices 0-2; no services-only
//   choice: capital is always material for a farm, custom work for others is Schedule C), so the
//   farm's save-time set is [1, 2] for a filer under 24, else [0, 1, 2]; a stored Services source
//   (Not needed and the Nothing branch) reopens as Not needed. "Both" on the Business page
//   (BusinessIncomeChoice), the K-1 (K1IncomeSource) and the farm (FarmWorkValue) asks the value of
//   the owner's work: AtLeastThirtyPercent (stored BusinessIncomeSource::MixedThirtyPercent, earned
//   income 30% of net profit after half SE) or Amount (stored Mixed { compensation }). With no net
//   profit the 30% limit does not apply (i8615 "Earned income": "the 30% limit doesn't apply if
//   there are no net profits"), so the no-amount answer is taken as capital with a warning
//   (BusinessIncomeSource::for_8615, WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL), for Schedule C/F in the
//   Form 8615 business loop and for partnerships through PartnerK1::young_earned. Not done: the page-level ConstrainedBy
//   allowed set. Studio rejects the root list as a PreviewDocument input: "CalculationRequest
//   'PreviewDocument' input 'id' has no CopyFrom field in current request 'SaveIncomes' or its
//   reference context for invoking field '$.allowed_income_source'" (and without the declaration:
//   "CalculationRequest 'PreviewDocument' CopyFrom '$.allowed_income_source' requires that source
//   field to declare CalculationRequest 'PreviewDocument' in current request 'SaveIncomes'"), so
//   the save-time "Not needed" checks for filers under 24 stay. STORAGE_FORMAT 69 at merge. Tests:
//   src/tests/family/family.rs thirty_percent_answer_is_thirty_percent_of_profit_after_half_se,
//   thirty_percent_business_reaches_the_form_8615_totals,
//   thirty_percent_answer_without_net_profit_is_refused_by_name; src/tests/passthrough/passthrough.rs
//   young_partner_thirty_percent_answer_needs_a_net_profit,
//   young_partner_thirty_percent_loss_is_refused_under_the_k1; src/tests/farm/interview.rs
//   farm_income_source_offers_no_services_choice_and_both_work_values_round_trip,
//   business_page_both_answer_takes_the_thirty_percent_choice_or_an_amount,
//   shared_k1_page_types_map_onto_their_records.
// Coverage: T2-cy young-filer income source allowed sets and follow-ups (2026-09-27).
//   Supersedes the "Not done" of T2-src and the F15/T2-29p save-only fallbacks: with the
//   contract's invariant calculation inputs (protocols.md, invariant-calculation-inputs) the
//   income page carries three hidden, read-only, service-filled sets, SaveIncomes
//   allowed_business_source (BusinessIncomeChoice, which gains NotNeeded at index 0, writing
//   Services like BusinessMore::Nothing), allowed_farm_source (FarmIncomeSource) and
//   allowed_partner_source (K1IncomeSource); EstimateDocuments and PreviewDocument copy them by
//   CopyFrom without declaring CalculationRequest. Each is [NotNeeded] when the calculation
//   cannot read the answer, else every other choice. Schedule C/F sources enter only Form 8615
//   (federal_rules::young_8615: not joint, kiddie_age, a parent alive); a partnership's also the
//   Standard Deduction Worksheet for Dependents (federal_rules::dependent_standard: the taxpayer
//   or, on a joint return, the spouse can be claimed; Form 1040 instructions line 12e; its
//   earned income counts Schedule 1 lines 3 and 6 whole, so C/F sources do not enter it). The
//   three fields drop their Default (each set can be the singleton [NotNeeded], which the client
//   selects itself; a NotNeeded Default would sit outside the needed set). The page reopens a
//   source the return does not read as Not needed. Save re-checks membership by name
//   (income_source_rule; the farm check in documents_candidate): SOURCE_NOT_NEEDED,
//   BUSINESS_SOURCE_NEEDED, PARTNER_SOURCE_NEEDED, FARM_SOURCE_NEEDED. A farm's More: Nothing
//   counts as Not needed (farm_income_source_index), so a Form 8615 return must answer it; a
//   stored farm with Services (Not needed) on a return that later takes Form 8615 is taken as
//   capital with a warning (BusinessIncomeSource::for_8615), not treated as services. Form
//   2210 PenaltyChoice::Figure.farmer drops Default(No) (its set is [ThisReturn] when 2026
//   qualifies). Form 5329 line 17: the excess record's traditional value is the Form 8606
//   year_end plus paid_next_year (form_5329_traditional_value; i5329 includes 2026
//   contributions made in 2027), and follow_ira_year_end first refreshes each IraBasis
//   paid_next_year from the IRA page's 2027 deposits. No stored shape change (STORAGE_FORMAT
//   unchanged). Tests: src/tests/cross/interview.rs income_source_sets_follow_the_calculation,
//   business_income_source_is_checked_against_its_allowed_set_on_save,
//   farm_left_on_nothing_follows_the_allowed_set,
//   partner_income_source_is_checked_against_its_allowed_set_on_save,
//   constrained_fields_with_a_singleton_set_declare_no_default,
//   qualifying_farm_payments_page_holds_no_farmer_value_outside_its_set,
//   form_5329_line_17_value_adds_2026_contributions_made_in_2027; src/tests/passthrough/passthrough.rs
//   joint_return_with_a_claimable_spouse_reads_partner_service_income.
// Coverage: T2-43p partner benefits (2026-09-27). SelfEmployedBenefits.partnerships: one read-only,
//   membership-only
//   PartnerBenefits row per Schedule K-1 (Form 1065) (partner_benefit_rows, matched by
//   FederalIncomeEntry.id; limited partners included, Q3), showing the stored box 13 code M
//   (health_guaranteed), box 13 code R (pensions) and hidden box 19 code A read-only. health
//   (SeHealth) maps the complete plan onto PartnerK1.health_insurance;
//   from_distributions onto health_distributions; PartnerPlan None/Sep{rate}/Other onto
//   PartnerRetirement None/Sep{rate, eligible Yes}/Other. Reopen preserves health facts; an
//   unconfirmed SEP remains an unrepresentable retirement shape. S-corp rows use the same
//   health owner with an existing same-owner W-2 source, preserved by K-1 resave. Rules both sides (partner_benefit_rule on save, the
//   stored K-1 amounts): PARTNER_PREMIUMS_BELOW_K1 (a sum over the months),
//   PARTNER_DISTRIBUTION_PREMIUMS_ABOVE_19A, PARTNER_BOX_13R_NEEDS_PLAN,
//   PARTNER_PLAN_WITHOUT_BOX_13R. Offered with a business or a partnership K-1
//   (has_benefit_rows); necessary while nothing is entered and a business exists or a K-1 shows
//   box 13 code M or R (benefits_needed, Q7); the hub status and the review count name every row
//   with something entered (benefits_entered). issue_page sends "Partner health insurance" and
//   "Partner retirement" here. ira_covered reads pensions or a plan. document_entry keeps only
//   health_insurance, health_distributions and retirement (boxes 13 M and R are the K-1 page's).
//   STORAGE_FORMAT 70 at merge. Tests: src/tests/passthrough/interview.rs
//   partner_benefits_rows_map_onto_the_record_and_reopen_unchanged,
//   partner_benefits_rules_reject_on_both_sides_at_their_boundaries,
//   partner_benefit_refusals_are_named_and_open_the_page,
//   partner_benefits_page_reproduces_the_partner_health_fixtures.
// Coverage: T3-6252 installment sales (2026-09-27).
//   DisposalKind::Sold, RentalSale::Sold and the farm LandSale gain `payment: SalePayment`
//   (InFull | Installment { assumed_debt, payments, interest: NoteInterest, special }, inline,
//   default InFull), stored as entered (LandSale onto FarmLandSale.payment).
//   INSTALLMENT_PAID_ABOVE_PRICE on both sides (disposal_rule, rental_rule, the farm rule).
//   Investment sales offers IncomeRecord::InstallmentPayments, a data answer onto FederalScreening.installment_payments (EarlierInstallment { owner,
//   sale }, at most 20): INSTALLMENT_PROFIT_ABOVE_CONTRACT and INSTALLMENT_PAID_ABOVE_CONTRACT on
//   both sides, a spouse only on a joint return. The page also offers BusinessPropertyLosses
//   when an earlier-year business-property sale or a stored lookback exists (lookback_records
//   and the reopen include InvestmentSales). STORAGE_FORMAT 71. Tests: src/tests/investment/interview.rs
//   installment_asset_fold_round_trips_and_rejects_payments_above_the_price,
//   installment_rental_fold_round_trips_and_rejects_payments_above_the_price,
//   installment_payments_kind_reaches_schedule_d_and_round_trips,
//   installment_payments_kind_rules_reject_on_both_sides.
// Coverage: Form 8824 pages (unit B2, 2026-10-09; tax-forms/form-8824.md). RentalSale gains
//   Exchanged(RentalExchange) (inline choice on the rental page, stored on FederalRental.sale;
//   the replacement continues the rental's entry); rental_rule repeats RENTAL_EXCHANGED_BEFORE_IN_SERVICE
//   and RentalExchange::rule (the EXCHANGE_* page rules, including the extension answer after
//   April 15, 2027). Investment sales offers IncomeRecord::RelatedExchange, a data answer onto FederalScreening.related_exchanges
//   (EarlierRelatedExchange { owner, exchange }, at most 20, a spouse only on a joint return); the
//   section 1231 lookback is offered with a business or rental record. Tests: src/tests/investment/interview.rs
//   exchange_fold_maps_and_its_rules_reject_on_both_sides,
//   related_exchange_kind_reaches_schedule_d_and_round_trips,
//   related_exchange_kind_rules_reject_on_both_sides.
// Coverage: T3-982 Canceled debt topic (2026-09-27).
//   Income topic TaxSection::CanceledDebt after PassThrough (INCOME_TOPICS 12, SEQUENCE 44),
//   checklist flag IncomeTopics::CANCELED_DEBT (bit 11; LESS_COMMON bit 12, FLAGS 37);
//   RareSituation::CanceledDebt removed (ALL 55). Kinds CanceledDebtDocument::Cancel1099C and
//   Acquisition1099A onto FederalIncome::PersonalDebt and the new
//   FederalIncome::PropertyAcquisition (debt_page.rs record/from_record); the creditor's or
//   lender's name is the entry label; hub rows count "1099-C"/"1099-A"; the preview is box 2 or
//   the amount realized. Rules on both sides through document_rule -> Cancel1099C::rule.
//   issue_page opens the topic for the Form 982 sections ("Insolvency tax attributes",
//   "Bankruptcy tax attributes", "Qualified farm debt", ...). STORAGE_FORMAT 71. Tests:
//   src/tests/cross/interview.rs canceled_debt_kinds_map_onto_records_and_reopen_unchanged,
//   canceled_debt_rules_reject_on_both_sides_at_their_boundaries,
//   canceled_debt_refusals_are_stored_named_and_open_the_page.
// Coverage: u5-credits, the unasked credits, taxes and payment (2026-09-28).
//   Other taxes is a list page (SaveOtherTaxes/EstimateOtherTaxes, value OtherTaxes { taxes:
//   Vec<OtherTax> }, hidden allowed_owners constraining HouseholdEmployer.owner and
//   TrumpAccount.owner), replacing its situations page: data items HouseholdEmployees (Schedule
//   H, screening.household_employment), TrumpAccounts (Form 5329 Parts X-XI,
//   screening.trump_accounts) and MortgageSubsidyRecapture (Form 8828, credits.mortgage_subsidies;
//   a sale's hidden home sale ID is bound on save), then the refusal kinds stored as situations
//   (TAX_REFUSALS, the More credits pattern). More credits' HomeKerosene item carries
//   FederalHomeKerosene (payments.home_kerosene). RareSituation::HouseholdEmployer and
//   HomeKerosene removed; the checklist's household, account-taxes and less-common ticks follow
//   the new records. Server twins: KEROSENE_COST_BELOW_CREDIT, TRUMP_EXEMPT_ABOVE_CONTRIBUTIONS,
//   TRUMP_LOSS_ABOVE_PRINCIPAL; server only (list pairs): HOUSEHOLD_EMPLOYER_TWICE,
//   TRUMP_ACCOUNT_TWICE, MORTGAGE_SUBSIDY_HOME_TWICE and the date windows. The walk's prune table
//   names each page; page_refuses covers Other taxes. STORAGE_FORMAT 73. Tests: src/tests/federal/other_taxes.rs;
//   src/tests/cross/change_audit.rs households carry every new record.
//   The Document center's allowed_credit_owners constrains each tax-credit bond.
//   Source Save refuses an unavailable owner. Household and checklist changes retain
//   accepted documents so their ownership can be corrected without losing source facts.
// Coverage: T3-deduct pages for the unasked deductions (2026-09-28). Deductions page (always in the
//   walk): car_loans (Vec<VehicleLoanSource>,
//   owned by the CAR_LOAN tick, replacing Deductions.car_loan and RareSituation::CarLoanInterest,
//   ALL 54), casualties (Vec<PersonalCasualty>) and repayment (RepaymentClaim constrained by
//   allowed_repayment, offered above $3,000 of recorded earlier-year repayments), all stored as
//   shown; deductions_rule holds the server twins. Gifts of property and carryovers page:
//   volunteer (VolunteerExpense) and less_common (LessCommonGift onto hosted_students,
//   whaling_captains by allowed_owners, prior_stock_charity, prior_household_charity);
//   less_common_issue owns the checks; gift_page_entered are the GIFT_CARRYOVERS
//   tick's data and the prune's itemizing line. Investment interest and other page:
//   annuity_losses (FinalAnnuityLoss naming a row of the hidden annuity_contracts catalog, onto
//   AnnuityLoss.income_id). sections_with_needs shows the deductions page for a casualty, a
//   repayment comparison or an offered one. fill_unasked keeps FederalDeductions' empty values as
//   the new record's "none" (each page owns its fields now). STORAGE_FORMAT 73. Tests:
//   src/tests/deductions/deduction_pages.rs; src/tests/cross/change_audit.rs households carry every new field.
// Coverage: sale of your main home (2026-10-27).
//   InvestmentSaleDocument::Form1099S records a printed Form 1099-S in Documents;
//   IncomeRecord::MainHomeSale records a sale without that form on Investment sales.
//   Both use FederalIncome::HomeSale and the shared home-sale worksheet. Source provenance
//   determines whether the sale was reported on Form 1099-S; printed-source corrections
//   retain topic answers, while an own-record edit supplies its complete home facts.
//   HomeSpouse::allowed and MainHomeSale::answers_issue enforce spouse answers exactly
//   on a joint return. The normal household prune removes obsolete spouse answers.
//   follow_home_sales binds a mortgage subsidy recapture to the home closing date.
//   Tests: src/tests/home/home_sale_page.rs and src/tests/cross/change_audit.rs.
// Coverage: unit B3 Form 1099-Q, the 1099-Q and 529 and Coverdell expenses documents (2026-10-09).
//   OtherIncomeDocument::Q1099 and IncomeRecord::EducationAccountExpenses on the Prizes,
//   royalties, alimony and 529 plans page (TaxSection::OtherIncome, retitled) write
//   FederalIncome::EducationDistribution and ::EducationAccountExpenses (q1099_record / q1099_item,
//   education_expenses_record / _item; the expense record is once per return,
//   EDUCATION_ACCOUNT_EXPENSES_TWICE on both sides; unnamed entries).
//   A student is named by identity, the EnumSource key of the
//   hidden people catalog (income_people, shared with the savings bonds); the page rules
//   Q1099_BOXES_DO_NOT_ADD_UP, _PART_ABOVE_GROSS, _ROTH_ROLLOVER_NOT_529, _NOT_STUDENT and _LOSS,
//   _CODE_NOT_COVERDELL and _EXCESS_NEEDS_BOXES are checked again on save (q1099_rule). A Withdrawal
//   shows no amount in the preview (the expense record and credits decide), a 4a transfer its
//   non-qualifying earnings, a code 2 excess its box 2, a 4b transfer and the record zero. The 529-to-
//   Roth record is box 4b: the IRA contributions page asks IraMore.qtp_roth_prior (stored as
//   FederalIraContribution.qtp_roth_lifetime_prior), opens with transfers present, and a return with a
//   box 4b form suggests the IRA topic (ira_qtp_transfer). The walk turns a vanished student into a
//   student not on the return and drops that person's expense row (walk.rs). Tests:
//   src/tests/family/education_accounts.rs, src/tests/cross/interview.rs education_account_*,
//   ira_page_asks_earlier_529_transfers_*, q1099_import_*.
// Coverage: unit B1 Form 8815, the savings bonds document (2026-10-09).
//   InterestDividendDocument::SavingsBonds on the Interest and dividends page writes
//   FederalIncome::SavingsBonds (savings_bonds_record / savings_bonds_item; one record per return,
//   SAVINGS_BONDS_TWICE on both sides; an unnamed entry, no amount preview). Each
//   student row names its person by identity, the EnumSource key of the hidden people catalog
//   (PersonChoice, FormattedText labels from people_students; income_people), so the stored row
//   needs no catalog index. SaveIncomes, EstimateDocuments and PreviewDocument carry the catalog.
//   Page rules SAVINGS_BONDS_INTEREST_ABOVE_PROCEEDS, _ISSUED_BEFORE_1990 and _ISSUED_AFTER_2026 are
//   DateOnly-literal rules on both sides; the estimate, not the save, blocks a person who is not on
//   the return. The walk erases a student row whose person left the return, and a record left with
//   no student (walk.rs). The 1099-INT special flag is InterestSpecial (one choice per unsupported
//   treatment) and box 3 is us_obligations. STORAGE_FORMAT is the dispatcher's. Tests:
//   src/tests/cross/interview.rs savings_bonds_*, savings_bond_rows_leave_with_their_person.
// Coverage: unit B2 Form 6781 Part I, the section 1256 documents (2026-10-09).
//   InvestmentSaleDocument::Contracts1099B on the Investment sales page writes
//   FederalIncome::Section1256 (contracts_record / contracts_item; the broker is the entry label,
//   empty when none is typed; state boxes in state_source as for a sale; special answers Yes
//   leaves the statement out with WARN_SECTION_1256_LEFT_OUT; the preview is box 11).
//   IncomeRecord::Section1256Carryback is a data answer onto
//   FederalScreening.section_1256_carryback (DataAnswers::InvestmentSales.carryback; at most one,
//   SECTION_1256_CARRYBACK_TWICE on both sides, range min 1; no preview). Tests: src/tests/cross/interview.rs
//   contracts_1099b_*, section_1256_carryback_*, investment_sales_offers_the_contracts_kinds.
// Coverage: unit B7a, the investment kinds (2026-10-09). Investment sales gains seven document
//   kinds after IsoShareSale: EmployeeShareSale (EmployeeStockSale), EsppShareSale (EsppSale),
//   NuaShareSale (NuaSale), QsbsSale (QualifiedSmallBusinessSale, one per issuer),
//   SmallBusinessStockLoss (SmallBusinessStock; the stored type is the page, with client rules and
//   server twins), WorthlessSecurity and BadDebt (the stored fields as the named structs
//   WorthlessSecurity and NonbusinessBadDebt). Interest and dividends gains CapitalReturn1099Div
//   (ReturnOfCapital; boxes 3, 9 and 10, so DividendSpecial::Liquidation is removed) and Form2439
//   (UndistributedGain). The Business and Farm "More" folds write the section 197 intangibles
//   (business only), the fuel tax credit and the cash fuel tax recovery. topic_of maps every stored
//   kind to its page; each page's rule() is the server twin of its client rules (DateOnly-literal
//   dates, as SavingsBonds), ISO_SALE_NOT_IN_RETURN_YEAR included. A page's import reads its printed
//   boxes only (1099-B boxes 1a, 1c, 1d and 4; QSBS issuer, sale rows, Form 8949 box and box 4;
//   1099-DIV boxes 3, 9, 10; Form 2439 boxes 1a-1d and 2); the plan facts, Form 3922 boxes,
//   distribution facts, issuer answers and every special answer stay the filer's. Search texts
//   and TAX_NAVIGATION_PROMPT name the kinds. A left-out entry previews zero; QSBS previews none.
//   STORAGE_FORMAT is the dispatcher's. Tests: src/tests/investment/investment_kinds.rs (F1-F12, import mappings,
//   the Form 8949 code B placement, the Schedule D footing, previews).
// Coverage: unit B7b, the Work abroad topic (2026-10-09; tax-forms/form-2555.md). Income topic
//   TaxSection::WorkAbroad after CanceledDebt (INCOME_TOPICS 13, SEQUENCE 47, FLAGS 38), checklist
//   flag IncomeTopics::WORK_ABROAD (bit 12). Kinds IncomeRecord::Form2555 the server twin of Form2555's client rules) and IncomeRecord::ForeignPay
//   (onto FederalIncome::ForeignEmployment, the employer as the entry label; topic_of maps it
//   here, document_kind_keys names it). The W-2 page's "Your answers about this W-2" gains the
//   box 1 pay for work abroad (W2Answers.abroad, W2_ABROAD_ABOVE_WAGES in document_rule) and the
//   business page's More about this business gains where the work was done (business_record and
//   business_item carry BusinessDetails.source, so a business run abroad reopens exactly). The
//   search texts, TaxSection::WorkAbroad documentation and TAX_NAVIGATION_PROMPT name the topic.
//   No import declarations. STORAGE_FORMAT is the dispatcher's. Tests: src/tests/foreign/work_abroad.rs
//   work_abroad_page_round_trips_and_checks_both_sides,
//   w2_abroad_amount_and_business_source_round_trip.
use crate::federal_rules::{estimate, validate_dependent, validate_income};
use crate::kernel::phase;
use crate::validation::{SaveFailure, ValidationIssue};
use crate::*;
use TaxInterviewProtocol as P;
use alloc::{string::String, vec::Vec};
use libertas::{LibertasMessageArgument, LibertasUser, libertas_formatted_text};
mod activity_followups;
mod cooperative_followups;
mod debt_followups;
mod document_center;
mod document_followups;
mod followup_page;
pub(crate) use debt_followups::answer_issue as debt_answer_issue;
#[cfg(test)]
pub(crate) use document_followups::fixture_issue_path;
pub(crate) use document_followups::followup_topic;
#[cfg(test)]
pub(crate) use followup_page::situations_pending;

mod document_owners;
#[cfg(test)]
pub(crate) use document_owners::assign_native_document_ids;
pub(crate) use document_owners::unknown_owner_issues;
mod estate_followups;
pub(crate) use estate_followups::retain as estate_retain;
mod k1_followups;
pub(crate) mod related_forms;
mod walk;
pub(crate) use document_center::center_document_kinds;
#[cfg(test)]
pub(crate) use walk::prune;
use walk::{
    Pruned, in_walk, page_refuses, prune_data, reopen_refused, save_warnings, topic_has_data,
};

// The data answers of M3-P15: kinds of an income topic that are
// FederalScreening facts, not income entries. The one list: a new kind adds
// itself here, and every match that must exclude the data answers
// (`document_income`, `document_entry`, the preview) reads this pattern.
macro_rules! data_answer_kinds {
    () => {
        IncomeSource::Record { value: crate::IncomeRecord::ForeignAccountOrTrust(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::TaxableBondPremium(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(_) }
            | IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form3921(_) }
            | IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form3922(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::InstallmentPayments(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::Section1256Carryback(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::RelatedExchange(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::QbiLossCarryforward(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::NetOperatingLoss(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::BusinessPropertyLosses(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::FarmAveraging(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::FarmLoss(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::Form2555(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::Form8915(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::AbleContributions(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::Clergy(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::NonfarmOptionalMethod(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::SpouseSection179(_) }
    };
}
// The kinds that write an Adjustments record (Form 8606, QCD gifts and the
// retirement adjustments), not an income entry either.
macro_rules! adjustment_kinds {
    () => {
        IncomeSource::Record { value: crate::IncomeRecord::Form8606(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::Qcd(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::SocialLumpSums(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::MissedRmd(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::RothHistory(_) }
    };
}

// The M2 sequence is setup, you, your spouse (joint only), the people you
// support (only when page 1 says you support someone), income documents and
// the review. The spouse page belongs to the filers section but reopens by
// its own section number.
const SETUP_PAGE: i32 = TaxSection::Setup as i32;
const PEOPLE_PAGE: i32 = TaxSection::People as i32;
const SPOUSE_PAGE: i32 = TaxSection::Spouse as i32;
// Steps only a separate return walks, inside an existing section: community
// property after About you (filers) and the spouse's standard deduction boxes
// after the deductions page. TaxSection is stored, so they have no section
// identity: their requests carry their section's page, and these routing
// numbers, outside every TaxSection value, open them (walk, Back, resume).
const COMMUNITY_PAGE: i32 = 1001;
const SPOUSE_BOXES_PAGE: i32 = 1002;
const SPOUSE_ITEMIZES_PAGE: i32 = 1003;
// The people page (M2 page 3) is sent only when page 1 answers that you
// support someone.
const DOCUMENTS_PAGE: i32 = TaxSection::Documents as i32;
const INCOME_WITHOUT_FORM_PAGE: i32 = TaxSection::IncomeWithoutForm as i32;
const CHILDREN_PAGE: i32 = TaxSection::Children as i32;
// The income hub (M3-P1): its topics are pages of their own, each the
// income documents page limited to the topic's document kinds.
const INCOME_PAGE: i32 = TaxSection::Income as i32;
const WAGES_PAGE: i32 = TaxSection::Wages as i32;
const INTEREST_PAGE: i32 = TaxSection::InterestDividends as i32;
const INVESTMENT_SALES_PAGE: i32 = TaxSection::InvestmentSales as i32;
const RETIREMENT_PAGE: i32 = TaxSection::Retirement as i32;
const SELF_EMPLOYMENT_PAGE: i32 = TaxSection::SelfEmployment as i32;
const RENTALS_PAGE: i32 = TaxSection::Rentals as i32;
const GOVERNMENT_BENEFITS_PAGE: i32 = TaxSection::GovernmentBenefits as i32;
const OTHER_INCOME_PAGE: i32 = TaxSection::OtherIncome as i32;
const LONG_TERM_CARE_BENEFITS_PAGE: i32 = TaxSection::LongTermCareBenefits as i32;
const GAMBLING_PAGE: i32 = TaxSection::Gambling as i32;
const FARMING_PAGE: i32 = TaxSection::Farming as i32;
const PASS_THROUGH_PAGE: i32 = TaxSection::PassThrough as i32;
const CANCELED_DEBT_PAGE: i32 = TaxSection::CanceledDebt as i32;
const WORK_ABROAD_PAGE: i32 = TaxSection::WorkAbroad as i32;
// M3-P5 adds the self-employment topic, M3-P6 the rental property topic,
// T2-44p gambling winnings, T2-45p farming and T2-40p the Schedule K-1 topic
// (the 1041 kind; the 1065 and 1120-S kinds follow).
const INCOME_TOPICS: [TaxSection; 15] = [
    TaxSection::Wages,
    TaxSection::InterestDividends,
    TaxSection::InvestmentSales,
    TaxSection::Retirement,
    TaxSection::SelfEmployment,
    TaxSection::Rentals,
    TaxSection::GovernmentBenefits,
    TaxSection::OtherIncome,
    TaxSection::LongTermCareBenefits,
    TaxSection::Gambling,
    TaxSection::Farming,
    TaxSection::PassThrough,
    TaxSection::CanceledDebt,
    TaxSection::WorkAbroad,
    TaxSection::HomeOffices,
];
// The adjustments hub (M3-P7, M3-P8) and its topics; later units add theirs.
const ADJUSTMENTS_PAGE: i32 = TaxSection::Adjustments as i32;
const HSA_PAGE: i32 = TaxSection::Hsa as i32;
const SELF_EMPLOYED_BENEFITS_PAGE: i32 = TaxSection::SelfEmployedBenefits as i32;
const IRA_CONTRIBUTIONS_PAGE: i32 = TaxSection::IraContributions as i32;
const EDUCATOR_PAGE: i32 = TaxSection::Educator as i32;
const STUDENT_LOAN_PAGE: i32 = TaxSection::StudentLoan as i32;
const EMPLOYEE_EXPENSES_PAGE: i32 = TaxSection::EmployeeExpenses as i32;
const ADJUSTMENT_TOPICS: [TaxSection; 6] = [
    TaxSection::Hsa,
    TaxSection::SelfEmployedBenefits,
    TaxSection::IraContributions,
    TaxSection::Educator,
    TaxSection::StudentLoan,
    TaxSection::EmployeeExpenses,
];
// The deductions page and the itemized hub (M3-P9). The hub lists every
// Schedule A topic; home mortgage and charity carryovers open with M3-P10.
const DEDUCTIONS_PAGE: i32 = TaxSection::Deductions as i32;
const ITEMIZED_PAGE: i32 = TaxSection::Itemized as i32;
const MEDICAL_PAGE: i32 = TaxSection::Medical as i32;
const TAXES_PAGE: i32 = TaxSection::Taxes as i32;
const HOME_OFFICES_PAGE: i32 = TaxSection::HomeOffices as i32;
const MORTGAGE_PAGE: i32 = TaxSection::Mortgage as i32;
const GIFT_CARRYOVERS_PAGE: i32 = TaxSection::GiftCarryovers as i32;
const OTHER_ITEMIZED_PAGE: i32 = TaxSection::OtherItemized as i32;
const ITEMIZED_TOPICS: [TaxSection; 5] = [
    TaxSection::Medical,
    TaxSection::Taxes,
    TaxSection::Mortgage,
    TaxSection::GiftCarryovers,
    TaxSection::OtherItemized,
];
// Every itemized topic now opens (M3-P10 completes the hub P9 began).
const OPEN_ITEMIZED_TOPICS: [TaxSection; 5] = ITEMIZED_TOPICS;
// The credits hub (M3-P11): child and dependent care is listed and opens the
// people page; more credits opens with M3-P12.
const CREDITS_PAGE: i32 = TaxSection::Credits as i32;
const CARE_PAGE: i32 = TaxSection::Care as i32;
const EDUCATION_PAGE: i32 = TaxSection::Education as i32;
const SAVER_PAGE: i32 = TaxSection::Saver as i32;
const MARKETPLACE_PAGE: i32 = TaxSection::Marketplace as i32;
const MORE_CREDITS_PAGE: i32 = TaxSection::MoreCredits as i32;
// The adjustments hub's last topic and the other taxes section.
const MORE_ADJUSTMENTS_PAGE: i32 = TaxSection::MoreAdjustments as i32;
const OTHER_TAXES_PAGE: i32 = TaxSection::OtherTaxes as i32;
// Form 8615's parent facts (u5-losses): after the other taxes page while
// Form 8615 applies (parent_tax_in_walk); part of that section.
const PARENT_TAX_PAGE: i32 = TaxSection::ParentTax as i32;
// The federal payments page (M3-P13). Its box C answer opens the income by
// period page (M3-P14), which belongs to the payments section.
const PAYMENTS_PAGE: i32 = TaxSection::Payments as i32;
const INCOME_BY_PERIOD_PAGE: i32 = TaxSection::IncomeByPeriod as i32;
// The injured spouse allocation (T3-8379): after the payments section while
// the checklist's injured spouse item is ticked on a joint return.
const INJURED_SPOUSE_PAGE: i32 = TaxSection::InjuredSpouse as i32;
// The last page before the review (step 5 of the walk).
const OFTEN_MISSED_PAGE: i32 = TaxSection::OftenMissed as i32;
const REVIEW_PAGE: i32 = TaxSection::Review as i32;
// The opening checklist (M3-P18), after the people pages.
const CHECKLIST_PAGE: i32 = TaxSection::Checklist as i32;
// The hubs a page opened from one returns to.
const HUB_PAGES: [i32; 4] = [INCOME_PAGE, ADJUSTMENTS_PAGE, ITEMIZED_PAGE, CREDITS_PAGE];

pub(crate) struct Interview {
    /// The verified user this return belongs to; every request carries it.
    pub(crate) user: LibertasUser,
    pub(crate) draft: FederalDraft,
    pub(crate) dirty: bool,
    // Request-local acceptance signal, including validated unchanged pages.
    pub(crate) accepted_save: bool,
    blocked: bool,
    // Built only for the walk's own checks (Interview::viewing): its pages
    // carry no estimate or progress text.
    viewing: bool,
}
// One calculation invocation owns this candidate/result pair. It is never
// persisted or retained on Interview, and no client-supplied flag is trusted.
struct PreparedEstimate {
    draft: alloc::boxed::Box<FederalDraft>,
    result: alloc::boxed::Box<FederalResult>,
    pruned: Pruned,
}
type CommitIssue = (ValidationIssue, Option<i64>, &'static str);

struct PreparedIncomeCommit {
    pruned: Pruned,
    changed: bool,
    income_finished: bool,
    result: alloc::boxed::Box<FederalResult>,
}
#[derive(Debug)]
pub(crate) enum Reply {
    Response(P),
    Edit(P),
}
// A review or hub row's status (ReviewItem.status, FormattedText): a fixed
// state is an argument-free REVIEW_* resource, which display.json
// (resource([status])) and with_checklist_rows compare by key, never by
// text. Counts remain plural arguments; amounts remain exact currency;
// authored explanations are keyed messages. TEXT is for user-entered names.
fn status(key: &str) -> Vec<u8> {
    libertas_formatted_text(key, &[])
}
fn status_count(key: &str, count: usize) -> Vec<u8> {
    libertas_formatted_text(
        key,
        &[LibertasMessageArgument::Plural(i64::try_from(count).unwrap_or(i64::MAX))],
    )
}
fn status_usd(key: &str, cents: i64) -> Vec<u8> {
    libertas_formatted_text(
        key,
        &[LibertasMessageArgument::Currency { currency_code: "USD", scale: 2, minor_units: cents }],
    )
}
// Whether a status is the argument-free resource `key`: a comparison by
// resource key, as display.json's resource([status]) makes, never by text.
fn status_is(value: &[u8], key: &str) -> bool {
    value == status(key).as_slice()
}
// A remaining item's status as a hub row's attention. Keep the complete
// formatted message as a nested argument so its own typed arguments and
// locale-specific word order survive the surrounding sentence.
pub(crate) fn attention_status(value: &[u8]) -> Vec<u8> {
    let mut offset = 0;
    let Ok((resource_name, decoded_arguments)) = <(
        alloc::string::String,
        Vec<libertas::LibertasMessageArgumentDecode>,
    ) as libertas::AvroDecode>::avro_decode(
        value, &mut offset
    ) else {
        return value.to_vec();
    };
    let arguments: Vec<LibertasMessageArgument> =
        decoded_arguments.iter().map(|a| a.as_message_argument()).collect();
    match resource_name.as_str() {
        "REVIEW_NOT_CALCULATED" => {
            libertas_formatted_text("REVIEW_ATTENTION_NOT_CALCULATED", &arguments)
        }
        // A topic not reached yet is not started; it needs no attention.
        "REVIEW_COMING_UP" => libertas_formatted_text("REVIEW_NOT_STARTED", &[]),
        _ => libertas_formatted_text(
            "REVIEW_ATTENTION",
            &[LibertasMessageArgument::formatted_text(value)],
        ),
    }
}
fn problem((key, _english): (&str, &str)) -> Reply {
    problem_key(key)
}
// Preserve the native owner's localized diagnostic. Legacy failures remain
// explicitly unlocated until their source walk is migrated.
pub(crate) fn stale_editor_reply() -> Reply {
    problem_key("PROBLEM_ACCEPTED_ANSWERS_CHANGED")
}
fn problem_key(key: &str) -> Reply {
    refused(libertas_formatted_text(key, &[]))
}
fn refused(failure: impl Into<crate::validation::SaveFailure>) -> Reply {
    let failure = failure.into();
    Reply::Response(P::Problem {
        message: failure.message,
        field: failure.field.map(crate::validation::FieldPath::square),
    })
}
fn refused_source(failure: impl Into<SaveFailure>, has_value_field: bool) -> Reply {
    let failure = failure.into();
    refused(if has_value_field {
        failure.map_field_prefix(&["document"], &["document", "value"])
    } else {
        failure
    })
}
// Native document-source validation shared by Save and preview.
// The young-filer income-source refusals (T2-cy), named with the form line
// that reads the answer.
const FARM_SOURCE_NEEDED: &str = "SOURCE_FARM_NEEDED";
const BUSINESS_SOURCE_NEEDED: &str = "SOURCE_BUSINESS_NEEDED";
const PARTNER_SOURCE_NEEDED: &str = "SOURCE_PARTNER_NEEDED";
const SOURCE_NOT_NEEDED: &str = "SOURCE_NOT_NEEDED";
// A business's or partnership K-1's income source inside the page's
// allowed sets, checked again on save (the ConstrainedBy twins).
fn income_source_rule(
    documents: &[IncomeDocumentItem],
    business_needed: bool,
    partner_needed: bool,
    require_partner_answer: bool,
) -> Result<(), SaveFailure> {
    let business = crate::BusinessIncomeChoice::allowed(business_needed);
    let partner = K1IncomeSource::allowed(partner_needed);
    let refusal = |needed: bool, key: &'static str| {
        ValidationIssue::new(if needed { key } else { SOURCE_NOT_NEEDED })
    };
    for (index, item) in documents.iter().enumerate() {
        match &item.document {
            IncomeSource::Record { value: crate::IncomeRecord::Business(b) }
                if !business.contains(&b.more.income_source.index()) =>
            {
                return Err(submitted_data_answer_issue(
                    refusal(business_needed, BUSINESS_SOURCE_NEEDED)
                        .at_field("income_source")
                        .in_field("more"),
                    index,
                )
                .into());
            }
            IncomeSource::K1 { value: crate::K1Document::K1Partnership(k1) }
                if require_partner_answer
                    && !partner.contains(&k1.calculation_page().income_source.index()) =>
            {
                return Err(submitted_data_answer_issue(
                    refusal(partner_needed, PARTNER_SOURCE_NEEDED).at_field("income_source"),
                    index,
                )
                .into());
            }
            _ => {}
        }
    }
    Ok(())
}
// A child's statements keep the rules of the filer's own 1099 pages.
fn child_statement_issue(child: &crate::Form8814) -> Option<ValidationIssue> {
    use crate::ChildStatement as C;
    child.statements.iter().enumerate().find_map(|(index, statement)| {
        document_issue(&match statement {
            C::Int1099(v) => IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(v.form.clone()),
            },
            C::Div1099(v) => IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(v.document.clone()),
            },
            C::Oid1099(v) => IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Oid1099(v.form.clone()),
            },
        })
        .map(|issue| match statement {
            C::Div1099(_) => issue.in_field("document"),
            _ => issue,
        })
        .or_else(|| match statement {
            // A dividend's personal answers are asked on the owning topic.
            C::Div1099(_) => None,
            C::Int1099(v) => int_source_issue(v, true),
            C::Oid1099(v) => oid_source_issue(v, true),
        })
        .map(|issue| issue.in_list("statements", index))
    })
}

fn document_issue(document: &IncomeSource) -> Option<ValidationIssue> {
    // The state-source preflight retains its existing priority.
    if let Some(key) = crate::state_return::document_rule(document) {
        return Some(ValidationIssue::new(key).at_record());
    }
    match document {
        IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(value) } => {
            let record: crate::PriorCapitalReturn = value.clone().into();
            if !crate::injured_spouse_sources::niit_owners_valid(&record) {
                return Some(
                    ValidationIssue::new("INJURED_NIIT_OWNER_TOTAL")
                        .at_field("injured_niit_owners"),
                );
            }
            if value.injured_owners.as_ref().is_some_and(|owners| {
                !matches!(value.owners, crate::PriorReturnOwners::Joint)
                    || !owners.valid(
                        crate::extended_rules::regular_capital_carry(&record).map(|n| n * 100),
                    )
            }) {
                return Some(
                    ValidationIssue::new("INJURED_CAPITAL_OWNER_TOTAL").at_field("injured_owners"),
                );
            }
            crate::extended_rules::investment_parts_issue(&record)
        }
        IncomeSource::Record { value: crate::IncomeRecord::InstallmentPayments(value) } => {
            value.record().livestock_issue().map(|issue| issue.in_field("InstallmentPayments"))
        }
        IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(value) } => value.issue(),
        IncomeSource::K1 { value: crate::K1Document::K1Scorp(value) } => value.issue(),
        IncomeSource::K1 { value: crate::K1Document::K1Partnership(value) } => value.issue(),
        IncomeSource::Record { value: crate::IncomeRecord::WorthlessSecurity(value) } => {
            value.issue()
        }
        IncomeSource::Record { value: crate::IncomeRecord::BadDebt(value) } => value.issue(),
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Form2439(value),
        } => value.issue(),
        IncomeSource::Record { value: crate::IncomeRecord::JuryPay(value) } => value.issue(),
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::WildfireRelief(value) } => {
            value.issue()
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::DisasterRelief(value) } => {
            value.issue()
        }
        IncomeSource::Record { value: crate::IncomeRecord::CasualtyLivingInsurance(value) } => {
            value.issue()
        }
        IncomeSource::Record { value: crate::IncomeRecord::DisasterMitigation(value) } => {
            value.issue()
        }
        IncomeSource::Record { value: crate::IncomeRecord::RothHistory(value) } => value.issue(),
        IncomeSource::Record { value: crate::IncomeRecord::Form8606(value) } => value.issue(),
        IncomeSource::Record { value: crate::IncomeRecord::InheritedIra(value) } => value.issue(),
        IncomeSource::Record { value: crate::IncomeRecord::Form8915(value) } => value.issue(),
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(_) }
        | IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Div1099(_) }
        | IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Oid1099(_) } => {
            investment_statement_issue(document)
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::R1099(value) } => {
            crate::retirement::source_issue(&crate::R1099::from(value))
        }
        IncomeSource::Record { value: crate::IncomeRecord::Business(value) } => {
            if value.more.notary_gross.is_some_and(|gross| gross < value.more.notary_profit) {
                return Some(
                    ValidationIssue::new("NOTARY_GROSS_BELOW_NET")
                        .at_field("notary_gross")
                        .in_field("more"),
                );
            }
            business_issue(value).map(|issue| {
                if issue.key == "ASSET_179_ABOVE_SUV_LIMIT" {
                    issue.in_field("Business")
                } else {
                    issue
                }
            })
        }
        IncomeSource::Record { value: crate::IncomeRecord::Royalties(value) } => {
            crate::royalties::page_issue(value)
        }
        IncomeSource::Farming(value) => patr_paper_input_issue(value),
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099S(value) } => {
            value.issue()
        }
        IncomeSource::Record { value: crate::IncomeRecord::MainHomeSale { sale, answers } } => {
            home_sale_record_issue(sale, answers, answers.spouse.joint())
        }
        IncomeSource::Record { value: crate::IncomeRecord::Form2555(value) } => value.issue(),
        IncomeSource::SelfEmployment {
            value: crate::SelfEmploymentDocument::Form1099Nec(value),
        } => value.issue(),
        IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099K(value) } => {
            value.issue()
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(value) } => {
            value.admission_issue()
        }
        IncomeSource::Record { value: crate::IncomeRecord::Farm(value) } => {
            crate::farm_page::farm_issue(value).map(|issue| {
                if issue.key == "ASSET_179_ABOVE_SUV_LIMIT" {
                    issue.in_field("Farm")
                } else {
                    issue
                }
            })
        }
        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(value) } => {
            crate::farm_page::farm_rental_issue(value)
        }
        IncomeSource::Record { value: crate::IncomeRecord::FarmAveraging(value) } => {
            crate::farm_page::averaging_issue(value)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Rental(value) } => {
            crate::rental_foreign::location_issue(&value.location)
                .map(|issue| issue.in_field("location"))
                .or_else(|| {
                    crate::rental_recovery::page_issue(value).or_else(|| rental_issue(value))
                })
        }
        IncomeSource::Wage { value: crate::WageDocument::W2(value) } => w2_input_issue(value),
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(value) } => {
            crate::LtcSource::printed(value.clone()).record().issue()
        }
        IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(value) } => {
            value.record().issue()
        }
        IncomeSource::Wage { value: crate::WageDocument::Form8919(value) } => value.issue(),
        IncomeSource::Record { value: crate::IncomeRecord::Scholarship(value) } => value.issue(),
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::OpportunityFund(value),
        } => value.issue(),
        IncomeSource::Record { value: crate::IncomeRecord::OpportunityFundDeferral(value) } => {
            value.issue()
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::QsbsSale(value) } => {
            value.issue()
        }
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::IsoShareSale(value),
        } => value.issue(),
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EmployeeShareSale(value),
        } => value.issue(),
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EsppShareSale(value),
        } => value.issue(),
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::NuaShareSale(value),
        } => value.issue(),
        IncomeSource::Record { value: crate::IncomeRecord::SmallBusinessStockLoss(value) } => {
            value.issue()
        }
        IncomeSource::CanceledDebt { value: crate::CanceledDebtDocument::Cancel1099C(_) } => None,
        IncomeSource::Record { value: crate::IncomeRecord::RelatedExchange(value) } => {
            value.issue()
        }
        IncomeSource::Record { value: crate::IncomeRecord::InstallmentSale(value) } => {
            value.issue().map(|issue| {
                issue
                    .map_field_prefix(&["second_disposition"], &["second_disposition", "Resold"])
                    .in_field("InstallmentSale")
            })
        }
        IncomeSource::Record { value: crate::IncomeRecord::SellerMortgageInterest(value) } => {
            value.buyer.issue().map(|issue| issue.in_field("buyer"))
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Qa1099(value) } => {
            value.boxes_issue()
        }
        // Other document kinds share their typed local owner.
        _ => document_local_issue(document),
    }
}

fn home_sale_record_issue(
    sale: &crate::Form1099S,
    answers: &crate::HomeSaleAnswers,
    joint: bool,
) -> Option<ValidationIssue> {
    sale.issue().map(|issue| issue.in_field("sale")).or_else(|| {
        crate::MainHomeSale::records(sale.clone(), answers.clone())
            .answers_issue(joint)
            .map(|issue| issue.in_field("answers"))
    })
}

#[cfg(test)]
pub(crate) fn document_rule(document: &IncomeSource) -> Option<&'static str> {
    document_issue(document).map(|issue| issue.key)
}

fn gambling_document_issue(form: &crate::FormW2GPaper) -> Option<ValidationIssue> {
    (form.withholding.unwrap_or(0) > form.winnings)
        .then(|| ValidationIssue::new("W2G_WITHHOLDING_ABOVE_WINNINGS").at_field("withholding"))
}

// Remaining local source checks return fields relative to their actual input value.
fn document_local_issue(document: &IncomeSource) -> Option<ValidationIssue> {
    match document {
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099B(form) } => {
            form.admission_issue()
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099DA(form) } => {
            form.admission_issue()
        }
        IncomeSource::Record { value: crate::IncomeRecord::CashTipsWithoutW2(v) } => {
            if v.employer.trim().is_empty() {
                Some(ValidationIssue::new("INCOME_SOURCE_NAME_REQUIRED").at_field("employer"))
            } else if !v.tips().valid() {
                Some(ValidationIssue::new("ISSUE_EMPLOYEE_TIPS_RECORDS").at_field("months"))
            } else {
                None
            }
        }
        IncomeSource::Record { value: crate::IncomeRecord::PrivateLoanInterest(v) } => {
            if v.borrower.trim().is_empty() {
                Some(ValidationIssue::new("INCOME_SOURCE_NAME_REQUIRED").at_field("borrower"))
            } else {
                v.buyer().and_then(|p| p.issue()).map(|i| i.in_field("seller_mortgage"))
            }
        }
        IncomeSource::Record { value: crate::IncomeRecord::ForeignAccountOrTrust(v) } => {
            if (v.account || v.trust) && v.highest_value.is_none() {
                Some(ValidationIssue::new("FOREIGN_ASSETS_VALUE_NEEDED").at_field("highest_value"))
            } else {
                crate::foreign_accounts::issue(v.account, v.fbar_required, &v.fbar_countries)
            }
        }
        IncomeSource::Record { value: crate::IncomeRecord::Form8814(v) } => {
            child_statement_issue(v)
        }
        IncomeSource::Record { value: crate::IncomeRecord::AlimonyReceived(v) } => alimony_issue(v),
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::SavingsBonds(v),
        } => savings_bonds_entry_issue(v),
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Form8818(v) } => {
            bond_redemption_paper_issue(v)
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(v) } => {
            q1099_input_issue(v)
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Sa1099(v) } => {
            crate::SaSource::printed(v.clone()).input_issue(true)
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::HsaTrusteeStatement(v) } => {
            crate::SaSource::trustee_statement(v.clone()).input_issue(true)
        }
        IncomeSource::Record { value: crate::IncomeRecord::HealthLoanRepayment(v) } => v.issue(),
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Sale1099B(v) } => {
            sale_issue(v)
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::Ssa1099(v) } => {
            crate::ssa_1099_model::net_benefits_issue(
                &crate::FederalSocial::from(v).form_boxes,
                v.net_benefits,
            )
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099(v) } => v.issue(),
        IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099R(v) } => {
            rrb_source_issue(&crate::Rrb1099R::from(v))
        }
        IncomeSource::Record { value: crate::IncomeRecord::SocialLumpSums(v) } => v.issue(),
        IncomeSource::GovernmentPayment { value: v, .. } => {
            if v.has_payments() {
                v.payments_admission_issue()
            } else {
                v.refund_admission_issue()
            }
        }
        IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(v) } => {
            v.issue().or_else(|| {
                (!v.valid()).then_some(ValidationIssue::new("OTHER_INCOME_INVALID").at_record())
            })
        }
        IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(v) } => {
            equipment_rent_issue(v)
        }
        IncomeSource::Gambling(v) => gambling_document_issue(v),
        _ => None,
    }
}
// Assign identities only in the candidate draft. A rejected section cannot consume IDs.
pub(crate) fn assign_record_ids<'a>(
    records: impl Iterator<Item = &'a mut i64>,
    accepted: impl Iterator<Item = i64>,
    next_id: &mut i64,
) -> Result<(), (&'static str, &'static str, usize)> {
    let accepted: alloc::collections::BTreeSet<_> = accepted.collect();
    let mut seen = alloc::collections::BTreeSet::new();
    for (index, id) in records.enumerate() {
        if *id == 0 {
            *id = *next_id;
            *next_id = next_id.checked_add(1).ok_or((
                "ISSUE_INTERVIEW_THE_RECORD_IDENTITY_LIMIT_HAS_BEEN_78BC135C",
                "The record identity limit has been reached.",
                index,
            ))?;
        } else if !accepted.contains(id) {
            return Err((
                "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C",
                "A record no longer belongs to this section. Reopen the section.",
                index,
            ));
        }
        if !seen.insert(*id) {
            return Err((
                "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425",
                "A record identity is repeated in this section.",
                index,
            ));
        }
    }
    Ok(())
}

// The income documents page is a projection of the stored income records. A
// document's `More::Nothing` branch stands for every other field of its record
// holding its "nothing applies" value; the reverse mapping picks `Nothing`
// exactly when that is so, so reopening a page and saving it unchanged
// reproduces the same records.
// The document kinds an income topic's page offers; None for a section that
// is not an income topic.
fn source_in_editor(d: &FederalDraft, entry: &FederalIncomeEntry, topic: TaxSection) -> bool {
    if topic == TaxSection::Documents {
        // Ownership survives an unrepresentable editor projection. A native
        // record that cannot be displayed still belongs to its record topic.
        match &entry.income {
            FederalIncome::Interest { data } => {
                let data = &crate::FederalInterest::from(data);
                !data.private_loan && data.seller_mortgage.is_none()
            }
            FederalIncome::Gambling { data } => !FederalGambling::from(data).own_records,
            FederalIncome::Wage { data } => !data.is_cash_tips(),
            FederalIncome::HomeSale { data } => data.form.paper().is_some(),
            FederalIncome::EmployeeWages { .. }
            | FederalIncome::Dividend { .. }
            | FederalIncome::Oid { .. }
            | FederalIncome::ReturnOfCapital { .. }
            | FederalIncome::UndistributedGain { .. }
            | FederalIncome::SavingsBonds { .. }
            | FederalIncome::BondRedemption { .. }
            | FederalIncome::Broker { .. }
            | FederalIncome::Sale { .. }
            | FederalIncome::OpportunityFund { .. }
            | FederalIncome::Section1256 { .. }
            | FederalIncome::IsoSale { .. }
            | FederalIncome::EmployeeStockSale { .. }
            | FederalIncome::EsppSale { .. }
            | FederalIncome::EsppSaleSource { .. }
            | FederalIncome::IsoSaleSource { .. }
            | FederalIncome::NuaSaleSource { .. }
            | FederalIncome::NuaSale { .. }
            | FederalIncome::QualifiedSmallBusinessSale { .. }
            | FederalIncome::Retirement { .. }
            | FederalIncome::Roth { .. }
            | FederalIncome::TrumpDeath { .. }
            | FederalIncome::TrumpAbleRollover { .. }
            | FederalIncome::SocialSecurity { .. }
            | FederalIncome::GovernmentPayment { .. }
            | FederalIncome::Cooperative { .. }
            | FederalIncome::EstateTrustPortfolio { .. }
            | FederalIncome::Partnership { .. }
            | FederalIncome::PartnershipActivities { .. }
            | FederalIncome::Scorp { .. }
            | FederalIncome::ScorpActivities { .. }
            | FederalIncome::PersonalDebt { .. }
            | FederalIncome::PropertyAcquisition { .. }
            | FederalIncome::Nec1099 { .. }
            | FederalIncome::K1099 { .. }
            | FederalIncome::Misc1099 { .. }
            | FederalIncome::EducationDistribution { .. }
            | FederalIncome::MedicalDistribution { .. }
            | FederalIncome::LtcBenefits { .. }
            | FederalIncome::Able { .. }
            | FederalIncome::WildfireRelief { .. }
            | FederalIncome::DisasterRelief { .. } => true,
            _ => false,
        }
    } else {
        topic_of(d, entry) == Some(topic) && !source_in_editor(d, entry, TaxSection::Documents)
    }
}

fn record_kinds(topic: TaxSection) -> Option<Vec<i32>> {
    Some(match topic {
        TaxSection::Wages => alloc::vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::CashTipsWithoutW2) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Clergy) as i32,
        ],
        TaxSection::InterestDividends => alloc::vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::PrivateLoanInterest) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::SellerMortgageInterest) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::ForeignAccountOrTrust) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::TaxableBondPremium) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form8814) as i32,
        ],
        TaxSection::InvestmentSales => alloc::vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::MainHomeSale) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Section1256Carryback) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::CapitalLossCarryover) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::SmallBusinessStockLoss) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::WorthlessSecurity) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::BadDebt) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::InstallmentPayments) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::InstallmentSale) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::RelatedExchange) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::OpportunityFundDeferral)
                as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses) as i32,
        ],
        TaxSection::Retirement => alloc::vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::InheritedIra) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form8606) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Qcd) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form8915) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::SocialLumpSums) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::MissedRmd) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::RothHistory) as i32,
        ],
        TaxSection::SelfEmployment => alloc::vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::NonfarmOptionalMethod) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::SpouseSection179) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Business) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::QbiLossCarryforward) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::NetOperatingLoss) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses) as i32,
        ],
        // The Farming topic (T2-45p): the 1099-PATR moved here (F3); the
        // Schedule J and loss kinds are offered by topic_page (F7).
        TaxSection::Farming => alloc::vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::Farm) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::FarmRental) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::FarmAveraging) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::FarmLoss) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::SpouseSection179) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::QbiLossCarryforward) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::NetOperatingLoss) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses) as i32,
        ],
        TaxSection::Rentals => alloc::vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::Royalties) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Rental) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses) as i32,
        ],
        TaxSection::GovernmentBenefits => alloc::vec![],
        TaxSection::LongTermCareBenefits => {
            alloc::vec![libertas_macros::variant_index!(crate::IncomeSourceKind::LtcInsured) as i32,]
        }
        TaxSection::OtherIncome => alloc::vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::OtherIncome) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::EquipmentRental) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::AlimonyReceived) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::EducationAccountExpenses)
                as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::AbleContributions) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Scholarship) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::JuryPay) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::HobbySales) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::CasualtyLivingInsurance)
                as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::DisasterMitigation) as i32,
        ],
        TaxSection::Gambling => {
            alloc::vec![
                libertas_macros::variant_index!(crate::IncomeSourceKind::GamblingLosses) as i32,
            ]
        }
        TaxSection::PassThrough => alloc::vec![],
        TaxSection::CanceledDebt => alloc::vec![libertas_macros::variant_index!(
            crate::IncomeSourceKind::HealthLoanRepayment
        ) as i32,],
        TaxSection::WorkAbroad => alloc::vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form2555) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::ForeignPay) as i32,
        ],
        _ => return None,
    })
}
fn document_index(document: &IncomeSource) -> i32 {
    (match document {
        IncomeSource::Record { value: crate::IncomeRecord::CashTipsWithoutW2(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::CashTipsWithoutW2)
        }
        IncomeSource::Record { value: crate::IncomeRecord::PrivateLoanInterest(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::PrivateLoanInterest)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Clergy(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Clergy)
        }
        IncomeSource::Record { value: crate::IncomeRecord::NonfarmOptionalMethod(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::NonfarmOptionalMethod)
        }
        IncomeSource::Record { value: crate::IncomeRecord::SpouseSection179(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::SpouseSection179)
        }
        IncomeSource::Wage { value: crate::WageDocument::W2(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::W2)
        }
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Int1099)
        }
        IncomeSource::Record { value: crate::IncomeRecord::SellerMortgageInterest(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::SellerMortgageInterest)
        }
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Div1099(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Div1099)
        }
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Oid1099(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Oid1099)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Form8814(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form8814)
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099B(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099B)
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099DA(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099DA)
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Sale1099B(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Sale1099B)
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::R1099(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::R1099)
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::Ssa1099(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Ssa1099)
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Rrb1099)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Form8606(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form8606)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Qcd(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Qcd)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Business(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Business)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Rental(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Rental)
        }
        IncomeSource::GovernmentPayment { .. } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099G)
        }
        IncomeSource::Farming(_) => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Patr1099)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Royalties(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Royalties)
        }
        IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::EquipmentRental)
        }
        IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::OtherIncome)
        }
        IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099Nec(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099Nec)
        }
        IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099K(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099K)
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099Misc)
        }
        IncomeSource::Record { value: crate::IncomeRecord::TaxableBondPremium(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::TaxableBondPremium)
        }
        IncomeSource::Record { value: crate::IncomeRecord::ForeignAccountOrTrust(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::ForeignAccountOrTrust)
        }
        IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::CapitalLossCarryover)
        }
        IncomeSource::Record { value: crate::IncomeRecord::QbiLossCarryforward(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::QbiLossCarryforward)
        }
        IncomeSource::Record { value: crate::IncomeRecord::NetOperatingLoss(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::NetOperatingLoss)
        }
        IncomeSource::Record { value: crate::IncomeRecord::BusinessPropertyLosses(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses)
        }
        IncomeSource::Gambling(_) => libertas_macros::variant_index!(crate::IncomeSourceKind::W2g),
        IncomeSource::Record { value: crate::IncomeRecord::GamblingLosses(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::GamblingLosses)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Farm(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Farm)
        }
        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::FarmRental)
        }
        IncomeSource::Record { value: crate::IncomeRecord::FarmAveraging(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::FarmAveraging)
        }
        IncomeSource::Record { value: crate::IncomeRecord::FarmLoss(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::FarmLoss)
        }
        IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::K1EstateTrust)
        }
        IncomeSource::K1 { value: crate::K1Document::K1PartnershipActivities(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::K1PartnershipActivities)
        }
        IncomeSource::K1 { value: crate::K1Document::K1ScorpActivities(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::K1ScorpActivities)
        }
        IncomeSource::K1 { value: crate::K1Document::K1Scorp(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::K1Scorp)
        }
        IncomeSource::K1 { value: crate::K1Document::K1Partnership(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::K1Partnership)
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form3921(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form3921)
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form3922(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form3922)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Form8915(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form8915)
        }
        IncomeSource::Record { value: crate::IncomeRecord::SocialLumpSums(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::SocialLumpSums)
        }
        IncomeSource::Record { value: crate::IncomeRecord::MissedRmd(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::MissedRmd)
        }
        IncomeSource::Record { value: crate::IncomeRecord::RothHistory(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::RothHistory)
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::IsoShareSale(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::IsoShareSale)
        }
        IncomeSource::Record { value: crate::IncomeRecord::InstallmentPayments(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::InstallmentPayments)
        }
        IncomeSource::Record { value: crate::IncomeRecord::InstallmentSale(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::InstallmentSale)
        }
        IncomeSource::CanceledDebt { value: crate::CanceledDebtDocument::Cancel1099C(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Cancel1099C)
        }
        IncomeSource::CanceledDebt { value: crate::CanceledDebtDocument::Acquisition1099A(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Acquisition1099A)
        }
        IncomeSource::Record { value: crate::IncomeRecord::AlimonyReceived(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::AlimonyReceived)
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099S(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099S)
        }
        IncomeSource::Record { value: crate::IncomeRecord::MainHomeSale { .. } } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::MainHomeSale)
        }
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::SavingsBonds(_),
        } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::SavingsBonds)
        }
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Form8818(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form8818)
        }
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Contracts1099B(_),
        } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Contracts1099B)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Section1256Carryback(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Section1256Carryback)
        }
        IncomeSource::Record { value: crate::IncomeRecord::RelatedExchange(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::RelatedExchange)
        }
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::OpportunityFund(_),
        } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::OpportunityFund)
        }
        IncomeSource::Record { value: crate::IncomeRecord::OpportunityFundDeferral(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::OpportunityFundDeferral)
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Q1099)
        }
        IncomeSource::Record { value: crate::IncomeRecord::EducationAccountExpenses(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::EducationAccountExpenses)
        }
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EmployeeShareSale(_),
        } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::EmployeeShareSale)
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::EsppShareSale(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::EsppShareSale)
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::NuaShareSale(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::NuaShareSale)
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::QsbsSale(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::QsbsSale)
        }
        IncomeSource::Record { value: crate::IncomeRecord::SmallBusinessStockLoss(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::SmallBusinessStockLoss)
        }
        IncomeSource::Record { value: crate::IncomeRecord::WorthlessSecurity(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::WorthlessSecurity)
        }
        IncomeSource::Record { value: crate::IncomeRecord::BadDebt(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::BadDebt)
        }
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Form2439(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form2439)
        }
        IncomeSource::Wage { value: crate::WageDocument::Form8919(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form8919)
        }
        IncomeSource::Record { value: crate::IncomeRecord::HealthLoanRepayment(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::HealthLoanRepayment)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Scholarship(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Scholarship)
        }
        IncomeSource::Record { value: crate::IncomeRecord::JuryPay(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::JuryPay)
        }
        IncomeSource::Record { value: crate::IncomeRecord::HobbySales(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::HobbySales)
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::WildfireRelief(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::WildfireRelief)
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::DisasterRelief(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::DisasterRelief)
        }
        IncomeSource::Record { value: crate::IncomeRecord::CasualtyLivingInsurance(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::CasualtyLivingInsurance)
        }
        IncomeSource::Record { value: crate::IncomeRecord::DisasterMitigation(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::DisasterMitigation)
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099R(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Rrb1099R)
        }
        IncomeSource::Record { value: crate::IncomeRecord::InheritedIra(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::InheritedIra)
        }
        IncomeSource::Record { value: crate::IncomeRecord::Form2555(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form2555)
        }
        IncomeSource::Record { value: crate::IncomeRecord::ForeignPay(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::ForeignPay)
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Sa1099(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Sa1099)
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::HsaTrusteeStatement(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::HsaTrusteeStatement)
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Ltc1099)
        }
        IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::LtcInsured)
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Qa1099(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::Qa1099)
        }
        IncomeSource::Record { value: crate::IncomeRecord::AbleContributions(_) } => {
            libertas_macros::variant_index!(crate::IncomeSourceKind::AbleContributions)
        }
    }) as i32
}
pub(super) fn savings_bond_exclusions(
    draft: &FederalDraft,
    owner: crate::DocumentOwner,
) -> Vec<crate::RecordChoice> {
    draft
        .income
        .iter()
        .filter(|entry| {
            owner != crate::DocumentOwner::Unknown
                && entry.owner == owner
                && matches!(entry.income, FederalIncome::SavingsBonds { .. })
        })
        .map(|entry| crate::RecordChoice {
            id: entry.id,
            label: income_display_label(entry, draft.dependents()).encode(),
        })
        .collect()
}

// The income topic whose page shows a stored record; None for a kind no
// topic page shows yet.
pub(crate) fn topic_of(d: &FederalDraft, entry: &FederalIncomeEntry) -> Option<TaxSection> {
    match &entry.income {
        FederalIncome::Wage { .. } | FederalIncome::EmployeeWages { .. } => Some(TaxSection::Wages),
        FederalIncome::Interest { .. }
        | FederalIncome::Dividend { .. }
        | FederalIncome::Oid { .. }
        | FederalIncome::SavingsBonds { .. }
        | FederalIncome::BondRedemption { .. }
        | FederalIncome::ReturnOfCapital { .. }
        | FederalIncome::UndistributedGain { .. }
        | FederalIncome::ChildElection { .. } => Some(TaxSection::InterestDividends),
        FederalIncome::InstallmentSale { .. }
        | FederalIncome::Broker { .. }
        | FederalIncome::Sale { .. }
        | FederalIncome::IsoSale { .. }
        | FederalIncome::EmployeeStockSale { .. }
        | FederalIncome::EsppSale { .. }
        | FederalIncome::EsppSaleSource { .. }
        | FederalIncome::IsoSaleSource { .. }
        | FederalIncome::NuaSaleSource { .. }
        | FederalIncome::NuaSale { .. }
        | FederalIncome::QualifiedSmallBusinessSale { .. }
        | FederalIncome::SmallBusinessStock { .. }
        | FederalIncome::WorthlessSecurity { .. }
        | FederalIncome::NonbusinessBadDebt { .. }
        | FederalIncome::HomeSale { .. }
        | FederalIncome::OpportunityFund { .. }
        | FederalIncome::OpportunityFundDeferral { .. }
        | FederalIncome::Section1256 { .. } => Some(TaxSection::InvestmentSales),
        FederalIncome::Retirement { .. }
        | FederalIncome::Roth { .. }
        | FederalIncome::TrumpDeath { .. }
        | FederalIncome::TrumpAbleRollover { .. }
        | FederalIncome::InheritedIra { .. }
        | FederalIncome::SocialSecurity { .. } => Some(TaxSection::Retirement),
        FederalIncome::Business { data }
            if matches!(
                data.activity,
                BusinessActivity::Ordinary { .. } | BusinessActivity::Gambling { .. }
            ) =>
        {
            Some(TaxSection::SelfEmployment)
        }
        FederalIncome::Business { data }
            if matches!(data.activity, BusinessActivity::Farming { .. }) =>
        {
            Some(TaxSection::Farming)
        }
        FederalIncome::Cooperative { .. } => {
            let source_claims = crate::related_forms::effective_claimants(d);
            let recipients = crate::cooperative::sources_with_claims(d, &source_claims)
                .into_iter()
                .filter(|source| source.parent == entry.id)
                .filter_map(|source| source.activity)
                .collect::<Vec<_>>();
            Some(
                if !recipients.is_empty()
                    && recipients.iter().all(|id| {
                        d.income.iter().any(|e| {
                            e.id == *id
                                && matches!(&e.income, FederalIncome::Business {data}
                    if matches!(data.activity, BusinessActivity::Ordinary{..}))
                        })
                    })
                {
                    TaxSection::SelfEmployment
                } else {
                    TaxSection::Farming
                },
            )
        }
        FederalIncome::FarmRental { .. } => Some(TaxSection::Farming),
        FederalIncome::Nec1099 { .. } | FederalIncome::K1099 { .. } => {
            Some(TaxSection::SelfEmployment)
        }
        FederalIncome::Rental { .. } | FederalIncome::Royalty { .. } => Some(TaxSection::Rentals),
        FederalIncome::GovernmentPayment { .. } => Some(TaxSection::GovernmentBenefits),
        FederalIncome::OtherIncome { .. }
        | FederalIncome::Misc1099 { .. }
        | FederalIncome::NonbusinessProperty { .. }
        | FederalIncome::Alimony { .. }
        | FederalIncome::EducationDistribution { .. }
        | FederalIncome::EducationAccountExpenses { .. }
        | FederalIncome::Scholarship { .. }
        | FederalIncome::JuryDuty { .. }
        | FederalIncome::HobbySales { .. }
        | FederalIncome::WildfireRelief { .. }
        | FederalIncome::DisasterRelief { .. }
        | FederalIncome::CasualtyLivingInsurance { .. }
        | FederalIncome::DisasterMitigation { .. }
        | FederalIncome::MedicalDistribution { .. }
        | FederalIncome::LtcBenefits { .. }
        | FederalIncome::Able { .. } => Some(TaxSection::OtherIncome),
        FederalIncome::LtcInsured { .. } => Some(TaxSection::LongTermCareBenefits),
        FederalIncome::Gambling { .. } => Some(TaxSection::Gambling),
        FederalIncome::EstateTrustPortfolio { .. }
        | FederalIncome::Scorp { .. }
        | FederalIncome::Partnership { .. }
        | FederalIncome::PartnershipActivities { .. }
        | FederalIncome::ScorpActivities { .. } => Some(TaxSection::PassThrough),
        FederalIncome::PersonalDebt { .. }
        | FederalIncome::PropertyAcquisition { .. }
        | FederalIncome::HealthLoanAssistance { .. } => Some(TaxSection::CanceledDebt),
        FederalIncome::ForeignEmployment { .. } => Some(TaxSection::WorkAbroad),
        _ => None,
    }
}

// A document's kind: its summary noun resources (SAVING_KIND_*) and its display
// name resource (LABEL_ENTRY_*, the capitalized noun over the entry identity).
// An entry stores only the name the person typed; an unnamed source is named by
// the second, built in one place (income_context).
pub(crate) use crate::income_names::{
    document_kind_keys, income_display_arg, income_display_label, record_display_arg,
};
fn document_warning_kind(entry: &FederalIncomeEntry) -> &'static str {
    document_kind_keys(entry).0
}
// Keep list separators in the locale template. A balanced tree bounds tuple
// nesting even when a return contains many different document kinds.
fn message_list(items: &[crate::Message]) -> crate::Message {
    use crate::{Message, MessageArg};
    match items {
        [] => Message::new("REVIEW_NOTHING_ENTERED", alloc::vec![]),
        [item] => item.clone(),
        _ => {
            let middle = items.len() / 2;
            Message::new(
                "SUMMARY_LIST_PAIR",
                alloc::vec![
                    MessageArg::Nested(alloc::boxed::Box::new(message_list(&items[..middle]))),
                    MessageArg::Nested(alloc::boxed::Box::new(message_list(&items[middle..]))),
                ],
            )
        }
    }
}
// Documents counted by kind; counts and both localized noun forms retain
// their meanings until the client renders the complete template.
fn document_summary<'a>(entries: impl Iterator<Item = &'a FederalIncomeEntry>) -> crate::Message {
    let mut counts: Vec<(&'static str, usize)> = Vec::new();
    for entry in entries {
        let kind = document_warning_kind(entry);
        match counts.iter_mut().find(|(k, _)| *k == kind) {
            Some((_, n)) => *n += 1,
            None => counts.push((kind, 1)),
        }
    }
    bounded_document_summary(counts.into_iter().map(|(kind, count)| (kind, count as i64)).collect())
}
// The most kinds a hub row names; a nested tuple is bounded to 4,096 bytes
// (message-format.md) and a hub row is one short sentence, not a list of every
// kind. About 119 bytes a kind, so six fit with room for the count of the rest.
const SUMMARY_KIND_LIMIT: usize = 6;
// The kinds with the highest counts first (stable otherwise, so entry order
// among equal counts), at most SUMMARY_KIND_LIMIT of them, and "and N more
// kinds" for the rest.
fn bounded_document_summary(mut counts: Vec<(&'static str, i64)>) -> crate::Message {
    use crate::{Message, MessageArg};
    counts.sort_by_key(|(_, count)| core::cmp::Reverse(*count));
    let more = counts.len().saturating_sub(SUMMARY_KIND_LIMIT);
    counts.truncate(SUMMARY_KIND_LIMIT);
    let mut phrases: Vec<Message> = counts
        .into_iter()
        .map(|(kind, count)| {
            Message::new(
                "SUMMARY_DOCUMENT_COUNT",
                alloc::vec![
                    MessageArg::Count(count),
                    MessageArg::Resource(alloc::format!("{kind}_ONE")),
                    MessageArg::Resource(alloc::format!("{kind}_MANY")),
                ],
            )
        })
        .collect();
    if more > 0 {
        phrases.push(Message::new(
            "SUMMARY_DOCUMENTS_MORE_KINDS",
            alloc::vec![MessageArg::Count(more as i64)],
        ));
    }
    message_list(&phrases)
}
// A farm's Schedule F line 34 from the calculation, in cents; None while
// the calculation has not produced it.
fn farm_line_34(result: &FederalResult, id: i64) -> Option<i64> {
    let row = alloc::format!("farm_{id}_net_profit");
    result.lines.iter().find(|l| l.id == row).map(|l| l.amount)
}
// A farm rental's Form 4835 line 32 income or line 34c allowed loss, the
// amount Schedule E line 40 takes, in cents.
fn farm_rental_line(result: &FederalResult, id: i64) -> Option<i64> {
    let find = |name: &str| {
        let row = alloc::format!("farm_rental_{id}_{name}");
        result.lines.iter().find(|l| l.id == row).map(|l| l.amount)
    };
    find("line34c_allowed_loss").or_else(|| find("net_income"))
}
// The Farming row (T2-45p): the count and the total of the farms' line 34
// (a routed 1099-PATR is already inside its farm's line 34).
fn farming_summary(entries: &[&FederalIncomeEntry], result: &FederalResult) -> crate::Message {
    let count = document_summary(entries.iter().copied());
    let profits: Vec<i64> = entries
        .iter()
        .filter_map(|e| match e.income {
            FederalIncome::Business { .. } => farm_line_34(result, e.id),
            FederalIncome::FarmRental { .. } => farm_rental_line(result, e.id),
            _ => None,
        })
        .collect();
    if profits.is_empty() {
        count
    } else {
        crate::Message::new(
            "SUMMARY_DOCUMENTS_WITH_AMOUNT",
            alloc::vec![
                crate::MessageArg::Nested(alloc::boxed::Box::new(count)),
                crate::MessageArg::usd_cents(profits.iter().sum()),
            ],
        )
    }
}
// A hub row's count and amount: "1 W-2 · $60,000.00", the sum of
// the documents' previews; a business shows no preview (its profit depends
// on the whole return), so the self-employment row shows the count.
pub(crate) fn topic_summary(entries: &[&FederalIncomeEntry], separate: bool) -> crate::Message {
    let count = document_summary(entries.iter().copied());
    let amounts: Vec<i64> = entries
        .iter()
        .filter(|e| !matches!(e.income, FederalIncome::Business { .. }))
        .filter_map(|e| document_amount(&e.income, separate))
        .collect();
    let total = amounts
        .iter()
        .try_fold(0_i128, |total, amount| total.checked_add(i128::from(*amount)))
        .and_then(|total| i64::try_from(total).ok());
    if let Some(total) = total.filter(|_| !amounts.is_empty()) {
        crate::Message::new(
            "SUMMARY_DOCUMENTS_WITH_AMOUNT",
            alloc::vec![
                crate::MessageArg::Nested(alloc::boxed::Box::new(count)),
                crate::MessageArg::usd_cents(total),
            ],
        )
    } else {
        count
    }
}
fn project_estate_issue(issue: ValidationIssue) -> ValidationIssue {
    issue.map_field_prefix(&["activities"], &["business"])
}
fn submitted_source_issue(
    id: i64,
    issue: ValidationIssue,
    origins: &[(i64, usize, bool)],
) -> ValidationIssue {
    let Some((_, index, _)) = origins.iter().find(|(source, _, _)| *source == id) else {
        return ValidationIssue::new(issue.key);
    };
    submitted_data_answer_issue(issue, *index)
}
fn submitted_graph_issue(
    failure: (i64, ValidationIssue),
    origins: &[(i64, usize, bool)],
    cooperative: bool,
) -> ValidationIssue {
    let (id, issue) = failure;
    let Some((_, index, automatic)) = origins.iter().find(|(source, _, _)| *source == id) else {
        // A retained source outside this request has no editable submitted row.
        return ValidationIssue::new(issue.key);
    };
    let issue = if cooperative {
        project_cooperative_issue(issue, *automatic)
    } else {
        project_estate_issue(issue)
    };
    issue.in_field("document").in_list("documents", *index).in_field("documents")
}
fn preview_graph_issue(
    failure: (i64, ValidationIssue),
    bound_id: i64,
    cooperative: bool,
    automatic: bool,
) -> ValidationIssue {
    let (id, issue) = failure;
    if id != bound_id {
        return ValidationIssue::new(issue.key);
    }
    let issue = if cooperative {
        project_cooperative_issue(issue, automatic)
    } else {
        project_estate_issue(issue)
    };
    issue.in_field("document")
}
fn is_adjustment_document(document: &IncomeSource) -> bool {
    matches!(
        document,
        IncomeSource::Record { value: crate::IncomeRecord::Form8606(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::Qcd(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::SocialLumpSums(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::MissedRmd(_) }
            | IncomeSource::Record { value: crate::IncomeRecord::RothHistory(_) }
    )
}

// The Schedule C page (unit M3-P5) is a projection of FederalBusiness. The
// page's vehicles, property, office and "more" answers each become the
// stored record's own facts; the reverse mapping rebuilds the page and is
// accepted only when saving it again reproduces the stored record exactly.
// The first day on or after which a continuing asset is no longer continuing.
const YEAR_FIRST_DAY: u32 = 20260101;
// A person's name, the first and last name the person typed.
pub(crate) use crate::person_names::person_name;
fn has_farm(d: &FederalDraft) -> bool {
    d.income.iter().any(|e| {
        matches!(e.income, FederalIncome::Business { .. })
            && topic_of(d, e) == Some(TaxSection::Farming)
    })
}

pub(crate) fn document_income(document: &IncomeSource) -> FederalIncome {
    match document {
        IncomeSource::Record { value: crate::IncomeRecord::CashTipsWithoutW2(v) } => {
            FederalIncome::Wage { data: crate::WageSource::CashTipsWithoutW2(v.clone()) }
        }
        IncomeSource::Record { value: crate::IncomeRecord::PrivateLoanInterest(v) } => {
            FederalIncome::Interest {
                data: crate::InterestSource::Record(FederalInterest {
                    private_loan: true,
                    taxable: v.interest,
                    seller_mortgage: v.buyer(),
                    ..Default::default()
                }),
            }
        }
        IncomeSource::Wage { value: crate::WageDocument::W2(w) } => {
            FederalIncome::Wage { data: crate::WageSource::W2(crate::W2Source::printed(w.clone())) }
        }
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(v) } => {
            FederalIncome::Interest {
                data: crate::InterestSource::Int1099(crate::Form1099IntSource::printed(v.clone())),
            }
        }
        IncomeSource::Record { value: crate::IncomeRecord::SellerMortgageInterest(v) } => {
            FederalIncome::Interest {
                data: crate::InterestSource::Record(seller_interest_record(v)),
            }
        }
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Div1099(v) } => {
            FederalIncome::Dividend { data: crate::DividendSource::printed(v.clone()) }
        }
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Oid1099(v) } => {
            FederalIncome::Oid { data: crate::Form1099OidSource::printed(v.clone()) }
        }
        IncomeSource::Record { value: crate::IncomeRecord::Form8814(v) } => {
            FederalIncome::ChildElection { data: v.record() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::InstallmentSale(v) } => {
            FederalIncome::InstallmentSale { data: v.clone() }
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099B(v) } => {
            FederalIncome::Broker {
                data: crate::BrokerPaper::Form1099B(v.clone()).into_document(None),
            }
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099DA(v) } => {
            FederalIncome::Broker {
                data: crate::BrokerPaper::Form1099DA(v.clone()).into_document(None),
            }
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Sale1099B(v) } => {
            FederalIncome::Sale { data: crate::SaleSource::printed(v.clone()) }
        }
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::OpportunityFund(v),
        } => FederalIncome::OpportunityFund { data: crate::QofInvestment::printed(v.clone()) },
        IncomeSource::Record { value: crate::IncomeRecord::OpportunityFundDeferral(v) } => {
            FederalIncome::OpportunityFundDeferral { data: v.clone() }
        }
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Contracts1099B(v),
        } => FederalIncome::Section1256 { data: crate::ContractsSource::printed(v.clone()) },
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::IsoShareSale(v) } => {
            FederalIncome::IsoSaleSource { data: v.clone() }
        }
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EmployeeShareSale(v),
        } => FederalIncome::EmployeeStockSale { data: v.record() },
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::EsppShareSale(v) } => {
            FederalIncome::EsppSaleSource { data: v.clone() }
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::NuaShareSale(v) } => {
            FederalIncome::NuaSaleSource { data: v.clone() }
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::QsbsSale(v) } => {
            FederalIncome::QualifiedSmallBusinessSale { data: v.record() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::SmallBusinessStockLoss(v) } => {
            FederalIncome::SmallBusinessStock { data: v.clone() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::WorthlessSecurity(v) } => {
            FederalIncome::WorthlessSecurity { data: v.clone() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::BadDebt(v) } => {
            FederalIncome::NonbusinessBadDebt { data: v.clone() }
        }
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Form2439(v) } => {
            FederalIncome::UndistributedGain {
                data: crate::UndistributedGainSource::printed(v.clone()),
            }
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099S(v) } => {
            FederalIncome::HomeSale { data: crate::MainHomeSale::printed(v.clone()) }
        }
        IncomeSource::Record { value: crate::IncomeRecord::MainHomeSale { sale, answers } } => {
            FederalIncome::HomeSale {
                data: crate::MainHomeSale::records(sale.clone(), answers.clone()),
            }
        }
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::SavingsBonds(v),
        } => FederalIncome::SavingsBonds { data: crate::SavingsBondsSource::printed(v.clone()) },
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Form8818(v) } => {
            FederalIncome::BondRedemption {
                data: crate::BondRedemptionSource { form: v.clone(), form_8815: None },
            }
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(v) } => {
            FederalIncome::EducationDistribution {
                data: crate::EducationDistributionSource::printed(v.clone()),
            }
        }
        IncomeSource::Record { value: crate::IncomeRecord::EducationAccountExpenses(v) } => {
            FederalIncome::EducationAccountExpenses { data: education_expenses_record(v) }
        }
        // The treatment answer files a form under the Roth IRA, Trump account or other kind.
        IncomeSource::Retirement { value: crate::RetirementDocument::R1099(v) } => {
            R1099Source::printed(crate::R1099::from(v)).income()
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099R(v) } => {
            FederalIncome::Retirement {
                data: crate::RetirementSource::Rrb1099R(crate::RailroadSource::printed(
                    crate::Rrb1099R::from(v),
                )),
            }
        }
        IncomeSource::Record { value: crate::IncomeRecord::InheritedIra(v) } => {
            FederalIncome::InheritedIra { data: v.record() }
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Sa1099(v) } => {
            FederalIncome::MedicalDistribution { data: crate::SaSource::printed(v.clone()) }
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::HsaTrusteeStatement(v) } => {
            FederalIncome::MedicalDistribution {
                data: crate::SaSource::trustee_statement(v.clone()),
            }
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(v) } => {
            FederalIncome::LtcBenefits { data: crate::LtcSource::printed(v.clone()) }
        }
        IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(v) } => {
            FederalIncome::LtcInsured { data: v.record() }
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Qa1099(v) } => {
            FederalIncome::Able { data: crate::AbleSource::printed(v.clone()) }
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::Ssa1099(v) } => {
            FederalIncome::SocialSecurity {
                data: crate::SocialBenefitStatement::Ssa1099(v.clone()),
            }
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099(v) } => {
            FederalIncome::SocialSecurity {
                data: crate::SocialBenefitStatement::Rrb1099(v.clone()),
            }
        }
        // Form 8606 and QCD gifts affect Adjustments, not an income entry;
        // documents_candidate and PreviewDocument split them out before
        // calling document_income.
        adjustment_kinds!() => {
            unreachable!(
                "Form 8606, QCD gifts and the retirement adjustments are not income entries"
            )
        }
        // The data answers are FederalScreening facts, split out the same way.
        data_answer_kinds!() => {
            unreachable!("the data answers are not income entries")
        }
        IncomeSource::Record { value: crate::IncomeRecord::Farm(f) } => {
            FederalIncome::Business { data: crate::farm_page::farm_record(f, None) }
        }
        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(f) } => {
            FederalIncome::FarmRental { data: crate::farm_page::farm_rental_record(f) }
        }
        IncomeSource::Record { value: crate::IncomeRecord::Business(b) } => {
            FederalIncome::Business { data: business_record(b, None) }
        }
        IncomeSource::Record { value: crate::IncomeRecord::Rental(r) } => {
            FederalIncome::Rental { data: rental_record(r) }
        }
        IncomeSource::GovernmentPayment { value: v, refund_state, refund_locality } => {
            let mut data = crate::GovernmentPaymentSource::printed(v.clone());
            // Filled once on the Document Center save: a state refund is paid by that
            // state's agency, so an unset state is read from its printed address. A
            // recorded state stays, whatever later edits the paper receives.
            data.refund_state = refund_state.or_else(|| {
                (v.state_or_local_income_tax_refunds != 0)
                    .then(|| crate::us_address::UsAddress::parse(&v.payer.address))
                    .flatten()
                    .map(|address| address.state)
            });
            data.refund_locality = refund_locality.clone();
            FederalIncome::GovernmentPayment { data }
        }
        IncomeSource::Farming(v) => {
            FederalIncome::Cooperative { data: crate::Patr1099Source::printed(v.clone()) }
        }
        IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(v) } => {
            FederalIncome::EstateTrustPortfolio { data: v.record() }
        }
        IncomeSource::K1 { value: crate::K1Document::K1PartnershipActivities(v) } => {
            FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(v.record()) }
        }
        IncomeSource::K1 { value: crate::K1Document::K1ScorpActivities(v) } => {
            FederalIncome::ScorpActivities { data: v.record() }
        }
        IncomeSource::K1 { value: crate::K1Document::K1Scorp(v) } => {
            FederalIncome::Scorp { data: v.record() }
        }
        IncomeSource::K1 { value: crate::K1Document::K1Partnership(v) } => {
            FederalIncome::Partnership { data: v.record() }
        }
        IncomeSource::CanceledDebt { value: crate::CanceledDebtDocument::Cancel1099C(v) } => {
            FederalIncome::PersonalDebt { data: crate::PersonalDebtSource::printed(v.clone()) }
        }
        IncomeSource::CanceledDebt { value: crate::CanceledDebtDocument::Acquisition1099A(v) } => {
            FederalIncome::PropertyAcquisition {
                data: crate::PropertyAcquisitionSource::printed(v.clone()),
            }
        }
        IncomeSource::Record { value: crate::IncomeRecord::AlimonyReceived(v) } => {
            FederalIncome::Alimony { data: v.clone() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::ForeignPay(v) } => {
            FederalIncome::ForeignEmployment { data: crate::ForeignWages { wages: v.wages } }
        }
        IncomeSource::Wage { value: crate::WageDocument::Form8919(v) } => {
            FederalIncome::EmployeeWages { data: v.clone() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::HealthLoanRepayment(v) } => {
            FederalIncome::HealthLoanAssistance { data: v.clone() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::Scholarship(v) } => {
            FederalIncome::Scholarship { data: v.clone() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::JuryPay(v) } => {
            FederalIncome::JuryDuty { data: v.clone() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::HobbySales(v) } => {
            FederalIncome::HobbySales {
                data: crate::FederalHobbySales {
                    related: v.related.forms.iter().map(|form| form.key).collect(),
                    receipts: v.receipts,
                    inventory: v.inventory.clone(),
                    ordinary: v.ordinary,
                },
            }
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::WildfireRelief(v) } => {
            FederalIncome::WildfireRelief { data: v.clone() }
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::DisasterRelief(v) } => {
            FederalIncome::DisasterRelief { data: v.clone() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::CasualtyLivingInsurance(v) } => {
            FederalIncome::CasualtyLivingInsurance { data: v.clone() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::DisasterMitigation(v) } => {
            FederalIncome::DisasterMitigation { data: v.clone() }
        }
        IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(v) } => {
            FederalIncome::OtherIncome { data: v.clone() }
        }
        IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099Nec(v) } => {
            FederalIncome::Nec1099 { data: crate::Nec1099Source::printed(v.clone()) }
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(v) } => {
            FederalIncome::Misc1099 { data: crate::Misc1099Source::printed(v.clone()) }
        }
        IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099K(v) } => {
            FederalIncome::K1099 { data: crate::K1099Source::printed(v.clone()) }
        }
        IncomeSource::Gambling(v) => FederalIncome::Gambling {
            data: crate::GamblingSource::W2g(crate::W2gSource::printed(v.clone()).into()),
        },
        IncomeSource::Record { value: crate::IncomeRecord::GamblingLosses(v) } => {
            FederalIncome::Gambling { data: crate::GamblingSource::OwnRecords(v.clone()) }
        }
        IncomeSource::Record { value: crate::IncomeRecord::Royalties(v) } => {
            FederalIncome::Royalty {
                data: crate::FederalRoyalty {
                    related: v.related.forms.iter().map(|form| form.key).collect(),
                    kind: v.kind.clone(),
                    receipts: v.receipts,
                    advance: v.advance,
                    expenses: rental_expense_records(&v.expenses),
                    depletion: v.depletion.clone(),
                    ordinary: !v.special,
                },
            }
        }
        IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(v) } => {
            FederalIncome::NonbusinessProperty {
                kind: NonbusinessPropertyKind::PersonalRental {
                    related: v.related.forms.iter().map(|form| form.key).collect(),
                    basis: v.basis,
                    depreciable_basis: v.depreciable_basis,
                    assets: v.assets.iter().map(|asset| asset_record(asset, false)).collect(),
                    history: v.history,
                },
                receipts: v.receipts,
                expenses: v.expenses,
                ordinary: true,
            }
        }
    }
}
pub(crate) fn estate_answer_issue(draft: &FederalDraft) -> Option<(i64, ValidationIssue)> {
    draft
        .income
        .iter()
        .find_map(|entry| match &entry.income {
            FederalIncome::EstateTrustPortfolio { data } => {
                let data = &crate::FederalEstateTrustPortfolioK1::from(data);
                estate_followups::answer_issue(data, true).map(|issue| (entry.id, issue))
            }
            _ => None,
        })
        .or_else(|| crate::estate_trust::graph_issue(draft))
}

// The data answers of M3-P15: kinds of an income topic that are
// FederalScreening facts, not income entries.
fn is_data_answer(document: &IncomeSource) -> bool {
    matches!(document, data_answer_kinds!())
}
// A topic page's data answers onto the FederalScreening fields that topic
// owns; a topic without data answers owns none.
enum DataAnswers {
    None,
    Clergy(Vec<crate::ClergyClaim>),
    Foreign {
        account: Option<ForeignAccountOrTrust>,
        premium: Option<bool>,
    },
    // Investment sales owns prior-return and non-document history.
    InvestmentSales {
        capital: Vec<crate::PriorCapitalReturn>,
        installments: Vec<crate::EarlierInstallment>,
        carryback: Option<crate::Section1256Carryback>,
        related: Vec<crate::EarlierRelatedExchange>,
    },
    QbiLoss(Option<QbiLossCarryforward>),
    // Self-employment: the QBI carryforward and the earlier-year operating
    // losses it shares with Farming.
    SelfEmployment {
        qbi: Option<QbiLossCarryforward>,
        losses: Option<crate::LossCarryforward>,
        nonfarm: Vec<(Owner, crate::NonfarmSeHistory)>,
    },
    // The Retirement topic: earlier Forms 8915-F (T3-8915).
    Disaster(Vec<crate::DisasterCarry>),
    // The Work abroad topic: each person's Form 2555.
    WorkAbroad(Vec<crate::ForeignEarnedClaim>),
    // The Other income topic: each ABLE beneficiary's contributions (unit B6a).
    Able(Vec<crate::AbleContributions>),
    // The Farming topic (T2-45p): the QBI carryforward it shares with
    // Self-employment, Schedule J and the farm loss choice.
    Farming {
        qbi: Option<QbiLossCarryforward>,
        losses: Option<crate::LossCarryforward>,
        averaging: Option<crate::FarmAveraging>,
        nol: Option<crate::FarmNolElection>,
    },
}
// The operating-loss kind onto Form 172's and Form 6251 line 2f's histories;
// none entered is no history.
fn apply_losses(screening: &mut FederalScreening, losses: Option<crate::LossCarryforward>) {
    (screening.nol, screening.atnol) = match losses {
        Some(losses) => losses.histories(),
        None => (NolHistory::None, AtnolHistory::None),
    };
}
impl DataAnswers {
    fn apply(self, screening: &mut FederalScreening) {
        match self {
            Self::None => {}
            Self::Foreign { account: value, premium } => {
                screening.taxable_premium_election = premium;
                let value = value.unwrap_or(ForeignAccountOrTrust {
                    account: false,
                    fbar_required: None,
                    fbar_countries: Vec::new(),
                    trust: false,
                    highest_value: None,
                });
                screening.foreign_account = value.account;
                screening.fbar_required = value.fbar_required;
                screening.fbar_countries = value.fbar_countries;
                screening.foreign_trust = value.trust;
                screening.foreign_asset_max =
                    value.highest_value.filter(|_| value.account || value.trust);
            }
            Self::InvestmentSales { capital, installments, carryback, related } => {
                screening.capital_history = capital;
                screening.installment_payments = installments;
                screening.section_1256_carryback = carryback.as_ref().map(|v| v.amount);
                screening.section_1256_carryback_owners = carryback.and_then(|v| v.injured_owners);
                screening.related_exchanges = related;
            }
            Self::SelfEmployment { qbi, losses, nonfarm } => {
                Self::QbiLoss(qbi).apply(screening);
                apply_losses(screening, losses);
                for election in &mut screening.optional_se {
                    election.nonfarm = None;
                }
                for (owner, history) in nonfarm {
                    if let Some(election) =
                        screening.optional_se.iter_mut().find(|e| e.owner == owner)
                    {
                        election.nonfarm = Some(history);
                    } else {
                        screening.optional_se.push(SeElection {
                            owner,
                            farm: false,
                            nonfarm: Some(history),
                        });
                    }
                }
                screening.optional_se.retain(|e| e.farm || e.nonfarm.is_some());
            }
            Self::Farming { qbi, losses, averaging, nol } => {
                Self::QbiLoss(qbi).apply(screening);
                apply_losses(screening, losses);
                screening.farm_averaging = averaging;
                screening.farm_nol = nol;
            }
            Self::QbiLoss(value) => {
                screening.qbi_history = value.map_or(QbiHistory::NONE, |value| QbiHistory {
                    business: value.business,
                    reit_ptp: value.reit_ptp,
                    injured_owners: value.injured_owners.clone(),
                })
            }
            Self::Disaster(records) => screening.disaster_carries = records,
            Self::WorkAbroad(records) => screening.foreign_earned = records,
            Self::Clergy(records) => screening.clergy = records,
            Self::Able(records) => screening.able_contributions = records,
        }
    }
}
// The page's data answers, with their server checks: at most one foreign
// account item, one QBI carryforward and two 2025 returns; one real estate
// professional per owner, your spouse only on a joint return, and only with
// a rental property on the page.
// The section 1231 lookback (Form 4797 line 9) is one record for the whole
// return, shown and saved on both the self-employment and rentals pages;
// None for a page that does not show it.
fn lookback_records(
    topic: TaxSection,
    items: &[IncomeDocumentItem],
) -> Result<Option<Option<crate::Section1231History>>, SaveFailure> {
    if !matches!(
        topic,
        TaxSection::SelfEmployment
            | TaxSection::Rentals
            | TaxSection::Farming
            | TaxSection::InvestmentSales
    ) {
        return Ok(None);
    }
    let mut found = items.iter().enumerate().filter_map(|(index, item)| match &item.document {
        IncomeSource::Record { value: crate::IncomeRecord::BusinessPropertyLosses(value) } => {
            Some((index, value.history()))
        }
        _ => None,
    });
    let first = found.next();
    if let Some((index, _)) = found.next() {
        return Err(submitted_data_answer_issue(
            ValidationIssue::new("BUSINESS_PROPERTY_LOSSES_TWICE").at_record(),
            index,
        )
        .into());
    }
    if first.as_ref().is_some_and(|(_, value)| !value.valid()) {
        return Err(crate::interview::status(
            "MESSAGE_ENTER_EACH_LOSS_YEAR_FROM_2021_THROUGH_2025_ONCE_WITH_A_REMAINING_BALANCE_NO",
        )
        .into());
    }
    Ok(Some(first.map(|(_, value)| value)))
}
// The page that offers the spouse's section 179 records (separate return;
// Pub. 946, Married Individuals): the one holding the business, while the
// return uses section 179.
fn spouse_179_page(d: &FederalDraft) -> Option<TaxSection> {
    crate::section179::spouse_records_asked(d)
        .then(|| issue_page(d, &crate::IssueContext::Business, None))
        .flatten()
}
// The spouse's section 179 records (Form 4562; separate return) are one record
// for the whole return, saved from the page that offers them: the record's
// records, or records not given when the page is saved without it; None for
// a page that does not offer it.
fn spouse_179_records(
    d: &FederalDraft,
    topic: TaxSection,
    items: &[IncomeDocumentItem],
) -> Result<Option<crate::Section179Spouse>, SaveFailure> {
    if !matches!(topic, TaxSection::SelfEmployment | TaxSection::Farming) {
        return Ok(None);
    }
    if let Some(issue) = repeated_document_issue(items, "SPOUSE_SECTION179_TWICE", |document| {
        matches!(document, IncomeSource::Record { value: crate::IncomeRecord::SpouseSection179(_) })
    }) {
        return Err(issue.into());
    }
    let Some((index, records)) =
        items.iter().enumerate().find_map(|(index, item)| match &item.document {
            IncomeSource::Record { value: crate::IncomeRecord::SpouseSection179(value) } => {
                Some((index, value.records()))
            }
            _ => None,
        })
    else {
        return Ok(
            (spouse_179_page(d) == Some(topic)).then_some(crate::Section179Spouse::NotSupplied)
        );
    };
    if !records.valid() {
        return Err(submitted_data_answer_issue(
            ValidationIssue::new("SECTION179_SPOUSE_SOURCE_INVALID").at_field("records"),
            index,
        )
        .into());
    }
    Ok(Some(records))
}
// The QBI loss carryforward kind, shared by Self-employment and Farming.
fn qbi_loss_answer(
    items: &[IncomeDocumentItem],
) -> Result<Option<QbiLossCarryforward>, SaveFailure> {
    let mut found = items.iter().enumerate().filter_map(|(index, item)| match &item.document {
        IncomeSource::Record { value: crate::IncomeRecord::QbiLossCarryforward(value) } => {
            Some((index, value))
        }
        _ => None,
    });
    let first = found.next();
    if let Some((index, _)) = found.next() {
        return Err(submitted_data_answer_issue(
            ValidationIssue::new("QBI_LOSS_CARRYFORWARD_TWICE").at_record(),
            index,
        )
        .into());
    }
    if let Some((index, value)) = first
        && value
            .injured_owners
            .as_ref()
            .is_some_and(|owners| !owners.valid([value.business, value.reit_ptp]))
    {
        return Err(submitted_data_answer_issue(
            ValidationIssue::new("INJURED_QBI_OWNER_TOTAL").at_field("injured_owners"),
            index,
        )
        .into());
    }
    Ok(first.map(|(_, value)| value.clone()))
}

// The earlier-year operating-loss kind (u5-losses), shared by
// Self-employment and Farming: at most one (NOL_TWICE), with the server
// twins of its client rules and the stored ranges.
fn loss_answer(
    items: &[IncomeDocumentItem],
) -> Result<Option<crate::LossCarryforward>, crate::validation::SaveFailure> {
    let mut found = items.iter().enumerate().filter_map(|(index, item)| match &item.document {
        IncomeSource::Record { value: crate::IncomeRecord::NetOperatingLoss(value) } => {
            Some((index, value.clone()))
        }
        _ => None,
    });
    let first = found.next();
    if let Some((index, _)) = found.next() {
        return Err(ValidationIssue::new("NOL_TWICE")
            .at_field("document")
            .in_list("documents", index)
            .in_field("documents")
            .into());
    }
    if let Some((index, value)) = &first {
        if let Some(issue) = value.issue() {
            return Err(issue
                .in_field("document")
                .in_list("documents", *index)
                .in_field("documents")
                .into());
        }
        let (regular, amt) = value.histories();
        if !regular.valid() || !amt.valid() {
            return Err(crate::interview::status(
                "MESSAGE_NET_OPERATING_LOSS_ENTER_LOSS_YEARS_FROM_1900_THROUGH_2025_WITH_AMOUNTS_IN_T",
            ).into());
        }
    }
    Ok(first.map(|(_, value)| value))
}
fn form3922_records(
    items: &[IncomeDocumentItem],
    accepted: &FederalDraft,
) -> Result<Vec<crate::EsppAcquisition>, SaveFailure> {
    let mut records = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form3922(form) } =
            &item.document
        else {
            continue;
        };
        if let Some(issue) = form.issue() {
            return Err(issue
                .in_field("document")
                .in_list("documents", index)
                .in_field("documents")
                .into());
        }
        let fixed_grant_price = if form.grant_price.is_none() {
            accepted
                .screening
                .iter()
                .flat_map(|s| &s.espp_acquisitions)
                .find(|row| row.id == item.id && item.id > 0)
                .map_or(crate::SourceCents::Unavailable, |row| row.fixed_grant_price.clone())
        } else {
            crate::SourceCents::Unavailable
        };
        records.push(crate::EsppAcquisition {
            id: item.id,
            assigned: crate::DocumentRecipient::Unknown,
            owner_source: None,
            joint_share: None,
            owner: item.owner.owned(),
            form: form.clone(),
            fixed_grant_price,
        });
    }
    Ok(records)
}

// Save the printed exercise independently of its downstream share details.
fn form3921_records(
    interview: &Interview,
    items: &[IncomeDocumentItem],
) -> Result<Vec<crate::IsoExerciseSource>, SaveFailure> {
    let mut records = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form3921(form) } =
            &item.document
        else {
            continue;
        };
        if form.granted > form.exercised {
            return Err(ValidationIssue::new("ISO_EXERCISED_BEFORE_GRANTED")
                .at_field("exercised")
                .in_field("document")
                .in_list("documents", index)
                .in_field("documents")
                .into());
        }
        if item.owner == crate::DocumentOwner::Spouses && !interview.joint() {
            return Err(crate::interview::status(
                "MESSAGE_FORM_3921_YOUR_SPOUSE_S_STOCK_OPTIONS_GO_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_J",
            ).into());
        }
        // The topic answers of an accepted exercise survive a correction of its paper.
        let more = interview
            .draft
            .screening
            .iter()
            .flat_map(|s| &s.iso_exercises)
            .find(|row| row.id == item.id && item.id > 0)
            .and_then(|row| row.more.clone());
        let record = crate::IsoExerciseSource {
            id: item.id,
            assigned: crate::DocumentRecipient::Unknown,
            owner_source: None,
            joint_share: None,
            owner: item.owner.owned(),
            form: form.clone(),
            more,
        };
        if !crate::extended_rules::valid_exercise_sources(core::slice::from_ref(&record)) {
            return Err(crate::interview::status(
                "MESSAGE_FORM_3921_CHECK_THE_SHARE_COUNTS_UP_TO_SIX_DECIMAL_PLACES_AND_THE_NAMES",
            )
            .into());
        }
        records.push(record);
    }
    Ok(records)
}
// Earlier-year installment answers retain their source facts in
// FederalScreening.installment_payments. Validate amounts and original
// property classification before accepting the sale.
fn installment_payment_issue(sale: &crate::InstallmentPayments) -> Option<ValidationIssue> {
    if sale.gross_profit > sale.contract_price {
        Some(ValidationIssue::new("INSTALLMENT_PROFIT_ABOVE_CONTRACT").at_field("gross_profit"))
    } else if i128::from(sale.earlier_payments) + i128::from(sale.payments)
        > i128::from(sale.contract_price)
    {
        Some(ValidationIssue::new("INSTALLMENT_PAID_ABOVE_CONTRACT").at_field("payments"))
    } else {
        sale.livestock_issue().map(|issue| issue.in_field("InstallmentPayments"))
    }
}
fn repeated_document_issue(
    items: &[IncomeDocumentItem],
    key: &'static str,
    selects: impl Fn(&IncomeSource) -> bool,
) -> Option<ValidationIssue> {
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| selects(&item.document))
        .nth(1)
        .map(|(index, _)| submitted_data_answer_issue(ValidationIssue::new(key).at_record(), index))
}
fn submitted_data_answer_issue(issue: ValidationIssue, index: usize) -> ValidationIssue {
    issue.in_field("document").in_list("documents", index).in_field("documents")
}
fn installment_records(
    interview: &Interview,
    items: &[IncomeDocumentItem],
) -> Result<Vec<crate::EarlierInstallment>, crate::validation::SaveFailure> {
    let mut records = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let IncomeSource::Record { value: crate::IncomeRecord::InstallmentPayments(page) } =
            &item.document
        else {
            continue;
        };
        let sale = page.record();
        if let Some(issue) = installment_payment_issue(&sale) {
            return Err(submitted_data_answer_issue(issue, index).into());
        }
        if item.owner == crate::DocumentOwner::Spouses && !interview.joint() {
            return Err(crate::interview::status(
                "MESSAGE_INSTALLMENT_PAYMENTS_YOUR_SPOUSE_S_SALES_GO_ON_THIS_RETURN_ONLY_WHEN_YOU_FIL",
            ).into());
        }
        if !sale.valid() {
            return Err(crate::interview::status(
                "MESSAGE_INSTALLMENT_PAYMENTS_ENTER_THE_PROPERTY_A_SALE_DATE_BEFORE_2026_AND_THE_AMOU",
            ).into());
        }
        records.push(crate::EarlierInstallment { owner: item.owner.owned(), sale: sale.clone() });
    }
    Ok(records)
}
// The 2026 section 1256 loss carryback (Form 6781 line6), preserving the
// existing duplicate-first, amount, then actual ownership predicate order.
fn section1256_carryback_issue(value: &crate::Section1256Carryback) -> Option<ValidationIssue> {
    if !(1..=100_000_000_000).contains(&value.amount) {
        return Some(
            ValidationIssue::new(
                "MESSAGE_SECTION_1256_ENTER_A_CARRYBACK_AMOUNT_WITHIN_THE_SUPPORTED_RANGE",
            )
            .at_field("amount"),
        );
    }
    if value.injured_owners.as_ref().is_some_and(|owners| !owners.valid(value.amount)) {
        // This existing composite native owner does not expose a narrower leaf.
        return Some(
            ValidationIssue::new("INJURED_CARRYBACK_OWNER_TOTAL").at_field("injured_owners"),
        );
    }
    None
}
fn carryback_record(
    items: &[IncomeDocumentItem],
) -> Result<Option<crate::Section1256Carryback>, crate::validation::SaveFailure> {
    let mut found = items.iter().enumerate().filter_map(|(index, item)| match &item.document {
        IncomeSource::Record { value: crate::IncomeRecord::Section1256Carryback(value) } => {
            Some((index, value))
        }
        _ => None,
    });
    let first = found.next();
    if let Some((index, _)) = found.next() {
        return Err(submitted_data_answer_issue(
            ValidationIssue::new("SECTION_1256_CARRYBACK_TWICE").at_record(),
            index,
        )
        .into());
    }
    if let Some((index, value)) = first
        && let Some(issue) = section1256_carryback_issue(value)
    {
        return Err(submitted_data_answer_issue(issue, index).into());
    }
    Ok(first.map(|(_, value)| value.clone()))
}
fn nonfarm_optional_records(
    interview: &Interview,
    items: &[IncomeDocumentItem],
) -> Result<Vec<(Owner, crate::NonfarmSeHistory)>, crate::validation::SaveFailure> {
    let mut records: Vec<(Owner, crate::NonfarmSeHistory)> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let IncomeSource::Record { value: crate::IncomeRecord::NonfarmOptionalMethod(history) } =
            &item.document
        else {
            continue;
        };
        if !interview.allowed_owners().contains(&(item.owner as i32)) {
            return Err(ValidationIssue::new("NONFARM_OPTIONAL_OWNER_INVALID")
                .at_field("owner")
                .in_list("documents", index)
                .in_field("documents")
                .into());
        }
        if records.iter().any(|(owner, _)| *owner == item.owner.owned()) {
            return Err(submitted_data_answer_issue(
                ValidationIssue::new("NONFARM_OPTIONAL_TWICE").at_record(),
                index,
            )
            .into());
        }
        if let Some(issue) = history.issue() {
            return Err(submitted_data_answer_issue(issue, index).into());
        }
        records.push((item.owner.owned(), history.clone()));
    }
    Ok(records)
}
// The earlier-year related-party exchange kind (Form 8824 Part II) onto
// FederalScreening.related_exchanges, with the server twins of the kind's
// client rule. Refusals (Something else applies) are stored.
fn related_exchange_records(
    interview: &Interview,
    items: &[IncomeDocumentItem],
) -> Result<Vec<crate::EarlierRelatedExchange>, SaveFailure> {
    let mut records = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let IncomeSource::Record { value: crate::IncomeRecord::RelatedExchange(exchange) } =
            &item.document
        else {
            continue;
        };
        if let Some(issue) = exchange.issue() {
            return Err(submitted_data_answer_issue(issue, index).into());
        }
        if item.owner == crate::DocumentOwner::Spouses && !interview.joint() {
            return Err(ValidationIssue::new("MESSAGE_RELATED_EXCHANGE_YOUR_SPOUSE_S_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY")
                .at_field("owner").in_list("documents", index).in_field("documents").into());
        }
        if !exchange.valid() {
            return Err(crate::interview::status(
                "MESSAGE_RELATED_EXCHANGE_ENTER_THE_PROPERTY_A_LAST_TRANSFER_IN_2024_OR_2025_AND_THE_DATE_IN_2026",
            ).into());
        }
        records.push(crate::EarlierRelatedExchange {
            owner: item.owner.owned(),
            exchange: exchange.clone(),
        });
    }
    Ok(records)
}
// The Work abroad topic's Form 2555 kind onto FederalScreening.foreign_earned,
// with the server twins of the kind's client rules: one per person
// (FORM_2555_TWICE), a spouse only on a joint return.
fn foreign_earned_records(
    interview: &Interview,
    items: &[IncomeDocumentItem],
) -> Result<Vec<crate::ForeignEarnedClaim>, SaveFailure> {
    let mut records: Vec<crate::ForeignEarnedClaim> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let IncomeSource::Record { value: crate::IncomeRecord::Form2555(v) } = &item.document
        else {
            continue;
        };
        if let Some(key) = v.rule() {
            return Err(libertas_formatted_text(key, &[]).into());
        }
        if !interview.allowed_owners().contains(&(item.owner as i32)) {
            return Err(ValidationIssue::new(
                "MESSAGE_FORM_2555_YOUR_SPOUSE_S_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
            )
            .at_field("owner")
            .in_list("documents", index)
            .in_field("documents")
            .into());
        }
        if records.iter().any(|r| r.owner == item.owner.owned()) {
            return Err(submitted_data_answer_issue(
                ValidationIssue::new("FORM_2555_TWICE").at_record(),
                index,
            )
            .into());
        }
        if !v.housing.valid() {
            return Err(crate::interview::status(
                "MESSAGE_FORM_2555_ENTER_VALID_HOUSING_EXPENSES_ALLOWANCES_HOUSEHOLD_AMOUNTS_AND_2026_STAY_DATES",
            ).into());
        }
        records.push(v.record(item.owner.owned()));
    }
    Ok(records)
}
fn data_answer_records(
    interview: &Interview,
    topic: TaxSection,
    items: &[IncomeDocumentItem],
) -> Result<DataAnswers, crate::validation::SaveFailure> {
    let key = |key: &str| crate::validation::SaveFailure::from(libertas_formatted_text(key, &[]));
    Ok(match topic {
        TaxSection::Wages => {
            let mut records: Vec<crate::ClergyClaim> = Vec::new();
            for (index, item) in items.iter().enumerate() {
                let IncomeSource::Record { value: crate::IncomeRecord::Clergy(record) } =
                    &item.document
                else {
                    continue;
                };
                if !interview.allowed_owners().contains(&(item.owner as i32)) {
                    return Err(key("CLERGY_OWNER_INVALID"));
                }
                if records.iter().any(|r| r.owner == item.owner.owned()) {
                    return Err(ValidationIssue::new("CLERGY_TWICE")
                        .at_field("owner")
                        .in_list("documents", index)
                        .in_field("documents")
                        .into());
                }
                if !record.role.valid() {
                    return Err(key("CLERGY_AMOUNT_INVALID"));
                }
                records.push(crate::ClergyClaim {
                    owner: item.owner.owned(),
                    role: record.role.clone(),
                });
            }
            DataAnswers::Clergy(records)
        }
        TaxSection::InterestDividends => {
            let mut found =
                items.iter().enumerate().filter_map(|(index, item)| match &item.document {
                    IncomeSource::Record {
                        value: crate::IncomeRecord::ForeignAccountOrTrust(value),
                    } => Some((index, value.clone())),
                    _ => None,
                });
            let first = found.next();
            if let Some((index, _)) = found.next() {
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new("FOREIGN_ACCOUNT_TWICE").at_record(),
                    index,
                )
                .into());
            }
            // The client rule on the kind, checked again.
            if first
                .as_ref()
                .is_some_and(|(_, v)| (v.account || v.trust) && v.highest_value.is_none())
            {
                return Err(key("FOREIGN_ASSETS_VALUE_NEEDED"));
            }
            let mut elections =
                items.iter().enumerate().filter_map(|(index, item)| match item.document {
                    IncomeSource::Record {
                        value: crate::IncomeRecord::TaxableBondPremium(value),
                    } => Some((index, value)),
                    _ => None,
                });
            let premium = elections.next();
            if let Some((index, _)) = elections.next() {
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new("TAXABLE_PREMIUM_ELECTION_TWICE").at_record(),
                    index,
                )
                .into());
            }
            DataAnswers::Foreign {
                account: first.map(|(_, value)| value),
                premium: premium.map(|(_, value)| value),
            }
        }
        TaxSection::InvestmentSales => {
            let returns: Vec<_> = items
                .iter()
                .filter_map(|item| match &item.document {
                    IncomeSource::Record {
                        value: crate::IncomeRecord::CapitalLossCarryover(value),
                    } => Some(value.clone().into()),
                    _ => None,
                })
                .collect();
            // Tax structure: one prior return per taxpayer/spouse; a joint return represents both.
            if returns.len() > 2 {
                return Err(key("CAPITAL_LOSS_CARRYOVER_LIMIT"));
            }
            DataAnswers::InvestmentSales {
                capital: returns,
                installments: installment_records(interview, items)?,
                carryback: carryback_record(items)?,
                related: related_exchange_records(interview, items)?,
            }
        }
        TaxSection::SelfEmployment => {
            // Preserve history refusal priority before QBI and loss collection.
            let nonfarm = nonfarm_optional_records(interview, items)?;
            DataAnswers::SelfEmployment {
                qbi: qbi_loss_answer(items)?,
                losses: loss_answer(items)?,
                nonfarm,
            }
        }
        TaxSection::Documents => {
            // Validate the source forms together without replacing topic-owned carries.
            disaster_records(interview, items, &[])?;
            DataAnswers::None
        }
        TaxSection::Retirement => DataAnswers::Disaster(disaster_records(interview, items, &[])?),
        TaxSection::WorkAbroad => {
            DataAnswers::WorkAbroad(foreign_earned_records(interview, items)?)
        }
        // At most one record of ABLE contributions for each beneficiary
        // (ABLE_CONTRIBUTIONS_TWICE, the client rule on IncomeRecords, checked
        // again); your spouse's only on a joint return.
        TaxSection::OtherIncome => {
            let mut records: Vec<crate::AbleContributions> = Vec::new();
            for (index, item) in items.iter().enumerate() {
                let IncomeSource::Record { value: crate::IncomeRecord::AbleContributions(page) } =
                    &item.document
                else {
                    continue;
                };
                if !interview.allowed_owners().contains(&(item.owner as i32)) {
                    return Err(ValidationIssue::new("MESSAGE_ABLE_CONTRIBUTIONS_YOUR_SPOUSE_S_GO_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY")
                .at_field("owner").in_list("documents", index).in_field("documents").into());
                }
                if records.iter().any(|r| r.owner == item.owner.owned()) {
                    return Err(ValidationIssue::new("ABLE_CONTRIBUTIONS_TWICE")
                        .at_field("owner")
                        .in_list("documents", index)
                        .in_field("documents")
                        .into());
                }
                records.push(page.record(item.owner.owned()));
            }
            if records.iter().any(|r| !r.valid()) {
                return Err(key("ABLE_CONTRIBUTIONS_INVALID"));
            }
            DataAnswers::Able(records)
        }
        // The two return-level farm kinds, at most one of each (the client
        // rules FARM_AVERAGING_TWICE and FARM_LOSS_TWICE, checked again).
        TaxSection::Farming => {
            let averaging: Vec<_> = items
                .iter()
                .filter_map(|item| match &item.document {
                    IncomeSource::Record { value: crate::IncomeRecord::FarmAveraging(value) } => {
                        Some(crate::farm_page::averaging_record(value))
                    }
                    _ => None,
                })
                .collect();
            if let Some(issue) =
                repeated_document_issue(items, "FARM_AVERAGING_TWICE", |document| {
                    matches!(
                        document,
                        IncomeSource::Record { value: crate::IncomeRecord::FarmAveraging(_) }
                    )
                })
            {
                return Err(issue.into());
            }
            let nol: Vec<_> = items
                .iter()
                .filter_map(|item| match &item.document {
                    IncomeSource::Record { value: crate::IncomeRecord::FarmLoss(value) } => {
                        Some(crate::farm_page::farm_loss_record(value))
                    }
                    _ => None,
                })
                .collect();
            if let Some(issue) = repeated_document_issue(items, "FARM_LOSS_TWICE", |document| {
                matches!(document, IncomeSource::Record { value: crate::IncomeRecord::FarmLoss(_) })
            }) {
                return Err(issue.into());
            }
            DataAnswers::Farming {
                qbi: qbi_loss_answer(items)?,
                losses: loss_answer(items)?,
                averaging: averaging.into_iter().next(),
                nol: nol.into_iter().next(),
            }
        }
        _ => DataAnswers::None,
    })
}
// The answer of a row that is no income item (a Form 8606, a QCD gift, an IRA
// history or a return-wide answer): on a community-property return the account
// owner's own, whatever the page's allowed set asks of the income rows.
fn fixed_community(d: &FederalDraft) -> CommunityShare {
    if d.setup.as_ref().is_some_and(crate::community::applies) {
        CommunityShare::Separate
    } else {
        CommunityShare::NotNeeded
    }
}
// The reverse of data_answer_records: the rows a topic page reopens with, or
// None for a stored answer this page could not have written.
fn data_answer_items(d: &FederalDraft, topic: TaxSection) -> Option<Vec<IncomeDocumentItem>> {
    let Some(screening) = d.screening.as_ref() else {
        return Some(Vec::new());
    };
    let item = |owner: Owner, document: IncomeSource| IncomeDocumentItem {
        refund_management: None,
        refund_management_needed: None,
        allowed_community: Vec::new(),
        investment_caps: match &document {
            IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(v) } => {
                Some(crate::extended_rules::capital_caps(&v.clone().into()))
            }
            _ => None,
        },
        id: 0,
        owner: crate::DocumentOwner::from(owner),
        document,
        preview: None,
        community: fixed_community(d),
    };
    Some(match topic {
        TaxSection::Wages => screening
            .clergy
            .iter()
            .map(|c| {
                item(
                    c.owner,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Clergy(crate::ClergyRecord {
                            role: c.role.clone(),
                        }),
                    },
                )
            })
            .collect(),
        TaxSection::InterestDividends => {
            let (account, trust) = (screening.foreign_account, screening.foreign_trust);
            let mut items = if account || trust {
                alloc::vec![item(
                    Owner::Taxpayer,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::ForeignAccountOrTrust(ForeignAccountOrTrust {
                            account,
                            fbar_required: screening.fbar_required,
                            fbar_countries: screening.fbar_countries.clone(),
                            trust,
                            highest_value: screening.foreign_asset_max,
                        })
                    },
                )]
            } else if screening.fbar_required.is_some()
                || !screening.fbar_countries.is_empty()
                || screening.foreign_asset_max.is_some()
            {
                return None;
            } else {
                Vec::new()
            };
            if let Some(answer) = screening.taxable_premium_election {
                items.push(item(
                    Owner::Taxpayer,
                    IncomeSource::Record { value: crate::IncomeRecord::TaxableBondPremium(answer) },
                ));
            }
            items
        }
        TaxSection::InvestmentSales => screening
            .capital_history
            .iter()
            .map(|value| {
                Some(item(
                    Owner::Taxpayer,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::CapitalLossCarryover(value.clone().into()),
                    },
                ))
            })
            .chain(screening.espp_acquisitions.iter().map(|value| {
                let mut row = item(
                    value.owner,
                    IncomeSource::InvestmentSale {
                        value: crate::InvestmentSaleDocument::Form3922(value.form.clone()),
                    },
                );
                row.id = value.id;
                Some(row)
            }))
            .chain(screening.iso_exercises.iter().map(|value| {
                let mut row = item(
                    value.owner,
                    IncomeSource::InvestmentSale {
                        value: crate::InvestmentSaleDocument::Form3921(value.form.clone()),
                    },
                );
                row.id = value.id;
                Some(row)
            }))
            .chain(screening.installment_payments.iter().map(|value| {
                Some(item(
                    value.owner,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::InstallmentPayments(value.sale.page()),
                    },
                ))
            }))
            .chain(screening.section_1256_carryback.map(|amount| {
                Some(item(
                    Owner::Taxpayer,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Section1256Carryback(
                            crate::Section1256Carryback {
                                amount,
                                injured_owners: screening.section_1256_carryback_owners.clone(),
                            },
                        ),
                    },
                ))
            }))
            .chain(screening.related_exchanges.iter().map(|value| {
                Some(item(
                    value.owner,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::RelatedExchange(value.exchange.clone()),
                    },
                ))
            }))
            .collect::<Option<Vec<_>>>()?,
        TaxSection::SelfEmployment | TaxSection::Farming => {
            // Zero carryforwards are no carryforward: the kind reopens only
            // with an amount.
            let history = &screening.qbi_history;
            let mut items = if history.any() {
                alloc::vec![item(
                    Owner::Taxpayer,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::QbiLossCarryforward(QbiLossCarryforward {
                            business: history.business,
                            reit_ptp: history.reit_ptp,
                            injured_owners: history.injured_owners.clone(),
                        })
                    },
                )]
            } else {
                Vec::new()
            };
            if let Some(losses) =
                crate::LossCarryforward::from_histories(&screening.nol, &screening.atnol)
            {
                items.push(item(
                    Owner::Taxpayer,
                    IncomeSource::Record { value: crate::IncomeRecord::NetOperatingLoss(losses?) },
                ));
            }
            if topic == TaxSection::SelfEmployment {
                items.extend(screening.optional_se.iter().filter_map(|e| {
                    e.nonfarm.as_ref().map(|h| {
                        item(
                            e.owner,
                            IncomeSource::Record {
                                value: crate::IncomeRecord::NonfarmOptionalMethod(h.clone()),
                            },
                        )
                    })
                }));
            }
            if topic == TaxSection::Farming {
                if let Some(value) = &screening.farm_averaging {
                    items.push(item(
                        Owner::Taxpayer,
                        IncomeSource::Record {
                            value: crate::IncomeRecord::FarmAveraging(
                                crate::farm_page::averaging_item(value)?,
                            ),
                        },
                    ));
                }
                if let Some(value) = &screening.farm_nol {
                    items.push(item(
                        Owner::Taxpayer,
                        IncomeSource::Record {
                            value: crate::IncomeRecord::FarmLoss(crate::farm_page::farm_loss_item(
                                value,
                            )?),
                        },
                    ));
                }
            }
            items
        }
        TaxSection::Retirement => screening
            .disaster_carries
            .iter()
            .map(|c| {
                item(
                    c.owner,
                    IncomeSource::Record { value: crate::IncomeRecord::Form8915(c.form.clone()) },
                )
            })
            .collect(),
        TaxSection::WorkAbroad => screening
            .foreign_earned
            .iter()
            .map(|c| {
                item(
                    c.owner,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Form2555(crate::Form2555::from_record(c)),
                    },
                )
            })
            .collect(),
        TaxSection::OtherIncome => screening
            .able_contributions
            .iter()
            .map(|c| {
                item(
                    c.owner,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::AbleContributions(c.page()),
                    },
                )
            })
            .collect(),
        _ => Vec::new(),
    })
}

// Supporting facts have no income-property classification. Their one fixed
// answer remains available and hidden by the existing single-choice template.
fn document_community_answers(draft: &FederalDraft, document: &IncomeSource) -> Vec<i32> {
    if matches!(document, adjustment_kinds!() | data_answer_kinds!()) {
        alloc::vec![crate::community::index(fixed_community(draft))]
    } else {
        crate::injured_spouse::community_answers(draft)
    }
}
fn document_community_issue(
    draft: &FederalDraft,
    item: &IncomeDocumentItem,
) -> Option<ValidationIssue> {
    if matches!(item.document, adjustment_kinds!() | data_answer_kinds!()) {
        (item.community != fixed_community(draft))
            .then(|| ValidationIssue::new("COMMUNITY_ANSWER_NOT_ALLOWED").at_record())
    } else {
        crate::injured_spouse::community_row_issue(draft, item.community)
    }
    .map(|issue| issue.in_field("community"))
}

// Supporting history and adjustment answers never become income entries.
fn document_refund_management_needed(
    draft: &FederalDraft,
    result: &FederalResult,
    item: &IncomeDocumentItem,
) -> Option<bool> {
    if !crate::injured_spouse::texas(draft)
        || matches!(item.document, adjustment_kinds!() | data_answer_kinds!())
    {
        return Some(false);
    }
    let entry = draft.income.iter().find(|e| item.id > 0 && e.id == item.id)?;
    crate::injured_spouse::management_needed(draft, result, entry)
}

fn income_refund_management(
    draft: &FederalDraft,
    result: &FederalResult,
    origins: &[(i64, usize, bool)],
    rows: usize,
) -> Vec<crate::IncomeRefundManagementDerived> {
    (0..rows)
        .map(|index| {
            let mut known = Some(false);
            for (id, _, _) in origins.iter().filter(|(_, submitted, _)| *submitted == index) {
                let current = draft
                    .income
                    .iter()
                    .find(|e| e.id == *id)
                    .and_then(|e| crate::injured_spouse::management_needed(draft, result, e));
                known = match (known, current) {
                    (Some(true), _) | (_, Some(true)) => Some(true),
                    (Some(false), Some(false)) => Some(false),
                    _ => None,
                };
            }
            crate::IncomeRefundManagementDerived { refund_management_needed: known }
        })
        .collect()
}

// A statement with payments after death requires that financial owner's death.
fn recorded_income_owner_issue(
    draft: &FederalDraft,
    record: &FederalIncomeEntry,
) -> Option<ValidationIssue> {
    let issue = crate::after_death::recorded_issue(&record.income)?;
    draft
        .people
        .as_ref()
        .and_then(|people| people.person(record.owner.owned()))
        .is_none_or(|person| person.died.is_none())
        .then_some(issue)
}

// One page row onto its stored record.
fn document_entry(item: IncomeDocumentItem, accepted: &[FederalIncomeEntry]) -> FederalIncomeEntry {
    let mut printed = item.document.printed_names();
    // Only a trusted accepted record can retain the association; the paper
    // itself carries printed parties and no matcher result.
    if let Some(names) = &mut printed {
        names.retain_assignment_from(
            accepted
                .iter()
                .find(|before| item.id != 0 && before.id == item.id)
                .and_then(FederalIncomeEntry::printed_names),
        );
    }
    let state_source = crate::IncomeStateSource::from_document(&item.document);
    let mut income = document_income(&item.document);
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut income
        && let Some(FederalIncome::QualifiedSmallBusinessSale { data: before }) = accepted
            .iter()
            .find(|entry| item.id != 0 && entry.id == item.id)
            .map(|entry| &entry.income)
    {
        data.keep_answers(before);
    }
    if let FederalIncome::Dividend { data } = &mut income
        && let Some(FederalIncome::ReturnOfCapital { kind, amount, property, basis, term, special }) =
            accepted
                .iter()
                .find(|entry| item.id != 0 && entry.id == item.id)
                .map(|entry| &entry.income)
    {
        data.capital_returns = alloc::vec![crate::CapitalReturn1099Div {
            kind: *kind,
            amount: *amount,
            property: property.clone(),
            basis: *basis,
            term: *term,
            special: *special,
        }];
    }
    if let FederalIncome::Partnership { data } = &mut income
        && let Some(source) = data.source.as_deref_mut()
    {
        accepted
            .iter()
            .find(|before| item.id != 0 && before.id == item.id)
            .and_then(|before| match &before.income {
                FederalIncome::Partnership { data } => data.source.as_deref(),
                _ => None,
            })
            .filter(|before| before.paper.partner.name == source.paper.partner.name)
            .map(|before| before.printed_names().assignment())
            .unwrap_or_default()
            .apply_to(&mut source.assigned, &mut source.owner_source, &mut source.joint_share);
        printed = Some(source.printed_names());
    }
    if let FederalIncome::Scorp { data } = &mut income {
        if let Some(FederalIncome::Scorp { data: kept }) = accepted
            .iter()
            .find(|before| item.id != 0 && before.id == item.id)
            .map(|before| &before.income)
        {
            data.keep_answers(kept);
        }
        printed = Some(data.printed_names());
    }

    if let FederalIncome::EstateTrustPortfolio { data } = &mut income
        && let Some(FederalIncome::EstateTrustPortfolio { data: old }) = accepted
            .iter()
            .find(|before| item.id != 0 && before.id == item.id)
            .map(|before| &before.income)
    {
        data.keep_answers(old);
    }

    if let FederalIncome::Cooperative { data } = &mut income
        && let Some(FederalIncome::Cooperative { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    if let FederalIncome::Nec1099 { data } = &mut income
        && let Some(FederalIncome::Nec1099 { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // A 1099-C's and a 1099-A's topic answers stay with their accepted record; the paper carries none.
    if let FederalIncome::PersonalDebt { data } = &mut income
        && let Some(FederalIncome::PersonalDebt { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    if let FederalIncome::PropertyAcquisition { data } = &mut income
        && let Some(FederalIncome::PropertyAcquisition { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    if let FederalIncome::K1099 { data } = &mut income
        && let Some(FederalIncome::K1099 { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.parts = kept.parts.clone();
    }
    if let FederalIncome::Misc1099 { data } = &mut income
        && let Some(FederalIncome::Misc1099 { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.component_ids = kept.component_ids.clone();
        data.other_use = kept.other_use.clone();
        data.client_funds = kept.client_funds;
    }
    if let FederalIncome::Broker { data } = &mut income {
        let accepted = accepted
            .iter()
            .find(|before| item.id != 0 && before.id == item.id)
            .and_then(|before| match &before.income {
                FederalIncome::Broker { data } => Some(data),
                _ => None,
            });
        if let Some(source) = crate::broker_page::broker_document(&item.document, accepted) {
            *data = source;
        }
        printed = Some(data.printed_names());
    }
    if let FederalIncome::OpportunityFund { data } = &mut income
        && let Some(FederalIncome::OpportunityFund { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.event = kept.event.clone();
        data.special = kept.special;
    }
    if let FederalIncome::BondRedemption { data } = &mut income
        && let Some(FederalIncome::BondRedemption { data: kept }) = accepted
            .iter()
            .find(|entry| item.id != 0 && entry.id == item.id)
            .map(|entry| &entry.income)
    {
        data.form_8815 = kept.form_8815;
    }
    if let FederalIncome::SavingsBonds { data } = &mut income
        && let Some(FederalIncome::SavingsBonds { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // A 1099-G's topic answers stay with its accepted record; the paper carries none.
    if let FederalIncome::GovernmentPayment { data } = &mut income
        && let Some(FederalIncome::GovernmentPayment { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // A W-2G's topic answer stays with its accepted record; the paper carries none.
    if let FederalIncome::Gambling { data: crate::GamblingSource::W2g(data) } = &mut income
        && let Some(FederalIncome::Gambling { data: crate::GamblingSource::W2g(kept) }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // Medical distribution topic answers stay with their accepted record.
    if let FederalIncome::MedicalDistribution { data } = &mut income
        && let Some(FederalIncome::MedicalDistribution { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // A 1099-Q's topic answers stay with its accepted record; the paper carries none.
    if let FederalIncome::EducationDistribution { data } = &mut income
        && let Some(FederalIncome::EducationDistribution { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // A 1099-QA's treatment stays with its accepted record; the paper carries none.
    if let FederalIncome::Able { data } = &mut income
        && let Some(FederalIncome::Able { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // A Form 2439's topic answers stay with its accepted record too.
    if let FederalIncome::UndistributedGain { data } = &mut income
        && let Some(FederalIncome::UndistributedGain { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // A 1099-R's topic answers stay with its accepted record; the paper carries none.
    if let IncomeSource::Retirement { value: crate::RetirementDocument::R1099(form) } =
        &item.document
        && let Some(kept) = accepted
            .iter()
            .find(|e| item.id != 0 && e.id == item.id)
            .and_then(|e| e.income.r1099_source())
    {
        let mut source = R1099Source::printed(crate::R1099::from(form));
        source.keep_answers(kept);
        income = source.income();
    }
    // An RRB-1099-R's topic answers stay with its accepted record; the paper carries none.
    if let FederalIncome::Retirement { data: crate::RetirementSource::Rrb1099R(data) } = &mut income
        && let Some(FederalIncome::Retirement { data: crate::RetirementSource::Rrb1099R(kept) }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // A 1099-LTC's topic answers stay with its accepted record too.
    if let FederalIncome::LtcBenefits { data } = &mut income
        && let Some(FederalIncome::LtcBenefits { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    if let FederalIncome::Sale { data } = &mut income
        && let Some(FederalIncome::Sale { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    if let FederalIncome::Section1256 { data } = &mut income
        && let Some(FederalIncome::Section1256 { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // A 1099-INT's and 1099-OID's topic answers stay with their accepted record too.
    if let FederalIncome::Interest { data: crate::InterestSource::Int1099(data) } = &mut income
        && let Some(FederalIncome::Interest { data: crate::InterestSource::Int1099(kept) }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    if let FederalIncome::Oid { data } = &mut income
        && let Some(FederalIncome::Oid { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // A 1099-DIV's topic answers stay with its accepted record too.
    if let FederalIncome::Dividend { data } = &mut income
        && let Some(FederalIncome::Dividend { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    if let FederalIncome::Wage { data } = &mut income
        && let Some(FederalIncome::Wage { data: kept }) = accepted
            .iter()
            .find(|entry| item.id != 0 && entry.id == item.id)
            .map(|entry| &entry.income)
    {
        data.keep_answers(kept);
    }
    // A W-2's box 14 meanings are filled once, here, and a recorded one stays.
    if let FederalIncome::Wage { data: crate::WageSource::W2(source) } = &mut income {
        source.classify_box14();
    }
    if let FederalIncome::BondRedemption { data } = &mut income
        && let Some(FederalIncome::BondRedemption { data: kept }) = accepted
            .iter()
            .find(|entry| item.id != 0 && entry.id == item.id)
            .map(|entry| &entry.income)
    {
        data.form_8815 = kept.form_8815;
    }
    if let FederalIncome::SavingsBonds { data } = &mut income
        && let Some(FederalIncome::SavingsBonds { data: kept }) = accepted
            .iter()
            .find(|entry| item.id != 0 && entry.id == item.id)
            .map(|entry| &entry.income)
    {
        data.keep_answers(kept);
    }
    // A printed home sale keeps its topic answers when its paper is corrected.
    if let FederalIncome::HomeSale { data } = &mut income
        && data.form.paper().is_some()
        && let Some(FederalIncome::HomeSale { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.keep_answers(kept);
    }
    // A business keeps the owner health insurance and retirement plan the
    // adjustments page stored on the accepted record.
    if let FederalIncome::Business { data } = &mut income
        && let Some(FederalIncome::Business { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.health_insurance = kept.health_insurance.clone();
        data.retirement = kept.retirement.clone();
        if let IncomeSource::Record { value: crate::IncomeRecord::Business(page) } = &item.document
        {
            data.intangibles = page
                .more
                .intangibles
                .iter()
                .map(|row| {
                    row.record(kept.intangibles.iter().find(|old| old.id == row.id && row.id > 0))
                })
                .collect();
        }
    }
    // Crop-insurance source facts belong to their dedicated accepted-fact actions.
    if let Some(kept) = accepted.iter().find(|e| item.id != 0 && e.id == item.id) {
        let mut projected = kept.clone();
        projected.income = income;
        let mut old_entry = kept.clone();
        if let (Some(target), Some(old)) = (
            crate::farm_insurance_paper::farm_mut(&mut projected),
            crate::farm_insurance_paper::farm_mut(&mut old_entry),
        ) {
            target.insurance_papers = old.insurance_papers.clone();
        }
        income = projected.income;
    }
    // A partnership K-1 keeps the partner's coverage, distribution-paid
    // premiums and plan choice of the self-employed benefits page; boxes 13
    // code M and R are the K-1 page's own (T2-43p).
    if let FederalIncome::Partnership { data } = &mut income
        && let Some(FederalIncome::Partnership { data: kept }) =
            accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income)
    {
        data.health_insurance = kept.health_insurance.clone();
        data.health_distributions = kept.health_distributions;
        data.retirement = kept.retirement.clone();
    }
    crate::k1_activity::preserve_person_sources(
        &mut income,
        accepted.iter().find(|e| item.id != 0 && e.id == item.id).map(|e| &e.income),
    );
    // The name the person typed; the page does not ask a W-2's, so a W-2 row
    // that keeps an accepted identity keeps its accepted name. A source with no
    // name stays unnamed: the document kind and identity name it where shown.
    let label = match item.document {
        IncomeSource::Wage { value: crate::WageDocument::W2(_) } => accepted
            .iter()
            .find(|e| item.id != 0 && e.id == item.id)
            .map(|e| e.label.clone())
            .unwrap_or_default(),
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(Form1099IntPaper { payer, .. }),
        }
        | IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(crate::Form1099DivPaper { payer, .. }),
        }
        | IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Oid1099(Form1099OidPaper { payer, .. }),
        } => payer.name,
        IncomeSource::Record { value: crate::IncomeRecord::CashTipsWithoutW2(v) } => v.employer,
        IncomeSource::Record { value: crate::IncomeRecord::PrivateLoanInterest(v) } => v.borrower,
        IncomeSource::Record { value: crate::IncomeRecord::InstallmentSale(v) } => v.property,
        IncomeSource::Record { value: crate::IncomeRecord::SellerMortgageInterest(v) } => {
            v.buyer.buyer
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099B(value) } => {
            value.box_1a_description
        }
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Form1099DA(value),
        } => value.box_1b_asset_name,
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Sale1099B(Sale1099B { description, .. }),
        } => description,
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::OpportunityFund(ref v),
        } => v.fund.clone(),
        IncomeSource::Record { value: crate::IncomeRecord::OpportunityFundDeferral(ref v) } => {
            v.fund.clone()
        }
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Contracts1099B(Contracts1099B { payer, .. }),
        } => payer.name,
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::IsoShareSale(ref v),
        } => v.label(),
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EmployeeShareSale(ref v),
        } => v.label(),
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EsppShareSale(ref v),
        } => v.label(),
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::NuaShareSale(ref v),
        } => v.label(),
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::QsbsSale(ref v) } => {
            v.issuer.clone()
        }
        IncomeSource::Record { value: crate::IncomeRecord::SmallBusinessStockLoss(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::WorthlessSecurity(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::BadDebt(_) } => Default::default(),
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Form2439(ref v),
        } => v.label(),
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099S(ref v) } => {
            crate::Form1099S::from(v).label()
        }
        IncomeSource::Record { value: crate::IncomeRecord::MainHomeSale { ref sale, .. } } => {
            sale.label()
        }
        IncomeSource::Record { value: crate::IncomeRecord::Farm(ref f) } => {
            crate::farm_page::farm_label(f)
        }
        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(ref f) } => {
            crate::farm_page::farm_rental_label(f)
        }
        IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(ref v) } => match &v.kind {
            crate::OtherIncomeKind::Other { description } => description.clone(),
            _ => alloc::string::String::new(),
        },
        IncomeSource::Record { value: crate::IncomeRecord::Business(Business { name, .. }) } => {
            name
        }
        IncomeSource::Record { value: crate::IncomeRecord::Rental(Rental { address, .. }) } => {
            address
        }
        IncomeSource::K1 {
            value:
                crate::K1Document::K1EstateTrust(crate::K1EstateTrustDocument {
                    form: crate::K1EstateTrustPaper { estate_trust, .. },
                    ..
                }),
        } => estate_trust.name,
        IncomeSource::K1 { value: crate::K1Document::K1PartnershipActivities(v) } => {
            v.partnership.name
        }
        IncomeSource::K1 { value: crate::K1Document::K1ScorpActivities(v) } => v.corporation.name,
        IncomeSource::K1 {
            value:
                crate::K1Document::K1Scorp(crate::K1ScorpDocument {
                    form: crate::K1ScorpPaper { corporation, .. },
                    ..
                }),
        } => corporation.name,
        IncomeSource::K1 {
            value:
                crate::K1Document::K1Partnership(crate::K1PartnershipDocument {
                    form: crate::K1PartnershipPaper { partnership, .. },
                    ..
                }),
        } => partnership.name,
        IncomeSource::Record { value: crate::IncomeRecord::InheritedIra(ref v) } => {
            v.decedent.clone()
        }
        IncomeSource::Record { value: crate::IncomeRecord::ForeignPay(ref v) } => {
            v.employer.clone()
        }
        // The name the person typed for an insured who is not on the return.
        IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(ref v) } => {
            match &v.insured {
                crate::LtcInsuredPerson::Other { name } => name.clone(),
                crate::LtcInsuredPerson::OnReturn { .. } => alloc::string::String::new(),
            }
        }
        IncomeSource::CanceledDebt {
            value: crate::CanceledDebtDocument::Cancel1099C(crate::Form1099C { creditor, .. }),
        } => creditor.name,
        IncomeSource::CanceledDebt {
            value: crate::CanceledDebtDocument::Acquisition1099A(crate::Form1099A { lender, .. }),
        } => lender.name,
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Form8818(_) } => {
            alloc::string::String::new()
        }
        // A child's election is named by its child on the people page.
        IncomeSource::Record { value: crate::IncomeRecord::Form8814(_) }
        | IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::SavingsBonds(_),
        }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Sa1099(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::HsaTrusteeStatement(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Qa1099(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::EducationAccountExpenses(_) }
        | IncomeSource::Retirement { value: crate::RetirementDocument::R1099(_) }
        | IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099R(_) }
        | IncomeSource::Retirement { value: crate::RetirementDocument::Ssa1099(_) }
        | IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099(_) }
        | IncomeSource::GovernmentPayment { .. }
        | IncomeSource::Farming(_)
        | IncomeSource::Record { value: crate::IncomeRecord::AlimonyReceived(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::Royalties(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(_) }
        | IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099Nec(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(_) }
        | IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099K(_) }
        | IncomeSource::Gambling(_)
        | IncomeSource::Record { value: crate::IncomeRecord::GamblingLosses(_) }
        | IncomeSource::Wage { value: crate::WageDocument::Form8919(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::HealthLoanRepayment(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::Scholarship(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::JuryPay(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::HobbySales(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::WildfireRelief(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::DisasterRelief(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::CasualtyLivingInsurance(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::DisasterMitigation(_) } => {
            alloc::string::String::new()
        }
        adjustment_kinds!() | data_answer_kinds!() => {
            unreachable!("Form 8606, QCD gifts and the data answers are not income entries")
        }
    };
    FederalIncomeEntry {
        printed,
        id: item.id,
        label,
        owner: item.owner,
        income,
        state_source,
        community: item.community,
        refund_management: item.refund_management,
    }
}

// One stored record onto its page row; None for a record the page cannot show.
// `farm_se` lists the owners whose farm optional method is stored, for the
// farm kind (T2-45p).
pub(crate) fn document_item(
    entry: &FederalIncomeEntry,
    farm_se: &[Owner],
    separate: bool,
) -> Option<IncomeDocumentItem> {
    let payer = || entry.label.clone();
    let mut document =
        match &entry.income {
            FederalIncome::Wage { data } => match data {
                crate::WageSource::W2(source) => {
                    IncomeSource::Wage { value: crate::WageDocument::W2(source.form.clone()) }
                }
                crate::WageSource::CashTipsWithoutW2(value) => IncomeSource::Record {
                    value: crate::IncomeRecord::CashTipsWithoutW2(value.clone()),
                },
            },
            FederalIncome::Interest { data: crate::InterestSource::Int1099(source) } => {
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(source.form.clone()),
                }
            }
            FederalIncome::Interest { data: crate::InterestSource::Record(data) }
                if data.private_loan =>
            {
                IncomeSource::Record {
                    value: crate::IncomeRecord::PrivateLoanInterest(crate::PrivateLoanInterest {
                        borrower: payer(),
                        interest: data.taxable,
                        seller_mortgage: data.seller_mortgage.as_ref().map(|v| {
                            crate::PrivateLoanResidence {
                                buyer_tin: v.buyer_tin.clone(),
                                buyer_address: v.buyer_address.clone(),
                            }
                        }),
                    }),
                }
            }
            FederalIncome::Interest { data: crate::InterestSource::Record(data) }
                if data.seller_mortgage.is_some() =>
            {
                IncomeSource::Record {
                    value: crate::IncomeRecord::SellerMortgageInterest(
                        crate::SellerMortgageInterest {
                            buyer: data.seller_mortgage.clone().unwrap(),
                            interest: data.taxable,
                        },
                    ),
                }
            }
            FederalIncome::Interest { data: crate::InterestSource::Record(data) } => {
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(
                        interest_page(
                            data,
                            &crate::federal_model::ChildStatementKept {
                                payer: crate::PrintedParty::from_name(payer()),
                                recipient: Default::default(),
                                state_details: Default::default(),
                            },
                        )
                        .form,
                    ),
                }
            }
            FederalIncome::Dividend { data } => IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(data.form.clone()),
            },
            FederalIncome::Oid { data } => IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Oid1099(data.form.clone()),
            },
            FederalIncome::ChildElection { data } => IncomeSource::Record {
                value: crate::IncomeRecord::Form8814(crate::Form8814::from_record(data)),
            },
            FederalIncome::InstallmentSale { data } => {
                IncomeSource::Record { value: crate::IncomeRecord::InstallmentSale(data.clone()) }
            }
            FederalIncome::Broker { data } => data.income_source(),
            FederalIncome::Sale { data } => IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Sale1099B(data.form.clone()),
            },
            FederalIncome::OpportunityFund { data } => IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::OpportunityFund(
                    crate::QofInvestmentPaper::from(data),
                ),
            },
            FederalIncome::OpportunityFundDeferral { data } => IncomeSource::Record {
                value: crate::IncomeRecord::OpportunityFundDeferral(data.clone()),
            },
            FederalIncome::Section1256 { data } => IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Contracts1099B(data.form.clone()),
            },
            FederalIncome::IsoSale { data } => IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::IsoShareSale(
                    crate::IsoShareSale::from_record(&entry.label, data),
                ),
            },
            FederalIncome::IsoSaleSource { data } => IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::IsoShareSale(data.clone()),
            },
            FederalIncome::NuaSaleSource { data } => IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::NuaShareSale(data.clone()),
            },
            FederalIncome::EmployeeStockSale { data } => IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(
                    crate::EmployeeShareSale::from_record(&entry.label, data),
                ),
            },
            FederalIncome::EsppSaleSource { data } => IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EsppShareSale(data.clone()),
            },
            FederalIncome::EsppSale { data } => IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EsppShareSale(
                    crate::EsppShareSale::from_record(&entry.label, data)?,
                ),
            },
            FederalIncome::NuaSale { data } => IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::NuaShareSale(
                    crate::NuaShareSale::from_record(&entry.label, data),
                ),
            },
            FederalIncome::QualifiedSmallBusinessSale { data } => IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::QsbsSale(crate::QsbsSale::from_record(data)),
            },
            FederalIncome::SmallBusinessStock { data } => IncomeSource::Record {
                value: crate::IncomeRecord::SmallBusinessStockLoss(data.clone()),
            },
            FederalIncome::WorthlessSecurity { data } => {
                IncomeSource::Record { value: crate::IncomeRecord::WorthlessSecurity(data.clone()) }
            }
            FederalIncome::NonbusinessBadDebt { data } => {
                IncomeSource::Record { value: crate::IncomeRecord::BadDebt(data.clone()) }
            }
            FederalIncome::ReturnOfCapital { kind, amount, property, .. } => {
                let mut paper = crate::Form1099DivPaper {
                    payer: crate::PrintedParty::from_name(payer()),
                    ..Default::default()
                };
                if *kind == crate::CapitalDistributionKind::Nondividend {
                    paper.nondividend_distributions = Some(*amount);
                } else {
                    paper.cash_liquidation = Some(*amount);
                    paper.noncash_liquidation = Some(crate::kernel::ratio::checked_sum(
                        property.iter().map(|value| value.value),
                    )?);
                }
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(paper),
                }
            }
            FederalIncome::UndistributedGain { data } => IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Form2439(data.form.clone()),
            },
            FederalIncome::HomeSale { data } => match &data.form {
                crate::HomeSaleSource::Paper(form) => IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Form1099S(form.clone()),
                },
                crate::HomeSaleSource::Records(sale) => IncomeSource::Record {
                    value: crate::IncomeRecord::MainHomeSale {
                        sale: sale.clone(),
                        answers: data.answers()?,
                    },
                },
            },
            FederalIncome::SavingsBonds { data } => IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::SavingsBonds(data.entry()),
            },
            FederalIncome::BondRedemption { data } => IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Form8818(data.form.clone()),
            },
            FederalIncome::EducationDistribution { data } => IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Q1099(data.form.clone()),
            },
            FederalIncome::EducationAccountExpenses { data } => IncomeSource::Record {
                value: crate::IncomeRecord::EducationAccountExpenses(education_expenses_item(data)),
            },
            FederalIncome::Retirement { data: crate::RetirementSource::Rrb1099R(data) } => {
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::Rrb1099R(crate::FormRrb1099RPaper::from(
                        &data.form,
                    )),
                }
            }
            FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) }
            | FederalIncome::Roth { data }
            | FederalIncome::TrumpDeath { data }
            | FederalIncome::TrumpAbleRollover { data } => IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(crate::Form1099RPaper::from(&data.form)),
            },
            FederalIncome::InheritedIra { data } => {
                IncomeSource::Record { value: crate::IncomeRecord::InheritedIra(data.account()) }
            }
            FederalIncome::MedicalDistribution { data } => IncomeSource::OtherIncome {
                value: match &data.form {
                    crate::MedicalSaPaper::Form1099Sa(form) => {
                        crate::OtherIncomeDocument::Sa1099(form.clone())
                    }
                    crate::MedicalSaPaper::TrusteeStatement(statement) => {
                        crate::OtherIncomeDocument::HsaTrusteeStatement(statement.clone())
                    }
                },
            },
            FederalIncome::LtcBenefits { data } => IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Ltc1099(data.form.clone()),
            },
            FederalIncome::LtcInsured { data } => {
                IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(data.page()) }
            }
            FederalIncome::Able { data } => IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Qa1099(data.form.clone()),
            },
            FederalIncome::SocialSecurity { data } => IncomeSource::Retirement {
                value: match data {
                    crate::SocialBenefitStatement::Ssa1099(form) => {
                        crate::RetirementDocument::Ssa1099(form.clone())
                    }
                    crate::SocialBenefitStatement::Rrb1099(form) => {
                        crate::RetirementDocument::Rrb1099(form.clone())
                    }
                },
            },
            FederalIncome::GovernmentPayment { data } => IncomeSource::GovernmentPayment {
                value: data.form.clone(),
                refund_state: data.refund_state,
                refund_locality: data.refund_locality.clone(),
            },
            FederalIncome::Cooperative { data } => IncomeSource::Farming(data.form.clone()),
            FederalIncome::EstateTrustPortfolio { data } => IncomeSource::K1 {
                value: crate::K1Document::K1EstateTrust(crate::K1EstateTrustDocument::from_source(
                    data,
                )),
            },
            FederalIncome::PartnershipActivities { data } => IncomeSource::K1 {
                value: crate::K1Document::K1PartnershipActivities(alloc::boxed::Box::new(
                    crate::K1PartnershipActivitiesPage::from_record(&entry.label, data),
                )),
            },
            FederalIncome::ScorpActivities { data } => IncomeSource::K1 {
                value: crate::K1Document::K1ScorpActivities(
                    crate::K1ScorpActivitiesPage::from_record(&entry.label, data),
                ),
            },
            FederalIncome::Scorp { data } => IncomeSource::K1 {
                value: crate::K1Document::K1Scorp(crate::K1ScorpDocument::from_source(data)),
            },
            FederalIncome::Partnership { data } => IncomeSource::K1 {
                value: crate::K1Document::K1Partnership(crate::K1PartnershipDocument::from_record(
                    &entry.label,
                    data,
                )?),
            },
            FederalIncome::PersonalDebt { data } => IncomeSource::CanceledDebt {
                value: crate::CanceledDebtDocument::Cancel1099C(data.form.clone()),
            },
            FederalIncome::PropertyAcquisition { data } => IncomeSource::CanceledDebt {
                value: crate::CanceledDebtDocument::Acquisition1099A(data.form.clone()),
            },
            FederalIncome::Alimony { data } => {
                IncomeSource::Record { value: crate::IncomeRecord::AlimonyReceived(data.clone()) }
            }
            FederalIncome::ForeignEmployment { data } => IncomeSource::Record {
                value: crate::IncomeRecord::ForeignPay(crate::ForeignPay {
                    employer: entry.label.clone(),
                    wages: data.wages,
                }),
            },
            FederalIncome::EmployeeWages { data } => {
                IncomeSource::Wage { value: crate::WageDocument::Form8919(data.clone()) }
            }
            FederalIncome::HealthLoanAssistance { data } => IncomeSource::Record {
                value: crate::IncomeRecord::HealthLoanRepayment(data.clone()),
            },
            FederalIncome::Scholarship { data } => {
                IncomeSource::Record { value: crate::IncomeRecord::Scholarship(data.clone()) }
            }
            FederalIncome::JuryDuty { data } => {
                IncomeSource::Record { value: crate::IncomeRecord::JuryPay(data.clone()) }
            }
            FederalIncome::HobbySales { data } => IncomeSource::Record {
                value: crate::IncomeRecord::HobbySales(crate::HobbySales {
                    related: crate::RelatedForms {
                        forms: data
                            .related
                            .iter()
                            .map(|&key| crate::RelatedFormView { key, summary: None })
                            .collect(),
                        ..Default::default()
                    },
                    receipts: data.receipts,
                    inventory: data.inventory.clone(),
                    ordinary: data.ordinary,
                }),
            },
            FederalIncome::WildfireRelief { data } => IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::WildfireRelief(data.clone()),
            },
            FederalIncome::DisasterRelief { data } => IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::DisasterRelief(data.clone()),
            },
            FederalIncome::CasualtyLivingInsurance { data } => IncomeSource::Record {
                value: crate::IncomeRecord::CasualtyLivingInsurance(data.clone()),
            },
            FederalIncome::DisasterMitigation { data } => IncomeSource::Record {
                value: crate::IncomeRecord::DisasterMitigation(data.clone()),
            },
            FederalIncome::OtherIncome { data } => {
                IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(data.clone()) }
            }
            FederalIncome::Nec1099 { data } => IncomeSource::SelfEmployment {
                value: crate::SelfEmploymentDocument::Form1099Nec(data.form.clone()),
            },
            FederalIncome::Misc1099 { data } => IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Form1099Misc(data.form.clone()),
            },
            FederalIncome::K1099 { data } => IncomeSource::SelfEmployment {
                value: crate::SelfEmploymentDocument::Form1099K(data.form.clone()),
            },
            FederalIncome::Royalty { data } => IncomeSource::Record {
                value: crate::IncomeRecord::Royalties(RoyaltyProperty {
                    related: crate::RelatedForms {
                        forms: data
                            .related
                            .iter()
                            .map(|&key| crate::RelatedFormView { key, summary: None })
                            .collect(),
                        ..Default::default()
                    },
                    kind: data.kind.clone(),
                    receipts: data.receipts,
                    advance: data.advance,
                    expenses: rental_expense_items(&data.expenses),
                    depletion: data.depletion.clone(),
                    special: !data.ordinary,
                }),
            },
            FederalIncome::NonbusinessProperty {
                kind:
                    NonbusinessPropertyKind::PersonalRental {
                        basis,
                        depreciable_basis,
                        assets,
                        history,
                        related,
                    },
                receipts,
                expenses,
                ..
            } => IncomeSource::Record {
                value: crate::IncomeRecord::EquipmentRental(EquipmentRent {
                    related: crate::RelatedForms {
                        forms: related
                            .iter()
                            .map(|&key| crate::RelatedFormView { key, summary: None })
                            .collect(),
                        ..Default::default()
                    },
                    receipts: *receipts,
                    expenses: *expenses,
                    basis: *basis,
                    depreciable_basis: *depreciable_basis,
                    assets: assets.iter().map(asset_item).collect(),
                    history: *history,
                }),
            },
            FederalIncome::Business { data }
                if matches!(data.activity, BusinessActivity::Farming { .. }) =>
            {
                IncomeSource::Record {
                    value: crate::IncomeRecord::Farm(crate::farm_page::farm_item(
                        &entry.label,
                        data,
                        farm_se.contains(&entry.owner.owned()),
                    )?),
                }
            }
            FederalIncome::Business { data } => IncomeSource::Record {
                value: crate::IncomeRecord::Business(business_item(&entry.label, data)?),
            },
            FederalIncome::Rental { data } => IncomeSource::Record {
                value: crate::IncomeRecord::Rental(rental_item(&entry.label, data)?),
            },
            FederalIncome::FarmRental { data } => IncomeSource::Record {
                value: crate::IncomeRecord::FarmRental(crate::farm_page::farm_rental_item(
                    &entry.label,
                    data,
                )?),
            },
            FederalIncome::Gambling { data } => match data {
                crate::GamblingSource::W2g(source) => IncomeSource::Gambling(source.form.clone()),
                crate::GamblingSource::OwnRecords(records) => IncomeSource::Record {
                    value: crate::IncomeRecord::GamblingLosses(records.clone()),
                },
            },
        };
    if !matches!(entry.income, FederalIncome::Wage { .. }) {
        if let Some(source) = &entry.state_source
            && !source.apply(&mut document)
        {
            return None;
        }
        if let Some(names) = entry.printed_names() {
            document.set_printed_names(names);
        }
    }
    // A business shows no amount: its profit depends on the whole return.
    // A farm's line 34 is filled from the calculation by topic_page.
    let preview = match document {
        IncomeSource::Record { value: crate::IncomeRecord::Business(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::Farm(_) } => None,
        _ => document_amount(&entry.income, separate),
    };
    Some(IncomeDocumentItem {
        refund_management: entry.refund_management,
        refund_management_needed: None,
        allowed_community: Vec::new(),
        investment_caps: None,
        id: entry.id,
        owner: entry.owner,
        preview,
        document,
        community: entry.community,
    })
}
// The children whose interest and dividends the return can report on Form
// 8814, in the dependents' order: a child, adopted child or stepchild (not a
// foster child) under 19 or a full-time student under 24 at the end of 2026,
// without a joint return of their own (child_election::eligible_child).
pub(crate) fn child_election_children(d: &FederalDraft) -> Vec<&FederalDependent> {
    d.dependents().iter().filter(|c| crate::child_election::eligible_child(c)).collect()
}
// The same children as the page's read-only catalog: the stored identity is
// the key, the label the child's name as FormattedText.
fn child_election_choices(d: &FederalDraft) -> Vec<crate::ChildChoice> {
    child_election_children(d)
        .into_iter()
        .map(|c| crate::ChildChoice { id: c.id, label: c.name() })
        .collect()
}
// The Form 1095-A page (T2-27p). The page is policy-major (one Form 1095-A,
// its months); the store is month-major (MarketplaceClaim::Yes, each month
// listing its policies). These functions transpose both ways.
fn filing_status(d: &FederalDraft) -> FederalStatus {
    d.setup.as_ref().map_or(FederalStatus::Single, |s| s.filing_status())
}
fn sale_id(home: &mut MortgageSubsidyEntry) -> Option<&mut i64> {
    match &mut home.disposition {
        MortgageSubsidyDisposition::Ordinary {
            transfer: MortgageSubsidyTransfer::Sale { home_sale_income_id, .. },
            ..
        } => Some(home_sale_income_id),
        _ => None,
    }
}
// A Form 1040 line of a calculated return, in cents; 0 when absent.
fn form_1040_line(result: &FederalResult, reference: &str) -> i64 {
    result
        .return_lines
        .iter()
        .find(|l| {
            l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form1040 && f.line == reference)
        })
        .map_or(0, |l| l.amount)
}
// A calculation line of a calculated return, in cents; 0 when absent.
fn calculation_line(result: &FederalResult, id: &str) -> i64 {
    result.lines.iter().find(|l| l.id == id).map_or(0, |l| l.amount)
}
// The position of a page in the sequence: TaxSection is declared in
// sequence order (M3-P19), and the income by period page belongs to the
// payments section.
fn rank(page: i32) -> i32 {
    let page = match page {
        INCOME_BY_PERIOD_PAGE => PAYMENTS_PAGE,
        PARENT_TAX_PAGE => OTHER_TAXES_PAGE,
        COMMUNITY_PAGE => PEOPLE_PAGE,
        SPOUSE_BOXES_PAGE | SPOUSE_ITEMIZES_PAGE => DEDUCTIONS_PAGE,
        _ => page,
    };
    SEQUENCE
        .iter()
        .position(|section| *section as i32 == page)
        .map_or(i32::MAX, |position| position as i32)
}
// The review's remaining items (M3-P16). Every page in sequence order, each
// hub followed by its topics: the items are listed in this order.
pub(crate) const SEQUENCE: [TaxSection; 51] = [
    TaxSection::Setup,
    TaxSection::People,
    TaxSection::Spouse,
    TaxSection::Children,
    TaxSection::Documents,
    TaxSection::IncomeWithoutForm,
    TaxSection::Checklist,
    TaxSection::Income,
    TaxSection::Wages,
    TaxSection::InterestDividends,
    TaxSection::InvestmentSales,
    TaxSection::Retirement,
    TaxSection::SelfEmployment,
    TaxSection::HomeOffices,
    TaxSection::Rentals,
    TaxSection::GovernmentBenefits,
    TaxSection::OtherIncome,
    TaxSection::LongTermCareBenefits,
    TaxSection::Gambling,
    TaxSection::Farming,
    TaxSection::PassThrough,
    TaxSection::CanceledDebt,
    TaxSection::WorkAbroad,
    TaxSection::Adjustments,
    TaxSection::Hsa,
    TaxSection::SelfEmployedBenefits,
    TaxSection::IraContributions,
    TaxSection::Educator,
    TaxSection::StudentLoan,
    TaxSection::EmployeeExpenses,
    TaxSection::MoreAdjustments,
    TaxSection::Deductions,
    TaxSection::Itemized,
    TaxSection::Medical,
    TaxSection::Taxes,
    TaxSection::Mortgage,
    TaxSection::GiftCarryovers,
    TaxSection::OtherItemized,
    TaxSection::Credits,
    TaxSection::Care,
    TaxSection::Education,
    TaxSection::Saver,
    TaxSection::Marketplace,
    TaxSection::MoreCredits,
    TaxSection::OtherTaxes,
    TaxSection::ParentTax,
    TaxSection::Payments,
    TaxSection::IncomeByPeriod,
    TaxSection::InjuredSpouse,
    TaxSection::OftenMissed,
    TaxSection::Review,
];
fn sequence_position(section: TaxSection) -> usize {
    SEQUENCE.iter().position(|s| *s == section).unwrap_or(SEQUENCE.len())
}
// Resolve current issue ownership against this return. An unavailable record
// leaves its issue readable on Review; equal display names never choose a record.
pub(crate) fn issue_page(
    d: &FederalDraft,
    context: &crate::IssueContext,
    message_key: Option<&str>,
) -> Option<TaxSection> {
    use crate::IssueContext as C;
    Some(match context {
        C::Corrective { sections, .. } => return sections.first().copied(),
        C::Scoped { source, .. } => return issue_page(d, source, message_key),
        C::Record { section, .. } => *section,
        C::Section(section) => *section,
        C::Income { id, .. } => {
            return d.income.iter().find(|e| e.id == *id).and_then(|e| {
                if source_in_editor(d, e, TaxSection::Documents) {
                    Some(TaxSection::Documents)
                } else if message_key.is_some_and(|key| {
                    key.starts_with("SA1099_")
                        || key.starts_with("Q1099_")
                        || key.starts_with("QA1099_")
                }) {
                    // A follow-up question is shown on the page that asks it.
                    document_followups::followup_topic(d, e)
                } else {
                    topic_of(d, e)
                }
            });
        }
        C::Business => {
            if !has_ordinary_business(d) && has_farm(d) {
                TaxSection::Farming
            } else {
                TaxSection::SelfEmployment
            }
        }
        C::Section1231 => {
            if has_ordinary_business(d) {
                TaxSection::SelfEmployment
            } else if has_farm(d) {
                TaxSection::Farming
            } else {
                TaxSection::Rentals
            }
        }
        C::ScorpAlternativeMinimum => TaxSection::PassThrough,
        C::Underpayment => TaxSection::Payments,
        C::Interview => return None,
    })
}
// Explicit correction targets take precedence over the source topic. A source
// can have several honest repair locations; an empty override has no editor.
pub(crate) fn issue_pages(
    d: &FederalDraft,
    context: &crate::IssueContext,
    key: Option<&str>,
) -> Vec<TaxSection> {
    match context {
        crate::IssueContext::Corrective { sections, .. } => sections.clone(),
        crate::IssueContext::Scoped { source, .. } => issue_pages(d, source, key),
        _ => issue_page(d, context, key).into_iter().collect(),
    }
}
// A farm or farm rental as a choice names it: the owner's own name for it, or
// its kind and identity.
fn farm_choice_label(source: &crate::farm::FarmSource) -> crate::Message {
    income_display_label(source.entry, &[])
}
// The one owner of an income entry's display name. A name the person typed
// stays literal. An unnamed entry reads as its kind's display noun over its
// identity (LABEL_ENTRY_*: "Business 2", "W-2 1"), a typed Number argument; a
// Form 8814 election reads as its child's name, looked up on the people page.
// The correction context of an income entry, named by income_display_label.
pub(crate) fn income_context(
    entry: &FederalIncomeEntry,
    dependents: &[FederalDependent],
) -> crate::IssueContext {
    crate::IssueContext::Income { id: entry.id, label: income_display_label(entry, dependents) }
}
// The same without the people page: a Form 8814 election is then named by its
// kind and identity. Callers that hold the draft use income_context.
// The situations the Form 1116 checklist offers, by stored code.
fn foreign_situation_choices() -> Vec<crate::ForeignSituationChoice> {
    crate::ForeignTaxSituation::ALL
        .into_iter()
        .map(|situation| crate::ForeignSituationChoice {
            code: situation.code(),
            label: crate::Message::new(situation.label_key(), alloc::vec![]).encode(),
        })
        .collect()
}
pub(crate) fn income_issue_context(entry: &FederalIncomeEntry) -> crate::IssueContext {
    income_context(entry, &[])
}
// Schedule B Part I lists every payer once interest or ordinary dividends
// are above $1,500 (the estimate emits the Schedule B line only then): a
// 1099 saved without a payer name is then a remaining item.
fn payer_names_needed(d: &FederalDraft, result: &FederalResult) -> bool {
    let above = |id: &str| result.lines.iter().any(|l| l.id == id);
    d.income.iter().any(|e| match e.income {
        FederalIncome::Interest { .. } => {
            e.label.trim().is_empty() && above("schedule_b_taxable_interest")
        }
        FederalIncome::Oid { .. } => {
            e.label.trim().is_empty() && above("schedule_b_taxable_interest")
        }
        FederalIncome::Dividend { .. } => {
            e.label.trim().is_empty() && above("schedule_b_ordinary_dividends")
        }
        _ => false,
    })
}
#[cfg(test)]
pub(crate) fn resource_text(key: &str) -> &'static str {
    resource(key)
}
#[cfg(test)]
fn resource(key: &str) -> &'static str {
    APP_STRINGS.iter().find(|(k, _)| *k == key).map_or("", |(_, text)| text)
}
// Everything left before the return can be finished, one item each: the
// page that fixes it and what is left. Refused situations, named by their
// page; refusals stored on pages and the estimate's inconsistencies, from
// typed corrective destinations; facts a later answer made necessary.
// Issues without an editor remain readable blockers in the estimate and
// receive no correction link. Keep distinct source identities in page order.
pub(crate) fn remaining_items(
    d: &FederalDraft,
    result: &FederalResult,
    open: TaxSection,
) -> Vec<ReviewItem> {
    let mut items: Vec<_> = remaining_rows(d, result, Some(open))
        .into_iter()
        .map(|(section, status, _, identity)| ReviewItem {
            field: document_reveal_field(d, section, identity.as_ref()),
            item: section_message(section).encode(),
            status,
            section,
        })
        .collect();
    items.extend(
        document_followups::pending_questions(d, result).into_iter().map(|(_, _, row)| row),
    );
    items.sort_by_key(|row| sequence_position(row.section));
    items
}
// The same items with whether each is a situation refused by design. The
// calculation's request to finish the interview opens `open`; without one
// the estimate's own pages leave it out, since the next step is Next.
fn remaining_rows(
    d: &FederalDraft,
    result: &FederalResult,
    open: Option<TaxSection>,
) -> Vec<(TaxSection, Vec<u8>, bool, Option<crate::federal_model::NoteIdentity>)> {
    let mut items: Vec<(TaxSection, Vec<u8>, Option<crate::federal_model::NoteIdentity>, bool)> =
        Vec::new();
    let mut refused: Vec<&str> = Vec::new();
    // The spouse page's injured spouse item is answered on its own page
    // (Form 8379); the refused whole-return items are named on About you.
    if injured_chosen(d) && d.injured_spouse.is_none() {
        items.push((TaxSection::InjuredSpouse, status("REVIEW_NOT_ANSWERED"), None, false));
    }
    for (bit, key, name_key) in crate::federal_rules::WHOLE_RETURN_REFUSALS {
        if d.whole_return() & bit != 0 {
            refused.push(key);
            items.push((
                TaxSection::People,
                libertas_formatted_text(
                    "REVIEW_NOT_CALCULATED",
                    &[LibertasMessageArgument::ResourceText(name_key)],
                ),
                None,
                true,
            ));
        }
    }
    for refusal in d.refusals() {
        refused.push(refusal.issue_key);
        items.push((
            TaxSection::OftenMissed,
            libertas_formatted_text(
                "REVIEW_NOT_CALCULATED",
                &[LibertasMessageArgument::ResourceText(refusal.name_key)],
            ),
            None,
            true,
        ));
    }
    let pending = document_followups::pending_questions(d, result);
    for issue in &result.issues {
        if pending.iter().any(|(id, key, row)| {
            *key == issue.display.key
                && match issue.context.source() {
                    crate::IssueContext::Income { id: source, .. } => *id == Some(*source),
                    _ => {
                        let pages = issue_pages(d, &issue.context, Some(key));
                        pages.contains(&row.section)
                            || (pages.is_empty()
                                && issue_page(d, issue.context.source(), Some(key))
                                    == Some(row.section))
                    }
                }
        }) {
            continue;
        }
        if refused.contains(&issue.display.key.as_str()) {
            continue;
        }
        let key = Some(issue.display.key.as_str());
        let pages = if issue.context == crate::IssueContext::Interview {
            match open {
                Some(page) => alloc::vec![page],
                None => continue,
            }
        } else {
            match issue_pages(d, &issue.context, key) {
                // An issue holds the estimate until an answer changes, so it
                // opens its source's page even where a note offers no editor.
                pages if pages.is_empty() => {
                    issue_page(d, issue.context.source(), key).into_iter().collect()
                }
                // Alternative pages that can each fix the issue list it once:
                // under its source's own topic when that is one of them, else
                // the first in the walk's order.
                pages => {
                    let own = match issue.context.source() {
                        crate::IssueContext::Income { id, .. } => {
                            d.income.iter().find(|e| e.id == *id).and_then(|e| topic_of(d, e))
                        }
                        source => issue_page(d, source, key),
                    }
                    .filter(|page| pages.contains(page));
                    own.or_else(|| pages.iter().copied().min_by_key(|p| sequence_position(*p)))
                        .into_iter()
                        .collect()
                }
            }
        };
        debug_assert!(!pages.is_empty(), "estimate issue {} names no page", issue.display.key);
        let identity = Some(issue.context.identity());
        let status = contextual_message(
            &issue.display,
            &display_issue_context(d, &issue.display, &issue.context),
        )
        .encode();
        for page in pages {
            items.push((page, status.clone(), identity.clone(), false));
        }
    }
    // The parent's return page while Form 8615 applies and it is not
    // answered, when no estimate issue names it already.
    if !items.iter().any(|(s, _, _, _)| *s == TaxSection::ParentTax) && parent_missing(d) {
        items.push((TaxSection::ParentTax, status("REVIEW_NOT_ANSWERED"), None, false));
    }
    if payer_names_needed(d, result) {
        items.push((TaxSection::Documents, status("REVIEW_PAYER_NAMES_NEEDED"), None, false));
    }
    if hsa_account_missing(d) {
        items.push((TaxSection::Hsa, status("REVIEW_HSA_ACCOUNT_NEEDED"), None, false));
    }
    if crate::moving::claim_missing(d) {
        items.push((
            TaxSection::MoreAdjustments,
            status("REVIEW_MOVING_EXPENSES_NEEDED"),
            None,
            false,
        ));
    }
    if crate::mortgage_recapture::gift_owner_missing(d, result) {
        items.push((TaxSection::OtherTaxes, status("MORTGAGE_GIFT_OWNER_REQUIRED"), None, false));
    }
    if mortgage_method_needed(d) {
        items.push((TaxSection::Mortgage, status("MORTGAGE_AVERAGE_NEEDED"), None, false));
    }
    for (page, key) in unmarked_refusals(d) {
        items.push((page, status(key), None, true));
    }
    let mut unique: Vec<(TaxSection, Vec<u8>, Option<crate::federal_model::NoteIdentity>, bool)> =
        Vec::new();
    for item in items {
        if !unique.contains(&item) {
            unique.push(item);
        }
    }
    unique.sort_by_key(|(section, ..)| sequence_position(*section));
    unique
        .into_iter()
        .map(|(section, status, identity, refusal)| (section, status, refusal, identity))
        .collect()
}
// Whether a page is still ahead on the path: the frontier has not passed it
// and nothing is entered on it.
fn coming_up(d: &FederalDraft, section: TaxSection) -> bool {
    let top = walk::topic_hub(section).unwrap_or(section);
    let (page, frontier) = (rank(top as i32), rank(first_open(d)));
    page > frontier
        || (page == frontier
            && !d.finished_sections.contains(&section)
            && !topic_entered(d, section))
}
// Whether a topic holds entries; the care page's are the plan itself.
fn topic_entered(d: &FederalDraft, topic: TaxSection) -> bool {
    topic_has_data(d, topic)
        || (topic == TaxSection::Care && d.credits.as_ref().is_some_and(|c| c.care.active()))
}
fn coming_up_item(section: TaxSection) -> ReviewItem {
    ReviewItem {
        field: None,
        item: section_message(section).encode(),
        status: status("REVIEW_COMING_UP"),
        section,
    }
}
// The answer an estimate item reveals when its page opens.
fn reveal_field(section: TaxSection, text: &[u8]) -> Option<String> {
    let path: &[&'static str] =
        if section == TaxSection::Income && status_is(text, "CROP_PAPER_FACTS_NEEDED") {
            &["crop_insurance"]
        } else if section == TaxSection::Mortgage && status_is(text, "MORTGAGE_AVERAGE_NEEDED") {
            &["value", "loans"]
        } else if status_is(text, "ISSUE_ANSWER_WHETHER_COMMUNITY_PROPERTY_APPLIES")
            || status_is(text, "ISSUE_ANSWER_YOUR_SPOUSE_STANDARD_DEDUCTION_BOXES")
            || status_is(text, "ISSUE_ANSWER_SPOUSE_ITEMIZES")
        {
            // Each opens its own step, whose answer is its value.
            &["value"]
        } else if status_is(text, "WARN_SECTION_179_SPOUSE_RECORDS_LEFT_OUT") {
            &["value", "documents"]
        } else {
            return None;
        };
    Some(path.iter().fold(libertas::SchemaPath::root(), |path, name| path.field(name)).into())
}
// Document destinations retain the accepted source identity while the existing
// field mappings above continue to own the other pages.
fn document_reveal_field(
    d: &FederalDraft,
    section: TaxSection,
    identity: Option<&crate::federal_model::NoteIdentity>,
) -> Option<String> {
    if section != TaxSection::Documents {
        return None;
    }
    let mut identity = identity?;
    loop {
        match identity {
            crate::federal_model::NoteIdentity::Income(id) => {
                return document_center::accepted_document_row_path(d, *id);
            }
            crate::federal_model::NoteIdentity::Item { parent, .. } => identity = parent,
            _ => return None,
        }
    }
}
// What the estimate still needs, each item once, in one of three states, and
// every item opening its page: coming up (the path has not reached the page),
// to fix (the page was passed with the fact missing or wrong), or not
// calculated (a situation refused by design, opening where it was answered).
// The review keeps this list and adds its finishing row.
fn estimate_items(d: &FederalDraft, result: &FederalResult, open: TaxSection) -> Vec<ReviewItem> {
    let mut items: Vec<ReviewItem> = Vec::new();
    for (section, text, refusal, identity) in remaining_rows(d, result, None) {
        if !refusal && section != open && coming_up(d, section) {
            // Every fact on a page ahead is the one page ahead.
            let item = coming_up_item(section);
            if !items.contains(&item) {
                items.push(item);
            }
        } else {
            let field = reveal_field(section, &text)
                .or_else(|| document_reveal_field(d, section, identity.as_ref()));
            items.push(ReviewItem {
                field,
                item: section_message(section).encode(),
                status: text,
                section,
            });
        }
    }
    items.extend(
        document_followups::pending_questions(d, result).into_iter().map(|(_, _, row)| row),
    );
    items.sort_by_key(|row| sequence_position(row.section));
    items
}
pub(crate) fn resolved_issue_context(
    d: &FederalDraft,
    context: &crate::IssueContext,
) -> crate::IssueContext {
    use crate::IssueContext as C;
    match context {
        C::Income { id, .. } => d
            .income
            .iter()
            .find(|entry| entry.id == *id)
            .map_or_else(|| context.clone(), |entry| income_context(entry, d.dependents())),
        C::Corrective { source, sections } => {
            resolved_issue_context(d, source).alternatives(sections)
        }
        C::Scoped { source, identity, label } => C::Scoped {
            source: alloc::boxed::Box::new(resolved_issue_context(d, source)),
            identity: identity.clone(),
            label: label.clone(),
        },
        _ => context.clone(),
    }
}
// A message that already names its record keeps that name; any other is prefixed with the
// shared document name.
fn display_issue_context(
    d: &FederalDraft,
    message: &crate::Message,
    context: &crate::IssueContext,
) -> crate::IssueContext {
    let resolved = resolved_issue_context(d, context);
    if crate::diagnostics::message_names_label(message, &resolved.label()) {
        resolved
    } else {
        named_issue_context(d, &resolved)
    }
}
// What a person reads: a document is named once, by its form, issuer and the person it
// was printed for. The stored context keeps its identity and label for comparison.
fn named_issue_context(d: &FederalDraft, context: &crate::IssueContext) -> crate::IssueContext {
    use crate::IssueContext as C;
    match resolved_issue_context(d, context) {
        C::Income { id, label } => C::Income {
            id,
            label: document_owners::document_name(d, id).map_or(label, |name| {
                crate::Message::new("TEXT", alloc::vec![crate::MessageArg::Encoded(name)])
            }),
        },
        C::Corrective { source, sections } => {
            named_issue_context(d, &source).alternatives(&sections)
        }
        C::Scoped { source, identity, label } => C::Scoped {
            source: alloc::boxed::Box::new(named_issue_context(d, &source)),
            identity,
            label,
        },
        other => other,
    }
}
// Affected-record names are display data. The typed identity still owns dedup
// and routing when two records have the same name or a name changes.

fn correction_notes(d: &FederalDraft, result: &mut FederalResult) -> Vec<ReviewItem> {
    let mut rows = Vec::new();
    result.review_notes.retain(|note| {
        let Some(context) = &note.context else {
            return true;
        };
        let pages = issue_pages(d, context, None);
        for section in &pages {
            let status = contextual_message(
                &note.display,
                &display_issue_context(d, &note.display, context),
            )
            .encode();
            rows.push(ReviewItem {
                field: reveal_field(*section, &status)
                    .or_else(|| document_reveal_field(d, *section, Some(&context.identity()))),
                item: section_message(*section).encode(),
                status,
                section: *section,
            });
        }
        pages.is_empty()
    });
    rows
}
// A topic's attention text: its first remaining item, which its hub row
// shows and the review lists.
fn attention_text(items: &[ReviewItem], topic: TaxSection) -> Option<Vec<u8>> {
    items.iter().find(|i| i.section == topic).map(|i| attention_status(&i.status))
}
// A hub's rows with their attention: a topic with a remaining item shows it
// instead of its summary, and is listed even when nothing is entered.
fn mark_attention(
    mut rows: Vec<ReviewItem>,
    topics: &[TaxSection],
    items: &[ReviewItem],
) -> Vec<ReviewItem> {
    let order =
        |section: TaxSection| topics.iter().position(|t| *t == section).unwrap_or(topics.len());
    for topic in topics {
        let Some(status) = attention_text(items, *topic) else {
            continue;
        };
        if let Some(row) = rows.iter_mut().find(|r| r.section == *topic) {
            row.status = status;
        } else {
            let at = rows.iter().take_while(|r| order(r.section) < order(*topic)).count();
            rows.insert(
                at,
                ReviewItem {
                    field: None,
                    item: section_message(*topic).encode(),
                    status,
                    section: *topic,
                },
            );
        }
    }
    rows
}
// Beyond your documents. Each flag of the three questions is one topic on
// the path: the page that answers it, the words its hub row and review item
// use, and the data its box owns. Two flags may share a box (gifts to
// charity are entered on the deductions page and, for property, on the gifts
// page); the box is checked while either has entries, and each flag clears
// its own page's share.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Catalog {
    Activities,
    Paid,
    Carryovers,
}
struct Flag {
    catalog: Catalog,
    bit: u32,
    // As "What you told us" lists it.
    label_key: &'static str,
    // Complete warning when this unchecked topic discards its answers.
    removal_key: &'static str,
    // The page that answers it.
    section: TaxSection,
    // Whether the return holds entries for it: the data its box owns, never
    // a document.
    entered: fn(&FederalDraft) -> bool,
    // Makes that data absent, as saving its page without it would (the
    // prune's line for an unchecked topic with entries).
    clear: fn(&mut FederalDraft),
}
const fn flag(
    catalog: Catalog,
    bit: u32,
    label_key: &'static str,
    removal_key: &'static str,
    section: TaxSection,
    entered: fn(&FederalDraft) -> bool,
    clear: fn(&mut FederalDraft),
) -> Flag {
    Flag { catalog, bit, label_key, removal_key, section, entered, clear }
}
// The document that checks a box, as its note names it: a 1099-NEC or 1099-K
// shows a business of your own, a 1099-PATR or a 1099-G agriculture payment
// (box 7) a farm, a W-2 box 12 code W or an HSA's 1099-SA a health savings
// account, a W-2 box 10 dependent care benefits (reported on Form 2441 even
// with no care paid). Such a box opens checked and is saved unchecked; it
// stays on the path while the document is entered.
fn evidence(d: &FederalDraft, flag: &Flag) -> Option<&'static str> {
    let business = flag.catalog == Catalog::Activities && flag.bit == ActivityTopics::BUSINESS.0;
    let farm = flag.catalog == Catalog::Activities && flag.bit == ActivityTopics::FARM.0;
    let hsa = flag.catalog == Catalog::Paid && flag.bit == PaidTopics::HSA.0;
    let care = flag.catalog == Catalog::Paid && flag.bit == PaidTopics::CARE.0;
    d.income.iter().find_map(|e| match &e.income {
        FederalIncome::Wage { data }
            if care && crate::FederalWage::from(data).care_benefits > 0 =>
        {
            Some("W-2")
        }
        FederalIncome::Wage { data }
            if hsa
                && crate::FederalWage::from(data)
                    .other_box12
                    .iter()
                    .any(|r| r.code == crate::W2Code::W && r.amount > 0) =>
        {
            Some("W-2")
        }
        FederalIncome::MedicalDistribution { data }
            if hsa && *data.account() == crate::MedicalSaAccount::Hsa =>
        {
            Some(if data.trustee_statement_paper().is_some() {
                "HSA/MSA trustee statement"
            } else {
                "1099-SA"
            })
        }
        FederalIncome::Nec1099 { .. } if business => Some("1099-NEC"),
        FederalIncome::K1099 { .. } if business => Some("1099-K"),
        FederalIncome::Cooperative { .. } if farm => Some("1099-PATR"),
        FederalIncome::GovernmentPayment { data }
            if farm && crate::FederalUnemployment::from(data).agriculture_payment > 0 =>
        {
            Some("1099-G")
        }
        _ => None,
    })
}
// Saving a topic page without the records `drop` names: its documents and
// every other record stay. A return whose other answers the page would
// refuse keeps them.
fn drop_records(d: &mut FederalDraft, topic: TaxSection, drop: fn(&crate::IncomeRecord) -> bool) {
    let viewing = Interview::viewing(d.clone());
    let Reply::Edit(P::SaveIncomes { documents, .. }) = viewing.topic_page(topic, false) else {
        return;
    };
    let kept = IncomeRecords {
        documents: documents.documents.into_iter().filter(|i| !drop(&i.document)).collect(),
        ..documents
    };
    if let Ok(mut draft) = viewing.documents_candidate(topic, kept.into_edits()) {
        draft.finished_sections = d.finished_sections.clone();
        *d = draft;
    }
}
fn qbi_group_input_issue(
    item: &IncomeDocumentItem,
    has_aggregations: bool,
) -> Option<ValidationIssue> {
    let group = match &item.document {
        IncomeSource::K1 { value: crate::K1Document::K1Partnership(v) } => {
            v.personal.as_ref().map(|records| records.qbi_group.as_str())
        }
        IncomeSource::Record { value: crate::IncomeRecord::Business(v) } if has_aggregations => {
            Some(v.more.qbi_group.as_str())
        }
        IncomeSource::Record { value: crate::IncomeRecord::Rental(v) } if has_aggregations => {
            Some(v.more.qbi_group.as_str())
        }
        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(v) } if has_aggregations => {
            Some(v.more.qbi_group.as_str())
        }
        _ => None,
    };
    let key = match group {
        Some(group) if !group.is_empty() && item.id == 0 => "QBI_AGGREGATION_ACCEPT_SOURCE_FIRST",
        Some(group)
            if !group.is_empty()
                && matches!(&item.document,
            IncomeSource::K1 { value: crate::K1Document::K1Partnership(v) } if v.form.publicly_traded) =>
        {
            "QBI_AGGREGATION_PARENT_NOT_BUSINESS"
        }
        _ => return None,
    };
    let issue = ValidationIssue::new(key).at_field("qbi_group");
    Some(if matches!(&item.document, IncomeSource::K1 { .. }) {
        issue
    } else {
        issue.in_field("more")
    })
}

fn topic_followups_present(d: &FederalDraft, topic: TaxSection) -> bool {
    Interview::document_followup_rows(d, Some(topic), None).into_iter().any(|(_, row)| {
        // A W-2 row asks only what its boxes call for; one with no call has
        // no question on the wages page.
        !matches!(row.answer, crate::DocumentAnswer::Wage { answers } if answers.calls == crate::W2Calls::default())
    })
}
fn topic_followups_needed(d: &FederalDraft, topic: TaxSection) -> bool {
    document_followups::topic_followups_needed(d, topic)
}
fn income_topic_entered(d: &FederalDraft, topic: TaxSection) -> bool {
    income_topic_records_entered(d, topic) || topic_followups_present(d, topic)
}
fn income_topic_records_entered(d: &FederalDraft, topic: TaxSection) -> bool {
    d.income.iter().any(|entry| source_in_editor(d, entry, topic))
        || data_answer_items(d, topic).is_some_and(|items| {
            items.iter().any(|row| matches!(row.document, IncomeSource::Record { .. }))
        })
        || (topic == TaxSection::Retirement
            && d.adjustments.as_ref().is_some_and(|a| {
                !a.ira_basis.is_empty()
                    || !a.qualified_charity.is_empty()
                    || a.social_election != SocialElection::Regular
                    || !a.required_distributions.is_empty()
                    || !a.roth_history.is_empty()
            }))
}
fn has_income(d: &FederalDraft, wanted: fn(&FederalIncome) -> bool) -> bool {
    d.income.iter().any(|e| wanted(&e.income))
}
// The Form 3921 exercises and ISO shares exercised and sold in 2026 on
// Investment sales.
fn iso_entered(d: &FederalDraft) -> bool {
    d.screening.as_ref().is_some_and(|s| !s.iso_exercises.is_empty())
        || d.income.iter().any(|e| {
            matches!(&e.income, FederalIncome::IsoSale { data } if data.exercised / 10000 == 2026)
        })
}
fn screening(d: &FederalDraft) -> Option<&FederalScreening> {
    d.screening.as_ref()
}
const FLAGS: [Flag; 19] = [
    flag(
        Catalog::Activities,
        ActivityTopics::BUSINESS.0,
        "CHECKLIST_A_BUSINESS_OF_YOUR_OWN",
        "SAVING_REMOVES_TOPIC_SELF_EMPLOYMENT_OR_GIG_WORK",
        TaxSection::SelfEmployment,
        |d| {
            has_income(d, |i| {
                matches!(i, FederalIncome::Business { data }
                    if matches!(data.activity, BusinessActivity::Ordinary { .. } | BusinessActivity::Gambling { .. }))
            }) || screening(d).is_some_and(|s| s.optional_se.iter().any(|e| e.nonfarm.is_some()))
        },
        |d| {
            drop_records(d, TaxSection::SelfEmployment, |s| {
                matches!(
                    s,
                    crate::IncomeRecord::Business(_)
                        | crate::IncomeRecord::NonfarmOptionalMethod(_)
                )
            })
        },
    ),
    flag(
        Catalog::Activities,
        ActivityTopics::RENTALS.0,
        "CHECKLIST_RENTAL_PROPERTY",
        "SAVING_REMOVES_TOPIC_A_RENTAL_PROPERTY",
        TaxSection::Rentals,
        |d| {
            has_income(d, |i| {
                matches!(
                    i,
                    FederalIncome::Rental { .. }
                        | FederalIncome::Royalty { .. }
                        | FederalIncome::NonbusinessProperty {
                            kind: NonbusinessPropertyKind::PersonalRental { .. },
                            ..
                        }
                )
            })
        },
        |d| {
            drop_records(d, TaxSection::Rentals, |s| {
                matches!(s, crate::IncomeRecord::Rental(_) | crate::IncomeRecord::Royalties(_))
            });
            drop_records(d, TaxSection::OtherIncome, |s| {
                matches!(s, crate::IncomeRecord::EquipmentRental(_))
            });
        },
    ),
    flag(
        Catalog::Activities,
        ActivityTopics::FARM.0,
        "CHECKLIST_FARMING",
        "SAVING_REMOVES_TOPIC_FARMING",
        TaxSection::Farming,
        |d| {
            has_income(d, |i| {
                matches!(i, FederalIncome::FarmRental { .. })
                    || matches!(i, FederalIncome::Business { data }
                        if matches!(data.activity, BusinessActivity::Farming { .. }))
            })
        },
        |d| {
            drop_records(d, TaxSection::Farming, |s| {
                matches!(
                    s,
                    crate::IncomeRecord::Farm(_)
                        | crate::IncomeRecord::FarmRental(_)
                        | crate::IncomeRecord::FarmAveraging(_)
                        | crate::IncomeRecord::FarmLoss(_)
                )
            })
        },
    ),
    flag(
        Catalog::Activities,
        ActivityTopics::HOUSEHOLD_EMPLOYEE.0,
        "CHECKLIST_A_HOUSEHOLD_EMPLOYEE",
        "SAVING_REMOVES_TOPIC_A_HOUSEHOLD_EMPLOYEE",
        TaxSection::OtherTaxes,
        |d| screening(d).is_some_and(|s| !s.household_employment.is_empty()),
        |d| {
            if let Some(s) = d.screening.as_mut() {
                s.household_employment.clear();
            }
        },
    ),
    flag(
        Catalog::Activities,
        ActivityTopics::ABROAD.0,
        "CHECKLIST_INCOME_A_JOB_OR_ACCOUNTS_OUTSIDE_THE_U_S",
        "SAVING_REMOVES_TOPIC_WORK_ABROAD_FOREIGN_EARNED_INCOME_OR_PAY_FROM_A_FOREIGN_EMPLOYER",
        TaxSection::WorkAbroad,
        |d| {
            has_income(d, |i| matches!(i, FederalIncome::ForeignEmployment { .. }))
                || screening(d).is_some_and(|s| {
                    !s.foreign_earned.is_empty() || s.foreign_account || s.foreign_trust
                })
        },
        |d| {
            drop_records(d, TaxSection::WorkAbroad, |s| {
                matches!(s, crate::IncomeRecord::Form2555(_) | crate::IncomeRecord::ForeignPay(_))
            });
            drop_records(d, TaxSection::InterestDividends, |s| {
                matches!(s, crate::IncomeRecord::ForeignAccountOrTrust(_))
            });
        },
    ),
    flag(
        Catalog::Paid,
        PaidTopics::PAYMENTS.0,
        "CHECKLIST_ESTIMATED_TAX_PAYMENTS",
        "SAVING_REMOVES_TOPIC_ESTIMATED_TAX_PAYMENTS_FOR_2026",
        TaxSection::Payments,
        |d| d.payments.as_ref().is_some_and(|p| p.estimated.count() != 0 || p.extension != 0),
        |d| {
            if let Some(p) = d.payments.as_mut() {
                p.estimated = crate::EstimatedPaymentEntryRecord::None;
                p.extension = 0;
                p.extension_payer = None;
                p.extension_refund_property = None;
            }
        },
    ),
    flag(
        Catalog::Paid,
        PaidTopics::TAXES.0,
        "CHECKLIST_STATE_LOCAL_OR_PROPERTY_TAXES",
        "SAVING_REMOVES_TOPIC_STATE_LOCAL_OR_PROPERTY_TAXES",
        TaxSection::Taxes,
        |d| itemized_entered(d, TaxSection::Taxes),
        |d| {
            if let Some(facts) = itemized_facts_mut(d) {
                facts.personal_tax = PersonalTaxElection::None;
                facts.real_estate_taxes = 0;
                facts.personal_property_taxes = 0;
            }
        },
    ),
    flag(
        Catalog::Paid,
        PaidTopics::GIFTS.0,
        "CHECKLIST_GIFTS_TO_CHARITY",
        "SAVING_REMOVES_TOPIC_GIFTS_TO_CHARITY",
        TaxSection::Deductions,
        gifts_entered,
        |d| {
            if let Some(v) = d.deductions.as_mut() {
                v.cash_charity = 0;
                v.fund_cash_charity = 0;
                v.other_cash_charity = 0;
            }
        },
    ),
    flag(
        Catalog::Paid,
        PaidTopics::GIFTS.0,
        "CHECKLIST_GIFTS_TO_CHARITY",
        "SAVING_REMOVES_TOPIC_GIFTS_OF_PROPERTY_OR_CHARITY_CARRIED_OVER",
        TaxSection::GiftCarryovers,
        gifts_entered,
        |d| {
            if let Some(v) = d.deductions.as_mut() {
                clear_property_gifts(v);
            }
        },
    ),
    flag(
        Catalog::Paid,
        PaidTopics::IRA_CONTRIBUTIONS.0,
        "CHECKLIST_IRA_CONTRIBUTIONS",
        "SAVING_REMOVES_TOPIC_IRA_CONTRIBUTIONS",
        TaxSection::IraContributions,
        |d| !ira_contributor_rows(d).unwrap_or_default().is_empty(),
        |d| {
            if let Some(a) = d.adjustments.as_mut() {
                a.iras = FederalIras::NONE;
            }
        },
    ),
    flag(
        Catalog::Paid,
        PaidTopics::HSA.0,
        "CHECKLIST_HEALTH_SAVINGS_ACCOUNT",
        "SAVING_REMOVES_TOPIC_A_HEALTH_SAVINGS_ACCOUNT",
        TaxSection::Hsa,
        |d| {
            !crate::contribution_documents::sources(d).is_empty()
                || !crate::health_employer::bindings(d).is_empty()
                || !hsa_accounts(d).is_empty()
                || d.adjustments
                    .as_ref()
                    .is_some_and(|a| matches!(a.archer, crate::ArcherContributions::Records { .. }))
        },
        |d| {
            if let Some(a) = d.adjustments.as_mut() {
                a.health_employer_w2.clear();
                a.hsa.clear();
                a.hsa_family_share = None;
                a.archer = crate::ArcherContributions::None;
            }
        },
    ),
    // The care page owns each person's care costs and the Form 2441 facts.
    flag(
        Catalog::Paid,
        PaidTopics::CARE.0,
        "REVIEW_CHILD_CARE_LABEL",
        "SAVING_REMOVES_TOPIC_CHILD_OR_DEPENDENT_CARE",
        TaxSection::Care,
        |d| {
            d.credits.as_ref().is_some_and(|c| c.care.active())
                || d.dependents().iter().any(|p| matches!(p.care, Some(DependentCare::Yes { .. })))
        },
        |d| {
            for person in &mut d.dependents {
                person.care = None;
                person.incapable_self_care = false;
            }
            if let Some(c) = d.credits.as_mut() {
                c.care = FederalCare { filing: c.care.filing.clone(), ..FederalCare::default() };
            }
        },
    ),
    flag(
        Catalog::Carryovers,
        CarryoverTopics::CAPITAL_LOSS.0,
        "CHECKLIST_A_CAPITAL_LOSS_CARRYOVER",
        "SAVING_REMOVES_TOPIC_A_CAPITAL_LOSS_CARRYOVER",
        TaxSection::InvestmentSales,
        |d| screening(d).is_some_and(|s| !s.capital_history.is_empty()),
        |d| {
            if let Some(s) = d.screening.as_mut() {
                s.capital_history.clear();
            }
        },
    ),
    flag(
        Catalog::Carryovers,
        CarryoverTopics::NET_OPERATING_LOSS.0,
        "CHECKLIST_A_NET_OPERATING_LOSS",
        "SAVING_REMOVES_TOPIC_A_NET_OPERATING_LOSS",
        TaxSection::SelfEmployment,
        |d| {
            screening(d).is_some_and(|s| {
                crate::LossCarryforward::from_histories(&s.nol, &s.atnol).is_some()
            })
        },
        |d| {
            if let Some(s) = d.screening.as_mut() {
                s.nol = NolHistory::None;
                s.atnol = AtnolHistory::None;
            }
        },
    ),
    flag(
        Catalog::Carryovers,
        CarryoverTopics::QBI_LOSS.0,
        "CHECKLIST_A_QBI_LOSS_CARRYFORWARD",
        "SAVING_REMOVES_TOPIC_A_QBI_LOSS_CARRYFORWARD",
        TaxSection::SelfEmployment,
        |d| screening(d).is_some_and(|s| s.qbi_history.any()),
        |d| {
            if let Some(s) = d.screening.as_mut() {
                s.qbi_history = QbiHistory::NONE;
            }
        },
    ),
    flag(
        Catalog::Carryovers,
        CarryoverTopics::CHARITY.0,
        "CHECKLIST_A_CHARITABLE_CONTRIBUTION_CARRYOVER",
        "SAVING_REMOVES_TOPIC_A_CHARITABLE_CONTRIBUTION_CARRYOVER",
        TaxSection::GiftCarryovers,
        |d| d.deductions.as_ref().is_some_and(|v| !v.charity_carryovers.is_empty()),
        |d| {
            if let Some(v) = d.deductions.as_mut() {
                v.charity_carryovers.clear();
            }
        },
    ),
    flag(
        Catalog::Carryovers,
        CarryoverTopics::MINIMUM_TAX_CREDIT.0,
        "CHECKLIST_A_MINIMUM_TAX_CREDIT",
        "SAVING_REMOVES_TOPIC_A_MINIMUM_TAX_CREDIT",
        TaxSection::MoreCredits,
        |d| d.credits.as_ref().is_some_and(|c| c.minimum_tax.is_some()),
        |d| {
            if let Some(c) = d.credits.as_mut() {
                c.minimum_tax = None;
            }
        },
    ),
    flag(
        Catalog::Carryovers,
        CarryoverTopics::IRA_BASIS.0,
        "CHECKLIST_BASIS_IN_NONDEDUCTIBLE_IRA_CONTRIBUTIONS",
        "SAVING_REMOVES_TOPIC_BASIS_IN_NONDEDUCTIBLE_IRA_CONTRIBUTIONS",
        TaxSection::Retirement,
        |d| d.adjustments.as_ref().is_some_and(|a| !a.ira_basis.is_empty()),
        |d| {
            if let Some(a) = d.adjustments.as_mut() {
                a.ira_basis.clear();
            }
        },
    ),
    flag(
        Catalog::Carryovers,
        CarryoverTopics::HOME_USE_RENTAL.0,
        "CHECKLIST_HOME_USE_RENTAL_CARRY",
        "SAVING_REMOVES_TOPIC_HOME_USE_RENTAL_CARRY",
        TaxSection::Rentals,
        |d| {
            d.income.iter().any(|e| {
                matches!(&e.income, FederalIncome::Rental { data } if data.prior_home_carry.is_some())
            })
        },
        |d| {
            for e in &mut d.income {
                if let FederalIncome::Rental { data } = &mut e.income {
                    data.prior_home_carry = None;
                }
            }
        },
    ),
];
// The deductions an itemizer claims on Schedule A: a box of one opens the
// deductions page on Compare.
const ITEMIZABLE: [(Catalog, u32); 2] =
    [(Catalog::Paid, PaidTopics::TAXES.0), (Catalog::Carryovers, CarryoverTopics::CHARITY.0)];
// The hub-level section a page belongs to, in sequence order; None before
// the income hub, for the injured spouse page (a section of its own) and for
// the review.
fn hub_of(section: TaxSection) -> Option<TaxSection> {
    const HUBS: [TaxSection; 6] = [
        TaxSection::Income,
        TaxSection::Adjustments,
        TaxSection::Deductions,
        TaxSection::Credits,
        TaxSection::OtherTaxes,
        TaxSection::Payments,
    ];
    SEQUENCE[..sequence_position(section).min(SEQUENCE.len())]
        .iter()
        .chain(core::iter::once(&section))
        .rev()
        .find(|s| HUBS.contains(s))
        .copied()
        .filter(|_| !matches!(section, TaxSection::Review | TaxSection::InjuredSpouse))
}
// The sections the walk may pass (M3-P19), in sequence order.
const WALKED_SECTIONS: [TaxSection; 9] = [
    TaxSection::Income,
    TaxSection::Adjustments,
    TaxSection::Deductions,
    TaxSection::Itemized,
    TaxSection::Credits,
    TaxSection::OtherTaxes,
    TaxSection::Payments,
    TaxSection::InjuredSpouse,
    TaxSection::OftenMissed,
];
// The sections a return shows (M3-P19): the income, adjustments and credits
// hubs always; the other taxes and deductions sections with a box, a Form
// 1098 (deductions), an entry or a remaining item; the payments page only
// with its box (estimated or extension payments, or last year's refund
// applied) or a remaining item; the itemized hub whenever the route itemizes;
// the injured spouse page while it is chosen; the last page before the review
// always.
fn sections_with_needs(d: &FederalDraft, items: &[ReviewItem]) -> Vec<TaxSection> {
    let value = checklist_value(d);
    // A box shows the hub of the page that answers it (a net operating loss
    // is answered under Income).
    let hub = |h: TaxSection| {
        FLAGS.iter().any(|f| hub_of(f.section) == Some(h) && ticked(&value, f))
            || missed_pages(d, &d.often_missed).any(|page| hub_of(page) == Some(h))
    };
    let attention = |hub: TaxSection| items.iter().any(|i| hub_of(i.section) == Some(hub));
    WALKED_SECTIONS
        .into_iter()
        .filter(|section| match section {
            TaxSection::Income
            | TaxSection::Adjustments
            | TaxSection::Credits
            | TaxSection::OftenMissed => true,
            TaxSection::OtherTaxes => hub(TaxSection::OtherTaxes),
            // The page's data answers without a tick show it too: casualty
            // events, and the repayment comparison while it is offered.
            TaxSection::Deductions => {
                hub(TaxSection::Deductions)
                    || gambling_losses(d) != 0
                    || itemizing(d)
                    || d.deductions.as_ref().is_some_and(|v| {
                        !v.mortgage.is_empty()
                            || !v.personal_casualties.is_empty()
                            || v.repayment != RepaymentClaim::None
                    })
                    || crate::claim_repayment::repayment_offered(d)
            }
            TaxSection::Itemized => itemizing(d),
            TaxSection::Payments => hub(TaxSection::Payments),
            _ => in_walk(d, TaxSection::InjuredSpouse),
        } || (*section != TaxSection::Itemized && attention(*section)))
        .collect()
}
// The sections this return shows, with the remaining items of the return
// as it stands once the income section is continued (the estimate waits
// for it).
fn shown_sections(d: &FederalDraft) -> Vec<TaxSection> {
    let mut continued = d.clone();
    finish_section(&mut continued, TaxSection::Income);
    let result = estimate(&continued);
    sections_with_needs(d, &remaining_items(&continued, &result, TaxSection::Review))
}
// The main section a page belongs to, for progress and the walk.
fn page_section(page: i32) -> Option<TaxSection> {
    Some(match page {
        SETUP_PAGE => TaxSection::Setup,
        PEOPLE_PAGE | SPOUSE_PAGE | COMMUNITY_PAGE => TaxSection::People,
        CHILDREN_PAGE => TaxSection::Children,
        DOCUMENTS_PAGE => TaxSection::Documents,
        INCOME_WITHOUT_FORM_PAGE => TaxSection::IncomeWithoutForm,
        CHECKLIST_PAGE => TaxSection::Checklist,
        INCOME_PAGE => TaxSection::Income,
        ADJUSTMENTS_PAGE => TaxSection::Adjustments,
        DEDUCTIONS_PAGE | ITEMIZED_PAGE | SPOUSE_BOXES_PAGE | SPOUSE_ITEMIZES_PAGE => {
            TaxSection::Deductions
        }
        CREDITS_PAGE => TaxSection::Credits,
        OTHER_TAXES_PAGE | PARENT_TAX_PAGE => TaxSection::OtherTaxes,
        PAYMENTS_PAGE | INCOME_BY_PERIOD_PAGE => TaxSection::Payments,
        INJURED_SPOUSE_PAGE => TaxSection::InjuredSpouse,
        OFTEN_MISSED_PAGE => TaxSection::OftenMissed,
        REVIEW_PAGE => TaxSection::Review,
        _ => return None,
    })
}
// The section after `section` in the sequence that this return shows, else
// the review (M3-P19).
fn next_shown(d: &FederalDraft, shown: &[TaxSection], section: TaxSection) -> TaxSection {
    [TaxSection::Checklist]
        .into_iter()
        .chain(WALKED_SECTIONS)
        .skip_while(|s| *s != section)
        .skip(1)
        .find(|s| match s {
            TaxSection::Itemized => itemizing(d),
            _ => shown.contains(s),
        })
        .unwrap_or(TaxSection::Review)
}
// The shown section before `section`, which Back and a hub's popup open:
// skipped sections are passed; the checklist is always shown.
fn previous_shown(d: &FederalDraft, shown: &[TaxSection], section: TaxSection) -> i32 {
    let order = [
        TaxSection::Documents,
        TaxSection::IncomeWithoutForm,
        TaxSection::Checklist,
        TaxSection::Income,
        TaxSection::Adjustments,
        TaxSection::Deductions,
        TaxSection::Itemized,
        TaxSection::Credits,
        TaxSection::OtherTaxes,
        TaxSection::Payments,
        TaxSection::InjuredSpouse,
    ];
    let at = order.iter().position(|s| *s == section).unwrap_or(order.len());
    order[..at]
        .iter()
        .rev()
        .find(|s| match s {
            TaxSection::IncomeWithoutForm | TaxSection::Checklist => true,
            TaxSection::Itemized => itemizing(d),
            _ => shown.contains(s),
        })
        .map_or(CHECKLIST_PAGE, |s| *s as i32)
}
// The credits hub's topic pages before More credits.
const CREDIT_TOPICS: [TaxSection; 4] =
    [TaxSection::Care, TaxSection::Education, TaxSection::Saver, TaxSection::Marketplace];
// The topics of a hub this return offers: those in its walk, which the
// return's facts decide (a business for self-employed benefits, a saver's
// credit that can apply, dependents for care). A topic's own tick never
// hides it: ticks order the walk and set each row's status.
fn offered_topics<'a>(
    d: &'a FederalDraft,
    topics: &'a [TaxSection],
) -> impl Iterator<Item = i32> + 'a {
    topics.iter().filter(|t| in_walk(d, **t)).map(|t| *t as i32)
}
// The topics of each hub in its order; the adjustments and credits hubs end
// with the page of their supported but uncommon items.
fn hub_topics(hub: TaxSection) -> Vec<TaxSection> {
    let (topics, more): (&[TaxSection], Option<TaxSection>) = match hub {
        TaxSection::Income => (&INCOME_TOPICS, None),
        TaxSection::Adjustments => (&ADJUSTMENT_TOPICS, Some(TaxSection::MoreAdjustments)),
        TaxSection::Itemized => (&OPEN_ITEMIZED_TOPICS, None),
        TaxSection::Credits => (&CREDIT_TOPICS, Some(TaxSection::MoreCredits)),
        _ => return Vec::new(),
    };
    topics.iter().copied().chain(more).collect()
}
// The hub a hub page is.
fn hub_section(page: i32) -> Option<TaxSection> {
    Some(match page {
        INCOME_PAGE => TaxSection::Income,
        ADJUSTMENTS_PAGE => TaxSection::Adjustments,
        ITEMIZED_PAGE => TaxSection::Itemized,
        CREDITS_PAGE => TaxSection::Credits,
        _ => return None,
    })
}
// The derived topics of a hub (M3-P18) the filer has not visited: the ones
// the return makes necessary without a tick. HSA while a W-2 reports box 12
// W for an owner with no account or an account's special answer is stored;
// self-employed benefits while a business exists and nothing is entered on
// it; IRA contributions while box 12 deferrals or a prior Form 8606 exist
// and nothing is entered; education while a student is on the return with
// no education entry; the saver's credit while a contribution is inside
// Form 8880's bands. An itemized topic is walked only when planned (M3-P19).
pub(crate) fn derived_topics(d: &FederalDraft, hub: TaxSection) -> Vec<TaxSection> {
    let mut derived = Vec::new();
    match hub {
        TaxSection::Adjustments => {
            if hsa_account_missing(d) {
                derived.push(TaxSection::Hsa);
            }
            if crate::moving::claim_missing(d) {
                derived.push(TaxSection::MoreAdjustments);
            }
            if benefits_needed(d) {
                derived.push(TaxSection::SelfEmployedBenefits);
            }
            if ira_contributor_rows(d).unwrap_or_default().is_empty()
                && (ira_deferral_codes(d) || ira_prior_return(d) || ira_qtp_transfer(d))
            {
                derived.push(TaxSection::IraContributions);
            }
        }
        TaxSection::Credits => {
            let credits = d.credits.as_ref();
            if d.dependents().iter().any(|p| p.care_expenses() > 0)
                || credits.is_some_and(|c| c.care.active() || c.care.has_benefits(d))
                || (joint(d)
                    && d.people.as_ref().is_some_and(|p| {
                        core::iter::once(&p.taxpayer)
                            .chain(p.spouse.iter())
                            .any(|p| p.full_time_student || p.incapable_self_care)
                    }))
            {
                derived.push(TaxSection::Care);
            }
            if credits.is_none_or(|c| c.education.is_empty()) && student_on_return(d) {
                derived.push(TaxSection::Education);
            }
            if credits.is_none_or(|c| c.saver.is_none()) && saver_hint(d) {
                derived.push(TaxSection::Saver);
            }
        }
        _ => {}
    }
    derived.retain(|t| !d.finished_sections.contains(t));
    derived
}
// Automatic steps ask only topic-owned facts. Documents remain reachable in their
// own editor and never change the meaning of a saved category answer.
fn topic_facts_entered(d: &FederalDraft, topic: TaxSection) -> bool {
    match topic {
        topic if INCOME_TOPICS.contains(&topic) => income_topic_entered(d, topic),
        TaxSection::Education => topic_followups_needed(d, topic),
        TaxSection::Hsa => topic_followups_needed(d, topic) || topic_has_data(d, topic),
        TaxSection::Mortgage => false,
        TaxSection::StudentLoan => d.adjustments.as_ref().is_some_and(|a| {
            a.student_loan.student_loan_interest != 0 || a.student_loan.employer_paid_interest != 0
        }),
        TaxSection::Marketplace => {
            d.credits.as_ref().and_then(|c| crate::marketplace::facts(&c.marketplace)).is_some_and(
                |facts| {
                    let mut facts = facts.clone();
                    facts.forms.clear();
                    facts != MarketplaceFacts::default()
                },
            )
        }
        _ => topic_has_data(d, topic),
    }
}
// Whether this return's facts make a topic apply, so the walk opens it once:
// its own records and answers, its own papers, or the questions its
// documents call for on its page (asked or not). A document whose rules call
// for nothing on the topic page leaves the topic off the walk.
fn topic_applies(d: &FederalDraft, topic: TaxSection) -> bool {
    topic_facts_entered(d, topic)
        || topic_followups_present(d, topic)
        || topic_followups_needed(d, topic)
        || match topic {
            // Papers that are the topic's own data, with no question rows.
            TaxSection::StudentLoan => {
                d.adjustments.as_ref().is_some_and(|a| !a.student_loan.forms.is_empty())
            }
            TaxSection::Marketplace => topic_has_data(d, topic),
            _ => false,
        }
}
fn automatic_topic_needed(d: &FederalDraft, topic: TaxSection) -> bool {
    // An explicit record plan still needs its answers when a form already
    // supplies another source in the same category.
    topic_applies(d, topic)
        || missed_pages(d, &d.often_missed).any(|planned| planned == topic)
        || !matches!(topic, TaxSection::Mortgage | TaxSection::Education)
}
// The walk through a hub (M3-P19): its planned, derived or applying topics
// not yet visited, among the ones it offers, in the hub's topic order.
fn walk_topics(d: &FederalDraft, hub: TaxSection, offered: &[i32]) -> Vec<TaxSection> {
    let planned = planned_sections(d);
    let derived = derived_topics(d, hub);
    hub_topics(hub)
        .into_iter()
        .filter(|t| {
            offered.contains(&(*t as i32))
                && (planned.contains(t) || derived.contains(t) || topic_applies(d, *t))
                && automatic_topic_needed(d, *t)
                && !d.finished_sections.contains(t)
        })
        .collect()
}
// The frontier of the sequence passes every section with nothing for this
// return: each is recorded in `finished_sections` and not sent.
// Applied to every accepted change, so the rule is recomputed on each save.
fn skip_empty_sections(d: &mut FederalDraft) {
    let shown = shown_sections(d);
    loop {
        let section = match first_open(d) {
            INCOME_PAGE => TaxSection::Income,
            ADJUSTMENTS_PAGE => TaxSection::Adjustments,
            DEDUCTIONS_PAGE => TaxSection::Deductions,
            ITEMIZED_PAGE => TaxSection::Itemized,
            CREDITS_PAGE => TaxSection::Credits,
            OTHER_TAXES_PAGE => TaxSection::OtherTaxes,
            PAYMENTS_PAGE => TaxSection::Payments,
            _ => return,
        };
        // A hub whose walk has nothing left is passed like an empty section;
        // the deductions page still opens the itemized hub once (next_page).
        if matches!(
            section,
            TaxSection::Income
                | TaxSection::Adjustments
                | TaxSection::Itemized
                | TaxSection::Credits
        ) {
            if Interview::viewing(d.clone()).pending_hub_topic(section).is_some() {
                return;
            }
        } else if shown.contains(&section) {
            return;
        }
        // A return with no payments to enter has none: no estimated or
        // extension payment and the IRS figures any penalty.
        if section == TaxSection::Payments && d.payments.is_none() {
            d.payments = Some(FederalPayments { joint: joint(d), ..FederalPayments::none() });
        }
        finish_section(d, section);
    }
}
// Complete a saved topic that was planned, derived or applying, or whose
// changed answers hold topic-owned facts, matching the automatic walk. An
// unchanged Save of a topic the walk does not hold remains navigation without
// changing the accepted return.
fn visit(before: &FederalDraft, draft: &mut FederalDraft, hub: TaxSection, topic: TaxSection) {
    if (draft != before && (topic_facts_entered(draft, topic) || topic_applies(draft, topic)))
        || (!before.finished_sections.contains(&hub)
            && (planned_sections(before).contains(&topic)
                || derived_topics(before, hub).contains(&topic)))
        // A topic the return's facts walk is visited by its save, also after
        // its hub, so the walk never offers it again.
        || topic_applies(before, topic)
    {
        finish_section(draft, topic);
    }
}
// "You're done with income": a hub's summary once its walk has ended.
fn done_with(section: TaxSection) -> crate::Message {
    let key = match section {
        TaxSection::Income => "PROGRESS_DONE_INCOME",
        TaxSection::Adjustments => "PROGRESS_DONE_ADJUSTMENTS",
        TaxSection::Itemized => "PROGRESS_DONE_ITEMIZED",
        TaxSection::Credits => "PROGRESS_DONE_CREDITS",
        _ => unreachable!("only completed interview hubs have this summary"),
    };
    crate::Message::new(key, alloc::vec![])
}
// A hub's suggestion (M3-P18): the first topic that needs attention, then
// the first topic of its walk (M3-P19: planned or derived, not yet
// visited), then the next section. Only a topic the hub offers is
// suggested.
fn hub_suggestion(
    topics: &[TaxSection],
    offered: &[i32],
    attention: &[ReviewItem],
    walk: &[TaxSection],
    next: TaxSection,
) -> TaxSection {
    topics
        .iter()
        .filter(|t| offered.contains(&(**t as i32)))
        .find(|t| attention.iter().any(|i| i.section == **t))
        .or_else(|| walk.first())
        .copied()
        .unwrap_or(next)
}
// A hub's rows with the checklist: a planned topic with nothing entered
// listed as not started. "What you told us" is the hub's own field (told).
fn with_checklist_rows(
    mut rows: Vec<ReviewItem>,
    topics: &[TaxSection],
    offered: &[i32],
    planned: &[TaxSection],
) -> Vec<ReviewItem> {
    let order =
        |section: TaxSection| topics.iter().position(|t| *t == section).unwrap_or(topics.len());
    for topic in topics {
        if !planned.contains(topic) || !offered.contains(&(*topic as i32)) {
            continue;
        }
        match rows.iter_mut().find(|r| r.section == *topic) {
            Some(row)
                if status_is(&row.status, "REVIEW_NONE")
                    || status_is(&row.status, "REVIEW_NOT_STARTED") =>
            {
                row.status = status("REVIEW_PLANNED_TOPIC");
            }
            Some(_) => {}
            None => {
                let at = rows.iter().take_while(|r| order(r.section) < order(*topic)).count();
                rows.insert(
                    at,
                    ReviewItem {
                        field: None,
                        item: section_message(*topic).encode(),
                        status: status("REVIEW_PLANNED_TOPIC"),
                        section: *topic,
                    },
                );
            }
        }
    }
    rows
}
// A hub's rows with every destination (M3-N1): after the topics with
// entries, attention or a planned tick, every other topic the hub offers
// as "Not started", and the less-common destination, which is always
// listed: "Not started" until the walk has asked it (gate fixes batch A),
// then "None" until something is chosen there. Continue is the hub's action.
// Every row opens its section in one tap.
fn with_every_destination(
    d: &FederalDraft,
    mut rows: Vec<ReviewItem>,
    hub: TaxSection,
    offered: &[i32],
) -> Vec<ReviewItem> {
    let all = hub_topics(hub);
    let Some((more, topics)) = all.split_last() else {
        return rows;
    };
    let at = rows.iter().position(|r| r.section == *more).unwrap_or(rows.len());
    let missing: Vec<ReviewItem> = topics
        .iter()
        .filter(|t| offered.contains(&(**t as i32)) && !rows.iter().any(|r| r.section == **t))
        .map(|t| ReviewItem {
            field: None,
            item: section_message(*t).encode(),
            status: status("REVIEW_NOT_STARTED"),
            section: *t,
        })
        .collect();
    rows.splice(at..at, missing);
    if offered.contains(&(*more as i32)) && !rows.iter().any(|r| r.section == *more) {
        rows.push(ReviewItem {
            field: None,
            item: section_message(*more).encode(),
            status: status(if d.finished_sections.contains(more) {
                "REVIEW_NONE"
            } else {
                "REVIEW_NOT_STARTED"
            }),
            section: *more,
        });
    }
    rows
}
// A hub's topic list, titled by the section's name.
fn hub_list(hub: TaxSection, items: Vec<ReviewItem>) -> HubTopics {
    HubTopics {
        name: status(match hub {
            TaxSection::Income => "SECTION_INCOME",
            TaxSection::Adjustments => "SECTION_ADJUSTMENTS",
            TaxSection::Itemized => "SECTION_ITEMIZED",
            _ => "SECTION_CREDITS",
        }),
        items,
    }
}
// A leading-page link preserves backward navigation after replacing Go to.
fn previous_section_row(page: i32) -> ReviewItem {
    let section = [TaxSection::Documents, TaxSection::IncomeWithoutForm, TaxSection::Checklist]
        .into_iter()
        .chain(WALKED_SECTIONS)
        .find(|section| *section as i32 == page)
        .expect("a hub's previous page belongs to the walkthrough");
    ReviewItem {
        field: None,
        item: status("DISPLAY_PREVIOUS_STEP"),
        status: section_message(section).encode(),
        section,
    }
}
impl Interview {
    // The pages of a return as they would open, for the walk's own checks:
    // nothing is stored and no user is served.
    fn viewing(draft: FederalDraft) -> Self {
        Self { user: 0, draft, dirty: false, accepted_save: false, blocked: false, viewing: true }
    }
    pub(crate) fn new(user: LibertasUser, mut draft: FederalDraft) -> Self {
        let blocked = !draft.valid_stored();
        // Invalid stored data is shown unchanged; it is never repaired here.
        if !blocked {
            fill_unasked(&mut draft);
            crate::injured_spouse_sources::clear_unused(&mut draft);
        }
        Self { user, draft, dirty: false, accepted_save: false, blocked, viewing: false }
    }
    #[cfg(test)]
    pub(crate) fn oracle_flags(&self) -> [bool; 3] {
        [self.dirty, self.blocked, self.viewing]
    }
    // Replay the captured state exactly, without startup normalization.
    #[cfg(test)]
    pub(crate) fn oracle_restore(
        user: LibertasUser,
        draft: FederalDraft,
        flags: [bool; 3],
    ) -> Self {
        Self {
            user,
            draft,
            dirty: flags[0],
            accepted_save: false,
            blocked: flags[1],
            viewing: flags[2],
        }
    }
    // The sections of this return's sequence: the people page counts only
    // when page 1 says you support someone.
    fn progress(&self, page: i32) -> Vec<u8> {
        self.topic_progress(page, None)
    }
    fn topic_progress(&self, page: i32, topic: Option<TaxSection>) -> Vec<u8> {
        if self.viewing {
            return Vec::new();
        }
        progress_text(page, &self.draft, topic)
    }
    // The estimate of the accepted return, shown when a page is built.
    fn estimate_now(&self, section: TaxSection) -> Option<EstimateSoFar> {
        if self.viewing {
            return None;
        }
        Some(self.estimate_view(&self.draft, &estimate(&self.draft), section))
    }
    // The estimate line and what it still needs for `result`, the calculation
    // of `d`. A topic the path walks that has nothing entered yet would be
    // counted as zero, so while one remains the line holds no figure and the
    // topic stays pending. The page shows only facts it can answer; Review
    // retains the complete list from the same producer.
    fn estimate_view(
        &self,
        d: &FederalDraft,
        result: &FederalResult,
        section: TaxSection,
    ) -> EstimateSoFar {
        let ahead: Vec<TaxSection> = [
            TaxSection::Income,
            TaxSection::Adjustments,
            TaxSection::Itemized,
            TaxSection::Credits,
        ]
        .into_iter()
        .filter(|hub| in_walk(d, *hub))
        .flat_map(|hub| {
            let less_common = hub_topics(hub).last().copied();
            walk_topics(d, hub, &self.hub_sections(hub))
                .into_iter()
                .filter(move |t| Some(*t) != less_common && !topic_entered(d, *t))
        })
        .collect();
        let line =
            if ahead.is_empty() { estimate_text(result) } else { status("ESTIMATE_UNAVAILABLE") };
        let mut remaining_items = estimate_items(d, result, section);
        if section != TaxSection::Review {
            let total = remaining_items.len();
            remaining_items.retain(|row| row.section == section);
            for row in &mut remaining_items {
                if !status_is(&row.status, "DOCUMENT_PENDING_FIX")
                    && !status_is(&row.status, "REVIEW_COMING_UP")
                {
                    row.item = row.status.clone();
                }
                row.status = status("DOCUMENT_PENDING_FIX");
            }
            let other = total - remaining_items.len();
            if other != 0 {
                remaining_items.push(ReviewItem {
                    field: None,
                    item: section_message(TaxSection::Review).encode(),
                    status: status_count("ESTIMATE_LATER_ITEMS", other),
                    section: TaxSection::Review,
                });
            }
        }
        EstimateSoFar { line, remaining_items }
    }
    fn estimate_so_far(
        &self,
        d: &FederalDraft,
        result: &FederalResult,
        section: TaxSection,
    ) -> Option<EstimateSoFar> {
        (!self.viewing).then(|| self.estimate_view(d, result, section))
    }
    fn income_result(&self) -> FederalResult {
        let mut draft = self.draft.clone();
        finish_section(&mut draft, TaxSection::Income);
        estimate(&draft)
    }
    // Whether the calculation reads what produces a Schedule C or F
    // business's income: only Form 8615 does (federal_rules::young_8615).
    fn business_source_needed(&self) -> bool {
        self.draft
            .people
            .as_ref()
            .is_some_and(|p| crate::federal_rules::young_8615(self.joint(), p))
    }
    // Whether it reads a partnership's: Form 8615, or the Standard Deduction
    // Worksheet for Dependents (federal_rules::dependent_standard).
    fn partner_source_needed(&self) -> bool {
        self.business_source_needed()
            || self.draft.people.as_ref().is_some_and(crate::federal_rules::dependent_standard)
    }
    // The income-source allowed sets of the income page (T2-cy): Not needed
    // alone when the calculation does not read the answer, otherwise every
    // answer except Not needed. Checked again on save.
    fn allowed_business_source(&self) -> Vec<i32> {
        crate::BusinessIncomeChoice::allowed(self.business_source_needed())
    }
    // The people a savings-bond row can name on the documents page: the
    // education catalog (you, your spouse on a joint return and the people
    // you support) keyed by person identity, so a stored row keeps its person
    // whatever the catalog's order.
    fn income_people(&self) -> Vec<crate::PersonChoice> {
        education_people(&self.draft)
    }
    // The farms and farm rentals a Form 1099-G box 7 payment or a Form
    // 1099-PATR can name, keyed by their income identity (so a stored choice
    // never depends on the catalog's order).
    fn income_farms(&self) -> Vec<crate::RecordChoice> {
        crate::farm::sources(&self.draft)
            .map(|source| crate::RecordChoice {
                id: source.entry.id,
                label: farm_choice_label(&source).encode(),
            })
            .collect()
    }
    // The records a Form 1099 amount can name (unit b9-2), each keyed by its
    // income identity and named as the owner names it, or by its kind and
    // identity.
    fn income_records(
        &self,
        wanted: impl Fn(&FederalIncomeEntry) -> bool,
    ) -> Vec<crate::RecordChoice> {
        self.draft
            .income
            .iter()
            .filter(|entry| wanted(entry))
            .map(|entry| crate::RecordChoice {
                id: entry.id,
                label: income_display_label(entry, &[]).encode(),
            })
            .collect()
    }
    fn income_businesses(&self) -> Vec<crate::RecordChoice> {
        self.income_records(|entry| {
            matches!(&entry.income, FederalIncome::Business { data }
                if matches!(data.activity, BusinessActivity::Ordinary { .. }))
        })
    }
    // The Schedule C and Schedule F businesses an earlier-year installment sale
    // can name for its qualified business income.
    fn income_qbi_businesses(&self) -> Vec<crate::RecordChoice> {
        self.income_records(|entry| {
            matches!(&entry.income, FederalIncome::Business { data }
            if matches!(
                data.activity,
                BusinessActivity::Ordinary { .. } | BusinessActivity::Farming { .. }
            ))
        })
    }
    // The Schedule C businesses run abroad a Form 1116 source row can name.
    fn foreign_business_choices(&self) -> Vec<crate::RecordChoice> {
        self.income_records(|entry| {
            matches!(&entry.income, FederalIncome::Business { data }
                if matches!(data.activity, BusinessActivity::Ordinary { .. })
                    && matches!(data.source, crate::BusinessSource::ForeignServices { .. }))
        })
    }
    fn income_rentals(&self) -> Vec<crate::RecordChoice> {
        self.income_records(|entry| matches!(entry.income, FederalIncome::Rental { .. }))
    }
    fn income_royalty_properties(&self) -> Vec<crate::RecordChoice> {
        self.income_records(|entry| matches!(entry.income, FederalIncome::Royalty { .. }))
    }
    fn income_equipment_rentals(&self) -> Vec<crate::RecordChoice> {
        self.income_records(|entry| {
            matches!(
                entry.income,
                FederalIncome::NonbusinessProperty {
                    kind: NonbusinessPropertyKind::PersonalRental { .. },
                    ..
                }
            )
        })
    }
    fn income_hobbies(&self) -> Vec<crate::RecordChoice> {
        self.income_records(|entry| matches!(entry.income, FederalIncome::HobbySales { .. }))
    }
    // The CCC loans forfeited in 2026 that a Form 1099-A can name, keyed by
    // the loan's own record identity.
    fn income_ccc_lots(&self) -> Vec<crate::RecordChoice> {
        ccc_lots(&self.draft)
    }
    // The Long-term care periods records a Form 1099-LTC can name, keyed by their
    // income identity (so a stored choice never depends on the catalog's order),
    // labelled by the insured person.
    fn ltc_source_choices(&self, annuity: bool) -> Vec<crate::LtcSourceChoice> {
        self.draft
            .income
            .iter()
            .filter_map(|entry| {
                let gross = match &entry.income {
                    FederalIncome::Retirement { data: source }
                        if {
                            let data = &crate::FederalRetirement::from(source);
                            annuity && data.annuity_benefit_source()
                        } =>
                    {
                        let data = &crate::FederalRetirement::from(source);
                        data.gross
                    }
                    FederalIncome::OtherIncome { data }
                        if !annuity
                            && matches!(data.kind, crate::OtherIncomeKind::Recovery { .. })
                            && data.amount > 0 =>
                    {
                        data.amount
                    }
                    _ => return None,
                };
                Some(crate::LtcSourceChoice {
                    id: entry.id,
                    label: income_display_label(entry, &[]).encode(),
                    owner: entry.owner.owned(),
                    gross,
                })
            })
            .collect()
    }
    fn income_ltc_insured(&self) -> Vec<crate::RecordChoice> {
        let people = education_people(&self.draft);
        self.draft
            .income
            .iter()
            .filter_map(|entry| {
                let FederalIncome::LtcInsured { data } = &entry.income else {
                    return None;
                };
                let name = match &data.insured {
                    crate::FederalLtcInsuredPerson::OnReturn { person } => {
                        crate::MessageArg::Encoded(
                            people
                                .iter()
                                .find(|p| p.id == *person)
                                .map(|p| p.label.clone())
                                .unwrap_or_default(),
                        )
                    }
                    crate::FederalLtcInsuredPerson::Other { name } => {
                        crate::MessageArg::Text(name.clone())
                    }
                };
                Some(crate::RecordChoice {
                    id: entry.id,
                    label: crate::Message::new("LTC_INSURED_CHOICE", alloc::vec![name]).encode(),
                })
            })
            .collect()
    }
    fn allowed_farm_source(&self) -> Vec<i32> {
        crate::FarmIncomeSource::allowed(self.business_source_needed())
    }
    fn allowed_partner_source(&self) -> Vec<i32> {
        K1IncomeSource::allowed(self.partner_source_needed())
    }
    // A farm's Schedule F line 34 on the accepted return with this farm in
    // place of the record of the same identity (or added): the row preview.
    fn farm_preview(&self, id: i64, owner: Owner, farm: &crate::Farm) -> Option<i64> {
        let mut draft = self.draft.clone();
        let mut data = crate::farm_page::farm_record(farm, None);
        let target = match draft.income.iter_mut().find(|e| id != 0 && e.id == id) {
            Some(entry) => {
                if let FederalIncome::Business { data: kept } = &entry.income {
                    data.health_insurance = kept.health_insurance.clone();
                    data.retirement = kept.retirement.clone();
                }
                entry.owner = crate::DocumentOwner::from(owner);
                entry.income = FederalIncome::Business { data };
                id
            }
            None => {
                let id = draft.next_id;
                draft.next_id += 1;
                draft.income.push(FederalIncomeEntry {
                    printed: None,
                    refund_management: None,
                    state_source: Default::default(),
                    id,
                    label: crate::farm_page::farm_label(farm),
                    owner: crate::DocumentOwner::from(owner),
                    income: FederalIncome::Business { data },
                    community: CommunityShare::NotNeeded,
                });
                id
            }
        };
        let entry = draft.income.iter().find(|e| e.id == target)?;
        validate_income(entry).ok()?;
        finish_section(&mut draft, TaxSection::Income);
        farm_line_34(&estimate(&draft), target)
    }
    // A farm rental's line 32 or 34c on the accepted return with this rental
    // in place of the record of the same identity (or added): the row preview.
    fn farm_rental_preview(
        &self,
        id: i64,
        owner: Owner,
        rental: &crate::FarmRental,
    ) -> Option<i64> {
        let mut draft = self.draft.clone();
        let data = crate::farm_page::farm_rental_record(rental);
        let target = match draft.income.iter_mut().find(|e| id != 0 && e.id == id) {
            Some(entry) => {
                entry.owner = crate::DocumentOwner::from(owner);
                entry.income = FederalIncome::FarmRental { data };
                id
            }
            None => {
                let id = draft.next_id;
                draft.next_id += 1;
                draft.income.push(FederalIncomeEntry {
                    printed: None,
                    refund_management: None,
                    state_source: Default::default(),
                    id,
                    label: crate::farm_page::farm_rental_label(rental),
                    owner: crate::DocumentOwner::from(owner),
                    income: FederalIncome::FarmRental { data },
                    community: CommunityShare::NotNeeded,
                });
                id
            }
        };
        let entry = draft.income.iter().find(|e| e.id == target)?;
        validate_income(entry).ok()?;
        finish_section(&mut draft, TaxSection::Income);
        farm_rental_line(&estimate(&draft), target)
    }
    // The return's filers: the spouse only on a joint return.
    fn allowed_owners(&self) -> Vec<i32> {
        let mut owners = alloc::vec![Owner::Taxpayer as i32];
        if self.joint() {
            owners.push(Owner::Spouse as i32);
        }
        owners
    }
    // Separate returns also show the spouse's financial accounts; the owner
    // step restricts other kinds to the community-property or joint return.
    pub(crate) fn income_owners(&self) -> Vec<i32> {
        let mut owners = self.allowed_owners();
        if self
            .draft
            .setup
            .as_ref()
            .is_some_and(|setup| setup.filing_status() == FederalStatus::Separate)
            && !owners.contains(&(Owner::Spouse as i32))
        {
            owners.push(Owner::Spouse as i32);
        }
        owners
    }
    fn allowed_sections(&self) -> Vec<i32> {
        // Match editor admission, including cross-section pages once the
        // filers are known. Conditional form prerequisites still apply.
        // Never trust an echoed list.
        let mut sections: Vec<i32> =
            [SETUP_PAGE, PEOPLE_PAGE].into_iter().filter(|page| self.reached(*page)).collect();
        if self.spouse_open() {
            sections.push(SPOUSE_PAGE);
        }
        if in_walk(&self.draft, TaxSection::Children) && self.reached(CHILDREN_PAGE) {
            sections.push(CHILDREN_PAGE);
        }
        if self.reached(CHILDREN_PAGE) {
            sections.push(DOCUMENTS_PAGE);
        }
        if self.reached(INCOME_WITHOUT_FORM_PAGE) {
            sections.push(INCOME_WITHOUT_FORM_PAGE);
        }
        if self.reached(CHECKLIST_PAGE) {
            sections.push(CHECKLIST_PAGE);
        }
        if self.reached(INCOME_PAGE) {
            sections.push(INCOME_PAGE);
            sections.extend(INCOME_TOPICS.iter().map(|topic| *topic as i32));
        }
        if self.reached(ADJUSTMENTS_PAGE) {
            sections.push(ADJUSTMENTS_PAGE);
            sections.extend(offered_topics(&self.draft, &ADJUSTMENT_TOPICS));
            sections.push(MORE_ADJUSTMENTS_PAGE);
        }
        if self.reached(DEDUCTIONS_PAGE) {
            sections.push(DEDUCTIONS_PAGE);
        }
        if in_walk(&self.draft, TaxSection::Itemized) && self.reached(ITEMIZED_PAGE) {
            sections.push(ITEMIZED_PAGE);
            sections.extend(OPEN_ITEMIZED_TOPICS.iter().map(|t| *t as i32));
        }
        if in_walk(&self.draft, TaxSection::OtherItemized)
            && self.reached(DEDUCTIONS_PAGE)
            && !sections.contains(&OTHER_ITEMIZED_PAGE)
        {
            sections.push(OTHER_ITEMIZED_PAGE);
        }
        if mortgage_available(&self.draft)
            && self.reached(MORTGAGE_PAGE)
            && !sections.contains(&MORTGAGE_PAGE)
        {
            sections.push(MORTGAGE_PAGE);
        }
        if self.reached(CREDITS_PAGE) {
            sections.push(CREDITS_PAGE);
            sections.extend(offered_topics(&self.draft, &CREDIT_TOPICS));
            sections.push(MORE_CREDITS_PAGE);
        }
        if self.reached(OTHER_TAXES_PAGE) {
            sections.push(OTHER_TAXES_PAGE);
            if in_walk(&self.draft, TaxSection::ParentTax) {
                sections.push(PARENT_TAX_PAGE);
            }
        }
        if self.reached(PAYMENTS_PAGE) {
            sections.push(PAYMENTS_PAGE);
            if in_walk(&self.draft, TaxSection::IncomeByPeriod) {
                sections.push(INCOME_BY_PERIOD_PAGE);
            }
        }
        if self.reached(OFTEN_MISSED_PAGE) {
            sections.push(OFTEN_MISSED_PAGE);
        }
        sections.push(REVIEW_PAGE);
        if in_walk(&self.draft, TaxSection::InjuredSpouse) {
            sections.push(INJURED_SPOUSE_PAGE);
        }
        sections
    }
    fn navigation_availability(&self, allowed: &[i32]) -> NavigationAvailability {
        let sections: Vec<_> = SEQUENCE
            .iter()
            .copied()
            .filter(|section| allowed.contains(&(*section as i32)))
            .collect();
        NavigationAvailability::from_sections(&sections)
    }
    fn primary_actions(allowed: &[i32]) -> Vec<i32> {
        allowed
            .iter()
            .copied()
            .filter(|action| {
                *action != libertas_macros::variant_index!(P::Navigate) as i32
                    && *action != libertas_macros::variant_index!(P::OpenSection) as i32
                    && *action != libertas_macros::variant_index!(P::ViewReviewedRelatedForm) as i32
                    && *action != libertas_macros::variant_index!(P::OpenCropInsuranceFacts) as i32
            })
            .collect()
    }
    // "Your answers": one row per reachable page in sequence order, each
    // opened by OpenSection with its section, and always one row for the
    // optional situations list. A section passed with nothing in it reads
    // "Nothing entered".
    fn answers(&self, result: &FederalResult, remaining: &[ReviewItem]) -> Vec<ReviewItem> {
        let d = &self.draft;
        let allowed = self.allowed_sections();
        let unanswered = || status("REVIEW_NOT_ANSWERED");
        let reminded = planned_sections(d);
        [
            TaxSection::Setup,
            TaxSection::People,
            TaxSection::Spouse,
            TaxSection::Children,
            TaxSection::Documents,
            TaxSection::IncomeWithoutForm,
            TaxSection::Checklist,
            TaxSection::Income,
            TaxSection::Adjustments,
            TaxSection::Deductions,
            TaxSection::Itemized,
            TaxSection::Credits,
            TaxSection::Marketplace,
            TaxSection::OtherTaxes,
            TaxSection::ParentTax,
            TaxSection::Payments,
            TaxSection::IncomeByPeriod,
            TaxSection::InjuredSpouse,
            TaxSection::OftenMissed,
        ]
        .into_iter()
        .filter(|section| allowed.contains(&(*section as i32)))
        // The Form 1095-A row, like the credits hub's, once a form is entered.
        .filter_map(|section| match section {
            TaxSection::Marketplace => {
                marketplace_status(d, result).map(|status| (section, Some(status)))
            }
            _ => Some((section, None)),
        })
        .map(|(section, marketplace)| {
            let (item, status) = match section {
                TaxSection::Setup => (
                    "SECTION_SETUP",
                    d.setup
                        .as_ref()
                        .map_or_else(unanswered, |s| status(filing_key(s.filing_status()))),
                ),
                TaxSection::People => (
                    "SECTION_PEOPLE",
                    d.people.as_ref().map_or_else(unanswered, |p| p.taxpayer.name()),
                ),
                TaxSection::Spouse => (
                    "SECTION_SPOUSE",
                    d.people
                        .as_ref()
                        .and_then(|p| p.spouse.as_ref())
                        .map_or_else(unanswered, |s| s.name()),
                ),
                TaxSection::Children => (
                    "SECTION_CHILDREN",
                    match d.dependents().len() {
                        _ if !d.finished_sections.contains(&TaxSection::Children) => unanswered(),
                        0 => status("REVIEW_NOTHING_ENTERED"),
                        n => status_count("REVIEW_PEOPLE", n),
                    },
                ),
                TaxSection::Documents => (
                    "SECTION_DOCUMENTS",
                    attention_text(remaining, TaxSection::Documents).unwrap_or_else(|| {
                        if d.finished_sections.contains(&TaxSection::Documents) {
                            status("REVIEW_ENTERED")
                        } else {
                            unanswered()
                        }
                    }),
                ),
                TaxSection::IncomeWithoutForm => {
                    ("SECTION_INCOME_WITHOUT_FORM", without_form_summary(d))
                }
                // The checklist counts its ticks (M3-P18).
                TaxSection::Checklist => (
                    "SECTION_CHECKLIST",
                    match FLAGS.iter().filter(|f| ticked(&checklist_value(d), f)).count() {
                        _ if !d.finished_sections.contains(&TaxSection::Checklist) => unanswered(),
                        0 => status("REVIEW_NONE_OF_THESE"),
                        n => status_count("REVIEW_TOPICS", n),
                    },
                ),
                // A hub summarizes its topics.
                TaxSection::Income => (
                    "SECTION_INCOME",
                    match d.income.len() {
                        _ if !d.finished_sections.contains(&TaxSection::Income) => unanswered(),
                        0 => status("REVIEW_NOTHING_ENTERED"),
                        _ => document_summary(d.income.iter()).encode(),
                    },
                ),
                TaxSection::Adjustments => (
                    "SECTION_ADJUSTMENTS",
                    match adjustments_count(d) {
                        _ if !d.finished_sections.contains(&TaxSection::Adjustments) => {
                            unanswered()
                        }
                        0 => status("REVIEW_NOTHING_ENTERED"),
                        n => status_count("REVIEW_ADJUSTMENTS", n),
                    },
                ),
                TaxSection::Deductions => (
                    "SECTION_DEDUCTIONS",
                    if !d.finished_sections.contains(&TaxSection::Deductions) {
                        unanswered()
                    } else if d.spouse_itemizes() {
                        status("REVIEW_ROUTE_NAME_SPOUSEITEMIZES")
                    } else if d
                        .deductions
                        .as_ref()
                        .is_some_and(|ded| ded.itemize_override.is_some())
                    {
                        status("REVIEW_ROUTE_NAME_ITEMIZE")
                    } else {
                        status("REVIEW_ROUTE_NAME_COMPARE")
                    },
                ),
                TaxSection::Itemized => (
                    "SECTION_ITEMIZED",
                    match itemized_facts(d).map(|facts| {
                        OPEN_ITEMIZED_TOPICS
                            .iter()
                            .filter(|t| itemized_topic_present(d, facts, **t))
                            .count()
                    }) {
                        _ if !d.finished_sections.contains(&TaxSection::Itemized) => unanswered(),
                        Some(0) | None => status("REVIEW_NOTHING_ENTERED"),
                        Some(n) => status_count("REVIEW_TOPICS", n),
                    },
                ),
                TaxSection::Credits => (
                    "SECTION_CREDITS",
                    match credits_count(d) {
                        _ if !d.finished_sections.contains(&TaxSection::Credits) => unanswered(),
                        0 => status("REVIEW_NOTHING_ENTERED"),
                        n => status_count("REVIEW_TOPICS", n),
                    },
                ),
                TaxSection::Marketplace => {
                    ("SECTION_MARKETPLACE", marketplace.unwrap_or_else(|| status("REVIEW_NONE")))
                }
                TaxSection::OtherTaxes => (
                    "SECTION_OTHER_TAXES",
                    match other_tax_items(d).len() {
                        _ if !d.finished_sections.contains(&TaxSection::OtherTaxes) => unanswered(),
                        0 if reminded.contains(&TaxSection::OtherTaxes) => {
                            status("REVIEW_PLANNED_TOPIC")
                        }
                        0 => status("REVIEW_NOTHING_ENTERED"),
                        n => status_count("REVIEW_TAXES", n),
                    },
                ),
                TaxSection::Payments => (
                    "SECTION_PAYMENTS",
                    match d.payments.as_ref().map_or(0, payments_count) {
                        _ if !d.finished_sections.contains(&TaxSection::Payments) => unanswered(),
                        0 if reminded.contains(&TaxSection::Payments) => {
                            status("REVIEW_PLANNED_TOPIC")
                        }
                        0 => status("REVIEW_NOTHING_ENTERED"),
                        n => status_count("REVIEW_PAYMENTS", n),
                    },
                ),
                TaxSection::IncomeByPeriod => (
                    "SECTION_INCOME_BY_PERIOD",
                    match d.payments.as_ref().and_then(|p| p.annualized_income.as_ref()) {
                        None => unanswered(),
                        Some(_) => status("REVIEW_ENTERED"),
                    },
                ),
                TaxSection::InjuredSpouse => (
                    "SECTION_INJURED_SPOUSE",
                    if d.injured_spouse.is_some() {
                        status("REVIEW_ENTERED")
                    } else {
                        unanswered()
                    },
                ),
                TaxSection::OftenMissed => (
                    "SECTION_OFTEN_MISSED",
                    match d.often_missed.chosen() {
                        _ if !d.finished_sections.contains(&TaxSection::OftenMissed) => {
                            unanswered()
                        }
                        0 => status("REVIEW_NONE_OF_THESE"),
                        n => status_count("REVIEW_SITUATIONS", n),
                    },
                ),
                _ => (
                    "SECTION_PARENT_TAX",
                    if parent_missing(d) { unanswered() } else { status("REVIEW_ENTERED") },
                ),
            };
            ReviewItem { field: None, item: libertas_formatted_text(item, &[]), status, section }
        })
        .collect()
    }
    // The review's remaining items for the accepted return.
    fn remaining_items(&self, result: &FederalResult) -> Vec<ReviewItem> {
        remaining_items(&self.draft, result, self.open_section())
    }
    // The estimate's items plus the review's finishing row: the first
    // unfinished page.
    fn review_view(&self, result: &FederalResult) -> EstimateSoFar {
        let mut view = self.estimate_view(&self.draft, result, TaxSection::Review);
        if !self.draft.finished && self.first_open() != REVIEW_PAGE {
            let section = self.open_section();
            view.remaining_items.push(ReviewItem {
                field: None,
                item: section_message(section).encode(),
                status: crate::Message::new(
                    "REVIEW_CONTINUE_FROM",
                    alloc::vec![crate::MessageArg::Nested(alloc::boxed::Box::new(
                        section_message(section),
                    ))],
                )
                .encode(),
                section,
            });
            view.remaining_items.sort_by_key(|item| sequence_position(item.section));
        }
        view
    }
    // The review: remaining items, then "Your answers". It suggests the
    // first item's page (the first unfinished section when nothing
    // remains), and offers Finish only when nothing remains.
    // Resume is offered only when it opens an unfinished interview step.
    fn review(&self) -> Reply {
        self.review_with_result(estimate(&self.draft))
    }
    fn review_with_result(&self, mut result: FederalResult) -> Reply {
        let allowed_sections = self.allowed_sections();
        let related_forms = related_forms::review_forms(&self.draft);
        let EstimateSoFar { line: estimate, remaining_items } = self.review_view(&result);
        let suggested = remaining_items
            .iter()
            .map(|i| i.section)
            .find(|s| allowed_sections.contains(&(*s as i32)))
            .unwrap_or_else(|| self.open_section());
        let mut allowed_actions = alloc::vec![
            libertas_macros::variant_index!(P::Navigate) as i32,
            libertas_macros::variant_index!(P::OpenSection) as i32,
        ];
        if !related_forms.is_empty() {
            allowed_actions
                .push(libertas_macros::variant_index!(P::ViewReviewedRelatedForm) as i32);
        }
        if !self.draft.finished && self.first_open() != REVIEW_PAGE {
            allowed_actions.push(libertas_macros::variant_index!(P::OpenInterview) as i32);
        }
        if result.issues.is_empty() && remaining_items.is_empty() {
            allowed_actions.push(libertas_macros::variant_index!(P::StartFinish) as i32);
        }
        let answers = self.answers(&result, &remaining_items);
        let correction_notes = correction_notes(&self.draft, &mut result);
        result.return_lines = crate::draft_return(&result);
        let navigation = self.navigation_availability(&allowed_sections);
        let primary_actions = Self::primary_actions(&allowed_actions);
        Reply::Response(P::Review {
            navigation,
            suggested_destination: suggested.into(),
            primary_actions,
            go_to_actions: alloc::vec![libertas_macros::variant_index!(P::Navigate) as i32],
            allowed_sections,
            cookie: self.draft.cookie.clone(),
            revision: self.draft.revision,
            estimate,
            remaining_items,
            correction_notes,
            answers,
            related_forms,
            result: result.into(),
            page: REVIEW_PAGE,
            suggested,
            allowed_actions,
        })
    }
    // The first unfinished section, which the review's Go to starts at; the
    // first section once every page of the sequence is answered.
    fn open_section(&self) -> TaxSection {
        if self.spouse_missing() {
            return TaxSection::Spouse;
        }
        match self.first_open() {
            PEOPLE_PAGE | COMMUNITY_PAGE => TaxSection::People,
            CHILDREN_PAGE => TaxSection::Children,
            DOCUMENTS_PAGE => TaxSection::Documents,
            INCOME_WITHOUT_FORM_PAGE => TaxSection::IncomeWithoutForm,
            CHECKLIST_PAGE => TaxSection::Checklist,
            INCOME_PAGE => TaxSection::Income,
            ADJUSTMENTS_PAGE => TaxSection::Adjustments,
            DEDUCTIONS_PAGE | SPOUSE_BOXES_PAGE | SPOUSE_ITEMIZES_PAGE => TaxSection::Deductions,
            ITEMIZED_PAGE => TaxSection::Itemized,
            CREDITS_PAGE => TaxSection::Credits,
            OTHER_TAXES_PAGE => TaxSection::OtherTaxes,
            PARENT_TAX_PAGE => TaxSection::ParentTax,
            PAYMENTS_PAGE => TaxSection::Payments,
            INCOME_BY_PERIOD_PAGE => TaxSection::IncomeByPeriod,
            INJURED_SPOUSE_PAGE => TaxSection::InjuredSpouse,
            OFTEN_MISSED_PAGE => TaxSection::OftenMissed,
            _ => TaxSection::Setup,
        }
    }
    // The page before the income hub in the sequence: the opening
    // checklist (M3-P18).
    fn before_income(&self) -> (i32, TaxSection) {
        (CHECKLIST_PAGE, TaxSection::Checklist)
    }
    // The direct income answers precede the opening checklist.
    fn before_checklist(&self) -> i32 {
        INCOME_WITHOUT_FORM_PAGE
    }
    // The section after the income hub that this return shows (M3-P19):
    // the adjustments hub when it has something, else the next one.
    fn after_income(&self) -> TaxSection {
        next_shown(&self.draft, &shown_sections(&self.draft), TaxSection::Income)
    }
    // The section after the adjustments hub that this return shows.
    fn after_adjustments(&self) -> TaxSection {
        next_shown(&self.draft, &shown_sections(&self.draft), TaxSection::Adjustments)
    }
    // The section after the deductions page and its itemized hub that this
    // return shows (the credits hub, M3-P11, when it has something).
    fn after_deductions(&self) -> TaxSection {
        next_shown(&self.draft, &shown_sections(&self.draft), TaxSection::Itemized)
    }
    // The shown section before the credits hub: the itemized hub when the
    // route itemizes, else the deductions page when shown, and so on.
    fn before_credits(&self) -> i32 {
        previous_shown(&self.draft, &shown_sections(&self.draft), TaxSection::Credits)
    }
    // The section after the credits hub that this return shows (other
    // taxes, M3-P15, then federal payments, M3-P13, when they have
    // something).
    fn after_credits(&self) -> TaxSection {
        next_shown(&self.draft, &shown_sections(&self.draft), TaxSection::Credits)
    }
    // The shown section before `section`, which its Back opens (M3-P19).
    fn previous_shown(&self, section: TaxSection) -> i32 {
        previous_shown(&self.draft, &shown_sections(&self.draft), section)
    }
    // What a hub offers.
    fn hub_sections(&self, hub: TaxSection) -> Vec<i32> {
        match hub {
            TaxSection::Income => self.income_hub_sections(),
            TaxSection::Adjustments => self.adjustments_hub_sections(),
            TaxSection::Itemized => self.itemized_hub_sections(),
            _ => self.credits_hub_sections(),
        }
    }
    // A hub's walk: its planned or derived topics not yet visited.
    fn walk_topics(&self, hub: TaxSection) -> Vec<TaxSection> {
        walk_topics(&self.draft, hub, &self.hub_sections(hub))
    }
    // Opens a page as the sequence reaches it (M3-P19), never one beyond the
    // first unfinished page: a move forward from a page reached by a jump
    // ahead opens the first unfinished page, so no step is skipped. A topic
    // saved from its hub continues the same way, and a page at or before the
    // first unfinished one opens as entered_page opens it.
    fn enter(&self, page: i32) -> Reply {
        let page = self.forward(page);
        if page == SPOUSE_PAGE { self.spouse_page(false) } else { self.page(page, false) }
    }
    fn forward(&self, page: i32) -> i32 {
        if rank(page) > rank(self.first_open()) {
            self.resumed_page()
        } else {
            self.entered_page(page)
        }
    }
    // Planned, derived and refused retained answers use the same topic owner
    // for automatic continuation and for opening the next editor.
    fn pending_hub_topic(&self, hub: TaxSection) -> Option<TaxSection> {
        self.walk_topics(hub).first().copied().or_else(|| {
            let d = &self.draft;
            hub_topics(hub).into_iter().find(|t| {
                in_walk(d, *t)
                    && !d.finished_sections.contains(t)
                    && topic_has_data(d, *t)
                    && page_refuses(self, *t)
            })
        })
    }
    // A hub opens its next required editor. Income, adjustments and credits
    // have no answers of their own, so their completed summaries are available
    // through Review and Go to without adding a step to the forward walk.
    fn entered_page(&self, mut page: i32) -> i32 {
        while let Some(hub) = hub_section(page) {
            if !self.reached(page) || !in_walk(&self.draft, hub) {
                break;
            }
            if let Some(topic) = self.pending_hub_topic(hub) {
                return topic as i32;
            }
            if hub == TaxSection::Itemized {
                break;
            }
            page = next_shown(&self.draft, &shown_sections(&self.draft), hub) as i32;
        }
        page
    }
    // What the income hub offers: the previous section, its topics and the
    // next section ("Continue").
    fn income_hub_sections(&self) -> Vec<i32> {
        let mut sections = alloc::vec![self.before_income().0];
        sections.extend(INCOME_TOPICS.iter().map(|t| *t as i32));
        sections.push(self.after_income() as i32);
        sections
    }
    // The income hub: "What you told us", then one row per topic with
    // documents, attention or a planned tick, each with its count and
    // amount ("1 W-2 · $60,000.00"). It suggests the first topic that needs
    // attention, then the first topic of its walk, then the next section
    // (M3-P18); once the walk has ended it is the section summary, "You're
    // done with income" (M3-P19).
    fn income_hub(&self, review: bool) -> Reply {
        self.income_hub_with_result(review, None)
    }
    fn income_hub_with_result(&self, review: bool, supplied: Option<&FederalResult>) -> Reply {
        let d = &self.draft;
        let owned;
        let result = match supplied {
            Some(result) => result,
            None => {
                owned = self.income_result();
                &owned
            }
        };
        let items = self.remaining_items(result);
        let planned = planned_sections(d);
        let offered = self.income_hub_sections();
        let hub_topics = hub_topics(TaxSection::Income);
        let walk = walk_topics(d, TaxSection::Income, &offered);
        let suggested = hub_suggestion(&hub_topics, &offered, &items, &walk, self.after_income());
        let topics = INCOME_TOPICS
            .iter()
            .filter_map(|topic| {
                let entries: Vec<&FederalIncomeEntry> =
                    d.income.iter().filter(|e| topic_of(d, e) == Some(*topic)).collect();
                let status = if !entries.is_empty() && *topic == TaxSection::Farming {
                    farming_summary(&entries, result).encode()
                } else if !entries.is_empty() {
                    topic_summary(&entries, crate::federal_rules::separate_return(d)).encode()
                } else if *topic == suggested {
                    status("REVIEW_NOT_STARTED")
                } else {
                    return None;
                };
                Some(ReviewItem {
                    field: None,
                    item: section_message(*topic).encode(),
                    status,
                    section: *topic,
                })
            })
            .collect();
        let topics = mark_attention(topics, &INCOME_TOPICS, &items);
        let topics = with_checklist_rows(topics, &hub_topics, &offered, &planned);
        let topics = with_every_destination(d, topics, TaxSection::Income, &offered);
        let done = walk.is_empty().then(|| done_with(TaxSection::Income));
        let allowed_sections = self.allowed_sections();
        let navigation = self.navigation_availability(&allowed_sections);
        let allowed_actions = self.hub_actions(TaxSection::Income);
        let primary_actions = Self::primary_actions(&allowed_actions);
        Reply::Response(P::Hub {
            crop_insurance: crate::farm_insurance_paper::activity_links(d),
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: INCOME_PAGE,
            previous: self.before_income().0,
            review,
            estimate: self.estimate_so_far(d, result, TaxSection::Income),
            progress: progress_line(INCOME_PAGE, &shown_sections(d), done),
            told: told_us(d, TaxSection::Income),
            topics: hub_list(TaxSection::Income, topics),
            previous_section: previous_section_row(self.before_income().0),
            navigation,
            suggested_destination: suggested.into(),
            primary_actions,
            go_to_actions: alloc::vec![libertas_macros::variant_index!(P::Navigate) as i32],
            allowed_actions,
            suggested,
            allowed_sections,
        })
    }
    // A topic page of the income hub, opened with its complete value: the
    // saved documents of the topic's kinds, or an empty list.
    fn topic_page(&self, topic: TaxSection, review: bool) -> Reply {
        self.topic_page_with_result(topic, review, None)
    }
    fn topic_page_with_result(
        &self,
        topic: TaxSection,
        review: bool,
        supplied: Option<&FederalResult>,
    ) -> Reply {
        let owned;
        let result = match supplied {
            Some(result) => result,
            None if self.viewing && topic != TaxSection::Farming => {
                owned = FederalResult::empty(self.draft.revision);
                &owned
            }
            None => {
                owned = self.income_result();
                &owned
            }
        };
        let d = &self.draft;
        let Some(mut allowed_documents) =
            (if topic == TaxSection::Documents { Some(Vec::new()) } else { record_kinds(topic) })
        else {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        };
        // A child's interest and dividends (Form 8814) are offered only with
        // a child the return can report.
        let children = if topic == TaxSection::InterestDividends {
            child_election_choices(d)
        } else {
            Vec::new()
        };
        if children.is_empty() {
            allowed_documents.retain(|kind| {
                *kind != libertas_macros::variant_index!(crate::IncomeSourceKind::Form8814) as i32
            });
        }
        // The spouse's section 179 records are asked on one page, the one that
        // holds the business, and only while the return needs them.
        let spouse_179 = spouse_179_page(d).filter(|page| *page == topic);
        if spouse_179.is_none() {
            allowed_documents.retain(|kind| {
                *kind
                    != libertas_macros::variant_index!(crate::IncomeSourceKind::SpouseSection179)
                        as i32
            });
        }
        // Investment sales offers the section 1231 lookback only for a
        // return with an earlier-year installment sale of business or
        // rental property (T3-6252), or one already holding the lookback.
        if topic == TaxSection::InvestmentSales
            && d.screening.as_ref().is_none_or(|s| {
                s.section1231.is_none()
                    && !s.installment_payments.iter().any(|v| {
                        matches!(v.sale.kind, crate::PropertyCharacter::BusinessProperty { .. })
                    })
                    && !s.related_exchanges.iter().any(|v| {
                        matches!(v.exchange.kind, crate::PropertyCharacter::BusinessProperty { .. })
                    })
            })
        {
            allowed_documents.retain(|kind| {
                *kind
                    != libertas_macros::variant_index!(
                        crate::IncomeSourceKind::BusinessPropertyLosses
                    ) as i32
            });
        }
        // Each topic edits its own half of the per-owner election.
        let farm_se: Vec<Owner> = d
            .screening
            .as_ref()
            .map(|s| s.optional_se.iter().filter(|e| e.farm).map(|e| e.owner).collect())
            .unwrap_or_default();
        let Some(mut documents) = d
            .income
            .iter()
            .filter(|e| source_in_editor(d, e, topic))
            .map(|entry| document_item(entry, &farm_se, crate::federal_rules::separate_return(d)))
            .collect::<Option<Vec<_>>>()
        else {
            return problem_key("PROBLEM_UNDISPLAYABLE_INCOME_RECORD");
        };
        // A return that does not read an income source reopens it as Not
        // needed, the only choice its allowed set offers (T2-cy); the stored
        // answer is not read, and saving writes Not needed's record.
        let (business_needed, partner_needed) =
            (self.business_source_needed(), self.partner_source_needed());
        let macrs = crate::macrs_source::MacrsElectionView::new(d);
        for item in &mut documents {
            if let IncomeSource::Record { value } = &mut item.document {
                let retained =
                    d.income.iter().find(|entry| entry.id == item.id).is_some_and(|entry| {
                        crate::interest_limit::source(&entry.income).is_some()
                    });
                crate::interest_limit::refresh(value, retained);
                crate::farm_rental::refresh_property_questions(value);
                refresh_record_macrs_questions(value, item.owner.owned(), item.id, &macrs);
                refresh_building_recovery_questions(value, &macrs);
                match value {
                    crate::IncomeRecord::Business(page) => {
                        related_forms::fill_activity_page(d, item.id, page)
                    }
                    crate::IncomeRecord::Farm(page) => {
                        related_forms::fill_activity_page(d, item.id, page)
                    }
                    crate::IncomeRecord::FarmRental(page) => {
                        related_forms::fill_activity_page(d, item.id, page)
                    }
                    crate::IncomeRecord::Rental(page) => {
                        related_forms::fill_activity_page(d, item.id, page)
                    }
                    crate::IncomeRecord::HobbySales(page) => {
                        related_forms::fill_activity_page(d, item.id, page)
                    }
                    crate::IncomeRecord::Royalties(page) => {
                        related_forms::fill_activity_page(d, item.id, page)
                    }
                    crate::IncomeRecord::EquipmentRental(page) => {
                        related_forms::fill_activity_page(d, item.id, page)
                    }
                    _ => (),
                }
            }
            match &mut item.document {
                IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(k1) } => {
                    k1.estate_mortgages = self
                        .mortgage_choices()
                        .into_iter()
                        .map(|v| crate::EstateSourceChoice { id: v.id, label: v.label })
                        .collect();
                }
                IncomeSource::Record { value: crate::IncomeRecord::Business(b) }
                    if !business_needed =>
                {
                    b.more.income_source = crate::BusinessIncomeChoice::NotNeeded
                }
                IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) }
                    if !business_needed =>
                {
                    crate::farm_page::farm_source_not_needed(farm)
                }
                IncomeSource::K1 { value: crate::K1Document::K1Partnership(k1) }
                    if !partner_needed =>
                {
                    k1.personal
                        .get_or_insert_with(crate::PartnerPersonalRecords::default)
                        .income_source = K1IncomeSource::NotNeeded
                }
                // On a joint return the other parent is the spouse: the only
                // choice the allowed set offers.
                IncomeSource::Record { value: crate::IncomeRecord::Form8814(child) }
                    if self.joint() =>
                {
                    child.other_parent = crate::OtherParentChoice::NotNeeded
                }
                _ => {}
            }
        }
        // Each farm shows its Schedule F line 34 from the calculation, and
        // the return-level farm kinds are offered only when they can apply
        // (question F7): Schedule J with a farm profit, the loss choice with
        // a farming loss that makes a net operating loss. The sets come from
        // the accepted return; a kind already entered stays offered.
        if topic == TaxSection::Farming {
            let mut profit = false;
            for item in &mut documents {
                if matches!(
                    item.document,
                    IncomeSource::Record { value: crate::IncomeRecord::Farm(_) }
                ) {
                    item.preview = farm_line_34(result, item.id);
                    profit |= item.preview.is_some_and(|amount| amount > 0);
                }
                if matches!(
                    item.document,
                    IncomeSource::Record { value: crate::IncomeRecord::FarmRental(_) }
                ) {
                    item.preview = farm_rental_line(result, item.id);
                }
            }
            let screening = d.screening.as_ref();
            if !profit && screening.is_none_or(|s| s.farm_averaging.is_none()) {
                allowed_documents.retain(|kind| {
                    *kind
                        != libertas_macros::variant_index!(crate::IncomeSourceKind::FarmAveraging)
                            as i32
                });
            }
            if !result.issues.iter().any(|i| i.display.key == "ISSUE_A_FARMING_NOL_REQUIRES_AN_EXPLICIT_VALID_CARRYBACK_WAIVER_OR_THE_SE_96CD586D")
                && screening.is_none_or(|s| s.farm_nol.is_none())
            {
                allowed_documents.retain(|kind| {
                    *kind != libertas_macros::variant_index!(crate::IncomeSourceKind::FarmLoss) as i32
                });
            }
        }
        // Form 8606 and QCD gifts live on Adjustments, not d.income; rebuild
        // their rows for the Retirement topic from there.
        if topic == TaxSection::Retirement
            && let Some(adjustments) = d.adjustments.as_ref()
        {
            for basis in &adjustments.ira_basis {
                documents.push(IncomeDocumentItem {
                    refund_management: None,
                    refund_management_needed: None,
                    allowed_community: Vec::new(),
                    investment_caps: None,
                    id: 0,
                    owner: crate::DocumentOwner::from(basis.owner),
                    document: IncomeSource::Record {
                        value: crate::IncomeRecord::Form8606(Form8606 {
                            basis: basis.prior,
                            adjustment: basis.adjustment,
                            year_end: basis.year_end,
                            reservist_repayments: basis.reservist_repayments.clone(),
                        }),
                    },
                    preview: None,
                    community: fixed_community(d),
                });
            }
            for charity in &adjustments.qualified_charity {
                for payment in &charity.payments {
                    documents.push(IncomeDocumentItem {
                        refund_management: None,
                        refund_management_needed: None,
                        allowed_community: Vec::new(),
                        investment_caps: None,
                        id: 0,
                        owner: crate::DocumentOwner::from(charity.owner),
                        document: IncomeSource::Record {
                            value: crate::IncomeRecord::Qcd(QcdGift {
                                date: payment.date,
                                amount: payment.amount,
                                inherited: payment.inherited,
                                split_interest: payment.split_interest,
                            }),
                        },
                        preview: None,
                        community: fixed_community(d),
                    });
                }
            }
            // The lump-sum election and the missed required distributions
            // (u5-adjust).
            if let Some(v) = crate::LumpSumElection::from_draft(d) {
                documents.push(IncomeDocumentItem {
                    refund_management: None,
                    refund_management_needed: None,
                    allowed_community: Vec::new(),
                    investment_caps: None,
                    id: 0,
                    owner: crate::DocumentOwner::Yours,
                    document: IncomeSource::Record {
                        value: crate::IncomeRecord::SocialLumpSums(v),
                    },
                    preview: None,
                    community: fixed_community(d),
                });
            }
            for v in &adjustments.required_distributions {
                documents.push(IncomeDocumentItem {
                    refund_management: None,
                    refund_management_needed: None,
                    allowed_community: Vec::new(),
                    investment_caps: None,
                    id: 0,
                    owner: crate::DocumentOwner::from(v.owner),
                    document: IncomeSource::Record {
                        value: crate::IncomeRecord::MissedRmd(
                            crate::MissedDistribution::from_record(v),
                        ),
                    },
                    preview: None,
                    community: fixed_community(d),
                });
            }
            // Each owner's Roth IRA history (u6-roth).
            for v in &adjustments.roth_history {
                documents.push(IncomeDocumentItem {
                    refund_management: None,
                    refund_management_needed: None,
                    allowed_community: Vec::new(),
                    investment_caps: None,
                    id: 0,
                    owner: crate::DocumentOwner::from(v.owner),
                    document: IncomeSource::Record {
                        value: crate::IncomeRecord::RothHistory(
                            crate::RothIraHistory::from_record(v),
                        ),
                    },
                    preview: None,
                    community: fixed_community(d),
                });
            }
        }
        let work_claims = crate::related_forms::effective_claimants(d);
        for item in &mut documents {
            let IncomeSource::Record { value: crate::IncomeRecord::Business(page) } =
                &mut item.document
            else {
                continue;
            };
            let (tips, overtime) =
                crate::information_returns::printed_work_amounts(d, item.id, &work_claims);
            page.more.tips_amount_kinds = alloc::vec![if tips.is_some() { 0 } else { 1 }];
            page.more.overtime_amount_kinds = alloc::vec![if overtime.is_some() { 0 } else { 1 }];
            if let Some(amount) = tips {
                page.more.tips = crate::BusinessTipsEntry::Printed {
                    data: crate::BusinessTipsPage {
                        amount: crate::BusinessTipsAmount::Printed { amount },
                        eligible: page.more.tips.clone().into_record().map(|facts| facts.eligible),
                    },
                };
            }
            if let Some(amount) = overtime {
                page.more.overtime = crate::BusinessOvertimeEntry::Printed {
                    data: crate::BusinessOvertimePage {
                        amount: crate::BusinessOvertimeAmount::Printed { amount },
                        eligible: page
                            .more
                            .overtime
                            .clone()
                            .into_record()
                            .map(|facts| facts.eligible),
                    },
                };
            }
        }
        // The data answers (M3-P15) reopen after the documents.
        let Some(mut answers) = data_answer_items(d, topic) else {
            return problem_key("PROBLEM_UNDISPLAYABLE_ANSWER");
        };
        if matches!(
            topic,
            TaxSection::SelfEmployment
                | TaxSection::Rentals
                | TaxSection::Farming
                | TaxSection::InvestmentSales
        ) && let Some(lookback) = d.screening.as_ref().and_then(|s| s.section1231.as_ref())
        {
            answers.push(IncomeDocumentItem {
                refund_management: None,
                refund_management_needed: None,
                allowed_community: Vec::new(),
                investment_caps: None,
                id: 0,
                owner: crate::DocumentOwner::Yours,
                document: IncomeSource::Record {
                    value: crate::IncomeRecord::BusinessPropertyLosses(
                        crate::BusinessPropertyLosses::from_history(lookback),
                    ),
                },
                preview: None,
                community: fixed_community(d),
            });
        }
        // Only saved records come back as the record: the person adds it,
        // starting unanswered, and records not given are never shown as one.
        if spouse_179.is_some()
            && let Some(crate::FilingChoice::Separate {
                section179: crate::Section179Spouse::Records { cost, allocation },
            }) = d.setup.as_ref().map(|setup| setup.status)
        {
            answers.push(IncomeDocumentItem {
                refund_management: None,
                refund_management_needed: None,
                allowed_community: Vec::new(),
                investment_caps: None,
                id: 0,
                owner: crate::DocumentOwner::Yours,
                document: IncomeSource::Record {
                    value: crate::IncomeRecord::SpouseSection179(crate::SpouseSection179 {
                        cost,
                        allocation,
                    }),
                },
                preview: None,
                community: fixed_community(d),
            });
        }
        documents.extend(answers);
        let retained_available = crate::retained_research::available(d);
        for item in &mut documents {
            if let IncomeSource::Record {
                value: crate::IncomeRecord::CapitalLossCarryover(capital),
            } = &mut item.document
            {
                let stored: crate::PriorCapitalReturn = capital.clone().into();
                let caps = crate::extended_rules::capital_caps(&stored);
                capital.allowed_research_parts = crate::retained_research::choices(
                    if capital.ordinary_history { caps.short + caps.long } else { 0 },
                    &retained_available,
                );
            }
            if let IncomeSource::Record { value: crate::IncomeRecord::NetOperatingLoss(loss) } =
                &mut item.document
            {
                for year in &mut loss.regular {
                    year.allowed_research_parts =
                        crate::retained_research::choices(year.balance, &retained_available);
                }
            }
        }
        // A community-property return shows this return's share of each amount.
        for (item, entry) in
            documents.iter_mut().zip(d.income.iter().filter(|e| source_in_editor(d, e, topic)))
        {
            if matches!(&entry.income, FederalIncome::Rental { data } if data.buildings.iter().any(|cost| cost.energy.is_some()))
            {
                item.preview = result
                    .lines
                    .iter()
                    .find(|line| line.id == alloc::format!("rental_{}_line21_income", entry.id))
                    .map(|line| line.amount);
            } else {
                item.preview = community_amount(d, entry, item.preview);
            }
        }
        for item in &mut documents {
            if !crate::injured_spouse_sources::requested(d) {
                crate::injured_spouse_sources::clear_document(&mut item.document);
            }
            item.refund_management_needed = document_refund_management_needed(d, result, item);
            item.allowed_community = document_community_answers(d, &item.document);
            // A row still unanswered (not needed before the return became a
            // community one) must show a value inside its own set; Save still
            // refuses it, so the person has to choose.
            let stored = crate::community::index(item.community);
            if !item.allowed_community.contains(&stored) {
                item.allowed_community.push(stored);
            }
        }
        Reply::Edit(P::SaveIncomes {
            initial_intangible_predecessors: Vec::new(),
            initial_intangible_choices: Vec::new(),
            initial_asset_amt_carry_choices: Vec::new(),
            initial_interest_limit_choices: Vec::new(),
            initial_rental_recovery_choices: Vec::new(),
            qbi_sources: crate::qbi_aggregation::source_views(d, d).into_iter().map(|v|v.member).collect(),
            initial_carbon_oxide_origins: Vec::new(),
            initial_renewable_facility_origins: Vec::new(),
            empty_k1_activity_choices: alloc::boxed::Box::default(),
            allowed_injured_source_facts: crate::injured_spouse_sources::allowed(d),
            allowed_injured_retained_owners: crate::injured_spouse_sources::retained_choices(d),
            ltc_payee_sources: d.income.iter().filter(|e| matches!(&e.income, FederalIncome::LtcBenefits { data } if data.record().periodic() > 0)).map(|e| RecordChoice { id: e.id, label: income_display_label(e, d.dependents()).encode() }).collect(),
            wage_scorporations: self.wage_scorporations(),
            allowed_wage_scorporations: self.allowed_wage_scorporations(),
            initial_k1_investment_choices: alloc::vec![0, 1],
            research_businesses: crate::business_credit_page::research_businesses(d),
            retained_research_choices: crate::retained_research::available(d),
            k1_credit_lines: crate::business_credit_page::k1_credit_lines(),
            research_credit_lines: crate::business_credit_page::research_credit_lines(),
            renewable_credit_lines: crate::business_credit_page::renewable_credit_lines(),
            transferable_credit_lines: crate::business_credit_page::transferable_credit_lines(),

            user: self.user,
            allowed_continuing_risk: crate::ScorpRiskStart::continuing_choices(),
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: INCOME_PAGE,
            previous: INCOME_PAGE,
            review,
            estimate: self.estimate_so_far(d, result, topic),
            warnings: Vec::new(),
            progress: Some(self.topic_progress(INCOME_PAGE, Some(topic))),
            topic,
            source_choices: {
                let mut choices = crate::IncomeSourceChoices::from_kinds(&allowed_documents);
                choices.printed_lump_years = crate::social::printed_year_pages(d);
                choices
            },
            allowed_documents,
            allowed_owners: self.income_owners(),
            allowed_community: crate::injured_spouse::community_answers(d),
            allowed_refund_management: crate::injured_spouse_payments::allowed_document_management(
                crate::injured_spouse::texas(d),
            ),
            people: self.income_people(),
            farms: self.income_farms(),
            cooperative_activities: self.cooperative_activities(),
            cooperative_properties: crate::farm_specials::basis_choices(d)
                .into_iter()
                .map(|choice| crate::RecordChoice {
                    id: choice.id,
                    label: choice.label,
                })
                .chain(crate::cooperative_basis::choices(d))
                .collect(),
            businesses: self.income_businesses(),
            qbi_businesses: self.income_qbi_businesses(),
            mortgage_loans: self.mortgage_choices(),
            rental_energy_buildings: self.rental_energy_buildings(),
            casualty_properties: self.rental_casualty_properties(),
            rentals: self.income_rentals(),
            royalty_properties: self.income_royalty_properties(),
            equipment_rentals: self.income_equipment_rentals(),
            hobbies: self.income_hobbies(),
            ccc_lots: self.income_ccc_lots(),
            farm_codes: if topic == TaxSection::Farming {
                crate::farm_specials::code_choices()
            } else {
                Vec::new()
            },
            farm_limitations: if topic == TaxSection::Farming {
                crate::farm_specials::limitation_choices()
            } else {
                Vec::new()
            },
            farm_sources: if topic == TaxSection::Farming {
                crate::farm_specials::source_choices(d)
            } else {
                Vec::new()
            },
            basis_assets: if topic == TaxSection::Farming {
                crate::farm_specials::basis_choices(d)
            } else {
                Vec::new()
            },
            farm_method_changes: if topic == TaxSection::Farming {
                crate::farm_specials::method_choices(d)
            } else {
                Vec::new()
            },
            inherited_iras: self.income_inherited_iras(),
            inherited_roth_iras: self.income_inherited_roth_iras(),
            ltc_insured: self.income_ltc_insured(),
            annuity_sources: self.ltc_source_choices(true),
            recovery_sources: self.ltc_source_choices(false),
            allowed_business_source: self.allowed_business_source(),
            allowed_farm_source: self.allowed_farm_source(),
            allowed_partner_source: self.allowed_partner_source(),
            allowed_home_spouse: crate::HomeSpouse::allowed(self.joint()),
            allowed_other_parent: crate::OtherParentChoice::allowed(!self.joint()),
            children,
            document_facts: DocumentFacts { year: 2026 },
            source_documents: self.topic_document_views(topic),
            followups: self.document_followups(topic, result),
            // Regrouped for the person at the protocol boundary (followup_page).
            questions: Vec::new(),
            situations: DocumentSituations::NoneOfThese,
            allowed_situations: Vec::new(),
            situation_answers: Vec::new(),
            situation_documents: Vec::new(),
            disaster_choices: Vec::new(),
            documents: IncomeRecords {
                qbi_aggregations: crate::qbi_aggregation::project_pages(
                    d.screening.as_ref().map_or(&[], |v|v.qbi_aggregations.as_slice()),
                    &crate::qbi_aggregation::source_views(d, d),
                ),
                documents: documents.into_iter().filter_map(IncomeRecordItem::from_income).collect(),
            },

            foreign_countries: if topic == TaxSection::InterestDividends {
                crate::foreign_accounts::countries()
            } else {
                Vec::new()
            },
            interest_statements: if topic == TaxSection::InterestDividends {
                crate::savings_bonds::source_choices(&self.draft.income)
            } else {
                Vec::new()
            },
        })
    }
    // Actions owns Continue or Review; Go to owns Navigate. Bound row links
    // share the response ceiling without appearing in either control.
    fn hub_actions(&self, section: TaxSection) -> Vec<i32> {
        let action = if self.first_open() == REVIEW_PAGE {
            libertas_macros::variant_index!(P::ReviewAccepted)
        } else {
            match section {
                TaxSection::Income => libertas_macros::variant_index!(P::ContinueIncome),
                TaxSection::Adjustments => libertas_macros::variant_index!(P::ContinueAdjustments),
                TaxSection::Itemized => libertas_macros::variant_index!(P::ContinueItemized),
                TaxSection::Credits => libertas_macros::variant_index!(P::ContinueCredits),
                _ => unreachable!("only section hubs have continuation actions"),
            }
        };
        let mut actions = alloc::vec![
            action as i32,
            libertas_macros::variant_index!(P::Navigate) as i32,
            libertas_macros::variant_index!(P::OpenSection) as i32,
        ];
        if section == TaxSection::Income
            && !crate::farm_insurance_paper::activity_links(&self.draft).is_empty()
        {
            actions.push(libertas_macros::variant_index!(P::OpenCropInsuranceFacts) as i32);
        }
        actions
    }
    // Each request names its own section. It advances accepted answers only:
    // editor input is submitted exclusively by that editor's Next request.
    fn continue_section(&mut self, section: TaxSection) -> Reply {
        if self.first_open() == REVIEW_PAGE
            || !self.reached(section as i32)
            || !in_walk(&self.draft, section)
        {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_HERE");
        }
        let next = match section {
            TaxSection::Income => self.after_income(),
            TaxSection::Adjustments => self.after_adjustments(),
            TaxSection::Itemized => self.after_deductions(),
            TaxSection::Credits => self.after_credits(),
            _ => return problem_key("PROBLEM_SECTION_UNAVAILABLE_HERE"),
        };
        let mut draft = self.draft.clone();
        finish_section(&mut draft, section);
        if let Err(message) = self.commit(draft) {
            return problem(message);
        }
        self.enter(next as i32)
    }
    // Go to and bound links open an eligible page without accepting answers
    // or completing a hub.
    fn navigate(&mut self, from: i32, section: TaxSection) -> Reply {
        let to = section as i32;
        let allowed = self.allowed_sections();
        if !allowed.contains(&from) {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_HERE");
        }
        if !allowed.contains(&to) {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        }
        // A hub's "What you told us" row opens the checklist, whose Back and
        // Next return to the hub (M3-P18).
        if to == CHECKLIST_PAGE && HUB_PAGES.contains(&from) {
            return self.checklist_page(from, false);
        }
        // A section whose own step still waits for its answer opens at that
        // step, the page its estimate issue names; Back leads to the section's
        // page.
        let to = match to {
            PEOPLE_PAGE if community_missing(&self.draft) => COMMUNITY_PAGE,
            DEDUCTIONS_PAGE if spouse_itemizes_missing(&self.draft) => SPOUSE_ITEMIZES_PAGE,
            DEDUCTIONS_PAGE if spouse_boxes_missing(&self.draft) => SPOUSE_BOXES_PAGE,
            _ => to,
        };
        let mut reply = self.page(to, from == REVIEW_PAGE);
        // The credits hub's household row opens Children and people you support,
        // whose Back and Next return to the hub.
        if from == CREDITS_PAGE
            && let Reply::Response(P::BeginChildren { previous, .. })
            | Reply::Edit(P::SaveChildren { previous, .. }) = &mut reply
        {
            *previous = CREDITS_PAGE;
        }
        // The itemized hub's gambling losses row opens the income topic
        // (C7), whose Back and save return to the hub.
        if from == ITEMIZED_PAGE
            && let Reply::Edit(P::SaveIncomes { previous, .. }) = &mut reply
        {
            *previous = ITEMIZED_PAGE;
        }
        reply
    }
    fn page(&self, page: i32, review: bool) -> Reply {
        self.page_with_income_result(page, review, None)
    }
    fn page_with_income_result(
        &self,
        page: i32,
        review: bool,
        supplied: Option<&FederalResult>,
    ) -> Reply {
        // A page outside this return's walk does not open.
        if SEQUENCE.iter().any(|s| *s as i32 == page && !in_walk(&self.draft, *s)) {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        }
        let available = match page {
            DOCUMENTS_PAGE => self.reached(CHILDREN_PAGE),
            SETUP_PAGE | PEOPLE_PAGE | INCOME_PAGE | CHECKLIST_PAGE | INCOME_WITHOUT_FORM_PAGE => {
                self.reached(page)
            }
            WAGES_PAGE
            | INTEREST_PAGE
            | INVESTMENT_SALES_PAGE
            | RETIREMENT_PAGE
            | SELF_EMPLOYMENT_PAGE
            | RENTALS_PAGE
            | GOVERNMENT_BENEFITS_PAGE
            | OTHER_INCOME_PAGE
            | LONG_TERM_CARE_BENEFITS_PAGE
            | GAMBLING_PAGE
            | FARMING_PAGE
            | PASS_THROUGH_PAGE
            | CANCELED_DEBT_PAGE
            | WORK_ABROAD_PAGE => self.reached(INCOME_PAGE),
            ADJUSTMENTS_PAGE
            | HSA_PAGE
            | SELF_EMPLOYED_BENEFITS_PAGE
            | IRA_CONTRIBUTIONS_PAGE
            | EDUCATOR_PAGE
            | STUDENT_LOAN_PAGE
            | EMPLOYEE_EXPENSES_PAGE
            | MORE_ADJUSTMENTS_PAGE => self.reached(ADJUSTMENTS_PAGE),
            DEDUCTIONS_PAGE => self.reached(DEDUCTIONS_PAGE),
            // The itemized hub and its topics exist only for a route that
            // itemizes (in_walk, above).
            HOME_OFFICES_PAGE => self.reached(INCOME_PAGE),
            MORTGAGE_PAGE => mortgage_available(&self.draft) && self.reached(MORTGAGE_PAGE),
            ITEMIZED_PAGE | MEDICAL_PAGE | TAXES_PAGE | GIFT_CARRYOVERS_PAGE
            | OTHER_ITEMIZED_PAGE => self.reached(ITEMIZED_PAGE),
            CREDITS_PAGE | CARE_PAGE | EDUCATION_PAGE | SAVER_PAGE | MARKETPLACE_PAGE
            | MORE_CREDITS_PAGE => self.reached(CREDITS_PAGE),
            OTHER_TAXES_PAGE => self.reached(OTHER_TAXES_PAGE),
            // Only while Form 8615 applies (in_walk, above).
            PARENT_TAX_PAGE => self.reached(OTHER_TAXES_PAGE),
            PAYMENTS_PAGE => self.reached(PAYMENTS_PAGE),
            // Only after box C is chosen on the payments page (in_walk).
            INCOME_BY_PERIOD_PAGE => self.reached(PAYMENTS_PAGE),
            SPOUSE_PAGE => self.spouse_open(),
            COMMUNITY_PAGE => separate(&self.draft) && self.draft.people.is_some(),
            SPOUSE_ITEMIZES_PAGE => separate(&self.draft) && self.reached(DEDUCTIONS_PAGE),
            SPOUSE_BOXES_PAGE => {
                crate::federal_rules::spouse_boxes_apply(&self.draft)
                    && self.draft.finished_sections.contains(&TaxSection::Deductions)
            }
            // After the filer pages, and only for a return that supports
            // someone (in_walk).
            CHILDREN_PAGE => self.reached(page),
            REVIEW_PAGE => true,
            // Only while the injured spouse item is ticked on a joint return
            // (in_walk).
            INJURED_SPOUSE_PAGE => true,
            OFTEN_MISSED_PAGE => self.reached(page),
            // Sections of later milestones keep their "nothing applies"
            // answers and are not opened.
            _ => return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET"),
        };
        if !available {
            return problem_key("PROBLEM_EARLIER_SECTION_REQUIRED");
        }
        let d = &self.draft;
        let cookie = d.cookie.clone();
        let revision = d.revision;
        // Review entry does not change the page order. Only the first page
        // has no predecessor and retains its return-to-review destination.
        let previous = match page {
            SETUP_PAGE if review => REVIEW_PAGE,
            SETUP_PAGE | PEOPLE_PAGE => SETUP_PAGE,
            CHILDREN_PAGE if self.joint() => SPOUSE_PAGE,
            CHILDREN_PAGE if separate(d) => COMMUNITY_PAGE,
            CHILDREN_PAGE => PEOPLE_PAGE,
            _ => REVIEW_PAGE,
        };
        let progress = self.progress(page);
        let estimate = if matches!(page, SETUP_PAGE | PEOPLE_PAGE | CHILDREN_PAGE) {
            self.estimate_now(match page {
                SETUP_PAGE => TaxSection::Setup,
                PEOPLE_PAGE => TaxSection::People,
                _ => TaxSection::Children,
            })
        } else {
            None
        };
        match page {
            DOCUMENTS_PAGE => self.document_center_page(review),
            INCOME_WITHOUT_FORM_PAGE => self.income_without_form_page(review),
            SETUP_PAGE => match &d.setup {
                Some(value) => Reply::Edit(P::SaveSetup {
                    user: self.user,
                    cookie,
                    revision,
                    page,
                    previous,
                    review,
                    estimate,
                    warnings: Vec::new(),
                    progress,
                    value: return_setup(value),
                }),
                None => Reply::Response(P::BeginSetup {
                    cookie,
                    revision,
                    page,
                    previous,
                    review,
                    estimate: estimate.map(Into::into),
                    progress,
                }),
            },
            PEOPLE_PAGE => match &d.people {
                Some(value) => Reply::Edit(P::SavePeople {
                    professional_election: professional_election(d, Owner::Taxpayer),
                    allowed_professional_elections: d.professional_elections(Owner::Taxpayer),
                    user: self.user,
                    cookie,
                    revision,
                    page,
                    previous,
                    review,
                    estimate,
                    warnings: Vec::new(),
                    progress,
                    you: professional_filer(d, Owner::Taxpayer, &value.taxpayer),
                }),
                None => Reply::Response(P::BeginPeople {
                    professional_election: professional_election(d, Owner::Taxpayer),
                    allowed_professional_elections: d.professional_elections(Owner::Taxpayer),
                    cookie,
                    revision,
                    page,
                    previous,
                    review,
                    estimate: estimate.map(Into::into),
                    progress,
                }),
            },
            SPOUSE_PAGE => self.spouse_page(review),
            COMMUNITY_PAGE => self.community_page(review),
            SPOUSE_BOXES_PAGE => self.spouse_boxes_page(review),
            SPOUSE_ITEMIZES_PAGE => self.spouse_itemizes_page(review),
            CHECKLIST_PAGE => self.checklist_page(self.before_checklist(), review),
            CHILDREN_PAGE => {
                let alive = crate::federal_rules::short_year_days(d);
                let allowed_lived = allowed_lived(alive);
                let days_alive = alive.unwrap_or(365);
                let Some(people) = d
                    .dependents()
                    .iter()
                    .map(|record| dependent_item(record, alive))
                    .collect::<Option<Vec<_>>>()
                else {
                    return problem_key("PROBLEM_UNDISPLAYABLE_PERSON_RECORD");
                };
                if people.is_empty() {
                    Reply::Response(P::BeginChildren {
                        cookie,
                        revision,
                        page,
                        previous,
                        review,
                        estimate: estimate.map(Into::into),
                        progress,
                        allowed_lived,
                        days_alive,
                    })
                } else {
                    Reply::Edit(P::SaveChildren {
                        user: self.user,
                        cookie,
                        revision,
                        page,
                        previous,
                        review,
                        estimate,
                        warnings: Vec::new(),
                        progress: Some(progress),
                        allowed_lived,
                        days_alive,
                        dependents: Dependents { people },
                        credits: person_credits(&self.draft),
                    })
                }
            }
            INCOME_PAGE => {
                if supplied.is_some() {
                    self.income_hub_with_result(review, supplied)
                } else {
                    self.income_hub(review)
                }
            }
            WAGES_PAGE => self.wages_answers_page(review, supplied),
            INTEREST_PAGE => {
                self.topic_page_with_result(TaxSection::InterestDividends, review, supplied)
            }
            INVESTMENT_SALES_PAGE => {
                self.topic_page_with_result(TaxSection::InvestmentSales, review, supplied)
            }
            RETIREMENT_PAGE => {
                self.topic_page_with_result(TaxSection::Retirement, review, supplied)
            }
            SELF_EMPLOYMENT_PAGE => {
                self.topic_page_with_result(TaxSection::SelfEmployment, review, supplied)
            }
            RENTALS_PAGE => self.topic_page_with_result(TaxSection::Rentals, review, supplied),
            GOVERNMENT_BENEFITS_PAGE => {
                self.topic_page_with_result(TaxSection::GovernmentBenefits, review, supplied)
            }
            OTHER_INCOME_PAGE => {
                self.topic_page_with_result(TaxSection::OtherIncome, review, supplied)
            }
            LONG_TERM_CARE_BENEFITS_PAGE => {
                self.topic_page_with_result(TaxSection::LongTermCareBenefits, review, supplied)
            }
            GAMBLING_PAGE => self.topic_page_with_result(TaxSection::Gambling, review, supplied),
            FARMING_PAGE => self.topic_page_with_result(TaxSection::Farming, review, supplied),
            PASS_THROUGH_PAGE => {
                self.topic_page_with_result(TaxSection::PassThrough, review, supplied)
            }
            CANCELED_DEBT_PAGE => {
                self.topic_page_with_result(TaxSection::CanceledDebt, review, supplied)
            }
            WORK_ABROAD_PAGE => {
                self.topic_page_with_result(TaxSection::WorkAbroad, review, supplied)
            }
            ADJUSTMENTS_PAGE => self.adjustments_hub(review),
            CARE_PAGE => self.care_page(review),
            HSA_PAGE => self.hsa_page(review),
            SELF_EMPLOYED_BENEFITS_PAGE => self.self_employed_benefits_page(review),
            IRA_CONTRIBUTIONS_PAGE => self.ira_contributions_page(review),
            EDUCATOR_PAGE => self.educator_page(review),
            STUDENT_LOAN_PAGE => self.student_loan_page(review),
            EMPLOYEE_EXPENSES_PAGE => self.employee_expenses_page(review),
            DEDUCTIONS_PAGE => self.deductions_page(review),
            ITEMIZED_PAGE => self.itemized_hub(review),
            MEDICAL_PAGE => self.medical_page(review),
            TAXES_PAGE => self.taxes_page(review),
            MORTGAGE_PAGE => self.mortgage_page(review),
            HOME_OFFICES_PAGE => self.home_offices_page(review),
            GIFT_CARRYOVERS_PAGE => self.gift_carryovers_page(review),
            OTHER_ITEMIZED_PAGE => self.other_itemized_page(review),
            CREDITS_PAGE => self.credits_hub(review),
            EDUCATION_PAGE => self.education_page(review),
            SAVER_PAGE => self.saver_page(review),
            MARKETPLACE_PAGE => self.marketplace_page(review),
            MORE_CREDITS_PAGE => self.more_credits_page(review),
            PAYMENTS_PAGE => self.payments_page(review),
            INCOME_BY_PERIOD_PAGE => self.income_by_period_page(review),
            MORE_ADJUSTMENTS_PAGE => self.more_adjustments_page(review),
            OTHER_TAXES_PAGE => self.other_taxes_page(review),
            INJURED_SPOUSE_PAGE => self.injured_spouse_page(review),
            PARENT_TAX_PAGE => self.parent_tax_page(review),
            OFTEN_MISSED_PAGE => self.often_missed_page(review),
            _ => self.review(),
        }
    }
    fn income_save_reply(
        &self,
        prepared: PreparedIncomeCommit,
        previous: i32,
        review: bool,
        source: Option<TaxSection>,
    ) -> Reply {
        let warnings = save_warnings(&prepared.pruned);
        if source == Some(TaxSection::Documents) && !warnings.is_empty() {
            let mut reply = self.document_center_page(review);
            if let Reply::Edit(P::SaveDocuments { warnings: shown, .. }) = &mut reply {
                *shown = warnings;
            }
            return reply;
        }
        if source == Some(TaxSection::Documents) && !review {
            return self.next_page(DOCUMENTS_PAGE);
        }
        if previous == ITEMIZED_PAGE && !review {
            return if itemizing(&self.draft) && self.reached(ITEMIZED_PAGE) {
                self.itemized_hub(false)
            } else {
                self.page_with_income_result(
                    self.forward(INCOME_PAGE),
                    false,
                    Some(&prepared.result),
                )
            };
        }
        if review && !prepared.changed {
            return if prepared.income_finished {
                self.review_with_result(*prepared.result)
            } else {
                self.review()
            };
        }
        let page = if review { self.resumed_page() } else { self.forward(INCOME_PAGE) };
        if page == SPOUSE_PAGE {
            self.spouse_page(false)
        } else if page == REVIEW_PAGE && prepared.income_finished {
            self.review_with_result(*prepared.result)
        } else {
            self.page_with_income_result(page, false, Some(&prepared.result))
        }
    }
    fn accept(&mut self, draft: FederalDraft, page: i32, review: bool) -> Reply {
        self.accept_form(draft, page, review, Self::next_page)
    }
    fn accept_form(
        &mut self,
        draft: FederalDraft,
        page: i32,
        review: bool,
        next: fn(&Self, i32) -> Reply,
    ) -> Reply {
        match self.commit(draft) {
            // A Next from a page the review opened returns to the review, or
            // to the page a change made necessary (the spouse or people page).
            Ok(false) if review => self.review(),
            Ok(true) if review => self.resume(),
            // Otherwise follow this page's navigation, changed or not. A
            // change prunes only what its walk no longer asks, so later
            // pages stay answered unless reopened.
            Ok(_) => next(self, page),
            Err(message) => problem(message),
        }
    }
    // The page after `page` in the sequence.
    fn next_page(&self, page: i32) -> Reply {
        let walks = |section: TaxSection| in_walk(&self.draft, section);
        let next = match page {
            SETUP_PAGE => PEOPLE_PAGE,
            PEOPLE_PAGE if separate(&self.draft) => COMMUNITY_PAGE,
            PEOPLE_PAGE if walks(TaxSection::Spouse) => SPOUSE_PAGE,
            PEOPLE_PAGE | SPOUSE_PAGE | COMMUNITY_PAGE if walks(TaxSection::Children) => {
                CHILDREN_PAGE
            }
            PEOPLE_PAGE | SPOUSE_PAGE | COMMUNITY_PAGE | CHILDREN_PAGE => DOCUMENTS_PAGE,
            DOCUMENTS_PAGE => INCOME_WITHOUT_FORM_PAGE,
            INCOME_WITHOUT_FORM_PAGE => CHECKLIST_PAGE,
            // The checklist is followed by the first section it shows,
            // entered at its walk's first topic (M3-P19).
            CHECKLIST_PAGE => {
                next_shown(&self.draft, &shown_sections(&self.draft), TaxSection::Checklist) as i32
            }
            // The deductions page is followed by the spouse boxes' step while
            // the rules need them, then by its itemized hub when the route
            // itemizes, visited or not.
            DEDUCTIONS_PAGE if separate(&self.draft) => SPOUSE_ITEMIZES_PAGE,
            DEDUCTIONS_PAGE | SPOUSE_ITEMIZES_PAGE
                if crate::federal_rules::spouse_boxes_apply(&self.draft) =>
            {
                SPOUSE_BOXES_PAGE
            }
            DEDUCTIONS_PAGE | SPOUSE_BOXES_PAGE | SPOUSE_ITEMIZES_PAGE
                if walks(TaxSection::Itemized) =>
            {
                ITEMIZED_PAGE
            }
            // Box C (IncomeTiming::ByPeriod) opens the income by period page
            // (M3-P14), entered or not.
            PAYMENTS_PAGE if walks(TaxSection::IncomeByPeriod) => INCOME_BY_PERIOD_PAGE,
            // Form 8615 opens its parent's return page (u5-losses),
            // entered or not.
            OTHER_TAXES_PAGE if walks(TaxSection::ParentTax) => PARENT_TAX_PAGE,
            _ => return self.resume(),
        };
        self.enter(next)
    }
    // The candidate as its save stores it (walk-owned data): the people
    // pages' renames carried to the rows naming them and each topic's tick
    // following its entries; the prune of everything the candidate's walk
    // and allowed lists no longer ask (level 1); the derived values and the
    // sections M2 does not ask ("nothing applies"); then each finished page
    // whose own save refuses its answers reopened (level 2). An unchanged
    // candidate is the accepted return, already pruned.
    // Source-link checks need the canonical pruned/derived values, without
    // reopening pages and recursively invoking their candidate validation.
    fn reconcile_values(&self, draft: FederalDraft) -> (FederalDraft, Pruned) {
        self.reconcile_values_for_page(draft, None)
    }
    fn reconcile_values_for_page(
        &self,
        mut draft: FederalDraft,
        source: Option<TaxSection>,
    ) -> (FederalDraft, Pruned) {
        if draft == self.draft {
            return (draft, Pruned::default());
        }
        follow_ticks(&self.draft, &mut draft, source);
        let mut removed = if source == Some(TaxSection::Documents) {
            walk::prune_data_with_plan(
                &mut draft,
                !self.draft.finished_sections.contains(&TaxSection::Documents),
            )
        } else {
            prune_data(&mut draft)
        };
        if source == Some(TaxSection::Documents) {
            walk::removed_rows(&self.draft, &draft, &mut removed);
        }
        fill_unasked(&mut draft);
        follow_saver(&self.draft, &mut draft);
        crate::contribution_documents::reconcile(&self.draft, &mut draft);
        derive_hsa_employer(&mut draft);
        crate::moving::derive_reimbursed(&mut draft);
        follow_ira_year_end(&mut draft);
        follow_annuity_age(&mut draft);
        follow_marketplace_coverage(&mut draft);
        follow_full_year(&mut draft);
        follow_home_sales(&mut draft);
        (draft, Pruned { removed, ..Pruned::default() })
    }
    fn reconcile(&self, draft: FederalDraft) -> (FederalDraft, Pruned) {
        self.reconcile_for_page(draft, None)
    }
    fn reconcile_for_page(
        &self,
        draft: FederalDraft,
        source: Option<TaxSection>,
    ) -> (FederalDraft, Pruned) {
        if draft == self.draft {
            return (draft, Pruned::default());
        }
        let (mut draft, mut pruned) = self.reconcile_values_for_page(draft, source);
        pruned.reopened = reopen_refused(&mut draft);
        (draft, pruned)
    }
    // Stores a candidate draft for an accepted page; Ok(false) when it is unchanged.
    fn commit(&mut self, draft: FederalDraft) -> Result<bool, (&'static str, &'static str)> {
        self.commit_issue(draft).map_err(|(issue, _, english)| (issue.key, english))
    }
    fn income_draft_for_commit(
        &self,
        draft: FederalDraft,
        origins: &[(i64, usize, bool)],
        source: Option<TaxSection>,
    ) -> Result<(Option<FederalDraft>, Pruned), crate::validation::SaveFailure> {
        self.draft_for_commit(draft, source).map_err(|(mut issue, origin, _)| {
            issue = if let Some(id) = origin {
                submitted_graph_issue((id, issue), origins, false)
            } else if source == Some(TaxSection::Documents)
                && issue.has_field_prefix(&["credits", "marketplace", "forms"])
            {
                issue.map_field_prefix(
                    &["credits", "marketplace", "forms"],
                    &["documents", "common", "marketplace", "documents"],
                )
            } else {
                issue.field = None;
                issue
            };
            crate::validation::SaveFailure::from(issue)
        })
    }
    #[inline(never)]
    fn commit_income(
        &mut self,
        draft: FederalDraft,
        origins: &[(i64, usize, bool)],
        source: Option<TaxSection>,
    ) -> Result<PreparedIncomeCommit, crate::validation::SaveFailure> {
        let (draft, pruned) = self.income_draft_for_commit(draft, origins, source)?;
        let changed = draft.is_some();
        let candidate = draft.as_ref().unwrap_or(&self.draft);
        let income_finished = candidate.finished_sections.contains(&TaxSection::Income);
        let mut calculation = candidate.clone();
        finish_section(&mut calculation, TaxSection::Income);
        let result = alloc::boxed::Box::new(estimate(&calculation));
        if let Some(draft) = draft {
            self.draft = draft;
            self.dirty = true;
        }
        self.accepted_save = true;
        Ok(PreparedIncomeCommit { pruned, changed, income_finished, result })
    }
    fn commit_marketplace(
        &mut self,
        draft: FederalDraft,
    ) -> Result<bool, crate::validation::SaveFailure> {
        self.commit_issue(draft).map_err(|(mut issue, _, _)| {
            if issue.has_field_prefix(&["credits", "marketplace"]) {
                issue = issue.map_field_prefix(&["credits", "marketplace"], &["value"]);
            } else {
                issue.field = None;
            }
            issue.into()
        })
    }
    fn commit_issue(&mut self, draft: FederalDraft) -> Result<bool, CommitIssue> {
        let (candidate, _) = self.draft_for_commit(draft, None)?;
        self.accepted_save = true;
        match candidate {
            Some(draft) => {
                self.draft = draft;
                self.dirty = true;
                Ok(true)
            }
            None => Ok(false),
        }
    }
    #[inline(never)]
    fn draft_for_commit(
        &self,
        draft: FederalDraft,
        source: Option<TaxSection>,
    ) -> Result<(Option<FederalDraft>, Pruned), CommitIssue> {
        // A review-only Next is navigation: compare answers before touching saved
        // history, completion, or revision. Cookie/navigation metadata is not in draft.
        // The accepted return is always pruned, so an unchanged page prunes nothing.
        if draft == self.draft {
            return Ok((None, Pruned::default()));
        }
        // Replace the accepted page and remove what its answers no longer ask
        // in the same persisted record. Never reuse identities from erased
        // dependent/document rows.
        let (mut draft, pruned) = self.reconcile_for_page(draft, source);
        if let Some(key) = crate::extended_rules::rental_mortgage_rule(&draft) {
            return Err((
                ValidationIssue::new(key),
                None,
                crate::fixture_english!(
                    "Unlink the loan in Rentals before removing it from Mortgage."
                ),
            ));
        }
        if let Some(issue) = crate::home_sites::graph_issue(&draft) {
            return Err((
                issue,
                None,
                crate::fixture_english!(
                    "Correct the saved home, activity, loan or casualty reference before saving."
                ),
            ));
        }
        skip_empty_sections(&mut draft);
        if let Some((source, issue)) = health_correction_reference_rule(&draft)
            .or_else(|| crate::care_benefit_return::reference_rule(&draft))
            .map(|key| (None, ValidationIssue::new(key)))
            .or_else(|| crate::injured_spouse_sources::reference_issue(&draft))
            .or_else(|| {
                crate::qbi_aggregation::reference_rule(&draft)
                    .map(|key| (None, ValidationIssue::new(key)))
            })
        {
            return Err((
                issue,
                source,
                crate::fixture_english!(
                    "The linked source must remain available for the same owner. Update or remove its correction or care-benefit link before removing or changing the source."
                ),
            ));
        }
        if let Some(issue) = draft.stored_issue() {
            return Err((
                issue,
                None,
                crate::fixture_english!("Review the accepted page values before saving."),
            ));
        }
        let Some(revision) = draft.revision.checked_add(1).filter(|r| *r < i64::MAX) else {
            return Err((
                ValidationIssue::new(
                    "ISSUE_INTERVIEW_THIS_RETURN_HAS_REACHED_THE_SUPPORTED_E3B1364A",
                ),
                None,
                crate::fixture_english!("This return has reached the supported revision limit."),
            ));
        };
        draft.revision = revision;
        draft.finished = false;
        Ok((Some(draft), pruned))
    }
    // The running estimate of a page's answers merged into the accepted
    // return. Pure: nothing is stored and no revision changes.
    #[inline(never)]
    fn prepare_running(
        &self,
        candidate: Result<FederalDraft, impl Into<crate::validation::SaveFailure>>,
        topic: Option<TaxSection>,
    ) -> Result<PreparedEstimate, crate::validation::SaveFailure> {
        self.prepare_running_for_page(candidate, topic, None)
    }
    #[inline(never)]
    fn prepare_running_for_page(
        &self,
        candidate: Result<FederalDraft, impl Into<crate::validation::SaveFailure>>,
        topic: Option<TaxSection>,
        source: Option<TaxSection>,
    ) -> Result<PreparedEstimate, crate::validation::SaveFailure> {
        let draft = candidate.map_err(Into::into)?;
        let (mut draft, pruned) = self.reconcile_for_page(draft, source);
        if let Some(key) = crate::extended_rules::rental_mortgage_rule(&draft) {
            return Err(status(key).into());
        }
        if topic.is_some() {
            finish_section(&mut draft, TaxSection::Income);
        }
        let result = alloc::boxed::Box::new(estimate(&draft));
        Ok(PreparedEstimate { draft: alloc::boxed::Box::new(draft), result, pruned })
    }
    fn running(
        &self,
        candidate: Result<FederalDraft, impl Into<crate::validation::SaveFailure>>,
        page: i32,
        topic: Option<TaxSection>,
    ) -> Reply {
        let section = topic.unwrap_or_else(|| {
            *SEQUENCE.iter().find(|section| **section as i32 == page).expect("known estimate page")
        });
        match self.prepare_running(candidate, topic) {
            Ok(prepared) => Reply::Response(P::RunningEstimate {
                estimate: self
                    .estimate_so_far(&prepared.draft, &prepared.result, section)
                    .map(Into::into),
                warnings: save_warnings(&prepared.pruned),
                progress: progress_text(page, &prepared.draft, topic),
            }),
            Err(message) => refused(message),
        }
    }
    fn income_running(
        &self,
        topic: TaxSection,
        input: crate::IncomeRecordsEstimateInput,
        followups: Vec<crate::DocumentQuestion>,
    ) -> Reply {
        if !INCOME_TOPICS.contains(&topic) {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        }
        let names = input.qbi_aggregations.iter().map(|g| g.facts.name.clone()).collect::<Vec<_>>();
        let rows = input.documents.len();
        let input = input.into_page();
        let mut origins = Vec::new();
        let question_keys: Vec<_> = followups.iter().map(|row| row.key.clone()).collect();
        let candidate = self
            .followup_candidate_with_mode(topic, followups, false, &input.documents)
            .and_then(|candidate| {
                candidate.documents_candidate_inner_with_origins(
                    topic,
                    input.into_edits(),
                    true,
                    Some(&mut origins),
                    Some(&question_keys),
                )
            });
        let (groups, sources) = candidate
            .as_ref()
            .map(|draft| {
                let sources = crate::qbi_aggregation::source_views(draft, &self.draft);
                let groups = names
                    .iter()
                    .map(|name| crate::qbi_aggregation::derived_group(name, &sources))
                    .collect();
                (groups, sources.into_iter().map(|v| v.member).collect())
            })
            .unwrap_or_default();
        match self.prepare_running(candidate, Some(topic)) {
            Ok(prepared) => Reply::Response(P::IncomeRunningEstimate {
                refund_management: income_refund_management(
                    &prepared.draft,
                    &prepared.result,
                    &origins,
                    rows,
                ),
                estimate: self
                    .estimate_so_far(&prepared.draft, &prepared.result, topic)
                    .map(Into::into),
                warnings: save_warnings(&prepared.pruned),
                progress: progress_text(INCOME_PAGE, &prepared.draft, Some(topic)),
                groups,
                sources,
            }),
            Err(message) => refused(message),
        }
    }

    fn joint(&self) -> bool {
        joint(&self.draft)
    }
    // The spouse page exists once a joint return's own filer page is saved.
    fn spouse_open(&self) -> bool {
        in_walk(&self.draft, TaxSection::Spouse) && self.draft.people.is_some()
    }
    // The spouse page belongs to the filers section: a joint return whose
    // spouse is not yet saved is still open at that section.
    fn spouse_missing(&self) -> bool {
        spouse_missing(&self.draft)
    }
    // The first unfinished page of the sequence, or the review.
    fn first_open(&self) -> i32 {
        first_open(&self.draft)
    }
    // Editor prerequisites, shared by page construction and save admission.
    // Walkthrough order still controls Resume, but does not prevent a jump
    // once the return and all required filers are known. Conditional pages
    // additionally use in_walk and their existing form-specific guards.
    fn reached(&self, page: i32) -> bool {
        rank(page) <= rank(self.first_open())
            || (self.draft.setup.is_some() && self.draft.people.is_some() && !self.spouse_missing())
    }
    // Opens the next step (M3-P19): a joint return's missing spouse page,
    // else the first unfinished section, entered at its walk's first topic
    // (the next planned or derived topic of a hub not yet visited), else
    // the review. Sections with nothing for this return were already passed
    // by the accepted change that reached them (skip_empty_sections).
    fn resumed_page(&self) -> i32 {
        if self.spouse_missing() { SPOUSE_PAGE } else { self.entered_page(self.first_open()) }
    }
    fn resume(&self) -> Reply {
        let page = self.resumed_page();
        if page == SPOUSE_PAGE { self.spouse_page(false) } else { self.page(page, false) }
    }
    // A topic page's documents merged into the accepted return: the entries of
    // the topic's kinds are replaced in place, and the other topics' entries
    // are kept.
    fn documents_candidate(
        &self,
        topic: TaxSection,
        documents: IncomeEdits,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        self.documents_candidate_inner(topic, documents, false)
    }
    fn documents_candidate_inner(
        &self,
        topic: TaxSection,
        documents: IncomeEdits,
        calculation_input: bool,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        self.documents_candidate_inner_with_origins(topic, documents, calculation_input, None, None)
    }
    fn documents_candidate_inner_with_origins(
        &self,
        topic: TaxSection,
        documents: IncomeEdits,
        calculation_input: bool,
        origins: Option<&mut Vec<(i64, usize, bool)>>,
        question_keys: Option<&[String]>,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        let value_fields: Vec<bool> = documents
            .documents
            .iter()
            .map(|item| topic == TaxSection::Documents && item.document.has_value_field())
            .collect();
        self.documents_candidate_sources(
            topic,
            documents,
            calculation_input,
            origins,
            question_keys,
        )
        .map_err(|failure| {
            failure.map_list_item_field_prefix(
                &["documents", "documents"],
                &["document"],
                &["document", "value"],
                &value_fields,
            )
        })
    }
    fn documents_candidate_sources(
        &self,
        topic: TaxSection,
        documents: IncomeEdits,
        calculation_input: bool,
        origins: Option<&mut Vec<(i64, usize, bool)>>,
        question_keys: Option<&[String]>,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        let d = &self.draft;
        let mut documents = documents;
        if topic == TaxSection::Documents {
            document_followups::retain_source_answers(d, &mut documents.documents);
        }
        let mut aggregation_pages = documents.qbi_aggregations.clone();
        for (submitted_index, item) in documents.documents.iter().enumerate() {
            if let Some(issue) = qbi_group_input_issue(item, !aggregation_pages.is_empty()) {
                return Err(issue
                    .in_field("document")
                    .in_list("documents", submitted_index)
                    .in_field("documents")
                    .into());
            }
        }
        for (index, item) in documents.documents.iter().enumerate() {
            let payroll = match &item.document {
                IncomeSource::Record { value: crate::IncomeRecord::Business(v) } => {
                    Some(&v.more.payroll)
                }
                IncomeSource::Record { value: crate::IncomeRecord::Rental(v) } => {
                    Some(&v.more.payroll)
                }
                IncomeSource::Record { value: crate::IncomeRecord::Farm(v) } => {
                    Some(&v.more.payroll)
                }
                IncomeSource::Record { value: crate::IncomeRecord::FarmRental(v) } => {
                    Some(&v.more.payroll)
                }
                _ => None,
            };
            if let Some(issue) = payroll.and_then(|payroll| payroll.issue()) {
                return Err(issue
                    .in_field("payroll")
                    .in_field("more")
                    .in_field("document")
                    .in_list("documents", index)
                    .in_field("documents")
                    .into());
            }
        }
        let Some(kinds) = (if topic == TaxSection::Documents {
            Some(center_document_kinds())
        } else {
            record_kinds(topic)
        }) else {
            return Err((crate::interview::status("PROBLEM_SECTION_UNAVAILABLE_YET")).into());
        };
        if let Some(issue) =
            documents.documents.iter().enumerate().find_map(|(index, item)| {
                (match &item.document {
                    document @ IncomeSource::InvestmentSale {
                        value:
                            crate::InvestmentSaleDocument::Form1099B(_)
                            | crate::InvestmentSaleDocument::Form1099DA(_),
                    } => crate::broker_page::year_issue(document, self.draft.tax_year)
                        .or_else(|| document_local_issue(document)),
                    IncomeSource::Wage { value: crate::WageDocument::W2(w) }
                        if topic == TaxSection::Documents =>
                    {
                        crate::state_return::document_rule(&item.document)
                            .map(|key| ValidationIssue::new(key).at_record())
                            .or_else(|| w2_input_issue(w))
                    }
                    document @ IncomeSource::InterestDividend {
                        value:
                            crate::InterestDividendDocument::Int1099(_)
                            | crate::InterestDividendDocument::Div1099(_)
                            | crate::InterestDividendDocument::Oid1099(_),
                    } if topic == TaxSection::Documents => {
                        crate::state_return::document_rule(document)
                            .map(|key| ValidationIssue::new(key).at_record())
                            .or_else(|| investment_statement_input_issue(document, false))
                    }
                    document @ IncomeSource::K1 {
                        value:
                            crate::K1Document::K1Partnership(_)
                            | crate::K1Document::K1Scorp(_)
                            | crate::K1Document::K1EstateTrust(_),
                    } if topic == TaxSection::Documents => k1_followups::issue(document, false),
                    document @ IncomeSource::OtherIncome {
                        value: crate::OtherIncomeDocument::Form1099Misc(value),
                    } if topic == TaxSection::Documents => {
                        crate::state_return::document_rule(document)
                            .map(|key| ValidationIssue::new(key).at_record())
                            .or_else(|| {
                                crate::information_returns::misc_input_issue(
                                    &crate::Form1099Misc::from(value),
                                    false,
                                )
                            })
                    }
                    document @ IncomeSource::SelfEmployment {
                        value: crate::SelfEmploymentDocument::Form1099K(value),
                    } if topic == TaxSection::Documents => {
                        crate::state_return::document_rule(document)
                            .map(|key| ValidationIssue::new(key).at_record())
                            .or_else(|| value.issue())
                    }
                    IncomeSource::Farming(value) if topic == TaxSection::Documents => {
                        patr_paper_input_issue(value)
                    }
                    IncomeSource::OtherIncome {
                        value: crate::OtherIncomeDocument::Ltc1099(value),
                    } if topic == TaxSection::Documents => value.intrinsic_issue(),
                    IncomeSource::OtherIncome {
                        value: crate::OtherIncomeDocument::Qa1099(value),
                    } if topic == TaxSection::Documents => value.boxes_issue(),
                    IncomeSource::OtherIncome {
                        value: crate::OtherIncomeDocument::Q1099(value),
                    } if topic == TaxSection::Documents => q1099_input_issue(value),
                    IncomeSource::OtherIncome {
                        value: crate::OtherIncomeDocument::Sa1099(value),
                    } if topic == TaxSection::Documents => value.input_issue(),
                    IncomeSource::OtherIncome {
                        value: crate::OtherIncomeDocument::HsaTrusteeStatement(value),
                    } if topic == TaxSection::Documents => value.input_issue(),
                    IncomeSource::Retirement { value } if topic == TaxSection::Documents => {
                        retirement_paper_issue(value)
                    }
                    IncomeSource::Record {
                        value: crate::IncomeRecord::MainHomeSale { sale, answers },
                    } => home_sale_record_issue(sale, answers, self.joint()),
                    document => document_issue(document),
                })
                .or_else(|| rental_casualty_year_issue(&item.document, self.draft.tax_year))
                .map(|issue| {
                    issue.in_field("document").in_list("documents", index).in_field("documents")
                })
            })
        {
            return Err(issue.into());
        }
        let people = self.income_people();
        if let Some(index) = documents.documents.iter().position(|item| {
            matches!(&item.document,
                IncomeSource::Record {
                    value: crate::IncomeRecord::LtcInsured(crate::LtcInsured {
                        insured: crate::LtcInsuredPerson::OnReturn { person }, ..
                    })
                } if !people.iter().any(|choice| choice.id == *person))
        }) {
            return Err(ValidationIssue::new(
                "MESSAGE_A_PERSON_IS_NOT_ON_THIS_RETURN_CHOOSE_THE_PERSON_AGAIN",
            )
            .at_field("person")
            .in_field("insured")
            .in_field("document")
            .in_list("documents", index)
            .in_field("documents")
            .into());
        }
        if let Some(index) = documents.documents.iter().position(|item| {
            // A document without an owner is saved and becomes a pending item.
            !(matches!(item.document, IncomeSource::Record { .. })
                || self.income_owners().contains(&(item.owner as i32))
                || (topic == TaxSection::Documents && item.owner == crate::DocumentOwner::Unknown))
        }) {
            return Err(ValidationIssue::new(
                "ISSUE_SPOUSE_OWNED_INCOME_NEEDS_A_JOINT_OR_COMMUNITY_PROPERTY_RETURN",
            )
            .at_field("owner")
            .in_list("documents", index)
            .in_field("documents")
            .into());
        }
        if documents.documents.iter().any(|item| !kinds.contains(&document_index(&item.document))) {
            return Err((crate::interview::status(
                "MESSAGE_THIS_DOCUMENT_BELONGS_TO_ANOTHER_INCOME_TOPIC",
            ))
            .into());
        }
        if topic != TaxSection::Documents {
            for (index, item) in documents.documents.iter().enumerate() {
                if let Some(issue) = wage_reference_issue(d, item) {
                    return Err(submitted_data_answer_issue(issue, index).into());
                }
            }
        }
        // EnumSource is checked again against this editor's actual accepted
        // source catalog, before reconciliation can remove stale relations.
        for (index, item) in documents.documents.iter().enumerate() {
            let sources = match &item.document {
                IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } => {
                    &farm.more.specials.external_sources
                }
                IncomeSource::Record { value: crate::IncomeRecord::FarmRental(farm) } => {
                    &farm.more.specials.external_sources
                }
                _ => continue,
            };
            if let Some(source_index) = sources.iter().position(|source| {
                !crate::farm_specials::external_source_available(d, source.source)
            }) {
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new("FARM_EXTERNAL_SOURCE_UNAVAILABLE")
                        .at_field("source")
                        .in_list("external_sources", source_index)
                        .in_field("specials")
                        .in_field("more"),
                    index,
                )
                .into());
            }
        }
        // A rental in which a real estate professional materially
        // participated needs the owner's recorded status (the People or
        // Spouse page); the estimate refuses it again if the status goes.
        if let Some(index) = documents.documents.iter().position(|item| {
            matches!(&item.document, IncomeSource::Record { value: crate::IncomeRecord::Rental(r) }
                if r.more.participation.professional())
                && d.professional(crate::community::professional_owner(
                    d.setup.as_ref(),
                    item.owner.owned(),
                ))
                .is_none()
        }) {
            return Err(submitted_data_answer_issue(
                ValidationIssue::new("RENTAL_PROFESSIONAL_NEEDS_STATUS")
                    .at_field("participation")
                    .in_field("more"),
                index,
            )
            .into());
        }
        // The Community or separate answer inside the set the page was given
        // (ConstrainedBy on the client), checked again.
        if topic != TaxSection::Documents
            && let Some(issue) = documents.documents.iter().enumerate().find_map(|(index, item)| {
                document_community_issue(d, item)
                    .map(|issue| issue.in_list("documents", index).in_field("documents"))
            })
        {
            return Err(issue.into());
        }
        let management = crate::injured_spouse_payments::allowed_document_management(
            crate::injured_spouse::texas(d),
        );
        for (index, item) in documents.documents.iter().enumerate() {
            if !crate::injured_spouse_sources::requested(d)
                && let Some(issue) =
                    crate::injured_spouse_sources::document_facts_issue(&item.document)
            {
                return Err(submitted_data_answer_issue(issue, index).into());
            }
            if item.refund_management.is_some_and(|v| {
                !management.contains(&crate::injured_spouse_payments::management_index(v))
            }) {
                return Err(ValidationIssue::new("INJURED_SPOUSE_WITHHOLDING_MANAGEMENT_NEEDED")
                    .at_field("refund_management")
                    .in_list("documents", index)
                    .in_field("documents")
                    .into());
            }
        }
        for (index, item) in documents.documents.iter().enumerate() {
            if matches!(
                item.document,
                IncomeSource::Record { value: crate::IncomeRecord::InstallmentSale(_) }
            ) && !self.income_owners().contains(&(item.owner as i32))
            {
                return Err(ValidationIssue::new("INSTALLMENT_OWNER_INVALID")
                    .at_field("owner")
                    .in_list("documents", index)
                    .in_field("documents")
                    .into());
            }
        }
        // Alimony received (u6-alimony): the owner inside allowed_owners
        // and the record's own checks, as More adjustments checks the
        // payer's row.
        let owners = self.allowed_owners();
        for (index, item) in documents.documents.iter().enumerate() {
            let IncomeSource::Record { value: crate::IncomeRecord::AlimonyReceived(payment) } =
                &item.document
            else {
                continue;
            };
            if !owners.contains(&(item.owner as i32)) {
                return Err(ValidationIssue::new("MESSAGE_ALIMONY_YOUR_SPOUSE_RECEIVED_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY")
                    .at_field("owner").in_list("documents", index).in_field("documents").into());
            }
            if let Some(issue) = payment.validation_issue("MESSAGE_ALIMONY_RECEIVED_ENTER_THE_AGREEMENT_DATE_BEFORE_2027_AND_AMOUNTS_WITH_THIS") {
                return Err(submitted_data_answer_issue(issue, index).into());
            }
        }
        if documents
            .documents
            .iter()
            .filter(|item| {
                matches!(
                    item.document,
                    IncomeSource::Record { value: crate::IncomeRecord::AlimonyReceived(_) }
                )
            })
            .count()
            > 20
        {
            return Err(
                (crate::interview::status("MESSAGE_ENTER_AT_MOST_20_ALIMONY_AGREEMENTS")).into()
            );
        }
        // What produces a business's income (T2-cy), inside the allowed set
        // the page was given (ConstrainedBy on the client), checked again.
        // Farms are checked with the farm kind below.
        income_source_rule(
            &documents.documents,
            self.business_source_needed(),
            self.partner_source_needed(),
            topic != TaxSection::Documents,
        )?;
        // A child's election (Form 8814): a child the return can report, once
        // (the client rule on IncomeRecords, checked again), and the other
        // parent's answer inside the allowed set the page was given.
        let reportable = child_election_children(d);
        let other_parent = crate::OtherParentChoice::allowed(!self.joint());
        let mut elected: Vec<i64> = Vec::new();
        for (index, item) in documents.documents.iter().enumerate() {
            let IncomeSource::Record { value: crate::IncomeRecord::Form8814(v) } = &item.document
            else {
                continue;
            };
            if elected.contains(&v.child) {
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new("CHILD_ELECTION_TWICE").at_field("child"),
                    index,
                )
                .into());
            }
            elected.push(v.child);
            if !reportable.iter().any(|c| c.id == v.child) {
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new("CHILD_ELECTION_CHILD_NOT_ELIGIBLE").at_field("child"),
                    index,
                )
                .into());
            }
            if !other_parent.contains(&v.other_parent.index()) {
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new(if self.joint() {
                        "CHILD_ELECTION_PARENT_NOT_NEEDED"
                    } else {
                        "CHILD_ELECTION_PARENT_NEEDED"
                    })
                    .at_field("other_parent"),
                    index,
                )
                .into());
            }
        }
        // The client rule on IncomeRecords, checked again.
        if let Some(issue) =
            repeated_document_issue(&documents.documents, "GAMBLING_LOSSES_TWICE", |document| {
                matches!(
                    document,
                    IncomeSource::Record { value: crate::IncomeRecord::GamblingLosses(_) }
                )
            })
        {
            return Err(issue.into());
        }
        // The client rule on IncomeRecords, checked again.
        if let Some(issue) =
            repeated_document_issue(&documents.documents, "SAVINGS_BONDS_TWICE", |document| {
                matches!(
                    document,
                    IncomeSource::InterestDividend {
                        value: crate::InterestDividendDocument::SavingsBonds(_)
                    }
                )
            })
        {
            return Err(issue.into());
        }
        if let Some(issue) = repeated_document_issue(
            &documents.documents,
            "EDUCATION_ACCOUNT_EXPENSES_TWICE",
            |document| {
                matches!(
                    document,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::EducationAccountExpenses(_)
                    }
                )
            },
        ) {
            return Err(issue.into());
        }
        // A 1099-INT box 6 or 1099-DIV box 7 foreign tax is stored on its
        // document and read from there by the payee-statement election.
        // The data answers (M3-P15) are FederalScreening facts of this
        // topic; checked against the page's own documents before they are
        // split out.
        // Capital history is collected in this submitted order before data answers
        // are partitioned away. These are original request indices, never income IDs.
        let capital_document_positions: Vec<usize> = documents
            .documents
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                matches!(
                    item.document,
                    IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(_) }
                )
                .then_some(index)
            })
            .collect();
        let submitted_asset_rows: Vec<(usize, Vec<ValidationIssue>)> = documents
            .documents
            .iter()
            .enumerate()
            .map(|(index, item)| (index, document_asset_rows(&item.document)))
            .collect();
        let foreign_claim_positions: Vec<(Owner, usize)> = documents
            .documents
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                matches!(
                    item.document,
                    IncomeSource::Record { value: crate::IncomeRecord::Form2555(_) }
                )
                .then_some((item.owner.owned(), index))
            })
            .collect();
        let mut foreign_pay_positions: Vec<(Owner, ValidationIssue, bool)> = Vec::new();
        for (index, item) in documents.documents.iter().enumerate() {
            let key = "ISSUE_FOREIGN_AWARDS_ABOVE_PAY";
            let (amount, issue) = match &item.document {
                IncomeSource::Wage { value: crate::WageDocument::W2(_) } => (
                    d.income
                        .iter()
                        .find(|entry| entry.id == item.id)
                        .and_then(|entry| match &entry.income {
                            FederalIncome::Wage { data } => {
                                Some(crate::FederalWage::from(data).abroad)
                            }
                            _ => None,
                        })
                        .unwrap_or(0),
                    ValidationIssue::new(key).at_field("abroad").in_field("answers"),
                ),
                IncomeSource::Record { value: crate::IncomeRecord::ForeignPay(value) } => {
                    (value.wages, ValidationIssue::new(key).at_field("wages"))
                }
                _ => continue,
            };
            let old = d.income.iter().find(|entry| entry.id == item.id && item.id != 0).and_then(
                |entry| match &entry.income {
                    FederalIncome::Wage { data } => {
                        Some((entry.owner, crate::FederalWage::from(data).abroad))
                    }
                    FederalIncome::ForeignEmployment { data } => Some((entry.owner, data.wages)),
                    _ => None,
                },
            );
            // These comparisons only describe a posted edit after the canonical
            // aggregate owner refuses. They never decide whether the pay is valid.
            if let Some((old_owner, old_amount)) = old
                && old_owner != item.owner
                && old_amount > 0
            {
                foreign_pay_positions.push((
                    old_owner.owned(),
                    ValidationIssue::new(key)
                        .at_field("owner")
                        .in_list("documents", index)
                        .in_field("documents"),
                    true,
                ));
            }
            foreign_pay_positions.push((
                item.owner.owned(),
                submitted_data_answer_issue(issue, index),
                old.is_some_and(|(old_owner, old_amount)| {
                    old_owner == item.owner && old_amount > amount
                }),
            ));
        }
        let nol_document_position = documents.documents.iter().position(|item| {
            matches!(
                item.document,
                IncomeSource::Record { value: crate::IncomeRecord::NetOperatingLoss(_) }
            )
        });
        let farm_averaging_position = documents.documents.iter().position(|item| {
            matches!(
                item.document,
                IncomeSource::Record { value: crate::IncomeRecord::FarmAveraging(_) }
            )
        });
        let data_answers = data_answer_records(self, topic, &documents.documents)?;
        let lookback = lookback_records(topic, &documents.documents)?;
        let spouse_179 = spouse_179_records(&self.draft, topic, &documents.documents)?;
        // The farm kind (T2-45p): its income source inside the allowed set
        // the page was given (T2-cy: the Nothing branch counts as Not
        // needed, so a return that reads the source must answer it under
        // Something else), and one farm optional method answer per owner
        // across that owner's farms (FARM_OPTIONAL_SE_DIFFERS).
        let allowed_sources = self.allowed_farm_source();
        let mut farm_se: Vec<(Owner, bool)> = Vec::new();
        for (index, item) in documents.documents.iter().enumerate() {
            let IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } = &item.document
            else {
                continue;
            };
            if !allowed_sources.contains(&crate::farm_page::farm_income_source_index(farm)) {
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new(if self.business_source_needed() {
                        FARM_SOURCE_NEEDED
                    } else {
                        SOURCE_NOT_NEEDED
                    })
                    .at_field("income_source")
                    .in_field("more"),
                    index,
                )
                .into());
            }
            let elected = crate::farm_page::farm_optional_se(farm);
            match farm_se.iter().find(|(owner, _)| *owner == item.owner.owned()) {
                Some((_, other)) if *other != elected => {
                    return Err(submitted_data_answer_issue(
                        ValidationIssue::new("FARM_OPTIONAL_SE_DIFFERS")
                            .at_field("optional_se")
                            .in_field("more"),
                        index,
                    )
                    .into());
                }
                Some(_) => {}
                None => farm_se.push((item.owner.owned(), elected)),
            }
        }
        let original_adjustment_positions: Vec<usize> = documents
            .documents
            .iter()
            .enumerate()
            .filter_map(|(index, item)| is_adjustment_document(&item.document).then_some(index))
            .collect();
        let original_income_positions: Vec<(usize, bool)> = documents
            .documents
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                !is_data_answer(&item.document) && !is_adjustment_document(&item.document)
            })
            .map(|(index, item)| (index, matches!(&item.document, IncomeSource::Farming(_))))
            .collect();
        let documents = IncomeEdits {
            qbi_aggregations: documents.qbi_aggregations,
            documents: documents
                .documents
                .into_iter()
                .filter(|item| !is_data_answer(&item.document))
                .collect(),
        };
        // Form 8606 and QCD gifts affect Adjustments, not an income entry;
        // split them out before the ordinary income-entry pipeline runs.
        let (adjustment_items, income_items): (Vec<_>, Vec<_>) = documents
            .documents
            .into_iter()
            .partition(|item| is_adjustment_document(&item.document));
        // FORM_8606_TWICE: at most one Form 8606 per owner (the client rule
        // is_unique_set(collect(...)) on IncomeRecords, checked again).
        let mut form8606_owners: Vec<Owner> = adjustment_items
            .iter()
            .filter_map(|item| {
                matches!(
                    item.document,
                    IncomeSource::Record { value: crate::IncomeRecord::Form8606(_) }
                )
                .then_some(item.owner.owned())
            })
            .collect();
        if let Some((index, _)) = form8606_owners
            .iter()
            .enumerate()
            .find(|(index, owner)| form8606_owners[..*index].contains(owner))
        {
            let adjustment_index = adjustment_items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    matches!(
                        item.document,
                        IncomeSource::Record { value: crate::IncomeRecord::Form8606(_) }
                    )
                })
                .nth(index)
                .unwrap()
                .0;
            return Err(ValidationIssue::new("FORM_8606_TWICE")
                .at_field("owner")
                .in_list("documents", original_adjustment_positions[adjustment_index])
                .in_field("documents")
                .into());
        }
        if topic == TaxSection::Documents {
            form8606_owners =
                d.adjustments.iter().flat_map(|a| &a.ira_basis).map(|b| b.owner).collect();
        }
        let social_election =
            social_election_record(d, &adjustment_items, &original_adjustment_positions)?;
        let required_distributions =
            missed_distribution_records(self, &adjustment_items, &original_adjustment_positions)?;
        let roth_history =
            roth_history_records(self, &adjustment_items, &original_adjustment_positions)?;
        // Preview values belong to the editor, never accepted storage or comparison.
        for (index, item) in income_items.iter().enumerate() {
            if let Some(issue) = crate::k1_activity::page_issue(item, &d.income) {
                return Err(issue
                    .in_field("document")
                    .in_list("documents", original_income_positions[index].0)
                    .in_field("documents")
                    .into());
            }
        }
        let mut entries: Vec<_> =
            income_items.into_iter().map(|item| document_entry(item, &d.income)).collect();
        for entry in &mut entries {
            crate::farm_rental::retain_inapplicable_property_facts(entry, &d.income);
        }
        for (entry_index, record) in entries.iter().enumerate() {
            crate::federal_rules::validate_income_issue_for_year_with_mode(
                record,
                u32::try_from(d.tax_year).unwrap_or(0),
                topic != TaxSection::Documents,
            )
            .map_err(|(issue, _)| {
                let issue = if matches!(record.income, FederalIncome::Cooperative { .. }) {
                    project_cooperative_issue(issue, original_income_positions[entry_index].1)
                } else {
                    issue
                };
                SaveFailure::from(submitted_data_answer_issue(
                    issue,
                    original_income_positions[entry_index].0,
                ))
            })?;
            // After-death allocations are personal topic answers. A printed
            // correction retains them for correction on that topic.
            if topic != TaxSection::Documents
                && let Some(issue) = recorded_income_owner_issue(d, record)
            {
                let issue =
                    submitted_data_answer_issue(issue, original_income_positions[entry_index].0);
                return Err(SaveFailure {
                    message: crate::Message::new(
                        issue.key,
                        alloc::vec![income_display_arg(record)],
                    )
                    .encode(),
                    field: issue.field,
                });
            }
        }
        // A new, unaccepted source cannot be selected by guessing the next
        // counter value. The edited graph below still owns current amounts.
        let accepted_bond_sources = crate::savings_bonds::sources(&d.income);
        if let Some((entry_index, _)) = entries.iter().enumerate().find(|(_, entry)| matches!(&entry.income,
            FederalIncome::SavingsBonds { data } if data.answers.as_ref().is_some_and(|answers| !accepted_bond_sources.iter().any(|source| source.id == answers.interest_statement)))) {
            return Err(submitted_data_answer_issue(ValidationIssue::new("SAVINGS_BONDS_SOURCE_INVALID").at_field("interest_statement"), original_income_positions[entry_index].0).into());
        }
        // The IRA basis worksheet applies to every one of an owner's IRA
        // distributions once that owner's Form 8606 is on this page (never
        // asked per document); removing the Form 8606 turns them back to box
        // 2a, since the box 2b method always starts from the payer amount or
        // the Simplified Method fold alone.
        retirement_ira_basis(&mut entries, &form8606_owners);
        let mut draft = d.clone();
        assign_record_ids(
            entries.iter_mut().map(|r| &mut r.id),
            d.income.iter().filter(|r| source_in_editor(d, r, topic)).map(|r| r.id),
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            ValidationIssue::new(key)
                .at_field("id")
                .in_list("documents", original_income_positions[index].0)
                .in_field("documents")
        })?;
        for (index, entry) in entries.iter_mut().enumerate() {
            let accepted = d
                .income
                .iter()
                .find(|old| old.id == entry.id)
                .map_or_else(Vec::new, |old| crate::cooperative::other_ids(&old.income));
            assign_record_ids(
                crate::cooperative::other_ids_mut(&mut entry.income).into_iter(),
                accepted.into_iter(),
                &mut draft.next_id,
            )
            .map_err(|(key, _, child)| {
                submitted_data_answer_issue(
                    ValidationIssue::new(key)
                        .at_field("id")
                        .in_list("other", child)
                        .in_field("credit_facts")
                        .in_field("more"),
                    original_income_positions[index].0,
                )
            })?;
        }
        related_forms::prepare_component_ids(&mut entries, &d.income, &mut draft.next_id).map_err(
            |(id, issue)| {
                let index = entries.iter().position(|entry| entry.id == id).unwrap();
                submitted_data_answer_issue(issue, original_income_positions[index].0)
            },
        )?;
        let submitted_origins: Vec<(i64, usize, bool)> = entries
            .iter()
            .zip(&original_income_positions)
            .map(|(entry, (index, automatic))| (entry.id, *index, *automatic))
            .collect();
        if let Some(origins) = origins {
            *origins = submitted_origins.clone();
        }
        related_forms::prepare_annual_ids(&mut entries, &d.income, &mut draft.next_id).map_err(
            |(id, issue)| {
                let index = entries.iter().position(|entry| entry.id == id).unwrap();
                submitted_data_answer_issue(issue, original_income_positions[index].0)
            },
        )?;
        related_forms::prepare_activity_ids(&mut entries, &d.income, &mut draft.next_id).map_err(
            |(id, issue)| {
                let index = entries.iter().position(|entry| entry.id == id).unwrap();
                submitted_data_answer_issue(issue, original_income_positions[index].0)
            },
        )?;
        if !crate::k1_activity::section179_amounts_in_range(&entries) {
            return Err(ValidationIssue::new(crate::k1_activity::ordinary::RANGE_KEY)
                .at_field("documents")
                .in_field("documents")
                .into());
        }
        if let Some((entry_index, issue)) =
            crate::estate_trust::activity_identity_issue(&entries, &d.income)
        {
            return Err(submitted_data_answer_issue(
                issue,
                original_income_positions[entry_index].0,
            )
            .into());
        }
        assign_record_ids(
            entries.iter_mut().flat_map(|e| crate::estate_trust::activity_ids_mut(&mut e.income)),
            d.income
                .iter()
                .filter(|e| source_in_editor(d, e, topic))
                .flat_map(|e| crate::estate_trust::activity_ids(&e.income)),
            &mut draft.next_id,
        )
        .map_err(|(key, _, _)| status(key))?;
        for (entry_index, entry) in entries.iter_mut().enumerate() {
            assign_record_ids(
                crate::investment_credit::k1_property_ids_mut(&mut entry.income).into_iter(),
                d.income.iter().filter(|accepted| accepted.id == entry.id).flat_map(|accepted| {
                    crate::investment_credit::k1_property_ids(&accepted.income)
                }),
                &mut draft.next_id,
            )
            .map_err(|(key, _, index)| {
                let issue = if matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. }) {
                    // The estate K-1 carries its property rows on its attached statements.
                    ValidationIssue::new(key)
                        .at_field("property")
                        .in_list("investment_property", index)
                        .in_field("statements")
                } else {
                    ValidationIssue::new(key)
                        .at_list_item("investment_property", index)
                        .in_field(if matches!(&entry.income, FederalIncome::Partnership { data } if data.source.is_some()) {
                            "statements"
                        } else {
                            "more"
                        })
                };
                submitted_data_answer_issue(issue, original_income_positions[entry_index].0)
            })?;
        }
        for (entry_index, entry) in entries.iter_mut().enumerate() {
            assign_record_ids(
                crate::amortization_records::ids_mut(&mut entry.income).into_iter(),
                d.income.iter().filter(|accepted| accepted.id == entry.id).flat_map(|accepted| {
                    crate::amortization_records::records(&accepted.income).iter().map(|row| row.id)
                }),
                &mut draft.next_id,
            )
            .map_err(|(key, _, index)| {
                submitted_data_answer_issue(
                    ValidationIssue::new(key)
                        .at_field("id")
                        .in_list("intangibles", index)
                        .in_field("more"),
                    original_income_positions[entry_index].0,
                )
            })?;
        }
        for (entry_index, entry) in entries.iter_mut().enumerate() {
            assign_record_ids(
                crate::BusinessMileage::ids_mut(&mut entry.income).into_iter(),
                d.income.iter().filter(|accepted| accepted.id == entry.id).flat_map(|accepted| {
                    crate::BusinessMileage::records(&accepted.income).iter().map(|row| row.id)
                }),
                &mut draft.next_id,
            )
            .map_err(|(key, _, index)| {
                submitted_data_answer_issue(
                    ValidationIssue::new(key).at_field("id").in_list("vehicles", index),
                    original_income_positions[entry_index].0,
                )
            })?;
        }
        for (entry_index, entry) in entries.iter_mut().enumerate() {
            if let Some(issue) = crate::macrs_records::building_facts_issue(
                crate::macrs_records::improvement_buildings(&entry.income),
            ) {
                return Err(submitted_data_answer_issue(
                    issue,
                    original_income_positions[entry_index].0,
                )
                .into());
            }
            assign_record_ids(
                crate::macrs_records::improvement_building_ids_mut(&mut entry.income).into_iter(),
                d.income.iter().filter(|accepted| accepted.id == entry.id).flat_map(|accepted| {
                    crate::macrs_records::improvement_buildings(&accepted.income)
                        .iter()
                        .map(|row| row.id)
                }),
                &mut draft.next_id,
            )
            .map_err(|(key, _, index)| {
                submitted_data_answer_issue(
                    ValidationIssue::new(key)
                        .at_field("id")
                        .in_list("improvement_buildings", index),
                    original_income_positions[entry_index].0,
                )
            })?;
        }
        for (entry_index, entry) in entries.iter_mut().enumerate() {
            assign_record_ids(
                crate::rental_energy_records::ids_mut(&mut entry.income).into_iter(),
                d.income
                    .iter()
                    .filter(|accepted| accepted.id == entry.id)
                    .flat_map(|accepted| crate::rental_energy_records::ids(&accepted.income)),
                &mut draft.next_id,
            )
            .map_err(|(key, _, _)| {
                submitted_data_answer_issue(
                    ValidationIssue::new(key).at_field("building"),
                    original_income_positions[entry_index].0,
                )
            })?;
        }
        for (entry_index, entry) in entries.iter_mut().enumerate() {
            assign_record_ids(
                crate::depreciation::income_asset_ids_mut(&mut entry.income).into_iter(),
                d.income
                    .iter()
                    .filter(|accepted| accepted.id == entry.id)
                    .flat_map(|accepted| crate::depreciation::income_asset_ids(&accepted.income)),
                &mut draft.next_id,
            )
            .map_err(|(key, _, index)| {
                let submitted = original_income_positions[entry_index].0;
                let mut issue = submitted_asset_rows
                    .iter()
                    .find(|(position, _)| *position == submitted)
                    .and_then(|(_, rows)| rows.get(index))
                    .cloned()
                    .unwrap_or_else(|| ValidationIssue::new(key).at_field("document"));
                issue.key = key;
                submitted_data_answer_issue(issue, submitted)
            })?;
        }
        for (entry_index, entry) in entries.iter_mut().enumerate() {
            assign_record_ids(
                crate::k1_items::k1_sale_ids_mut(&mut entry.income).into_iter(),
                d.income
                    .iter()
                    .filter(|accepted| accepted.id == entry.id)
                    .flat_map(|accepted| crate::k1_items::k1_sale_ids(&accepted.income)),
                &mut draft.next_id,
            )
            .map_err(|(key, _, index)| {
                submitted_data_answer_issue(
                    ValidationIssue::new(key)
                        .at_list_item("section179_sales", index)
                        .in_field(if matches!(&entry.income, FederalIncome::Partnership { data } if data.source.is_some()) {
                            "statements"
                        } else {
                            "more"
                        }),
                    original_income_positions[entry_index].0,
                )
            })?;
        }
        for entry in &mut entries {
            if let FederalIncome::Partnership { data } = &mut entry.income
                && let Some(mut source) = data.source.take()
            {
                source.bind_accepted_child_ids(data);
                data.source = Some(source);
            }
        }
        // A farm's CCC loans carry their own identities, which a Form 1099-A
        // names: new loans receive one, kept loans keep theirs wherever they
        // moved in the list, and a loan of another section is refused.
        assign_record_ids(
            entries.iter_mut().flat_map(|r| crate::farm::ccc_lot_ids_mut(&mut r.income)),
            d.income
                .iter()
                .filter(|r| source_in_editor(d, r, topic))
                .flat_map(|r| crate::farm::ccc_lot_ids(&r.income)),
            &mut draft.next_id,
        )
        .map_err(|(key, _, _)| status(key))?;
        for (entry_index, entry) in entries.iter_mut().enumerate() {
            assign_record_ids(
                crate::farm_specials::land_ids_mut(&mut entry.income).into_iter(),
                d.income
                    .iter()
                    .filter(|accepted| accepted.id == entry.id)
                    .flat_map(|accepted| crate::farm_specials::parcel_ids(&accepted.income)),
                &mut draft.next_id,
            )
            .map_err(|(key, _, index)| {
                submitted_data_answer_issue(
                    ValidationIssue::new(key).at_list_item("land_basis", index).in_field("more"),
                    original_income_positions[entry_index].0,
                )
            })?;
        }
        for (entry_index, entry) in entries.iter_mut().enumerate() {
            assign_record_ids(
                crate::farm_specials::sale_ids_mut(&mut entry.income).into_iter(),
                d.income
                    .iter()
                    .filter(|accepted| accepted.id == entry.id)
                    .flat_map(|accepted| crate::farm_specials::sale_ids(&accepted.income)),
                &mut draft.next_id,
            )
            .map_err(|(key, _, index)| {
                submitted_data_answer_issue(
                    ValidationIssue::new(key).at_list_item("land_sales", index).in_field("more"),
                    original_income_positions[entry_index].0,
                )
            })?;
        }
        for entry in &mut entries {
            assign_record_ids(
                crate::method_change::records_mut(&mut entry.income)
                    .into_iter()
                    .flat_map(|records| records.iter_mut().map(|record| &mut record.id)),
                d.income.iter().filter(|accepted| accepted.id == entry.id).flat_map(|accepted| {
                    crate::method_change::records(&accepted.income)
                        .into_iter()
                        .flat_map(|records| records.iter().map(|record| record.id))
                }),
                &mut draft.next_id,
            )
            .map_err(|(key, _, _)| status(key))?;
        }
        for (entry_index, entry) in entries.iter_mut().enumerate() {
            assign_record_ids(
                crate::farm_specials::production_ids_mut(&mut entry.income).into_iter(),
                d.income
                    .iter()
                    .filter(|accepted| accepted.id == entry.id)
                    .flat_map(|accepted| crate::farm_specials::production_ids(&accepted.income)),
                &mut draft.next_id,
            )
            .map_err(|(key, _, cohort_index)| {
                submitted_data_answer_issue(
                    ValidationIssue::new(key)
                        .at_list_item("production", cohort_index)
                        .in_field("specials")
                        .in_field("more"),
                    original_income_positions[entry_index].0,
                )
            })?;
        }
        let submitted: alloc::collections::BTreeSet<_> =
            entries.iter().map(|entry| entry.id).collect();
        draft
            .income
            .retain(|entry| !source_in_editor(d, entry, topic) || submitted.contains(&entry.id));
        for entry in entries {
            if let Some(accepted) = draft.income.iter_mut().find(|old| old.id == entry.id) {
                *accepted = entry;
            } else {
                draft.income.push(entry);
            }
        }
        let aggregation_sources = crate::qbi_aggregation::source_views(&draft, d);
        if calculation_input || topic == TaxSection::Documents {
            crate::qbi_aggregation::refresh_derived(&mut aggregation_pages, &aggregation_sources);
        }
        let aggregation_facts =
            crate::qbi_aggregation::accept_pages(&aggregation_pages, &aggregation_sources)
                .map_err(|issue| issue.in_field("documents"))?;
        if let Some(screening) = draft.screening.as_mut() {
            screening.qbi_aggregations = aggregation_facts;
        } else if !aggregation_facts.is_empty() {
            return Err(crate::validation::ValidationIssue::new("QBI_AGGREGATION_SOURCES")
                .at_field("qbi_aggregations")
                .in_field("documents")
                .into());
        }
        if let Some(key) = crate::qbi_aggregation::history_transition_rule(d, &draft) {
            return Err(crate::validation::ValidationIssue::new(key)
                .at_field("qbi_aggregations")
                .in_field("documents")
                .into());
        }
        if let Some((id, issue)) = crate::information_returns::receipt_issue(&draft) {
            return Err(submitted_source_issue(
                id,
                issue.map_field_prefix(&["activity", "returns"], &["returns"]),
                &submitted_origins,
            )
            .into());
        }
        if let Some((id, issue)) = crate::farm_conservation::method_issue(&draft) {
            return Err(submitted_source_issue(
                id,
                issue.map_field_prefix(&["conservation"], &["more", "conservation"]),
                &submitted_origins,
            )
            .into());
        }
        if let Some((id, issue)) = crate::employee_wages_admission::return_issue(&draft) {
            return Err(submitted_source_issue(id, issue, &submitted_origins).into());
        }
        if let Some((id, issue)) = crate::amortization::identity_issue(&draft) {
            return Err(submitted_source_issue(id, issue, &submitted_origins).into());
        }
        if let Some((id, issue)) = crate::cohorts::source_issue(&draft) {
            return Err(submitted_source_issue(id, issue, &submitted_origins).into());
        }
        if let Some((id, issue)) = crate::amortization::timber_source_issue(&draft) {
            return Err(submitted_source_issue(id, issue, &submitted_origins).into());
        }
        if let Some(failure) =
            crate::estate_trust::graph_issue_with_mode(&draft, topic != TaxSection::Documents)
        {
            return Err(submitted_graph_issue(failure, &submitted_origins, false).into());
        }
        prepare_cooperative_source_ids(&mut draft, d)
            .map_err(|failure| submitted_graph_issue(failure, &submitted_origins, true))?;
        // Physical answers are checked after their keyed followups are applied.
        // Records-only availability probes must leave their pending issues alone.
        let cooperative_sources = submitted_origins.iter().map(|(id, _, _)| *id).collect();
        if topic != TaxSection::Documents
            && let Some(failure) =
                crate::cooperative::graph_issue_for_sources(&draft, &cooperative_sources)
        {
            return Err(submitted_graph_issue(failure, &submitted_origins, true).into());
        }
        if crate::extended_rules::professional_election_rule(&draft).is_some() {
            return Err(
                (libertas_formatted_text("ISSUE_PROFESSIONAL_ELECTION_ALL_RENTALS", &[])).into()
            );
        }
        if let Some(key) = crate::tax_credit_bonds::source_rule(&draft) {
            return Err((libertas_formatted_text(key, &[])).into());
        }
        // A component removed by its own topic no longer authorizes a retained
        // activity link. Use the normal pruning owner only for unchanged rows;
        // changed activity submissions still face exact membership validation.
        let related_center = crate::related_forms::catalog(&draft);
        for entry in &mut draft.income {
            if d.income.iter().any(|old| old.id == entry.id && old.income == entry.income) {
                walk::prune_entry_related(entry, &related_center);
            }
        }
        for &(id, submitted, _) in &submitted_origins {
            let Some(source) = crate::farm::sources(&draft).find(|source| source.entry.id == id)
            else {
                continue;
            };
            let center = crate::related_forms::catalog_for(&draft, id);
            let claimed = crate::related_forms::claimed_elsewhere(&draft, id);
            let invalid = if source.rental {
                crate::RelatedForms::validate::<crate::FarmRental>(
                    &source.farm.related,
                    &center,
                    &claimed,
                )
            } else {
                crate::RelatedForms::validate::<crate::Farm>(
                    &source.farm.related,
                    &center,
                    &claimed,
                )
            };
            if let Err(member) = invalid {
                let variant = if source.rental { "FarmRental" } else { "Farm" };
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new("PROBLEM_RELATED_FORM_UNAVAILABLE")
                        .at_field("key")
                        .in_list("forms", member)
                        .in_field("related")
                        .in_field(variant),
                    submitted,
                )
                .into());
            }
        }
        for &(id, submitted, _) in &submitted_origins {
            let Some(entry) = draft.income.iter().find(|e| e.id == id) else {
                continue;
            };
            if let Some(issue) = crate::related_forms::activity_issue(&draft, entry) {
                return Err(submitted_data_answer_issue(issue, submitted).into());
            }
        }

        if topic == TaxSection::Documents {
            // Printed corrections retain their evidence; topic associations
            // whose source or owner is gone return to the unlinked answer.
            while let Some((id, _)) = crate::savings_bonds::redemption_link_issue(&draft.income) {
                let entry = draft.income.iter_mut().find(|entry| entry.id == id).unwrap();
                let FederalIncome::BondRedemption { data } = &mut entry.income else {
                    unreachable!()
                };
                data.form_8815 = None;
            }
        }
        if let Some((id, issue)) = crate::savings_bonds::redemption_link_issue(&draft.income) {
            return Err(submitted_source_issue(id, issue, &submitted_origins).into());
        }
        if let Some((id, issue)) = crate::savings_bonds::source_issue(&draft.income) {
            return Err(submitted_source_issue(id, issue, &submitted_origins).into());
        }
        // The topic that owns an activity kind refuses its save while a saved
        // paper's amount still needs a receiving activity of that kind that no
        // record links; the estimate keeps the same issue for the Review.
        if !calculation_input
            && matches!(
                topic,
                TaxSection::Rentals | TaxSection::Farming | TaxSection::SelfEmployment
            )
            && let Some(unplaced) = crate::related_forms::unplaced(
                &draft,
                &crate::related_forms::effective_claimants(&draft),
            )
            .into_iter()
            .find(|unplaced| unplaced.owner() == Some(topic))
        {
            let key = match topic {
                TaxSection::Rentals => "ISSUE_RELATED_FORM_PAGE_RENTALS",
                TaxSection::Farming => "ISSUE_RELATED_FORM_PAGE_FARMING",
                _ => "ISSUE_RELATED_FORM_PAGE_BUSINESS",
            };
            let issue = ValidationIssue::new(key).at_field("documents");
            return Err(SaveFailure {
                message: crate::Message::new(
                    key,
                    alloc::vec![crate::MessageArg::Encoded(unplaced.component.part)],
                )
                .encode(),
                field: issue.field,
            });
        }
        // ASSET_BONUS_ELECTION_BY_CLASS: the bonus election out applies to a
        // whole recovery class across the return (businesses and rented
        // equipment alike); a list of assets cannot express that on the client.
        if let Some((first, second)) = crate::depreciation::election_conflict(&draft) {
            // Prefer the second conflicting source when it is posted; otherwise the
            // first posted source in this exact pair. Employee/retained-only pairs
            // have no editable asset in this rejected Income request.
            for (id, asset_index) in second.into_iter().chain(first) {
                if let Some((_, submitted, _)) =
                    submitted_origins.iter().find(|(source, _, _)| *source == id)
                    && let Some((_, rows)) =
                        submitted_asset_rows.iter().find(|(index, _)| index == submitted)
                    && let Some(issue) = rows.get(asset_index)
                {
                    return Err(submitted_data_answer_issue(issue.clone(), *submitted).into());
                }
            }
            return Err(status("ASSET_BONUS_ELECTION_BY_CLASS").into());
        }
        let has_foreign_tax = draft.income.iter().any(|entry| match &entry.income {
            FederalIncome::Interest { data } => crate::FederalInterest::from(data).foreign_tax > 0,
            FederalIncome::Dividend { data } => data.form.foreign_tax.unwrap_or(0) > 0,
            FederalIncome::EstateTrustPortfolio { data } => {
                let data = &crate::FederalEstateTrustPortfolioK1::from(data);
                data.foreign_statement.as_ref().is_some_and(|v| {
                    v.tax > 0
                        && v.treatment
                            == crate::EstateForeignTreatment::QualifiedPassivePayeeStatement
                })
            }
            _ => false,
        });
        if matches!(
            topic,
            TaxSection::InterestDividends | TaxSection::PassThrough | TaxSection::Documents
        ) && let Some(credits) = draft.credits.as_mut()
        {
            // This topic starts and ends the payee-statement election only; the
            // 1099 amounts are on the documents. A Form 1116 claim chosen on
            // More credits stays as it is, and the section 904(j)(2)
            // eligibility answer is the person's (More credits): a 1099 proves
            // only its own box, not that all foreign income is passive and on
            // statements, so a new election starts unconfirmed and a confirmed
            // answer survives later saves.
            credits.foreign_tax =
                match core::mem::replace(&mut credits.foreign_tax, ForeignTaxClaim::No) {
                    claim @ (ForeignTaxClaim::Ordinary { .. }
                    | ForeignTaxClaim::Deduction { .. }) => claim,
                    claim @ ForeignTaxClaim::Statements { .. } if has_foreign_tax => claim,
                    ForeignTaxClaim::No if has_foreign_tax => {
                        ForeignTaxClaim::Statements { eligible: None }
                    }
                    _ => ForeignTaxClaim::No,
                };
        }
        if let Some(records) = spouse_179
            && let Some(setup) = draft.setup.as_mut()
            && let crate::FilingChoice::Separate { section179 } = &mut setup.status
        {
            *section179 = records;
        }
        if let Some(screening) = draft.screening.as_mut() {
            data_answers.apply(screening);
            if let Some(lookback) = lookback {
                screening.section1231 = lookback;
            }
            // The Farming page owns the farm optional method: one election
            // for each owner whose farms tick it (question F6).
            if topic == TaxSection::Farming {
                for election in &mut screening.optional_se {
                    election.farm = false;
                }
                for (owner, elected) in farm_se.iter().filter(|(_, elected)| *elected) {
                    if let Some(election) =
                        screening.optional_se.iter_mut().find(|e| e.owner == *owner)
                    {
                        election.farm = *elected;
                    } else {
                        screening.optional_se.push(SeElection {
                            owner: *owner,
                            farm: true,
                            nonfarm: None,
                        });
                    }
                }
                screening.optional_se.retain(|e| e.farm || e.nonfarm.is_some());
            }
        }
        if topic == TaxSection::Retirement {
            let canonical_repayments = d
                .adjustments
                .iter()
                .flat_map(|a| &a.ira_basis)
                .flat_map(|basis| {
                    basis
                        .special_repayments
                        .iter()
                        .cloned()
                        .map(move |payment| (basis.owner, payment))
                })
                .collect();
            let ira_basis =
                ira_basis_records(&adjustment_items, d.adjustments.as_ref(), canonical_repayments);
            let qualified_charity = qcd_records(&adjustment_items);
            if let Some(adjustments) = draft.adjustments.as_mut() {
                adjustments.ira_basis = ira_basis;
                adjustments.qualified_charity = qualified_charity;
                adjustments.social_election = social_election;
                adjustments.required_distributions = required_distributions;
                adjustments.roth_history = roth_history;
            }
        }
        if topic == TaxSection::Documents {
            // A paper correction changes no premium or repayment answer; only a removed source's go.
            apply_retirement_personal_records(
                &mut draft,
                &alloc::collections::BTreeSet::new(),
                Vec::new(),
                Vec::new(),
            );
        }
        if topic == TaxSection::Retirement {
            retirement_ira_basis(&mut draft.income, &form8606_owners);
        }
        if topic == TaxSection::Retirement {
            retirement_save_rules(
                &draft,
                &adjustment_items,
                &submitted_origins,
                &original_adjustment_positions,
                question_keys,
                None,
            )?;
        }
        if let Some(issue) = draft
            .screening
            .as_ref()
            .and_then(|screen| screen.farm_averaging.as_ref())
            .and_then(|facts| facts.sources_issue(&draft))
        {
            return Err(if let Some(index) = farm_averaging_position {
                submitted_data_answer_issue(issue, index).into()
            } else {
                // A retained election absent from this request has no editable field here.
                status(issue.key).into()
            });
        }
        if let Some((owner, issue)) = crate::foreign_housing::source_pay_issue(&draft) {
            if let Some((_, index)) =
                foreign_claim_positions.iter().find(|(person, _)| *person == owner)
            {
                return Err(submitted_data_answer_issue(issue, *index).into());
            }
            if let Some((_, field, _)) = foreign_pay_positions
                .iter()
                .find(|(person, _, reduced)| *person == owner && *reduced)
            {
                return Err(field.clone().into());
            }
            let mut posted = foreign_pay_positions.iter().filter(|(person, _, _)| *person == owner);
            if let Some((_, field, _)) = posted.next() {
                // A relation over several unchanged pay rows names their actual
                // submitted collection rather than inventing a causal row.
                return Err(if posted.next().is_none() {
                    field.clone()
                } else {
                    ValidationIssue::new(issue.key).at_field("documents").in_field("documents")
                }
                .into());
            }
            // Neither the owning claim nor a contributing pay source is in this request.
            return Err(status(issue.key).into());
        }
        if matches!(
            topic,
            TaxSection::SelfEmployment | TaxSection::Farming | TaxSection::InvestmentSales
        ) && let Some((origin, mut issue)) = crate::retained_research::issue(&draft)
        {
            if let crate::retained_research::RetainedIssueOrigin::Capital(index) = origin
                && topic == TaxSection::InvestmentSales
                && let Some(submitted) = capital_document_positions.get(index)
            {
                return Err(submitted_data_answer_issue(issue, *submitted).into());
            }
            if let crate::retained_research::RetainedIssueOrigin::Nol(index) = origin
                && matches!(topic, TaxSection::SelfEmployment | TaxSection::Farming)
                && let Some(submitted) = nol_document_position
            {
                return Err(submitted_data_answer_issue(
                    issue.in_list("regular", index),
                    submitted,
                )
                .into());
            }
            // Retained charity/NOL, or capital records not submitted by this topic,
            // keep their existing unlocated refusal. Never target an unrelated row.
            issue.field = None;
            return Err(issue.into());
        }
        // Document edits may remove components held by accepted activities. Their
        // references follow the normal walk before the retained graph is checked;
        // submitted activity keys are still validated against the raw candidate.
        let (accepted, _) = self.reconcile_values(draft.clone());
        if let Some(failure) = crate::extended_rules::rental_mortgage_issue(&accepted) {
            return Err(submitted_graph_issue(failure, &submitted_origins, false).into());
        }
        // Keep the source identity of linked corrections. Amount edits may
        // save and reopen the HSA review; deleting or changing the source's
        // owner/type requires removing its correction link first.
        if let Some(issue) = health_correction_income_issue(d, &accepted, &submitted_origins) {
            return Err(issue.into());
        }
        let ltc_issue = if topic == TaxSection::Documents {
            crate::care_benefit_return::source_reference_issue(d, &accepted)
        } else {
            crate::care_benefit_return::reference_issue(&accepted)
        };
        if let Some(failure) = ltc_issue {
            return Err(submitted_graph_issue(failure, &submitted_origins, false).into());
        }
        // A walked topic saved is visited: the walk passes it (M3-P19).
        visit(d, &mut draft, TaxSection::Income, topic);
        Ok(draft)
    }
    // The student loan interest page merged into the accepted return
    // (M3-P8; T2-33x): no form and no other interest is No; otherwise the
    // forms are stored as entered and the interest paid with no form is
    // StudentLoanClaim's own amount.
    fn document_answer_ids(
        accepted: &[i64],
        supplied: &[i64],
        field: &'static str,
    ) -> Result<(), SaveFailure> {
        if accepted.len() != supplied.len() {
            return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                .at_field(field)
                .in_field("value")
                .into());
        }
        for (index, id) in supplied.iter().enumerate() {
            if !accepted.contains(id) || supplied[..index].contains(id) {
                return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                    .at_field("id")
                    .in_list(field, index)
                    .in_field("value")
                    .into());
            }
        }
        Ok(())
    }
    // Preserve saved identities, allocate new ones in the candidate only and
    // validate the actual edited graph against accepted selector sources.
    fn mortgages_candidate(&self, value: &Mortgages) -> Result<FederalDraft, SaveFailure> {
        let candidate = self.candidate_editor(self.draft.clone());
        let mut sources = self
            .draft
            .deductions
            .as_ref()
            .map_or_else(Vec::new, |deductions| deductions.mortgage.clone());
        // Answered loans and new ones together: each saved loan once.
        let rows: Vec<(&'static str, usize, &crate::MortgageDocumentView)> = value
            .loans
            .iter()
            .enumerate()
            .map(|(index, row)| ("loans", index, row))
            .chain(value.new_loans.iter().enumerate().map(|(index, row)| ("new_loans", index, row)))
            .collect();
        Self::document_answer_ids(
            &sources.iter().map(|source| source.id).collect::<Vec<_>>(),
            &rows.iter().map(|(_, _, loan)| loan.id).collect::<Vec<_>>(),
            "loans",
        )?;
        for &(list, index, row) in &rows {
            let source = sources.iter_mut().find(|source| source.id == row.id).unwrap();
            let amount = source.form.box_6;
            match &row.points {
                crate::MortgagePointTreatment::None if amount == 0 => {}
                crate::MortgagePointTreatment::Purchase
                | crate::MortgagePointTreatment::Amortized { .. }
                | crate::MortgagePointTreatment::Improvement { .. }
                    if amount > 0 => {}
                _ => {
                    return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                        .at_field("points")
                        .in_list(list, index)
                        .in_field("value")
                        .into());
                }
            }
            source.answered = true;
            source.points = row.points.clone();
            source.property = row.property.clone();
            source.lender = row.lender.clone();
            if source.form.box_4 == 0 && row.refund_origin.is_some() {
                return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                    .at_field("refund_origin")
                    .in_list(list, index)
                    .in_field("value")
                    .into());
            }
            // Answered on the questions page: the page only echoes it.
            if row.refund_origin != source.refund_origin {
                return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                    .at_field("refund_origin")
                    .in_list(list, index)
                    .in_field("value")
                    .into());
            }
            source.details = row.details.clone().into();
            source.paid_share = row.paid_share.clone();
            source.insurance_contract = row.insurance_contract.clone();
            source.debt_date = row.debt_date.clone();
        }
        let targets = sources
            .iter()
            .map(|source| {
                let (list, index, _) =
                    *rows.iter().find(|(_, _, row)| row.id == source.id).unwrap();
                ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                    .at_record()
                    .in_list(list, index)
                    .in_field("value")
                    .field
            })
            .collect::<Vec<_>>();
        candidate
            .mortgage_loans_candidate(&sources, true)
            .map_err(|error| error.map_submitted_items(&["value", "loans"], &targets))
    }
    // Keep the document conversion temporaries off unrelated dispatch paths.
    // IncomeSource contains inline K-1 data; reserving this arm's frame during
    // navigation needlessly retained it across nested estimate passes.
    fn preview_document_with_path(
        &self,
        id: i64,
        owner: Owner,
        community: CommunityShare,
        document: IncomeSource,
        has_value_field: bool,
    ) -> Option<Reply> {
        // Pure calculation: no revision is an input. Client
        // transaction ownership rejects stale replies after edits.
        if let IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(v) } =
            &document
        {
            let mut source: crate::PriorCapitalReturn = v.clone().into();
            source.investment_parts = None;
            source.injured_owners = None;
            if !crate::extended_rules::valid_capital_history(&[source]) {
                return Some(problem_key("INVESTMENT_CAPITAL_SOURCE"));
            }
            return Some(Reply::Response(P::IncomePreview {
                preview: None,
                investment_caps: Some(crate::extended_rules::capital_caps(&v.clone().into())),
            }));
        }
        if let Some(issue) = (match &document {
            IncomeSource::Record { value: crate::IncomeRecord::MainHomeSale { sale, answers } } => {
                home_sale_record_issue(sale, answers, self.joint())
            }
            _ => document_issue(&document),
        })
        .or_else(|| crate::broker_page::year_issue(&document, self.draft.tax_year))
        .or_else(|| rental_casualty_year_issue(&document, self.draft.tax_year))
        .or_else(|| crate::care_benefit_return::document_issue(&self.draft, owner, &document))
        {
            return Some(refused_source(issue.in_field("document"), has_value_field));
        }
        if let IncomeSource::Record { value: crate::IncomeRecord::InstallmentSale(sale) } =
            &document
        {
            let amount = if sale.refusal().is_some() {
                None
            } else {
                let full = sale.full_sale();
                let Ok(calculation) = sale.calculation() else {
                    return Some(refused_source(
                        crate::validation::ValidationIssue::new("INSTALLMENT_AMOUNT_OVERFLOW")
                            .at_field("price")
                            .in_field("InstallmentSale")
                            .in_field("document"),
                        has_value_field,
                    ));
                };
                let gain = calculation.map_or_else(
                    || {
                        let gain = full.gain();
                        if sale.buyer != crate::Buyer::Unrelated { gain.max(0) } else { gain }
                    },
                    |current| current.gain() * 100,
                );
                let income = FederalIncome::Sale { data: crate::SaleSource::from_sale(&full, "") };
                Some(
                    self.draft
                        .setup
                        .as_ref()
                        .and_then(|setup| crate::community::share(setup, owner, community, &income))
                        .map_or(gain, |share| share.gain(gain)),
                )
            };
            return Some(Reply::Response(P::IncomePreview {
                investment_caps: None,
                preview: amount,
            }));
        }
        // A farm shows its Schedule F line 34, figured on the
        // accepted return with this farm in place (T2-45p); the
        // row's identity and owner place it.
        if let IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } = &document {
            return Some(Reply::Response(P::IncomePreview {
                investment_caps: None,
                preview: self.farm_preview(id, owner, farm),
            }));
        }
        // A farm rental shows its Form 4835 line 32 or 34c the same way.
        if let IncomeSource::Record { value: crate::IncomeRecord::FarmRental(rental) } = &document {
            return Some(Reply::Response(P::IncomePreview {
                investment_caps: None,
                preview: self.farm_rental_preview(id, owner, rental),
            }));
        }
        // Form 8606 and a QCD gift show no amount: neither is a
        // standalone income entry, and the 8606 owner-basis override
        // needs the whole page, not one document.
        // A business shows none either: its profit depends on the
        // whole return (the MACRS convention, the home-office limit).
        // Nor do the data answers (M3-P15): they are facts about
        // the return, not income.
        if matches!(
            document,
            IncomeSource::Record { value: crate::IncomeRecord::Business(_) }
                | adjustment_kinds!()
                | data_answer_kinds!()
        ) {
            return Some(Reply::Response(P::IncomePreview {
                investment_caps: None,
                preview: None,
            }));
        }
        let automatic_cooperative = matches!(&document, IncomeSource::Farming(_));
        let income = document_income(&document);
        if let Err((issue, _)) = crate::federal_rules::validate_income_amount_issue_for_year(
            &income,
            u32::try_from(self.draft.tax_year).unwrap_or(0),
        ) {
            let issue = if matches!(income, FederalIncome::Cooperative { .. }) {
                project_cooperative_issue(issue, automatic_cooperative)
            } else {
                issue
            };
            return Some(refused_source(issue.in_field("document"), has_value_field));
        }
        Some(Reply::Response(P::IncomePreview {
            investment_caps: None,
            preview: {
                let entry = document_entry(
                    IncomeDocumentItem {
                        refund_management: None,
                        refund_management_needed: None,
                        allowed_community: Vec::new(),
                        investment_caps: None,
                        id,
                        owner: crate::DocumentOwner::from(owner),
                        document,
                        preview: None,
                        community,
                    },
                    &self.draft.income,
                );
                let mut candidate = self.draft.clone();
                candidate.income.retain(|old| id == 0 || old.id != id);
                let is_cooperative = matches!(entry.income, FederalIncome::Cooperative { .. });
                let bound_index = candidate.income.len();
                candidate.income.push(entry);
                if let Err((_, issue)) = related_forms::prepare_component_ids(
                    core::slice::from_mut(&mut candidate.income[bound_index]),
                    &self.draft.income,
                    &mut candidate.next_id,
                ) {
                    return Some(refused_source(issue, has_value_field));
                }
                if matches!(&candidate.income[bound_index].income, FederalIncome::Rental { data } if data.buildings.iter().any(|cost| cost.energy.is_some()))
                    && let Err((key, _, _)) = assign_record_ids(
                        crate::rental_energy_records::ids_mut(
                            &mut candidate.income[bound_index].income,
                        )
                        .into_iter(),
                        self.draft
                            .income
                            .iter()
                            .filter(|old| old.id == id)
                            .flat_map(|old| crate::rental_energy_records::ids(&old.income)),
                        &mut candidate.next_id,
                    )
                {
                    return Some(problem_key(key));
                }
                // New K-1 (1041) activities take provisional IDs from the disposable
                // candidate, as Save assigns them, so the graph check sees saved-looking rows.
                if let Err((key, _, _)) = assign_record_ids(
                    crate::estate_trust::activity_ids_mut(
                        &mut candidate.income[bound_index].income,
                    ),
                    self.draft
                        .income
                        .iter()
                        .filter(|old| old.id == id)
                        .flat_map(|old| crate::estate_trust::activity_ids(&old.income)),
                    &mut candidate.next_id,
                ) {
                    return Some(problem_key(key));
                }
                if let Some(failure) = crate::estate_trust::graph_issue(&candidate) {
                    return Some(refused_source(
                        preview_graph_issue(
                            failure,
                            candidate.income[bound_index].id,
                            false,
                            false,
                        ),
                        has_value_field,
                    ));
                }
                if is_cooperative {
                    // The normal allocator acts only on this disposable
                    // candidate; Preview cannot consume accepted IDs.
                    if let Err((key, _, _)) = assign_record_ids(
                        candidate.income.iter_mut().map(|e| &mut e.id),
                        self.draft.income.iter().map(|e| e.id),
                        &mut candidate.next_id,
                    ) {
                        return Some(problem_key(key));
                    }
                    if let Err(failure) =
                        prepare_cooperative_source_ids(&mut candidate, &self.draft)
                    {
                        return Some(refused_source(
                            preview_graph_issue(
                                failure,
                                candidate.income[bound_index].id,
                                true,
                                automatic_cooperative,
                            ),
                            has_value_field,
                        ));
                    }
                    if let Some(failure) = crate::cooperative::graph_issue(&candidate) {
                        return Some(refused_source(
                            preview_graph_issue(
                                failure,
                                candidate.income[bound_index].id,
                                true,
                                automatic_cooperative,
                            ),
                            has_value_field,
                        ));
                    }
                }
                if let Some(issue) =
                    crate::related_forms::activity_issue(&candidate, &candidate.income[bound_index])
                {
                    return Some(refused_source(issue.in_field("document"), has_value_field));
                }
                if let Some(failure) = crate::extended_rules::rental_mortgage_issue(&candidate) {
                    return Some(refused_source(
                        preview_graph_issue(
                            failure,
                            candidate.income[bound_index].id,
                            false,
                            false,
                        ),
                        has_value_field,
                    ));
                }
                if let Some(failure) = crate::care_benefit_return::reference_issue(&candidate) {
                    return Some(refused_source(
                        preview_graph_issue(
                            failure,
                            candidate.income[bound_index].id,
                            false,
                            false,
                        ),
                        has_value_field,
                    ));
                }
                let mut notices = Vec::new();
                let supported = crate::unknowns::supported(&candidate, &mut notices);
                let shown = supported.as_ref().unwrap_or(&candidate);
                shown
                    .income
                    .iter()
                    .find(|entry| {
                        entry.id == id
                            || (id == 0
                                && is_cooperative
                                && matches!(entry.income, FederalIncome::Cooperative { .. })
                                && !self.draft.income.iter().any(|old| old.id == entry.id))
                    })
                    .map_or(Some(0), |entry| {
                        community_amount(shown, entry, preview_amount(shown, entry))
                    })
            },
        }))
    }
    pub(crate) fn admit_request(
        &mut self,
        request: &P,
    ) -> Result<(), Option<alloc::boxed::Box<Reply>>> {
        self.accepted_save = false;
        // The Hub has already verified the field against the session; this
        // guards the App's own dispatch from routing a request to another
        // user's return.
        if request.user() != Some(self.user) {
            return Err(None);
        }
        let identity = match request {
            P::OpenInterview { .. }
            | P::Enter { .. }
            | P::ReviewAccepted { .. }
            | P::AssetMacrsSourceOptions { .. }
            | P::AssetMacrsClassLifeOptions { .. }
            | P::PropertyMacrsSourceOptions { .. }
            | P::PropertyMacrsClassLifeOptions { .. }
            | P::AssetAmtCarryOptions { .. }
            | P::VehicleAmtCarryOptions { .. }
            | P::NonMacrsAssetCarryOptions { .. }
            | P::IntangibleSourceOptions { .. }
            | P::NonMacrsCanonicalCarryOptions { .. }
            | P::CanonicalAssetAmtCarryOptions { .. }
            | P::RentalRecoveryOptions { .. }
            | P::DocumentPropertyOptions { .. }
            | P::HobbyRelatedOptions { .. }
            | P::EquipmentRelatedOptions { .. }
            | P::RoyaltyRelatedOptions { .. }
            | P::FarmRelatedOptions { .. }
            | P::FarmRentalRelatedOptions { .. }
            | P::RentalRelatedOptions { .. }
            | P::AnswerQsbsZone { .. }
            | P::AnswerQsbsValuation { .. }
            | P::AnswerQsbsBoolean { .. }
            | P::AnswerQsbsPriorGain { .. }
            | P::AnswerRentalRisk { .. }
            | P::ProjectRelatedForm { .. }
            | P::ViewRelatedForm { .. }
            | P::ViewReviewedRelatedForm { .. }
            | P::PreviewDocument { .. }
            | P::EstimateSetup { .. }
            | P::EstimateYou { .. }
            | P::EstimateSpouse { .. }
            | P::EstimateDocuments { .. }
            | P::EstimateWagesAnswers { .. }
            | P::DocumentOwnershipOptions { .. }
            | P::EstimateIncomeWithoutForm { .. }
            | P::EstimateOftenMissed { .. }
            | P::EstimateDependents { .. }
            | P::EstimateHsa { .. }
            | P::EstimateSelfEmployedBenefits { .. }
            | P::EstimateIraContributions { .. }
            | P::EstimateEducator { .. }
            | P::EstimateStudentLoan { .. }
            | P::EstimateEmployeeExpenses { .. }
            | P::EstimateDeductions { .. }
            | P::EstimateMedical { .. }
            | P::EstimateTaxes { .. }
            | P::EstimateOtherItemized { .. }
            | P::EstimateMortgages { .. }
            | P::EstimateHomeOffices { .. }
            | P::EstimateGiftCarryovers { .. }
            | P::EstimateCare { .. }
            | P::EstimateEducation { .. }
            | P::EstimateSaver { .. }
            | P::EstimateMarketplace { .. }
            | P::EstimateMoreCredits { .. }
            | P::EstimateOtherTaxes { .. }
            | P::EstimateMoreAdjustments { .. }
            | P::EstimatePayments { .. }
            | P::EstimateIncomeByPeriod { .. }
            | P::EstimateInjuredSpouse { .. }
            | P::EstimateCommunityProperty { .. }
            | P::EstimateSpouseBoxes { .. }
            | P::EstimateSpouseItemizes { .. }
            | P::EstimateParentTax { .. }
            | P::EstimateChecklist { .. }
            | P::HsaOptions { .. }
            | P::CalculateMarketplaceAppeal { .. }
            | P::BusinessCreditElectionOptions { .. }
            | P::ResearchCreditSourceOptions { .. }
            | P::RetainedLossSourceOptions { .. }
            | P::RetainedCharityAppreciationOptions { .. }
            | P::RetainedCapitalSourceOptions { .. }
            | P::InvestmentOriginOptions { .. } => None,
            P::SavePaymentParts { cookie, revision, .. }
            | P::BusinessRelatedOptions { cookie, revision, .. }
            | P::OpenCropInsuranceFacts { cookie, revision, .. }
            | P::SaveCropInsuranceTiming { cookie, revision, .. }
            | P::CloseCropInsuranceFacts { cookie, revision, .. }
            | P::OpenCropPaperFacts { cookie, revision, .. }
            | P::SaveCropPaperFacts { cookie, revision, .. }
            | P::OpenCropPractice { cookie, revision, .. }
            | P::SaveCropPractice { cookie, revision, .. }
            | P::K1InvestmentOptions { cookie, revision, .. }
            | P::SaveSetup { cookie, revision, .. }
            | P::SavePeople { cookie, revision, .. }
            | P::SaveSpouse { cookie, revision, .. }
            | P::SaveDeductions { cookie, revision, .. }
            | P::SaveIncomeWithoutForm { cookie, revision, .. }
            | P::SaveOftenMissed { cookie, revision, .. }
            | P::SaveChildren { cookie, revision, .. }
            | P::SaveDocuments { cookie, revision, .. }
            | P::SaveIncomes { cookie, revision, .. }
            | P::SaveDocumentQuestions { cookie, revision, .. }
            | P::SaveWagesAnswers { cookie, revision, .. }
            | P::SaveHsa { cookie, revision, .. }
            | P::SaveCare { cookie, revision, .. }
            | P::SaveSelfEmployedBenefits { cookie, revision, .. }
            | P::SaveIraContributions { cookie, revision, .. }
            | P::SaveEducator { cookie, revision, .. }
            | P::SaveStudentLoan { cookie, revision, .. }
            | P::SaveEmployeeExpenses { cookie, revision, .. }
            | P::SaveMedical { cookie, revision, .. }
            | P::SaveTaxes { cookie, revision, .. }
            | P::SaveOtherItemized { cookie, revision, .. }
            | P::SaveMortgages { cookie, revision, .. }
            | P::SaveHomeOffices { cookie, revision, .. }
            | P::SaveGiftCarryovers { cookie, revision, .. }
            | P::SaveEducation { cookie, revision, .. }
            | P::SaveSaver { cookie, revision, .. }
            | P::SaveMarketplace { cookie, revision, .. }
            | P::SaveMoreCredits { cookie, revision, .. }
            | P::SaveOtherTaxes { cookie, revision, .. }
            | P::SaveMoreAdjustments { cookie, revision, .. }
            | P::SavePayments { cookie, revision, .. }
            | P::SaveIncomeByPeriod { cookie, revision, .. }
            | P::SaveInjuredSpouse { cookie, revision, .. }
            | P::SaveCommunityProperty { cookie, revision, .. }
            | P::SaveSpouseBoxes { cookie, revision, .. }
            | P::SaveSpouseItemizes { cookie, revision, .. }
            | P::SaveParentTax { cookie, revision, .. }
            | P::SaveChecklist { cookie, revision, .. }
            | P::Back { cookie, revision, .. }
            | P::Navigate { cookie, revision, .. }
            | P::OpenSection { cookie, revision, .. }
            | P::OpenDocumentQuestions { cookie, revision, .. }
            | P::OpenHub { cookie, revision, .. }
            | P::ContinueIncome { cookie, revision, .. }
            | P::ContinueAdjustments { cookie, revision, .. }
            | P::ContinueItemized { cookie, revision, .. }
            | P::ContinueCredits { cookie, revision, .. }
            | P::StartFinish { cookie, revision, .. }
            | P::Finish { cookie, revision, .. } => Some((cookie, *revision)),
            _ => return Err(None),
        };
        if self.blocked {
            return Err(Some(alloc::boxed::Box::new(problem_key("PROBLEM_STORED_DATA_INVALID"))));
        }
        if let Some((cookie, revision)) = identity {
            if cookie != &self.draft.cookie {
                return Err(Some(alloc::boxed::Box::new(problem_key(
                    "PROBLEM_RETURN_IDENTITY_UNAVAILABLE",
                ))));
            }
            // Back and section navigation are identity-based; mutations, selections and previews are revision-based.
            if !matches!(
                request,
                P::Back { .. } | P::Navigate { .. } | P::OpenSection { .. } | P::OpenHub { .. }
            ) && revision != self.draft.revision
            {
                return Err(Some(alloc::boxed::Box::new(problem_key(
                    "PROBLEM_ACCEPTED_ANSWERS_CHANGED",
                ))));
            }
        }
        if let P::SaveDocuments { documents, .. } = request {
            if !self.reached(CHILDREN_PAGE) {
                return Err(Some(alloc::boxed::Box::new(problem_key(
                    "PROBLEM_SECTION_UNAVAILABLE_YET",
                ))));
            }
            if let Err(failure) = self.document_center_admission(documents) {
                return Err(Some(alloc::boxed::Box::new(refused(failure))));
            }
        }
        Ok(())
    }
    pub(crate) fn handle(&mut self, request: P) -> Option<Reply> {
        if let Err(reply) = self.admit_request(&request) {
            return reply.map(|reply| *reply);
        }
        self.handle_admitted(request)
    }
    fn handle_admitted(&mut self, request: P) -> Option<Reply> {
        if let Some(reply) = self.handle_crop_insurance(&request) {
            return Some(reply);
        }
        let d = &self.draft;
        Some(match request {
            P::Enter { .. } => Reply::Response(P::Actions {
                allowed_actions: alloc::vec![
                    libertas_macros::variant_index!(P::OpenInterview) as i32
                ],
            }),
            P::ReviewAccepted { .. } => {
                if d.setup.is_none() {
                    problem_key("PROBLEM_INTERVIEW_NOT_STARTED")
                } else {
                    self.review()
                }
            }
            P::OpenInterview { .. } => {
                if d.finished {
                    self.review()
                } else {
                    self.resume()
                }
            }
            P::Back { previous, .. } => self.page(previous, false),
            P::ContinueIncome { .. } => self.continue_section(TaxSection::Income),
            P::ContinueAdjustments { .. } => self.continue_section(TaxSection::Adjustments),
            P::ContinueItemized { .. } => self.continue_section(TaxSection::Itemized),
            P::ContinueCredits { .. } => self.continue_section(TaxSection::Credits),
            P::Navigate { from, section, .. } => {
                // An echoed allowed set cannot authorize a wrong group or leaf.
                let leaf = section.section();
                if section.group_index() != NavigationDestination::from(leaf).group_index() {
                    problem_key("PROBLEM_SECTION_UNAVAILABLE_HERE")
                } else {
                    self.navigate(from, leaf)
                }
            }
            P::OpenSection { from, section, .. } => self.navigate(from, section),
            // Reloads a hub as it is now; a page that is not a hub has no
            // topics to reload.
            P::OpenHub { page, review, .. } => {
                if HUB_PAGES.contains(&page) {
                    self.page(page, review)
                } else {
                    problem_key("PROBLEM_SECTION_UNAVAILABLE_HERE")
                }
            }
            P::SaveDocumentQuestions { .. } => self.save_document_questions(request),
            P::OpenDocumentQuestions { topic, review, .. } => {
                self.open_document_questions(topic, review)
            }
            P::SaveSetup { .. }
            | P::SavePeople { .. }
            | P::SaveSpouse { .. }
            | P::SaveDeductions { .. }
            | P::SaveIncomeWithoutForm { .. }
            | P::SaveOftenMissed { .. }
            | P::SaveChecklist { .. }
            | P::EstimateChecklist { .. }
            | P::SaveChildren { .. }
            | P::SaveDocuments { .. }
            | P::SaveIncomes { .. }
            | P::SaveWagesAnswers { .. }
            | P::SaveHsa { .. }
            | P::SaveCare { .. }
            | P::SaveSelfEmployedBenefits { .. }
            | P::SaveIraContributions { .. }
            | P::SaveEducator { .. }
            | P::SaveStudentLoan { .. }
            | P::SaveEmployeeExpenses { .. }
            | P::SaveMedical { .. }
            | P::SaveTaxes { .. }
            | P::SaveOtherItemized { .. }
            | P::SaveMortgages { .. }
            | P::SaveHomeOffices { .. }
            | P::SaveGiftCarryovers { .. }
            | P::SaveEducation { .. }
            | P::SaveSaver { .. }
            | P::SaveMarketplace { .. }
            | P::SaveMoreCredits { .. }
            | P::SaveOtherTaxes { .. }
            | P::SaveMoreAdjustments { .. }
            | P::SavePayments { .. }
            | P::SaveIncomeByPeriod { .. }
            | P::SaveInjuredSpouse { .. }
            | P::SaveCommunityProperty { .. }
            | P::SaveSpouseBoxes { .. }
            | P::SaveSpouseItemizes { .. }
            | P::SaveParentTax { .. }
            | P::StartFinish { .. }
            | P::Finish { .. } => return self.handle_save(request),
            _ => return self.handle_calculation(request),
        })
    }

    fn refresh_submitted_macrs_questions(&self, request: &mut P) -> bool {
        match request {
            P::SaveIncomes { documents, topic, page, warnings, review, .. }
                if INCOME_TOPICS.contains(topic) && *page == INCOME_PAGE && self.reached(*page) =>
            {
                let Ok(candidate) =
                    self.documents_candidate_inner(*topic, documents.clone().into_edits(), false)
                else {
                    return false;
                };
                let view = crate::macrs_source::MacrsElectionView::new(&candidate);
                let mut changed = false;
                for (row, item) in documents.documents.iter_mut().enumerate() {
                    let retained = self
                        .draft
                        .income
                        .iter()
                        .find(|e| e.id == item.id)
                        .is_some_and(|e| crate::interest_limit::source(&e.income).is_some());
                    if crate::farm_rental::refresh_property_questions(&mut item.document) {
                        warnings.push(crate::SaveWarning {
                            message: crate::Message::new(
                                "FARM_RENTAL_PROPERTY_QUESTIONS_UPDATED",
                                alloc::vec![],
                            )
                            .encode(),
                        });
                        changed = true;
                    }
                    if crate::interest_limit::refresh(&mut item.document, retained) {
                        warnings.push(crate::SaveWarning {
                            message: crate::Message::new(
                                "INTEREST_LIMIT_ELECTION_QUESTIONS_UPDATED",
                                alloc::vec![crate::MessageArg::Number((row + 1) as i64)],
                            )
                            .encode(),
                        });
                        changed = true;
                    }
                    for building in refresh_building_recovery_questions(&mut item.document, &view) {
                        warnings.push(crate::SaveWarning {
                            message: crate::Message::new(
                                "BUILDING_RECOVERY_SOURCE_QUESTIONS_UPDATED",
                                alloc::vec![
                                    crate::MessageArg::Number(row as i64 + 1),
                                    crate::MessageArg::Number(building as i64 + 1)
                                ],
                            )
                            .encode(),
                        });
                        changed = true;
                    }
                    for asset in refresh_record_macrs_questions(
                        &mut item.document,
                        item.owner,
                        item.id,
                        &view,
                    ) {
                        warnings.push(macrs_question_warning(false, row, asset));
                        changed = true;
                    }
                }
                if changed {
                    // Output-only choice sets are omitted from submitted requests.
                    // Rebuild the ordinary page shell before returning an editor;
                    // preserve every submitted answer and the new review warnings.
                    let Reply::Edit(mut refreshed) = self.topic_page(*topic, *review) else {
                        return false;
                    };
                    if let (
                        P::SaveIncomes {
                            documents,
                            followups,
                            warnings,
                            previous,
                            qbi_sources: submitted_qbi_sources,
                            ..
                        },
                        P::SaveIncomes {
                            documents: next_documents,
                            followups: next_followups,
                            warnings: next_warnings,
                            previous: next_previous,
                            qbi_sources,
                            ..
                        },
                    ) = (&mut *request, &mut refreshed)
                    {
                        core::mem::swap(next_documents, documents);
                        core::mem::swap(next_followups, followups);
                        core::mem::swap(next_warnings, warnings);
                        *next_previous = *previous;
                        if crate::qbi_aggregation::uses_facts(&candidate) {
                            *qbi_sources =
                                crate::qbi_aggregation::source_views(&candidate, &self.draft)
                                    .into_iter()
                                    .map(|source| source.member)
                                    .collect();
                        } else {
                            // This catalog is transmitted metadata. Only the
                            // aggregation path reads it; preserve it otherwise.
                            core::mem::swap(qbi_sources, submitted_qbi_sources);
                        }
                        for item in &mut next_documents.documents {
                            item.allowed_community = document_community_answers(
                                &self.draft,
                                &IncomeSource::Record { value: item.document.clone() },
                            );
                            related_forms::refresh_submitted_activity_page(&self.draft, item);
                        }
                    }
                    *request = refreshed;
                }
                changed
            }
            P::SaveEmployeeExpenses { value, page, warnings, .. }
                if *page == ADJUSTMENTS_PAGE && self.reached(*page) =>
            {
                let Ok(candidate) = self.employee_expenses_candidate(value) else {
                    return false;
                };
                let view = crate::macrs_source::MacrsElectionView::new(&candidate);
                let mut changed = false;
                for (row, form) in value.forms.iter_mut().enumerate() {
                    for (asset, item) in form.equipment_assets.iter_mut().enumerate() {
                        if refresh_asset_macrs_questions(item, form.owner, None, false, &view) {
                            warnings.push(macrs_question_warning(true, row, asset));
                            changed = true;
                        }
                    }
                }
                changed
            }
            _ => false,
        }
    }
    fn handle_form_save(&mut self, request: &P) -> Option<Reply> {
        let (candidate, page, review, next): (_, _, _, fn(&Self, i32) -> Reply) = match request {
            P::SaveSetup { value, page, review, .. } => {
                (self.save_setup(value, *page), *page, *review, Self::next_page)
            }
            P::SaveSpouse { spouse, review, .. } => {
                (self.spouse_candidate(spouse), SPOUSE_PAGE, *review, Self::next_page)
            }
            P::SaveCommunityProperty { value, page, review, .. } => (
                if *page == PEOPLE_PAGE && self.reached(*page) && self.draft.people.is_some() {
                    self.community_candidate(*value)
                } else {
                    Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into())
                },
                COMMUNITY_PAGE,
                *review,
                Self::next_page,
            ),
            P::SaveSpouseBoxes { value, page, review, .. } => (
                if *page == DEDUCTIONS_PAGE && self.reached(*page) {
                    self.spouse_boxes_candidate(*value)
                } else {
                    Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into())
                },
                SPOUSE_BOXES_PAGE,
                *review,
                Self::next_page,
            ),
            P::SaveSpouseItemizes { value, page, review, .. } => (
                if *page == DEDUCTIONS_PAGE && self.reached(*page) {
                    self.spouse_itemizes_candidate(*value)
                } else {
                    Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into())
                },
                SPOUSE_ITEMIZES_PAGE,
                *review,
                Self::next_page,
            ),
            P::SaveDeductions { value, page, review, .. } => {
                (self.save_deductions(value, *page), *page, *review, Self::next_page)
            }
            P::SaveOftenMissed { value, page, review, .. } => {
                (self.save_often_missed(value, *page), *page, *review, Self::next_page)
            }
            P::SaveHsa { value, followups, page, review, .. } => (
                self.hsa_followup_candidate(followups, true)
                    .and_then(|candidate| candidate.save_hsa(value, *page)),
                *page,
                *review,
                Self::enter,
            ),
            P::SaveCare { value, page, review, .. } => {
                (self.save_care(value, *page), *page, *review, Self::enter)
            }
            P::SaveSelfEmployedBenefits { value, page, review, .. } => {
                (self.save_self_employed_benefits(value, *page), *page, *review, Self::enter)
            }
            P::SaveIraContributions { value, page, review, .. } => {
                (self.save_ira_contributions(value, *page), *page, *review, Self::enter)
            }
            P::SaveEducator { value, page, review, .. } => {
                (self.save_educator(value, *page), *page, *review, Self::enter)
            }
            P::SaveStudentLoan { value, page, review, .. } => {
                (self.save_student_loan(value, *page), *page, *review, Self::enter)
            }
            P::SaveEmployeeExpenses { value, page, review, .. } => {
                (self.save_employee_expenses(value, *page), *page, *review, Self::enter)
            }
            P::SaveMedical { value, page, review, .. } => {
                (self.save_medical(value, *page), *page, *review, Self::enter)
            }
            P::SaveTaxes { value, page, review, .. } => {
                (self.save_taxes(value, *page), *page, *review, Self::enter)
            }
            P::SaveHomeOffices { value, page, review, .. } => {
                (self.save_home_offices(value, *page), *page, *review, Self::enter)
            }
            P::SaveMortgages { value, page, review, .. } => {
                (self.save_mortgages(value, *page), *page, *review, Self::enter)
            }
            P::SaveGiftCarryovers { value, page, review, .. } => {
                (self.save_gift_carryovers(value, *page), *page, *review, Self::enter)
            }
            P::SaveSaver { value, page, review, .. } => {
                (self.save_saver(value, *page), *page, *review, Self::enter)
            }
            P::SaveMoreCredits { value, page, review, .. } => {
                (self.save_more_credits(value, *page), *page, *review, Self::enter)
            }
            P::SaveOtherTaxes { value, page, review, .. } => {
                (self.save_other_taxes(value, *page), *page, *review, Self::next_page)
            }
            P::SaveMoreAdjustments { value, page, review, .. } => {
                (self.save_more_adjustments(value, *page), *page, *review, Self::enter)
            }
            P::SavePayments { value, page, review, .. } => {
                (self.save_payments(value, *page), *page, *review, Self::next_page)
            }
            P::SaveIncomeByPeriod { value, page, review, .. } => {
                (self.save_income_by_period(value, *page), *page, *review, Self::next_page)
            }
            P::SaveParentTax { value, page, review, .. } => {
                (self.save_parent_tax(value, *page), *page, *review, Self::next_page)
            }
            P::SaveOtherItemized {
                value,
                page,
                review,
                interest_sources,
                investment_k1_sources,
                investment_origin_choices,
                ..
            } => (
                self.save_other_itemized(
                    value,
                    *page,
                    interest_sources,
                    investment_k1_sources,
                    investment_origin_choices,
                ),
                *page,
                *review,
                Self::after_other_itemized_save,
            ),
            _ => return None,
        };
        Some(match candidate {
            // A save that leaves the other filer's HSA eligibility needed and
            // unanswered reopens the HSA page, which asks it.
            Ok(draft) if matches!(request, P::SaveHsa { .. }) => match self.commit(draft) {
                Ok(_) if self.hsa_other_open().is_some() => self.hsa_page(review),
                Ok(false) if review => self.review(),
                Ok(true) if review => self.resume(),
                Ok(_) => next(self, page),
                Err(message) => problem(message),
            },
            Ok(draft) => self.accept_form(draft, page, review, next),
            Err(message) => refused(message),
        })
    }
    fn handle_save(&mut self, mut request: P) -> Option<Reply> {
        if self.refresh_submitted_macrs_questions(&mut request) {
            return Some(Reply::Edit(request));
        }
        if let Some(reply) = self.handle_form_save(&request) {
            return Some(reply);
        }
        let d = &self.draft;
        Some(match request {
            P::SavePeople { you, page, review, .. } => {
                if page != PEOPLE_PAGE || !self.reached(page) {
                    return Some(problem_key("PROBLEM_SECTION_UNAVAILABLE_YET"));
                }
                let draft = match self.you_candidate(&you) {
                    Ok(draft) => draft,
                    Err(message) => return Some(refused(message)),
                };
                let joint = self.joint();
                match self.commit(draft) {
                    Ok(false) if review => self.review(),
                    Ok(_) if joint => self.spouse_page(false),
                    Ok(true) if review => self.resume(),
                    Ok(_) => self.next_page(page),
                    Err(message) => problem(message),
                }
            }
            P::SaveIncomeWithoutForm { value, page, review, .. } => {
                if page != INCOME_WITHOUT_FORM_PAGE || !self.reached(page) {
                    return Some(problem_key("PROBLEM_SECTION_UNAVAILABLE_YET"));
                }
                match self.income_without_form_candidate(&value) {
                    Ok(draft) => self.accept(draft, page, review),
                    Err(message) => refused(message),
                }
            }
            P::SaveChecklist { value, page, previous, review, .. } => {
                if page != CHECKLIST_PAGE || !self.reached(page) {
                    return Some(problem_key("PROBLEM_SECTION_UNAVAILABLE_YET"));
                }
                match self.checklist_candidate(&value) {
                    // Opened from a hub's "What you told us" row, the page
                    // returns to the hub.
                    Ok(draft) if HUB_PAGES.contains(&previous) && !review => {
                        match self.commit(draft) {
                            Ok(_) => self.page(previous, false),
                            Err(message) => problem(message),
                        }
                    }
                    Ok(draft) => self.accept(draft, page, review),
                    Err(message) => refused(message),
                }
            }
            P::EstimateChecklist { value, .. } => match self.checklist_candidate(&value) {
                Ok(draft) => {
                    let (draft, pruned) = self.reconcile(draft);
                    let result = estimate(&draft);
                    Reply::Response(P::RunningEstimate {
                        estimate: self
                            .estimate_so_far(&draft, &result, TaxSection::Checklist)
                            .map(Into::into),
                        warnings: save_warnings(&pruned),
                        progress: checklist_path(&draft, &result),
                    })
                }
                Err(message) => refused(message),
            },
            P::SaveChildren { dependents, page, previous, review, .. } => {
                if page != CHILDREN_PAGE || !self.reached(page) {
                    return Some(problem_key("PROBLEM_SECTION_UNAVAILABLE_YET"));
                }
                match self.dependents_candidate(dependents) {
                    // Opened from the credits hub's care row, the page
                    // returns to the hub.
                    Ok(draft) if previous == CREDITS_PAGE && !review => match self.commit(draft) {
                        Ok(_) if self.reached(CREDITS_PAGE) => self.credits_hub(false),
                        Ok(_) => self.resume(),
                        Err(message) => problem(message),
                    },
                    Ok(draft) => self.accept(draft, page, review),
                    Err(message) => refused(message),
                }
            }
            P::SaveDocuments { documents, document_owners, review, .. } => {
                self.save_document_center(documents, &document_owners, review)
            }
            P::SaveIncomes {
                qbi_sources,
                documents,
                followups,
                source_documents: _,
                page,
                previous,
                topic,
                review,
                ..
            } => {
                if !INCOME_TOPICS.contains(&topic) || page != INCOME_PAGE || !self.reached(page) {
                    return Some(problem_key("PROBLEM_SECTION_UNAVAILABLE_YET"));
                }
                let value_fields: Vec<bool> = documents.documents.iter().map(|_| false).collect();
                let mut origins = Vec::new();
                let question_keys: Vec<_> = followups.iter().map(|row| row.key.clone()).collect();
                match self
                    .followup_candidate(topic, followups, &documents.documents)
                    .and_then(|candidate| {
                        candidate.documents_candidate_inner_with_origins(
                            topic,
                            documents.into_edits(),
                            false,
                            Some(&mut origins),
                            Some(&question_keys),
                        )
                    })
                    .and_then(|mut draft| {
                        followup_page::answer_situations(&mut draft, topic);
                        if crate::qbi_aggregation::uses_facts(&draft) {
                            let sources = crate::qbi_aggregation::source_views(&draft, &self.draft);
                            if !crate::qbi_aggregation::catalog_matches(&qbi_sources, &sources) {
                                return Err(crate::validation::ValidationIssue::new(
                                    "QBI_AGGREGATION_SOURCES",
                                )
                                .at_field("qbi_sources")
                                .into());
                            }
                        }
                        Ok(draft)
                    }) {
                    Ok(draft) => match self.commit_income(draft, &origins, None) {
                        Ok(prepared)
                            if topic == TaxSection::InvestmentSales
                                && crate::qsbs_followups::needed(&self.draft) =>
                        {
                            self.topic_page_with_result(topic, review, Some(&prepared.result))
                        }
                        // A rental this save created cannot be asked before it
                        // exists: its section 179 question is asked now, not
                        // from Review.
                        Ok(prepared)
                            if topic == TaxSection::Rentals
                                && document_followups::rental_section179_open(&self.draft) =>
                        {
                            self.topic_page_with_result(topic, review, Some(&prepared.result))
                        }
                        Ok(prepared) => self.income_save_reply(prepared, previous, review, None),
                        Err(message) => refused(message.map_list_item_field_prefix(
                            &["documents", "documents"],
                            &["document"],
                            &["document", "value"],
                            &value_fields,
                        )),
                    },
                    Err(message) => refused(message),
                }
            }
            P::SaveWagesAnswers { wages, followups, records, page, previous, review, .. } => {
                if page != WAGES_PAGE || !self.reached(page) {
                    return Some(problem_key("PROBLEM_SECTION_UNAVAILABLE_YET"));
                }
                let mut origins = Vec::new();
                match self
                    .wages_answers_candidate(wages, followups, records, true, Some(&mut origins))
                    .and_then(|mut draft| {
                        followup_page::answer_situations(&mut draft, TaxSection::Wages);
                        self.commit_income(draft, &origins, None)
                    })
                    .map_err(|failure| {
                        failure.map_field_prefix(
                            &["documents", "documents"],
                            &["records", "documents"],
                        )
                    }) {
                    Ok(prepared) => self.income_save_reply(prepared, previous, review, None),
                    Err(message) => refused(message),
                }
            }
            P::SaveEducation { value, followups, page, review, .. } => {
                if page != CREDITS_PAGE || !self.reached(page) {
                    return Some(problem_key("PROBLEM_SECTION_UNAVAILABLE_YET"));
                }
                match self
                    .education_followup_candidate(&followups, true)
                    .and_then(|candidate| candidate.education_candidate(&value))
                {
                    // Saved from its hub, the page returns to the hub with a
                    // new suggestion.
                    Ok(draft) => match self.commit(draft) {
                        Ok(false) if review => self.review(),
                        Ok(true) if review => self.resume(),
                        Ok(_) => self.enter(CREDITS_PAGE),
                        Err(message) => problem(message),
                    },
                    Err(message) => refused(message),
                }
            }
            P::SaveMarketplace { value, page, review, .. } => {
                if page != CREDITS_PAGE || !self.reached(page) {
                    return Some(problem_key("PROBLEM_SECTION_UNAVAILABLE_YET"));
                }
                match self.marketplace_candidate(&value) {
                    // Saved from its hub, the page returns to the hub with a
                    // new suggestion.
                    Ok(draft) => match self.commit_marketplace(draft) {
                        Ok(false) if review => self.review(),
                        Ok(true) if review => self.resume(),
                        Ok(_) => self.enter(CREDITS_PAGE),
                        Err(message) => refused(message),
                    },
                    Err(message) => refused(message),
                }
            }
            P::SaveInjuredSpouse { value, followups, page, review, .. } => {
                if page != INJURED_SPOUSE_PAGE || !injured_chosen(&self.draft) {
                    return Some(problem_key("PROBLEM_SECTION_UNAVAILABLE_YET"));
                }
                let mut answering = self.candidate_editor(self.draft.clone());
                answering.draft.injured_spouse =
                    Some(crate::InjuredSpouseAllocation::from_page(&value));
                let context_result = estimate(&answering.draft);
                let expected_questions =
                    Self::injured_spouse_followups_for(&answering.draft, &context_result);
                if followups.len() != expected_questions.len()
                    || expected_questions
                        .iter()
                        .any(|expected| !followups.iter().any(|row| row.key == expected.key))
                {
                    return Some(self.injured_spouse_page_for(review, &answering.draft));
                }
                let mut candidate =
                    match answering.injured_spouse_followup_candidate(&followups, true) {
                        Ok(candidate) => candidate.draft,
                        Err(failure) => return Some(refused(failure)),
                    };
                let result = estimate(&candidate);
                if let Some(issue) = crate::injured_spouse::facts_issue(&value, &candidate, &result)
                {
                    return Some(refused(issue.in_field("value")));
                }
                let expected =
                    result.injured_spouse.as_ref().map(|p| p.required.as_slice()).unwrap_or(&[]);
                let refreshed =
                    candidate.injured_spouse.as_ref().unwrap().page(&candidate, &result);
                let catalogs_changed = value.allowed_debt_facts != refreshed.allowed_debt_facts;
                let new_keys_match = value.allocation_rows().count() == expected.len()
                    && value
                        .allocation_rows()
                        .all(|v| expected.iter().any(|r| r.component == v.component));
                let bounds_changed = value.allocation_rows().any(|v| {
                    expected
                        .iter()
                        .find(|r| r.component == v.component)
                        .is_some_and(|r| r.minimum != v.minimum || r.maximum != v.maximum)
                });
                if catalogs_changed || !new_keys_match || bounds_changed {
                    return Some(self.injured_spouse_page_for(review, &candidate));
                }
                if let Some(issue) = crate::injured_spouse::debt_facts_issue(&value) {
                    return Some(refused(issue.in_field("value")));
                }
                // Reuse the one candidate and native result already checked
                // above: combine the two request arrays exactly once.
                if let Some(issue) =
                    crate::injured_spouse::allocation_issue(&value, &candidate, &result)
                {
                    return Some(refused(issue.in_field("value")));
                }
                candidate.injured_spouse.as_mut().unwrap().order_allocations(&result);
                // Without a plan (income rows still unanswered) the page offered no
                // rows, so it cannot have changed the answers already accepted.
                if result.injured_spouse.is_none() {
                    candidate.injured_spouse.as_mut().unwrap().allocations = self
                        .draft
                        .injured_spouse
                        .as_ref()
                        .map(|stored| stored.allocations.clone())
                        .unwrap_or_default();
                }
                self.accept(candidate, page, review)
            }
            P::StartFinish { .. } => {
                let result = estimate(d);
                if result.state != ResultState::EstimatedForSupportedScope
                    || !result.issues.is_empty()
                    || !self.remaining_items(&result).is_empty()
                {
                    return Some(problem_key("PROBLEM_REVIEW_ISSUES_REMAIN"));
                }
                Reply::Response(P::BeginFinish {
                    cookie: d.cookie.clone(),
                    revision: d.revision,
                    page: REVIEW_PAGE,
                    previous: REVIEW_PAGE,
                    review: true,
                    progress: crate::interview::status(
                        "MESSAGE_COMPLETE_THE_PROTOTYPE_REVIEW_NOTHING_IS_SIGNED_OR_FILED",
                    ),
                    refund: self.refund_choice(),
                })
            }
            P::Finish { refund, .. } => {
                let result = estimate(d);
                if result.state != ResultState::EstimatedForSupportedScope
                    || !result.issues.is_empty()
                    || !self.remaining_items(&result).is_empty()
                {
                    return Some(problem_key("PROBLEM_REVIEW_NOT_READY"));
                }
                let mut draft = match self.refund_candidate(&refund) {
                    Ok(draft) => draft,
                    Err(message) => return Some(refused(message)),
                };
                let Some(revision) = self.draft.revision.checked_add(1).filter(|r| *r < i64::MAX)
                else {
                    return Some(problem_key("PROBLEM_REVISION_LIMIT"));
                };
                draft.revision = revision;
                draft.finished = true;
                self.draft = draft;
                self.dirty = true;
                Reply::Response(P::Finished {
                    message: crate::interview::status(
                        "MESSAGE_PROTOTYPE_REVIEW_COMPLETED_NO_TAX_RETURN_HAS_BEEN_SIGNED_OR_FILED",
                    ),
                    result: estimate(&self.draft).into(),
                })
            }
            _ => return None,
        })
    }

    fn handle_calculation(&mut self, request: P) -> Option<Reply> {
        Some(match request {
            P::EstimateIncomeWithoutForm { value, .. } => self.running(
                self.income_without_form_candidate(&value),
                INCOME_WITHOUT_FORM_PAGE,
                None,
            ),
            P::FarmRelatedOptions { id, owner, .. } => {
                let center = crate::related_forms::catalog_for_provisional(
                    &self.draft,
                    &crate::related_forms::ProvisionalActivity {
                        id,
                        owner,
                        kind: crate::related_forms::ActivityKind::Farm,
                        cooperative_eligible: true,
                    },
                );
                let claimed = crate::related_forms::claimed_elsewhere(&self.draft, id);
                let related = crate::RelatedForms::page::<crate::Farm>(&[], &center, &claimed);
                Reply::Response(P::FarmRelatedChoices { candidates: related.candidates })
            }
            P::FarmRentalRelatedOptions { id, owner, .. } => {
                let center = crate::related_forms::catalog_for_provisional(
                    &self.draft,
                    &crate::related_forms::ProvisionalActivity {
                        id,
                        owner,
                        kind: crate::related_forms::ActivityKind::FarmRental,
                        cooperative_eligible: true,
                    },
                );
                let claimed = crate::related_forms::claimed_elsewhere(&self.draft, id);
                let related =
                    crate::RelatedForms::page::<crate::FarmRental>(&[], &center, &claimed);
                Reply::Response(P::FarmRentalRelatedChoices { candidates: related.candidates })
            }
            P::HobbyRelatedOptions { id, owner, .. } => {
                let center = crate::related_forms::catalog_for_provisional(
                    &self.draft,
                    &crate::related_forms::ProvisionalActivity {
                        id,
                        owner,
                        kind: crate::related_forms::ActivityKind::Hobby,
                        cooperative_eligible: false,
                    },
                );
                let claimed = crate::related_forms::claimed_elsewhere(&self.draft, id);
                let related =
                    crate::RelatedForms::page::<crate::HobbySales>(&[], &center, &claimed);
                Reply::Response(P::HobbyRelatedChoices { candidates: related.candidates })
            }
            P::EquipmentRelatedOptions { id, owner, .. } => {
                let center = crate::related_forms::catalog_for_provisional(
                    &self.draft,
                    &crate::related_forms::ProvisionalActivity {
                        id,
                        owner,
                        kind: crate::related_forms::ActivityKind::Equipment,
                        cooperative_eligible: false,
                    },
                );
                let claimed = crate::related_forms::claimed_elsewhere(&self.draft, id);
                let related = crate::RelatedForms::page::<EquipmentRent>(&[], &center, &claimed);
                Reply::Response(P::EquipmentRelatedChoices { candidates: related.candidates })
            }
            P::RoyaltyRelatedOptions { id, owner, .. } => {
                let center = crate::related_forms::catalog_for_provisional(
                    &self.draft,
                    &crate::related_forms::ProvisionalActivity {
                        id,
                        owner,
                        kind: crate::related_forms::ActivityKind::Royalty,
                        cooperative_eligible: false,
                    },
                );
                let claimed = crate::related_forms::claimed_elsewhere(&self.draft, id);
                let related = crate::RelatedForms::page::<RoyaltyProperty>(&[], &center, &claimed);
                Reply::Response(P::RoyaltyRelatedChoices { candidates: related.candidates })
            }
            P::SavePaymentParts { key, value, .. } => self.save_payment_parts(&key, value),
            P::BusinessRelatedOptions { id, owner, source, gambling, .. } => {
                let center = if gambling.is_some() {
                    Vec::new()
                } else {
                    crate::related_forms::catalog_for_provisional(
                        &self.draft,
                        &crate::related_forms::ProvisionalActivity {
                            id,
                            kind: crate::related_forms::ActivityKind::Business,
                            owner,
                            cooperative_eligible: source == crate::BusinessSource::UnitedStates,
                        },
                    )
                };
                let claimed = crate::related_forms::claimed_elsewhere(&self.draft, id);
                let related = crate::RelatedForms::page::<Business>(&[], &center, &claimed);
                Reply::Response(P::BusinessRelatedChoices { candidates: related.candidates })
            }
            P::AnswerRentalRisk { value, .. } => {
                if let Some(issue) = value
                    .issue_for_method(&crate::RiskEffectiveCategory::HoldingRealProperty, false)
                {
                    refused(issue.in_field("value"))
                } else {
                    Reply::Response(P::RentalRiskAnswered {
                        known: alloc::vec![value.clone()],
                        value,
                    })
                }
            }
            P::RentalRelatedOptions { id, owner, personal_use, .. } => {
                let mut center = crate::related_forms::catalog_for_provisional(
                    &self.draft,
                    &crate::related_forms::ProvisionalActivity {
                        id,
                        owner,
                        kind: crate::related_forms::ActivityKind::Rental,
                        cooperative_eligible: false,
                    },
                );
                if matches!(personal_use, PersonalUse::No) {
                    center.retain(|d| d.kind != crate::related_forms::DocumentKind::Mortgage1098);
                }
                let claimed = crate::related_forms::claimed_elsewhere(&self.draft, id);
                let related = crate::RelatedForms::page::<Rental>(&[], &center, &claimed);
                Reply::Response(P::RentalRelatedChoices { candidates: related.candidates })
            }
            P::AnswerQsbsZone { value, .. } => {
                Reply::Response(P::QsbsZoneAnswered { value, known: alloc::vec![value] })
            }
            P::AnswerQsbsValuation { value, .. } => {
                Reply::Response(P::QsbsValuationAnswered { value, known: alloc::vec![value] })
            }
            P::AnswerQsbsBoolean { value, .. } => {
                Reply::Response(P::QsbsBooleanAnswered { value, known: alloc::vec![value] })
            }
            P::AnswerQsbsPriorGain { value, .. } => Reply::Response(P::QsbsPriorGainAnswered {
                value,
                known: alloc::vec![crate::QsbsGainValue { value }],
            }),
            P::ProjectRelatedForm { key, .. } => Reply::Response(P::RelatedFormProjected {
                summary: crate::related_forms::catalog(&self.draft)
                    .into_iter()
                    .find(|source| source.id == key)
                    .map(|source| source.summary),
            }),
            P::ViewRelatedForm { key, .. } | P::ViewReviewedRelatedForm { key, .. } => {
                self.view_related_form(key)
            }
            P::AssetMacrsSourceOptions { history, context, .. } => {
                let original_system = if matches!(history, AssetHistory::Continuing { .. })
                    || context.as_ref().is_some_and(|context| context.retained_original_system)
                {
                    alloc::vec![0, 1]
                } else {
                    Vec::new()
                };
                Reply::Response(P::AssetMacrsSourceChoices { original_system })
            }
            P::AssetMacrsClassLifeOptions { class, source, original_system, context, .. } => {
                let class_life = source.class_life_choices(
                    original_system.as_ref(),
                    class == AssetClass::ThirtyNine,
                    context.as_deref(),
                    equipment_class(class),
                );
                Reply::Response(P::AssetMacrsClassLifeChoices { class_life })
            }
            P::PropertyMacrsSourceOptions { history, context, .. } => {
                let original_system = if matches!(history, EquipmentHistory::Continuing { .. })
                    || context.as_ref().is_some_and(|context| context.retained_original_system)
                {
                    alloc::vec![0, 1]
                } else {
                    Vec::new()
                };
                Reply::Response(P::PropertyMacrsSourceChoices { original_system })
            }
            P::PropertyMacrsClassLifeOptions {
                class, source, original_system, context, ..
            } => {
                let class_life = source.class_life_choices(
                    original_system.as_ref(),
                    class == crate::GdsEquipmentClass::ThirtyNine,
                    context.as_deref(),
                    class.into(),
                );
                Reply::Response(P::PropertyMacrsClassLifeChoices { class_life })
            }
            P::AssetAmtCarryOptions { history, .. } => Reply::Response(P::AssetAmtCarryChoices {
                allowed: crate::AssetAmt179CarryAnswer::page_choices(&history),
            }),
            P::VehicleAmtCarryOptions { history, .. } => {
                Reply::Response(P::VehicleAmtCarryChoices {
                    allowed: crate::AssetAmt179CarryAnswer::page_choices(&history),
                })
            }
            P::NonMacrsAssetCarryOptions { recovery, .. } => {
                Reply::Response(P::NonMacrsAssetCarryChoices {
                    allowed: recovery.amt_carry_choices(),
                })
            }
            P::IntangibleSourceOptions {
                kind,
                obtained,
                context,
                acquired,
                business_started,
                ..
            } => {
                let (origin_choices, prior_choices) = crate::BusinessIntangiblePage::choices(
                    &kind,
                    &obtained,
                    &context,
                    acquired,
                    business_started,
                );
                Reply::Response(P::IntangibleSourceChoices { origin_choices, prior_choices })
            }
            P::NonMacrsCanonicalCarryOptions { recovery, .. } => {
                Reply::Response(P::NonMacrsCanonicalCarryChoices {
                    allowed: recovery.amt_carry_choices(),
                })
            }
            P::CanonicalAssetAmtCarryOptions { financial, .. } => {
                Reply::Response(P::CanonicalAssetAmtCarryChoices {
                    allowed: financial.amt_carry_choices(),
                })
            }
            P::RentalRecoveryOptions { value, .. } => {
                let choices = crate::rental_recovery::choices(&value);
                Reply::Response(P::RentalRecoveryChoices {
                    building: choices.building,
                    more: choices.more,
                })
            }
            P::DocumentPropertyOptions { document, .. } => {
                Reply::Response(P::DocumentPropertyChoices {
                    allowed_community: document_community_answers(
                        &self.draft,
                        &IncomeSource::Record { value: document },
                    ),
                })
            }
            P::PreviewDocument { id, owner, community, document, .. } => {
                return self.preview_document_with_path(
                    id,
                    owner,
                    community,
                    IncomeSource::Record { value: document },
                    false,
                );
            }
            // Pure calculations: the page's answers merged into the accepted
            // return; no revision is an input and nothing is stored.
            P::EstimateSetup { value, .. } => {
                self.running(self.setup_candidate(&value), SETUP_PAGE, None)
            }
            P::EstimateYou { you, .. } => self.running(self.you_candidate(&you), PEOPLE_PAGE, None),
            P::EstimateCommunityProperty { value, .. } => {
                self.running(self.community_candidate(value), PEOPLE_PAGE, None)
            }
            P::EstimateSpouseBoxes { value, .. } => {
                self.running(self.spouse_boxes_candidate(value), DEDUCTIONS_PAGE, None)
            }
            P::EstimateSpouseItemizes { value, .. } => {
                self.running(self.spouse_itemizes_candidate(value), DEDUCTIONS_PAGE, None)
            }
            P::EstimateSpouse { spouse, .. } => {
                self.running(self.spouse_candidate(&spouse), SPOUSE_PAGE, None)
            }
            P::DocumentOwnershipOptions { person, documents, .. } => {
                Reply::Response(P::DocumentOwnershipChoices {
                    documents: document_owners::ownership_choices(&self.draft, person, &documents),
                })
            }
            P::EstimateWagesAnswers { wages, followups, records, .. } => self.running(
                self.wages_answers_candidate(wages, followups, records, false, None),
                WAGES_PAGE,
                Some(TaxSection::Wages),
            ),
            P::EstimateDocuments { topic, documents, followups, .. } => {
                self.income_running(topic, documents, followups)
            }
            P::EstimateOftenMissed { value, .. } => {
                self.running(self.often_missed_candidate(&value), OFTEN_MISSED_PAGE, None)
            }
            P::EstimateDependents { dependents, .. } => {
                match self.prepare_running(self.dependents_candidate(dependents), None) {
                    Ok(prepared) => Reply::Response(P::DependentsRunningEstimate {
                        estimate: self
                            .estimate_so_far(
                                &prepared.draft,
                                &prepared.result,
                                TaxSection::Children,
                            )
                            .map(Into::into),
                        warnings: save_warnings(&prepared.pruned),
                        progress: progress_text(CHILDREN_PAGE, &prepared.draft, None),
                        credits: person_credits(&prepared.draft),
                    }),
                    Err(message) => refused(message),
                }
            }
            P::EstimateHsa { value, followups, .. } => self.running(
                self.hsa_followup_candidate(&followups, false)
                    .and_then(|candidate| candidate.hsa_candidate(&value)),
                ADJUSTMENTS_PAGE,
                Some(TaxSection::Hsa),
            ),
            P::EstimateSelfEmployedBenefits { value, .. } => self.running(
                self.self_employed_benefits_candidate(&value),
                ADJUSTMENTS_PAGE,
                Some(TaxSection::SelfEmployedBenefits),
            ),
            P::EstimateIraContributions { value, .. } => self.running(
                self.ira_contributions_candidate(&value),
                ADJUSTMENTS_PAGE,
                Some(TaxSection::IraContributions),
            ),
            P::EstimateEducator { value, .. } => self.running(
                self.educator_candidate(&value),
                ADJUSTMENTS_PAGE,
                Some(TaxSection::Educator),
            ),
            P::EstimateStudentLoan { value, .. } => self.running(
                self.student_loan_candidate(&value),
                ADJUSTMENTS_PAGE,
                Some(TaxSection::StudentLoan),
            ),
            P::EstimateEmployeeExpenses { value, .. } => self.running(
                self.employee_expenses_candidate(&value),
                ADJUSTMENTS_PAGE,
                Some(TaxSection::EmployeeExpenses),
            ),
            P::EstimateDeductions { value, .. } => {
                self.running(self.deductions_candidate(&value), DEDUCTIONS_PAGE, None)
            }
            P::EstimateMedical { value, .. } => self.running(
                self.medical_candidate(&value),
                ITEMIZED_PAGE,
                Some(TaxSection::Medical),
            ),
            P::EstimateTaxes { value, .. } => {
                self.running(self.taxes_candidate(&value), ITEMIZED_PAGE, Some(TaxSection::Taxes))
            }
            P::EstimateOtherItemized { value, .. } => self.running(
                self.other_itemized_candidate(&value),
                if itemizing(&self.draft) { ITEMIZED_PAGE } else { DEDUCTIONS_PAGE },
                Some(TaxSection::OtherItemized),
            ),
            P::EstimateHomeOffices { value, .. } => self.running(
                self.home_offices_candidate(&value),
                INCOME_PAGE,
                Some(TaxSection::HomeOffices),
            ),
            P::EstimateMortgages { value, .. } => self.running(
                self.mortgages_candidate(&value),
                mortgage_parent(&self.draft),
                Some(TaxSection::Mortgage),
            ),
            P::EstimateGiftCarryovers { value, .. } => self.running(
                self.gift_carryovers_candidate(&value),
                ITEMIZED_PAGE,
                Some(TaxSection::GiftCarryovers),
            ),
            P::EstimateCare { value, .. } => {
                self.running(self.care_candidate(&value), CREDITS_PAGE, Some(TaxSection::Care))
            }
            P::EstimateEducation { value, followups, .. } => self.running(
                self.education_followup_candidate(&followups, false)
                    .and_then(|candidate| candidate.education_candidate(&value)),
                CREDITS_PAGE,
                Some(TaxSection::Education),
            ),
            P::EstimateSaver { value, .. } => {
                self.running(self.saver_candidate(&value), CREDITS_PAGE, Some(TaxSection::Saver))
            }
            P::EstimateMarketplace { value, .. } => self.running(
                self.marketplace_candidate(&value),
                CREDITS_PAGE,
                Some(TaxSection::Marketplace),
            ),
            P::EstimateMoreCredits { value, .. } => self.running(
                self.more_credits_candidate(&value),
                CREDITS_PAGE,
                Some(TaxSection::MoreCredits),
            ),
            P::EstimateMoreAdjustments { value, .. } => self.running(
                self.more_adjustments_candidate(&value),
                ADJUSTMENTS_PAGE,
                Some(TaxSection::MoreAdjustments),
            ),
            P::EstimateOtherTaxes { value, .. } => {
                self.running(self.other_taxes_candidate(&value), OTHER_TAXES_PAGE, None)
            }
            P::EstimatePayments { value, .. } => {
                match self.prepare_running(self.payments_candidate(&value.into()), None) {
                    Ok(prepared) => Reply::Response(P::PaymentsEstimate {
                        estimate: self
                            .estimate_so_far(
                                &prepared.draft,
                                &prepared.result,
                                TaxSection::Payments,
                            )
                            .map(Into::into),
                        warnings: save_warnings(&prepared.pruned),
                        progress: progress_text(PAYMENTS_PAGE, &prepared.draft, None),
                    }),
                    Err(failure) => refused(failure),
                }
            }
            P::EstimateIncomeByPeriod { value, .. } => {
                self.running(self.income_by_period_candidate(&value), INCOME_BY_PERIOD_PAGE, None)
            }
            P::EstimateInjuredSpouse { value, followups, .. } => {
                let mut answering = self.candidate_editor(self.draft.clone());
                answering.draft.injured_spouse =
                    Some(crate::InjuredSpouseAllocation::from_page(&value));
                self.running(
                    answering
                        .injured_spouse_followup_candidate(&followups, false)
                        .and_then(|candidate| candidate.injured_spouse_candidate(&value)),
                    INJURED_SPOUSE_PAGE,
                    None,
                )
            }
            P::EstimateParentTax { value, .. } => {
                self.running(self.parent_tax_candidate(&value), PARENT_TAX_PAGE, None)
            }
            P::CalculateMarketplaceAppeal { determined, .. } => {
                match crate::dates::add_days_checked(determined, 120) {
                    Some(required_deadline) => {
                        Reply::Response(P::MarketplaceAppealDeadline { required_deadline })
                    }
                    None => problem_key("PTC_APPEAL_DATE_INVALID"),
                }
            }
            // The family split's allowed set from the accounts alone; the
            // split itself is never an input.
            P::HsaOptions { accounts, other_coverage, family_method, .. } => {
                Reply::Response(P::HsaFamilyOptions {
                    allowed_family_split: allowed_family_split(
                        &accounts,
                        &self.draft,
                        other_coverage.as_ref().map(crate::HsaEligibilityPage::record).as_ref(),
                        family_method,
                    ),
                })
            }
            P::BusinessCreditElectionOptions { credit, .. } => {
                Reply::Response(P::BusinessCreditElectionChoices {
                    allowed_elections: crate::business_credit_page::election_choices(credit),
                    allowed_carbon_oxide_origins:
                        crate::business_credit_page::carbon_oxide_origin_choices(credit),
                    allowed_renewable_facility_origins:
                        crate::business_credit_page::renewable_facility_origin_choices(credit),
                })
            }
            P::InvestmentOriginOptions { source, carry, amt_carry, .. } => {
                Reply::Response(P::InvestmentOriginChoices {
                    allowed_origin_choices: crate::investment_interest::loan_origin_choices(
                        &self.draft,
                        source,
                        carry,
                        amt_carry,
                    ),
                })
            }
            P::RetainedCapitalSourceOptions {
                agi,
                deductions,
                senior_deduction,
                short,
                long,
                deducted_loss,
                ordinary_history,
                available,
                ..
            } => {
                let carry = crate::extended_rules::capital_carryover(
                    crate::kernel::ratio::dollars(short),
                    crate::kernel::ratio::dollars(long),
                    crate::kernel::ratio::dollars(deducted_loss),
                    crate::kernel::ratio::dollars(agi) - crate::kernel::ratio::dollars(deductions)
                        + crate::kernel::ratio::dollars(senior_deduction),
                );
                Reply::Response(P::RetainedCapitalSourceChoices {
                    allowed_research_parts: crate::retained_research::choices(
                        if ordinary_history { (carry[0] + carry[1]) * 100 } else { 0 },
                        &available,
                    ),
                })
            }
            P::RetainedCharityAppreciationOptions { election, records, available, .. } => {
                Reply::Response(P::RetainedCharityAppreciationChoices {
                    records: records
                        .into_iter()
                        .map(|r| crate::CharityAppreciationChoices {
                            allowed_source_appreciation:
                                crate::retained_charity::appreciation_choices(
                                    r.class,
                                    r.balance,
                                    r.appreciation,
                                    election,
                                    &available,
                                ),
                        })
                        .collect(),
                })
            }
            P::RetainedLossSourceOptions { balance, available, .. } => {
                Reply::Response(P::RetainedLossSourceChoices {
                    allowed_research_parts: crate::retained_research::choices(balance, &available),
                })
            }
            P::ResearchCreditSourceOptions { credit, .. } => {
                Reply::Response(P::ResearchCreditSourceChoices {
                    allowed_research_choices: crate::business_credit_page::research_choices(credit),
                })
            }
            P::K1InvestmentOptions { key, properties, .. } => {
                match k1_followups::proposed_investment_choices(
                    &self.draft,
                    &key,
                    properties.into_iter().map(Into::into).collect(),
                ) {
                    Ok(choices) => Reply::Response(P::K1InvestmentChoices { choices }),
                    Err(issue) => refused(issue),
                }
            }
            _ => return None,
        })
    }
}
pub(crate) fn joint(d: &FederalDraft) -> bool {
    d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Joint)
}
fn parent_missing(d: &FederalDraft) -> bool {
    d.people.as_ref().is_some_and(|p| p.parent_tax.is_none()) && parent_tax_in_walk(d)
}
// The checklist's injured spouse item on a joint return opens Form 8379's
// page (T3-8379).
pub(crate) fn injured_chosen(d: &FederalDraft) -> bool {
    joint(d) && d.whole_return() & SpouseSituations::INJURED_SPOUSE.0 != 0
}
// A separate return asks community property on its own step after About you.
fn separate(d: &FederalDraft) -> bool {
    d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Separate)
}
fn community_missing(d: &FederalDraft) -> bool {
    separate(d)
        && d.people.is_some()
        && d.setup.as_ref().is_some_and(|s| s.community_property.answer().is_none())
}
// The spouse boxes' step waits for its answer while the rules need it.
fn spouse_itemizes_missing(d: &FederalDraft) -> bool {
    separate(d)
        && d.finished_sections.contains(&TaxSection::Deductions)
        && d.deductions.as_ref().is_some_and(|ded| ded.spouse_itemizes.is_none())
}
fn spouse_boxes_missing(d: &FederalDraft) -> bool {
    crate::federal_rules::spouse_boxes_apply(d)
        && d.finished_sections.contains(&TaxSection::Deductions)
        && d.setup.as_ref().is_some_and(|s| s.separate_spouse.answer().is_none())
}
fn spouse_missing(d: &FederalDraft) -> bool {
    in_walk(d, TaxSection::Spouse) && d.people.as_ref().is_some_and(|p| p.spouse.is_none())
}
// The first page in the walk whose required data is absent, or the review
// (walk-owned data: present is answered, absent is asked). A section's
// required datum is its mark; the people pages' are their answers.
fn first_open(d: &FederalDraft) -> i32 {
    let finished = |section: TaxSection| d.finished_sections.contains(&section);
    let walks = |section: TaxSection| in_walk(d, section);
    if d.setup.is_none() {
        SETUP_PAGE
    } else if d.people.is_none() || spouse_missing(d) {
        PEOPLE_PAGE
    } else if community_missing(d) {
        COMMUNITY_PAGE
    } else if !d.finished_sections.contains(&TaxSection::Children) {
        CHILDREN_PAGE
    } else if !finished(TaxSection::Documents) {
        DOCUMENTS_PAGE
    } else if d.without_form.is_none() {
        INCOME_WITHOUT_FORM_PAGE
    } else if !finished(TaxSection::Checklist) {
        CHECKLIST_PAGE
    } else if !finished(TaxSection::Income) {
        INCOME_PAGE
    } else if !finished(TaxSection::Adjustments) {
        ADJUSTMENTS_PAGE
    } else if !finished(TaxSection::Deductions) {
        DEDUCTIONS_PAGE
    } else if spouse_itemizes_missing(d) {
        SPOUSE_ITEMIZES_PAGE
    } else if spouse_boxes_missing(d) {
        SPOUSE_BOXES_PAGE
    } else if walks(TaxSection::Itemized) && !finished(TaxSection::Itemized) {
        ITEMIZED_PAGE
    } else if !finished(TaxSection::Credits) {
        CREDITS_PAGE
    } else if !finished(TaxSection::OtherTaxes) {
        OTHER_TAXES_PAGE
    } else if parent_missing(d) {
        PARENT_TAX_PAGE
    } else if !finished(TaxSection::Payments) {
        PAYMENTS_PAGE
    } else if walks(TaxSection::IncomeByPeriod)
        && d.payments.as_ref().is_some_and(|p| p.annualized_income.is_none())
    {
        INCOME_BY_PERIOD_PAGE
    } else if walks(TaxSection::InjuredSpouse) && d.injured_spouse.is_none() {
        INJURED_SPOUSE_PAGE
    } else if !finished(TaxSection::OftenMissed) {
        OFTEN_MISSED_PAGE
    } else {
        REVIEW_PAGE
    }
}
// "2026 federal · section N of M" over the sections this return shows
// (M3-P19): the setup, filer, people and checklist
// pages, each walked section that has something for this return, the
// page's own section, and the review; a topic page adds its name,
// "section 4 of 5 · Wages".
fn progress_text(page: i32, d: &FederalDraft, topic: Option<TaxSection>) -> Vec<u8> {
    progress_line(page, &shown_sections(d), topic.map(section_message))
}
fn progress_line(page: i32, shown: &[TaxSection], suffix: Option<crate::Message>) -> Vec<u8> {
    let current = page_section(page);
    if current.is_none() {
        return crate::interview::status("MESSAGE_2026_FEDERAL_ANYTHING_ELSE");
    }
    let mut sections = alloc::vec![
        TaxSection::Setup,
        TaxSection::People,
        TaxSection::Children,
        TaxSection::Documents,
        TaxSection::IncomeWithoutForm,
        TaxSection::Checklist,
    ];
    sections.extend(
        WALKED_SECTIONS
            .into_iter()
            .filter(|s| *s != TaxSection::Itemized && (shown.contains(s) || current == Some(*s))),
    );
    sections.push(TaxSection::Review);
    let section =
        sections.iter().position(|s| Some(*s) == current).map_or(sections.len(), |at| at + 1);
    let mut arguments = alloc::vec![
        crate::MessageArg::Number(section as i64),
        crate::MessageArg::Number(sections.len() as i64),
    ];
    let key = if let Some(suffix) = suffix {
        arguments.push(crate::MessageArg::Nested(alloc::boxed::Box::new(suffix)));
        "PROGRESS_SECTION_WITH_TOPIC"
    } else {
        "PROGRESS_SECTION"
    };
    crate::Message::new(key, arguments).encode()
}
// Records a section left forward; each section is recorded once.
fn finish_section(d: &mut FederalDraft, section: TaxSection) {
    if !d.finished_sections.contains(&section) {
        d.finished_sections.push(section);
    }
}
// Sections M2 does not ask hold their "nothing applies" answers until the
// milestone that adds their page, so the estimate never waits on them. The
// people page is asked only when page 1 says you support someone.
// Every unasked section present, for an owner outside the interview that
// records an answer there.
pub(crate) fn fill_unasked_screening(d: &mut FederalDraft) {
    fill_unasked(d);
}
fn fill_unasked(d: &mut FederalDraft) {
    d.adjustments.get_or_insert_with(|| FederalAdjustments {
        social_premiums: Default::default(),
        health_employer_w2: Vec::new(),
        health_statements: Vec::new(),
        health_contribution_allocations: Vec::new(),
        health_marketplace: HealthMarketplace::default(),
        social_election: SocialElection::Regular,
        alimony_paid: Vec::new(),
        moving: Vec::new(),
        pension_contributions: Vec::new(),
        hsa: Vec::new(),
        hsa_family_share: None,
        archer: ArcherContributions::None,
        qualified_charity: Vec::new(),
        ira_basis: Vec::new(),
        roth_history: Vec::new(),
        required_distributions: Vec::new(),
        iras: FederalIras::NONE,
        student_loan: StudentLoanClaim::NONE,
        educators: EducatorClaim::NONE,
        employee_expenses: Vec::new(),
    });
    // Every FederalDeductions field is owned by a page (T3-deduct); a new
    // record starts with each page's "nothing entered".
    d.deductions.get_or_insert_with(|| FederalDeductions {
        vehicle_recoveries: Default::default(),
        investment_interest: None,
        mortgage: Vec::new(),
        repayment: RepaymentClaim::None,
        itemize_override: None,
        itemized: no_itemized_facts(),
        spouse_itemizes: None,
        personal_casualties: Vec::new(),
        annuity_losses: Vec::new(),
        vehicle_loans: Vec::new(),
        cash_charity: 0,
        fund_cash_charity: 0,
        other_cash_charity: 0,
        property_gifts: Vec::new(),
        vehicle_gifts: Vec::new(),
        vehicle_gift_documents: Vec::new(),
        appraisals: Vec::new(),
        charity_carryovers: Vec::new(),
        basis_election: false,
        volunteer_expenses: Vec::new(),
        hosted_students: Vec::new(),
        whaling_captains: Vec::new(),
    });
    if d.credits.is_none() {
        d.credits = Some(FederalCredits {
            decline_earned_income_credit: false,
            tip_fica_carryforward: None,
            business_credits: None,
            bba_audit_interest: None,
            tax_credit_bonds: Vec::new(),
            after_disallowance: None,
            minimum_tax: None,
            mortgage_credit: None,
            mortgage_subsidies: Vec::new(),
            dc_homebuyer_carry: None,
            residential_energy_carry: None,
            personal_refueling: Vec::new(),
            clean_vehicles: None,
            legacy_ev_passive_allowed: None,
            education: Vec::new(),
            students: Vec::new(),
            education_recapture: None,
            saver: None.into(),
            foreign_tax: ForeignTaxClaim::No,
            exempt_pensions: Vec::new(),
            adoptions: Vec::new(),
            adoption_carry: Vec::new(),
            marketplace: MarketplaceClaim::No,
            care: FederalCare::default(),
        });
    }
    d.screening.get_or_insert_with(|| FederalScreening {
        clergy: Vec::new(),
        farm_averaging: None,
        optional_se: Vec::new(),
        foreign_earned: Vec::new(),
        farm_nol: None,
        household_employment: Vec::new(),
        able_contributions: Vec::new(),
        trump_accounts: Vec::new(),
        public_safety: Vec::new(),
        iso_exercises: Vec::new(),
        espp_acquisitions: Vec::new(),
        espp_sale_answers: Vec::new(),
        iso_sale_answers: Vec::new(),
        nua_sale_answers: Vec::new(),
        capital_history: Vec::new(),
        section1231: None,
        atnol: AtnolHistory::None,
        nol: NolHistory::None,
        qbi_history: QbiHistory::NONE,
        qbi_aggregations: Vec::new(),
        foreign_account: false,
        fbar_required: None,
        fbar_countries: Vec::new(),
        foreign_trust: false,
        disaster_carries: Vec::new(),
        foreign_asset_max: None,
        installment_payments: Vec::new(),
        section_1256_carryback: None,
        section_1256_carryback_owners: None,
        related_exchanges: Vec::new(),

        taxable_premium_election: None,
    });
}
// Issue keys of designed scope boundaries: no answer can complete the return.
const SCOPE_REFUSALS: [&str; 1] = ["SECTION179_CARE_EARNED_SOURCE_REQUIRED"];

// The refund or balance due, or why there is no estimate yet.
pub(crate) fn estimate_text(result: &FederalResult) -> Vec<u8> {
    use crate::{Message, MessageArg};
    // The refusal itself is an item of the list, never repeated here; the line
    // only says that no answer can complete it. Unsupported alone also marks
    // answers still to come, so only a designed scope boundary refuses.
    if result.state == ResultState::Unsupported
        && result.issues.iter().any(|issue| SCOPE_REFUSALS.contains(&issue.display.key.as_str()))
    {
        return Message::new("ESTIMATE_OUTSIDE_COVERAGE", alloc::vec![]).encode();
    }
    if result.state != ResultState::EstimatedForSupportedScope {
        return Message::new("ESTIMATE_UNAVAILABLE", alloc::vec![]).encode();
    }
    if result.refund.is_none() {
        // A retained planned credit may await source-complete capacity while
        // the ordinary supported-scope tax remains available. Cash is not zero.
        return Message::new("PAYMENT_APPLIED_CREDIT_UNAVAILABLE", alloc::vec![]).encode();
    }
    let (key, cents) = match (result.refund, result.balance) {
        (Some(refund), _) if refund > 0 => ("ESTIMATED_REFUND", Some(refund)),
        (_, Some(balance)) if balance > 0 => ("ESTIMATED_BALANCE", Some(balance)),
        _ => ("ESTIMATE_ZERO_BALANCE", None),
    };
    Message::new(key, cents.into_iter().map(MessageArg::usd_cents).collect()).encode()
}
// A partnership K-1's Schedule E line 28 amount after its basis, at-risk and
// own passive limits (the row preview, design A3a): the box 1 part after
// those limits, the guaranteed payments and a qualified 2025 fuel credit.
// None while the calculation refuses the K-1, and while a return-level limit
// decides it: a passive loss (Form 8582), a section 179 deduction (Form
// 4562) or a box 2 or 3 rental activity (Form 8582 and the recorded real
// estate professional status).
fn partner_amount(data: &PartnerK1) -> Option<i64> {
    if !data.complete()
        || data.regular.rentals.any()
        || data.regular.carried.waits_for_return_forms()
        || !data.unentered.is_empty()
        || data.qbi.is_none()
        || data.section179 != 0
        || data.section179_carryover != 0
        || !data.valid()
    {
        return None;
    }
    let pool = data.pool(&data.regular).ok()?;
    if pool.passive < 0 {
        return None;
    }
    let fuel = match &data.prior_fuel_credit {
        None => 0,
        Some(prior) if prior.in_scope() => crate::kernel::ratio::dollars(prior.taxable_amount()),
        Some(_) => 0,
    };
    Some(
        (crate::kernel::ratio::dollars(pool.nonpassive + pool.passive)
            + crate::kernel::ratio::dollars(data.guaranteed_services)
            + crate::kernel::ratio::dollars(data.guaranteed_capital)
            + fuel)
            * 100,
    )
}
// A document's preview inside the return: a prior-year recovery and a state
// refund of the same year share one Worksheet 2, so each shows its own share of it.
fn preview_amount(d: &FederalDraft, entry: &FederalIncomeEntry) -> Option<i64> {
    if matches!(&entry.income, FederalIncome::Rental { data } if data.buildings.iter().any(|b| b.energy.is_some()))
    {
        let result = estimate(d);
        return result
            .lines
            .iter()
            .find(|line| line.id == alloc::format!("rental_{}_line21_income", entry.id))
            .map(|line| line.amount);
    }

    let graph_kind = match &entry.income {
        FederalIncome::PartnershipActivities { data } => {
            data.before_section179(false).ok()?;
            data.before_section179(true).ok()?;
            Some("partnership")
        }
        FederalIncome::ScorpActivities { data } => {
            data.before_section179(false).ok()?;
            data.before_section179(true).ok()?;
            Some("scorp")
        }
        _ => None,
    };
    if let Some(kind) = graph_kind {
        let result = estimate(d);
        result.tax?;
        return result
            .lines
            .iter()
            .find(|row| row.id == alloc::format!("{kind}_{}_schedule_e", entry.id))
            .map(|row| row.amount);
    }

    if let FederalIncome::MedicalDistribution { data } = &entry.income
        && let data = &crate::FederalMedicalDistribution::from(data)
        && let Some(crate::MedicalSaPayment::Own { medical, rollover: Some(rollover), .. }) =
            data.effective_payment()
    {
        if !data.valid() || data.rule().is_some() {
            return None;
        }
        let projection = crate::medical_rollover_return::classify(d).ok()?;
        use crate::kernel::ratio::exact::{self, Exact};
        let failed: Exact = projection
            .failed
            .iter()
            .filter(|p| p.source == entry.id)
            .map(|p| exact::whole(p.deposit.amount))
            .sum();
        return exact::rounded(
            &(exact::whole(data.gross) - exact::whole(*medical) - (rollover.amount() - failed)),
        );
    }
    if matches!(&entry.income,FederalIncome::MedicalDistribution{data} if matches!(data.account(),crate::MedicalSaAccount::Archer) && matches!(crate::FederalMedicalDistribution::from(data).effective_payment(),Some(crate::MedicalSaPayment::ExcessReturned)))
        && d.adjustments.as_ref().is_some_and(|a| {
            crate::hsa_correction::archer_linked(&a.archer, entry.id, entry.owner.owned())
        })
    {
        return None;
    }
    let year = match &entry.income {
        FederalIncome::OtherIncome { data } => match &data.kind {
            crate::OtherIncomeKind::Recovery { worksheet } if worksheet.ordinary => {
                Some((worksheet.year, false))
            }
            _ => None,
        },
        FederalIncome::GovernmentPayment { data }
            if data.form.state_or_local_income_tax_refunds != 0 =>
        {
            crate::FederalTaxRefund::from(data).history.single_year().map(|y| (y, true))
        }
        _ => None,
    };
    // An Olympic or Paralympic prize its exclusion removes previews as zero.
    if matches!(
        &entry.income,
        FederalIncome::OtherIncome { data } if matches!(data.kind, crate::OtherIncomeKind::OlympicPrize)
    ) && estimate(d).lines.iter().any(|line| line.id == "olympic_exclusion" && line.amount > 0)
    {
        return Some(0);
    }
    if let Some((year, refund)) = year
        && let Some(found) = crate::other_income::worksheets(d).into_iter().find(|w| w.year == year)
    {
        return if refund {
            let FederalIncome::GovernmentPayment { data } = &entry.income else {
                unreachable!("a refund worksheet belongs to a government payment source")
            };
            found.sheet.to_line_1.checked_add(crate::FederalUnemployment::from(data).amount)
        } else {
            Some(found.sheet.to_8z)
        };
    }
    document_amount(&entry.income, crate::federal_rules::separate_return(d))
}
// A document's amount on this return: its share after the Community or
// separate answer (Form 8958), the whole amount on any other return.
pub(crate) fn community_amount(
    d: &FederalDraft,
    entry: &FederalIncomeEntry,
    amount: Option<i64>,
) -> Option<i64> {
    // Energy previews come from the already allocated native rental output.
    if matches!(&entry.income, FederalIncome::Rental { data } if data.buildings.iter().any(|cost| cost.energy.is_some()))
    {
        return amount;
    }
    let (Some(setup), Some(amount)) = (d.setup.as_ref(), amount) else {
        return amount;
    };
    allocated_preview_amount(
        entry,
        amount,
        crate::community::entry_share(setup, entry),
        crate::federal_rules::separate_return(d),
    )
}

pub(crate) fn allocated_preview_amounts(
    entry: &FederalIncomeEntry,
    amount: i64,
    share: crate::community::Share,
    separate: bool,
) -> Option<[i64; 2]> {
    Some([
        allocated_preview_amount(entry, amount, Some(share), separate)?,
        allocated_preview_amount(entry, amount, Some(share.counterpart()), separate)?,
    ])
}

fn allocated_preview_amount(
    entry: &FederalIncomeEntry,
    amount: i64,
    share: Option<crate::community::Share>,
    separate: bool,
) -> Option<i64> {
    let Some(share) = share else { return Some(amount) };
    let (ordinary, gain, fund_gain, exclusion) = match &entry.income {
        FederalIncome::Dividend { data } => {
            let data = crate::FederalDividend::from(data);
            (
                data.ordinary,
                data.capital_distributions,
                data.section_1202_gain,
                data.section_1202_exclusion,
            )
        }
        FederalIncome::UndistributedGain { data } => {
            let data = crate::UndistributedGain::from(data);
            (0, data.gain, data.small_business, data.section_1202_exclusion)
        }
        _ => return Some(share.filer(amount)),
    };
    let fund = crate::investment_statements::allocated_fund_gain(
        fund_gain,
        exclusion,
        Some(share),
        separate,
    );
    crate::kernel::ratio::checked_sum([share.filer(ordinary), share.gain(gain), -fund.excluded])
}
// `separate`: a married person filing separately (the fund section 1202 issuer limit is halved).
pub(crate) fn document_amount(value: &FederalIncome, separate: bool) -> Option<i64> {
    Some(match value {
        // Broker components can have different tax treatments and unresolved
        // records; the shared final result owns their combined display amount.
        FederalIncome::Broker { .. } => return None,
        // Actual final owner output is required; original boxes are source controls.
        FederalIncome::PartnershipActivities { .. } | FederalIncome::ScorpActivities { .. } => {
            return None;
        }
        FederalIncome::Able { data } => return crate::AbleDistribution::from(data).taxable(),
        FederalIncome::TrumpDeath { data: source } => {
            let data = &crate::TrumpDeathAcquisition::from(source);
            return data.valid().then(|| data.taxable()).flatten();
        }
        FederalIncome::TrumpAbleRollover { data: source } => {
            let data = &crate::TrumpAbleRollover::from(source);
            return data.valid().then_some(0);
        }
        // The limit is shared among the insured's forms: only an excluded form shows 0.
        FederalIncome::LtcBenefits { data } => return data.record().preview(),
        // Depletion is a return-level figure (its 65 percent limit and carryover); the item
        // Form 8958 lists is the receipts less the expenses.
        FederalIncome::Royalty { data } => {
            return crate::kernel::ratio::exact::rounded(
                &(crate::kernel::ratio::exact::whole(data.receipts)
                    - crate::RentalExpense::total_exact(&data.expenses)),
            );
        }
        FederalIncome::LtcInsured { .. } => return None,
        FederalIncome::MedicalDistribution { data } => {
            return crate::FederalMedicalDistribution::from(data).preview();
        }
        FederalIncome::NonbusinessProperty { receipts, expenses, .. } => receipts - expenses,
        FederalIncome::SmallBusinessStock { data } => -data.loss(),
        FederalIncome::QualifiedSmallBusinessSale { .. } => return None,
        FederalIncome::GovernmentPayment { data } => {
            let payments = crate::FederalUnemployment::from(data).amount;
            if data.form.state_or_local_income_tax_refunds == 0 {
                payments
            } else {
                let crate::FederalTaxRefund { amount, history, .. } = data.into();
                return payments.checked_add(history.taxable(amount).ok()?);
            }
        }
        FederalIncome::WorthlessSecurity { data } => -data.basis,
        FederalIncome::NonbusinessBadDebt { data } => -data.source.unrecovered_basis(),
        FederalIncome::HomeSale { .. } => return None,
        // The exclusion depends on the return's income and credits.
        FederalIncome::SavingsBonds { .. } | FederalIncome::BondRedemption { .. } => return None,
        FederalIncome::Section1256 { data } => data.form.aggregate,
        // The taxable earnings depend on the expense record and the credits.
        FederalIncome::EducationDistribution { data } => {
            return crate::FederalEducationDistribution::from(data).preview();
        }
        FederalIncome::EducationAccountExpenses { .. } => 0,
        FederalIncome::ReturnOfCapital { amount, basis, kind, property, .. } => {
            let total = crate::kernel::ratio::checked_sum(
                core::iter::once(*amount).chain(property.iter().map(|property| property.value)),
            )?;
            if kind.needs_liquidation_stage(total, *basis) {
                return None;
            }
            kind.gain(total, *basis)
        }
        // The fund's whole gain less the part its section 1202 exclusion takes out.
        FederalIncome::UndistributedGain { data } => {
            let data = &crate::UndistributedGain::from(data);
            data.gain
                - crate::qsbs::fund_gain(data.small_business, data.section_1202_exclusion, separate)
                    .excluded
        }
        // The preview follows the estimate's working copy: a firm or sale left
        // out of the estimate shows nothing.
        FederalIncome::HobbySales { data } => data.on_return(),
        FederalIncome::PersonalDebt { data } => data.form.amount,
        FederalIncome::PropertyAcquisition { data } => {
            crate::PropertyAcquisition::from(data).realized()
        }
        FederalIncome::Cooperative { data } => crate::FederalCooperative::from(data).taxable(),
        // Excluded payments preview as zero; an unconfirmed one is income.
        FederalIncome::HealthLoanAssistance { data } => data.on_return(),
        FederalIncome::WildfireRelief { data } => data.on_return(),
        FederalIncome::DisasterRelief { data } => data.on_return(),
        FederalIncome::CasualtyLivingInsurance { data } => {
            if data.prior_taxable_reported != data.expected_prior_taxable() {
                return None;
            }
            data.included_in_return_year()
        }
        FederalIncome::DisasterMitigation { .. } => 0,
        FederalIncome::OtherIncome { data } => data.preview(),
        FederalIncome::Nec1099 { data } => crate::FederalForm1099Nec::from(data).compensation,
        FederalIncome::K1099 { data } => crate::FederalForm1099K::from(data).income(),
        FederalIncome::Misc1099 { data } => crate::FederalForm1099Misc::from(data).income(),
        FederalIncome::Scholarship { data } => data.taxable().0,
        FederalIncome::JuryDuty { data } => data.received,
        FederalIncome::Alimony { data } => data.amount(),
        FederalIncome::Gambling { data } => FederalGambling::from(data).winnings,
        FederalIncome::ForeignEmployment { data: v } => v.wages,
        FederalIncome::Rental { data: v } if !v.related.is_empty() => return None,
        FederalIncome::Rental { data: v } => return crate::extended_rules::rental_net(v).ok(),
        // Its line 32 or 34c depends on the whole return's passive limits.
        FederalIncome::FarmRental { .. } => return None,
        FederalIncome::Wage { data: v } => {
            let v = &crate::FederalWage::from(v);
            return v.total_income().ok().map(|amount| {
                amount
                    - v.deferred_pension
                    - v.inmate_pay
                    - if v.statutory_employee { v.wages } else { 0 }
            });
        }
        FederalIncome::EmployeeWages { data } => return data.income().ok(),
        // The income the election adds to your return: line 6 (nothing for a
        // child who need not file).
        FederalIncome::ChildElection { data } => {
            let lines = data.lines();
            if lines.reported() { lines.line_6 * 100 } else { 0 }
        }
        FederalIncome::Interest { data: v } => crate::FederalInterest::from(v).net_taxable(),
        FederalIncome::Dividend { data } => {
            let v = &crate::FederalDividend::from(data);
            let mut parts = alloc::vec![
                v.ordinary,
                v.capital_distributions,
                -crate::qsbs::fund_gain(v.section_1202_gain, v.section_1202_exclusion, separate)
                    .excluded,
            ];
            for lot in &v.capital_returns {
                let amount = crate::kernel::ratio::checked_sum(
                    core::iter::once(lot.amount)
                        .chain(lot.property.iter().map(|property| property.value)),
                )?;
                if lot.kind.needs_liquidation_stage(amount, lot.basis) {
                    return None;
                }
                parts.push(lot.kind.gain(amount, lot.basis));
            }
            crate::kernel::ratio::checked_sum(parts)?
        }
        FederalIncome::Oid { data: v } => crate::FederalOid::from(v).net_taxable(),
        FederalIncome::EstateTrustPortfolio { data } => {
            let data = &crate::FederalEstateTrustPortfolioK1::from(data);
            if data.has_activity_boxes() {
                return None;
            }
            if !data.complete_portfolio_only
                || !data.unentered.is_empty()
                || (data.section199a_dividends.amount() > 0
                    && (!data.code_i_dividends_only
                        || data.section199a_dividends.qualified()
                            != data.section199a_dividends.amount()))
                || (data.box_10_interest_ird_estate_tax > 0
                    && (!data.box_10_interest_ird_only
                        || data.interest == 0
                        || data.niit_adjustment.is_some()))
                || (data.amt_box12a != 0
                    && !data.amt_box12a_standalone
                    && !data.amt_box12_components_confirmed)
                || (data.royalty_income > 0 && !data.box_5_royalty_only)
                || (data.box_9_royalty_depreciation == 0 && data.box_9_royalty_sound_recording_only)
                || (data.box_9_royalty_depreciation > 0
                    && (data.box_9_royalty_depreciation > data.royalty_income
                        || data.box_9_royalty_depletion > 0
                        || data.box_9_royalty_amortization > 0
                        || !data.box_9_royalty_sound_recording_only
                        || data.final_successor()
                        || data.niit_adjustment.is_some()))
                || (data.box_9_royalty_depletion == 0 && data.box_9_royalty_cost_depletion_only)
                || (data.box_9_royalty_depletion > 0
                    && (data.box_9_royalty_depletion > data.royalty_income
                        || data.box_9_royalty_amortization > 0
                        || data.box_9_royalty_depreciation > 0
                        || !data.box_9_royalty_cost_depletion_only
                        || data.final_successor()
                        || data.niit_adjustment.is_some()))
                || (data.box_9_royalty_amortization > 0
                    && (data.box_9_royalty_amortization > data.royalty_income
                        || data.box_9_royalty_depletion > 0
                        || data.box_9_royalty_depreciation > 0
                        || !data.box_9_royalty_amortization_only
                        || data.niit_adjustment.is_some()))
                || ((data.final_67e_deduction > 0
                    || data.final_personal_property_tax > 0
                    || data.final_real_estate_tax > 0
                    || data.final_state_income_tax > 0
                    || data.final_state_sales_tax > 0
                    || data.final_short_carry > 0
                    || data.final_long_carry > 0
                    || data.final_nol.is_some())
                    && !data.final_successor())
                || data.final_nol.as_ref().is_some_and(|nol| !nol.valid())
                || (data.final_67e_deduction > 0 && !data.final_67e_no_niit_allocation)
                || ((data.final_personal_property_tax > 0
                    || data.final_real_estate_tax > 0
                    || data.final_state_income_tax > 0
                    || data.final_state_sales_tax > 0)
                    && !data.final_taxes_only)
                || ((data.final_short_carry > 0 || data.final_long_carry > 0)
                    && !data.final_carry_same_tax_character)
            {
                return None;
            }
            data.interest
                + data.ordinary_dividends
                + data.short_gain
                + data.long_gain
                + data.royalty_income
                - data.box_9_royalty_depreciation
                - data.box_9_royalty_depletion
                - data.box_9_royalty_amortization
                + data.activity_net()
                - data.final_short_carry
                - data.final_long_carry
                - data.final_67e_deduction
        }
        FederalIncome::NuaSale { data } => {
            if !data.confirmed() {
                return None;
            }
            data.parts().iter().map(|v| v.gain()).sum()
        }
        FederalIncome::EmployeeStockSale { data } => {
            if data.special {
                return None;
            }
            data.sale().gain()
        }
        FederalIncome::EsppSaleSource { .. }
        | FederalIncome::IsoSaleSource { .. }
        | FederalIncome::NuaSaleSource { .. } => return None,
        FederalIncome::EsppSale { data } => {
            if data.special {
                return None;
            }
            data.sale().gain() + if data.in_wages { 0 } else { data.compensation() }
        }
        FederalIncome::IsoSale { data } => {
            if !data.confirmed {
                return None;
            }
            data.sale().gain() + if !data.in_wages { data.compensation() } else { 0 }
        }
        FederalIncome::InstallmentSale { data } => {
            if data.refusal().is_some() {
                return None;
            }
            data.calculation().ok()?.map_or_else(
                || {
                    let gain = data.full_sale().gain();
                    if data.buyer != crate::Buyer::Unrelated { gain.max(0) } else { gain }
                },
                |current| current.gain() * 100,
            )
        }
        FederalIncome::Sale { data } => crate::FederalSale::from(data).gain(),
        // The deferred gain is included on December 31, 2026 or when the investment was sold.
        FederalIncome::OpportunityFund { data } => {
            let inclusion = data.inclusion()?;
            inclusion.short + inclusion.long + inclusion.section_1231
        }
        FederalIncome::OpportunityFundDeferral { data } => -(data.short_term + data.long_term),
        FederalIncome::Scorp { data } => return crate::ScorpK1::from(data).schedule_e_amount(),
        FederalIncome::Partnership { data } => return partner_amount(data),
        FederalIncome::Roth { .. } | FederalIncome::InheritedIra { .. } => return None,
        FederalIncome::Retirement { data: source } => {
            let v = &crate::FederalRetirement::from(source);
            // An inherited IRA payment is taxed with its record's group.
            if matches!(
                v.taxable,
                Some(crate::RetirementTaxable::IraBasis)
                    | Some(crate::RetirementTaxable::Inherited { .. })
            ) {
                return None;
            }
            // A corrective distribution from an employer plan is wages less the excess
            // deferral already counted from the W-2s (Form 1040 line 1h).
            let counted = match v.ordinary_distribution {
                Some(RetirementTreatment::Corrective { excess_deferral }) if !v.kind.ira() => {
                    excess_deferral
                }
                _ => 0,
            };
            v.taxable_income() - counted
                + match &v.taxable {
                    Some(crate::RetirementTaxable::LegacyAnnuity { data }) => {
                        if !data.confirmed() {
                            return None;
                        }
                        0
                    }
                    Some(crate::RetirementTaxable::PropertyProceeds { data }) => {
                        if !data.confirmed {
                            return None;
                        }
                        data.sale(v.gross, v.rolled_amount()).gain()
                    }
                    _ => 0,
                }
        }
        FederalIncome::SocialSecurity { data } => crate::FederalSocial::from(data).benefits,
        FederalIncome::Business { data: v } => {
            // Taxpayer-wide farm limits need the return context, not a document-only preview.
            if matches!(&v.activity, crate::BusinessActivity::Farming { data }
                if data.supplies.is_some() || matches!(data.conservation, Some(crate::FarmConservation::Deduct { .. })))
            {
                return None;
            }
            // A standalone document preview lacks the return-wide MACRS convention.
            if v.assets.iter().any(|a| {
                a.needs_macrs()
                    || a.available179() > 0
                    || a.macrs_source.as_ref().is_some_and(|s| s.improvement.is_some())
            }) {
                return None;
            }
            v.receipts() - v.expense_total(false)
        }
    })
}

#[path = "../../home/page.rs"]
mod home_page;

// Per-form modules split out by tools/split-hubs; the tool regenerates this block.
#[path = "../../deductions/adjustments_page.rs"]
mod adjustments_page;
#[path = "../../deductions/charity_page.rs"]
mod charity_page;
#[path = "../../core/checklist_page.rs"]
mod checklist_page;
#[path = "../../credits/credit_questions.rs"]
mod credit_questions;
#[path = "../../credits/credits_page.rs"]
mod credits_page;
#[path = "../../family/dependents_page.rs"]
mod dependents_page;
#[path = "../../home/disaster_page.rs"]
mod disaster_page;
#[path = "../../deductions/educator_page.rs"]
mod educator_page;
#[path = "../../family/filers_page.rs"]
mod filers_page;
#[path = "../../deductions/form_1098_e_page.rs"]
mod form_1098_e_page;
#[path = "../../home/form_1098_page.rs"]
mod form_1098_page;
#[path = "../../investment/form_1099_b_page.rs"]
mod form_1099_b_page;
#[path = "../../investment/form_1099_div_page.rs"]
mod form_1099_div_page;
#[path = "../../payments/form_1099_g_page.rs"]
mod form_1099_g_page;
#[path = "../../investment/form_1099_int_page.rs"]
mod form_1099_int_page;
#[path = "../../investment/form_1099_oid_page.rs"]
mod form_1099_oid_page;
#[path = "../../farm/form_1099_patr_page.rs"]
mod form_1099_patr_page;
#[path = "../../family/form_1099_q_page.rs"]
mod form_1099_q_page;
#[path = "../../retirement/form_1099_r_page.rs"]
mod form_1099_r_page;
#[path = "../../business/form_2106_page.rs"]
mod form_2106_page;
#[path = "../../family/form_2441_page.rs"]
mod form_2441_page;
#[path = "../../family/form_8615_page.rs"]
mod form_8615_page;
#[path = "../../investment/form_8815_page.rs"]
mod form_8815_page;
#[path = "../../family/form_8863_page.rs"]
mod form_8863_page;
#[path = "../../credits/form_8880_page.rs"]
mod form_8880_page;
#[path = "../../health/form_8889_page.rs"]
mod form_8889_page;
#[path = "../../health/form_8962_page.rs"]
mod form_8962_page;
#[path = "../../attribution/injured_spouse_page.rs"]
mod injured_spouse_page;
#[path = "../../investment/investment_statements_page.rs"]
mod investment_statements_page;
#[path = "../../retirement/ira_contributions_page.rs"]
mod ira_contributions_page;
#[path = "../../payments/other_taxes_page.rs"]
mod other_taxes_page;
#[path = "../../payments/payments_page.rs"]
mod payments_page;
#[path = "../../deductions/schedule_a_page.rs"]
mod schedule_a_page;
#[path = "../../business/schedule_c_page.rs"]
mod schedule_c_page;
#[path = "../../investment/schedule_e_page.rs"]
mod schedule_e_page;
#[path = "../../health/se_benefits_page.rs"]
mod se_benefits_page;
#[path = "../../retirement/ssa_1099_page.rs"]
mod ssa_1099_page;
#[path = "../../income/w2_page.rs"]
mod w2_page;
#[path = "../../income/wages_answers_page.rs"]
mod wages_answers_page;
#[allow(unused_imports)]
pub use adjustments_page::*;
#[allow(unused_imports)]
pub use charity_page::*;
#[allow(unused_imports)]
pub use checklist_page::*;
#[allow(unused_imports)]
pub use credits_page::*;
#[allow(unused_imports)]
pub use dependents_page::*;
#[allow(unused_imports)]
pub use disaster_page::*;
#[allow(unused_imports)]
pub use educator_page::*;
#[allow(unused_imports)]
pub use filers_page::*;
#[allow(unused_imports)]
pub use form_1098_e_page::*;
#[allow(unused_imports)]
pub use form_1098_page::*;
#[allow(unused_imports)]
pub use form_1099_b_page::*;
#[allow(unused_imports)]
pub use form_1099_div_page::*;
#[allow(unused_imports)]
pub use form_1099_g_page::*;
#[allow(unused_imports)]
pub use form_1099_int_page::*;
#[allow(unused_imports)]
pub use form_1099_oid_page::*;
#[allow(unused_imports)]
pub use form_1099_patr_page::*;
#[allow(unused_imports)]
pub use form_1099_q_page::*;
#[allow(unused_imports)]
pub use form_1099_r_page::*;
#[allow(unused_imports)]
pub use form_2106_page::*;
#[allow(unused_imports)]
pub use form_2441_page::*;
#[allow(unused_imports)]
pub use form_8615_page::*;
#[allow(unused_imports)]
pub use form_8815_page::*;
#[allow(unused_imports)]
pub use form_8863_page::*;
#[allow(unused_imports)]
pub use form_8880_page::*;
#[allow(unused_imports)]
pub use form_8889_page::*;
#[allow(unused_imports)]
pub use form_8962_page::*;
#[allow(unused_imports)]
pub use injured_spouse_page::*;
#[allow(unused_imports)]
pub use investment_statements_page::*;
#[allow(unused_imports)]
pub use ira_contributions_page::*;
#[allow(unused_imports)]
pub use other_taxes_page::*;
#[allow(unused_imports)]
pub use payments_page::*;
#[allow(unused_imports)]
pub use schedule_a_page::*;
#[allow(unused_imports)]
pub use schedule_c_page::*;
#[allow(unused_imports)]
pub use schedule_e_page::*;
#[allow(unused_imports)]
pub use se_benefits_page::*;
#[allow(unused_imports)]
pub use ssa_1099_page::*;
#[allow(unused_imports)]
pub use w2_page::*;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) const HUB_SOURCE: &str = concat!(
    include_str!("mod.rs"),
    include_str!("../../deductions/adjustments_page.rs"),
    include_str!("../../deductions/charity_page.rs"),
    include_str!("../../core/checklist_page.rs"),
    include_str!("../../credits/credits_page.rs"),
    include_str!("../../family/dependents_page.rs"),
    include_str!("../../home/disaster_page.rs"),
    include_str!("../../deductions/educator_page.rs"),
    include_str!("../../family/filers_page.rs"),
    include_str!("../../deductions/form_1098_e_page.rs"),
    include_str!("../../home/form_1098_page.rs"),
    include_str!("../../investment/form_1099_b_page.rs"),
    include_str!("../../investment/form_1099_div_page.rs"),
    include_str!("../../payments/form_1099_g_page.rs"),
    include_str!("../../investment/form_1099_int_page.rs"),
    include_str!("../../investment/form_1099_oid_page.rs"),
    include_str!("../../farm/form_1099_patr_page.rs"),
    include_str!("../../family/form_1099_q_page.rs"),
    include_str!("../../retirement/form_1099_r_page.rs"),
    include_str!("../../business/form_2106_page.rs"),
    include_str!("../../family/form_2441_page.rs"),
    include_str!("../../family/form_8615_page.rs"),
    include_str!("../../investment/form_8815_page.rs"),
    include_str!("../../family/form_8863_page.rs"),
    include_str!("../../credits/form_8880_page.rs"),
    include_str!("../../health/form_8889_page.rs"),
    include_str!("../../health/form_8962_page.rs"),
    include_str!("../../attribution/injured_spouse_page.rs"),
    include_str!("../../investment/investment_statements_page.rs"),
    include_str!("../../retirement/ira_contributions_page.rs"),
    include_str!("../../payments/other_taxes_page.rs"),
    include_str!("../../payments/payments_page.rs"),
    include_str!("../../deductions/schedule_a_page.rs"),
    include_str!("../../business/schedule_c_page.rs"),
    include_str!("../../investment/schedule_e_page.rs"),
    include_str!("../../health/se_benefits_page.rs"),
    include_str!("../../retirement/ssa_1099_page.rs"),
    include_str!("../../income/w2_page.rs"),
    include_str!("../../income/wages_answers_page.rs"),
);
// End of the tools/split-hubs block.

// The CCC loans forfeited in 2026 that a Form 1099-A can name, keyed by
// the loan's own record identity.
pub(super) fn ccc_lots(draft: &FederalDraft) -> Vec<crate::RecordChoice> {
    crate::farm::sources(draft)
        .flat_map(|source| {
            let label = farm_choice_label(&source);
            source
                .farm
                .ccc
                .iter()
                .flat_map(|ccc| ccc.forfeited_lots())
                .map(|(index, id, principal)| crate::RecordChoice {
                    id,
                    label: crate::Message::new(
                        "CHOICE_CCC_LOAN",
                        alloc::vec![
                            crate::MessageArg::Nested(alloc::boxed::Box::new(label.clone())),
                            crate::MessageArg::Number(index as i64 + 1),
                            crate::MessageArg::usd_cents(principal),
                        ],
                    )
                    .encode(),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
