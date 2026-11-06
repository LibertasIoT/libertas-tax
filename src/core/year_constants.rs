// Coverage: single owner of the year-keyed, inflation-adjusted and statutory rate/limit constants
//   that are written in source (not table-driven). Role for the parity plan's step 4 (final-revision
//   recheck): when the IRS publishes a final Revenue Procedure, Notice, instruction or form, walk this
//   file top to bottom, compare every constant with its cited source, and change the value and its
//   comment here only; no call site embeds one of these amounts. A constant whose source line says
//   "not cited at the former site" was moved without a recorded citation and needs the source found
//   before the recheck can close.
//
//   Units are in each constant's name: DOLLARS (whole dollars, the unit of the whole-dollar form
//   lines), CENTS (the App's money unit) or HALF_CENTS (mileage rates, twice the cents so a 72.5
//   cent rate is an integer). Values are unchanged from the sites they were moved from.
//
//   A year in a constant's name (2025, 2026) names the year of the published figure, not the
//   return's year: phase C selects a rule set by the return's year from these year-named tables.
//
//   Fixture- or table-owned values that are deliberately NOT here (change them at their owner):
//   - EIC parameters and the earned-income credit tables: src/forms/schedule_eic.rs formulas and
//     src/historical_eic_tables.rs (prior-year tables); the 2026 EIC amounts remain in
//     schedule_eic.rs.
//   - The ordinary-tax Tax Table below $100,000 (Sep. 16 draft p1040): the table fixtures read by
//     src/income_tax.rs; its bracket, capital-gain and AMT year tables (2022-2026, match arms in
//     src/income_tax.rs AmtRates::for_year and the bracket function) are already one cited,
//     year-keyed owner and stay there.
//   - Underpayment (Form 2210) rate periods and the annualization columns in src/underpayment.rs
//     (derived from the wage base below, but kept as the form's line-4 table).
//   - Marketplace poverty guidelines and premium tables: src/marketplace.rs.
//   - Foreign housing location amounts (Notice 2026-25): src/foreign_housing.rs.
//   - MACRS rate tables: src/kernel/cost_recovery/macrs_tables.rs; RMD and annuity tables: src/rmd_tables.rs and
//     src/annuity_tables.rs.
// Keep this in plain `//` comments: `///` and `//!` text becomes UI labels, help and package
// documentation.

// ---- Standard deduction (Pub. 501 Table 8, using Rev. Proc. 2025-32 amounts for 2026) ----
// Standard deduction, single or married filing separately, 2026, whole dollars. Source: Rev. Proc.
// 2025-32 section 4.14(1).
pub(crate) const STANDARD_DEDUCTION_SINGLE_OR_SEPARATE_DOLLARS: i64 = 16_100;
// Standard deduction, head of household, 2026, whole dollars. Source: Rev. Proc. 2025-32 section
// 4.14(1).
pub(crate) const STANDARD_DEDUCTION_HEAD_DOLLARS: i64 = 24_150;
// Standard deduction, joint and surviving spouse, 2026, whole dollars. Source: Rev. Proc. 2025-32
// section 4.14(1).
pub(crate) const STANDARD_DEDUCTION_JOINT_DOLLARS: i64 = 32_200;
// Educator expense deduction per educator, 2026, whole dollars. Source: section 62(a)(2)(D) as
// indexed by Rev.Proc.2025-32 section4.12 (Schedule1 line11). Eligibility includes early
// childhood educators under P.L.119-111 divisionB section1 (Sept18,2026), effective for
// expenses in taxable years beginning after2025; §67(g)(2) carries it to Schedule A17k.
// https://www.govinfo.gov/content/pkg/PLAW-119publ111/html/PLAW-119publ111.htm
pub(crate) const EDUCATOR_EXPENSE_LIMIT_DOLLARS: i64 = 350;
// Dependent's standard deduction floor, 2026, whole dollars. Source: Rev. Proc. 2025-32 section
// 4.14(2); Form 1040 instructions, Standard Deduction Worksheet for Dependents.
pub(crate) const STANDARD_DEDUCTION_DEPENDENT_MINIMUM_DOLLARS: i64 = 1_350;
// Dependent's standard deduction: earned income plus this amount, 2026, whole dollars. Source:
// Rev. Proc. 2025-32 section 4.14(2); Form 1040 instructions, Standard Deduction Worksheet for
// Dependents.
pub(crate) const STANDARD_DEDUCTION_DEPENDENT_EARNED_ADDITION_DOLLARS: i64 = 450;
// Additional standard deduction per age-65/blind box, single or head of household, 2026, whole
// dollars. Source: Rev. Proc. 2025-32 section 4.14(3).
pub(crate) const STANDARD_DEDUCTION_AGE_BLIND_UNMARRIED_DOLLARS: i64 = 2_050;
// Additional standard deduction per age-65/blind box, married or surviving spouse, 2026, whole
// dollars. Source: Rev. Proc. 2025-32 section 4.14(3).
pub(crate) const STANDARD_DEDUCTION_AGE_BLIND_MARRIED_DOLLARS: i64 = 1_650;

// ---- Form 8615 / 8814 child's unearned income ----
// Form 8615 line 2 base standard-deduction amount (half the line 2 floor), 2026, whole dollars.
// Source: Rev. Proc. 2025-32 section 4.02 ($1,350); Form 8615 instructions "Who Must File".
pub(crate) const KIDDIE_BASE_DEDUCTION_DOLLARS: i64 = 1_350;
// Form 8615 unearned-income threshold and line 2 floor (twice the base), 2026, whole dollars.
// Source: Rev. Proc. 2025-32 section 4.02 (twice $1,350); Form 8615 instructions "Who Must File".
pub(crate) const KIDDIE_UNEARNED_THRESHOLD_DOLLARS: i64 = 2_700;

// ---- Form 8814 (parents' election to report a child's interest and dividends) ----
// Form 8814 line 13 amount not taxed: the section 1(g)(4)(A)(ii)(I) amount, 2026, whole dollars.
// Source: Rev. Proc. 2025-32 section 4.02 ("a child's gross income for 2026 must be more than
// $1,350 but less than $13,500"); Form 8814 (2026 draft) line 13.
pub(crate) const CHILD_ELECTION_UNTAXED_DOLLARS: i64 = 1_350;
// Form 8814 line 5 base amount: twice the amount above (section 1(g)(7)(B)(i)), 2026, whole
// dollars. Source: Rev. Proc. 2025-32 section 4.02; Form 8814 (2026 draft) line 5.
pub(crate) const CHILD_ELECTION_BASE_DOLLARS: i64 = 2_700;
// Form 8814 line 4 ceiling: the election is unavailable at or above ten times the amount above
// (section 1(g)(7)(A)(ii)), 2026, whole dollars. Source: Rev. Proc. 2025-32 section 4.02; Form
// 8814 (2026 draft) line 4.
pub(crate) const CHILD_ELECTION_LIMIT_DOLLARS: i64 = 13_500;

// ---- Social Security wage base and Medicare (Schedule SE, Form 8959, Form 8960) ----
// Social Security wage base, 2026, whole dollars. Source: i1040sse 2026 draft (Schedule SE
// line 7); SSA 2026 cost-of-living announcement ($184,500).
pub(crate) const SOCIAL_SECURITY_WAGE_BASE_DOLLARS: i64 = 184_500;
// Social Security wage base, 2026, cents. Source: same as the whole-dollar constant above; used by
// the Form W-2 and household-employer wage caps and the Form 2210 annualization columns. The W-2
// page's Social Security wage range and rule name it as `$SOCIAL_SECURITY_WAGE_BASE_CENTS`.
pub(crate) const SOCIAL_SECURITY_WAGE_BASE_CENTS: i64 = SOCIAL_SECURITY_WAGE_BASE_DOLLARS * 100;
// Maximum employee Social Security tax withheld on one wage base (6.2% of the wage base), 2026,
// cents. Source: derived, 6.2% (IRC 3101(a)) of the wage base: $11,439.
pub(crate) const MAX_EMPLOYEE_SOCIAL_SECURITY_TAX_CENTS: i64 = 1_143_900;
// Reporting-rounding tolerance for the W-2 payroll-rate checks (box 4 against 6.2% of box 3, box 6
// against 1.45% of box 5), cents. Employers compute payroll tax per pay period and may print
// whole-dollar W-2 amounts, so the printed annual totals can differ from the annual rate by up to
// $1.00 without any over- or under-withholding. Source: General Instructions for Forms W-2 and
// W-3 (whole-dollar entries permitted); IRC 3102(a) (tax withheld as wages are paid).
pub(crate) const W2_PAYROLL_RATE_ROUNDING_TOLERANCE_CENTS: i64 = 100;
// Additional Medicare Tax threshold, joint return, whole dollars. Statutory (IRC 3101(b)(2)),
// not indexed; Form 8959.
pub(crate) const ADDITIONAL_MEDICARE_THRESHOLD_JOINT_DOLLARS: i64 = 250_000;
// Additional Medicare Tax threshold, married filing separately, whole dollars. Statutory.
pub(crate) const ADDITIONAL_MEDICARE_THRESHOLD_SEPARATE_DOLLARS: i64 = 125_000;
// Additional Medicare Tax threshold, all other statuses, whole dollars. Statutory.
pub(crate) const ADDITIONAL_MEDICARE_THRESHOLD_OTHER_DOLLARS: i64 = 200_000;
// Net investment income tax threshold for a surviving spouse (shares the joint threshold under
// IRC 1411(b)), whole dollars. Statutory, not indexed; Form 8960.
pub(crate) const NIIT_THRESHOLD_SURVIVING_DOLLARS: i64 = 250_000;

// ---- Schedule 1-A (qualified tips, overtime, passenger vehicle loan interest) ----
// Statutory amounts of the 2025 reconciliation act (P.L. 119-21), cited in the draft f1040s1a;
// IRC 224 (tips, section 70201), IRC 225 (overtime, section 70202), IRC 163(h)(4) (vehicle loan
// interest, section 70203). Not indexed.
// Qualified tips deduction cap, whole dollars. Statutory (IRC 224(b)(1)).
pub(crate) const SCHEDULE_1A_TIPS_CAP_DOLLARS: i64 = 25_000;
// Qualified overtime deduction cap, single, whole dollars. Statutory (IRC 225(b)(2)).
pub(crate) const SCHEDULE_1A_OVERTIME_CAP_SINGLE_DOLLARS: i64 = 12_500;
// Qualified overtime deduction cap, joint return, whole dollars. Statutory (IRC 225(b)(2)).
pub(crate) const SCHEDULE_1A_OVERTIME_CAP_JOINT_DOLLARS: i64 = 25_000;
// MAGI above which the tips and overtime deductions phase out, single, whole dollars.
// Statutory (IRC 224(b)(2), 225(b)(3)).
pub(crate) const SCHEDULE_1A_WORK_PHASEOUT_START_SINGLE_DOLLARS: i64 = 150_000;
// MAGI above which the tips and overtime deductions phase out, joint, whole dollars.
// Statutory (IRC 224(b)(2), 225(b)(3)).
pub(crate) const SCHEDULE_1A_WORK_PHASEOUT_START_JOINT_DOLLARS: i64 = 300_000;
// Tips/overtime reduction: dollars per whole $1,000 of MAGI over the start (floored steps).
// Statutory (IRC 224(b)(2), 225(b)(3)).
pub(crate) const SCHEDULE_1A_WORK_REDUCTION_PER_THOUSAND_DOLLARS: i64 = 100;
// Passenger vehicle loan interest deduction cap, whole dollars. Statutory (IRC 163(h)(4)(A)(ii)).
pub(crate) const SCHEDULE_1A_VEHICLE_INTEREST_CAP_DOLLARS: i64 = 10_000;
// MAGI above which vehicle loan interest phases out, single, whole dollars.
// Statutory (IRC 163(h)(4)(A)(iii)).
pub(crate) const SCHEDULE_1A_VEHICLE_PHASEOUT_START_SINGLE_DOLLARS: i64 = 100_000;
// MAGI above which vehicle loan interest phases out, joint, whole dollars.
// Statutory (IRC 163(h)(4)(A)(iii)).
pub(crate) const SCHEDULE_1A_VEHICLE_PHASEOUT_START_JOINT_DOLLARS: i64 = 200_000;
// Vehicle interest reduction: dollars per $1,000 (or part, ceiling steps) of MAGI over the start.
// Statutory (IRC 163(h)(4)(A)(iii)).
pub(crate) const SCHEDULE_1A_VEHICLE_REDUCTION_PER_THOUSAND_DOLLARS: i64 = 200;

// ---- Child tax credit (Schedule 8812) ----
// Child tax credit per qualifying child, 2026, whole dollars. Source: IRC 24 as amended by
// P.L. 119-21 and Rev. Proc. 2025-32 section 4.05(1) ("$2,200").
pub(crate) const CHILD_TAX_CREDIT_PER_CHILD_DOLLARS: i64 = 2_200;
// Credit for other dependents, whole dollars. Statutory (IRC 24(h)(4)), not indexed.
pub(crate) const OTHER_DEPENDENT_CREDIT_DOLLARS: i64 = 500;
// Refundable (additional) child tax credit cap per child, 2026, whole dollars. Source: Rev. Proc.
// 2025-32 section 4.05(2) ("$1,700").
pub(crate) const ADDITIONAL_CHILD_TAX_CREDIT_PER_CHILD_DOLLARS: i64 = 1_700;
// Child tax credit phaseout start, MAGI, single/other, whole dollars. Statutory (IRC 24(b)).
pub(crate) const CHILD_TAX_CREDIT_PHASEOUT_START_OTHER_DOLLARS: i64 = 200_000;
// Child tax credit phaseout start, MAGI, joint, whole dollars. Statutory (IRC 24(b)).
pub(crate) const CHILD_TAX_CREDIT_PHASEOUT_START_JOINT_DOLLARS: i64 = 400_000;
// Credit reduction per $1,000 (or part) of MAGI over the start, whole dollars. Statutory.
pub(crate) const CHILD_TAX_CREDIT_REDUCTION_PER_THOUSAND_DOLLARS: i64 = 50;
// Earned income floor for the refundable child credit, whole dollars. Statutory (IRC 24(d)).
pub(crate) const CHILD_TAX_CREDIT_EARNED_INCOME_FLOOR_DOLLARS: i64 = 2_500;

// ---- Saver's credit (Form 8880; Notice 2025-67 supplies the 2026 boundaries) ----
// AGI limits of the 50% / 20% / 10% rates, whole dollars, joint return. Source: Notice 2025-67.
pub(crate) const SAVER_AGI_LIMITS_JOINT_DOLLARS: [i64; 3] = [48_500, 52_500, 80_500];
// AGI limits of the 50% / 20% / 10% rates, whole dollars, head of household. Source: Notice
// 2025-67.
pub(crate) const SAVER_AGI_LIMITS_HEAD_DOLLARS: [i64; 3] = [36_375, 39_375, 60_375];
// AGI limits of the 50% / 20% / 10% rates, whole dollars, all other statuses (including a
// surviving spouse). Source: Notice 2025-67.
pub(crate) const SAVER_AGI_LIMITS_OTHER_DOLLARS: [i64; 3] = [24_250, 26_250, 40_250];

