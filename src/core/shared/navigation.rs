use alloc::string::String;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Expected tax year for document import
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
pub struct DocumentFacts {
    /// Tax year
    #[libertas_fixed(2026)]
    pub year: u16,
}

/// Tax return section
// The page identity used by `page`, `previous`, `Navigate`, the hubs' topic
// rows and the review rows, and stored in `FederalDraft.finished_sections`.
// Append new identities to preserve every saved identity. Navigation groups
// constrain these same identities; renumbering them is a storage format change.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport,
)]
pub enum TaxSection {
    /// Your 2026 return
    /// Married filing separately spouse age and blindness boxes on Form 1040 line 12d, for a spouse with no income who does not file and is not another person's dependent. Filing status: single, married filing jointly (also for the year your spouse died, if you did not remarry in 2026), married filing separately, head of household (including married to a nonresident alien) or qualifying surviving spouse. Married filing separately in a community-property state (Form 8958, Pub. 555; Arizona, California, Idaho, Louisiana, Nevada, New Mexico, Texas, Washington, Wisconsin), spouses living apart all year (section 66) and a nonresident alien spouse (sections 2(b)(2)(B), 879). Rental, partnership or S corporation income divided between spouses.
    #[libertas_protocol_variant("SaveSetup")]
    Setup,
    /// About you
    /// Your name, date of birth, legal blindness, Social Security number or ITIN, whether someone else can claim you as a dependent, and a death in 2026: the decedent's final return, date of death, deceased taxpayer, estate-paid medical expenses (section 213(c) election) and savings bonds held at death. Form 1310 is not prepared. A dual-status, part-year resident, nonresident alien or U.S. territory return (Form 1040-NR, Forms 4563 and 8689) and community property with a registered domestic partner (Form 8958).
    #[libertas_protocol_variant("SavePeople")]
    People,
    /// About your spouse
    /// Your spouse's name, date of birth, legal blindness, Social Security number or ITIN, whether someone else can claim your spouse as a dependent, and a death in 2026 (the decedent's final return, date of death, estate-paid medical expenses and savings bonds held at death), for a joint return. The choice to treat a nonresident spouse as a resident (section 6013(g) or (h)) and its net investment income tax election, injured spouse allocation (Form 8379) and innocent spouse relief (Form 8857).
    #[libertas_protocol_variant("SaveSpouse")]
    Spouse,
    /// Children and people you support
    /// Dependents, child tax credit, credit for other dependents and earned income credit, including a custodial parent's release of a child to the other parent (Form 8332, divorced or separated parents, noncustodial parent) and a multiple support agreement (Form 2120, multiple support declaration). Someone else who could also claim a child (qualifying child of more than one person, the section 152(c)(4) tiebreaker rules) and a filer who can be claimed as a dependent (no dependents of their own); a child who died in 2026 (date of death, Form 8862 line 8); a partner's or friend's child who is someone else's qualifying child (no credit for other dependents); and the child's other parent who could also claim the child (longer residence, higher AGI).
    #[libertas_protocol_variant("SaveChildren")]
    Children,
    /// Beyond your documents
    /// What no document shows: a business or side work of your own, gig and app work, rental property, equipment rented out, royalties without a 1099-MISC, a farm or crop-share rent, a household employee, income, a job or accounts outside the U.S.; estimated tax payments, an extension payment or last year's refund applied, state or local taxes you paid yourself, gifts to charity, IRA contributions, HSA contributions outside payroll, child or dependent care; and what your 2025 return carried into 2026: a capital loss carryover, a net operating loss, a QBI loss carryforward, a charitable contribution carryover, a minimum tax credit (Form 8801) and IRA basis (Form 8606).
    #[libertas_protocol_variant("SaveChecklist")]
    Checklist,
    /// Income
    /// Overview of income topics and entered income documents.
    #[libertas_protocol_variant("Hub")]
    Income,
    /// Wages
    /// W-2 wages, withholding, statutory employee Schedule C pay, railroad retirement (RRTA) compensation and tier 1 and tier 2 tax (Form 8959 Part II; Form CT-2 not calculated), allocated/unreported and noncash tips (Form 4137), qualified tips deduction, nontaxable combat pay (code Q), Medicaid waiver payments (code II, Notice 2014-7, Schedule 1 line 8s), and Trump account employer contributions (code TA). Employee pay a firm reported on Form 1099-NEC or 1099-MISC although you were its employee: Form 8919, uncollected Social Security and Medicare tax (Schedule 2 line 16b), reason codes A, C, G, H, Form SS-8, misclassified worker. Form 4852, substitute W-2 when the employer gave none. Box 1 pay for work abroad (Form 2555) is marked on its W-2; the exclusion itself is on Work abroad. Box 1 pension from a nonqualified deferred compensation or nongovernmental 457 plan (box 11, Schedule 1 line 8t) and pay earned while incarcerated (line 8u) are marked on the W-2.
    /// Minister, clergy, pastor, rabbi, priest, imam, religious order, Christian Science practitioner, housing or parsonage allowance, Form4361, Form4029, Amish or Mennonite exemption. A ministerial W-2 and the per-person clergy record are entered here.
    #[libertas_protocol_variant("SaveWagesAnswers")]
    Wages,
    /// Interest and dividends
    /// Forms 1099-INT, 1099-DIV and 1099-OID, interest or dividends paid after the owner's death (income in respect of a decedent), Schedule B, REIT dividends and investment income. Series EE and I U.S. savings bond interest excluded for college tuition (Form 8815, Form 8818 records). Foreign financial accounts and foreign trusts (Schedule B Part III). A child's interest and dividends reported on your return (Form 8814), the child's Alaska Permanent Fund dividend, Form 1040 line 3c and line 16 box 1 (the parents' election, kiddie tax election). Nondividend distributions and return of capital (Form 1099-DIV box 3), liquidating distributions (boxes 9 and 10) and undistributed long-term capital gains (Form 2439, the fund's tax paid credit).
    #[libertas_protocol_variant("SaveIncomes")]
    InterestDividends,
    /// Investment sales
    /// Form 1099-B, Form 8949 and Schedule D capital gains, stock sales and capital loss carryovers. Sale of your main home when you sell your house (1099-S). Incentive stock options (ISO): Form 3921 exercises and ISO shares sold. Installment sales of investment or personal-use property in 2026 (Form 6252, Schedule D lines 4 and 11), and payments from an earlier-year installment sale, a like-kind exchange with a related person from 2024 or 2025 whose property was sold in 2026 (Form 8824 Part II, section 1031(f) two-year rule) and 2021–2025 business-property losses (Form 4797 line 7). Futures, foreign currency contracts and section 1256 options such as broad-based index options (Form 1099-B boxes 8–11, Form 6781 Part I, 60/40 treatment), and the net section 1256 contracts loss carryback election (Form 6781 box D). Employee stock: shares from nonqualified stock options, restricted stock and RSUs (Form W-2 box 12 code V), employee stock purchase plan (ESPP) shares (Form 3922, qualifying and disqualifying dispositions) and employer stock from a retirement plan with net unrealized appreciation (NUA, Form 1099-R box 6). Qualified small business stock (QSBS, section 1202, Form 8949 code Q). Small business stock losses (section 1244, Form 4797). Worthless securities and nonbusiness bad debts (personal loans that were never repaid). Stock and equity options you bought or wrote (calls and puts: sold, bought back, expired or exercised), short sales, wash sales (Form 1099-B box 1g), market discount bonds (box 1f, Form 8949 code D) and sales you received as a nominee (code N). Digital assets: Form 1099-DA, cryptocurrency, bitcoin, stablecoins and NFTs sold or exchanged, with or without a form (Form 8949 boxes G–L). Qualified opportunity fund (QOF, opportunity zone) investments: the deferred gain taxed in 2026 and a 2026 gain invested in a fund in 2027 (Form 8997, Form 8949 codes Y and Z).
    #[libertas_protocol_variant("SaveIncomes")]
    InvestmentSales,
    /// Retirement and Social Security
    /// Forms 1099-R, RRB-1099-R, SSA-1099 and 8606; payments made after the owner's death (income in respect of a decedent); pensions, IRA basis and IRA or Roth distributions, Social Security, rollovers, required minimum distributions (RMD) and qualified charitable distributions (QCD), also from an inherited IRA. Qualified disaster distributions and repayments (Form 8915-F), including earlier Forms 8915-F for 2024 or 2025 distributions, a missed required minimum distribution (Form 5329 Part IX), Social Security lump sums for earlier years and Roth IRA history. Roth IRA first-home distributions and the other IRA exceptions on Roth withdrawals, Roth-to-Roth rollovers, Roth SIMPLE and Roth SEP IRAs (codes J and S, employer Roth contributions reported as converted), Roth repayments of birth or adoption, emergency, domestic-abuse and terminal-illness distributions, Roth disaster distributions, Form 8606 line 2 adjustments (divorce, worksheet carryovers), reservist repayments, the date-of-death IRA value on a final return, QCDs from an IRA with basis, a SIMPLE IRA rolled into another SIMPLE IRA in its first two years, two Form 5329 exceptions on one distribution, box 9b. Lump-sum distributions from plans of participants born before 1936: the 10-year tax option and 20% capital gain election (Form 4972, Form 1099-R code A). Disability pensions before minimum retirement age (Form 1099-R code 3, Form 1040 line 1h). Form RRB-1099-R railroad retirement annuities (tier 2, vested dual benefits, supplemental annuities). Taxable amounts the payer could not figure (Form 1099-R box 2b): the General Rule for commercial and older pension annuities, variable annuities (Publication 939), surrenders, withdrawals, loans and pledges before annuity payments, contracts bought before August 14, 1982, gifts of annuities, CSRS and FERS death lump sums and child survivor annuities (Publication 721). Section 1035 annuity exchanges (code 6), employer stock (NUA, box 6) and distributed property sold and rolled over. Inherited IRAs and inherited Roth IRAs (beneficiary distributions, Form 1099-R code 4, a separate Form 8606 for each person you inherited from, estate tax on income in respect of a decedent), qualified charitable distributions from an inherited IRA, and Trump accounts on Form 1099-R box 7c: a distribution after the child's death and the transfer to the child's ABLE account. Designated Roth account (Roth 401(k), 403(b), 457(b)) distributions, in-plan Roth rollovers (IRR, box 10) and direct rollovers to a Roth IRA (codes B, G, H); plan loans treated as distributions and loan offsets (codes L, M); corrective distributions of excess deferrals and contributions (codes 8, E, P); recharacterizations (codes N, R); prohibited transactions (code 5); the cost of life insurance in a plan (code 9), ESOP dividends (code U), charitable gift annuities (code F) and long-term care charges (code W); QCDs reported with code Y and the one-time QCD to a split-interest entity (charitable remainder trust or gift annuity); substantially equal periodic payments (72(t), SEPP); repayments of birth or adoption, emergency, domestic-abuse and terminal-illness distributions; Form 8915-F distributions to buy a main home in a disaster area; the Form 4972 death benefit exclusion.
    #[libertas_protocol_variant("SaveIncomes")]
    Retirement,
    /// Self-employment
    /// Form 1099-PATR cooperative statements for a Schedule C business: actual source allocations, qualified payments, SSTB items and issuer-allocated credits. Farm and farm-rental recipients use Farming.
    /// Schedule C contractor, freelance and sole-proprietor income, including Forms 1099-NEC (box 1a nonemployee compensation, box 4 backup withholding) and 1099-K (payment cards and payment apps such as PayPal, Venmo, Etsy, eBay, Uber or Airbnb; box 1a gross before refunds and fees), each entered once and sent to its business, rental or hobby; 1099-K amounts included in error or for personal items sold at a loss (top of Schedule 1, not taxed). Business expenses, home office deduction on Form 8829, car and truck expenses, standard mileage, actual vehicle expenses and depreciation on Form 4562. Business loss carryforwards and qualified business income or QBI deduction inputs. The employer credits for this business's employees: work opportunity credit (Form 5884), small employer retirement plan start-up and auto-enrollment credits (Form 8881), employer Social Security and Medicare tax on tips (Form 8846), and small employer health insurance premiums credit for SHOP Marketplace coverage, small business health care tax credit (Form 8941, FTE, full-time equivalent employees), and employer credit for paid family and medical leave, wages paid during leave or paid leave insurance premiums (Form 8994, FMLA credit, section 45S), each with its reduction of the related expense. The general QBI entry for Section 199A (199a), Form 8995 (8995) or Form 8995-A (8995-A); rental and K-1 questions also have their own supporting pages. Accounting-method changes (Form 3115): the section 481(a) adjustment on Schedule C line 6 or Part V, spread over four years or taken in one year (de minimis election), including catch-up depreciation (DCN 7), cash-to-accrual and small-business cash or inventory changes. Investment credit (Form 3468) for a certified historic building's rehabilitation or for solar, wind, energy storage or geothermal heat pump property of the business (historic tax credit, rehab credit, solar investment tax credit, solar ITC, section 48 and section 48E energy credit, clean electricity investment credit), with its section 50(c) basis reduction and the recapture (Form 4255, investment credit recapture) when the property is sold within five years. Qualified U.S. timber reforestation costs and original section 194 expense/amortization elections (Form 4562-B), including actual continuing cost-year history. Research expenditure deductions and original section 174/174A elections, certified pollution-control facilities (section 169), lease-acquisition costs (section 178), preference write-offs (section 59(e)) and qualifying unproduced creative-property costs (Form 4562-B). Amortization of goodwill and other section 197 intangibles bought with a business (Form 4562-B), and the federal fuel tax credit (Form 4136) with an earlier year's credit or refund received. Gambling as a trade or business (professional gambler, wagering losses limited to 90% and to winnings, W-2G). A business run abroad (services performed wholly in a foreign country, Form 2555) is marked on its own page; the exclusion itself is on Work abroad. An asset sold in the year you bought it, scrapped, condemned (section 1033), sold to a relative or to your own company (sections 267 and 1239), a vehicle or section 179 property whose business use fell to 50% or less (Form 4797 Part IV recapture), part of a building replaced (partial disposition), an installment sale of equipment, and the sale of a whole business (Form 8594).
    /// De minimis safe-harbor election, employee pension and profit-sharing plans, mixed 50%/DOT 80%/fully deductible meals, leased-vehicle actual expenses and inclusion amounts, inventory valuation and section 471(c), returns and allowances, other business income, section 481(a) regular and AMT amounts, working-interest depletion and self-employed overtime.
    /// Nonfarm optional method, notary public fees and totalization agreement certificate of coverage; the farm optional method stays on Farming.
    #[libertas_protocol_variant("SaveIncomes")]
    SelfEmployment,
    /// Rental property
    /// Direct rental-owner section 179D (179d) energy-efficient commercial building property, Form 7205 (7205): actual physical-building area, certification, printed energy savings, qualifying cost and all-party deduction history. The supported 2026 direct EECBP source uses its documented building allowance and separate depreciation basis reduction; designer allocations belong to Self-employment. Retrofit elections, unresolved historical basis and other years retain named source boundaries.
    /// Vacation or short-term rental (Airbnb, VRBO), average stay, significant or substantial services; land rental or ground lease; self-rental to your own business; appliances, furniture and improvements; a building converted from your home, inherited or received as a gift; Form 6198 at-risk limits; the professional's single-activity election and Schedule E line 43. Royalties from oil, gas, coal and other mineral leases, copyrights and patents (Schedule E line 4), expenses, cost and percentage depletion, advance royalties and lease bonuses, the 65% limit carryover and the AMT adjustment (Form 6251 line 2d).
    /// Schedule E rental income, property expenses and rental QBI inputs for Section 199A and Forms 8995/8995-A. Real estate professional status. A rental sold in 2026 (Form 4797) or exchanged for other real property in a like-kind exchange (Form 8824, section 1031, 1031 exchange, deferred exchange through a qualified intermediary, boot, replacement property, related-party exchange). Accounting-method changes (Form 3115): the section 481(a) adjustment for missed or excess rental depreciation (DCN 7 or 107) on Schedule E line 3 or line 19. The rehabilitation credit for a certified historic rental building (Form 3468 Part VII, historic tax credit, HTC) on an improvement row, with the passive credit special allowance (Form 8582-CR) and the recapture (Form 4255) when the property is sold within five years. A rental sold after personal use, held one year or less, condemned, sold to a relative, or sold by installment with suspended losses (section 469(g)(3)), and a replaced roof or other partial disposition.
    #[libertas_protocol_variant("SaveIncomes")]
    Rentals,
    /// Unemployment and state refunds
    /// Form 1099-G unemployment compensation, taxable state income-tax refunds, paid family leave and other supported government-payment boxes, including RTAA payments (box 5, Schedule 1 line 8z), taxable grants (box 6) to you or your business, and a refund of a business income tax (box 8).
    #[libertas_protocol_variant("SaveIncomes")]
    GovernmentBenefits,
    /// Prizes, scholarships, relief payments, account payouts and other income
    /// Household employee pay without a W-2 (Form 1040 line 1b), strike or lockout benefits (line 1h), and Medicaid waiver payments without a W-2 (line 1d, Notice 2014-7).
    /// Form 1099-MISC: rents, royalties, other income, prizes and awards, medical and health care payments, fishing boat proceeds, substitute payments in lieu of dividends, gross proceeds paid to an attorney, fish sold for cash. Other income (Schedule 1 line 8): Alaska Permanent Fund dividend (8g), Olympic and Paralympic medals and USOC prize money (8m, 24c), wages earned while incarcerated not on a W-2 (8u), digital assets received as income such as staking or mining rewards and airdrops (8v), refunds or reimbursements of an earlier year's itemized deductions (tax benefit rule, recoveries, Form 1098 box 4 mortgage interest refund, medical reimbursement), credit card insurance benefits, private or union unemployment fund benefits, and other taxable income on line 8z such as rewards, found property, taxable damages, executor fees, medical research study pay. Equipment you rent out. Alimony received. Scholarships and fellowship grants not on a W-2 (Schedule 1 line 8r, Pell grants, room and board, stipends, Form 1098-T box 5); jury duty pay and pay turned over to an employer (lines 8h and 24a); hobby income, activity not engaged in for profit (line 8j, section 183, 1099-K sales of crafts or collectibles made as a hobby); qualified wildfire relief payments (section 139M), qualified disaster relief payments (section 139, FEMA individual assistance), insurance for additional living expenses after a casualty (section 123, loss of use, temporary housing), and hazard mitigation grants (section 139(g), FEMA HMGP, flood mitigation). East Palestine train derailment payments are not taxable and need not be entered. Form 1099-QA distributions from ABLE accounts (section 529A), the 10% additional tax (Form 5329 Part II), and contributions to an ABLE account above the limit (Form 5329 Part VIII).
    #[libertas_protocol_variant("SaveIncomes")]
    OtherIncome,
    /// Gambling winnings
    /// Form W-2G gambling winnings and the related gambling-loss facts.
    #[libertas_protocol_variant("SaveIncomes")]
    Gambling,
    /// Farming
    /// Section 126 conservation cost-sharing improvements, supported income-right appraisals and actual excluded cost-share funding applied to the selected improvement.
    /// Schedule F farm income and expenses, including farm car and truck expenses, mileage, actual vehicle expenses and depreciation on Form 4562. Agricultural cooperative income on Form 1099-PATR, with actual personal-purchase and property source parts, nonqualified notice history, qualified-payment wage allocation, SSTB items, DPAD and issuer-allocated credits; Schedule C cooperative recipients use Self-employment, farm income averaging on Schedule J, the 2026 farm loss carryback choice and net operating loss carryforwards. Farm rental income and expenses on Form 4835 (4835) for a landowner paid in crop-share or livestock-share rent (landlord share, farmland rented to a tenant on shares) who did not materially participate (Schedule E line 40). Accounting-method changes (Form 3115): the section 481(a) adjustment on Schedule F other income or other expenses. Investment credit (Form 3468) for farm solar, wind, energy storage or geothermal heat pump property or a certified historic building, with its basis reduction and the recapture (Form 4255) when the property is sold within five years. The federal fuel tax credit for off-highway farm fuel (Form 4136).
    #[libertas_protocol_variant("SaveIncomes")]
    Farming,
    /// Partnerships, S corporations, estates and trusts
    /// Partnership business property (box 10 and the box 9c business part), its section 1231 loss carryovers and qualified section 199A share use this topic. Direct assets use the business topic; rental property dispositions need their own rental calculation.
    /// K-1 royalties (Form 1065 box 7, Form 1120-S box 6) and code I deductions on Schedule E lines 4 and 19, after basis and at-risk limits and outside Form 8582.
    /// Schedules K-1 from partnerships, S corporations, estates and trusts, including box 2 net rental real estate and box 3 other rental income or loss under the passive activity loss limits (Form 8582, the $25,000 special allowance); pass-through QBI information for Section 199A and Forms 8995/8995-A. Schedule K-3 (Forms 1065 and 1120-S) Parts II and III for the foreign tax credit, including foreign branch income (column (b)) and capital gains (lines 11, 12, 27, 28), K-1 box 16 and box 21 (Form 1120-S box 14 and box 16 code F) foreign taxes. The low-income housing credit (Form 8586) from K-1 box 15 or box 13 codes C and D, with its passive activity credit limit on Form 8582-CR. General business credits from partnership K-1 box 15 and S corporation K-1 box 13 include code M research credit (R&D, Form 6765). Credit codes and document-sourced statement lines come from the K-1; code M eligibility is answered separately from your own records. K-1 investment credit property (Form 3468): qualified rehabilitation expenditures (box 15 code E and box 20 code D; Form 1120-S box 13 code E and box 17 code C) and the basis of energy or clean electricity property (box 20 codes E and AV; Form 1120-S box 17 codes D and AV), with the section 50(c)(5) reduction of your basis. Basis and at-risk loss carryovers by kind (Form 7203 Part III, the partner basis worksheet, Form 6198): rental, short- and long-term capital, section 179, cash contribution and foreign tax losses held back by your basis or amount at risk. Other income on Schedules K-1 (Form 1065 box 11, Form 1120-S box 10: other portfolio income, section 1256 contracts, canceled debt, gambling, non-portfolio capital gains) and other deductions (Form 1065 box 13, Form 1120-S box 12: investment interest, charitable contributions, early withdrawal penalty); section 199A dividends and PTP items on the K-1's section 199A statement; a final K-1, the sale of a partnership interest or S corporation shares with the section 751 ordinary part (box 20 codes AB, AC, AD), a liquidation for cash and distributions above basis.
    #[libertas_protocol_variant("SaveIncomes")]
    PassThrough,
    /// Canceled debt
    /// Forms 1099-C and 1099-A: canceled debt, foreclosure or repossession, and the Form 982 exclusions such as insolvency and bankruptcy. Student loan repayment by the National Health Service Corps Loan Repayment Program or a state health-professional loan repayment program (section 108(f)(4), NHSC LRP).
    #[libertas_protocol_variant("SaveIncomes")]
    CanceledDebt,
    /// Work abroad
    /// Form 2555 foreign earned income exclusion (up to $132,900 for 2026), foreign housing exclusion and housing deduction for a U.S. citizen or resident alien living and working abroad (expat) with a foreign tax home: bona fide residence test, physical presence test (330 full days in 12 months), qualifying days, housing expenses with the high-cost locality limits of Notice 2026-25, and the housing deduction carryover; pay from a foreign employer without a W-2. W-2 pay for work abroad is marked on its W-2 under Wages, and a business run abroad on Self-employment. The foreign tax credit (Form 1116) is on More credits; tax treaty positions (Form 8833) are under Situations people often miss.
    /// Revoked exclusion, IRS consent to re-elect, bonus or RSU for work abroad and pay for prior-year services.
    #[libertas_protocol_variant("SaveIncomes")]
    WorkAbroad,
    /// Adjustments to income
    /// Overview of adjustments to income on Schedule 1.
    #[libertas_protocol_variant("Hub")]
    Adjustments,
    /// Health and medical savings accounts
    /// Form 8889 contributions to your health savings account, excess contributions and their corrections, qualified HSA funding distributions (QHFD), lifetime IRA-to-HSA funding uses, the last-month rule and prior testing histories. Form 1099-SA and trustee statements from your own or inherited HSA, Archer MSA or Medicare Advantage MSA: ordinary distributions, rollovers and account events (Form 8889 Part II or Form 8853 Sections A and B). Archer MSA contributions (Form 8853 Part I): personal and employer deposits, the shared family limit, corrections and excess.
    #[libertas_protocol_variant("SaveHsa")]
    Hsa,
    /// Self-employed health insurance and retirement plan
    /// Form 7206 self-employed health insurance, SEP, SIMPLE and other retirement-plan contributions for existing businesses or partnerships.
    #[libertas_protocol_variant("SaveSelfEmployedBenefits")]
    SelfEmployedBenefits,
    /// IRA contributions
    /// Traditional and Roth IRA contributions, nondeductible contributions reported on Form 8606, and excess contribution tax facts for Form 5329: contributions returned in 2027 with their earnings, a carried excess absorbed into this year's limit (Form 5329 lines 10 and 19), a pre-2026 excess paid back in 2026 (line 12) and a 529-to-Roth rollover beside an excess.
    #[libertas_protocol_variant("SaveIraContributions")]
    IraContributions,
    /// Educator expenses
    /// Early childhood through grade 12 teachers, instructors, counselors, principals and aides (Schedule 1 line 11, up to $350 each), and interscholastic sports administrators and coaches. Excess classroom expenses, nonathletic health and physical education supplies, and materials used in instruction outside the classroom are Schedule A line 17k.
    #[libertas_protocol_variant("SaveEducator")]
    Educator,
    /// Student loan interest
    /// Form 1098-E and student loan interest deduction facts.
    #[libertas_protocol_variant("SaveStudentLoan")]
    StudentLoan,
    /// Reservist, performing artist and fee-basis official expenses
    /// Form 2106 employee business expenses: Armed Forces reserve or National Guard drill travel more than 100 miles from home, per diem, lodging; qualified performing artist, actor, musician, dancer with two or more employers; fee-basis state or local government official; employee vehicle standard mileage or actual expenses and depreciation, and a Form 2106 vehicle sold or kept for personal use; W-2 box 12 code L reimbursements. Impairment-related work expenses are on Investment interest and other deductions.
    #[libertas_protocol_variant("SaveEmployeeExpenses")]
    EmployeeExpenses,
    /// More adjustments
    /// Alimony paid under a divorce or separation agreement signed before 2019. Moving expenses of Armed Forces members and intelligence community employees (Form 3903, Schedule 1 line 14): a permanent change of station (PCS) under military orders or an intelligence reassignment, moving household goods, travel and mileage to the new home, storage, and government moving reimbursements shown in W-2 box 12 code P. Also named as not calculated yet: Archer MSA contributions (Form 8853), and other Schedule 1 adjustments.
    #[libertas_protocol_variant("SaveMoreAdjustments")]
    MoreAdjustments,
    /// Deductions
    /// Standard deduction or itemizing on Schedule A. Cash gifts to charity, personal casualty or theft losses on Form 4684, and repayments over $3,000 of income reported in an earlier year (claim of right). Personal car loan interest: Schedule 1-A qualified passenger vehicle loan interest reported on Form 1098-VLI (1098-VLI), with the vehicle's original use and final assembly in the United States. Business vehicle expenses and donated cars have separate pages.
    #[libertas_protocol_variant("SaveDeductions")]
    Deductions,
    /// Itemized deductions
    /// Overview of Schedule A itemized deductions.
    #[libertas_protocol_variant("Hub")]
    Itemized,
    /// Medical and dental expenses
    /// Schedule A medical and dental expenses, long-term-care insurance premiums and health-related itemized deductions.
    #[libertas_protocol_variant("SaveMedical")]
    Medical,
    /// Taxes you paid
    /// State and local income or sales taxes (Schedule A line 5a), real estate taxes (line 5b) and annual value-based personal property taxes (line 5c).
    #[libertas_protocol_variant("SaveTaxes")]
    Taxes,
    /// Home mortgage interest
    /// Form 1098 and Schedule A lines 8 through 8e: mortgage interest, purchase and refinance points (box 6), mortgage insurance (box 5), refunds (box 4), home equity loans and lines of credit (HELOC), cash-out and mixed-use loans (Pub. 936 Table 1), refinanced loans and their earlier dates, a second home including a boat or RV, a third home, seller financing, loans with no Form 1098, and a loan sold to a new lender (box 11). The $750,000 and $1,000,000 limits.
    #[libertas_protocol_variant("SaveMortgages")]
    Mortgage,
    /// Gifts of property and carryovers
    /// Form 8283 noncash charitable property gifts and charitable-contribution carryovers: clothing and household goods, publicly traded stock, a student you hosted and a whaling captain's expenses. Includes donating a personally owned car, truck or other supported road vehicle to charity, with Form 1098-C (1098-C) acknowledgment and sale proceeds or certified use. Out-of-pocket volunteer costs, including charitable car mileage, gas, parking and tolls. Supported gifts and explicit limits are shown on the page.
    #[libertas_protocol_variant("SaveGiftCarryovers")]
    GiftCarryovers,
    /// Investment interest and other deductions
    /// Investment interest (Schedule A line 9, Form 4952), investment land taxes, unrecovered annuity cost at death (17i), generation-skipping transfer tax on a trust income distribution (line 6), amortizable bond premium (17f), a contingent payment or inflation-indexed debt instrument loss such as TIPS (17g), and impairment-related work expenses of a disabled employee (17j).
    #[libertas_protocol_variant("SaveOtherItemized")]
    OtherItemized,
    /// Credits
    /// Overview of federal tax credits.
    #[libertas_protocol_variant("Hub")]
    Credits,
    /// Education credits
    /// Tuition and fees on Form 1098-T, Form 8863, American opportunity tax credit (AOTC) and lifetime learning credit (LLC) for education expenses, including a Pell grant or scholarship the student includes in income to raise the credit (Pub. 970 coordination, room and board, Schedule 1 line 8r).
    /// Form 1099-Q distributions from 529 plans (qualified tuition programs, QTP) and Coverdell education savings accounts (ESA): taxable earnings, qualified education expenses paid from them, including K-12 tuition, room and board, student-loan payments and apprenticeships, the 10% additional tax (Form 5329 Part II) and 529-to-Roth IRA rollovers.
    #[libertas_protocol_variant("SaveEducation")]
    Education,
    /// Retirement savings credit
    /// Form 8880 retirement savings contributions credit, also called the saver’s credit.
    #[libertas_protocol_variant("SaveSaver")]
    Saver,
    /// Health insurance marketplace
    /// Form 1095-A and Form 8962 health insurance marketplace premium tax credit and reconciliation.
    #[libertas_protocol_variant("SaveMarketplace")]
    Marketplace,
    /// More credits
    /// Choosing not to claim the earned income credit (Form 1040 line 27c). Foreign income tax deduction instead of a credit (Schedule A line 6), or foreign tax credit on Form 1116 by category (general, passive, foreign branch, sanctioned-country section 901(j) income) and country, accrued or paid foreign taxes, the high-tax kickout, foreign capital gains (Worksheets A and B, Worksheet for Line 18), a business run abroad, Schedule B (Form 1116) carryover reconciliation (Schedule K-3 amounts are read from the K-1 page), adoption on Form 8839, mortgage interest credit on Form 8396, business credit carryovers on Form 3800, including alternative motor vehicle credit (Form 8910) carryforwards and low-income housing credit (Form 8586) carryforwards and unallowed passive credits (Form 8582-CR), prior minimum tax credit on Form 8801 (exclusion-only AMT, MTCNOL carryovers/carrybacks, MTFTCE, retained foreign-earned-income worksheets, prior joint or separate returns, old qualified electric vehicle credit), residential energy credit carryovers on Form 5695 (solar panels, home batteries or geothermal heat pumps from earlier years; the credits for new home energy property ended after 2025), and credit for the elderly or disabled on Schedule R. Clean vehicle, fuel cell vehicle or electric car (EV) credits and dealer-transfer reconciliation on Form 8936, including a new vehicle used in a saved business or farm and its Form 3800 line 1y business part, alternative fuel vehicle refueling or charging property on Form 8911, and electric vehicle credit carryovers on Form 8834. The D.C. first-time homebuyer credit carryover (Form 8859), federal tax on home kerosene (Form 4136), qualified tax-credit bonds (Form 8912), the employer tip FICA credit carryover (Form 8846), a BBA audit adjustment (Form 8978), 2025 education credit recapture and credits after an earlier disallowance (Form 8862, where a 2-year or 10-year ban leaves the credit out). General business credits figured on their own source forms can be entered by Form 3800 line: research/R&D Form 6765, disabled access Form 8826, employer childcare Form 8882, differential wage Form 8932, orphan drug Form 8820, new markets Form 8874, renewable electricity Form 8835, advanced manufacturing production Form 7207, clean hydrogen Form 7210 and carbon oxide sequestration Form 8933. Includes eligible elective payment (direct pay), credits bought under section 6418, transfers, empowerment-zone Form 8844 carryforwards and passive credit limitations. This section covers credits and the foreign income tax deduction; personal car loan interest is on Deductions. Rehabilitation (Form 3468 Part VII), energy (Part VI) and clean electricity investment (Part V) credit carryovers and their carrybacks to 2023 through 2025 are business credit carryovers.
    #[libertas_protocol_variant("SaveMoreCredits")]
    MoreCredits,
    /// Other taxes
    /// Household employment taxes for a nanny or housekeeper on Schedule H, Trump account excess contributions (Form 5329 Parts X and XI), and recapture of a subsidized home mortgage on Form 8828. Also named as not calculated yet: other alternative minimum tax items (Form 6251), excess contributions to a Coverdell ESA or ABLE account (Form 5329), credit recapture and look-back interest (Forms 8611, 8697, 8866), a deferred section 965 tax and other Schedule 2 taxes. Net investment income tax (Form 8960) and additional Medicare tax (Form 8959) are figured from your other entries.
    #[libertas_protocol_variant("SaveOtherTaxes")]
    OtherTaxes,
    /// Your parent's return (Form 8615)
    /// Form 8615 kiddie tax: the parent’s return needed for a child’s investment-income tax, including the parent's Form 2555 exclusion and Form 4952 election.
    #[libertas_protocol_variant("SaveParentTax")]
    ParentTax,
    /// Federal payments
    /// Federal income tax withheld, 2026 estimated tax payments (including a 2025 refund applied to 2026), payment with an extension request, the underpayment penalty with its 2025 facts on Form 2210, or on Form 2210-F for farmers and fishers (two-thirds of gross income from farming or fishing, one payment due January 15, 2027, no penalty when the return is filed and paid by March 1, 2027), and your 2027 estimated tax (Form 1040-ES).
    /// On separate returns, enter estimated payments you paid separately and your allocated part of joint estimated payments. Joint payments may be divided as you agree; without agreement, use the ratio of your separate tax to both spouses' separate taxes (Pub. 555). Community property does not automatically halve the payments. Withholding comes from its income source.
    #[libertas_protocol_variant("SavePayments")]
    Payments,
    /// Income by period
    /// Form 2210 Schedule AI annualized income installment method and income by period.
    #[libertas_protocol_variant("SaveIncomeByPeriod")]
    IncomeByPeriod,
    /// Injured spouse allocation (Form 8379)
    /// Form 8379 injured spouse allocation: joint refund taken for your spouse's debt, tax refund offset, child support, student loan, federal tax debt, state income tax or unemployment debt. Allocate wages, Social Security, capital gains, business income, deductions, child and education credits, premium tax credit and advance credit repayment, withholding and estimated payments. Community or separate property in Arizona, California, Idaho, Louisiana, Nevada, New Mexico, Texas, Washington or Wisconsin. Offered once injured spouse allocation is selected on About your spouse. Innocent-spouse relief (Form 8857) remains a separate scope notice.
    #[libertas_protocol_variant("SaveInjuredSpouse")]
    InjuredSpouse,
    /// Situations people often miss
    /// Income and taxes people often miss, the situations this App does not prepare, and the deductions and credits people often miss. A situation checked here brings its page onto the path; one the App does not prepare makes the estimate not calculated.
    /// Situations the estimate names as not calculated yet: railroad employee representative compensation and taxes (Form CT-2), straddles (Form 6781 Part II), special depreciation, tips or overtime not on a W-2, the sale of purchased goodwill or another section 197 intangible, or of section 1254 or 1255 property (Form 4797), like-kind exchanges of business, farm or investment real property that is not a rental, and section 1043 conflict-of-interest sales (Form 8824 Part IV), tax treaty positions (Form 8833), foreign corporations or PFICs (Schedule 1 lines 8n and 8o), a Chapter 11 bankruptcy debtor with self-employment income, casualty or theft of business, rental or investment property, other adjustments to income, alternative minimum tax items, credit recapture and look-back interest, a deferred section 965 tax, and other additional taxes.
    #[libertas_protocol_variant("SaveOftenMissed")]
    OftenMissed,
    /// Review
    /// Review the federal estimate, entered answers and remaining issues before finishing. Choose how much of your overpayment to apply to 2027 estimated tax and whose account should receive that planned credit.
    #[libertas_protocol_variant("Review")]
    Review,
    /// Child and dependent care
    /// Form 2441: care costs paid for each child or dependent (under 13, or unable to care for themselves), daycare providers (Form W-10), W-2 box10 dependent care benefits and FSA, forfeitures, grace balances, student or incapable spouse deemed income, prior-year care paid this year (Worksheet A), and own-business or partnership care programs (K-1 box13 code O).
    /// Earned income comes from each person's actual income sources. Supply actual individual earnings where asked, rather than the community-property income-tax share.
    #[libertas_protocol_variant("SaveCare")]
    Care,
    /// Business use of your homes
    /// Home office, business use of home, Form 8829, daycare, farm home office and simplified home deduction.
    #[libertas_protocol_variant("SaveHomeOffices")]
    HomeOffices,
    /// Document center
    /// Add and correct W-2, 1099, K-1, mortgage, student loan, tuition, Marketplace and other standalone tax documents.
    #[libertas_protocol_variant("SaveDocuments")]
    Documents,
    /// Income without a form
    /// Work for yourself, tips you did not report to your employer, and other income that no tax form shows. Enter other income with its amount and a short description; business receipts and unreported tips stay on their own pages.
    #[libertas_protocol_variant("SaveIncomeWithoutForm")]
    IncomeWithoutForm,
    /// Long-term care benefits
    /// Form 1099-LTC long-term care insurance and accelerated death benefits, insured-person and benefit-period records, and the per diem limit (Form 8853 Section C).
    #[libertas_protocol_variant("SaveIncomes")]
    LongTermCareBenefits,
}

