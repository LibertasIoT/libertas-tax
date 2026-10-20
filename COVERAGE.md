# Libertas Tax coverage audit

**Audited October 6, 2026; source rechecked October 8 at Tax commit `f7003c7`;
inventory reconciled with the reachability audit on October 8, 2026, at Tax
commit `c05275e`; form rows updated for the October 10 merges (platform adoption,
fix units F3-F7, B9-2 and B9-3) at Tax commit `51ba7f2`; form rows rechecked against
the `// Coverage:` blocks for the later merges (fix units F8-F16, B8, B9-1, B9-4 to B9-6,
B10-3, B10-4, B11 and the IRS-examples corpus) at Tax commit `aceb2e6`, after the second code-health unit; rows rechecked again for the later merges (Form 8379, the investment, farm and Schedule 2 units, the B11 disposition and IRA units, F17 and the storage-format 96 bump) at Tax commit `01eb922`; rows for W-2, Schedules A, C and SE, Form 2441 and Form 8283 updated for the October 12-13 merges (care, Schedule C specials, the foreign tax deduction, abroad and clergy, charity classes, F18 and the W-2 tip fix) at Tax commit `a83ee93`. Tax year: 2026.**

This is the App's only coverage document. The original audit used samples
revision `683d411`; comparison with the standalone Tax source found no tax-rule
changes, only the landing-action simplification and relocated documentation
references. Detailed evidence remains beside the linked source and tests.
Tax-year source revalidation remains an open acceptance gate: use the linked
IRS sources and source coverage comments, checking each actual tax year and
revision. The October 8 documentation cleanup did not perform a new line-by-line
tax-law audit of all forms.

Libertas Tax calculates a broad range of U.S. federal individual income tax
estimates, including wages, investments, retirement, self-employment, rentals,
dependents, deductions and credits. It remains a prototype for synthetic data.
It does not prepare a filing package, file a return, or calculate state or local
income tax. Significant federal situations and combinations remain outside its
calculation scope.

The audit finds **92 checklist entries with calculation paths a person can reach, one with a filing-requirement check only, and
0 without the listed calculation or relief implementation a person can reach**. Four other entries are deferred
or obsolete. These counts describe implementation presence, not complete forms
or the percentage of taxpayers served. **A current percentage of federal returns
calculated completely has not been established.**

## What the counts mean