// ---- Section 199A (Form 8995-A; Rev. Proc. 2025-32 and Form 8995-A) ----
// Taxable-income threshold where the wage/property limit begins, joint, whole dollars. Source:
// Rev. Proc. 2025-32 section 4.26 ($403,500).
pub(crate) const QBI_THRESHOLD_JOINT_DOLLARS: i64 = 403_500;
// Same threshold, married filing separately, whole dollars. Source: Rev. Proc. 2025-32 section
// 4.26 ($201,775).
pub(crate) const QBI_THRESHOLD_SEPARATE_DOLLARS: i64 = 201_775;
// Same threshold, all other statuses, whole dollars. Source: Rev. Proc. 2025-32 section 4.26
// ($201,750).
pub(crate) const QBI_THRESHOLD_OTHER_DOLLARS: i64 = 201_750;
// Phase-in range above the threshold, joint, whole dollars. Source: Rev. Proc. 2025-32 section
// 4.26 (phase-in range amount $553,500 less the $403,500 threshold).
pub(crate) const QBI_PHASE_RANGE_JOINT_DOLLARS: i64 = 150_000;
// Phase-in range above the threshold, other statuses, whole dollars. Source: Rev. Proc. 2025-32
// section 4.26 ($276,750 less $201,750; the separate column agrees: $276,775 less $201,775).
pub(crate) const QBI_PHASE_RANGE_OTHER_DOLLARS: i64 = 75_000;

// ---- HSA (Form 8889; 2026 contribution limits: Rev. Proc. 2025-19) ----
// Self-only HDHP annual contribution limit, 2026, cents. Source: Rev. Proc. 2025-19 ($4,400).
pub(crate) const HSA_SELF_ONLY_LIMIT_2026_CENTS: i64 = 440_000;
// Family HDHP annual contribution limit, 2026, cents. Source: Rev. Proc. 2025-19 ($8,750).
pub(crate) const HSA_FAMILY_LIMIT_2026_CENTS: i64 = 875_000;
// Age-55 catch-up contribution, cents. Statutory (IRC 223(b)(3)), not indexed.
pub(crate) const HSA_CATCH_UP_CENTS: i64 = 100_000;
// Self-only limit for the 2025 history used by the last-month rule, whole dollars. Source: Rev.
// Proc. 2024-25 ($4,300).
pub(crate) const HSA_SELF_ONLY_LIMIT_2025_DOLLARS: i64 = 4_300;
// Family limit for the 2025 history used by the last-month rule, whole dollars. Source: Rev.
// Proc. 2024-25 ($8,550).
pub(crate) const HSA_FAMILY_LIMIT_2025_DOLLARS: i64 = 8_550;
// Age-55 catch-up in the 2025 history, whole dollars. Statutory.
pub(crate) const HSA_CATCH_UP_DOLLARS: i64 = 1_000;

// ---- Section 179 (Form 4562 Part I; 2026 limits) ----
// Section 179 dollar limit, 2026, cents ($2,560,000). Source: Rev. Proc. 2025-32 section 4.24.
pub(crate) const SECTION_179_DOLLAR_LIMIT_2026_CENTS: i64 = 256_000_000;
// Section 179 investment-limit phaseout threshold, 2026, cents ($4,090,000). Source: Rev. Proc.
// 2025-32 section 4.24.
pub(crate) const SECTION_179_PHASEOUT_THRESHOLD_2026_CENTS: i64 = 409_000_000;

// Heavy passenger SUV section179 elected-cost limit, 2026, cents ($32,000).
// Source: IRC179(b)(5), Rev.Proc.2025-32 section4.24; statutory design exceptions are excluded.
pub(crate) const SECTION_179_SUV_LIMIT_2026_CENTS: i64 = 3_200_000;

// ---- Section 280F passenger-automobile limits (Rev. Proc. tables) ----
// By the calendar year the vehicle was placed in service, in cents: first year with the section
// 168(k) allowance, first year without it, second year, third year, each succeeding year. Lease
// inclusion Table 3 is not used by the App.
// Placed in service in 2025. Source: Rev. Proc. 2025-16 Tables 1 and 2:
// $20,200 / $12,200 / $19,600 / $11,800 / $7,060.
pub(crate) const PASSENGER_AUTOMOBILE_LIMITS_2025_CENTS: [i64; 5] =
    [2_020_000, 1_220_000, 1_960_000, 1_180_000, 706_000];
// Placed in service in 2026. Source: Rev. Proc. 2026-15 Tables 1 and 2:
// $20,300 / $12,300 / $19,800 / $11,900 / $7,160.
pub(crate) const PASSENGER_AUTOMOBILE_LIMITS_2026_CENTS: [i64; 5] =
    [2_030_000, 1_230_000, 1_980_000, 1_190_000, 716_000];

// ---- Standard mileage rates (Schedule C line 9, Form 4562 basis reduction) ----
// Business standard mileage rate, 2026 January through June, in half-cents per mile (72.5 cents).
// Source: IRS Notice 2026-10 (also cited by the 2026 draft Instructions for Form 2106).
pub(crate) const MILEAGE_RATE_2026_JAN_JUN_HALF_CENTS: i64 = 145;
// Business standard mileage rate, 2026 July through December, in half-cents per mile (76 cents).
// Source: IRS Announcement 2026-11 (the July 1, 2026 change; confirmed through secondary coverage,
// the IRB text itself was not retrieved at the 2026-10-08 map).
pub(crate) const MILEAGE_RATE_2026_JUL_DEC_HALF_CENTS: i64 = 152;
// Depreciation component of the standard mileage rate, 2026, in cents per mile (35 cents); the
// basis reduction for an owned vehicle. Source: IRS Notice 2026-10.
pub(crate) const MILEAGE_DEPRECIATION_COMPONENT_2026_CENTS_PER_MILE: i64 = 35;

// ---- Moving expenses (Form 3903 line 2) ----
// Moving standard mileage rate for travel through 2026-06-30, in half-cents per mile (20.5 cents).
// Source: Instructions for Form 3903 (2026 draft dated 2026-08-27), "What's New"; recheck against
// the IRS mileage-rate notice at the step 4 recheck.
pub(crate) const MOVING_MILEAGE_RATE_2026_THROUGH_JUN_HALF_CENTS: i64 = 41;
// Moving standard mileage rate for travel from 2026-07-01, in half-cents per mile (23.5 cents).
// Source: Instructions for Form 3903 (2026 draft dated 2026-08-27), "What's New"; same recheck.
pub(crate) const MOVING_MILEAGE_RATE_2026_FROM_JUL_HALF_CENTS: i64 = 47;
// First arrival date (yyyyMMdd) that takes the July-December moving rate. Source: same.
pub(crate) const MOVING_MILEAGE_RATE_CHANGE_DATE: u32 = 20260701;

// ---- IRA deduction phaseout for a worker covered by a workplace plan (Form 1040 IRA worksheet) ----
// MAGI start and range, covered, joint return, 2026, whole dollars (129,000 + 20,000). Source:
// Notice 2025-67 (IRA deduction phaseout for an active participant; confirmed against the notice
// text in the line audit of 2026-10-10).
pub(crate) const IRA_PHASEOUT_START_COVERED_JOINT_DOLLARS: i64 = 129_000;
// Width of the joint covered phaseout, whole dollars. Source: Notice 2025-67; IRC 219(g)(3)(B)(i)
// range for a joint return.
pub(crate) const IRA_PHASEOUT_RANGE_JOINT_DOLLARS: i64 = 20_000;
// MAGI start, covered, single/head, 2026, whole dollars (81,000 + 10,000). Source: Notice 2025-67.
pub(crate) const IRA_PHASEOUT_START_COVERED_SINGLE_DOLLARS: i64 = 81_000;
// Width of the single/head covered phaseout, whole dollars. Source: Notice 2025-67; IRC
// 219(g)(3)(B)(ii) range for other returns.
pub(crate) const IRA_PHASEOUT_RANGE_SINGLE_DOLLARS: i64 = 10_000;

// ---- Section 501(c)(18)(D) contributions (pension_contributions.rs; Notice 2025-67) ----
// Regular elective-deferral limit shared across plans, 2026, cents ($24,500). Source: Notice
// 2025-67 (IRC 402(g)(1)).
pub(crate) const ELECTIVE_DEFERRAL_LIMIT_2026_CENTS: i64 = 2_450_000;
// Age-50 catch-up on elective deferrals, 2026, cents ($8,000). Source: Notice 2025-67 (IRC
// 414(v)(2)(B)(i)).
pub(crate) const ELECTIVE_CATCH_UP_2026_CENTS: i64 = 800_000;
// Age 60 to 63 catch-up on elective deferrals, 2026, cents ($11,250). Source: Notice 2025-67 (IRC
// 414(v)(2)(E)(i)).
pub(crate) const ELECTIVE_CATCH_UP_AGES_60_63_2026_CENTS: i64 = 1_125_000;
// SIMPLE-only deferral limit, 2026, cents ($17,000). Source: Notice 2025-67 (IRC 408(p)(2)(A)).
pub(crate) const SIMPLE_DEFERRAL_LIMIT_2026_CENTS: i64 = 1_700_000;
// SIMPLE age-50 catch-up, 2026, cents ($4,000). Source: Notice 2025-67 (IRC 414(v)(2)(B)(ii)).
pub(crate) const SIMPLE_CATCH_UP_2026_CENTS: i64 = 400_000;
// SIMPLE age 60 to 63 catch-up, 2026, cents ($5,250). Source: Notice 2025-67 (IRC
// 414(v)(2)(E)(ii)).
pub(crate) const SIMPLE_CATCH_UP_AGES_60_63_2026_CENTS: i64 = 525_000;
// Section 501(c)(18)(D) contribution dollar cap, cents ($7,000). Statutory (IRC 501(c)(18)(D)(i));
// whether an indexed 2026 figure applies is not recorded here (doubtful, see the improvements
// table of 2026-10-09).
pub(crate) const SECTION_501C18D_CONTRIBUTION_CAP_CENTS: i64 = 700_000;

// ---- Medical (Rev. Proc. 2025-32 section 4.27) ----
// Eligible long-term care premium limit by attained age, 2026, cents: age 40 or under, 41-50,
// 51-60, 61-70, over 70. Source: Rev. Proc. 2025-32 section 4.27.
pub(crate) const LONG_TERM_CARE_PREMIUM_LIMITS_2026_CENTS: [i64; 5] =
    [50_000, 93_000, 186_000, 496_000, 620_000];

// ---- Series EE and I savings bond interest exclusion (Form 8815; IRC 135) ----
// MAGI above which the exclusion phases out, single, head of household and qualifying surviving
// spouse, 2026, whole dollars. Source: Rev. Proc. 2025-32 section 4.17 ("begins to phase out for
// modified adjusted gross income above $152,650 for joint returns and $101,800 for all other
// returns"); agrees with the 2026 draft Form 8815 line 10.
pub(crate) const SAVINGS_BOND_PHASEOUT_START_OTHER_DOLLARS: i64 = 101_800;
// Same, joint return, 2026, whole dollars. Source: Rev. Proc. 2025-32 section 4.17.
pub(crate) const SAVINGS_BOND_PHASEOUT_START_JOINT_DOLLARS: i64 = 152_650;
// Width of the phaseout, single, head of household and qualifying surviving spouse, whole dollars.
// Statutory (IRC 135(b)(2)(A): $15,000), not indexed; the exclusion ends at $116,800 in 2026.
pub(crate) const SAVINGS_BOND_PHASEOUT_RANGE_OTHER_DOLLARS: i64 = 15_000;
// Width of the phaseout, joint return, whole dollars. Statutory (IRC 135(b)(2)(A): $30,000), not
// indexed; the exclusion ends at $182,650 in 2026.
pub(crate) const SAVINGS_BOND_PHASEOUT_RANGE_JOINT_DOLLARS: i64 = 30_000;
// Age the bond owner must have attained before the bond was issued. Statutory (IRC 135(c)(1)(B)).
pub(crate) const SAVINGS_BOND_OWNER_MINIMUM_AGE: u32 = 24;
// Bonds qualify only when issued after this date. Statutory (IRC 135(c)(1)(A): after 1989).
pub(crate) const SAVINGS_BOND_FIRST_ISSUE_DATE: u32 = 19_900_101;

// ---- Education credits (Form 8863) ----
// Expenses the American opportunity credit takes into account per student: $2,000 at 100% plus
// the next $2,000 at 25% (Form 8863 line 27), whole dollars. Statutory (IRC 25A(i)), not indexed.
pub(crate) const AMERICAN_OPPORTUNITY_EXPENSE_CAP_DOLLARS: i64 = 4_000;
// Expenses the lifetime learning credit takes into account per return (Form 8863 line 10), whole
// dollars. Statutory (IRC 25A(c)(1)), not indexed.
pub(crate) const LIFETIME_LEARNING_EXPENSE_CAP_DOLLARS: i64 = 10_000;

// ---- Small employer health insurance credit (Form 8941; IRC 45R) ----
// The section 45R(d)(3)(B) dollar amount, 2026, whole dollars. It starts the average-wage phaseout
// (Form 8941 line 9, Worksheet 6) and, doubled, is the average-wage ceiling of an eligible small
// employer (section 45R(d)(1)(B)). Source: Rev. Proc. 2025-32 section 4.09 ("For taxable years
// beginning in 2026, the dollar amount in effect under section 45R(d)(3)(B) is $34,100"); the 2026
// draft Form 8941 prints it as $34,000 or less (line 9) and $69,000 or more (line 3) because line 3 is
// rounded down to a multiple of $1,000.
pub(crate) const SMALL_EMPLOYER_HEALTH_WAGE_AMOUNT_DOLLARS: i64 = 34_100;
// Average annual wages are rounded down to a multiple of this amount, whole dollars. Statutory
// (IRC 45R(d)(3)); Form 8941 line 3 and Worksheet 3.
pub(crate) const SMALL_EMPLOYER_HEALTH_WAGE_ROUNDING_DOLLARS: i64 = 1_000;
// Hours of service in one full-time equivalent employee and the per-employee hours cap. Statutory
// (IRC 45R(d)(2)); Worksheet 2.
pub(crate) const SMALL_EMPLOYER_HEALTH_FTE_HOURS: u32 = 2_080;
// Full-time equivalent employees above which the credit phases out. Statutory (IRC 45R(c)(1)(A));
// Worksheet 5.
pub(crate) const SMALL_EMPLOYER_HEALTH_FTE_PHASEOUT_START: i64 = 10;
// Width of the full-time equivalent phaseout: the credit ends at 25 employees. Statutory (IRC
// 45R(c)(1)(A)); Worksheet 5.
pub(crate) const SMALL_EMPLOYER_HEALTH_FTE_PHASEOUT_RANGE: i64 = 15;
// An eligible small employer has fewer than this many full-time equivalent employees. Statutory (IRC
// 45R(d)(1)(A)); Form 8941 line 2 ("25 or more": enter 0 on line 12).
pub(crate) const SMALL_EMPLOYER_HEALTH_FTE_LIMIT: i64 = 25;
// Credit percentage of a taxable employer's premiums, whole percent. Statutory (IRC 45R(a), (b));
// the 35% rate of a tax-exempt employer (section 45R(f)) is never this App's filer.
pub(crate) const SMALL_EMPLOYER_HEALTH_CREDIT_PERCENT: i64 = 50;