/// Go to a tax page
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroDecode,
    LibertasAvroEncode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum NavigationDestination {
    /// Your return
    Return {
        /// Page
        #[libertas_constrained_by("$.navigation.return_sections")]
        section: TaxSection,
    },
    /// Income
    Income {
        /// Page
        #[libertas_constrained_by("$.navigation.income")]
        section: TaxSection,
    },
    /// Adjustments to income
    Adjustments {
        /// Page
        #[libertas_constrained_by("$.navigation.adjustments")]
        section: TaxSection,
    },
    /// Deductions
    Deductions {
        /// Page
        #[libertas_constrained_by("$.navigation.deductions")]
        section: TaxSection,
    },
    /// Credits
    Credits {
        /// Page
        #[libertas_constrained_by("$.navigation.credits")]
        section: TaxSection,
    },
    /// Other taxes
    OtherTaxes {
        /// Page
        #[libertas_constrained_by("$.navigation.other_taxes")]
        section: TaxSection,
    },
    /// Payments
    Payments {
        /// Page
        #[libertas_constrained_by("$.navigation.payments")]
        section: TaxSection,
    },
}

impl NavigationDestination {
    pub fn section(&self) -> TaxSection {
        match self {
            Self::Return { section }
            | Self::Income { section }
            | Self::Adjustments { section }
            | Self::Deductions { section }
            | Self::Credits { section }
            | Self::OtherTaxes { section }
            | Self::Payments { section } => *section,
        }
    }

