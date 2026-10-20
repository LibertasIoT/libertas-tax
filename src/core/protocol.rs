use crate::*;
use alloc::{string::String, vec::Vec};
use libertas::LibertasUser;
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
// Original identities retain their sequence order (M3-P19); new Care is
// appended to preserve every saved identity. Navigation groups own its hub, so
// navigation groups constrain these same identities; renumbering them is a
// storage format change.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport,
)]
pub enum TaxSection {
    /// Your 2026 return
    /// Married filing separately spouse age and blindness boxes on Form 1040 line 12d, for a spouse with no income who does not file and is not another person's dependent. Filing status: single, married filing jointly (also for the year your spouse died, if you did not remarry in 2026), married filing separately, head of household (including married to a nonresident alien) or qualifying surviving spouse. Married filing separately in a community-property state (Form 8958, Pub. 555; Arizona, California, Idaho, Louisiana, Nevada, New Mexico, Texas, Washington, Wisconsin), spouses living apart all year (section 66) and a nonresident alien spouse (sections 2(b)(2)(B), 879). Rental, partnership or S corporation income divided between spouses.
    #[libertas_protocol_variant("SaveSetup")]
    Setup,
    /// About you
    /// Your name, date of birth, legal blindness, Social Security number or ITIN, whether someone else can claim you as a dependent, and a death in 2026: the decedent's final return, date of death, deceased taxpayer, estate-paid medical expenses (section 213(c) election) and savings bonds held at death. Form 1310 is not prepared.
    #[libertas_protocol_variant("SavePeople")]
    People,
    /// About your spouse
    /// Your spouse's name, date of birth, legal blindness, Social Security number or ITIN, whether someone else can claim your spouse as a dependent, and a death in 2026 (the decedent's final return, date of death, estate-paid medical expenses and savings bonds held at death), for a joint return.
    #[libertas_protocol_variant("SaveSpouse")]
    Spouse,
    /// Children and people you support
    /// Dependents, child tax credit, credit for other dependents, earned income credit and dependent-care facts for Form 2441, including a custodial parent's release of a child to the other parent (Form 8332, divorced or separated parents, noncustodial parent) and a multiple support agreement (Form 2120, multiple support declaration). Someone else who could also claim a child (qualifying child of more than one person, the section 152(c)(4) tiebreaker rules) and a filer who can be claimed as a dependent (no dependents of their own); a child who died in 2026 (date of death, Form 8862 line 8); a partner's or friend's child who is someone else's qualifying child (no credit for other dependents); and the child's other parent who could also claim the child (longer residence, higher AGI).
    #[libertas_protocol_variant("SaveChildren")]
    Children,
    /// Your 2026 at a glance
    /// Choose the income, adjustment, deduction, credit, other tax and payment topics in your return, and what applies to the whole return: a dual-status, part-year resident, nonresident alien or U.S. territory return (Form 1040-NR, Forms 4563 and 8689), community property with a registered domestic partner (Form 8958), the choice to treat a nonresident spouse as a resident (section 6013(g) or (h)), injured spouse allocation (Form 8379) and innocent spouse relief (Form 8857).
    #[libertas_protocol_variant("SaveChecklist")]
    Checklist,
    /// Income
    /// Overview of income topics and entered income documents.
    #[libertas_protocol_variant("Hub")]
    Income,
    /// Wages
    /// W-2 wages, withholding, statutory employee Schedule C pay, railroad retirement (RRTA) compensation and tier 1 and tier 2 tax (Form 8959 Part II; Form CT-2 not calculated), allocated/unreported and noncash tips (Form 4137), qualified tips deduction, nontaxable combat pay (code Q), Medicaid waiver payments (code II, Notice 2014-7, Schedule 1 line 8s), and Trump account employer contributions (code TA). Employee pay a firm reported on Form 1099-NEC or 1099-MISC although you were its employee: Form 8919, uncollected Social Security and Medicare tax (Schedule 2 line 16b), reason codes A, C, G, H, Form SS-8, misclassified worker. Form 4852, substitute W-2 when the employer gave none. Box 1 pay for work abroad (Form 2555) is marked on its W-2; the exclusion itself is on Work abroad. Box 1 pension from a nonqualified deferred compensation or nongovernmental 457 plan (box 11, Schedule 1 line 8t) and pay earned while incarcerated (line 8u) are marked on the W-2.
    /// Minister, clergy, pastor, rabbi, priest, imam, religious order, Christian Science practitioner, housing or parsonage allowance, Form4361, Form4029, Amish or Mennonite exemption. A ministerial W-2 and the per-person clergy record are entered here.
    #[libertas_protocol_variant("SaveIncomes")]
    Wages,
    /// Interest and dividends
    /// Forms 1099-INT, 1099-DIV and 1099-OID, interest or dividends paid after the owner's death (income in respect of a decedent), Schedule B, REIT dividends and investment income. Series EE and I U.S. savings bond interest excluded for college tuition (Form 8815, Form 8818 records). Foreign financial accounts and foreign trusts (Schedule B Part III). A child's interest and dividends reported on your return (Form 8814), the child's Alaska Permanent Fund dividend, Form 1040 line 3c and line 16 box 1 (the parents' election, kiddie tax election). Nondividend distributions and return of capital (Form 1099-DIV box 3), liquidating distributions (boxes 9 and 10) and undistributed long-term capital gains (Form 2439, the fund's tax paid credit).
    #[libertas_protocol_variant("SaveIncomes")]
    InterestDividends,
    /// Investment sales
    /// Form 1099-B, Form 8949 and Schedule D capital gains, stock sales and capital loss carryovers. Sale of your main home (1099-S). Incentive stock options (ISO): Form 3921 exercises and ISO shares sold. Installment payments from an earlier-year sale (Form 6252), a like-kind exchange with a related person from 2024 or 2025 whose property was sold in 2026 (Form 8824 Part II, section 1031(f) two-year rule) and 2021–2025 business-property losses (Form 4797 line 7). Futures, foreign currency contracts and section 1256 options such as broad-based index options (Form 1099-B boxes 8–11, Form 6781 Part I, 60/40 treatment), and the net section 1256 contracts loss carryback election (Form 6781 box D). Employee stock: shares from nonqualified stock options, restricted stock and RSUs (Form W-2 box 12 code V), employee stock purchase plan (ESPP) shares (Form 3922, qualifying and disqualifying dispositions) and employer stock from a retirement plan with net unrealized appreciation (NUA, Form 1099-R box 6). Qualified small business stock (QSBS, section 1202, Form 8949 code Q). Small business stock losses (section 1244, Form 4797). Worthless securities and nonbusiness bad debts (personal loans that were never repaid). Stock and equity options you bought or wrote (calls and puts: sold, bought back, expired or exercised), short sales, wash sales (Form 1099-B box 1g), market discount bonds (box 1f, Form 8949 code D) and sales you received as a nominee (code N). Digital assets: Form 1099-DA, cryptocurrency, bitcoin, stablecoins and NFTs sold or exchanged, with or without a form (Form 8949 boxes G–L). Qualified opportunity fund (QOF, opportunity zone) investments: the deferred gain taxed in 2026 and a 2026 gain invested in a fund in 2027 (Form 8997, Form 8949 codes Y and Z).
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
    /// Form 1099-MISC: rents, royalties, other income, prizes and awards, medical and health care payments, fishing boat proceeds, substitute payments in lieu of dividends, gross proceeds paid to an attorney, fish sold for cash. Other income (Schedule 1 line 8): Alaska Permanent Fund dividend (8g), Olympic and Paralympic medals and USOC prize money (8m, 24c), wages earned while incarcerated not on a W-2 (8u), digital assets received as income such as staking or mining rewards and airdrops (8v), refunds or reimbursements of an earlier year's itemized deductions (tax benefit rule, recoveries, Form 1098 box 4 mortgage interest refund, medical reimbursement), credit card insurance benefits, private or union unemployment fund benefits, and other taxable income on line 8z such as rewards, found property, taxable damages, executor fees, medical research study pay. Equipment you rent out. Alimony received. Scholarships and fellowship grants not on a W-2 (Schedule 1 line 8r, Pell grants, room and board, stipends, Form 1098-T box 5); jury duty pay and pay turned over to an employer (lines 8h and 24a); hobby income, activity not engaged in for profit (line 8j, section 183, 1099-K sales of crafts or collectibles made as a hobby); qualified wildfire relief payments (section 139M), qualified disaster relief payments (section 139, FEMA individual assistance), insurance for additional living expenses after a casualty (section 123, loss of use, temporary housing), and hazard mitigation grants (section 139(g), FEMA HMGP, flood mitigation). East Palestine train derailment payments are not taxable and need not be entered. Form 1099-Q distributions from 529 plans (qualified tuition programs, QTP) and Coverdell education savings accounts (ESA): taxable earnings, qualified education expenses paid from them, including K-12 tuition, room and board, student-loan payments and apprenticeships, the 10% additional tax (Form 5329 Part II) and 529-to-Roth IRA rollovers. Form 1099-SA and trustee statements from your own or inherited HSA, Archer MSA or Medicare Advantage MSA: ordinary distributions, rollovers and account events (Form 8889 Part II or Form 8853 Sections A and B). Form 1099-LTC long-term care insurance and accelerated death benefits, the per diem limit (Form 8853 Section C). Form 1099-QA distributions from ABLE accounts (section 529A), the 10% additional tax (Form 5329 Part II), and contributions to an ABLE account above the limit (Form 5329 Part VIII).
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
    /// Form 2555 foreign earned income exclusion (up to $132,900 for 2026), foreign housing exclusion and housing deduction for a U.S. citizen or resident alien living and working abroad (expat) with a foreign tax home: bona fide residence test, physical presence test (330 full days in 12 months), qualifying days, housing expenses with the high-cost locality limits of Notice 2026-25, and the housing deduction carryover; pay from a foreign employer without a W-2. W-2 pay for work abroad is marked on its W-2 under Wages, and a business run abroad on Self-employment. The foreign tax credit (Form 1116) is on More credits; tax treaty positions (Form 8833) are on Less common income.
    /// Revoked exclusion, IRS consent to re-elect, bonus or RSU for work abroad and pay for prior-year services.
    #[libertas_protocol_variant("SaveIncomes")]
    WorkAbroad,
    /// Less common income
    /// Less common income the estimate names as not calculated yet: railroad employee representative compensation and taxes (Form CT-2), an installment sale (Form 6252), straddles (Form 6781 Part II), special depreciation, tips or overtime not on a W-2, the sale of purchased goodwill or another section 197 intangible, or of section 1254 or 1255 property (Form 4797), like-kind exchanges of business, farm or investment real property that is not a rental, and section 1043 conflict-of-interest sales (Form 8824 Part IV), tax treaty positions (Form 8833), and foreign corporations or PFICs (Schedule 1 lines 8n and 8o).
    /// Chapter11 bankruptcy debtor with self-employment income; the separate bankruptcy-estate return is not prepared. Clergy housing and religious SE exemptions belong to Wages.
    #[libertas_protocol_variant("SaveSituations")]
    MoreIncome,
    /// Adjustments to income
    /// Overview of adjustments to income on Schedule 1.
    #[libertas_protocol_variant("Hub")]
    Adjustments,
    /// Health and medical savings accounts
    /// Form 8889 contributions to your health savings account, excess contributions and their corrections, qualified HSA funding distributions (QHFD), lifetime IRA-to-HSA funding uses, the last-month rule and prior testing histories. Ordinary own-HSA Form1099-SA distributions and trustee account events are entered once on Other income. Archer MSA contributions (Form 8853 Part I): personal and employer deposits, the shared family limit, corrections and excess.
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
    /// More itemized deductions
    /// Other gifts of property and casualty or theft of business, rental or investment property.
    #[libertas_protocol_variant("SaveSituations")]
    MoreItemized,
    /// Credits
    /// Overview of federal tax credits.
    #[libertas_protocol_variant("Hub")]
    Credits,
    /// Education credits
    /// Form 1098-T, Form 8863, American opportunity tax credit (AOTC) and lifetime learning credit (LLC) for education expenses, including a Pell grant or scholarship the student includes in income to raise the credit (Pub. 970 coordination, room and board, Schedule 1 line 8r).
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
    /// Choosing not to claim the earned income credit (Form 1040 line 27c). Foreign income tax deduction instead of a credit (Schedule A line 6), or foreign tax credit on Form 1116 by category (general, passive, foreign branch, sanctioned-country section 901(j) income) and country, accrued or paid foreign taxes, the high-tax kickout, foreign capital gains (Worksheets A and B, Worksheet for Line 18), a business run abroad, Schedule B (Form 1116) carryover reconciliation (Schedule K-3 amounts are read from the K-1 page), adoption on Form 8839, mortgage interest credit on Form 8396, business credit carryovers on Form 3800, including alternative motor vehicle credit (Form 8910) carryforwards and low-income housing credit (Form 8586) carryforwards and unallowed passive credits (Form 8582-CR), prior minimum tax credit on Form 8801 (exclusion-only AMT, MTCNOL carryovers/carrybacks, MTFTCE, retained foreign-earned-income worksheets, prior joint or separate returns, old qualified electric vehicle credit), residential energy credit carryovers on Form 5695, and credit for the elderly or disabled on Schedule R. Clean vehicle, fuel cell vehicle or electric car (EV) credits and dealer-transfer reconciliation on Form 8936, including a new vehicle used in a saved business or farm and its Form 3800 line 1y business part, alternative fuel vehicle refueling or charging property on Form 8911, and electric vehicle credit carryovers on Form 8834. The D.C. first-time homebuyer credit carryover (Form 8859), federal tax on home kerosene (Form 4136), qualified tax-credit bonds (Form 8912), the employer tip FICA credit carryover (Form 8846), a BBA audit adjustment (Form 8978), 2025 education credit recapture and credits after an earlier disallowance (Form 8862, where a 2-year or 10-year ban leaves the credit out). General business credits figured on their own source forms can be entered by Form 3800 line: research/R&D Form 6765, disabled access Form 8826, employer childcare Form 8882, differential wage Form 8932, orphan drug Form 8820, new markets Form 8874, renewable electricity Form 8835, advanced manufacturing production Form 7207, clean hydrogen Form 7210 and carbon oxide sequestration Form 8933. Includes eligible elective payment (direct pay), credits bought under section 6418, transfers, empowerment-zone Form 8844 carryforwards and passive credit limitations. This section covers credits and the foreign income tax deduction; personal car loan interest is on Deductions. Rehabilitation (Form 3468 Part VII), energy (Part VI) and clean electricity investment (Part V) credit carryovers and their carrybacks to 2023 through 2025 are business credit carryovers.
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
    /// Form 8379 injured spouse allocation: joint refund taken for your spouse's debt, tax refund offset, child support, student loan, federal tax debt, state income tax or unemployment debt. Allocate wages, Social Security, capital gains, business income, deductions, child and education credits, premium tax credit and advance credit repayment, withholding and estimated payments. Community or separate property in Arizona, California, Idaho, Louisiana, Nevada, New Mexico, Texas, Washington or Wisconsin. Offered once the checklist's injured spouse item is ticked. Innocent-spouse relief (Form 8857) remains a separate scope notice.
    #[libertas_protocol_variant("SaveInjuredSpouse")]
    InjuredSpouse,
    /// Review
    /// Review the federal estimate, entered answers and remaining issues before finishing.
    #[libertas_protocol_variant("Review")]
    Review,
    /// Child and dependent care
    /// Form 2441: daycare providers (Form W-10), W-2 box10 dependent care benefits and FSA, forfeitures, grace balances, student or incapable spouse deemed income, prior-year care paid this year (Worksheet A), and own-business or partnership care programs (K-1 box13 code O).
    /// Earned income comes from each person's actual income sources. Supply actual individual earnings where asked, rather than the community-property income-tax share.
    #[libertas_protocol_variant("SaveCare")]
    Care,
    /// Business use of your homes
    /// Home office, business use of home, Form 8829, daycare, farm home office and simplified home deduction.
    #[libertas_protocol_variant("SaveHomeOffices")]
    HomeOffices,
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
            | TaxSection::Checklist
            | TaxSection::InjuredSpouse
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
            | TaxSection::Gambling
            | TaxSection::Farming
            | TaxSection::PassThrough
            | TaxSection::CanceledDebt
            | TaxSection::WorkAbroad
            | TaxSection::HomeOffices
            | TaxSection::MoreIncome => Self::Income { section },
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
            | TaxSection::OtherItemized
            | TaxSection::MoreItemized => Self::Deductions { section },
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

/// Tax return
/// Accepted pages are saved. Larger forms offer recovery of completed field edits on this device; small-page edits and unfinished typing are discarded on exit. Synthetic data only; no return is filed.
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroDecode,
    LibertasAvroEncode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
// No request of the tax interview is offered to the Hub LLM as a tool call:
// every page needs its editor, its cookie and the filer's confirmation.
#[libertas_llm_tool_call(false)]
// Complete editor payloads are owned inline; keep the wire model free of allocation-only wrappers.
#[allow(clippy::large_enum_variant)]
pub enum TaxInterviewProtocol {
    // Entry action, also offered on Review while an interview step remains.
    // Workflow PrevRequest exposes earlier-section navigation through Back.
    /// Start or resume
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    OpenInterview {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
    },
    /// Your 2026 return
    #[libertas_response]
    #[libertas_workflow_request("SaveSetup")]
    BeginSetup {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        estimate: Vec<u8>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
    },
    // Cache large forms by return identity, accepted-data revision, and navigation
    // context. A changed accepted page invalidates drafts; review-only Next does not.
    // Keeping review in the tuple prevents restoring an obsolete Submit destination.
    // The first section has no previous section. Native Back follows page provenance;
    // do not expose a PrevRequest that would reload this same editor.
    /// Your 2026 return
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveSetup {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Your 2026 return
        #[libertas_calculation_request("EstimateSetup")]
        value: ReturnSetup,
    },
    /// About you
    #[libertas_response]
    #[libertas_workflow_request("SavePeople")]
    BeginPeople {
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        professional_election: crate::ProfessionalRentalElection,
        /// Existing annual rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_professional_elections: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        estimate: Vec<u8>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
    },
    /// About you
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SavePeople {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.professional_election")]
        professional_election: crate::ProfessionalRentalElection,
        /// Existing annual rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_professional_elections")]
        allowed_professional_elections: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// About you
        #[libertas_calculation_request("EstimateYou")]
        you: Filer,
    },
    // Sent only on a joint return, after the filer's own page.
    /// About your spouse
    #[libertas_response]
    #[libertas_workflow_request("SaveSpouse")]
    BeginSpouse {
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        professional_election: crate::ProfessionalRentalElection,
        /// Existing annual rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_professional_elections: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        estimate: Vec<u8>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
    },
    /// About your spouse
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveSpouse {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.professional_election")]
        professional_election: crate::ProfessionalRentalElection,
        /// Existing annual rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_professional_elections")]
        allowed_professional_elections: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// About your spouse
        #[libertas_calculation_request("EstimateSpouse")]
        spouse: SpouseAnswers,
    },
    // The deductions section page (unit M3-P9): the route and cash gifts.
    /// Deductions
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveDeductions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Available deduction routes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_routes")]
        #[libertas_calculation_request("EstimateDeductions")]
        allowed_routes: Vec<i32>,
        /// Available repayment answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_repayment")]
        #[libertas_calculation_request("EstimateDeductions")]
        allowed_repayment: Vec<i32>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Deductions
        #[libertas_calculation_request("EstimateDeductions")]
        value: Deductions,
        /// Saved business homes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.home_sites")]
        home_sites: Vec<RecordChoice>,
    },
    /// Less common situations
    #[libertas_response]
    #[libertas_workflow_request("SaveSituations")]
    BeginSituations {
        /// Topic
        #[libertas_hidden]
        #[libertas_read_only]
        topic: TaxSection,
        /// Allowed situations
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_situations: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        estimate: Vec<u8>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
    },
    /// Less common situations
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveSituations {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        // Part of the draft identity, so a less-common income draft never
        // restores into another hub's list.
        /// Topic
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.topic")]
        #[libertas_calculation_request("EstimateSituations")]
        topic: TaxSection,
        /// Allowed situations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_situations")]
        #[libertas_calculation_request("EstimateSituations")]
        allowed_situations: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Less common situations
        #[libertas_calculation_request("EstimateSituations")]
        value: Situations,
    },
    // The people page (M2 page 3), asked once on every return after the
    // filer pages (gate fixes batch A). A return that lists nobody yet opens
    // it through the workflow response.
    /// Children and people you support
    #[libertas_response]
    #[libertas_workflow_request("SaveChildren")]
    BeginChildren {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        estimate: Vec<u8>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Allowed residence answers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_lived: Vec<i32>,
        /// Days you were alive in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        days_alive: u32,
    },
    /// Children and people you support
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveChildren {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        /// [NullText]
        /// Current estimate is unavailable while these answers need correction.
        /// [SetText]
        /// Current estimate
        estimate: Option<Vec<u8>>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        /// [NullText]
        /// Current interview progress is unavailable while these answers need correction.
        /// [SetText]
        /// Current interview progress
        progress: Option<Vec<u8>>,
        /// Allowed residence answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_lived")]
        #[libertas_calculation_request("EstimateDependents")]
        allowed_lived: Vec<i32>,
        /// Days you were alive in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.days_alive")]
        #[libertas_calculation_request("EstimateDependents")]
        days_alive: u32,
        /// Children and people you support
        #[libertas_calculation_request("EstimateDependents")]
        dependents: Dependents,
    },
    /// Income documents
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveIncomes {
        /// Empty construction choices until actual rental dates are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_rental_recovery_choices")]
        initial_rental_recovery_choices: Vec<i32>,
        /// Businesses available for individual QBI aggregation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.qbi_sources")]
        qbi_sources: Vec<crate::QbiAggregationMemberChoice>,
        /// Saved physical buildings for section 179D cost-source claims
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rental_energy_buildings")]
        rental_energy_buildings: Vec<RecordChoice>,
        /// Empty construction choices for a new K-1 source; accepted parent choices come with its supplied editor value.
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.empty_k1_activity_choices")]
        empty_k1_activity_choices: alloc::boxed::Box<crate::K1ActivityChoiceContext>,
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// Actual ownership answers for retained Form 8960 and absorbed carryback sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_retained_owners")]
        allowed_injured_retained_owners: Vec<i32>,
        /// Existing long-term-care statements for actual period payments
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_payee_sources")]
        ltc_payee_sources: Vec<RecordChoice>,

        /// S-corporation K-1 records that may have paid a W-2
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.wage_scorporations")]
        wage_scorporations: Vec<RecordChoice>,
        /// Available S-corporation employer relation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_wage_scorporations")]
        allowed_wage_scorporations: Vec<i32>,
        /// Initial automatic investment-confirmation choices for a fresh K-1 row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_k1_investment_choices")]
        initial_k1_investment_choices: Vec<i32>,
        /// Actual current businesses available for retained research attribution
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Applicable retained-source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Available k1 credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.k1_credit_lines")]
        k1_credit_lines: Vec<i32>,
        /// Available research credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_credit_lines")]
        research_credit_lines: Vec<i32>,
        /// Available renewable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.renewable_credit_lines")]
        renewable_credit_lines: Vec<i32>,
        /// Available transferable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.transferable_credit_lines")]
        transferable_credit_lines: Vec<i32>,
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available at-risk histories for ownership begun before 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_continuing_risk")]
        allowed_continuing_risk: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        /// [NullText]
        /// Awaiting corrected income source answers
        /// [SetText]
        /// Current estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<Vec<u8>>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        /// [NullText]
        /// Awaiting corrected income source answers
        /// [SetText]
        /// Current interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Option<Vec<u8>>,
        // Part of the draft identity, so a Wages draft never restores into
        // the Interest and dividends page.
        /// Topic
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.topic")]
        #[libertas_calculation_request("EstimateDocuments")]
        topic: TaxSection,
        /// Allowed document kinds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_documents")]
        #[libertas_calculation_request("EstimateDocuments")]
        allowed_documents: Vec<i32>,
        /// Allowed document owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateDocuments")]
        allowed_owners: Vec<i32>,
        /// Allowed community-property answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_community")]
        #[libertas_calculation_request("EstimateDocuments")]
        allowed_community: Vec<i32>,
        /// Allowed management of community funds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_refund_management")]
        allowed_refund_management: Vec<i32>,
        /// Foreign countries for the separate FBAR reporting answer
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        foreign_countries: Vec<RecordChoice>,
        /// Accepted interest statements and annual instruments for education bonds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        interest_statements: Vec<RecordChoice>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,
        /// Farms and farm rentals on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<RecordChoice>,
        /// Accepted same-owner business, farm and farm-rental recipients
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_activities")]
        cooperative_activities: Vec<RecordChoice>,
        /// Accepted property-owner links; unavailable owners supply no choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_properties")]
        cooperative_properties: Vec<RecordChoice>,
        /// Businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<RecordChoice>,
        /// Schedule C and Schedule F businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.qbi_businesses")]
        qbi_businesses: Vec<RecordChoice>,
        /// Rental properties on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rentals")]
        rentals: Vec<RecordChoice>,
        /// Saved canonical mortgage loans for home rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Royalty entries on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.royalty_properties")]
        royalty_properties: Vec<RecordChoice>,
        /// Equipment rented out on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.equipment_rentals")]
        equipment_rentals: Vec<RecordChoice>,
        /// Hobbies on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hobbies")]
        hobbies: Vec<RecordChoice>,
        /// Commodity Credit Corporation loans forfeited in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ccc_lots")]
        ccc_lots: Vec<RecordChoice>,
        /// Current principal agricultural codes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        // Fixed farm-code taxonomy supplied by farm_specials::code_choices (FARM_CODES).
        #[libertas_size(max = 17)]
        farm_codes: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_limitations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        // Fixed farm-code taxonomy supplied by farm_specials::limitation_choices (codes 1 through 10).
        #[libertas_size(max = 10)]
        farm_limitations: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        farm_sources: Vec<FarmChoice>,
        /// Current accepted farm source catalog: basis_assets
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        basis_assets: Vec<FarmChoice>,
        /// Current accepted farm source catalog: farm_method_changes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        farm_method_changes: Vec<FarmChoice>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_iras")]
        inherited_iras: Vec<RecordChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_roth_iras")]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Long-term care periods records on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_insured")]
        ltc_insured: Vec<RecordChoice>,
        /// Actual nonqualified annuity sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.annuity_sources")]
        annuity_sources: Vec<crate::LtcSourceChoice>,
        /// Actual prior-year deduction recovery sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.recovery_sources")]
        recovery_sources: Vec<crate::LtcSourceChoice>,
        /// Allowed business income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_business_source")]
        allowed_business_source: Vec<i32>,
        /// Allowed farm income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farm_source")]
        allowed_farm_source: Vec<i32>,
        /// Allowed partnership income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_partner_source")]
        allowed_partner_source: Vec<i32>,
        /// Allowed home sale spouse answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_home_spouse")]
        allowed_home_spouse: Vec<i32>,
        /// Allowed answers about the return reporting a child's income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_other_parent")]
        allowed_other_parent: Vec<i32>,
        /// Children who can be reported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.children")]
        children: Vec<crate::ChildChoice>,
        /// Expected W-2 tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Income documents
        #[libertas_calculation_request("EstimateDocuments")]
        documents: IncomeDocuments,
        /// Initial equipment-origin choices for a fresh credit row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_carbon_oxide_origins")]
        initial_carbon_oxide_origins: Vec<i32>,
        /// Initial applicability for a newly added renewable-facility source row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_renewable_facility_origins")]
        #[libertas_output_only]
        initial_renewable_facility_origins: Vec<i32>,
    },
    // The HSA topic of the adjustments hub (unit M3-P7). HsaOptions reads
    // only the accounts and rewrites the family split's allowed set; it never
    // reads the split, which EstimateHsa reads downstream.
    /// Health and medical savings accounts
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveHsa {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed account owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateHsa,HsaOptions")]
        allowed_owners: Vec<i32>,
        /// Other-filer HSA eligibility applies on this married return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.other_coverage_allowed")]
        other_coverage_allowed: bool,
        /// Allowed family contribution splits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_family_split")]
        #[libertas_calculation_request("EstimateHsa")]
        allowed_family_split: Vec<i32>,
        /// Actual failed cash deposits into HSA accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hsa_failed_deposits")]
        hsa_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual failed cash deposits into ARCHER accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.archer_failed_deposits")]
        archer_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual accepted HSA corrective statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hsa_return_documents")]
        hsa_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// Actual accepted Archer corrective statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.archer_return_documents")]
        archer_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// IRA distributions on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ira_sources")]
        #[libertas_calculation_request("EstimateHsa")]
        ira_sources: Vec<crate::HsaFundingSourceChoice>,
        /// Earned compensation on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.archer_income")]
        #[libertas_calculation_request("EstimateHsa")]
        archer_income: Vec<RecordChoice>,
        /// Health and medical savings accounts
        #[libertas_calculation_request("EstimateHsa")]
        value: HsaPage,
    },
    /// Child and dependent care
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveCare {
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// People already on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,

        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed filers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Filers eligible for monthly deemed income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.care_filers")]
        care_filers: Vec<i32>,
        /// Calendar months
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.care_months")]
        care_months: Vec<RecordChoice>,
        /// Businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<CareSourceChoice>,
        /// Farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<CareSourceChoice>,
        /// Partnership K-1s
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.partnerships")]
        partnerships: Vec<CareSourceChoice>,
        /// Child and dependent care
        #[libertas_calculation_request("EstimateCare")]
        value: CarePage,
    },
    // The self-employed health insurance and retirement plan topic of the
    // adjustments hub (unit M3-P8). Its rows are membership-only: the task
    // builds one row per business the return already has, and the client
    // edits only each row's health and retirement fields.
    /// Self-employed health insurance and retirement plan
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveSelfEmployedBenefits {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        /// [NullText]
        /// Estimate will appear after the current answers are calculated.
        /// [SetText]
        /// Estimate
        estimate: Option<Vec<u8>>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        /// [NullText]
        /// Interview progress will appear after the current answers are calculated.
        /// [SetText]
        /// Interview progress
        progress: Option<Vec<u8>>,
        /// Accepted Marketplace source statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_policies")]
        ptc_policies: Vec<RecordChoice>,
        /// Current businesses and entities
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.health_activities")]
        health_activities: Vec<RecordChoice>,
        /// Current people available to insure
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.health_people")]
        health_people: Vec<PersonChoice>,
        /// Existing wage statements for shareholder plan linkage
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.health_wages")]
        health_wages: Vec<RecordChoice>,
        /// Self-employed health insurance and retirement plan
        #[libertas_calculation_request("EstimateSelfEmployedBenefits")]
        value: SelfEmployedBenefits,
    },
    // The IRA contributions topic of the adjustments hub (unit M3-P8).
    /// IRA contributions
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveIraContributions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed contributors
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateIraContributions")]
        allowed_owners: Vec<i32>,
        /// IRA contributions
        #[libertas_calculation_request("EstimateIraContributions")]
        value: IraContributions,
    },
    // The educator expenses topic of the adjustments hub (unit M3-P8).
    /// Educator expenses
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveEducator {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed educators
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateEducator")]
        allowed_owners: Vec<i32>,
        /// Educator expenses
        #[libertas_calculation_request("EstimateEducator")]
        value: EducatorExpenses,
    },
    // The student loan interest topic of the adjustments hub (unit M3-P8).
    /// Student loan interest
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveStudentLoan {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Student loan interest
        #[libertas_calculation_request("EstimateStudentLoan")]
        value: StudentLoanInterest,
    },
    // The Form 2106 employee business expenses topic of the adjustments hub.
    /// Reservist, performing artist and fee-basis official expenses
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveEmployeeExpenses {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateEmployeeExpenses")]
        allowed_owners: Vec<i32>,
        /// Reservist, performing artist and fee-basis official expenses
        #[libertas_calculation_request("EstimateEmployeeExpenses")]
        value: EmployeeExpenses,
    },
    // The medical topic of the itemized hub (unit M3-P9).
    /// Medical and dental expenses
    /// On a community-property separate return, enter your share of personal expenses paid from community funds and the expenses you paid from your separate funds. Do not enter the household total again.
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveMedical {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Medical and dental expenses
        #[libertas_calculation_request("EstimateMedical")]
        value: Medical,
    },
    // The taxes topic of the itemized hub (unit M3-P9).
    /// Taxes you paid
    /// On a community-property separate return, enter the amount attributable to this return under the community or separate source of the payment. Keep the full home-office costs where the page asks for them; the App allocates those costs once.
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveTaxes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Taxes you paid
        #[libertas_calculation_request("EstimateTaxes")]
        value: Taxes,
    },
    // The investment interest and other topic of the itemized hub (unit M3-P9).
    /// Investment interest and other deductions
    /// On a community-property separate return, enter only the deduction attributable to this return. An amount already supplied by an income statement or another source page is not entered again.
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveOtherItemized {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Available filers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Investment borrowing sources on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.interest_sources")]
        interest_sources: Vec<crate::InvestmentSourceChoice>,
        /// Actual K-1 entities for a retained interest balance
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.investment_k1_sources")]
        investment_k1_sources: Vec<crate::RecordChoice>,
        /// Applicable retained-interest origin question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.investment_origin_choices")]
        investment_origin_choices: Vec<i32>,
        /// Investment interest and other deductions
        #[libertas_calculation_request("EstimateOtherItemized")]
        value: OtherItemized,
    },
    // The home mortgage topic of the itemized hub (unit M3-P10).
    /// Home mortgage interest
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveMortgages {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Loans currently selected by rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rental_mortgage_loans")]
        rental_mortgage_loans: Vec<i64>,
        /// Home mortgage interest
        #[libertas_calculation_request("EstimateMortgages")]
        value: Mortgages,
    },
    /// Business use of your homes
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveHomeOffices {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Saved businesses and farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.home_activities")]
        home_activities: Vec<RecordChoice>,
        /// Saved casualty events
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.casualty_events")]
        casualty_events: Vec<RecordChoice>,
        /// Allowed home owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_home_owners")]
        allowed_home_owners: Vec<i32>,
        /// Business use of your homes
        #[libertas_calculation_request("EstimateHomeOffices")]
        value: HomeOffices,
    },
    // The charity carryovers topic of the itemized hub (unit M3-P10).
    /// Gifts of property and carryovers
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveGiftCarryovers {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed captains
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateGiftCarryovers")]
        allowed_owners: Vec<i32>,
        /// Saved appraisals available to gift rows
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.appraisal_choices")]
        appraisal_choices: Vec<RecordChoice>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Actual current K-1 sources for retained activity gifts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_charity_k1s")]
        retained_charity_k1s: Vec<RecordChoice>,
        /// Actual retained-gift origin choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_charity_origins")]
        retained_charity_origins: Vec<i32>,
        /// Applicable retained-gift source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Gifts of property and carryovers
        #[libertas_calculation_request("EstimateGiftCarryovers")]
        value: GiftCarryovers,
    },
    // The education topic of the credits hub (unit M3-P11).
    /// Education credits
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveEducation {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Education credits
        #[libertas_calculation_request("EstimateEducation")]
        value: EducationCredits,
    },
    // The retirement savings topic of the credits hub (unit M3-P11).
    /// Retirement savings credit
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveSaver {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Retirement savings credit
        #[libertas_calculation_request("EstimateSaver")]
        value: SaverCredit,
    },
    // The Form 1095-A topic of the credits hub (unit T2-27p).
    /// Health insurance marketplace
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveMarketplace {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// People supported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,
        /// Current people and saved external enrollees
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_people")]
        ptc_people: Vec<RecordChoice>,
        /// Saved other tax families
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_families")]
        ptc_families: Vec<RecordChoice>,
        /// Saved policy sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_policies")]
        ptc_policies: Vec<RecordChoice>,
        /// Accepted W-2 code FF sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_w2")]
        ptc_w2: Vec<RecordChoice>,
        /// Employees on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_employees")]
        ptc_employees: Vec<RecordChoice>,
        /// Current people outside the calculated tax family
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_nonreturn_people")]
        ptc_nonreturn_people: Vec<RecordChoice>,
        /// Claimed dependents on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_claimed_dependents")]
        ptc_claimed_dependents: Vec<RecordChoice>,
        /// Available marriage calculation choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_marriage")]
        allowed_marriage: Vec<i32>,
        /// Available separate-return answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_separate")]
        allowed_separate: Vec<i32>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Health insurance marketplace
        #[libertas_calculation_request("EstimateMarketplace")]
        value: MarketplaceFactsPage,
    },
    // The credits hub's last topic (unit M3-P12).
    /// More credits
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveMoreCredits {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        /// [NullText]
        /// Current estimate is unavailable while these answers need correction.
        /// [SetText]
        /// Current estimate
        estimate: Option<Vec<u8>>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        /// [NullText]
        /// Current interview progress is unavailable while these answers need correction.
        /// [SetText]
        /// Current interview progress
        progress: Option<Vec<u8>>,
        /// Available owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateMoreCredits")]
        allowed_owners: Vec<i32>,
        /// Businesses run abroad on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.foreign_businesses")]
        #[libertas_calculation_request("EstimateMoreCredits")]
        foreign_businesses: Vec<RecordChoice>,
        /// Investment sales on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.capital_sales")]
        #[libertas_calculation_request("EstimateMoreCredits")]
        capital_sales: Vec<RecordChoice>,
        /// Foreign tax situations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.foreign_situations")]
        #[libertas_calculation_request("EstimateMoreCredits")]
        foreign_situations: Vec<ForeignSituationChoice>,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Available businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<RecordChoice>,
        /// Available farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<RecordChoice>,
        /// Saved clean vehicle property choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.clean_vehicle_properties")]
        clean_vehicle_properties: Vec<RecordChoice>,
        /// Available rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rentals")]
        rentals: Vec<RecordChoice>,
        /// Available research businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Available entered credits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.entered_credits")]
        entered_credits: Vec<i32>,
        /// Available carryback credits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.carryback_credits")]
        carryback_credits: Vec<i32>,
        /// Available allowed carry filers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_carry_filers")]
        allowed_carry_filers: Vec<i32>,
        /// Available initial credit elections
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_credit_elections")]
        initial_credit_elections: Vec<i32>,
        /// Available initial research choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_research_choices")]
        initial_research_choices: Vec<i32>,
        /// More credits
        #[libertas_calculation_request("EstimateMoreCredits")]
        #[libertas_copy_from("$.value")]
        value: MoreCredits,
        /// Initial equipment-origin choices for a fresh credit row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_carbon_oxide_origins")]
        initial_carbon_oxide_origins: Vec<i32>,
        /// Initial applicability for a newly added renewable-facility source row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_renewable_facility_origins")]
        #[libertas_output_only]
        initial_renewable_facility_origins: Vec<i32>,
    },
    // The other taxes section's page (u5-credits): its calculated taxes
    // and the refusal kinds, one list.
    /// Other taxes
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveOtherTaxes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        /// [NullText]
        /// Current estimate is unavailable while these answers need correction.
        /// [SetText]
        /// Current estimate
        estimate: Option<Vec<u8>>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        /// [NullText]
        /// Current interview progress is unavailable while these answers need correction.
        /// [SetText]
        /// Current interview progress
        progress: Option<Vec<u8>>,
        /// Available owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateOtherTaxes")]
        allowed_owners: Vec<i32>,
        /// Allowed owners of a joint subsidized-home gift
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_gift_owners")]
        mortgage_gift_owners: Vec<i32>,
        /// Existing current-year Coverdell correction sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.coverdell_corrections")]
        #[libertas_calculation_request("EstimateOtherTaxes")]
        coverdell_corrections: Vec<RecordChoice>,
        /// Other taxes
        #[libertas_calculation_request("EstimateOtherTaxes")]
        value: OtherTaxes,
    },
    // The adjustments hub's last topic (unit u5-adjust).
    /// More adjustments
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveMoreAdjustments {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// More adjustments
        #[libertas_calculation_request("EstimateMoreAdjustments")]
        value: MoreAdjustments,
    },
    // The federal payments section page (unit M3-P13).
    /// Federal payments
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SavePayments {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Available 2025 return choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_prior_returns")]
        #[libertas_calculation_request("EstimatePayments")]
        allowed_prior_returns: Vec<i32>,
        /// Available farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farmer")]
        #[libertas_calculation_request("EstimatePayments")]
        allowed_farmer: Vec<i32>,
        /// Available 2027 farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farmer_next_year")]
        #[libertas_calculation_request("EstimatePayments")]
        allowed_farmer_next_year: Vec<i32>,
        /// Allowed management of community funds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_refund_management")]
        allowed_refund_management: Vec<i32>,
        /// Federal payments
        #[libertas_calculation_request("EstimatePayments")]
        #[libertas_copy_from("$.value")]
        value: Payments,
    },
    // Form 2210 Schedule AI's income by period, after the payments page when
    // box C is chosen (unit M3-P14).
    /// Income by period
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveIncomeByPeriod {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Income by period
        #[libertas_calculation_request("EstimateIncomeByPeriod")]
        value: IncomeByPeriod,
    },
    // Form 8379, after the payments section while the checklist's injured
    // spouse item is ticked on a joint return (unit T3-8379).
    /// Injured spouse allocation (Form 8379)
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveInjuredSpouse {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Injured spouse allocation (Form 8379)
        #[libertas_calculation_request("EstimateInjuredSpouse")]
        #[libertas_copy_from("$.value")]
        value: InjuredSpouse,
    },
    // Form 8615's parent facts (u5-losses), after the other taxes page while
    // Form 8615 applies to this return.
    /// Your parent's return (Form 8615)
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveParentTax {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Your parent's return (Form 8615)
        #[libertas_calculation_request("EstimateParentTax")]
        value: ParentReturn,
    },
    // The opening checklist after the people pages (unit M3-P18), opened by
    // EditRequest with every tick computed; a hub's "What you told us" row
    // reopens it.
    /// Your 2026 at a glance
    #[libertas_request]
    #[libertas_prev_request("Back")]
    // The whole-return members a joint return needs are the bits from 4 up
    // (WHOLE_RETURN_JOINT_ONLY); the server checks the same on save.
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    SaveChecklist {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Joint return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.joint")]
        joint: bool,
        /// Your 2026 at a glance
        #[libertas_calculation_request("EstimateChecklist")]
        value: Checklist,
    },
    /// Back
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    Back {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Previous page
        #[libertas_hidden]
        #[libertas_copy_from("$.previous")]
        previous: i32,
    },
    // Opened from a hub or the review: `from` is where a saved page returns,
    // and the popup starts at the service's suggestion.
    /// Go to
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    Navigate {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Opened from
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        from: i32,
        /// Page
        #[libertas_constrained_by("$.navigation.groups")]
        #[libertas_copy_from("$.suggested_destination")]
        section: NavigationDestination,
        /// Available choices
        #[libertas_hidden]
        #[libertas_copy_from("$.navigation")]
        navigation: NavigationAvailability,
    },
    // The link-only target of every hub and review row (M3-N1): it has no input,
    // so a tap on a row sends it at once with the tapped row's section. When
    // no row is bound, CopyFrom falls back to the service's suggestion.
    /// Open the suggestion
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    #[libertas_link_only]
    OpenSection {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Opened from
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        from: i32,
        /// Section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<ReviewItem>.section,$.suggested")]
        section: TaxSection,
    },
    // The refresh request of the hub's master-detail arrangement
    // (display.json): after a topic is saved in the trailing pane, the client
    // builds it from the hub as shown and the service answers with the hub as
    // it is now. It only navigates: it is admitted by identity, stores
    // nothing, and is never offered as an action.
    /// Reload the topics
    #[libertas_request]
    #[libertas_next_response("Hub,Problem")]
    #[libertas_exclude_ui]
    OpenHub {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.review")]
        review: bool,
    },
    // The task offers this section's Continue until the whole walkthrough is
    // complete, then Review. Link-only topic and previous-section rows stay
    // available independently of Actions and the secondary Go to control.
    /// Topics
    #[libertas_response]
    #[libertas_ai_navigation_prompt("TAX_NAVIGATION_PROMPT")]
    #[libertas_next_request(
        "ContinueIncome,ContinueAdjustments,ContinueItemized,ContinueCredits,ReviewAccepted,Navigate,OpenSection"
    )]
    #[libertas_constrained_by("$.allowed_actions")]
    Hub {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        estimate: Vec<u8>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// What you told us
        /// The checklist's answer for this section; a tap opens the checklist.
        #[libertas_read_only]
        told: ReviewItem,
        /// Section topics
        #[libertas_read_only]
        topics: HubTopics,
        /// Previous section
        /// Open the preceding section without submitting an open editor.
        #[libertas_read_only]
        previous_section: ReviewItem,
        /// Available actions
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_actions: Vec<i32>,
        /// Suggested next step
        #[libertas_hidden]
        #[libertas_read_only]
        suggested: TaxSection,
        /// Available steps
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_sections: Vec<i32>,
        /// Continue or review
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_action_control("Actions")]
        primary_actions: Vec<i32>,
        /// Go to
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_action_control("GoTo")]
        go_to_actions: Vec<i32>,
        /// Available navigation choices
        #[libertas_hidden]
        #[libertas_read_only]
        navigation: NavigationAvailability,
        /// Suggested destination
        #[libertas_hidden]
        #[libertas_read_only]
        suggested_destination: NavigationDestination,
    },
    // Finish is offered only when nothing remains (M3-P16): the response's
    // allowed actions hold StartFinish only with an empty remaining list.
    // Navigate and link-only OpenSection are always allowed. OpenInterview
    // is offered only when it opens an unfinished interview step. The review
    // remains visible when Go to is its sole menu action (M3-N1).
    /// Review your federal estimate
    #[libertas_response]
    #[libertas_ai_navigation_prompt("TAX_NAVIGATION_PROMPT")]
    #[libertas_next_request("Navigate,OpenSection,StartFinish,OpenInterview")]
    #[libertas_constrained_by("$.allowed_actions")]
    Review {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        estimate: Vec<u8>,
        /// Remaining items
        /// Fix these before finishing. Tap an item to open its page.
        /// ----
        /// Item
        #[libertas_suppress_label]
        #[libertas_read_only]
        remaining_items: Vec<ReviewItem>,
        /// Estimate notes you can update
        /// These assumptions and items left out do not prevent finishing. Tap a note to review its answers.
        /// ----
        /// Note
        #[libertas_suppress_label]
        #[libertas_read_only]
        correction_notes: Vec<ReviewItem>,
        /// Your answers
        /// Tap a section to open it.
        /// ----
        /// Item
        #[libertas_suppress_label]
        answers: Vec<ReviewItem>,
        /// Federal estimate and coverage
        result: FederalEstimate,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Suggested next step
        #[libertas_hidden]
        #[libertas_read_only]
        suggested: TaxSection,
        /// Available sections
        #[libertas_hidden]
        allowed_sections: Vec<i32>,
        /// Available actions
        #[libertas_hidden]
        allowed_actions: Vec<i32>,
        /// Resume or finish
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_action_control("Actions")]
        primary_actions: Vec<i32>,
        /// Go to
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_action_control("GoTo")]
        go_to_actions: Vec<i32>,
        /// Available navigation choices
        #[libertas_hidden]
        #[libertas_read_only]
        navigation: NavigationAvailability,
        /// Suggested destination
        #[libertas_hidden]
        #[libertas_read_only]
        suggested_destination: NavigationDestination,
    },
    /// Finish review
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    StartFinish {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
    },
    /// Finish prototype review
    #[libertas_response]
    #[libertas_workflow_request("Finish")]
    BeginFinish {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
    },
    /// Finish prototype review
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    Finish {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
    },
    /// Prototype review completed
    #[libertas_response]
    #[libertas_next_request("ReviewAccepted")]
    Finished {
        /// Completion status
        #[libertas_formatted_text]
        #[libertas_read_only]
        message: Vec<u8>,
        /// Federal estimate
        result: FederalEstimate,
    },
    /// Please review your answer
    #[libertas_response]
    #[libertas_error]
    Problem {
        /// What needs attention
        #[libertas_formatted_text]
        #[libertas_read_only]
        message: Vec<u8>,
        /// Native source field needing attention
        /// [NullText]
        /// No source field has been identified
        /// [SetText]
        /// Source field needing attention
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_error_field_path]
        field: Option<String>,
    },
    /// Applicable property answers for this document
    #[libertas_request]
    #[libertas_next_response("DocumentPropertyChoices")]
    DocumentPropertyOptions {
        /// Empty construction choices until actual rental dates are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_rental_recovery_choices")]
        initial_rental_recovery_choices: Vec<i32>,
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// Actual ownership answers for retained Form 8960 and absorbed carryback sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_retained_owners")]
        allowed_injured_retained_owners: Vec<i32>,
        /// Existing long-term-care statements for actual period payments
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_payee_sources")]
        ltc_payee_sources: Vec<RecordChoice>,

        /// S-corporation K-1 records that may have paid a W-2
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.wage_scorporations")]
        wage_scorporations: Vec<RecordChoice>,
        /// Available S-corporation employer relation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_wage_scorporations")]
        allowed_wage_scorporations: Vec<i32>,
        /// Initial automatic investment-confirmation choices for a fresh K-1 row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_k1_investment_choices")]
        initial_k1_investment_choices: Vec<i32>,
        /// Actual current businesses available for retained research attribution
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Applicable retained-source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Available k1 credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.k1_credit_lines")]
        k1_credit_lines: Vec<i32>,
        /// Available research credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_credit_lines")]
        research_credit_lines: Vec<i32>,
        /// Available renewable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.renewable_credit_lines")]
        renewable_credit_lines: Vec<i32>,
        /// Available transferable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.transferable_credit_lines")]
        transferable_credit_lines: Vec<i32>,
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available at-risk histories for ownership begun before 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_continuing_risk")]
        allowed_continuing_risk: Vec<i32>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,
        /// Farms and farm rentals on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<RecordChoice>,
        /// Businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<RecordChoice>,
        /// Schedule C and Schedule F businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.qbi_businesses")]
        qbi_businesses: Vec<RecordChoice>,
        /// Rental properties on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rentals")]
        rentals: Vec<RecordChoice>,
        /// Saved canonical mortgage loans for home rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Saved physical buildings for section 179D cost-source claims
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rental_energy_buildings")]
        rental_energy_buildings: Vec<RecordChoice>,
        /// Current accepted farm source catalog: basis_assets
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.basis_assets")]
        basis_assets: Vec<FarmChoice>,
        /// Current accepted farm source catalog: farm_sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_sources")]
        farm_sources: Vec<FarmChoice>,
        /// Foreign countries for the separate FBAR reporting answer
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.foreign_countries")]
        foreign_countries: Vec<RecordChoice>,
        /// Accepted interest statements and annual instruments for education bonds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.interest_statements")]
        interest_statements: Vec<RecordChoice>,
        /// Royalty entries on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.royalty_properties")]
        royalty_properties: Vec<RecordChoice>,
        /// Equipment rented out on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.equipment_rentals")]
        equipment_rentals: Vec<RecordChoice>,
        /// Hobbies on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hobbies")]
        hobbies: Vec<RecordChoice>,
        /// Commodity Credit Corporation loans forfeited in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ccc_lots")]
        ccc_lots: Vec<RecordChoice>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_iras")]
        inherited_iras: Vec<RecordChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_roth_iras")]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Long-term care periods records on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_insured")]
        ltc_insured: Vec<RecordChoice>,
        /// Allowed business income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_business_source")]
        allowed_business_source: Vec<i32>,
        /// Allowed farm income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farm_source")]
        allowed_farm_source: Vec<i32>,
        /// Allowed partnership income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_partner_source")]
        allowed_partner_source: Vec<i32>,
        /// Allowed home sale spouse answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_home_spouse")]
        allowed_home_spouse: Vec<i32>,
        /// Allowed answers about the return reporting a child's income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_other_parent")]
        allowed_other_parent: Vec<i32>,
        /// Children who can be reported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.children")]
        children: Vec<crate::ChildChoice>,
        /// Current principal agricultural codes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_codes")]
        farm_codes: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_limitations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_limitations")]
        farm_limitations: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_method_changes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_method_changes")]
        farm_method_changes: Vec<FarmChoice>,
        /// Actual nonqualified annuity sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.annuity_sources")]
        annuity_sources: Vec<crate::LtcSourceChoice>,
        /// Actual prior-year deduction recovery sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.recovery_sources")]
        recovery_sources: Vec<crate::LtcSourceChoice>,
        /// Accepted same-owner business, farm and farm-rental recipients
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_activities")]
        cooperative_activities: Vec<RecordChoice>,
        /// Accepted property-owner links; unavailable owners supply no choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_properties")]
        cooperative_properties: Vec<RecordChoice>,
        /// Document
        #[libertas_copy_from("^<IncomeDocumentItem>.document")]
        document: IncomeDocument,
    },
    /// Applicable property answers for the bound row
    #[libertas_response]
    DocumentPropertyChoices {
        /// Allowed community-property answers
        #[libertas_copy_to("^<IncomeDocumentItem>.allowed_community")]
        allowed_community: Vec<i32>,
    },
    /// Calculate document preview
    #[libertas_request]
    #[libertas_next_response("IncomePreview,Problem")]
    PreviewDocument {
        /// Empty construction choices until actual rental dates are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_rental_recovery_choices")]
        initial_rental_recovery_choices: Vec<i32>,
        /// Empty construction choices for a new K-1 source; accepted parent choices come with its supplied editor value.
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.empty_k1_activity_choices")]
        empty_k1_activity_choices: alloc::boxed::Box<crate::K1ActivityChoiceContext>,
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// Actual ownership answers for retained Form 8960 and absorbed carryback sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_retained_owners")]
        allowed_injured_retained_owners: Vec<i32>,
        /// Existing long-term-care statements for actual period payments
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_payee_sources")]
        ltc_payee_sources: Vec<RecordChoice>,

        /// S-corporation K-1 records that may have paid a W-2
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.wage_scorporations")]
        wage_scorporations: Vec<RecordChoice>,
        /// Available S-corporation employer relation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_wage_scorporations")]
        allowed_wage_scorporations: Vec<i32>,
        /// Initial automatic investment-confirmation choices for a fresh K-1 row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_k1_investment_choices")]
        initial_k1_investment_choices: Vec<i32>,
        /// Actual current businesses available for retained research attribution
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Applicable retained-source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Available k1 credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.k1_credit_lines")]
        k1_credit_lines: Vec<i32>,
        /// Available research credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_credit_lines")]
        research_credit_lines: Vec<i32>,
        /// Available renewable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.renewable_credit_lines")]
        renewable_credit_lines: Vec<i32>,
        /// Available transferable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.transferable_credit_lines")]
        transferable_credit_lines: Vec<i32>,
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available at-risk histories for ownership begun before 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_continuing_risk")]
        allowed_continuing_risk: Vec<i32>,
        /// Accepted record identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<IncomeDocumentItem>.id")]
        id: i64,
        /// Whose document
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<IncomeDocumentItem>.owner")]
        owner: Owner,
        /// Community or separate
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<IncomeDocumentItem>.community")]
        community: CommunityShare,
        /// Foreign countries for the separate FBAR reporting answer
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.foreign_countries")]
        foreign_countries: Vec<RecordChoice>,
        /// Accepted interest statements and annual instruments for education bonds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.interest_statements")]
        interest_statements: Vec<RecordChoice>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,
        /// Farms and farm rentals on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<RecordChoice>,
        /// Accepted same-owner business, farm and farm-rental recipients
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_activities")]
        cooperative_activities: Vec<RecordChoice>,
        /// Accepted property-owner links; unavailable owners supply no choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_properties")]
        cooperative_properties: Vec<RecordChoice>,
        /// Businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<RecordChoice>,
        /// Schedule C and Schedule F businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.qbi_businesses")]
        qbi_businesses: Vec<RecordChoice>,
        /// Rental properties on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rentals")]
        rentals: Vec<RecordChoice>,
        /// Saved canonical mortgage loans for home rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Saved physical buildings for section 179D cost-source claims
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rental_energy_buildings")]
        rental_energy_buildings: Vec<RecordChoice>,
        /// Royalty entries on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.royalty_properties")]
        royalty_properties: Vec<RecordChoice>,
        /// Equipment rented out on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.equipment_rentals")]
        equipment_rentals: Vec<RecordChoice>,
        /// Hobbies on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hobbies")]
        hobbies: Vec<RecordChoice>,
        /// Commodity Credit Corporation loans forfeited in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ccc_lots")]
        ccc_lots: Vec<RecordChoice>,
        /// Current principal agricultural codes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_codes")]
        farm_codes: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_limitations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_limitations")]
        farm_limitations: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_sources")]
        farm_sources: Vec<FarmChoice>,
        /// Current accepted farm source catalog: basis_assets
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.basis_assets")]
        basis_assets: Vec<FarmChoice>,
        /// Current accepted farm source catalog: farm_method_changes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_method_changes")]
        farm_method_changes: Vec<FarmChoice>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_iras")]
        inherited_iras: Vec<RecordChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_roth_iras")]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Long-term care periods records on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_insured")]
        ltc_insured: Vec<RecordChoice>,
        /// Actual nonqualified annuity sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.annuity_sources")]
        annuity_sources: Vec<crate::LtcSourceChoice>,
        /// Actual prior-year deduction recovery sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.recovery_sources")]
        recovery_sources: Vec<crate::LtcSourceChoice>,
        /// Allowed business income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_business_source")]
        allowed_business_source: Vec<i32>,
        /// Allowed farm income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farm_source")]
        allowed_farm_source: Vec<i32>,
        /// Allowed partnership income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_partner_source")]
        allowed_partner_source: Vec<i32>,
        /// Allowed home sale spouse answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_home_spouse")]
        allowed_home_spouse: Vec<i32>,
        /// Allowed answers about the return reporting a child's income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_other_parent")]
        allowed_other_parent: Vec<i32>,
        /// Children who can be reported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.children")]
        children: Vec<crate::ChildChoice>,
        /// Document
        #[libertas_copy_from("^<IncomeDocumentItem>.document")]
        document: IncomeDocument,
    },
    /// Document preview
    #[libertas_response]
    IncomePreview {
        /// Calculated carryover bounds from the edited source values
        #[libertas_copy_to("^<IncomeDocumentItem>.investment_caps")]
        investment_caps: Option<crate::InvestmentCapitalCap>,
        /// Document amount before return-level calculations
        #[libertas_money("USD", 2)]
        #[libertas_copy_to("^<IncomeDocumentItem>.preview")]
        preview: Option<i64>,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateSetup {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Your 2026 return
        #[libertas_copy_from("$.value")]
        value: ReturnSetup,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateYou {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.professional_election")]
        professional_election: crate::ProfessionalRentalElection,
        /// Current editor's existing rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_professional_elections")]
        allowed_professional_elections: Vec<i32>,
        /// About you
        #[libertas_copy_from("$.you")]
        you: Filer,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateSpouse {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.professional_election")]
        professional_election: crate::ProfessionalRentalElection,
        /// Current editor's existing rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_professional_elections")]
        allowed_professional_elections: Vec<i32>,
        /// About your spouse
        #[libertas_copy_from("$.spouse")]
        spouse: SpouseAnswers,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("IncomeRunningEstimate,Problem")]
    EstimateDocuments {
        /// Empty construction choices until actual rental dates are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_rental_recovery_choices")]
        initial_rental_recovery_choices: Vec<i32>,
        /// Empty construction choices for a new K-1 source; accepted parent choices come with its supplied editor value.
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.empty_k1_activity_choices")]
        empty_k1_activity_choices: alloc::boxed::Box<crate::K1ActivityChoiceContext>,
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// Actual ownership answers for retained Form 8960 and absorbed carryback sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_retained_owners")]
        allowed_injured_retained_owners: Vec<i32>,
        /// Existing long-term-care statements for actual period payments
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_payee_sources")]
        ltc_payee_sources: Vec<RecordChoice>,

        /// S-corporation K-1 records that may have paid a W-2
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.wage_scorporations")]
        wage_scorporations: Vec<RecordChoice>,
        /// Available S-corporation employer relation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_wage_scorporations")]
        allowed_wage_scorporations: Vec<i32>,
        /// Initial automatic investment-confirmation choices for a fresh K-1 row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_k1_investment_choices")]
        initial_k1_investment_choices: Vec<i32>,
        /// Actual current businesses available for retained research attribution
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Applicable retained-source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Available k1 credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.k1_credit_lines")]
        k1_credit_lines: Vec<i32>,
        /// Available research credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_credit_lines")]
        research_credit_lines: Vec<i32>,
        /// Available renewable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.renewable_credit_lines")]
        renewable_credit_lines: Vec<i32>,
        /// Available transferable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.transferable_credit_lines")]
        transferable_credit_lines: Vec<i32>,
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available at-risk histories for ownership begun before 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_continuing_risk")]
        allowed_continuing_risk: Vec<i32>,
        /// Topic
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.topic")]
        topic: TaxSection,
        /// Allowed document kinds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_documents")]
        allowed_documents: Vec<i32>,
        /// Allowed document owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Allowed community-property answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_community")]
        allowed_community: Vec<i32>,
        /// Allowed management of community funds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_refund_management")]
        allowed_refund_management: Vec<i32>,
        /// Foreign countries for the separate FBAR reporting answer
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.foreign_countries")]
        foreign_countries: Vec<RecordChoice>,
        /// Accepted interest statements and annual instruments for education bonds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.interest_statements")]
        interest_statements: Vec<RecordChoice>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,
        /// Farms and farm rentals on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<RecordChoice>,
        /// Accepted same-owner business, farm and farm-rental recipients
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_activities")]
        cooperative_activities: Vec<RecordChoice>,
        /// Accepted property-owner links; unavailable owners supply no choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_properties")]
        cooperative_properties: Vec<RecordChoice>,
        /// Businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<RecordChoice>,
        /// Schedule C and Schedule F businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.qbi_businesses")]
        qbi_businesses: Vec<RecordChoice>,
        /// Rental properties on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rentals")]
        rentals: Vec<RecordChoice>,
        /// Saved canonical mortgage loans for home rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Saved physical buildings for section 179D cost-source claims
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rental_energy_buildings")]
        rental_energy_buildings: Vec<RecordChoice>,
        /// Royalty entries on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.royalty_properties")]
        royalty_properties: Vec<RecordChoice>,
        /// Equipment rented out on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.equipment_rentals")]
        equipment_rentals: Vec<RecordChoice>,
        /// Hobbies on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hobbies")]
        hobbies: Vec<RecordChoice>,
        /// Commodity Credit Corporation loans forfeited in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ccc_lots")]
        ccc_lots: Vec<RecordChoice>,
        /// Current principal agricultural codes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_codes")]
        farm_codes: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_limitations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_limitations")]
        farm_limitations: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_sources")]
        farm_sources: Vec<FarmChoice>,
        /// Current accepted farm source catalog: basis_assets
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.basis_assets")]
        basis_assets: Vec<FarmChoice>,
        /// Current accepted farm source catalog: farm_method_changes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farm_method_changes")]
        farm_method_changes: Vec<FarmChoice>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_iras")]
        inherited_iras: Vec<RecordChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_roth_iras")]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Long-term care periods records on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_insured")]
        ltc_insured: Vec<RecordChoice>,
        /// Actual nonqualified annuity sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.annuity_sources")]
        annuity_sources: Vec<crate::LtcSourceChoice>,
        /// Actual prior-year deduction recovery sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.recovery_sources")]
        recovery_sources: Vec<crate::LtcSourceChoice>,
        /// Allowed business income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_business_source")]
        allowed_business_source: Vec<i32>,
        /// Allowed farm income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farm_source")]
        allowed_farm_source: Vec<i32>,
        /// Allowed partnership income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_partner_source")]
        allowed_partner_source: Vec<i32>,
        /// Allowed home sale spouse answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_home_spouse")]
        allowed_home_spouse: Vec<i32>,
        /// Allowed answers about the return reporting a child's income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_other_parent")]
        allowed_other_parent: Vec<i32>,
        /// Children who can be reported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.children")]
        children: Vec<crate::ChildChoice>,
        /// Income documents
        #[libertas_copy_from("$.documents")]
        documents: crate::IncomeDocumentsEstimateInput,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateSituations {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Topic
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.topic")]
        topic: TaxSection,
        /// Allowed situations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_situations")]
        allowed_situations: Vec<i32>,
        /// Less common situations
        #[libertas_copy_from("$.value")]
        value: Situations,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateDependents {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed residence answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_lived")]
        allowed_lived: Vec<i32>,
        /// Days you were alive in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.days_alive")]
        days_alive: u32,
        /// Children and people you support
        #[libertas_copy_from("$.dependents")]
        dependents: Dependents,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateHsa {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed account owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        // The page's accounts carry HsaOptions wherever they appear, so this
        // request's copy of the owners is its input too.
        #[libertas_calculation_request("HsaOptions")]
        allowed_owners: Vec<i32>,
        /// Other-filer HSA eligibility applies on this married return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.other_coverage_allowed")]
        other_coverage_allowed: bool,
        /// Allowed family contribution splits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_family_split")]
        allowed_family_split: Vec<i32>,
        /// Actual failed cash deposits into HSA accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hsa_failed_deposits")]
        hsa_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual failed cash deposits into ARCHER accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.archer_failed_deposits")]
        archer_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual accepted HSA corrective statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hsa_return_documents")]
        hsa_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// Actual accepted Archer corrective statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.archer_return_documents")]
        archer_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// IRA distributions on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ira_sources")]
        ira_sources: Vec<crate::HsaFundingSourceChoice>,
        /// Earned compensation on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.archer_income")]
        archer_income: Vec<RecordChoice>,
        /// Health and medical savings accounts
        #[libertas_copy_from("$.value")]
        value: HsaPage,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateSelfEmployedBenefits {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Accepted Marketplace source statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_policies")]
        ptc_policies: Vec<RecordChoice>,
        /// Current businesses and entities
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.health_activities")]
        health_activities: Vec<RecordChoice>,
        /// Current people available to insure
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.health_people")]
        health_people: Vec<PersonChoice>,
        /// Existing wage statements for shareholder plan linkage
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.health_wages")]
        health_wages: Vec<RecordChoice>,
        /// Self-employed health insurance and retirement plan
        #[libertas_copy_from("$.value")]
        value: SelfEmployedBenefits,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateIraContributions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed contributors
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// IRA contributions
        #[libertas_copy_from("$.value")]
        value: IraContributions,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateEducator {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed educators
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Educator expenses
        #[libertas_copy_from("$.value")]
        value: EducatorExpenses,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateStudentLoan {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Student loan interest
        #[libertas_copy_from("$.value")]
        value: StudentLoanInterest,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateEmployeeExpenses {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Reservist, performing artist and fee-basis official expenses
        #[libertas_copy_from("$.value")]
        value: EmployeeExpenses,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateDeductions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available deduction routes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_routes")]
        allowed_routes: Vec<i32>,
        /// Available repayment answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_repayment")]
        allowed_repayment: Vec<i32>,
        /// Deductions
        #[libertas_copy_from("$.value")]
        value: Deductions,
        /// Saved business homes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.home_sites")]
        home_sites: Vec<RecordChoice>,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateMedical {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Medical and dental expenses
        #[libertas_copy_from("$.value")]
        value: Medical,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateTaxes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Taxes you paid
        #[libertas_copy_from("$.value")]
        value: Taxes,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateOtherItemized {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed deduction owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Investment borrowing sources on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.interest_sources")]
        interest_sources: Vec<crate::InvestmentSourceChoice>,
        /// Actual K-1 entities for a retained interest balance
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.investment_k1_sources")]
        investment_k1_sources: Vec<crate::RecordChoice>,
        /// Applicable retained-interest origin question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.investment_origin_choices")]
        investment_origin_choices: Vec<i32>,
        /// Investment interest and other deductions
        #[libertas_copy_from("$.value")]
        value: OtherItemized,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateMortgages {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Loans currently selected by rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rental_mortgage_loans")]
        rental_mortgage_loans: Vec<i64>,
        /// Home mortgage interest
        #[libertas_copy_from("$.value")]
        value: Mortgages,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateHomeOffices {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Saved businesses and farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.home_activities")]
        home_activities: Vec<RecordChoice>,
        /// Saved casualty events
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.casualty_events")]
        casualty_events: Vec<RecordChoice>,
        /// Allowed home owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_home_owners")]
        allowed_home_owners: Vec<i32>,
        /// Business use of your homes
        #[libertas_copy_from("$.value")]
        value: HomeOffices,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateGiftCarryovers {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed captains
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Actual current K-1 sources for retained activity gifts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_charity_k1s")]
        retained_charity_k1s: Vec<RecordChoice>,
        /// Actual retained-gift origin choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_charity_origins")]
        retained_charity_origins: Vec<i32>,
        /// Applicable retained-gift source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Gifts of property and carryovers
        #[libertas_copy_from("$.value")]
        value: GiftCarryovers,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateCare {
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// People already on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,

        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed filers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Filers eligible for monthly deemed income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.care_filers")]
        care_filers: Vec<i32>,
        /// Calendar months
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.care_months")]
        care_months: Vec<RecordChoice>,
        /// Businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<CareSourceChoice>,
        /// Farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<CareSourceChoice>,
        /// Partnership K-1s
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.partnerships")]
        partnerships: Vec<CareSourceChoice>,
        /// Child and dependent care
        #[libertas_copy_from("$.value")]
        value: CarePage,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateEducation {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Education credits
        #[libertas_copy_from("$.value")]
        value: EducationCredits,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateSaver {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Retirement savings credit
        #[libertas_copy_from("$.value")]
        value: SaverCredit,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateMarketplace {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// People supported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,
        /// Current people and saved external enrollees
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_people")]
        ptc_people: Vec<RecordChoice>,
        /// Saved other tax families
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_families")]
        ptc_families: Vec<RecordChoice>,
        /// Saved policy sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_policies")]
        ptc_policies: Vec<RecordChoice>,
        /// Accepted W-2 code FF sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_w2")]
        ptc_w2: Vec<RecordChoice>,
        /// Employees on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_employees")]
        ptc_employees: Vec<RecordChoice>,
        /// Current people outside the calculated tax family
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_nonreturn_people")]
        ptc_nonreturn_people: Vec<RecordChoice>,
        /// Claimed dependents on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_claimed_dependents")]
        ptc_claimed_dependents: Vec<RecordChoice>,
        /// Available marriage calculation choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_marriage")]
        allowed_marriage: Vec<i32>,
        /// Available separate-return answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_separate")]
        allowed_separate: Vec<i32>,
        /// Health insurance marketplace
        #[libertas_copy_from("$.value")]
        value: MarketplaceFactsPage,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateMoreCredits {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Businesses run abroad on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.foreign_businesses")]
        foreign_businesses: Vec<RecordChoice>,
        /// Investment sales on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.capital_sales")]
        capital_sales: Vec<RecordChoice>,
        /// Foreign tax situations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.foreign_situations")]
        foreign_situations: Vec<ForeignSituationChoice>,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Available businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<RecordChoice>,
        /// Available farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<RecordChoice>,
        /// Saved clean vehicle property choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.clean_vehicle_properties")]
        clean_vehicle_properties: Vec<RecordChoice>,
        /// Available rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rentals")]
        rentals: Vec<RecordChoice>,
        /// Available research businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Available entered credits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.entered_credits")]
        entered_credits: Vec<i32>,
        /// Available carryback credits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.carryback_credits")]
        carryback_credits: Vec<i32>,
        /// Available allowed carry filers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_carry_filers")]
        allowed_carry_filers: Vec<i32>,
        /// Available initial credit elections
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_credit_elections")]
        initial_credit_elections: Vec<i32>,
        /// Available initial research choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_research_choices")]
        initial_research_choices: Vec<i32>,
        /// More credits
        #[libertas_copy_from("^<SaveMoreCredits>.value")]
        value: MoreCredits,
        /// Initial equipment-origin choices for a fresh credit row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_carbon_oxide_origins")]
        initial_carbon_oxide_origins: Vec<i32>,
        /// Initial applicability for a newly added renewable-facility source row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_renewable_facility_origins")]
        #[libertas_output_only]
        initial_renewable_facility_origins: Vec<i32>,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateOtherTaxes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Allowed owners of a joint subsidized-home gift
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_gift_owners")]
        mortgage_gift_owners: Vec<i32>,
        /// Existing current-year Coverdell correction sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.coverdell_corrections")]
        coverdell_corrections: Vec<RecordChoice>,
        /// Other taxes
        #[libertas_copy_from("$.value")]
        value: OtherTaxes,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateMoreAdjustments {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// More adjustments
        #[libertas_copy_from("$.value")]
        value: MoreAdjustments,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("PaymentsEstimate,Problem")]
    EstimatePayments {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available 2025 return choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_prior_returns")]
        allowed_prior_returns: Vec<i32>,
        /// Available farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farmer")]
        allowed_farmer: Vec<i32>,
        /// Available 2027 farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farmer_next_year")]
        allowed_farmer_next_year: Vec<i32>,
        /// Allowed management of community funds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_refund_management")]
        allowed_refund_management: Vec<i32>,
        /// Federal payments
        #[libertas_copy_from("^<SavePayments>.value")]
        value: crate::PaymentsEstimateInput,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateIncomeByPeriod {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Income by period
        #[libertas_copy_from("$.value")]
        value: IncomeByPeriod,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateInjuredSpouse {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Injured spouse allocation (Form 8379)
        #[libertas_copy_from("$.value")]
        value: InjuredSpouse,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateParentTax {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Your parent's return (Form 8615)
        #[libertas_copy_from("$.value")]
        value: ParentReturn,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateChecklist {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Your 2026 at a glance
        #[libertas_copy_from("$.value")]
        value: Checklist,
    },
    /// Running estimate
    #[libertas_response]
    RunningEstimate {
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_copy_to("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_copy_to("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_copy_to("$.progress")]
        progress: Vec<u8>,
    },

    /// Calculate the successful Marketplace appeal payment deadline
    #[libertas_request]
    #[libertas_next_response("MarketplaceAppealDeadline,Problem")]
    CalculateMarketplaceAppeal {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Appeal decision date
        #[libertas_date_only]
        #[libertas_copy_from("^<MarketplaceAppealPage>.determined")]
        determined: u32,
    },
    /// Successful Marketplace appeal payment deadline
    #[libertas_response]
    MarketplaceAppealDeadline {
        /// Applicable payment deadline
        #[libertas_date_only]
        #[libertas_read_only]
        #[libertas_copy_to("^<MarketplaceAppealPage>.required_deadline")]
        required_deadline: u32,
    },
    /// Calculate the family split choices
    #[libertas_request]
    #[libertas_next_response("HsaFamilyOptions,Problem")]
    HsaOptions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Actual failed cash deposits into HSA accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hsa_failed_deposits")]
        hsa_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual failed cash deposits into ARCHER accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.archer_failed_deposits")]
        archer_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual accepted HSA corrective statements used by account-row constraints
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hsa_return_documents")]
        hsa_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// Actual accepted Archer corrective statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.archer_return_documents")]
        archer_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// Allowed account owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Accounts
        #[libertas_copy_from("$.value.accounts")]
        accounts: Vec<HsaAccount>,
        /// Other filer's actual eligibility context
        /// [NullText]
        /// No separate other-filer HSA eligibility context is needed
        /// [SetText]
        /// The other filer's HSA eligibility is
        #[libertas_copy_from("$.value.other_coverage")]
        #[libertas_default($null)]
        other_coverage: Option<crate::HsaEligibility>,
        /// Contribution-limit method
        #[libertas_copy_from("$.value.family_method")]
        family_method: crate::HsaAllocationMethod,
    },
    /// Family split choices
    #[libertas_response]
    HsaFamilyOptions {
        /// Allowed family contribution splits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("$.allowed_family_split")]
        allowed_family_split: Vec<i32>,
    },
    /// Open tax return
    #[libertas_request]
    #[libertas_default_request]
    #[libertas_next_response("Actions,Problem")]
    Enter {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
    },
    /// Tax interview actions
    #[libertas_response]
    #[libertas_next_request("OpenInterview")]
    #[libertas_constrained_by("$.allowed_actions")]
    Actions {
        /// Available actions
        #[libertas_hidden]
        allowed_actions: Vec<i32>,
    },
    /// Review
    /// Review accepted answers and the estimate.
    #[libertas_request]
    #[libertas_next_response("Review,Problem")]
    ReviewAccepted {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
    },
    /// Applicable originating-entity question for retained investment interest
    #[libertas_request]
    #[libertas_next_response("InvestmentOriginChoices,Problem")]
    InvestmentOriginOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Existing deduction destination
        #[libertas_copy_from("^.source")]
        source: i64,
        /// Retained regular balance
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("^.carry")]
        carry: i64,
        /// Retained AMT balance
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("^.amt_carry")]
        amt_carry: i64,
    },
    /// Applicable originating-entity choices
    #[libertas_response]
    InvestmentOriginChoices {
        /// Allowed originating-entity question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_origin_choices")]
        allowed_origin_choices: Vec<i32>,
    },
    /// Applicable choices for this credit
    #[libertas_request]
    #[libertas_next_response("BusinessCreditElectionChoices,Problem")]
    BusinessCreditElectionOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Credit
        #[libertas_copy_from("^.credit")]
        credit: crate::BusinessCredit,
    },
    /// Applicable credit choices
    #[libertas_response]
    BusinessCreditElectionChoices {
        /// Allowed choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_elections")]
        allowed_elections: Vec<i32>,
        /// Applicable carbon-capture source detail
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_carbon_oxide_origins")]
        allowed_carbon_oxide_origins: Vec<i32>,
        /// Applicable renewable-facility source detail
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_renewable_facility_origins")]
        allowed_renewable_facility_origins: Vec<i32>,
    },
    /// Applicable physical-source detail for this actual K-1 credit code
    #[libertas_request]
    #[libertas_next_response("CarbonOxideOriginChoices,Problem")]
    K1CarbonOxideOriginOptions {
        /// Actual current-return credit-line catalog
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.k1_credit_lines")]
        k1_credit_lines: Vec<i32>,
        /// Actual current-return research credit-line catalog
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_credit_lines")]
        research_credit_lines: Vec<i32>,
        /// Actual current-return renewable credit-line catalog
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.renewable_credit_lines")]
        renewable_credit_lines: Vec<i32>,
        /// Actual current-return transferable credit-line catalog
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.transferable_credit_lines")]
        transferable_credit_lines: Vec<i32>,
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Actual code and reported line
        #[libertas_copy_from("^.code")]
        code: crate::K1CreditCodePage,
    },
    /// Applicable carbon-capture source detail
    #[libertas_response]
    CarbonOxideOriginChoices {
        /// Applicable source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_carbon_oxide_origins")]
        allowed_carbon_oxide_origins: Vec<i32>,
        /// Applicable renewable-facility source detail
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_renewable_facility_origins")]
        allowed_renewable_facility_origins: Vec<i32>,
    },
    /// Applicable choices for this credit
    #[libertas_request]
    #[libertas_next_response("ResearchCreditSourceChoices,Problem")]
    ResearchCreditSourceOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Credit
        #[libertas_copy_from("^.credit")]
        credit: crate::BusinessCredit,
    },
    /// Applicable credit choices
    #[libertas_response]
    ResearchCreditSourceChoices {
        /// Allowed choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_research_choices")]
        allowed_research_choices: Vec<i32>,
    },
    // A 1099-R row's treatment choices from its printed box 7a codes and
    // box 7b (retirement.rs DistributionCodes::leads_to), bound to the row:
    // box7 and ira invoke it, and the reply writes the row's hidden set.
    /// Calculate the box 7a treatments
    #[libertas_request]
    #[libertas_next_response("Box7Treatments,Problem")]
    Box7Options {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Box 7a distribution codes
        #[libertas_bitflags(DistributionCodes)]
        #[libertas_copy_from("^.box7")]
        box7: u32,
        /// Box 7b IRA/SEP/SIMPLE
        #[libertas_copy_from("^.ira")]
        ira: bool,
        /// Box 7c Trump account
        #[libertas_copy_from("^.trump")]
        trump: bool,
    },
    /// Box 7a treatments
    #[libertas_response]
    Box7Treatments {
        /// Allowed treatments
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_treatments")]
        allowed_treatments: Vec<i32>,
        /// Allowed methods
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_methods")]
        allowed_methods: Vec<i32>,
    },
    /// Continue
    /// Leave income and open the next section of the walkthrough.
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    ContinueIncome {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
    },
    /// Continue
    /// Leave adjustments and open the next section of the walkthrough.
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    ContinueAdjustments {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
    },
    /// Continue
    /// Leave itemized deductions and open the next section of the walkthrough.
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    ContinueItemized {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
    },
    /// Continue
    /// Leave credits and open the next section of the walkthrough.
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginSituations,SaveSituations,BeginChildren,SaveChildren,SaveIncomes,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,Hub,Review,BeginFinish,Finish,Finished,Problem"
    )]
    ContinueCredits {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
    },
    /// Applicable automatic Form3468 confirmation question
    #[libertas_request]
    #[libertas_next_response("K1InvestmentChoices,Problem")]
    K1InvestmentOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// The actual property rows
        #[libertas_copy_from("^.investment_property")]
        properties: Vec<crate::K1InvestmentProperty>,
    },
    /// Automatic Form3468 confirmation choices
    #[libertas_response]
    K1InvestmentChoices {
        /// Allowed automatic confirmation choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_investment_confirmations")]
        allowed_investment_confirmations: Vec<i32>,
    },
    /// Applicable retained regular-loss source question
    #[libertas_request]
    #[libertas_next_response("RetainedLossSourceChoices,Problem")]
    RetainedLossSourceOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// This loss year's retained regular balance
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("^.balance")]
        balance: i64,
        /// Accepted research-source applicability
        #[libertas_hidden]
        #[libertas_copy_from("$.retained_research_choices")]
        available: Vec<i32>,
    },
    /// Applicable retained regular-loss choices
    #[libertas_response]
    RetainedLossSourceChoices {
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_research_parts")]
        allowed_research_parts: Vec<i32>,
    },
    /// Applicable retained regular-capital source question
    #[libertas_request]
    #[libertas_next_response("RetainedCapitalSourceChoices,Problem")]
    RetainedCapitalSourceOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("^.agi")]
        agi: i64,
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("^.deductions")]
        deductions: i64,
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("^.senior_deduction")]
        senior_deduction: i64,
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("^.short")]
        short: i64,
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("^.long")]
        long: i64,
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("^.deducted_loss")]
        deducted_loss: i64,
        #[libertas_copy_from("^.ordinary_history")]
        ordinary_history: bool,
        /// Accepted research-source applicability
        #[libertas_hidden]
        #[libertas_copy_from("$.retained_research_choices")]
        available: Vec<i32>,
    },
    /// Applicable retained regular-capital choices
    #[libertas_response]
    RetainedCapitalSourceChoices {
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.allowed_research_parts")]
        allowed_research_parts: Vec<i32>,
    },
    /// Applicable retained charitable-gift appreciation questions
    #[libertas_request]
    #[libertas_next_response("RetainedCharityAppreciationChoices,Problem")]
    RetainedCharityAppreciationOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        #[libertas_copy_from("^.basis_election")]
        election: bool,
        #[libertas_copy_from("^.carryovers")]
        records: Vec<crate::CharityAppreciationFacts>,
        #[libertas_hidden]
        #[libertas_copy_from("$.retained_research_choices")]
        available: Vec<i32>,
    },
    /// Derived appreciation applicability for the same retained rows
    #[libertas_response]
    RetainedCharityAppreciationChoices {
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^.carryovers")]
        records: Vec<crate::CharityAppreciationChoices>,
    },
    /// Income estimate and current aggregation choices
    #[libertas_response]
    IncomeRunningEstimate {
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_copy_to("$.estimate")]
        estimate: Vec<u8>,
        /// What saving these edits would remove or reopen
        #[libertas_read_only]
        #[libertas_copy_to("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_copy_to("$.progress")]
        progress: Vec<u8>,
        /// Current group members and distinct business counts
        #[libertas_read_only]
        #[libertas_copy_to("$.documents.qbi_aggregations")]
        groups: Vec<crate::QbiAggregationDerived>,
        /// Businesses available for individual QBI aggregation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("$.qbi_sources")]
        sources: Vec<crate::QbiAggregationMemberChoice>,
        /// Actual native cash-source applicability in the current document order
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("$.documents.documents")]
        refund_management: Vec<crate::IncomeRefundManagementDerived>,
    },

    /// Federal payments
    #[libertas_response]
    #[libertas_workflow_request("SavePayments")]
    BeginPayments {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Available 2025 return choices
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_prior_returns: Vec<i32>,
        /// Available farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_farmer: Vec<i32>,
        /// Available 2027 farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_farmer_next_year: Vec<i32>,
        /// Allowed management of community funds
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_refund_management: Vec<i32>,
        /// Federal payments
        #[libertas_read_only]
        value: crate::federal_model::PaymentsInitial,
    },

    /// More credits
    #[libertas_response]
    #[libertas_workflow_request("SaveMoreCredits")]
    BeginMoreCredits {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        /// [NullText]
        /// Current estimate is unavailable while these answers need correction.
        /// [SetText]
        /// Current estimate
        estimate: Option<Vec<u8>>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        /// [NullText]
        /// Current interview progress is unavailable while these answers need correction.
        /// [SetText]
        /// Current interview progress
        progress: Option<Vec<u8>>,
        /// Available owners
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_owners: Vec<i32>,
        /// Businesses run abroad on this return
        #[libertas_hidden]
        #[libertas_read_only]
        foreign_businesses: Vec<RecordChoice>,
        /// Investment sales on this return
        #[libertas_hidden]
        #[libertas_read_only]
        capital_sales: Vec<RecordChoice>,
        /// Foreign tax situations
        #[libertas_hidden]
        #[libertas_read_only]
        foreign_situations: Vec<ForeignSituationChoice>,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        mortgage_loans: Vec<RecordChoice>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Available businesses
        #[libertas_hidden]
        #[libertas_read_only]
        businesses: Vec<RecordChoice>,
        /// Available farms
        #[libertas_hidden]
        #[libertas_read_only]
        farms: Vec<RecordChoice>,
        /// Saved clean vehicle property choices
        #[libertas_hidden]
        #[libertas_read_only]
        clean_vehicle_properties: Vec<RecordChoice>,
        /// Available rentals
        #[libertas_hidden]
        #[libertas_read_only]
        rentals: Vec<RecordChoice>,
        /// Available research businesses
        #[libertas_hidden]
        #[libertas_read_only]
        research_businesses: Vec<RecordChoice>,
        /// Available entered credits
        #[libertas_hidden]
        #[libertas_read_only]
        entered_credits: Vec<i32>,
        /// Available carryback credits
        #[libertas_hidden]
        #[libertas_read_only]
        carryback_credits: Vec<i32>,
        /// Available allowed carry filers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_carry_filers: Vec<i32>,
        /// Available initial credit elections
        #[libertas_hidden]
        #[libertas_read_only]
        initial_credit_elections: Vec<i32>,
        /// Available initial research choices
        #[libertas_hidden]
        #[libertas_read_only]
        initial_research_choices: Vec<i32>,
        /// More credits
        #[libertas_read_only]
        value: crate::federal_model::MoreCreditsInitial,
        /// Initial equipment-origin choices for a fresh credit row
        #[libertas_hidden]
        #[libertas_read_only]
        initial_carbon_oxide_origins: Vec<i32>,
        /// Initial applicability for a newly added renewable-facility source row
        #[libertas_hidden]
        #[libertas_read_only]
        initial_renewable_facility_origins: Vec<i32>,
    },
    /// Injured spouse allocation (Form 8379)
    #[libertas_response]
    #[libertas_workflow_request("SaveInjuredSpouse")]
    BeginInjuredSpouse {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Accepted allocation and question context
        #[libertas_read_only]
        value: InjuredSpouseInitial,
    },
    /// Applicable actual AMT section179 carry source for this property
    #[libertas_request]
    #[libertas_next_response("AssetAmtCarryChoices")]
    AssetAmtCarryOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Current source facts used only to decide whether a separate answer is needed
        #[libertas_copy_from("^<Asset>.history")]
        history: crate::AssetHistory,
    },
    /// Current applicable actual AMT source answer
    #[libertas_response]
    AssetAmtCarryChoices {
        #[libertas_copy_to("^<Asset>.allowed_amt_carry")]
        allowed: Vec<i32>,
    },

    /// Applicable actual AMT section179 carry source for this property
    #[libertas_request]
    #[libertas_next_response("VehicleAmtCarryChoices")]
    VehicleAmtCarryOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Current source facts used only to decide whether a separate answer is needed
        #[libertas_copy_from("^<VehicleMethod>.Actual.history")]
        history: crate::AssetHistory,
    },
    /// Current applicable actual AMT source answer
    #[libertas_response]
    VehicleAmtCarryChoices {
        #[libertas_copy_to("^<VehicleMethod>.Actual.allowed_amt_carry")]
        allowed: Vec<i32>,
    },

    /// Applicable actual AMT section179 carry source for this property
    #[libertas_request]
    #[libertas_next_response("CanonicalAssetAmtCarryChoices")]
    CanonicalAssetAmtCarryOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Current source facts used only to decide whether a separate answer is needed
        #[libertas_copy_from("^<CanonicalAssetPage>.financial")]
        financial: crate::BusinessAsset,
    },
    /// Current applicable actual AMT source answer
    #[libertas_response]
    CanonicalAssetAmtCarryChoices {
        #[libertas_copy_to("^<CanonicalAssetPage>.allowed_amt_carry")]
        allowed: Vec<i32>,
    },
    /// Running estimate
    #[libertas_response]
    PaymentsEstimate {
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_copy_to("$.estimate")]
        estimate: Vec<u8>,
        /// What saving the current edits would remove or reopen
        /// ----
        /// Change
        #[libertas_read_only]
        #[libertas_copy_to("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_copy_to("$.progress")]
        progress: Vec<u8>,
        /// Available overpayment before the planned credit
        #[libertas_money("USD", 2)]
        #[libertas_read_only]
        #[libertas_copy_to("$.value.credit_capacity")]
        credit_capacity: Option<i64>,
    },
    /// Applicable historical choices for the actual rental source dates
    #[libertas_request]
    #[libertas_next_response("RentalRecoveryChoices")]
    RentalRecoveryOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Actual property kind and source dates
        #[libertas_copy_from("^<Rental>")]
        value: crate::RentalRecoveryInputs,
    },
    /// Historical choices for the same current building and improvement rows
    #[libertas_response]
    RentalRecoveryChoices {
        /// Original building choices, without replacing its selected Rental document
        #[libertas_copy_to("^<Rental>.building")]
        building: crate::RentalRecoveryAllowedRow,
        /// Existing improvement choices, preserving source values and row membership
        #[libertas_copy_to("^<Rental>.more")]
        more: crate::RentalRecoveryMoreAllowed,
    },
}