// ---- Like-kind exchanges (Form 8824; IRC 1031(a)(3)) ----
// Days after the transfer of the property given up within which the new property must be
// identified in writing, or received. Statutory (IRC 1031(a)(3)(A)), not indexed.
pub(crate) const EXCHANGE_IDENTIFICATION_DAYS: i32 = 45;
// Days after the transfer within which the new property must be received, if the return due date
// does not come first. Statutory (IRC 1031(a)(3)(B)), not indexed.
pub(crate) const EXCHANGE_RECEIPT_DAYS: i32 = 180;
// Due date of a 2026 return without an extension (the exchange period ends no later than this
// date), yyyymmdd. Source: IRC 6072(a), April 15, 2027 (a Thursday).
pub(crate) const EXCHANGE_RETURN_DUE_DATE: u32 = 20_270_415;

// ---- Form 4972 lump-sum distributions (TRA 1986 section 1122(h)(5), former section 402(d)) ----
// Frozen 1986 law, not indexed. Source: the 2026 draft Form 4972 (created 6/9/26), line 13-16
// minimum distribution allowance and the page 4 Tax Rate Schedule.
// Minimum distribution allowance ceiling (line 13: 50% of line 12, not more than this), whole
// dollars.
pub(crate) const LUMP_SUM_MINIMUM_ALLOWANCE_CAP_DOLLARS: i64 = 10_000;
// Line 14 subtracts this from line 12 (the allowance falls 20 cents per dollar above it), whole
// dollars. The allowance is zero from the adjusted total taxable amount of $70,000. Statutory
// (frozen TRA 1986 section 1122(h)(5)); Source: 2026 draft Form 4972 line 14.
pub(crate) const LUMP_SUM_MINIMUM_ALLOWANCE_PHASEOUT_START_DOLLARS: i64 = 20_000;
// The 1986 Tax Rate Schedule for lines 24 and 27, as printed, in cents: (over, base tax, rate in
// percent of the amount over). The form taxes one tenth of the amount; the App applies the
// schedule to ten times that tenth in closed form (lump_sum.rs ten_times_tax). Source: 2026 draft
// Form 4972 page 4 Tax Rate Schedule; statutory (frozen 1986 law).
pub(crate) const LUMP_SUM_RATE_SCHEDULE_CENTS: [(i64, i64, i64); 15] = [
    (0, 0, 11),
    (119_000, 13_090, 12),
    (227_000, 26_050, 14),
    (453_000, 57_690, 15),
    (669_000, 90_090, 16),
    (917_000, 129_770, 18),
    (1_144_000, 170_630, 20),
    (1_371_000, 216_030, 23),
    (1_716_000, 295_380, 26),
    (2_288_000, 444_100, 30),
    (2_860_000, 615_700, 34),
    (3_432_000, 810_180, 38),
    (4_230_000, 1_113_420, 42),
    (5_719_000, 1_738_800, 48),
    (8_579_000, 3_111_600, 50),
];

// ---- 529 plans and Coverdell education savings accounts (Form 1099-Q; IRC 529, 530) ----
// Cash a 529 plan may pay for a beneficiary's K-12 tuition and the other qualified elementary
// or secondary expenses in one taxable year, in aggregate, cents. Source: IRC 529(e)(3)(A) last
// sentence as amended by P.L. 119-21 section 70413(b) ($20,000, taxable years beginning after
// December 31, 2025; $10,000 for 2025 and earlier; Pub. 970 (2025) ch. 7 still prints $10,000).
// Not indexed.
pub(crate) const QTP_K12_ANNUAL_LIMIT_CENTS: i64 = 2_000_000;
// Principal and interest of a beneficiary's (or a sibling's) qualified education loans a 529 plan
// may pay over the individual's lifetime, cents. Source: IRC 529(c)(9)(B), Pub. 970 (2025) ch. 7
// ("Qualified education loan repayments"). Statutory, not indexed.
pub(crate) const QTP_LOAN_LIFETIME_LIMIT_CENTS: i64 = 1_000_000;
// Lifetime ceiling on a beneficiary's 529-to-Roth IRA transfers, cents. Source: IRC 529(c)(3)(E)
// (SECURE 2.0 section 126); the annual Roth IRA contribution limit also applies (Notice 2025-67).
// Statutory.
pub(crate) const QTP_ROTH_LIFETIME_LIMIT_CENTS: i64 = 3_500_000;
// Years a 529 account must have been maintained for the same beneficiary before a transfer to a
// Roth IRA. Source: IRC 529(c)(3)(E). Statutory.
pub(crate) const QTP_ROTH_ACCOUNT_YEARS: u32 = 15;
// The additional tax on the includible earnings of a distribution, percent. Source: IRC
// 530(d)(4)(A), applied to 529 plans by IRC 529(c)(6); Form 5329 Part II line 8. Statutory.
pub(crate) const EDUCATION_ACCOUNT_ADDITIONAL_TAX_PERCENT: i64 = 10;

// ---- Health, long-term care, ABLE and Trump accounts (Forms 8853, 5329; IRC 7702B, 529A, 530A) ----
// The per diem limitation for long-term care contract payments, per day of the LTC period, cents.
// Source: IRC 7702B(d)(4) as indexed by Rev. Proc. 2025-32 section 4.62 ($430 for 2026); Form 8853
// Section C line 21 prints "$430 by the number of days" (2026 draft).
pub(crate) const LTC_PER_DIEM_LIMIT_CENTS: i64 = 43_000;
// The annual limit on contributions to a designated beneficiary's ABLE accounts, whole dollars.
// Source: IRC 529A(b)(2)(B)(i) (the section 2503(b) annual exclusion) as indexed by Rev. Proc.
// 2025-32 section 4.34 ($20,000 for 2026); Form 5329 Part VIII (2026 draft).
pub(crate) const ABLE_ANNUAL_CONTRIBUTION_LIMIT_DOLLARS: i64 = 20_000;
// The annual limit on contributions to a Trump account for a child, cents. Source: IRC 530A(c)(1)
// ($5,000 for 2026, indexed after 2027; P.L. 119-21 section 70204; Notice 2025-68); Form 5329 Parts
// X and XI (2026 draft). Pilot, qualified general and rollover contributions do not count.
pub(crate) const TRUMP_ACCOUNT_ANNUAL_LIMIT_CENTS: i64 = 500_000;

// ---- Employer credit for paid family and medical leave (Form 8994; IRC 45S) ----
// The most 2025 compensation a qualifying employee may have received from the employer, 2026, whole
// dollars: 60% of the 2025 section 414(q)(1)(B)(i) amount of $160,000. Source: IRC 45S(d)(2) as
// amended by P.L. 119-21 section 70304 (taxable years beginning after December 31, 2025); the
// $160,000 amount is in Notice 2024-80; the i8994 compensation table (Rev. December 2024) prints the
// same $96,000 for 2026. The attribute range on PaidLeaveEmployee.prior_year_compensation names the cents constant
// below as `$PAID_LEAVE_COMPENSATION_LIMIT_2026_CENTS`.
pub(crate) const PAID_LEAVE_COMPENSATION_LIMIT_2026_DOLLARS: i64 = 96_000;
// The same limit in cents, the unit of PaidLeaveEmployee.prior_year_compensation. Source: as above.
pub(crate) const PAID_LEAVE_COMPENSATION_LIMIT_2026_CENTS: i64 =
    PAID_LEAVE_COMPENSATION_LIMIT_2026_DOLLARS * 100;
// Hours a week a qualifying employee must customarily work. Source: IRC 45S(d)(1)(B) as amended by
// P.L. 119-21 section 70304; Notice 2026-28. The attribute range on PaidLeaveEmployee.weekly_hours
// names it as `$PAID_LEAVE_MIN_WEEKLY_HOURS`.
pub(crate) const PAID_LEAVE_MIN_WEEKLY_HOURS: u32 = 20;
// Weeks of leave in a taxable year that earn the credit. Source: IRC 45S(b)(3); Form 8994 worksheet.
// The validation rule on PaidLeaveEmployee names it as `$PAID_LEAVE_MAX_WEEKS * 100` hundredths of the weekly hours.
pub(crate) const PAID_LEAVE_MAX_WEEKS: i64 = 12;
// The lowest rate of payment a written policy may have (50%). Source: IRC 45S(c)(1)(B). The
// attribute range on the rate fields names it as `$PAID_LEAVE_MIN_RATE_PERCENT`.
pub(crate) const PAID_LEAVE_MIN_RATE_PERCENT: i64 = 50;
// The same lowest rate in hundredths of a percent, for the credit worksheet. Source: IRC 45S(c)(1)(B).
pub(crate) const PAID_LEAVE_MIN_RATE_HUNDREDTHS: i64 = PAID_LEAVE_MIN_RATE_PERCENT * 100;
// The applicable percentage at the 50% rate, in hundredths of a percent (12.5%). Source: IRC
// 45S(a)(2)(A); the i8994 Applicable Percentage Worksheet.
pub(crate) const PAID_LEAVE_BASE_PERCENT_HUNDREDTHS: i64 = 1_250;
// The applicable percentage grows 0.25 percentage point for each point of the rate above 50%, so four
// hundredths of the rate's excess make one hundredth of the percentage. Source: IRC 45S(a)(2)(B).
pub(crate) const PAID_LEAVE_RATE_POINTS_PER_PERCENT_POINT: i64 = 4;
// The highest applicable percentage, in hundredths of a percent (25%). Source: IRC 45S(a)(2).
pub(crate) const PAID_LEAVE_MAX_PERCENT_HUNDREDTHS: i64 = 2_500;

// ---- Adoption credit and assistance exclusion (Form 8839; IRC 23, 137) ----
// Maximum adoption credit (and special-needs credit), 2026, whole dollars. Source: Rev. Proc.
// 2025-32 section 4.04(1)-(2) ("$17,670"); section 4.18 gives the same amount for IRC 137.
pub(crate) const ADOPTION_CREDIT_MAXIMUM_DOLLARS: i64 = 17_670;
// Same amount in cents. Source: Rev. Proc. 2025-32 section 4.04. The attribute range on the
// adoption credit and exclusion fields names it as `$ADOPTION_CREDIT_MAXIMUM_CENTS`.
pub(crate) const ADOPTION_CREDIT_MAXIMUM_CENTS: i64 = ADOPTION_CREDIT_MAXIMUM_DOLLARS * 100;
// MAGI above which the adoption credit and exclusion phase out, 2026, whole dollars. Source:
// Rev. Proc. 2025-32 section 4.04(2) and section 4.18 ("in excess of $265,080").
pub(crate) const ADOPTION_PHASEOUT_START_DOLLARS: i64 = 265_080;
// Width of the adoption phaseout, whole dollars. Source: Rev. Proc. 2025-32 section 4.04(2)
// (completely phased out at $305,080 or more, that is $265,080 + $40,000).
pub(crate) const ADOPTION_PHASEOUT_RANGE_DOLLARS: i64 = 40_000;
// Refundable portion of the adoption credit, 2026, whole dollars. Source: Rev. Proc. 2025-32
// section 4.04(3) ("$5,120").
pub(crate) const ADOPTION_REFUNDABLE_DOLLARS: i64 = 5_120;

// ---- State and local tax deduction cap (Schedule A line 5e; IRC 164(b)(6)-(7)) ----
// Cap on the state and local tax deduction, 2026, whole dollars. Source: IRC 164(b)(6) as
// amended by P.L. 119-21 section 70120 ($40,000 for 2025, increased 1% for 2026); not listed in
// Rev. Proc. 2025-32.
pub(crate) const SALT_CAP_DOLLARS: i64 = 40_400;
// MAGI above which the cap is reduced by 30% of the excess, 2026, whole dollars. Source: IRC
// 164(b)(7) as amended by P.L. 119-21 section 70120 ($500,000 for 2025, increased 1% for 2026).
pub(crate) const SALT_PHASEDOWN_THRESHOLD_DOLLARS: i64 = 505_000;
// Same threshold, married filing separately (half), whole dollars. Source: IRC 164(b)(7).
pub(crate) const SALT_PHASEDOWN_THRESHOLD_SEPARATE_DOLLARS: i64 = 252_500;
// The reduced cap never falls below this amount, whole dollars. Statutory (IRC 164(b)(7)),
// not indexed.
pub(crate) const SALT_REDUCED_CAP_FLOOR_DOLLARS: i64 = 10_000;

// ---- Mortgage insurance premiums (Schedule A line 8d; IRC 163(h)(3)(E)-(F)) ----
// AGI above which qualified mortgage insurance premiums phase out, 2026, whole dollars. Source:
// IRC 163(h)(3)(E)(ii) (statutory, not indexed), reinstated for 2026 by P.L. 119-21 section 70108;
// Schedule A 2026 draft (created 5/12/26) line 8d; Pub. 936 (2026) draft (modified 2026-10-07),
// "What's New" and "Mortgage Insurance Premiums" worksheet.
pub(crate) const MORTGAGE_INSURANCE_PHASEOUT_START_DOLLARS: i64 = 100_000;
// Same threshold, married filing separately, whole dollars. Source: IRC 163(h)(3)(E)(ii); Pub. 936
// (2026) draft.
pub(crate) const MORTGAGE_INSURANCE_PHASEOUT_START_SEPARATE_DOLLARS: i64 = 50_000;
// Each $1,000 of AGI over the threshold, or part of it, cuts the premiums by 10%, whole dollars.
// Source: IRC 163(h)(3)(E)(ii); Pub. 936 (2026) draft.
pub(crate) const MORTGAGE_INSURANCE_PHASEOUT_STEP_DOLLARS: i64 = 1_000;
// Same step, married filing separately, whole dollars. Source: IRC 163(h)(3)(E)(ii) ($500).
pub(crate) const MORTGAGE_INSURANCE_PHASEOUT_STEP_SEPARATE_DOLLARS: i64 = 500;
// Ten steps remove the whole deduction (a 10% cut per step). Source: IRC 163(h)(3)(E)(ii).
pub(crate) const MORTGAGE_INSURANCE_PHASEOUT_STEPS: i64 = 10;

// ---- Student loan interest deduction (Schedule 1 line 21; IRC 221) ----
// Maximum deduction. Statutory (IRC 221(b)(1)), not indexed; Rev. Proc. 2025-32 section 4.29.
pub(crate) const STUDENT_LOAN_INTEREST_MAXIMUM_DOLLARS: i64 = 2_500;
// MAGI above which the deduction phases out, 2026, whole dollars, other returns. Source: Rev.
// Proc. 2025-32 section 4.29 ("in excess of $85,000 ($175,000 for joint returns)").
pub(crate) const STUDENT_LOAN_PHASEOUT_START_OTHER_DOLLARS: i64 = 85_000;
// Same, joint return. Source: Rev. Proc. 2025-32 section 4.29.
pub(crate) const STUDENT_LOAN_PHASEOUT_START_JOINT_DOLLARS: i64 = 175_000;
// Width of the phaseout, other returns, whole dollars. Source: Rev. Proc. 2025-32 section 4.29
// (completely phased out at $100,000 or more).
pub(crate) const STUDENT_LOAN_PHASEOUT_RANGE_OTHER_DOLLARS: i64 = 15_000;
// Width of the phaseout, joint return, whole dollars. Source: Rev. Proc. 2025-32 section 4.29
// (completely phased out at $205,000 or more).
pub(crate) const STUDENT_LOAN_PHASEOUT_RANGE_JOINT_DOLLARS: i64 = 30_000;