    pub fn group_index(&self) -> i32 {
        match self {
            Self::Return { .. } => libertas_macros::variant_index!(Self::Return) as i32,
            Self::Income { .. } => libertas_macros::variant_index!(Self::Income) as i32,
            Self::Adjustments { .. } => libertas_macros::variant_index!(Self::Adjustments) as i32,
            Self::Deductions { .. } => libertas_macros::variant_index!(Self::Deductions) as i32,
            Self::Credits { .. } => libertas_macros::variant_index!(Self::Credits) as i32,
            Self::OtherTaxes { .. } => libertas_macros::variant_index!(Self::OtherTaxes) as i32,
            Self::Payments { .. } => libertas_macros::variant_index!(Self::Payments) as i32,
        }
    }
}

impl From<TaxSection> for NavigationDestination {
    fn from(section: TaxSection) -> Self {
        match section {
            TaxSection::Setup
            | TaxSection::People
            | TaxSection::Spouse
            | TaxSection::Children
            | TaxSection::Documents
            | TaxSection::IncomeWithoutForm
            | TaxSection::Checklist
            | TaxSection::InjuredSpouse
            | TaxSection::OftenMissed
            | TaxSection::Review => Self::Return { section },
            TaxSection::Income
            | TaxSection::Wages
            | TaxSection::InterestDividends
            | TaxSection::InvestmentSales
            | TaxSection::Retirement
            | TaxSection::SelfEmployment
            | TaxSection::Rentals
            | TaxSection::GovernmentBenefits
            | TaxSection::OtherIncome
            | TaxSection::LongTermCareBenefits
            | TaxSection::Gambling
            | TaxSection::Farming
            | TaxSection::PassThrough
            | TaxSection::CanceledDebt
            | TaxSection::WorkAbroad
            | TaxSection::HomeOffices => Self::Income { section },
            TaxSection::Adjustments
            | TaxSection::Hsa
            | TaxSection::SelfEmployedBenefits
            | TaxSection::IraContributions
            | TaxSection::Educator
            | TaxSection::StudentLoan
            | TaxSection::EmployeeExpenses
            | TaxSection::MoreAdjustments => Self::Adjustments { section },
            TaxSection::Deductions
            | TaxSection::Itemized
            | TaxSection::Medical
            | TaxSection::Taxes
            | TaxSection::Mortgage
            | TaxSection::GiftCarryovers
            | TaxSection::OtherItemized => Self::Deductions { section },
            TaxSection::Credits
            | TaxSection::Care
            | TaxSection::Education
            | TaxSection::Saver
            | TaxSection::Marketplace
            | TaxSection::MoreCredits => Self::Credits { section },
            TaxSection::OtherTaxes | TaxSection::ParentTax => Self::OtherTaxes { section },
            TaxSection::Payments | TaxSection::IncomeByPeriod => Self::Payments { section },
        }
    }
}