impl TaxInterviewProtocol {
    /// The verified user carried by every request; None for a response.
    pub fn user(&self) -> Option<LibertasUser> {
        match self {
            Self::OpenInterview { user, .. }
            | Self::SaveSetup { user, .. }
            | Self::OpenHub { user, .. }
            | Self::ContinueIncome { user, .. }
            | Self::ContinueAdjustments { user, .. }
            | Self::ContinueItemized { user, .. }
            | Self::ContinueCredits { user, .. }
            | Self::SavePeople { user, .. }
            | Self::SaveSpouse { user, .. }
            | Self::SaveDeductions { user, .. }
            | Self::SaveSituations { user, .. }
            | Self::SaveChildren { user, .. }
            | Self::SaveIncomes { user, .. }
            | Self::SaveHsa { user, .. }
            | Self::SaveCare { user, .. }
            | Self::SaveSelfEmployedBenefits { user, .. }
            | Self::SaveIraContributions { user, .. }
            | Self::SaveEducator { user, .. }
            | Self::SaveStudentLoan { user, .. }
            | Self::SaveEmployeeExpenses { user, .. }
            | Self::SaveMedical { user, .. }
            | Self::SaveTaxes { user, .. }
            | Self::SaveOtherItemized { user, .. }
            | Self::SaveMortgages { user, .. }
            | Self::SaveHomeOffices { user, .. }
            | Self::SaveGiftCarryovers { user, .. }
            | Self::SaveEducation { user, .. }
            | Self::SaveSaver { user, .. }
            | Self::SaveMarketplace { user, .. }
            | Self::SaveMoreCredits { user, .. }
            | Self::SaveMoreAdjustments { user, .. }
            | Self::SaveOtherTaxes { user, .. }
            | Self::SavePayments { user, .. }
            | Self::SaveIncomeByPeriod { user, .. }
            | Self::SaveInjuredSpouse { user, .. }
            | Self::SaveParentTax { user, .. }
            | Self::SaveChecklist { user, .. }
            | Self::Back { user, .. }
            | Self::Navigate { user, .. }
            | Self::OpenSection { user, .. }
            | Self::StartFinish { user, .. }
            | Self::Finish { user, .. }
            | Self::AssetAmtCarryOptions { user, .. }
            | Self::VehicleAmtCarryOptions { user, .. }
            | Self::CanonicalAssetAmtCarryOptions { user, .. }
            | Self::RentalRecoveryOptions { user, .. }
            | Self::DocumentPropertyOptions { user, .. }
            | Self::PreviewDocument { user, .. }
            | Self::EstimateSetup { user, .. }
            | Self::EstimateYou { user, .. }
            | Self::EstimateSpouse { user, .. }
            | Self::EstimateDocuments { user, .. }
            | Self::EstimateSituations { user, .. }
            | Self::EstimateDependents { user, .. }
            | Self::EstimateHsa { user, .. }
            | Self::EstimateSelfEmployedBenefits { user, .. }
            | Self::EstimateIraContributions { user, .. }
            | Self::EstimateEducator { user, .. }
            | Self::EstimateStudentLoan { user, .. }
            | Self::EstimateEmployeeExpenses { user, .. }
            | Self::EstimateDeductions { user, .. }
            | Self::EstimateMedical { user, .. }
            | Self::EstimateTaxes { user, .. }
            | Self::EstimateOtherItemized { user, .. }
            | Self::EstimateMortgages { user, .. }
            | Self::EstimateHomeOffices { user, .. }
            | Self::EstimateGiftCarryovers { user, .. }
            | Self::EstimateCare { user, .. }
            | Self::EstimateEducation { user, .. }
            | Self::EstimateSaver { user, .. }
            | Self::EstimateMarketplace { user, .. }
            | Self::EstimateMoreCredits { user, .. }
            | Self::EstimateMoreAdjustments { user, .. }
            | Self::EstimateOtherTaxes { user, .. }
            | Self::EstimatePayments { user, .. }
            | Self::EstimateIncomeByPeriod { user, .. }
            | Self::EstimateInjuredSpouse { user, .. }
            | Self::EstimateParentTax { user, .. }
            | Self::EstimateChecklist { user, .. }
            | Self::HsaOptions { user, .. }
            | Self::CalculateMarketplaceAppeal { user, .. }
            | Self::BusinessCreditElectionOptions { user, .. }
            | Self::K1CarbonOxideOriginOptions { user, .. }
            | Self::ResearchCreditSourceOptions { user, .. }
            | Self::K1InvestmentOptions { user, .. }
            | Self::RetainedLossSourceOptions { user, .. }
            | Self::RetainedCharityAppreciationOptions { user, .. }
            | Self::RetainedCapitalSourceOptions { user, .. }
            | Self::InvestmentOriginOptions { user, .. }
            | Self::Box7Options { user, .. }
            | Self::Enter { user, .. }
            | Self::ReviewAccepted { user, .. } => Some(*user),
            _ => None,
        }
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
    /// ----
    /// Topic
    #[libertas_suppress_label]
    #[libertas_read_only]
    pub items: Vec<ReviewItem>,
}
// A hub or review row: a tap opens its section (M3-N1).
/// Item
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
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