// ---- Excess business loss (Form 461; IRC 461(l)) ----
// Threshold, 2026, whole dollars, other returns. Source: Rev. Proc. 2025-32 section 4.31
// ("$256,000 ($512,000 for joint returns)").
pub(crate) const EXCESS_BUSINESS_LOSS_THRESHOLD_OTHER_DOLLARS: i64 = 256_000;
// Threshold, 2026, whole dollars, joint return. Source: Rev. Proc. 2025-32 section 4.31.
pub(crate) const EXCESS_BUSINESS_LOSS_THRESHOLD_JOINT_DOLLARS: i64 = 512_000;

// ---- Foreign earned income exclusion (Form 2555; IRC 911) ----
// Maximum exclusion for a full qualifying year, 2026, whole dollars. Source: Rev. Proc. 2025-32
// section 4.39 ("$132,900").
pub(crate) const FOREIGN_EARNED_INCOME_EXCLUSION_DOLLARS: i64 = 132_900;
// 2025 service-year limit. Source: Rev. Proc. 2024-40 section 2.39
// (https://www.irs.gov/pub/irs-drop/rp-24-40.pdf), $130,000.
pub(crate) const FOREIGN_EARNED_INCOME_EXCLUSION_2025_DOLLARS: i64 = 130_000;
// Base housing amount for a full qualifying year, whole dollars: 16% of the exclusion limit
// (Source: IRC 911(c)(1)(B); Form 2555 line 32, $21,264).
pub(crate) const FOREIGN_BASE_HOUSING_DOLLARS: i64 =
    crate::kernel::ratio::trunc_i64(FOREIGN_EARNED_INCOME_EXCLUSION_DOLLARS * 16, 100);
// Same, per qualifying day, cents, rounded to the cent ($58.26). Source: Form 2555 line 32.
pub(crate) const FOREIGN_BASE_HOUSING_DAILY_CENTS: i64 = daily_cents(FOREIGN_BASE_HOUSING_DOLLARS);
// Housing expense limit for a location with no higher listed limit, a full year, whole dollars:
// 30% of the exclusion limit (Source: IRC 911(c)(2)(A)(i); Notice 2026-25 general limit, $39,870).
pub(crate) const FOREIGN_HOUSING_LIMIT_DOLLARS: i64 =
    crate::kernel::ratio::trunc_i64(FOREIGN_EARNED_INCOME_EXCLUSION_DOLLARS * 30, 100);
// Same, per qualifying day, cents ($109.23). Source: Notice 2026-25 general limit, i2555 line 29b.
pub(crate) const FOREIGN_HOUSING_LIMIT_DAILY_CENTS: i64 =
    daily_cents(FOREIGN_HOUSING_LIMIT_DOLLARS);
// An annual amount over 365 days, whole cents rounded half up.
const fn daily_cents(annual_dollars: i64) -> i64 {
    crate::kernel::ratio::half_offset_i64(annual_dollars * 200, 730)
}

// ---- Foreign tax credit (Form 1116; IRC 901, 904) ----
// Form 1116 adjustment exception: the foreign net capital gain plus the foreign qualified dividends
// must be below this amount, whole dollars. Source: 2026 draft i1116, "Adjustment exception" 2.
pub(crate) const FOREIGN_RATE_ADJUSTMENT_EXCEPTION_DOLLARS: i64 = 20_000;
// Gross foreign income at or below which all interest expense is allocated to U.S. income by
// choice, whole dollars. Source: 2026 draft i1116, lines 4a and 4b ("doesn't exceed $5,000"), for
// a U.S. citizen, resident alien or domestic estate.
pub(crate) const FOREIGN_SMALL_INCOME_INTEREST_DOLLARS: i64 = 5_000;
// The election without Form 1116 (section 904(j)): $300, $600 on a joint return, whole dollars.
// Source: IRC 904(j)(2)(B); 2026 draft i1116, "Election To Claim the Foreign Tax Credit Without
// Filing Form 1116".
pub(crate) const FOREIGN_DE_MINIMIS_DOLLARS: i64 = 300;
// The same on a joint return, whole dollars. Source: IRC 904(j)(2)(B).
pub(crate) const FOREIGN_DE_MINIMIS_JOINT_DOLLARS: i64 = 600;
// Section 901(j) countries for 2026: Iran, North Korea, Sudan and Syria, by IRS country code (Libya
// is waived). Source: Pub. 514 (2025), Table 1; recheck against the 2026 revision.
pub(crate) const SECTION_901J_COUNTRIES_2026: [&str; 4] = ["IR", "KN", "SU", "SY"];
pub(crate) fn is_section_901j_country(code: &str) -> bool {
    SECTION_901J_COUNTRIES_2026.contains(&code)
}

// ---- Education credits (Form 8863; IRC 25A(d)) ----
// MAGI at which the education credits are completely phased out, whole dollars, other returns.
// Source: Form 8863 lines 3 and 6 ($90,000); not listed in Rev. Proc. 2025-32; recheck against
// the final 2026 Form 8863 instructions at the step 4 recheck.
pub(crate) const EDUCATION_CREDIT_PHASEOUT_END_OTHER_DOLLARS: i64 = 90_000;
// Same, joint return. Source: Form 8863 lines 3 and 6 ($180,000); same recheck note.
pub(crate) const EDUCATION_CREDIT_PHASEOUT_END_JOINT_DOLLARS: i64 = 180_000;
// Width of the education credit phaseout, whole dollars, other returns. Source: Form 8863 line 5
// ($10,000); IRC 25A(d)(2); same recheck note.
pub(crate) const EDUCATION_CREDIT_PHASEOUT_RANGE_OTHER_DOLLARS: i64 = 10_000;
// Same, joint return. Source: Form 8863 line 5 ($20,000); IRC 25A(d)(2); same recheck note.
pub(crate) const EDUCATION_CREDIT_PHASEOUT_RANGE_JOINT_DOLLARS: i64 = 20_000;

// ---- Medical savings account high deductible plan (Form 8853; IRC 220(c)(2)) ----
// Self-only coverage: lowest annual deductible, 2026, cents. Source: Rev. Proc. 2025-32 section
// 4.28(1) ("not less than $2,900 and not more than $4,400 ... $5,850").
pub(crate) const ARCHER_SELF_ONLY_DEDUCTIBLE_MINIMUM_CENTS: i64 = 290_000;
// Self-only coverage: highest annual deductible, 2026, cents. Source: Rev. Proc. 2025-32
// section 4.28(1).
pub(crate) const ARCHER_SELF_ONLY_DEDUCTIBLE_MAXIMUM_CENTS: i64 = 440_000;
// Self-only coverage: out-of-pocket maximum, 2026, cents. Source: Rev. Proc. 2025-32 section
// 4.28(1).
pub(crate) const ARCHER_SELF_ONLY_OUT_OF_POCKET_MAXIMUM_CENTS: i64 = 585_000;
// Family coverage: lowest annual deductible, 2026, cents. Source: Rev. Proc. 2025-32 section
// 4.28(2) ("not less than $5,850 and not more than $8,750 ... $10,700").
pub(crate) const ARCHER_FAMILY_DEDUCTIBLE_MINIMUM_CENTS: i64 = 585_000;
// Family coverage: highest annual deductible, 2026, cents. Source: Rev. Proc. 2025-32 section
// 4.28(2).
pub(crate) const ARCHER_FAMILY_DEDUCTIBLE_MAXIMUM_CENTS: i64 = 875_000;
// Family coverage: out-of-pocket maximum, 2026, cents. Source: Rev. Proc. 2025-32 section 4.28(2).
pub(crate) const ARCHER_FAMILY_OUT_OF_POCKET_MAXIMUM_CENTS: i64 = 1_070_000;

// ---- Capital loss limit (Schedule D line 21; IRC 1211(b)) ----
// Net capital loss deductible against ordinary income, whole dollars. Statutory (IRC 1211(b)(1)),
// not indexed.
pub(crate) const CAPITAL_LOSS_LIMIT_DOLLARS: i64 = 3_000;
// Same, married filing separately, whole dollars. Statutory (IRC 1211(b)(2)).
pub(crate) const CAPITAL_LOSS_LIMIT_SEPARATE_DOLLARS: i64 = 1_500;

// ---- Small business stock (IRC 1244 and 1202; Pub. 550; Instructions for Schedule D) ----
// Section 1244 ordinary-loss limit per year, in cents: $50,000, or $100,000 on a joint return "even
// if only one spouse has this type of loss". Statutory (IRC 1244(b)), not indexed. Source: Pub. 550
// "Losses on Section 1244 (Small Business) Stock"; Instructions for Form 4797 Part II.
pub(crate) const SECTION_1244_ORDINARY_LOSS_LIMIT_CENTS: i64 = 5_000_000;
// Same, joint return, in cents. Statutory (IRC 1244(b)). Source: Pub. 550, same section.
pub(crate) const SECTION_1244_ORDINARY_LOSS_LIMIT_JOINT_CENTS: i64 = 10_000_000;
// Section 1202 per-issuer eligible-gain limit, in cents: $10,000,000 less gain taken into account in
// earlier years for the issuer. Statutory (IRC 1202(b)(1)(A)), not indexed for stock issued on or
// before July 4, 2025, the only stock a 2026 sale can reach (the $15,000,000 cap of IRC 1202(b)(4)
// applies to later stock and is indexed after 2026). Source: P.L. 119-21 section 70431;
// Instructions for Schedule D "Qualified small business stock".
pub(crate) const SECTION_1202_ISSUER_LIMIT_CENTS: i64 = 1_000_000_000;
// Same, married filing separately, in cents. Statutory (IRC 1202(b)(3)(A)). Source: Instructions
// for Schedule D, Form 8949 code Q.
pub(crate) const SECTION_1202_ISSUER_LIMIT_SEPARATE_CENTS: i64 = 500_000_000;
// Section 1202 alternative limit: ten times the adjusted basis of the issuer's stock sold in the
// year. Statutory (IRC 1202(b)(1)(B)). Source: Instructions for Schedule D "Qualified small
// business stock".
pub(crate) const SECTION_1202_BASIS_MULTIPLE: i64 = 10;
// First original-issue date of qualified small business stock (stock issued after August 10, 1993),
// as yyyymmdd. Statutory (IRC 1202(c)(1)). Source: Instructions for Schedule D "Qualified small
// business stock".
pub(crate) const SECTION_1202_FIRST_ISSUE_DATE: u32 = 19930811;
// Last original-issue date of the original section 1202 tiers (50/75/100 percent after a five-year
// hold), as yyyymmdd. Source: P.L. 119-21 section 70431 applies the three-, four- and five-year
// tiers of IRC 1202(a)(5), the $15,000,000 cap and the $75,000,000 asset test only to stock issued
// after July 4, 2025. A 2026 sale of that later stock cannot meet its three-year minimum (the
// earliest qualifying sale is July 5, 2028), so the original tiers are the only ones a 2026 sale
// uses.
pub(crate) const SECTION_1202_LAST_ORIGINAL_TIER_ISSUE_DATE: u32 = 20250704;
// Last issue date of the 50 percent exclusion tier, as yyyymmdd. Statutory (IRC 1202(a)(1), (a)(3),
// (a)(4)): stock acquired on or before February 17, 2009 is 50 percent. Source: Instructions for
// Schedule D, Form 8949 code Q.
pub(crate) const SECTION_1202_FIFTY_PERCENT_LAST_DATE: u32 = 20090217;
// Last issue date of the 75 percent tier, as yyyymmdd. Statutory (IRC 1202(a)(3), (a)(4)): stock
// acquired after February 17, 2009 and before September 28, 2010 is 75 percent; later stock is 100
// percent. Source: Instructions for Schedule D, Form 8949 code Q.
pub(crate) const SECTION_1202_SEVENTY_FIVE_PERCENT_LAST_DATE: u32 = 20100927;
// First issue date of the 60 percent empowerment-zone increase, as yyyymmdd. Statutory (IRC
// 1202(a)(2)): stock acquired after December 21, 2000 and on or before February 17, 2009 (the end
// of the 50 percent tier above). Source: Instructions for Schedule D "Qualified small business
// stock".
pub(crate) const SECTION_1202_ZONE_FIRST_DATE: u32 = 20001222;
// Years of holding that must be exceeded, as the yyyymmdd increment (five years). Statutory (IRC
// 1202(a)(1) for the original tiers). Source: Instructions for Schedule D "Qualified small business
// stock".
pub(crate) const SECTION_1202_HOLDING_YEARS_AS_DATE: u32 = 50000;
// Percentage of the excluded gain that is an AMT preference for stock acquired before September 28,
// 2010 (the 50 and 75 percent tiers). Statutory (IRC 57(a)(7)). Source: Form 6251 line 2h (2026
// draft); stock acquired later has no preference.
pub(crate) const SECTION_1202_AMT_PREFERENCE_PERCENT: i64 = 7;
// Earliest date of a qualified opportunity fund investment, as yyyymmdd: the enactment of IRC 1400Z-2
// (P.L. 115-97 section 13823, December 22, 2017). Source: Instructions for Form 8997 (2025 form).
pub(crate) const QOF_FIRST_INVESTMENT_DATE: u32 = 20171222;
// The date the deferral of a gain invested in a qualified opportunity fund before 2027 ends and the
// remaining deferred gain is included, as yyyymmdd. Statutory (IRC 1400Z-2(b)(1)(B)); P.L. 119-21's
// amendments apply only to amounts invested after December 31, 2026. Source: Instructions for Form
// 8949 "How To Report Gain Previously Deferred in a QOF Investment"; Treas. Reg. 1.1400Z2(b)-1(c).
pub(crate) const QOF_DEFERRAL_END_DATE: u32 = 20261231;
// Last investment date of a pre-2026 record, as yyyymmdd: an investment made in 2026 defers nothing
// past December 31, 2026, so its gain is taxed on its sale entry and it is not entered as a deferral.
// Source: IRC 1400Z-2(b)(1)(B); Form 8997 (2026 draft) Part I.
pub(crate) const QOF_LAST_EARLIER_INVESTMENT_DATE: u32 = 20251231;
// Last investment date that has been held five years on the December 31, 2026 inclusion, as
// yyyymmdd. Statutory (IRC 1400Z-2(b)(2)(B)(iii)). Source: Form 8997 Instructions (2025) Part III,
// special gain code and basis adjustment.
pub(crate) const QOF_FIVE_YEAR_LAST_INVESTMENT_DATE: u32 = 20211231;
// Same, seven years. Statutory (IRC 1400Z-2(b)(2)(B)(iv)). Source: Form 8997 Instructions (2025)
// Part III.
pub(crate) const QOF_SEVEN_YEAR_LAST_INVESTMENT_DATE: u32 = 20191231;
// Percent of the deferred gain added to basis when the investment was held at least five years.
// Statutory (IRC 1400Z-2(b)(2)(B)(iii)). Source: Treas. Reg. 1.1400Z2(b)-1(g)(2).
pub(crate) const QOF_FIVE_YEAR_PERCENT: i64 = 10;
// Further percent added to basis when held at least seven years. Statutory (IRC
// 1400Z-2(b)(2)(B)(iv)). Source: Treas. Reg. 1.1400Z2(b)-1(g)(2).
pub(crate) const QOF_SEVEN_YEAR_PERCENT: i64 = 5;
// First investment date under the amended IRC 1400Z-2(a) of P.L. 119-21, as yyyymmdd: amounts
// invested after December 31, 2026 may defer a 2026 gain. Source: P.L. 119-21 (its amendments of IRC 1400Z-2(a)(2), (b) and (c), applicable to amounts invested after December 31, 2026);
// Instructions for Form 8949 "How To Report an Election To Defer Tax on Eligible Gain Invested in a
// QOF".
pub(crate) const QOF_AMENDED_FIRST_INVESTMENT_DATE: u32 = 20270101;
// Last investment date that can defer a gain from a sale on December 31, 2026, as yyyymmdd: 180 days
// beginning December 31, 2026. Statutory (IRC 1400Z-2(a)(1)(A)). Source: Instructions for Form 8949
// "How To Report an Election To Defer Tax on Eligible Gain Invested in a QOF".
pub(crate) const QOF_DEFERRAL_LAST_INVESTMENT_DATE: u32 = 20270628;
// A worthless security is treated as sold on the last day of the tax year, as yyyymmdd; it is long
// term when held more than a year to that date. Statutory (IRC 165(g)(1)). Source: Pub. 550
// "Worthless Securities".
pub(crate) const WORTHLESS_SECURITY_DEEMED_SALE_DATE: u32 = 20261231;