/// Available navigation choices
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport,
)]
pub struct NavigationAvailability {
    /// Available groups
    pub groups: Vec<i32>,
    /// Your return
    pub return_sections: Vec<i32>,
    /// Income
    pub income: Vec<i32>,
    /// Adjustments to income
    pub adjustments: Vec<i32>,
    /// Deductions
    pub deductions: Vec<i32>,
    /// Credits
    pub credits: Vec<i32>,
    /// Other taxes
    pub other_taxes: Vec<i32>,
    /// Payments
    pub payments: Vec<i32>,
}

impl NavigationAvailability {
    pub fn from_sections(sections: &[TaxSection]) -> Self {
        let mut value = Self::default();
        for &section in sections {
            let destination = NavigationDestination::from(section);
            let group = destination.group_index();
            if !value.groups.contains(&group) {
                value.groups.push(group);
            }
            match destination {
                NavigationDestination::Return { .. } => value.return_sections.push(section as i32),
                NavigationDestination::Income { .. } => value.income.push(section as i32),
                NavigationDestination::Adjustments { .. } => value.adjustments.push(section as i32),
                NavigationDestination::Deductions { .. } => value.deductions.push(section as i32),
                NavigationDestination::Credits { .. } => value.credits.push(section as i32),
                NavigationDestination::OtherTaxes { .. } => value.other_taxes.push(section as i32),
                NavigationDestination::Payments { .. } => value.payments.push(section as i32),
            }
        }
        value
    }
}