The inventory retains the App's September 2026 checklist of 96 grouped entries
based on [Intuit's published individual-return forms list](https://turbotax.intuit.com/personal-taxes/irs-forms/),
checked again on October 6, and adds Form 4562-B as entry 97 (the 2026 draft
moved business amortization off Form 4562). Some entries combine several
documents or forms; others separate a worksheet from its source document. It is
a comparison inventory fixed apart from that addition, not a count of every IRS form, every form line, or every supported
tax situation. Additional implemented features appear separately below.

| Evidence category | Entries | Meaning |
| --- | ---: | --- |
| Calculation | 92 | At least part of the listed calculation exists and an interview page can enter its facts. Its limits still apply. |
| Requirement check only | 1 | Form 8938 can produce a filing notice; the statement is not prepared. |
| Missing | 0 | The listed computation or relief implementation is absent. Related questions, notices or historical input values do not count as implementation. |
| Deferred | 2 | Forms 1310 and 8857 are preparation outside the calculation scope. |
| Not applicable | 2 | Form 8903 is a legacy checklist entry; Form 8910 has no 2026 credit. |
| **Total** | **97** | **93 entries after excluding the deferred and obsolete entries.** |

There is some implementation for all 93 of those entries, including the
requirement-only check. **That is not 93 fully supported forms.** In particular,
a calculation label does not establish that every line, interview branch,
cross-form combination or filing requirement is implemented. A large form and
a small worksheet each occupy one row, so these counts cannot measure progress
toward complete consumer-return coverage.

## Estimates with assumptions and items left out

An estimate can be produced even when it leaves an item out. The
[estimate entry point](src/federal/federal_rules.rs) first uses the
[supported working-copy projection](src/core/unknowns.rs), which removes certain
unsupported or unconfirmed items and adds notices without deleting the accepted
answers. Other unknown answers take a documented conservative path. These
warnings do not necessarily block finishing the interview.

For example, an incentive-stock-option sale with unsupported special treatment
can be left out while the remaining wages still produce an estimate. That return
is not fully calculated. The behavior is covered by
`iso_sale_with_unsupported_special_treatment_is_left_out_with_a_notice`
in [business tests](src/tests/business/business.rs), and by
`omitted_equal_name_records_keep_distinct_nonblocking_correction_links` in
[correction-note tests](src/tests/federal/navigation_notes.rs).

Explicit unsupported selections can still block an estimate. The current
[refusal inventory](src/federal/federal_rules.rs) contains 14 `RareSituation` choices,
plus refused branches in the adjustment, credit and other-tax choices. Some
names cover only an unsupported portion of a wider implemented feature: for
example, there are foreign-income calculations, but not all treaty situations;
there are account calculations, but not Form 1099-QA ABLE distributions.
Whole-return exclusions are now dual-status, part-year, nonresident alien and
U.S. territory returns, and community property with a registered domestic partner;
both stay refused by name. The decedent's final return (the "Died in 2026" choice),
the section 6013(g)/(h) nonresident-spouse election (with its optional net
investment income tax joint election and the "you were the nonresident" choice)
and community property for a married-filing-separately return calculate
(rows 28, 47 and 79).

A return counts as completely calculated only when all its relevant income,
deductions, credits and taxes are included, with every required combination
supported. Merely reaching Review or receiving a refund/balance estimate does
not establish that condition. Conservative assumptions also need to be resolved
before treating an estimate as an exact result for the supplied facts.

## Federal implementation inventory

The linked files contain the owning records and implementation. Descriptions
state the principal implemented path and material limits, not an exhaustive
line-by-line tax-law specification. All Calculation rows remain subject to
source eligibility, history, validation and combination limits.

### Core return and common source documents

| Entry | Form or schedule | Evidence category | Current scope |
| ---: | --- | --- | --- |
| 1 | [Form 1040](src/federal/federal_rules.rs) | Calculation | Federal tax, payments, refund/balance and displayed return lines for accepted situations, including excess elective deferrals on line 1h, the Form 8814 tax in total tax, a rounded Additional Medicare Tax and footing checks on line 24a; line 38 is reduced by an estimated-tax penalty waiver request (see Form 2210); special income lines, taxes and combinations remain limited. |
| 2 | [Form W-2](src/forms/w2.rs) | Calculation | Wages, withholding and selected benefits, tips and payroll-tax boxes; box 12 code T is matched against the adoption records (a code T dollar no record accounts for is taxable on line 1f with a warning); code K (section 4999 tax) is calculated to Schedule 2 and no longer left out; box 12 code TP needs no tip records, and Form 4137 applies only to unreported tips, whose records need cover only the tips beyond code TP; church employee and ministerial wages are separated (see Schedule SE); other unsupported special boxes can leave a document out. |
| 3 | [Schedule 1](src/federal/federal_rules.rs) | Calculation | Every computed line is shown, including additional-income lines 8g, 8h, 8i, 8j, 8m, 8r, 8t, 8u, 8v and 8z (typed Other income kinds with the Olympic 24c rule and the Pub. 525 Worksheet 2 recovery figured with a same-year refund; a W-2's deferred pension and inmate pay to 8t/8u and out of earned income), operating-loss carryforwards, alimony, educator expenses reduced by the Form 8815 exclusion and the performing-artist deduction; residual adjustments remain gaps (line 24f is refused by name). |
| 4 | [Forms 1099-INT, DIV and OID](src/federal/federal_model.rs) | Calculation | Interest, dividends, discount and selected adjustments; boxes 3, 9 and 10 of a 1099-DIV (return of capital and corporate liquidation) are the Interest and dividends page's own document, `IncomeDocument::CapitalReturn1099Div`; box 1c and box 2c section 1202 gain follow the Form 8949 code Q choice (see Form 1099-B); the investment-statement unit adds the printed boxes with their source subsets (INT boxes 1-13, DIV 1a/1b, 2a-2f, 4/5, 7/8 and 12/13, OID 1-6 and 8-11), disjoint nominee and later-recipient components, the after-death share, annual debt instruments with the taxable bond-premium election, Treasury and private-activity OID/premium components, foreign-tax and REIT/special-capital components and documented capital stock losses, each counted once; informational expenses are accepted without an investment-expense deduction; prior-inclusion cases and other specialized ownership treatments remain limited. |
| 5 | [Schedule B](src/federal/federal_rules.rs) | Calculation | Interest/dividend totals listing payer amounts with their nominee, estate, prior-inclusion, accrued-interest, bond-premium and OID adjustments (retained investment components, no second tax calculation), a canonical Form 8815 link that reduces current interest once, seller-home mortgage interest first, and foreign account/trust facts with distinct accounts, countries and trust answers (a Not sure answer keeps ordinary tax and is not a completed filing answer); specialized subtractions and foreign reporting are not comprehensive. |
| 6 | [Schedule 8812](src/forms/schedule_8812.rs) | Calculation | Child, other-dependent and additional child credits, including income limits and credit ordering; a child attains 17 on the birthday, a child born and died in 2026 without an SSN counts on line 4, a child who died before the 17th birthday never attains 17 and keeps the child tax credit, a citizen child who lived abroad is not excluded, line 18a deducts the joint special-needs deemed excess, and every printed line except 2a, 2c (Puerto Rico and American Samoa income) and the reserved line 15 is a return row; a running Form 8862 ban makes the credits zero. |
| 7 | [Schedule 3](src/federal/federal_rules.rs) | Calculation | A full schedule that aggregates implemented credits and payments, with footing checks; a line for a credit is not implementation of every source credit. |
| 8 | [Form 1099-R](src/retirement/retirement.rs) | Calculation | Simplified-method, payer-reported and IRA-basis (Form 8606) paths. The 1099-R page also offers, as its `TaxableMethod` choices, the General Rule, variable annuity, full discharge, plan and annuity withdrawals, pre-1982 (legacy) withdrawals, civil service (CSRS/FERS) lump sum and child annuity; a railroad RRB-1099-R is its own document (`Rrb1099R`), employer-securities (NUA) share sales are `NuaShareSale`, rollover property proceeds are `Disposition::SoldAndRolledOver`, annuity exchanges are `DistributionCode::Exchange` (code 6 or W) and a charitable gift annuity is code F. Every box 7a code and Table 1 pair routes to a calculated branch (corrective code 8, not-income N/P/R, no-additional-tax 9/E/F/U, prohibited transaction code 5 with box 7b), with Roth rollover destinations, the periodic-payment (SEPP) and age-59½ exceptions, SECURE 2.0 repayments on IRA rows, the split-interest QCD, the once-a-year IRA rollover rule, Form 8915-F Part IV and Form 4972 lines 9 and 18. The IRA unit (B11) adds Roth IRA rows (`RothWithdrawal`) with a 60-day Roth-to-Roth rollover, any IRA early-distribution exception, a repayment of a birth or adoption, emergency, domestic-abuse or terminal-illness distribution, a qualified disaster share and the Roth SIMPLE IRA 25% rate (codes J and S); an employer's Roth SEP or Roth SIMPLE contribution (`RetirementTreatment::EmployerRothContribution`, taxable, a 2026 conversion); a code S row that is Kept or rolled into another SIMPLE IRA (`SimpleDisposition`); a code 8J return of an excess with earnings as an IRA amount; a code 5 prohibited transaction tested as a January 1 distribution; and an owner past 59½ before 2026 never taxed the 10% additional tax (exception 12). Refused by name: a Roth IRA prohibited transaction without box 7b, a reportable death benefit (code C), a charitable gift annuity with a box 3 gain (code F) and code C with D; a changed SEPP series is refused (`ISSUE_SEPP_RECAPTURE`). Other special histories remain limited. |
| 9 | [Schedule C](src/federal/federal_rules.rs) | Calculation | Business income, expenses, inventory, selected assets, vehicles and owner deductions; a standard-mileage vehicle's property tax on line 29 and the employer tip FICA on line 23, start-up and section 197 amortization on line 27b (Form 4562-B), the Form 8881 reduction on line 17, listed-property recapture on line 6, qualified tips (Schedule 1-A) removed from QBI, and the Form 4136 fuel records; the [header and expense specials](src/business/schedule_c_specials.rs) add the de minimis safe-harbor election, leased vehicles, depletion, returns and allowances, other business income, owner SIMPLE and solo retirement plans, and information-return filing; ministerial fees and notary income follow [Pub. 517](src/income/clergy.rs); specialized businesses and treatments remain limited. |
| 10 | [Schedule SE](src/federal/federal_rules.rs) | Calculation | Self-employment tax, wage-base coordination and optional methods (printed lines follow the instructions); church employee income (lines 5a-5b) is calculated, also beside a partnership K-1, and farm 1099-G and PATR income count; ministers and religious workers follow [Pub. 517](src/income/clergy.rs), including the parsonage allowance in SE income and approved Form 4361 and Form 4029 exemptions. |
| 11 | [Schedule D](src/investment/capital.rs) | Calculation | Gain/loss netting, carryovers and preferential-rate classes; lines 1a-3 and 8a-10 total Form 8949 columns (d), (e) and (g) as well as (h); lines 4 and 11 carry the Form 4684 casualty gains, Form 4797 Part I gain (line 11), Form 6252, Form 6781, Form 8824 and Form 2439 amounts so the lines foot to 7 and 15, collectibles (code C) and personal-use losses (code L) are lines 1b/8b, and code E selling expenses never create a loss on personal-use property; qualified opportunity fund inclusions and deferrals (Form 8997, codes Y and Z) and a fund's section 1202 gain are calculated; straddles and non-rental like-kind exchanges are refused by name. |
| 12 | [Form 1099-B and Form 8949](src/federal/federal_model.rs) | Calculation | Investment lots from a Form 1099-B or 1099-DA (stocks, bonds, digital assets, options bought or written), basis, selected wash-sale and incentive-stock-option sale paths; Form 8949 rows per lot with boxes A-L and codes B, D (market discount), E (option premium), M (several sales), N (nominee), O, Y, Z (opportunity funds) and Q; a written option is always short-term, and an empty basis counts as zero with a warning. The Investment sales page also writes employee-stock, ESPP and NUA share sales, section 1202 sales, the section 1244 loss, worthless securities and nonbusiness bad debts; section 1242 stock is not supported; the personal-use part of a mixed-use business or rental sale and equipment rented out outside a business also write Form 8949 rows. A contingent payment debt instrument, a code R rollover, the code X exclusion, a trader election, a carried-interest gain and a loss the broker says the proceeds cannot support are one `SaleSpecial` choice and are left out with a notice; there is no broker-summary entry. |
| 13 | [Forms 8995 and 8995-A](src/activity/business_rules/mod.rs) | Calculation | QBI limits, losses, selected aggregation and REIT/PTP amounts; section 224 tips are out of QBI, line 5 is 20% of line 4, a 1099-PATR box 7 patron files Form 8995-A with Schedule D line 6, distributions count as the box 8 rule says, the UBIA test uses the depreciable period, and a released earlier-year loss is part of line 3; a K-1 cooperative section 199A item is refused by name and not every PTP or K-1 combination is supported. |
| 14 | [Schedule EIC](src/forms/schedule_eic.rs) | Calculation | Earned-income credit using stored draft 2026 tables and qualification facts; a child attains 19 or 24 on the birthday, a child born and died in 2026 without an SSN qualifies, a married child counts only when claimable, a separated spouse filing separately keeps the self-only credit, and Schedule EIC lines 1-6 are rows for each child. |
| 15 | [Schedule 2](src/federal/federal_rules.rs) | Calculation | A full schedule of implemented additional taxes and repayments, including the Form 8814 tax and education-credit recapture outside the credit base, with footing checks; it now carries the section 4999 tax on excess golden parachute payments (line 13k; W-2 code K and 1099-NEC box 3), uncollected Social Security and Medicare tax on tips and group-term life (W-2 box 12 A, B, M and N), the Coverdell excess (Form 5329 Part V, a beneficiary's own accounts) and ABLE excess (Part VIII). Refused by name on the Other taxes page: other alternative minimum tax adjustments, housing-credit and look-back recapture (Forms 8611, 8697, 8866), a deferred section 965 tax and sixteen named rare situations (`RareAdditionalTax`: Form 4255 net or excessive payment, other credit recapture, installment lots and timeshares, installment interest history, direct housing partnerships, fractional charitable recapture, sections 409A, 457A, 72(m)(5), 4970 and 4985, and PFIC interest); residual taxes and recaptures remain gaps. |
| 16 | [Forms 1099-K, NEC and MISC](src/federal/federal_model.rs) | Calculation | Each document is entered or imported once and every nonzero box is routed by the person's answer to the business, rental, royalty, equipment-rental, hobby or Other income destination (the 1099-K top-of-Schedule 1 entry for money that was not income); hobby sales and the Schedule 1 line 8 kinds are pages, no longer refused. Box 3 of the 1099-NEC (excess golden parachute) is included in box 1a and adds only the 20% section 4999 tax on Schedule 2 line 13k (no longer refused); box 15 of the 1099-MISC (section 409A) is refused by name; cash-tip and overtime boxes, boxes 7 and 12 and the FATCA checkbox are not read. |
| 17 | [Schedule E page 1](src/federal/extended_rules.rs) | Calculation | Rental/royalty income, expenses, depreciation, passive limits and selected vacation-home/professional cases; days both rented and personal are a third count for the section 280A home test and expense split, line 22 carries prior-year losses (also a professional's nonpassive loss), the section 280A carryover is limited in a non-home year, K-1 (1041) royalties are line 33(f) and line 42 prints from any source. Rental specials: short-stay participation and 39-year transient depreciation, nondepreciable-property and self-rental recharacterization, converted basis, Form 6198 at-risk before the passive limits, professional and former-passive rentals (line 43), and mortgage interest, points (straight line over the term) and premiums from the canonical loan ledger (see Form 1098). Royalty columns with cost and percentage depletion, the mineral and oil-and-gas property limits, the 65% limit and Form 6251 line 2d. The disposition unit calculates a rental sold after personal use (the price, expenses and cost divided between the rental and personal parts, with a required personal-use share), a rental held one year or less, the real estate professional net investment income safe harbor, an installment sale with a loss on one part or with suspended losses (section 469(g)(3) ratio), a condemnation (section 1033) and a partial disposition of a building (lines 1b and 1c). Refused by name: a REMIC residual interest, substantial guest services (belong on Schedule C), mixed Professional answers under the all-rentals election, the sale of a converted or gifted rental, a prior at-risk sale loss on a rental with personal use, a rental with a rehabilitation credit sold on installment or exchanged, a partial disposition on an exchanged rental or after the sale, and an operating-interest, services, foreign or production-above-1,000-barrel royalty; other specialized activities and histories remain limited. |
| 18 | [Schedule A](src/federal/federal_rules.rs) | Calculation | Medical, SALT, mortgage lines 8a-8e (see Form 1098), investment interest, gifts with carryover labels, the GST tax (line 6), gambling losses on line 17a, qualified disaster loss and the increased standard deduction (17b, 17c), the estate tax on income in respect of a decedent (17e), bond premium (17f), a debt-instrument loss (17g), claim of right (17h), an annuity loss (17i), disabled-employee expenses (17j) and educator and coach expenses (17k, including early-childhood educators), the 2/37 limitation and ordinary casualty losses; foreign income taxes on line 6 when chosen as a deduction instead of the foreign tax credit, with the matching Form 8960, Form 2555 and Form 6251 coordination; every numeric line carries its form line. Casualty of business or income property stays with Form 4684; the two checkbox rows (8 and 19) are not yet rows; the older "other itemized lines" refusal is gone. |
| 19 | [Form 4562](src/cost_recovery/depreciation.rs) | Calculation | MACRS, bonus and section 179; listed-property recapture when business use falls to 50% or less; amortization is on the separate Form 4562-B (entry 97), whose cost records the Business page's "More about this business" asks (start-up costs and section 197 intangibles; the farm page asks start-up costs); an asset placed in service and sold in 2026 takes no MACRS or bonus and keeps its section 179 as recapture, an unused section 179 carryforward of a disposed asset leaves line 10 and restores basis, a mid-quarter asset's last short year is prorated to the disposal quarter and a partial disposition of a building is calculated (these disposition refusals are gone); special property classes (`SpecialDepreciation`) and other recapture combinations remain limited. |
| 20 | [Form 1098](src/home/mortgage_limit.rs) | Calculation | One canonical loan ledger: Form 1098 documents from transferred lenders link by saved ID to one balance ledger (home, use and balance facts asked on the first lender). Use of proceeds (home, refinance, partly home, other), main, second, boat/RV and third homes, mixed-use principal applied monthly, Pub. 936 Table 1 with the first/last, monthly and within-limit balances, points (purchase points on line 8a, ratable or improvement points on 8c, the unamortized balance of an ended loan), the box 5 premiums with the phase-out, box 4 refunds, box 10 as taxes (5b) and the rental share of a linked loan are calculated. Binding-contract date exceptions and exact draw-ledger tracing are excluded. |
| 21 | [Form 8889 and Form 1099-SA](src/health/hsa.rs) | Calculation | HSA contributions, the funding, rollover, deemed-distribution and correction fields on the HSA page, and distributions and inheritance through the 1099-SA document (own, returned excess, prohibited, inherited); a person who can be claimed as a dependent (no deduction, employer amounts as excess), the equal family split without an agreement, and W-2 HSA deposits bound to their source year. |
| 22 | [Form 8863 and Form 1098-T](src/federal/federal_rules.rs) | Calculation | American opportunity and lifetime learning credits, expense coordination, phaseouts and recapture support; the scholarship-income election (none, Automatic within the living costs, or an amount) is calculated, with Automatic searching the whole included range including the earned income credit and bracket steps, the credit requires a valid SSN for the filer (either spouse jointly) and the student, and a running Form 8862 ban on the American opportunity credit figures its students with the lifetime learning credit. |
| 23 | [Form 1099-G](src/payments/tax_refund.rs) | Calculation | Unemployment and selected tax-refund/recovery paths; box 5 RTAA payments are calculated, box 6 names the recipient and box 8 goes to the refund entry; source-specific special treatments remain limited. |
| 24 | [Form 2210](src/payments/underpayment.rs) | Calculation | Regular penalty method, safe harbors, withholding dates and a restricted Schedule AI (dated credits); a waiver request (box B) reduces line 19 and Form 1040 line 38 by the amount requested, capped at the penalty, and the computed penalty stays as its own row; line 2 includes the Form 4255 recapture and the Trump-account tax, household employment tax follows section 3510(b)(2), and the child tax credit, other-dependent credit and additional child tax credit follow children's birth dates while every other credit is its annual amount in every column; future rates are provisional; farmers and fishermen route to Form 2210-F. |

### Business and commonly used supporting forms

| Entry | Form or schedule | Evidence category | Current scope |
| ---: | --- | --- | --- |
| 25 | [Form 8880](src/retirement/pension_contributions.rs) | Calculation | Saver's credit with contribution, distribution-lookback, eligibility, income and tax limits. |
| 26 | [Form 1116 and its Schedule B](src/federal/extended_rules.rs) | Calculation | Typed Form 1116 for each category (general, passive, foreign branch, section 901(j)) and tax system (regular, AMT) with country, HTKO and Total columns, Schedule B for each category with the origin years 2016-2025 as columns, the qualified-dividend and capital-gain adjustments (Worksheets A and B, the Worksheet for Line 18), the paid/accrued election and Schedule K-3 rows by country; allocation facts a person cannot know take a conservative path with one note each. A checked unsupported situation, a Pub. 514 rate-group mix and a tax figured on Form 8615 or Schedule J leave the credit out with a note; line 3e counts each K-1 gain and each Schedule K-3 box 11 item once; section 951A, treaty re-sourced income, lump sums, PTEP, section 962, Form 1116 line 22 and contested taxes are not calculated. |
| 27 | [Form 8962 and Form 1095-A](src/health/marketplace.rs) | Calculation | Annual/monthly credit, repayment (line 27), selected policy allocations and self-employed coordination; household income includes a Form 8814 child's income and nontaxable Social Security once; the marriage-year Part V alternative, QSEHRA permitted benefit and affordability (including a dependent employee's household), lawful-presence reference months, payment exceptions (grace period, appeal), months where only some covered people qualify, policies shared among three or more tax families, allocated-plus-other policy months, and a filer who can be claimed but is not claimed; coverage established under more than one business remains limited to the admitted classes. |
| 28 | [Form 8960](src/federal/federal_rules.rs) | Calculation | Net investment income tax for supported investment, property and pass-through sources, with farm rental recapture, personal casualty losses on line 7, a Form 8960 printed whenever MAGI exceeds the threshold, and the section 6013(g) joint-return election (with the nonresident spouse's identity); foreign-corporation and PFIC cases remain gaps, and a K-1 gain's line 5c adjustment is refused by name. |
| 29 | [Schedule K-1 from Form 1065](src/passthrough/partnership.rs) | Calculation | Selected partnership income, basis, at-risk, passive, SE, QBI and credit paths. Losses carry by class through the basis, at-risk and passive limits (the former class-limited refusals are gone; 2025 carries are entered per class). Typed per-code records for boxes 11, 13 and 20 (including the box 1 adjustments of codes F and V, section 199A REIT and PTP items, box 20 codes AB, AC, AD and AI), the sale or cash liquidation of the whole interest (section 751/741 split, at-risk and section 469(g) releases, NIIT and QBI treatment) and box 19 distributions above basis; codes the App does not compute are left out with a notice. Refused by name: other box 11, 13 and 20 codes, a disposition other than a sale or cash liquidation, a limited or released long-term box 9b/9c part, cooperative section 199A items and investment interest with an at-risk limit; a complete K-1 box set is not supported. |
| 30 | [Form 2441](src/forms/form_2441.rs) | Calculation | Parts I-III from the [Child and dependent care page](src/family/care.rs): providers, the credit for qualifying dependents (a qualifying person with no taxpayer number earns none; a child attains 13 on the birthday), employer benefits per W-2 box 10 and the own-business deduction, monthly deemed income for a disabled or student spouse, and prior-year expenses paid in 2026 (Worksheet A). |
| 31 | [Schedule K-1 from Form 1120S](src/passthrough/scorp/mod.rs) | Calculation | Selected S-corporation income, basis, debt, passive, QBI and AMT paths; losses carry by class through the basis and at-risk limits, and the box 10, 12 and 17 codes, the sale of all shares and a distribution above stock basis (a passive gain taken as not passive) are calculated; codes the App does not compute are left out with a notice, and unmodeled boxes and activity combinations remain limited. |
| 32 | [Form 8582](src/federal/extended_rules.rs) | Calculation | Supported rental/pass-through passive-loss pools, allocation and history, with lines 6 to 11 per the form (line 6 never below zero, a professional's rental loss out of line 6 and MAGI, none of lines 4-9 for a separate filer who lived with the spouse); a publicly traded partnership's box 1 is limited by itself under section 469(k), and its prior-year unallowed loss does not re-enter Part V; footing checks run on the printed lines; not general passive-credit computation. |
| 33 | [Form 1098-E](src/federal/federal_rules.rs) | Calculation | Student-loan interest deduction with eligibility and income limits. |
| 34 | [Form 8283 and Form 1098-C](src/deductions/charity.rs) | Calculation | Noncash gifts by charity class: used goods, similar-property groups, stock, vehicles, conservation easements (including historic buildings) and appraised property, with Section A/B placement, appraisal dates and requirements, the percentage limits, the floor and per-class carryovers in Schedule A lines 11-15. Partial interests, debt-encumbered and business-inventory gifts keep their own notes; attachments and printed text columns are not prepared. |
| 35 | [Form 4797](src/investment/section1231.rs) | Calculation | Selected asset, rental and farm-land sales, section 1231 history and recapture; line 9 is the whole nonrecaptured section 1231 loss balance, Part IV is on the 2026 line numbers, selling expenses join line 23, source rows foot lines 2-8, 15- and 20-year classes recapture as section 1250 property and livestock use the 24- and 12-month holding periods; Part III columns and lines 6, 12-14, 16 and 19 are not printed. The B11 disposition unit computed thirteen former refusals: a sale in the placed-in-service year, an unused section 179 carryforward, a mid-quarter last-year disposal, a personal-use part's gain, mixed-use and short-held installment sales, Part IV recapture (section 179 column (a) and a Form 2106 vehicle's column (b)), short-held and personal-use rental sales, the net investment income safe harbor, and rental installment gain/loss and the section 469(g)(3) release; it also calculates a related buyer (section 267 loss disallowance, section 1239 ordinary gain), condemnations (section 1033), partial dispositions (Reg. 1.168(i)-8), the whole-business sale (Form 8594 allocation) and rented-out equipment. `AssetDisposed` is now only purchased section 197 intangibles and sections 1254 and 1255 property; installment sales to related buyers or dealers, contingent-price sales, like-kind exchanges and casualties keep their refusals. |
| 36 | [Form 8959](src/federal/federal_rules.rs) | Calculation | Additional Medicare tax for supported wages and SE income, with explanations on the 2026 Parts I, III, IV and V; railroad (RRTA) wages are refused by name. |
| 37 | [Form 4952](src/investment/investment_interest.rs) | Calculation | Investment-interest limits with numeric regular and AMT lines 1-8, selected gain elections (the election covers a child's Form 8814 dividends and PTP passive income on line 4a; capital-first or allocated line 4g), carryovers and AMT coordination; source-paid interest and carry, short- and long-term investment loss portions, allowed connected income-tax expenses, gross direct royalties with allowed costs and source-determined Schedule A line 9 or Schedule E line 13 destinations (also with the standard deduction). An unresolved mixed-source allowance or carry, an incomplete early royalty context or untyped bond-premium character leaves only that deduction or election benefit out; the former Coverdell, capital-carryover and income-tax-expense refusals are gone. |
| 38 | [Form 5695](src/federal/federal_rules.rs) | Calculation | Prior residential-energy credit carryforward treatment; this is not a claim of support for new 2026 purchases. |
| 39 | [Form 8829](src/home/home_office.rs) | Calculation | Actual home-office expenses and carryovers, including mortgage insurance premiums, the line 11 taxes step and column labels; the simplified method is also available through Schedule C (its income limit takes line D). |
| 40 | [Schedule K-1 from Form 1041](src/federal/federal_model.rs) | Calculation | Selected estate/trust income, final-year deductions and loss histories; unmodeled boxes and activity/loss cases remain limited. |
| 41 | [Form 8606](src/retirement/retirement.rs) | Calculation | Nondeductible IRA basis, conversions, Roth distributions and selected repayment paths (line 6 drops 2026 disaster and SECURE 2.0 repayments); inherited IRA and Roth distributions are the Inherited IRA record (`IncomeDocument::InheritedIra`) linked from a 1099-R (`DistributionCode::Inherited`) or a qualified charitable distribution (`QcdGift.inherited`); the IRA unit adds a line 2 adjustment (`IraBasis.adjustment`) beside the imported line 14, reservist repayments on the item, the date-of-death value on a final return, lines 22 and 24 adjustments, a Roth row's early-distribution exception and the QCD capped at the pre-tax pool, IRA Social Security MAGI from all benefits under the regular method, and a Form 8915-F Part IV home-purchase repayment that leaves line 7 while line 6 stays; the recharacterization, returned-contribution and Roth-history refusals are gone; other incomplete or special histories remain limited. |
| 42 | [Form 5329](src/retirement/retirement.rs) | Calculation | Selected early-distribution, contribution-excess, RMD and IRA, Roth and HSA additional taxes, with exceptions 02 (periodic payments) and 12 (age 59½ before 2026), a disabled Roth owner's rows, code S rollovers at 25%, the first required minimum deferred to April 1, 2027 and the Trump-account tax; the ABLE distribution path is reachable through the 1099-QA document; the IRA unit adds a second exception on one distribution (code 99), line 10 absorbing a carried traditional excess into the deduction and Form 8606 line 1, line 12 returned prior excess outside line 4b, a 2027 return of a 2026 contribution (`ReturnedContribution`), a 529-to-Roth transfer above the year's IRA room as a Roth excess and a taxable 529 withdrawal, the long-term-care $2,600 exception limit across plans, an Exception 12 test on January 1 for a code 5 row and the four former RMD confirmations; Part V (Coverdell, `coverdell.rs`) and Part VIII (ABLE, `able.rs`) are owned by their own records and no longer refused through `EducationAccountTaxes`; exception 14 cannot apply in 2026, and some excess-account taxes remain gaps. |
| 43 | [Form 7206](src/health/health.rs) | Calculation | Self-employed health deduction, owner/business limits and Marketplace coordination within the supported business structures. |
| 44 | [Form W-2G](src/federal/federal_model.rs) | Calculation | Winnings, withholding and 2026 gambling-loss treatment; unsupported special records can be left out. |
| 45 | [Schedule F](src/farm/farm.rs) | Calculation | Cash-method farm income, expenses and selected cooperative, conservation, deferral and land-sale rules; a standard-mileage vehicle's property tax on line 29, Part II rows for start-up amortization, prepaid supplies and the CCC deficiency, crop insurance on line 41, CCC-pledged sales on line 37, and one farm-income figure (with routed 1099-G and 1099-PATR income) for the section 175 limit, Schedule J and the at-risk test; the farm-specials unit (B12) adds Schedule F header and source facts with canonical land and production identities, section 126 cost-sharing exclusion with linked parcel basis, section 175 and prepaid-supply pools allocated proportionally with separate regular and AMT carries, the current section 180 fertilizer election, held production costs, documented whole-month poultry ratable periods, weather-timing and CCC election and commodity histories, and Schedule J retained source histories; the broad completeness and whole-farm special gates are gone, and an unavailable optional section 175, Schedule J or crop benefit is left out with a named notice; accrual, ADS, interest allocation, syndicate and other special farming cases remain limited. |
| 46 | [Form 8862](src/forms/form_8862.rs) | Calculation | Selected credit claims after disallowance, using filer/dependent facts and denial history; line 10 ages follow the form, line 8 prints the month and day of a child's birth or death, and a running 2-year or 10-year ban makes that credit group zero with a note (an American opportunity student takes the lifetime learning credit); a days entry contradicting the main-home answer is refused. |
| 47 | [Form 1310](src/federal/federal_model.rs) | Deferred | Preparing Form 1310 (the refund-claim form, signatures and court certificate) is deferred. The decedent's final return itself calculates: the "Died in 2026" choice on About you or your spouse, the full standard deduction, residence tests over the days alive, the after-death part of 1099-INT/DIV/R statements left to the estate with a section 691(a) note, carryovers ending with a note, and the rest of every Form 8915-F spread; death adjustments the App cannot compute are left out with a notice. |
| 48 | [Form 7203](src/passthrough/scorp_basis.rs) | Calculation | Stock/debt basis, losses, distributions and loan-repayment effects for supported S-corporation histories; Part III runs over the loss-class catalog (ordinary, rentals, capital, section 1231, section 179, gifts, foreign taxes and four box 10/12 classes), and the held-back amounts carry instead of being refused. |
| 49 | [Form 1040-ES computation](src/forms/form_1040_es.rs) | Calculation | An optional **2027** estimated-tax worksheet based on the return or selected expected changes; uses 2026 parameters and is not a voucher or final 2027 computation. |

### Less common income and tax situations

| Entry | Form or schedule | Evidence category | Current scope |
| ---: | --- | --- | --- |
| 50 | [Forms 8936 and 8936a](src/credits/clean_vehicle.rs) | Calculation | Eligible personal-use vehicle claims and transfer reconciliation; acquisition/service dates and special/business uses limit scope. |
| 51 | [Form 8915-F and the 8915-D checklist family](src/retirement/disaster/mod.rs) | Calculation | Selected current and prior Form 8915-F disaster distribution/spread/repayment paths, including Part IV home-purchase distributions and a final return that includes the rest of the spread for any disaster year; a qualified disaster share of a Roth IRA distribution (line 25b) and a home-purchase distribution repaid in 2026 are calculated; this does not establish a separate complete Form 8915-D implementation. |
| 52 | [Form 2555](src/foreign/foreign_housing.rs) | Calculation | Per-person foreign earned income exclusion (bona fide residence or physical presence with the entered day count), the housing exclusion and deduction with the 2026 amounts, the stacking worksheet, the §911(d)(6) Form 1116 interaction; foreign pay marked on W-2s and businesses. Tax treaties (Form 8833) are refused by name. |
| 53 | [Form 982 and Forms 1099-C/A](src/home/debt.rs) | Calculation | Selected debt cancellation, foreclosure, exclusions and attribute reduction; specialized debt/property combinations remain limited. |
| 54 | [Form 8379](src/attribution/injured_spouse/mod.rs) | Calculation | Injured-spouse allocation and an estimated refund share, entered on an editor with live parent-relative bounds and the ordinary accepted-save lifecycle; Part III keeps the accepted joint amounts and their source owners (line 15 is line 12e only, line 18 includes Schedule 2 line 1z and Part II), allocates the child tax credit and earned income credit with explicit feasibility-bounded choices for mixed child pools, keeps actual payer and property source for payments, reconciles withholding lines 25a, 25b and 25c separately, and treats community-property refund rules as a separate source-property and debt overlay. Verified historical ruling patterns retain their assumptions. Multistate or unverified collection regimes, a community federal-tax debt and a farmland installment offset leave out only the named Form 8379 benefit; the ordinary federal estimate is kept. Uses stated allocation assumptions and does not replace the IRS's final determination. |
| 55 | [Form 8938](src/foreign/foreign_assets.rs) | Requirement check only | Notices from foreign account/trust facts and highest asset value; preparing the statement itself is deferred (inventor decision of 2026-10-08); no year-end asset inventory or general foreign-reporting compliance. |
| 56 | [Form 6252](src/investment/installment.rs) | Calculation | Selected business/rental/farm and earlier-year installment sales; a sale of property held one year or less, a mixed-use sale (the business part only), a rental installment sale with gain and loss on different parts and the earlier-year installment qualified business income (by business) are now calculated; current investment, related-party, dealer, contingent-price and other special sales remain limited. |
| 57 | [Form 3800](src/credits/general_business_credit.rs) | Calculation | Selected component credits, K-1 amounts, limits and carryovers (the regular-liability base includes the Form 8814 tax and leaves out the education-credit recapture); not every business credit, passive release, elective payment or transfer. |
| 58 | [Form 8615](src/family/kiddie.rs) | Calculation | Young-filer tax with parent and sibling facts and selected gains; the parent's Form 2555 and the parent's and other children's Form 4952 line 4g election are calculated; Schedule J on the parent's or another child's return and the child's own Form 4952 election are refused by confirmation, and other special worksheet combinations remain limited. |
| 59 | [Form 1099-Q](src/family/education_account.rs) | Calculation | Taxable 529/Coverdell earnings (Pub 970 ch. 7, credit first, K-12 cap), Form 5329 Part II, educator and student-loan reductions, 529-to-Roth as box 4b; a 529-to-Roth transfer is limited by the owner's IRA dollar room (the excess is a Roth excess and a taxable 529 withdrawal); code 6 (a prohibited transaction) is a withdrawal of the January 1 value that no expenses cover, with the 10% tax. Forms 1099-QA are not covered. |
| 60 | [Form 4835](src/farm/farm_rental.rs) | Calculation | Lines 1-7 and Part II with the shared Schedule F reports and the section 469 pool; the farm-specials records are shared and the whole-rental special switch is gone; line 32 is blank on a loss (line 34c after Form 8582) and a publicly traded partnership is limited by itself; the reused expense fields show Schedule F line numbers on the rental page (Display Template `label` cannot target a scalar field). |
| 61 | [Form 2106](src/business/employee_expenses.rs) | Calculation | Reservist, qualified performing artist and fee-basis official expenses under §67(h), with W-2 code L reimbursements, the artist tests and Part II vehicles on the shared vehicle types; impairment-related expenses (Schedule A) and a sold vehicle's gain and Part IV column (b) excess depreciation (Schedule 1 line 8z) are calculated; leased or employer-provided vehicles remain gaps. |
| 62 | [Form 6781](src/investment/section1256.rs) | Calculation | Part I (1099-B boxes 8-11, 40/60 split, open-contract marks, section 1212(c) carryback) and K-1 box 11 code C. Part II straddles are refused by name. |
| 63 | [Form 8801](src/federal/minimum_tax.rs) | Calculation | Prior minimum-tax credit from selected retained histories (the regular-liability base includes the Form 8814 tax); Parts I-III come from identified prior Form 6251 and exclusion facts, exclusion-only minimum-tax loss carryovers and carrybacks, retained modified Form 2555 rate worksheets and general, passive and foreign-branch foreign tax credit categories, with separate prior returns keeping their status before sharing one current limit; unavailable historical worksheets, former-joint attribution, nonresident real-property floors, other foreign categories and loss-history combinations leave only this credit out. |
| 64 | [Forms 8332 and 2120](src/federal/federal_rules.rs) | Calculation | The declarations' tax effects as dependent facts (§152(e), §152(d)(3)); preparing or attaching the forms is outside scope. |
| 65 | [Form 6251](src/federal/federal_rules.rs) | Calculation | AMT with selected income, ISO, depreciation, passive, gain and ATNOL adjustments, a property-tax refund on line 2b, the oil and gas 65% limit on line 2d and the section 1202 fund preference on line 2h; supported income-limited deductions refigured through their existing owners, separate regular and AMT farm supply balances and business carries, and investment-interest AMT carries; other preference and cross-form cases remain limited. |
| 66 | [Schedule H](src/business/household_employment.rs) | Calculation | Household payroll taxes, FUTA and selected sick-pay/state-plan facts; an unknown credit-reduction rate takes the gross-tax path with a notice. |
| 67 | [Form 4136](src/credits/fuel_credit.rs) | Calculation | Fuel-use lines (1a-17, including alternative fuels, vendor and card-issuer lines), the prior-year cash-method recovery and the partner K-1 box 20 code C credit: the fuel records live on the business and farm pages ("More about this business" and "More about this farm", B7a) and home kerosene on More credits; some aviation/mixed-use records are outside scope. |
| 68 | [Form 4137](src/income/employee_tips.rs) | Calculation | Selected unreported/allocated tip payroll taxes; railroad treatment remains limited. |
| 69 | [Form 8814](src/family/child_election.rs) | Calculation | Parent's election per eligible child with the child's 1099-INT/DIV/OID statements, the 2026 amounts, qualified-dividend and capital-gain pass-through, the Alaska PFD share and EIC/NIIT routing; every disqualification leaves that child out with a note. |
| 70 | [Form 8853](src/health/medical_account.rs) | Calculation | Archer MSA contributions (Part I) on the HSA page; Archer and Medicare Advantage MSA distributions and inheritance (Sections A and B) through the 1099-SA document; long-term care contracts (Section C) through the 1099-LTC document with per-insured period records; W-2 code R Archer employer contributions with outside-W-2 amounts and prior carry using only unused room, nonqualified contracts and qualified viatical/accelerated-death proceeds with the other-payee fact. |
| 71 | [Schedule R](src/federal/extended_rules.rs) | Calculation | Elderly/disabled credit for supported age and benefit facts; employer disability pension income is a stored computation, no page. |
| 72 | [Form 4684](src/home/casualty/mod.rs) | Calculation | Selected personal-use casualty calculations; a 2026 presidentially declared major disaster (FEMA DR-) is a qualified disaster loss under P.L. 119-108 (the $500 floor, no 10% reduction, added to the standard deduction) while emergency and fire declarations are not, and line 4 counts only reimbursement received; business/income property, Ponzi, other sections and prior-year elections remain gaps (`Casualty`). |
| 73 | [Form 3903](src/business/moving.rs) | Calculation | Armed Forces and intelligence-community moves (§217 as amended by P.L. 119-21) with the 2026 split mileage rate and W-2 code P reimbursements; other moves are left out with a note; Form 2555 allocation is excluded. |
| 74 | [Form 8815](src/investment/savings_bonds.rs) | Calculation | Series EE/I savings-bond interest exclusion with Form 8818 facts, tuition coordinated credit-first with Form 8863, the 2026 phaseout and the age-24 issue-date rule; a bond reported on an earlier return is left out with a note. |
| 75 | [Form 8824](src/investment/exchange.rs) | Calculation | Rental-for-real-property exchanges: deadlines, boot, basis, carried-over and excess basis depreciation, §1031(f) related parties, failed exchanges taxed as sales. |

### Specialized credits and limitations

| Entry | Form or schedule | Evidence category | Current scope |
| ---: | --- | --- | --- |
| 76 | [Form 8839](src/family/adoption.rs) | Calculation | Selected adoption credit, employer exclusion and carryovers; Part III for joint employees has no child-count, history or over-limit special-needs gates, a partly excluded child splits in proportion to code T, and an over-limit code T is refused; foreign-adoption prior taxable benefits and a foreign child paid to both spouses are refused. |
| 77 | [Form 4972](src/retirement/lump_sum.rs) | Calculation | Participants born before 1936-01-02: capital-gain election, ten-year averaging, allowance, multiple recipients, NUA, the line 9 death benefit exclusion (a participant who died before 1996-08-21) and the line 18 estate tax divided by the box 9a percentage; a trust-only Step 1 (Form 1041) remains outside. |
| 78 | [Form 8396](src/federal/federal_rules.rs) | Calculation | Mortgage-credit-certificate limits, selected reissues and carryovers; the certified loan is chosen on More credits and section 163(g) reduces that loan's interest in both its Schedule A and Schedule E shares; multiple current certificates and special histories remain limited. |
| 79 | [Form 8958](src/federal/federal_rules.rs) | Calculation | Married filing separately in a community-property state: the Setup state and rule, a Community, Separate or Partly answer on every income row, and one factor per item where each kind enters the totals (income, gains and withholding follow it; the withholding on a Form 1099 routed to a business follows the business's share; Social Security and Medicare wages, self-employment tax, IRA compensation and earned-income earnings stay with the earner); a routed amount is listed once, as its destination's row; section 66(a) and 879 spouses; Schedule 1 lines 3 and 6 report the community share, and a community royalty's depletion follows the filer's share and has its own Form 8958 row. Rentals, farm rentals and K-1s are whole as the filer's own or refused by name when shared. Section 66(b)/(c) relief, Alaska, South Dakota and Tennessee elective regimes, registered domestic partners and Form 2555 are not covered. |
| 80 | [Forms 3468 and 4255](src/credits/investment_credit.rs) | Calculation | §47 rehabilitation, §48 energy (construction begun before 2025) and §48E clean electricity on asset and improvement rows and from K-1 property lists; §50(c) basis reduction; recapture from the disposal record to Schedule 2 line 13a; Form 8582-CR Part III. §48A–§48D and cooperative codes excluded by name. |
| 81 | [Schedules K-3 from 1065/1120S](src/passthrough/schedule_k3.rs) | Calculation | Parts I-III feed Form 1116 (line 1a by category and country, 3e, 2, 5, 4b, Part II, line 12, Schedule B, the foreign branch column and capital lines 27-30); a K-1's deductions and taxes follow the share its basis and at-risk limits allow, and a K-1 foreign tax without a K-3 is credited under the conservative assumptions with one note. Parts IV-XIII excluded by name. |
| 82 | [Form 6198](src/passthrough/scorp_risk.rs) | Calculation | Selected detailed-method K-1 at-risk limits and recapture, shared pro rata across every at-risk class (ordinary, rentals, capital, section 1231) with a carry for each; a direct rental's at-risk amounts and carries run before the passive limits; not Schedule C, simplified-method or financing coverage. |
| 83 | [Form 8857](src/federal/federal_rules.rs) | Deferred | Preparing the innocent-spouse relief application is deferred (inventor decision of 2026-10-08, like Form 1310); it changes no computed amount, and a notice lets the ordinary joint-return calculation proceed. |
| 84 | [Schedule J](src/farm/farm_averaging.rs) | Calculation | Selected ordinary farm-income averaging with retained base-year histories; capital-gain, fishing and several combination cases remain limited. |
| 85 | [Form 2439](src/investment/stock_loss.rs) | Calculation | Box 1a on Schedule D line 11, box 1c section 1202 gain with its Form 8949 code Q exclusion, box 2 on Schedule 3 line 13a and the basis adjustment. |
| 86 | [Form 2210-F](src/payments/underpayment.rs) | Calculation | Farmer/fisherman two-thirds test (on gross gains), the single January 15 installment at the smaller of 66⅔% of the current or 100% of the prior-year tax, the March 1 rule and the single-rate penalty period; a waiver request reduces line 16, with the computed penalty kept as its own row; livestock-gain and fishing-crew income markers remain gaps. |
| 87 | [Form 8586](src/credits/general_business_credit.rs) | Calculation | K-1 codes C and D on Form 3800 line 4d (pre-2008 on Part IV line 2b) under a Form 8582-CR passive-credit limit with the §469(i) housing allowance; direct building owners (Form 8609-A) stay refused by name. |
| 88 | [Form 8911](src/federal/federal_rules.rs) | Calculation | Personal-use refueling-property credit within eligibility/date/tax limits; business and mixed-use computation is absent. |
| 89 | [Form 8941](src/credits/general_business_credit.rs) | Calculation | Lines 1–16 with entered county average premiums (the 2026 table is not published until November 2026); one record per owner. |
| 90 | [Form 3115](src/cost_recovery/method_change.rs) | Calculation | The §481(a) adjustment from an entered line 26 (Rev. Proc. 2015-13 periods, de minimis election, acceleration when the business stops) on Schedules C, E and F; preparing the application is deferred (inventor decision of 2026-10-08). |
| 91 | [Form 8881](src/credits/general_business_credit.rs) | Calculation | Startup-cost and auto-enrollment credits; direct contribution-credit and military-spouse parts are not computed, although selected K-1 amounts can transfer. |
| 92 | [Form 8994](src/credits/general_business_credit.rs) | Calculation | §45S as amended by P.L. 119-21: wage and insurance-premium claims per business with employee tests, the 12-week limit, Schedule C reductions and K-1 code BB. |
| 93 | [Form 8834](src/federal/federal_rules.rs) | Calculation | Tax limit on an entered legacy EV passive-credit release; the underlying Form 8582-CR release is not computed. |
| 94 | [Form 8910](src/credits/general_business_credit.rs) | Not applicable | §30B ended for vehicles purchased after 2021; the 2026 Form 3800 line 1r and Schedule 3 line 6e are reserved. Only a Form 3800 line 2x carryforward is entered, under Form 3800. |
| 95 | [Form 8859](src/federal/federal_rules.rs) | Calculation | Prior D.C. homebuyer credit carryforward and tax limit, with combination restrictions. |
| 96 | [Form 8903](src/lib.rs) | Not applicable | Legacy domestic production deduction checklist entry; no current implementation claimed. |
| 97 | [Form 4562-B](src/cost_recovery/amortization.rs) | Calculation | Business amortization on its own form: line 1 for amortization that begins in 2026 (start-up and organizational costs under section 195 and section 197 intangibles, with columns (c), (e) and (f)), line 2 for earlier starts and line 3 the total, carried to Schedule C line 27b; it never calls for Form 4562. The cost records are asked on the Business page's "More about this business" (the farm page asks start-up costs only). See [start-up costs](src/cost_recovery/startup.rs). |

## Additional implemented paths

These are outside the table's 97-row denominator or are embedded in broader rows:

- **Schedule 1-A:** selected qualified tips and overtime on the W-2 route,
  qualified tips of a business or farm (on its "More about this business" and
  "More about this farm" pages, removed from QBI), vehicle-loan interest and
  senior deductions. The self-employed overtime record is not wired to any page.
  Not every off-document pay situation is supported.
  See [additional deduction rules](src/federal/federal_rules.rs) and
  [deduction-page tests](src/tests/deductions/deduction_pages.rs).
- **Income in respect of a decedent:** the part of a 1099-INT, 1099-DIV or 1099-R
  paid after a 2026 death is left to the estate (a statement printing other
  components is calculated as printed with a notice). See
  [after-death income](src/family/after_death.rs).
- **Main-home sale:** selected section 121 exclusion and gain computation.
  See [home sale](src/home/home_sale.rs) and [interview tests](src/tests/home/home_sale_page.rs).
- **Form 172 and AMT NOL history:** selected retained business-loss carryforward
  deductions and absorption, not a general reconstruction of every loss year.
  See [NOL rules](src/federal/nol.rs) and [loss tests](src/tests/federal/losses.rs).
- **Selected payroll/business and other taxes:** Forms 5884
  and 8846 employer credits, Form 8828 mortgage recapture, Form 8912 tax-credit
  bonds, a restricted Form 8978 adjustment and Form 1062 farmland installment
  election. Each has source and combination limits. Form 8919 wages are entered through the
  Form 8919 document on the Wages page. See
  [business credits](src/credits/general_business_credit.rs),
  [mortgage recapture](src/credits/mortgage_recapture.rs), [tax-credit bonds](src/credits/tax_credit_bonds.rs),
  [BBA adjustments](src/passthrough/bba_audit.rs) and [farm land](src/farm/farm_land.rs).
- **State-source records:** printed state/local boxes and allocations can be
  retained alongside federal inputs. This is data capture, not state/local
  return calculation. See [state-source records](src/federal/state_return.rs).
- **Document import and topic search:** source declarations provide import
  mappings and searchable navigation. Tax computation support does not certify
  live AI extraction accuracy, model ranking or complete form intake.
  See [interview declarations](src/federal/interview/mod.rs) and [navigation tests](src/tests/passthrough/income_navigation.rs).

## Corrections to older coverage claims

The current implementation takes precedence over older comments and summaries:

| Older claim | Current evidence |
| --- | --- |
| No Form 2210 penalty or annualized computation | [Underpayment rules](src/payments/underpayment.rs) implement both, with restrictions and provisional rates. |
| Mortgage debt limits are only a confirmation | [Mortgage rules](src/home/mortgage_limit.rs) calculate the qualified-loan/average-balance worksheet and selected points treatment. |
| Form 8881 is missing | [Business-credit rules](src/credits/general_business_credit.rs) calculate selected parts; the rest remains limited. |
| An unsupported item always prevents a result | [Working-copy rules](src/core/unknowns.rs) can omit the item with a notice and keep an estimate available. |
| Stored computations counted as implemented | A reachability audit (an October 8 baseline) found 46 stored union variants and 6 stored fields that no page constructs; they were not counted as implementation. Later federal parity work has since given the retirement, investment, amortization and fuel kinds pages, and the rows above name the page choice or document kind that writes each. |

Older coverage percentages based on assumed return frequencies and overlap
cannot be carried forward as a measured result. A defensible current percentage
would require a defined representative return population and checking each
complete return, including omitted items and unsupported combinations. Neither
source size nor passing test count supplies that denominator.

## Verification and remaining limits

This is an implementation-inventory audit: owning source, calculation wiring,
interview declarations, refusals/omissions and existing regression evidence were
reviewed. It is not an independent reconciliation of every IRS line or an
exhaustive live-client acceptance run. In particular, calculation presence is
counted separately from whether every relevant fact pattern can be entered and
finished through the interview.

Checks run against the October 9, 2026 source snapshot (Tax commit `858ed5f`, after the defect-fix batch and parity batch B1); only the test count and the display-warning count were updated afterward, to the figures at `591725f`:

| Check | Result |
| --- | --- |
| `cargo test -q` | **4,883 passed** at `591725f` (the Form 8379, investment, farm and Schedule 2 units, B11 4797 and IRA, F17 and the presentation density pass merged since `e8e56c5`) (4,242 passed, 0 failed, 1 ignored at `858ed5f`); the other rows below were not rerun. No doc-tests were present. |
| `cargo check -q` | Passed. Studio shared validation at `591725f`: `OK: 1354 types, 1 functions, 1 display warnings` (the accepted editable `Mortgage1098` property label; the presentation density pass removed the screen-density warnings). |
| `cargo clippy --all-targets -- -D warnings` | Passed. |
| `cargo fmt -p libertas-tax -- --check` | Passed for the Tax App. |
| `cargo fmt --all -- --check` | Passed (the five macro test files were formatted in libertas-rs `76dcd77`). |

The ignored test is `profile_full_cache_memory` in
[storage tests](src/tests/core/storage.rs), a manual 10,000-user memory/load profile.
It was not counted as passed. No instrumented line or branch coverage percentage
was collected. Passing tests establish the assertions exercised, not complete
tax-law correctness.

The [full-return fixture loader](src/tests/cross/returns.rs) runs six JSON fixtures
with separately recorded expected lines and computation explanations:
single W-2; head of household with two children; two W-2 employers with excess
Social Security/Additional Medicare; interest/dividend/OID income; joint
Schedule B; and a joint three-child credit phaseout. Many other cases live in
Rust tests, including hand-computed checks. The six reusable fixtures are not
a representative consumer-return population or an independent tax-software
comparison across all 97 entries.

The [stored EIC fixture](tests/fixtures/irs-eic-2026-draft.csv) and
[tax-table fixture](tests/fixtures/irs-tax-table-2026.csv) have regression checks
in [table tests](src/tests/health/health_accounts.rs). Those checks compare against the
stored fixtures, not every subsequent IRS publication. The
[2026 Publication 1040](https://www.irs.gov/pub/irs-dft/p1040--dft.pdf) and
[2026 Form 1040](https://www.irs.gov/pub/irs-dft/f1040--dft.pdf) retrieved during
this audit are drafts. Final-form and final-instruction reconciliation remains
a release prerequisite.

Two future-period inputs are explicitly provisional: the
[2027 estimated-tax worksheet](src/forms/form_1040_es.rs) uses 2026 worksheet
parameters, and the [penalty-rate fixture](tests/fixtures/irs-underpayment-rates-2026.csv)
leaves 2027 Q1 blank. [Underpayment rules](src/payments/underpayment.rs) substitute the last
published rate for a blank period. This is an estimated calculation, not a
verified final future-period rate.

Production readiness also requires current live-client acceptance, real Hub
restart/persistence validation, broader independent complete-return comparisons,
and resolution of the named gaps. This audit ran local tests; it did not deploy
or re-test a live Hub, validate real taxpayer returns, or publish a filing-ready
product. Printing, electronic submission, amendments, filing administration,
state/local returns and nonresident Form 1040-NR remain outside the implemented
product scope.