// ---- Taxable Social Security benefits (Pub. 915; IRC 86) ----
// Base amount, joint return, whole dollars. Statutory (IRC 86(c)(1)(B)), not indexed.
pub(crate) const SOCIAL_SECURITY_BASE_JOINT_DOLLARS: i64 = 32_000;
// Base amount, all other returns, whole dollars. Statutory (IRC 86(c)(1)(A)), not indexed.
pub(crate) const SOCIAL_SECURITY_BASE_OTHER_DOLLARS: i64 = 25_000;
// Width of the 50% band, joint return, whole dollars (adjusted base $44,000). Statutory (IRC
// 86(c)(2)(B)), not indexed.
pub(crate) const SOCIAL_SECURITY_BAND_JOINT_DOLLARS: i64 = 12_000;
// Width of the 50% band, all other returns, whole dollars (adjusted base $34,000). Statutory
// (IRC 86(c)(2)(A)), not indexed.
pub(crate) const SOCIAL_SECURITY_BAND_OTHER_DOLLARS: i64 = 9_000;

// ---- Olympic and Paralympic medals and USOC prize money (section 74(d); Schedule 1 lines 8m, 24c) ----
// Adjusted gross income above which the Olympic and Paralympic medal and USOC prize money exclusion
// is lost, in cents ($1,000,000): the AGI determined without the exclusion must not exceed it.
// Statutory (IRC 74(d)(2)(A)), not indexed. Source: Instructions for Form 1040 line 8m; Schedule 1
// (2026 draft) line 24c.
pub(crate) const OLYMPIC_EXCLUSION_AGI_LIMIT_CENTS: i64 = 100_000_000;
// Same, married filing separately, in cents ($500,000). Statutory (IRC 74(d)(2)(A)(ii)). Source:
// Instructions for Form 1040 line 8m.
pub(crate) const OLYMPIC_EXCLUSION_AGI_LIMIT_SEPARATE_CENTS: i64 = 50_000_000;

// ---- Qualified disaster distributions (Form 8915-F; SECURE 2.0 section 331) ----
// Most one disaster allows across all plans and years, cents ($22,000). Source: Form 8915-F
// instructions line 1a (disasters beginning 2021 and later); IRC 72(t)(2)(M) as added by SECURE
// 2.0 section 331. Statutory, not indexed. The attribute ranges on the disaster amount fields and
// the DISASTER_ABOVE_LIMIT rule name it as `$DISASTER_DISTRIBUTION_LIMIT_CENTS`.
pub(crate) const DISASTER_DISTRIBUTION_LIMIT_CENTS: i64 = 2_200_000;

// ---- Early-distribution exception caps indexed for 2026 (Form 5329 exceptions 23 and 24) ----
// Most an eligible distribution to a domestic-abuse survivor may exclude from the additional tax,
// cents ($10,500; the lesser of 50% of the account or this indexed $10,000). Source: Notice
// 2025-67 (IRC 72(t)(2)(K)(i), (ii)); 2026 draft Instructions for Form 5329, exception 23. The
// DomesticAbuse attribute ranges name it as `$DOMESTIC_ABUSE_DISTRIBUTION_LIMIT_CENTS`.
pub(crate) const DOMESTIC_ABUSE_DISTRIBUTION_LIMIT_CENTS: i64 = 1_050_000;
// Most a certified long-term-care insurance premium distribution may exclude from the additional
// tax per year, cents ($2,600; the lesser of 10% of the vested balance or this indexed $2,500).
// Source: Notice 2025-67 (IRC 72(t)(2)(G)(ii)); 2026 draft Instructions for Form 5329, exception 24.
pub(crate) const LONG_TERM_CARE_DISTRIBUTION_LIMIT_CENTS: i64 = 260_000;
// Most an IRA owner's qualified first-time homebuyer distributions may total in a lifetime, cents
// ($10,000; statutory, not indexed). Source: section 72(t)(8)(B); 2026 draft Instructions for Form
// 8606 line 20 and Instructions for Form 5329, exception 09.
pub(crate) const FIRST_HOME_DISTRIBUTION_LIMIT_CENTS: i64 = 1_000_000;

// ---- Qualified charitable distributions (section 408(d)(8); Notice 2025-67) ----
// Most one person excludes as qualified charitable distributions in 2026, cents ($111,000). Source:
// section 408(d)(8)(A) as indexed by Notice 2025-67.
pub(crate) const QCD_LIMIT_CENTS: i64 = 11_100_000;
// Most the one-time split-interest election of section 408(d)(8)(F) allows in 2026, cents
// ($55,000), counted within the annual limit above. Source: section 408(d)(8)(F)(i)(II) as indexed
// by Notice 2025-67; Pub. 590-B. The QCD gift's split-interest rule names it as
// `$SPLIT_INTEREST_QCD_LIMIT_CENTS`.
pub(crate) const SPLIT_INTEREST_QCD_LIMIT_CENTS: i64 = 5_500_000;

// ---- Form 4972 death benefit exclusion (former section 101(b); Form 4972 line 9) ----
// Most the former death benefit exclusion allows for all recipients together, cents ($5,000), for a
// plan participant who died before August 21, 1996. Statutory, not indexed (former IRC 101(b)(1)
// as in effect before the Small Business Job Protection Act of 1996 section 1402). Source: Form
// 4972 (2026 draft) line 9 and the Death Benefit Worksheet. The Form 4972 page fields name it as
// `$DEATH_BENEFIT_EXCLUSION_CENTS`.
pub(crate) const DEATH_BENEFIT_EXCLUSION_CENTS: i64 = 500_000;

// ---- Work opportunity credit first-year wage caps (Form 5884; IRC 51(b)(3), 51(e)) ----
// Qualified first-year wages per employee, cents, standard group ($6,000). Source: IRC 51(b)(3).
// Statutory, not indexed. The WOTC_PRIOR_YEAR_WAGES_ABOVE_CAP rules name the caps as `$WOTC_FIRST_YEAR_CAP_*_CENTS`.
pub(crate) const WOTC_FIRST_YEAR_CAP_STANDARD_CENTS: i64 = 600_000;
// Disabled veteran, cents ($12,000). Source: IRC 51(b)(3); Form 5884
// instructions line 1a. Statutory, not indexed.
pub(crate) const WOTC_FIRST_YEAR_CAP_DISABLED_VETERAN_CENTS: i64 = 1_200_000;
// Long-term unemployed veteran, cents ($14,000). Source: IRC 51(b)(3); see
// Form 5884 instructions. Statutory, not indexed.
pub(crate) const WOTC_FIRST_YEAR_CAP_LONG_UNEMPLOYED_VETERAN_CENTS: i64 = 1_400_000;
// Disabled and long-term unemployed veteran, cents ($24,000). Source: IRC 51(b)(3) (the two
// veteran caps combined); Form 5884 instructions. Statutory, not indexed.
pub(crate) const WOTC_FIRST_YEAR_CAP_DISABLED_LONG_UNEMPLOYED_VETERAN_CENTS: i64 = 2_400_000;
// Long-term family assistance recipient, cents ($10,000 per year). Source: IRC 51(e). Statutory,
// not indexed.
pub(crate) const WOTC_FIRST_YEAR_CAP_FAMILY_ASSISTANCE_CENTS: i64 = 1_000_000;

// ---- Dependent care assistance programs (Form 2441 Part III; IRC 129) ----
// Most dependent care benefits an employer plan may exclude (joint or single filer), 2026, whole
// dollars. Source: IRC 129(a)(2)(A) as amended by P.L. 119-21 section 70404 (taxable years
// beginning after December 31, 2025). Statutory, not indexed. Form 2441 line 21 caps the
// documented combined plan maximum; plan maxima and W-2 box 10 are not statutory bounds.
pub(crate) const DEPENDENT_CARE_BENEFIT_LIMIT_DOLLARS: i64 = 7_500;
// Same, married filing separately (half), 2026, whole dollars. Source: IRC 129(a)(2)(A) as
// amended by P.L. 119-21 section 70404. Statutory.
pub(crate) const DEPENDENT_CARE_BENEFIT_LIMIT_SEPARATE_DOLLARS: i64 = 3_750;

// ---- Investment credit (Forms 3468 and 4255; IRC 46, 47, 48, 48E, 50) ----
// Rehabilitation credit determined for a certified historic building, percent of the qualified
// rehabilitation expenditures. Source: IRC 47(a)(2); Reg. 1.47-7(c). Statutory, not indexed.
pub(crate) const REHABILITATION_CREDIT_PERCENT: i64 = 20;
// Ratable share allowed in each year of the five-year period, percent of the expenditures (a fifth
// of 20%). Source: IRC 47(a)(1); Reg. 1.47-7(b); Form 3468 (2026 draft) Part VII line 1j
// "multiply line 1g by 4%". Statutory, not indexed.
pub(crate) const REHABILITATION_RATABLE_PERCENT: i64 = 4;
// Years in the rehabilitation credit period, beginning with the year the building is placed in
// service. Source: IRC 47(a)(1); Form 3468 (2026 draft) Part VII note to line 1j. Statutory.
pub(crate) const REHABILITATION_PERIOD_YEARS: u32 = 5;
// First placed-in-service year of a rehabilitation credit the App calculates: the credit is the
// 20% credit allowed ratably over five years, for expenditures after 2017. Source: P.L. 115-97
// section 13402 (IRC 47 as amended); Form 3468 (2025) What's New (the transition rule removed).
pub(crate) const REHABILITATION_FIRST_PLACED_YEAR: u32 = 2018;
// Floor of the substantial-rehabilitation test, cents: expenditures during the measuring period
// must exceed the greater of this and the building's adjusted basis. Source: IRC
// 47(c)(1)(B)(i)(II). Statutory, not indexed. The REHABILITATION_NOT_SUBSTANTIAL rule names it as
// `$REHABILITATION_SUBSTANTIAL_FLOOR_CENTS`.
pub(crate) const REHABILITATION_SUBSTANTIAL_FLOOR_CENTS: i64 = 500_000;
// Months in the substantial-rehabilitation measuring period (24, or 60 for phased rehabilitation
// under written plans). Source: IRC 47(c)(1)(B)(i)-(ii). Statutory. The REHABILITATION_PERIOD_NOT_IN_
// SERVICE_YEAR rules name both counts as `$REHABILITATION_PERIOD_MONTHS` and
// `$REHABILITATION_PHASED_PERIOD_MONTHS`.
pub(crate) const REHABILITATION_PERIOD_MONTHS: u32 = 24;
// Same, phased rehabilitation. Source: IRC 47(c)(1)(B)(ii). Statutory.
pub(crate) const REHABILITATION_PHASED_PERIOD_MONTHS: u32 = 60;
// Percent of the investment credit recaptured by whole years from the first day of the month the
// property was placed in service to the day it ceased to be investment credit property (index 0
// is under one full year; five or more is zero). Source: IRC 50(a)(1)(B); Form 4255 (Rev.
// December 2026 draft) instructions line 15 table. Statutory, not indexed.
pub(crate) const INVESTMENT_RECAPTURE_PERCENT: [i64; 6] = [100, 80, 60, 40, 20, 0];
// Percent of the rehabilitation credit that reduces the basis of the building (the whole credit
// determined). Source: IRC 50(c)(1); Reg. 1.47-7(d), (e)(2). Statutory.
pub(crate) const REHABILITATION_BASIS_REDUCTION_PERCENT: i64 = 100;
// Percent of an energy or clean electricity investment credit that reduces the basis of the
// property. Source: IRC 50(c)(3)(A). Statutory.
pub(crate) const ENERGY_BASIS_REDUCTION_PERCENT: i64 = 50;
// Base energy credit rate, percent of the credit base (the 6% rate that becomes 30% when the
// prevailing wage and apprenticeship requirements are met or an exception applies). Source: IRC
// 48(a)(2)(A)(i)(II), (a)(7)(A) and 48E(a)(2)(A)(ii)(II); Form 3468 (2026 draft) Part V line 1b,
// Part VI lines 3b, 15b. Statutory.
pub(crate) const ENERGY_BASE_RATE_PERCENT: i64 = 6;
// Increased energy credit rate, percent (five times the base rate). Source: IRC 48(a)(9)(B),
// 48E(a)(2)(A)(ii)(I); Form 3468 (2026 draft) Part V line 1b. Statutory.
pub(crate) const ENERGY_INCREASED_RATE_PERCENT: i64 = 30;
// Domestic content and energy community bonus, percent of the credit base, with the increased
// rate. Source: IRC 48(a)(12), (14) and 48E(a)(3); Form 3468 (2026 draft) Part V lines 1d, 1f, 3d,
// 3f and Part VI lines 3k, 3m. Statutory.
pub(crate) const ENERGY_BONUS_INCREASED_PERCENT: i64 = 10;
// Same bonus with the base rate, percent. Source: IRC 48(a)(12), (14) and 48E(a)(3); Form 3468
// (2026 draft) Part I lines 9b and 10b. Statutory.
pub(crate) const ENERGY_BONUS_BASE_PERCENT: i64 = 2;
// First day of the year after the section 48 construction window: the energy credit (Part VI)
// applies to solar, small wind and storage property whose construction began before this date;
// later construction is the clean electricity investment credit (Part V). Source: IRC
// 48(a)(2)(A)(i)(II) and 48E(b)(3)(A) as amended by P.L. 117-169 section 13702 and P.L. 119-21
// section 70513; Form 3468 (2026 draft) instructions What's New. Statutory.
pub(crate) const ENERGY_CONSTRUCTION_CUTOFF_DATE: u32 = 20250101;
// Geothermal heat pump property qualifies under section 48 when construction began before this
// date. Source: IRC 48(a)(3)(A)(vii) and (a)(7); Form 3468 (2026 draft) Part VI Section H.
// Statutory.
pub(crate) const GEOTHERMAL_HEAT_PUMP_CUTOFF_DATE: u32 = 20350101;