// A hub's topic list; display.json uses its section name as the page title: every topic
// the return's facts allow, whatever the ticks.
/// Section topics
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
pub struct HubTopics {
    /// Section
    /// The current section's name.
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_hidden]
    pub name: Vec<u8>,
    /// Topics
    #[libertas_suppress_label]
    #[libertas_read_only]
    /// ----
    /// Topic
    pub items: Vec<ReviewItem>,
}
/// Not listed? See all topics or Search
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
pub struct AllTopicsLink {
    /// See all topics or Search
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_hidden]
    pub label: Vec<u8>,
}

// A hub or review row: a tap opens its section (M3-N1).
/// Item
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
#[libertas_read_only]
#[libertas_link("OpenSection")]
pub struct ReviewItem {
    /// Page
    #[libertas_read_only]
    #[libertas_hidden]
    #[libertas_formatted_text]
    pub item: Vec<u8>,
    /// Status
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub status: Vec<u8>,
    /// Section
    #[libertas_hidden]
    pub section: TaxSection,
    /// Answer field
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_reveal_path]
    pub field: Option<String>,
}

/// Estimate so far
// Each page shows its own missing answers and one link to Review for the rest.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport,
)]
#[libertas_read_only]
pub struct EstimateSoFar {
    /// Estimate
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub line: Vec<u8>,
    /// What the estimate still needs
    /// Fix an item here, or open Review for items in other sections.
    #[libertas_suppress_label]
    #[libertas_read_only]
    /// ----
    /// Item
    pub remaining_items: Vec<ReviewItem>,
}