// Source: IRS 2026 Schedule A; tax-year end for field bounds and month-end calculations.
pub(crate) const TAX_YEAR: u32 = 2026;
// Source: IRC 441, end of the same calendar tax year.
pub const TAX_YEAR_END_DATE: u32 = TAX_YEAR * 10000 + 1231;
// Source: IRC 441, first day of the same calendar tax year.
pub const TAX_YEAR_START_DATE: u32 = TAX_YEAR * 10000 + 101;
// Source: Rev. Proc. 2025-32 section 4.30, section 448(c) gross-receipt test.
pub(crate) const SMALL_BUSINESS_GROSS_RECEIPTS_LIMIT_CENTS: i64 = 3_200_000_000;
// Source: IRS 2026 Schedule A; December 31 of the owning tax year.
// Source: IRS 2026 draft Form 8801; its prior-return worksheets use 2025.
pub(crate) const PRIOR_TAX_YEAR: u32 = TAX_YEAR - 1;

// Form6198 current computation and immediately preceding detailed return.
pub const RISK_TAX_YEAR: u32 = TAX_YEAR;
// The extractor resolves this literal; retain the common prior-year footing.
pub const RISK_PRIOR_YEAR: u32 = 2025;
const _: () = assert!(RISK_PRIOR_YEAR == PRIOR_TAX_YEAR);
// Source: IRS 2026 contribution year begins January 1; shared annual owner.
// Source: IRC 530(b)(4); 2026-designated deposits by April 15, 2027.
pub(crate) const COVERDELL_CONTRIBUTION_LAST_DAY: u32 = 20270415;
// Source: IRC 530(d)(4)(C); 2026 contributions and net income returned before June 1, 2027.
pub(crate) const COVERDELL_CORRECTION_LAST_DAY: u32 = 20270531;
// Source: IRC 530(b)(1)(A)(iii), (c); annual Coverdell contribution limit.
pub(crate) const COVERDELL_LIMIT_DOLLARS: i64 = 2000;
// Source: IRC 530(c)(1)(A); individual contributor MAGI phaseout start.
pub(crate) const COVERDELL_SINGLE_START_DOLLARS: i64 = 95000;
// Source: IRC 530(c)(1)(B); individual contributor phaseout span.
pub(crate) const COVERDELL_SINGLE_SPAN_DOLLARS: i64 = 15000;
// Source: IRC 530(c)(1)(A)(ii); joint contributor MAGI phaseout start.
pub(crate) const COVERDELL_JOINT_START_DOLLARS: i64 = 190000;
// Source: IRC 530(c)(1)(B); joint contributor phaseout span.
pub(crate) const COVERDELL_JOINT_SPAN_DOLLARS: i64 = 30000;
// Source: IRC 4973(a); annual tax on excess Coverdell contributions.
pub(crate) const COVERDELL_EXCESS_RATE_PERCENT: i64 = 6;
// Source: Pub.936 grandfathered debt last origination date; IRC163(h)(3)(D).
pub(crate) const MORTGAGE_GRANDFATHERED_LAST_DATE: u32 = 19871013;
// Source: IRC163(h)(3)(B),(D); acquisition debt begins the next day.
pub(crate) const MORTGAGE_ACQUISITION_FIRST_DATE: u32 = 19871014;
// Source: IRC163(h)(3)(F); last day for the earlier acquisition-debt limit.
pub(crate) const MORTGAGE_EARLIER_LIMIT_LAST_DATE: u32 = 20171215;
// ---- Statutory and indexed amounts moved by the second constants pass (improvements log row 209) ----
// Home-sale exclusion, one owner, cents ($250,000). Source: IRC 121(b)(1). Statutory, not indexed.
pub(crate) const HOME_SALE_EXCLUSION_SINGLE_CENTS: i64 = 25_000_000;
// Home-sale exclusion, joint return, cents ($500,000). Source: IRC 121(b)(2). Statutory.
pub(crate) const HOME_SALE_EXCLUSION_JOINT_CENTS: i64 = 50_000_000;
// Acquisition-debt limit for a home mortgage incurred before December 16, 2017, cents
// ($1,000,000). Source: IRC 163(h)(3)(B); Pub. 936 Table 1 line 3. Statutory.
pub(crate) const MORTGAGE_DEBT_LIMIT_GRANDFATHERED_CENTS: i64 = 100_000_000;
// Same, married filing separately, cents ($500,000). Source: IRC 163(h)(3)(B)(ii).
pub(crate) const MORTGAGE_DEBT_LIMIT_GRANDFATHERED_SEPARATE_CENTS: i64 = 50_000_000;
// Acquisition-debt limit for a mortgage incurred after December 15, 2017, cents ($750,000).
// Source: IRC 163(h)(3)(F), as made permanent by P.L. 119-21. Statutory.
pub(crate) const MORTGAGE_DEBT_LIMIT_CENTS: i64 = 75_000_000;
// Same, married filing separately, cents ($375,000). Source: IRC 163(h)(3)(F)(i)(II).
pub(crate) const MORTGAGE_DEBT_LIMIT_SEPARATE_CENTS: i64 = 37_500_000;
// Form 2441 qualifying expenses for one qualifying person, whole dollars. Source: IRC 21(c);
// Form 2441 line 3. Statutory, not indexed.
pub(crate) const CARE_EXPENSE_LIMIT_ONE_PERSON_DOLLARS: i64 = 3_000;
// Same, two or more qualifying persons, whole dollars. Source: IRC 21(c).
pub(crate) const CARE_EXPENSE_LIMIT_TWO_PERSONS_DOLLARS: i64 = 6_000;
// Most dependent care benefits a 2025 employer plan could exclude, cents ($5,000). Source: IRC
// 129(a)(2)(A) before P.L. 119-21 section 70404; the 2025 Form 2441 history of prior-year care.
pub(crate) const DEPENDENT_CARE_BENEFIT_LIMIT_2025_CENTS: i64 = 500_000;
// Senior deduction per eligible person, cents ($6,000). Source: IRC 151(d)(5)(A) as added by
// P.L. 119-21 section 70103. Statutory; 2025 through 2028.
pub(crate) const SENIOR_DEDUCTION_PER_PERSON_CENTS: i64 = 600_000;
// SEP annual additions limit, cents ($72,000, 2026). Source: IRC 415(c)(1)(A), as indexed by
// Notice 2025-67 for 2026.
pub(crate) const SEP_ANNUAL_ADDITIONS_LIMIT_CENTS: i64 = 7_200_000;
// SEP compensation limit, cents ($360,000, 2026). Source: IRC 401(a)(17), as indexed by Notice
// 2025-67 for 2026.
pub(crate) const SEP_COMPENSATION_LIMIT_CENTS: i64 = 36_000_000;
// Optional self-employment methods, 2026, whole dollars. Source: Schedule SE (2026 draft) Part II
// lines 4b, 15 and 17: the two-thirds methods' maximum ($7,560), the farm gross income ceiling
// ($11,340) and the net profit ceiling ($8,186) below which the nonfarm method is allowed.
pub(crate) const OPTIONAL_SE_MAXIMUM_DOLLARS: i64 = 7_560;
// Farm optional method gross income ceiling, 2026, whole dollars. Source: Schedule SE (2026 draft) line 15.
pub(crate) const OPTIONAL_SE_FARM_GROSS_LIMIT_DOLLARS: i64 = 11_340;
// Net profit ceiling of the optional methods, 2026, whole dollars. Source: Schedule SE (2026 draft) lines 15 and 17.
pub(crate) const OPTIONAL_SE_NONFARM_PROFIT_LIMIT_DOLLARS: i64 = 8_186;
// 2025 Form 1040 basic standard deduction, whole dollars (Form 8978 prior-year taxable income).
// Source: Rev. Proc. 2024-40 section 3.15(1).
pub(crate) const STANDARD_DEDUCTION_2025_SINGLE_OR_SEPARATE_DOLLARS: i64 = 15_750;
// Same, joint and surviving spouse, 2025. Source: Rev. Proc. 2024-40 section 3.15(1).
pub(crate) const STANDARD_DEDUCTION_2025_JOINT_DOLLARS: i64 = 31_500;
// Same, head of household, 2025. Source: Rev. Proc. 2024-40 section 3.15(1).
pub(crate) const STANDARD_DEDUCTION_2025_HEAD_DOLLARS: i64 = 23_625;

// ---- Special allowance for rental real estate with active participation (Form 8582 Part II; IRC 469(i)) ----
// Most the special allowance offsets in a year, whole dollars ($25,000). Source: IRC 469(i)(2);
// Form 8582 (2025) instructions line 9. Statutory, not indexed. Also the most of the rehabilitation
// credit's passive allowance (Form 8582-CR Part III, IRC 469(i)(3)(A)).
pub(crate) const PASSIVE_ALLOWANCE_DOLLARS: i64 = 25_000;
// Same, married filing separately and living apart all year (half), whole dollars ($12,500).
// Source: IRC 469(i)(5)(B); Form 8582 (2025) instructions line 9. Statutory.
pub(crate) const PASSIVE_ALLOWANCE_SEPARATE_DOLLARS: i64 = 12_500;
// Modified adjusted gross income at which the allowance is gone, whole dollars ($150,000): half of
// the income over $100,000 reduces it. Source: IRC 469(i)(3)(A); Form 8582 (2025) Part II line 5
// ("$150,000"). Statutory, not indexed.
pub(crate) const PASSIVE_ALLOWANCE_PHASEOUT_END_DOLLARS: i64 = 150_000;
// Same, married filing separately and living apart all year, whole dollars ($75,000). Source:
// IRC 469(i)(5)(B); Form 8582 (2025) Part II line 5 ("$75,000"). Statutory.
pub(crate) const PASSIVE_ALLOWANCE_PHASEOUT_END_SEPARATE_DOLLARS: i64 = 75_000;
// Source: IRC441, calendar tax year; existing estimated-payment App input
// window begins on January1 of2026. This constant does not set installment due dates.
pub(crate) const ESTIMATED_PAYMENT_START_DATE: u32 = 20260101;
// Source: IRC6072(a), April15 following the calendar taxable year; existing
// estimated-payment App input window. Underpayment algorithms keep their own dates.
pub(crate) const ESTIMATED_PAYMENT_LAST_DATE: u32 = 20270415;

// Source: IRC 4999(a); 2026 W-2 instructions code K and Schedule 2.
// Form 1099-NEC box 3 reports the determined excess payment, not this tax.
pub(crate) const GOLDEN_PARACHUTE_EXCISE_PERCENT: i64 = 20;
// Calendar year of the2026 prototype return. Source:2026 draft Form8962 title.
pub const PTC_TAX_YEAR: u32 = 2026;
// Calendar year of the2026 health-account forms. Source:2026 drafts Forms8889/8853.
pub const HEALTH_TAX_YEAR: u32 = PTC_TAX_YEAR;
// Earlier lifetime QHFD uses are separate from the existing prior-year testing-period owner.
// Source: Notice 2008-51, One-time qualified HSA funding distribution, pp. 4–5.
pub const HEALTH_EARLIER_FUNDING_END: u32 = 20241231;

// 2026 final W-2 instructions, section 128(b)(1); Pub. 926 (2026).
pub const TRUMP_EMPLOYER_EXCLUSION_CENTS: i64 = 250_000;
pub const HOUSEHOLD_CASH_WAGE_THRESHOLD_CENTS: i64 = 300_000;
// Source: IRS 2025 final i2441, Worksheet A, $6000 expense ceiling.
pub(crate) const CARE_EXPENSE_LIMIT_TWO_PERSONS_2025_CENTS: i64 = 600_000;

// Retained 2025 Form 2441 Worksheet A / credit table (2025 final i2441, Worksheet A).
// Source: IRS 2025 final i2441, Worksheet A and credit percentage table.
pub(crate) const CARE_EXPENSE_LIMIT_ONE_PERSON_2025_DOLLARS: i64 = 3_000;
// Source: IRS 2025 final i2441, Worksheet A and credit percentage table.
pub(crate) const CARE_EXPENSE_LIMIT_TWO_PERSONS_2025_DOLLARS: i64 = 6_000;
// Source: IRS 2025 final i2441, Worksheet A and credit percentage table.
pub(crate) const CARE_CREDIT_MAXIMUM_2025_PERCENT: i64 = 35;
// Source: IRS 2025 final i2441, Worksheet A and credit percentage table.
pub(crate) const CARE_CREDIT_MINIMUM_2025_PERCENT: i64 = 20;
// Source: IRS 2025 final i2441, Worksheet A and credit percentage table.
pub(crate) const CARE_CREDIT_PHASEOUT_START_2025_DOLLARS: i64 = 15_000;
// Source: IRS 2025 final i2441, Worksheet A and credit percentage table.
pub(crate) const CARE_CREDIT_PHASEOUT_STEP_2025_DOLLARS: i64 = 2_000;

type LeaseInclusionBand = (i64, i64, [i64; 5]);
type LeaseInclusionTable = (u32, &'static [LeaseInclusionBand]);

// Source: IRC 280F(c), Rev. Proc. 2018-25 Table 4 and Rev. Procs. 2019-26,
// 2020-37, 2021-31, 2022-17, 2023-14, 2024-13, 2025-16, 2026-15 Table 3.
// Each row: FMV over/not over in cents, five annual inclusion amounts in dollars.
pub(crate) const LEASE_INCLUSION_TABLES: &[LeaseInclusionTable] = &[
    (
        2018,
        &[
            (5000000, 5100000, [1, 3, 5, 5, 6]),
            (5100000, 5200000, [4, 9, 13, 16, 19]),
            (5200000, 5300000, [7, 15, 22, 27, 31]),
            (5300000, 5400000, [10, 21, 31, 37, 44]),
            (5400000, 5500000, [12, 27, 40, 48, 56]),
            (5500000, 5600000, [15, 33, 49, 59, 68]),
            (5600000, 5700000, [18, 39, 58, 69, 81]),
            (5700000, 5800000, [20, 45, 67, 80, 93]),
            (5800000, 5900000, [23, 51, 76, 91, 105]),
            (5900000, 6000000, [26, 57, 85, 101, 117]),
            (6000000, 6200000, [30, 66, 98, 118, 135]),
            (6200000, 6400000, [36, 78, 116, 139, 160]),
            (6400000, 6600000, [41, 90, 134, 160, 185]),
            (6600000, 6800000, [46, 102, 152, 181, 210]),
            (6800000, 7000000, [52, 114, 169, 203, 235]),
            (7000000, 7200000, [57, 126, 187, 225, 259]),
            (7200000, 7400000, [63, 138, 205, 246, 284]),
            (7400000, 7600000, [68, 150, 223, 267, 309]),
            (7600000, 7800000, [74, 162, 241, 288, 333]),
            (7800000, 8000000, [79, 174, 259, 310, 357]),
            (8000000, 8500000, [89, 195, 290, 347, 401]),
            (8500000, 9000000, [102, 225, 335, 400, 463]),
            (9000000, 9500000, [116, 255, 379, 454, 525]),
            (9500000, 10000000, [130, 285, 423, 508, 586]),
            (10000000, 11000000, [150, 330, 491, 587, 679]),
            (11000000, 12000000, [178, 390, 579, 695, 802]),
            (12000000, 13000000, [205, 450, 669, 801, 926]),
            (13000000, 14000000, [232, 510, 758, 908, 1049]),
            (14000000, 15000000, [260, 570, 847, 1015, 1172]),
            (15000000, 16000000, [287, 630, 936, 1122, 1296]),
            (16000000, 17000000, [314, 691, 1024, 1230, 1419]),
            (17000000, 18000000, [342, 750, 1114, 1336, 1543]),
            (18000000, 19000000, [369, 810, 1204, 1442, 1666]),
            (19000000, 20000000, [396, 871, 1292, 1550, 1789]),
            (20000000, 21000000, [424, 930, 1382, 1656, 1913]),
            (21000000, 22000000, [451, 991, 1470, 1764, 2036]),
            (22000000, 23000000, [478, 1051, 1559, 1871, 2159]),
            (23000000, 24000000, [505, 1111, 1649, 1977, 2283]),
            (24000000, 100000000000, [533, 1171, 1738, 2084, 2406]),
        ],
    ),
    (
        2019,
        &[
            (5000000, 5100000, [0, 1, 1, 3, 3]),
            (5100000, 5200000, [4, 11, 15, 20, 23]),
            (5200000, 5300000, [9, 20, 30, 36, 43]),
            (5300000, 5400000, [13, 30, 44, 53, 63]),
            (5400000, 5500000, [17, 40, 58, 70, 83]),
            (5500000, 5600000, [22, 49, 72, 88, 102]),
            (5600000, 5700000, [26, 59, 86, 105, 122]),
            (5700000, 5800000, [31, 68, 101, 122, 142]),
            (5800000, 5900000, [35, 78, 115, 139, 161]),
            (5900000, 6000000, [39, 88, 129, 156, 181]),
            (6000000, 6200000, [46, 102, 151, 181, 211]),
            (6200000, 6400000, [55, 121, 179, 216, 250]),
            (6400000, 6600000, [63, 140, 208, 251, 289]),
            (6600000, 6800000, [72, 160, 236, 284, 329]),
            (6800000, 7000000, [81, 179, 265, 318, 369]),
            (7000000, 7200000, [90, 198, 293, 353, 408]),
            (7200000, 7400000, [98, 217, 322, 387, 448]),
            (7400000, 7600000, [107, 236, 351, 421, 487]),
            (7600000, 7800000, [116, 255, 379, 456, 526]),
            (7800000, 8000000, [125, 275, 407, 489, 567]),
            (8000000, 8500000, [140, 308, 458, 549, 635]),
            (8500000, 9000000, [162, 356, 529, 635, 734]),
            (9000000, 9500000, [184, 404, 600, 720, 833]),
            (9500000, 10000000, [206, 452, 671, 806, 931]),
            (10000000, 11000000, [238, 525, 778, 934, 1079]),
            (11000000, 12000000, [282, 621, 920, 1105, 1277]),
            (12000000, 13000000, [326, 717, 1063, 1276, 1474]),
            (13000000, 14000000, [370, 812, 1206, 1447, 1672]),
            (14000000, 15000000, [413, 909, 1348, 1618, 1869]),
            (15000000, 16000000, [457, 1005, 1491, 1788, 2067]),
            (16000000, 17000000, [501, 1101, 1633, 1960, 2264]),
            (17000000, 18000000, [545, 1197, 1776, 2130, 2461]),
            (18000000, 19000000, [588, 1293, 1919, 2301, 2659]),
            (19000000, 20000000, [632, 1389, 2061, 2473, 2856]),
            (20000000, 21000000, [676, 1485, 2204, 2643, 3053]),
            (21000000, 22000000, [720, 1581, 2346, 2815, 3250]),
            (22000000, 23000000, [763, 1677, 2489, 2986, 3448]),
            (23000000, 24000000, [807, 1773, 2632, 3156, 3645]),
            (24000000, 100000000000, [851, 1869, 2774, 3328, 3842]),
        ],
    ),
    (
        2020,
        &[
            (5000000, 5100000, [0, 1, 0, 2, 2]),
            (5100000, 5200000, [2, 6, 9, 10, 13]),
            (5200000, 5300000, [5, 11, 17, 20, 24]),
            (5300000, 5400000, [7, 17, 24, 30, 35]),
            (5400000, 5500000, [10, 22, 32, 39, 46]),
            (5500000, 5600000, [12, 27, 41, 48, 57]),
            (5600000, 5700000, [15, 32, 49, 58, 68]),
            (5700000, 5800000, [17, 38, 56, 68, 79]),
            (5800000, 5900000, [19, 44, 64, 77, 90]),
            (5900000, 6000000, [22, 49, 72, 87, 100]),
            (6000000, 6200000, [26, 56, 84, 102, 117]),
            (6200000, 6400000, [30, 68, 99, 121, 139]),
            (6400000, 6600000, [35, 78, 116, 139, 161]),
            (6600000, 6800000, [40, 89, 131, 159, 183]),
            (6800000, 7000000, [45, 99, 148, 177, 205]),
            (7000000, 7200000, [50, 110, 163, 197, 227]),
            (7200000, 7400000, [55, 121, 179, 215, 249]),
            (7400000, 7600000, [60, 131, 195, 235, 271]),
            (7600000, 7800000, [64, 142, 211, 254, 293]),
            (7800000, 8000000, [69, 153, 227, 272, 315]),
            (8000000, 8500000, [78, 172, 254, 306, 353]),
            (8500000, 9000000, [90, 198, 295, 353, 408]),
            (9000000, 9500000, [102, 225, 334, 401, 463]),
            (9500000, 10000000, [114, 252, 373, 449, 518]),
            (10000000, 11000000, [133, 292, 433, 520, 600]),
            (11000000, 12000000, [157, 345, 513, 615, 710]),
            (12000000, 13000000, [181, 399, 592, 710, 820]),
            (13000000, 14000000, [206, 452, 671, 805, 931]),
            (14000000, 15000000, [230, 506, 750, 901, 1040]),
            (15000000, 16000000, [254, 559, 830, 996, 1150]),
            (16000000, 17000000, [279, 612, 909, 1091, 1260]),
            (17000000, 18000000, [303, 666, 988, 1186, 1370]),
            (18000000, 19000000, [327, 720, 1067, 1281, 1480]),
            (19000000, 20000000, [351, 773, 1147, 1377, 1589]),
            (20000000, 21000000, [376, 826, 1227, 1471, 1700]),
            (21000000, 22000000, [400, 880, 1306, 1566, 1810]),
            (22000000, 23000000, [424, 934, 1385, 1661, 1920]),
            (23000000, 24000000, [449, 987, 1464, 1757, 2029]),
            (24000000, 100000000000, [473, 1040, 1544, 1852, 2139]),
        ],
    ),
    (
        2021,
        &[
            (5100000, 5200000, [0, 0, 1, 0, 1]),
            (5200000, 5300000, [1, 1, 1, 2, 2]),
            (5300000, 5400000, [1, 2, 2, 3, 4]),
            (5400000, 5500000, [1, 3, 3, 5, 5]),
            (5500000, 5600000, [2, 3, 5, 6, 6]),
            (5600000, 5700000, [2, 4, 6, 7, 8]),
            (5700000, 5800000, [2, 5, 7, 8, 10]),
            (5800000, 5900000, [3, 5, 8, 10, 11]),
            (5900000, 6000000, [3, 6, 9, 11, 13]),
            (6000000, 6200000, [3, 7, 11, 13, 15]),
            (6200000, 6400000, [4, 9, 13, 15, 18]),
            (6400000, 6600000, [5, 10, 15, 18, 21]),
            (6600000, 6800000, [5, 12, 17, 21, 24]),
            (6800000, 7000000, [6, 13, 20, 23, 27]),
            (7000000, 7200000, [7, 14, 22, 26, 30]),
            (7200000, 7400000, [7, 16, 24, 29, 33]),
            (7400000, 7600000, [8, 18, 26, 31, 36]),
            (7600000, 7800000, [9, 19, 28, 34, 39]),
            (7800000, 8000000, [9, 21, 30, 37, 42]),
            (8000000, 8500000, [11, 23, 34, 41, 48]),
            (8500000, 9000000, [12, 27, 40, 47, 55]),
            (9000000, 9500000, [14, 30, 45, 55, 62]),
            (9500000, 10000000, [16, 34, 50, 61, 70]),
            (10000000, 11000000, [18, 40, 58, 71, 81]),
            (11000000, 12000000, [21, 47, 70, 83, 97]),
            (12000000, 13000000, [25, 54, 81, 96, 112]),
            (13000000, 14000000, [28, 62, 91, 110, 127]),
            (14000000, 15000000, [31, 69, 103, 123, 141]),
            (15000000, 16000000, [35, 76, 114, 136, 157]),
            (16000000, 17000000, [38, 84, 124, 149, 172]),
            (17000000, 18000000, [41, 91, 135, 163, 187]),
            (18000000, 19000000, [45, 98, 146, 176, 202]),
            (19000000, 20000000, [48, 106, 157, 188, 218]),
            (20000000, 21000000, [51, 113, 168, 202, 232]),
            (21000000, 22000000, [55, 120, 179, 215, 247]),
            (22000000, 23000000, [58, 128, 190, 227, 263]),
            (23000000, 24000000, [61, 135, 201, 241, 278]),
            (24000000, 100000000000, [65, 142, 212, 254, 293]),
        ],
    ),
    (
        2022,
        &[
            (5600000, 5700000, [1, 1, 1, 2, 2]),
            (5700000, 5800000, [2, 4, 5, 7, 7]),
            (5800000, 5900000, [3, 7, 9, 11, 13]),
            (5900000, 6000000, [4, 9, 14, 16, 19]),
            (6000000, 6200000, [6, 13, 20, 23, 28]),
            (6200000, 6400000, [9, 19, 27, 34, 38]),
            (6400000, 6600000, [11, 24, 36, 43, 50]),
            (6600000, 6800000, [14, 30, 43, 53, 61]),
            (6800000, 7000000, [16, 35, 52, 63, 72]),
            (7000000, 7200000, [19, 40, 61, 72, 83]),
            (7200000, 7400000, [21, 46, 68, 82, 95]),
            (7400000, 7600000, [24, 51, 77, 91, 106]),
            (7600000, 7800000, [26, 57, 85, 101, 117]),
            (7800000, 8000000, [29, 62, 93, 111, 128]),
            (8000000, 8500000, [33, 72, 107, 128, 148]),
            (8500000, 9000000, [39, 86, 127, 152, 176]),
            (9000000, 9500000, [45, 100, 147, 177, 204]),
            (9500000, 10000000, [52, 113, 167, 201, 233]),
            (10000000, 11000000, [61, 133, 198, 238, 274]),
            (11000000, 12000000, [73, 161, 239, 286, 330]),
            (12000000, 13000000, [86, 188, 279, 335, 387]),
            (13000000, 14000000, [98, 216, 319, 384, 443]),
            (14000000, 15000000, [111, 243, 360, 432, 499]),
            (15000000, 16000000, [123, 270, 401, 481, 555]),
            (16000000, 17000000, [135, 298, 441, 529, 612]),
            (17000000, 18000000, [148, 325, 482, 578, 667]),
            (18000000, 19000000, [160, 352, 523, 626, 724]),
            (19000000, 20000000, [173, 379, 563, 676, 779]),
            (20000000, 21000000, [185, 407, 603, 724, 836]),
            (21000000, 22000000, [198, 434, 644, 773, 892]),
            (22000000, 23000000, [210, 461, 685, 821, 949]),
            (23000000, 24000000, [222, 489, 725, 870, 1005]),
            (24000000, 100000000000, [235, 516, 766, 918, 1061]),
        ],
    ),
    (
        2023,
        &[
            (6000000, 6200000, [0, 0, 1, 3, 5]),
            (6200000, 6400000, [13, 29, 43, 54, 63]),
            (6400000, 6600000, [26, 57, 86, 104, 122]),
            (6600000, 6800000, [39, 86, 128, 154, 181]),
            (6800000, 7000000, [52, 114, 170, 206, 239]),
            (7000000, 7200000, [65, 143, 212, 256, 297]),
            (7200000, 7400000, [78, 171, 255, 306, 356]),
            (7400000, 7600000, [91, 200, 296, 358, 414]),
            (7600000, 7800000, [104, 228, 339, 408, 473]),
            (7800000, 8000000, [117, 257, 381, 459, 531]),
            (8000000, 8500000, [140, 306, 455, 548, 634]),
            (8500000, 9000000, [172, 378, 560, 674, 780]),
            (9000000, 9500000, [204, 449, 666, 801, 926]),
            (9500000, 10000000, [237, 520, 772, 927, 1073]),
            (10000000, 11000000, [286, 627, 930, 1117, 1292]),
            (11000000, 12000000, [350, 769, 1142, 1371, 1584]),
            (12000000, 13000000, [415, 912, 1353, 1623, 1877]),
            (13000000, 14000000, [480, 1054, 1564, 1877, 2169]),
            (14000000, 15000000, [545, 1196, 1776, 2130, 2462]),
            (15000000, 16000000, [610, 1339, 1986, 2384, 2754]),
            (16000000, 17000000, [675, 1481, 2198, 2637, 3046]),
            (17000000, 18000000, [740, 1623, 2409, 2891, 3338]),
            (18000000, 19000000, [805, 1766, 2620, 3143, 3632]),
            (19000000, 20000000, [870, 1908, 2831, 3397, 3924]),
            (20000000, 21000000, [935, 2050, 3043, 3650, 4216]),
            (21000000, 22000000, [999, 2193, 3254, 3903, 4509]),
            (22000000, 23000000, [1064, 2335, 3465, 4157, 4801]),
            (23000000, 24000000, [1129, 2478, 3676, 4410, 5094]),
            (24000000, 100000000000, [1194, 2620, 3887, 4664, 5386]),
        ],
    ),
    (
        2024,
        &[
            (6200000, 6400000, [7, 16, 24, 28, 32]),
            (6400000, 6600000, [21, 47, 69, 82, 94]),
            (6600000, 6800000, [35, 77, 114, 136, 157]),
            (6800000, 7000000, [49, 107, 159, 191, 219]),
            (7000000, 7200000, [62, 138, 204, 245, 281]),
            (7200000, 7400000, [76, 168, 250, 298, 344]),
            (7400000, 7600000, [90, 199, 294, 353, 406]),
            (7600000, 7800000, [104, 229, 340, 406, 469]),
            (7800000, 8000000, [118, 259, 385, 461, 531]),
            (8000000, 8500000, [142, 313, 463, 556, 640]),
            (8500000, 9000000, [177, 388, 577, 690, 797]),
            (9000000, 9500000, [211, 465, 689, 826, 952]),
            (9500000, 10000000, [246, 541, 802, 961, 1108]),
            (10000000, 11000000, [298, 655, 971, 1163, 1343]),
            (11000000, 12000000, [367, 807, 1196, 1435, 1655]),
            (12000000, 13000000, [437, 958, 1423, 1704, 1968]),
            (13000000, 14000000, [506, 1111, 1647, 1975, 2280]),
            (14000000, 15000000, [575, 1263, 1873, 2245, 2592]),
            (15000000, 16000000, [645, 1414, 2099, 2516, 2904]),
            (16000000, 17000000, [714, 1566, 2325, 2786, 3216]),
            (17000000, 18000000, [783, 1719, 2549, 3057, 3529]),
            (18000000, 19000000, [852, 1871, 2775, 3327, 3841]),
            (19000000, 20000000, [922, 2022, 3001, 3598, 4153]),
            (20000000, 21000000, [991, 2175, 3226, 3868, 4465]),
            (21000000, 22000000, [1060, 2327, 3452, 4138, 4778]),
            (22000000, 23000000, [1130, 2478, 3678, 4409, 5089]),
            (23000000, 24000000, [1199, 2631, 3902, 4680, 5402]),
            (24000000, 100000000000, [1268, 2783, 4128, 4950, 5714]),
        ],
    ),
    (
        2025,
        &[
            (6200000, 6400000, [13, 26, 36, 43, 50]),
            (6400000, 6600000, [25, 53, 78, 92, 107]),
            (6600000, 6800000, [38, 81, 118, 142, 163]),
            (6800000, 7000000, [50, 109, 159, 191, 220]),
            (7000000, 7200000, [63, 136, 201, 240, 277]),
            (7200000, 7400000, [76, 164, 241, 289, 334]),
            (7400000, 7600000, [88, 192, 282, 338, 391]),
            (7600000, 7800000, [101, 219, 324, 387, 447]),
            (7800000, 8000000, [113, 247, 364, 437, 504]),
            (8000000, 8500000, [135, 295, 437, 522, 604]),
            (8500000, 9000000, [167, 364, 539, 646, 745]),
            (9000000, 9500000, [198, 434, 641, 768, 888]),
            (9500000, 10000000, [230, 502, 744, 892, 1029]),
            (10000000, 11000000, [277, 606, 898, 1076, 1242]),
            (11000000, 12000000, [340, 744, 1103, 1322, 1525]),
            (12000000, 13000000, [403, 882, 1308, 1568, 1809]),
            (13000000, 14000000, [466, 1021, 1512, 1814, 2093]),
            (14000000, 15000000, [529, 1159, 1717, 2060, 2377]),
            (15000000, 16000000, [592, 1297, 1923, 2305, 2661]),
            (16000000, 17000000, [655, 1435, 2128, 2551, 2945]),
            (17000000, 18000000, [718, 1573, 2333, 2797, 3229]),
            (18000000, 19000000, [781, 1711, 2538, 3043, 3513]),
            (19000000, 20000000, [844, 1850, 2742, 3289, 3797]),
            (20000000, 21000000, [907, 1988, 2948, 3534, 4081]),
            (21000000, 22000000, [970, 2126, 3153, 3780, 4364]),
            (22000000, 23000000, [1033, 2264, 3358, 4026, 4648]),
            (23000000, 24000000, [1096, 2402, 3563, 4272, 4932]),
            (24000000, 100000000000, [1159, 2540, 3768, 4518, 5216]),
        ],
    ),
    (
        2026,
        &[
            (6200000, 6400000, [8, 15, 21, 25, 27]),
            (6400000, 6600000, [19, 38, 56, 66, 76]),
            (6600000, 6800000, [29, 62, 91, 108, 123]),
            (6800000, 7000000, [40, 86, 125, 149, 172]),
            (7000000, 7200000, [51, 109, 160, 191, 220]),
            (7200000, 7400000, [61, 133, 195, 232, 268]),
            (7400000, 7600000, [72, 156, 230, 274, 316]),
            (7600000, 7800000, [83, 179, 265, 316, 364]),
            (7800000, 8000000, [93, 203, 299, 358, 412]),
            (8000000, 8500000, [112, 244, 360, 431, 496]),
            (8500000, 9000000, [139, 302, 447, 535, 617]),
            (9000000, 9500000, [166, 361, 534, 639, 737]),
            (9500000, 10000000, [192, 420, 621, 743, 858]),
            (10000000, 11000000, [232, 507, 752, 900, 1038]),
            (11000000, 12000000, [286, 624, 926, 1108, 1279]),
            (12000000, 13000000, [339, 742, 1099, 1317, 1520]),
            (13000000, 14000000, [392, 859, 1273, 1526, 1760]),
            (14000000, 15000000, [446, 976, 1447, 1734, 2001]),
            (15000000, 16000000, [499, 1093, 1621, 1943, 2242]),
            (16000000, 17000000, [553, 1210, 1795, 2151, 2483]),
            (17000000, 18000000, [606, 1328, 1968, 2360, 2723]),
            (18000000, 19000000, [659, 1445, 2142, 2569, 2964]),
            (19000000, 20000000, [713, 1562, 2316, 2777, 3205]),
            (20000000, 21000000, [766, 1679, 2490, 2986, 3445]),
            (21000000, 22000000, [820, 1796, 2664, 3194, 3686]),
            (22000000, 23000000, [873, 1913, 2838, 3403, 3927]),
            (23000000, 24000000, [926, 2031, 3012, 3610, 4168]),
            (24000000, 25000000, [980, 2148, 3185, 3820, 4408]),
            (25000000, 26000000, [1033, 2265, 3360, 4027, 4650]),
            (26000000, 27000000, [1086, 2382, 3534, 4236, 4890]),
            (27000000, 28000000, [1140, 2499, 3708, 4444, 5131]),
            (28000000, 29000000, [1193, 2617, 3881, 4653, 5372]),
            (29000000, 30000000, [1247, 2733, 4056, 4862, 5612]),
            (30000000, 31000000, [1300, 2851, 4229, 5070, 5854]),
            (31000000, 32000000, [1353, 2968, 4403, 5279, 6094]),
            (32000000, 33000000, [1407, 3085, 4577, 5487, 6335]),
            (33000000, 34000000, [1460, 3202, 4751, 5696, 6576]),
            (34000000, 35000000, [1514, 3319, 4925, 5904, 6817]),
            (35000000, 36000000, [1567, 3437, 5098, 6113, 7057]),
            (36000000, 37000000, [1620, 3554, 5273, 6321, 7298]),
            (37000000, 38000000, [1674, 3671, 5446, 6530, 7539]),
            (38000000, 39000000, [1727, 3788, 5621, 6738, 7779]),
            (39000000, 40000000, [1781, 3905, 5794, 6947, 8020]),
            (40000000, 41000000, [1834, 4022, 5969, 7155, 8261]),
            (41000000, 42000000, [1887, 4140, 6142, 7364, 8501]),
            (42000000, 43000000, [1941, 4257, 6316, 7572, 8742]),
            (43000000, 44000000, [1994, 4374, 6490, 7781, 8983]),
            (44000000, 45000000, [2047, 4491, 6664, 7990, 9223]),
            (45000000, 46000000, [2101, 4608, 6838, 8198, 9464]),
            (46000000, 47000000, [2154, 4726, 7011, 8407, 9705]),
            (47000000, 48000000, [2208, 4843, 7185, 8615, 9946]),
            (48000000, 49000000, [2261, 4960, 7359, 8824, 10186]),
            (49000000, 50000000, [2314, 5077, 7533, 9033, 10427]),
            (50000000, 100000000000, [2368, 5194, 7707, 9241, 10668]),
        ],
    ),
];
// Reg.1.263(a)-1(f), Notice2015-82: invoice/item limits, cents.
// Source: Reg. 1.263(a)-1(f)(1)(ii)(D); Notice 2015-82.
pub(crate) const DE_MINIMIS_LIMIT_CENTS: i64 = 250_000;
// Source: Reg. 1.263(a)-1(f)(1)(i)(D).
pub(crate) const DE_MINIMIS_AFS_LIMIT_CENTS: i64 = 500_000;

// ---- Charitable contributions (IRC170(p), P.L.119-21 section70424, and Form8283 instructions) ----
// Non-itemizer cash contributions, 2026; direct eligible charities only.
// Source: IRC170(p), https://www.govinfo.gov/content/pkg/PLAW-119publ21/html/PLAW-119publ21.htm
pub(crate) const CHARITY_NONITEMIZER_SINGLE_DOLLARS: i64 = 1_000;
// Source: IRC170(p), https://www.govinfo.gov/content/pkg/PLAW-119publ21/html/PLAW-119publ21.htm
pub(crate) const CHARITY_NONITEMIZER_JOINT_DOLLARS: i64 = 2_000;
// Form8283 filing threshold and qualified-appraisal/SectionB general threshold, cents.
// Source: https://www.irs.gov/instructions/i8283
pub(crate) const CHARITY_FORM8283_THRESHOLD_CENTS: i64 = 50_000;
// Source: https://www.irs.gov/instructions/i8283
pub(crate) const CHARITY_SECTION_B_THRESHOLD_CENTS: i64 = 500_000;

// IRS Form8826 (Rev.September2017), line8; 26USC44: maximum disabled-access credit.
// Source: https://www.irs.gov/pub/irs-pdf/f8826.pdf
pub(crate) const DISABLED_ACCESS_CREDIT_LIMIT_CENTS: i64 = 500_000;

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    const OWN_SOURCE: &str = include_str!("year_constants.rs");

    // Every constant here carries a citation: the comment block directly above it holds a
    // `Source:` or `Statutory` marker, so an uncited constant fails CI.
    #[test]
    fn every_constant_has_a_citation_comment_directly_above() {
        let lines: Vec<&str> = OWN_SOURCE.lines().collect();
        let mut constants = 0;
        let mut uncited = alloc::vec::Vec::new();
        for (index, line) in lines.iter().enumerate() {
            let Some(rest) = line.strip_prefix("pub(crate) const ") else {
                continue;
            };
            constants += 1;
            let name = rest.split(':').next().unwrap_or(rest);
            let mut cited = false;
            let mut above = index;
            while above > 0 && lines[above - 1].trim_start().starts_with("//") {
                above -= 1;
                let comment = lines[above];
                cited |= comment.contains("Source:") || comment.contains("Statutory");
            }
            if !cited {
                uncited.push(name);
            }
        }
        assert!(constants > 100, "found only {constants} constants");
        assert!(uncited.is_empty(), "constants without a citation: {uncited:?}");
    }

    // The bound `key` ("min" or "max") that every `#[libertas_number(..)]` above each declaration
    // of `field` in `source` states. Declarations of the field without that bound are skipped.
    fn attribute_bounds(source: &str, field: &str, key: &str) -> alloc::vec::Vec<i64> {
        let lines: Vec<&str> = source.lines().collect();
        let mut found = alloc::vec::Vec::new();
        for (index, line) in lines.iter().enumerate() {
            let text = line.trim_start().trim_start_matches("pub ");
            if !text.starts_with(&alloc::format!("{field}:")) {
                continue;
            }
            let mut above = index;
            while above > 0 {
                let previous = lines[above - 1].trim_start();
                if !(previous.starts_with("#[") || previous.starts_with("//")) {
                    break;
                }
                above -= 1;
                if let Some(attribute) = previous.strip_prefix("#[libertas_number(") {
                    for part in attribute.split([',', ')']) {
                        if let Some(value) = part.trim().strip_prefix(&alloc::format!("{key} = ")) {
                            found.push(value.parse().expect("integer bound"));
                        }
                    }
                }
            }
        }
        found
    }

    // Attributes name a year constant as `$NAME` (the parser resolves it), while native rules
    // use their owning Rust constant, so a constant change reaches every range and rule.
    // Owning constant references stay present and repeated numeric literals stay absent.
    // Removed schema hour ceilings must not return as numeric or named bounds.
    #[test]
    fn attribute_and_rule_values_name_their_year_constants() {
        let federal_model = crate::federal_model::HUB_SOURCE;
        let adoption = include_str!("../family/adoption_source.rs");
        let disaster = include_str!("../retirement/disaster_source.rs");
        let care = include_str!("../family/care_source.rs");
        let credits = include_str!("../credits/general_business_credit.rs");
        let investment = include_str!("../credits/investment_credit_source.rs");

        // Each reference appears where the constant was repeated; the repeated literal is gone.
        for (source, reference, literals) in [
            (federal_model, "$SOCIAL_SECURITY_WAGE_BASE_CENTS", &["18450000"][..]),
            (disaster, "crate::year_constants::DISASTER_DISTRIBUTION_LIMIT_CENTS", &["2200000"]),
            (federal_model, "$QTP_ROTH_LIFETIME_LIMIT_CENTS", &["3500000"]),
            (investment, "REHABILITATION_PERIOD_MONTHS\n", &[]),
            (investment, "REHABILITATION_PHASED_PERIOD_MONTHS\n", &[]),
            (adoption, "$ADOPTION_CREDIT_MAXIMUM_CENTS", &["1767000"]),
            (disaster, "$DISASTER_DISTRIBUTION_LIMIT_CENTS", &["2200000"]),
            (care, "$DEPENDENT_CARE_BENEFIT_LIMIT_2025_CENTS", &["max = 500000"]),
            (investment, ".max(REHABILITATION_SUBSTANTIAL_FLOOR_CENTS)", &["> 500000"]),
            (credits, "$PAID_LEAVE_MIN_WEEKLY_HOURS", &["min = 20,"]),
            (credits, "$PAID_LEAVE_COMPENSATION_LIMIT_2026_CENTS", &["9600000"]),
            (credits, "$PAID_LEAVE_MIN_RATE_PERCENT", &["min = 50,"]),
            (credits, "PAID_LEAVE_MAX_WEEKS * 100 * i64::from(self.weekly_hours)", &["1200 *"]),
            (credits, "$SMALL_EMPLOYER_HEALTH_FTE_HOURS", &["max = 2080"]),
            (credits, "crate::year_constants::WOTC_FIRST_YEAR_CAP_STANDARD_CENTS", &["<= 600000"]),
            (
                credits,
                "crate::year_constants::WOTC_FIRST_YEAR_CAP_DISABLED_VETERAN_CENTS",
                &["<= 1200000"],
            ),
            (
                credits,
                "crate::year_constants::WOTC_FIRST_YEAR_CAP_LONG_UNEMPLOYED_VETERAN_CENTS",
                &["<= 1400000"],
            ),
            (
                credits,
                "crate::year_constants::WOTC_FIRST_YEAR_CAP_FAMILY_ASSISTANCE_CENTS",
                &["<= 1000000 :"],
            ),
            (credits, "$WOTC_FIRST_YEAR_CAP_DISABLED_LONG_UNEMPLOYED_VETERAN_CENTS", &["2400000"]),
        ] {
            assert!(source.contains(reference), "missing {reference}");
            for literal in literals {
                assert!(
                    !source.contains(literal),
                    "{literal} repeats a constant that {reference} names"
                );
            }
        }

        // 2026-10-16: Unit 47 removed the redundant schema hour ceilings. The source-specific
        // 12-week native constraint still names PAID_LEAVE_MAX_WEEKS, checked above.
        assert!(attribute_bounds(credits, "weekly_hours", "max").is_empty());
        assert!(attribute_bounds(credits, "leave_hours", "max").is_empty());
    }
}