/// Person
// One listed person's child tax credit, credit for other dependents and
// earned income credit, worked out from the people page's answers.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
#[libertas_read_only]
pub struct PersonCredits {
    /// Credits
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_suppress_label]
    pub credits: Vec<u8>,
}

/// Estimate so far, on its way to the page that shows it
// The same pair without links: a response the client never displays (the
// cookie that opens an editor, a running estimate) has no row to tap.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport,
)]
#[libertas_read_only]
pub struct EstimateCarry {
    /// Estimate
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub line: Vec<u8>,
    /// What the estimate still needs
    #[libertas_read_only]
    /// ----
    /// Item
    pub remaining_items: Vec<CarriedItem>,
}

/// Item
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
pub struct CarriedItem {
    /// Page
    #[libertas_read_only]
    #[libertas_hidden]
    #[libertas_formatted_text]
    pub item: Vec<u8>,
    /// Status
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub status: Vec<u8>,
    /// Section
    #[libertas_hidden]
    pub section: TaxSection,
    /// Answer field
    #[libertas_hidden]
    #[libertas_read_only]
    pub field: Option<String>,
}

impl From<EstimateSoFar> for EstimateCarry {
    fn from(view: EstimateSoFar) -> Self {
        Self {
            line: view.line,
            remaining_items: view
                .remaining_items
                .into_iter()
                .map(|i| CarriedItem {
                    item: i.item,
                    status: i.status,
                    section: i.section,
                    field: i.field,
                })
                .collect(),
        }
    }
}

// Tests play the client, which copies a cookie's estimate into its editor.
#[cfg(test)]
impl From<EstimateCarry> for EstimateSoFar {
    fn from(view: EstimateCarry) -> Self {
        Self {
            line: view.line,
            remaining_items: view
                .remaining_items
                .into_iter()
                .map(|i| ReviewItem {
                    item: i.item,
                    status: i.status,
                    section: i.section,
                    field: i.field,
                })
                .collect(),
        }
    }
}

/// Warning about saving the current edits
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
#[libertas_read_only]
pub struct SaveWarning {
    /// What saving would remove or reopen
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub message: Vec<u8>,
}
