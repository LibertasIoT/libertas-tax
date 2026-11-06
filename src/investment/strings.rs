pub(crate) const INVESTMENT_STRINGS: &[(&str, &str)] = &[
    (
        "QSBS_ANSWER_NEEDED",
        "Answer the question about this stock sale to finish its section 1202 treatment.",
    ),
    (
        "QSBS_QUESTION_SPECIAL",
        "Did you acquire these shares other than for cash at original issue, or does a gift, inheritance, exchange, pass-through ownership, installment sale, rollover or another special treatment apply? Yes leaves this sale out of the estimate with a notice.",
    ),
    (
        "QSBS_QUESTION_ISSUER",
        "Was the issuer a qualified domestic C corporation at every issue date, with aggregate assets of the issuer, its predecessors and controlled group no more than $50 million immediately before and after issuance? Check issuer records and statutory contribution valuations. No taxes the whole gain without a section 1202 exclusion.",
    ),
    (
        "QSBS_QUESTION_ACTIVE",
        "Did at least 80% of asset value serve a qualified active business throughout substantially all of each holding period? Check issuer records, including subsidiaries, working capital and exceptions. Professional services, finance, investment, farming, mining and hospitality businesses do not qualify. No taxes the whole gain without a section 1202 exclusion.",
    ),
    (
        "QSBS_QUESTION_REDEMPTIONS",
        "Have you confirmed there was no disqualifying issuer redemption or other section 1202 exception? Check shareholder and significant-redemption windows around every issuance and any rule or election changing qualification, recognition or character. No taxes the whole gain without a section 1202 exclusion.",
    ),
    (
        "QSBS_QUESTION_HISTORY",
        "Have you confirmed this issuer's complete prior section 1202 history and all current-year sold lots? Include the taxpayer's and both spouses' separate and joint prior returns and every current-year disposition once. No taxes the whole gain without a section 1202 exclusion and adds a notice.",
    ),
    (
        "QSBS_QUESTION_PRIOR_GAIN",
        "How much eligible gain was already taken into account under section 1202 for this issuer before 2026? Reconcile prior returns, allocating earlier joint-return eligible gain equally to its spouses. Enter zero only after checking the complete history; do not enter prior proceeds or taxable gain.",
    ),
    (
        "QSBS_QUESTION_ZONE",
        "Is the additional 60% empowerment-zone exclusion established for these eligible lots? Check the issuer's retained zone certification and historical valuation. Without those records the increase remains unclaimed with a notice; an explicit No confirms the issuer did not qualify.",
    ),
    (
        "QSBS_QUESTION_ZONE_GAIN",
        "Enter the gain attributable to time through December 31, 2018 for this sold lot, using its historical valuation or an earlier cessation date and limiting it to this lot's realized gain. Enter gain, rather than proceeds or a second basis. No established valuation leaves the increase unclaimed with a notice.",
    ),
    (
        "BROKER_BARTER_RECEIPTS_NOT_CALCULATED",
        "These barter receipts need their actual income treatment. This estimate does not calculate that classification.",
    ),
    (
        "BROKER_CREATOR_RECEIPTS_NOT_CALCULATED",
        "These first-sale creator receipts need their actual income treatment. They are not capital-sale proceeds in this estimate.",
    ),
    (
        "BROKER_CONTRACTS_AGGREGATE_NEEDED",
        "Enter the actual box 11 aggregate from the statement. Boxes 8–10 cannot supply a missing printed amount.",
    ),
    (
        "BROKER_CONTRACTS_RECONCILE",
        "Reconcile boxes 8–10 with the actual printed box 11 aggregate.",
    ),
    (
        "BROKER_LOSS_STATEMENT_NEEDED",
        "The printed proceeds cannot establish this loss. Keep the broker’s separate statement for review.",
    ),
    (
        "BROKER_OPTIONAL_METHOD_CONFLICT",
        "The statement cannot report both optional-method classifications on the same form.",
    ),
    (
        "BROKER_ORDINARY_NOT_CALCULATED",
        "This statement reports ordinary treatment that this capital-sale calculation does not cover.",
    ),
    (
        "BROKER_PRINTED_PROCEEDS_CONFLICT",
        "The printed gross-proceeds and net-proceeds selections conflict. Check the actual statement.",
    ),
    (
        "BROKER_PRINTED_TERM_CONFLICT",
        "The printed short-term and long-term selections conflict. Check the actual statement.",
    ),
    (
        "BROKER_QOF_RECORDS_NEEDED",
        "This statement reports a qualified opportunity fund disposition. Keep its actual opportunity-fund records for review.",
    ),
    (
        "BROKER_SALE_BEFORE_ACQUIRED",
        "The reported sale date precedes the reported acquisition date.",
    ),
    (
        "BROKER_SALE_PROCEEDS_NEEDED",
        "Enter the actual proceeds from the statement; a missing amount is not zero.",
    ),
    (
        "BROKER_SOURCE_AMOUNT_NEEDED",
        "Enter the actual statement amounts; an empty form does not establish zero income.",
    ),
    (
        "BROKER_SALE_ARITHMETIC_UNAVAILABLE",
        "These actual sale amounts cannot be represented in this estimate. Keep the paper and records for review.",
    ),
    (
        "BROKER_WITHHOLDING_ARITHMETIC_UNAVAILABLE",
        "The actual annual withholding total cannot be represented in this estimate. Keep the source records for review.",
    ),
    (
        "BROKER_CONTRACTS_ARITHMETIC_UNAVAILABLE",
        "The actual contract totals cannot be represented in this estimate. Keep the source records for review.",
    ),
    (
        "BROKER_CORRECTION_PRINTED_BASIS_NEEDED",
        "A basis-correction code needs the basis actually printed by the broker. Review the source document or choose the applicable adjustment.",
    ),
    ("DISPLAY_BROKER_SALE", "Box 1 — Sale facts"),
    ("DISPLAY_BROKER_CLASSIFICATION", "Printed classifications and withholding"),
    ("DISPLAY_BROKER_CONTRACTS", "Boxes 8–13 — Contracts and other amounts"),
    ("DISPLAY_BROKER_OPTIONAL_METHOD", "Box 11 — Optional reporting"),
    ("DISPLAY_BROKER_TRANSFER", "Box 12 — Transfer-in facts"),
    ("DISPLAY_BROKER_STATE", "Boxes 14–16 — State withholding"),
    (
        "RENTAL_FOREIGN_POSSESSION_UNAVAILABLE",
        "This rental needs its actual U.S.-possession income and recovery records before its territorial tax treatment is available.",
    ),
    (
        "RENTAL_FOREIGN_COUNTRY_INVALID",
        "Use the actual IRS two-letter foreign country code for this property's location.",
    ),
    (
        "RENTAL_FOREIGN_ASSET_ADS_UNAVAILABLE",
        "The foreign rental's incidental assets need their actual ADS class lives and original no-bonus recovery records.",
    ),
    (
        "RENTAL_HOME_PRIOR_TOTAL_UNAVAILABLE",
        "Supply the independently tracked prior home-use worksheet carry for this tax system.",
    ),
    (
        "RENTAL_HOME_PRIOR_CHARACTER_UNAVAILABLE",
        "Supply the casualty character of the prior home-use carry before separated deductions are available.",
    ),
    (
        "RENTAL_HOME_PRIOR_CARRY_RECORDS",
        "The casualty carry must be nonnegative and cannot exceed its corresponding combined carry.",
    ),
    ("RENTAL_SAVED_CASUALTY_PROPERTY", "{0}: {1}"),
    (
        "NOTE_PARTNER_PROPERTY_QBI_BASIS_CARRY",
        "Partnership {0}, section 1231 loss suspended by the basis limit {1,number,currency}; qualified portion {2,number,currency}. Keep this balance separate from at-risk and passive losses. Retain the qualified-loss records; final property character and deductibility still require review.",
    ),
    (
        "NOTE_PARTNER_PROPERTY_QBI_RISK_CARRY",
        "Partnership {0}, section 1231 loss suspended by the at-risk limit {1,number,currency}; qualified portion {2,number,currency}. This balance already passed basis; do not repeat it there. Retain the qualified-loss records; final property character and deductibility still require review.",
    ),
    (
        "NOTE_PARTNER_PROPERTY_QBI_PASSIVE_CARRY",
        "Partnership {0}, section 1231 loss suspended by the passive limit {1,number,currency}; qualified portion {2,number,currency}. This balance already passed basis and at-risk limits; do not repeat it in either earlier balance. Retain the qualified-loss records; final property character and deductibility still require review.",
    ),
    (
        "RENTAL_CASUALTY_EVENT_WITHOUT_CASUALTY",
        "Enter this property's casualty or theft records, or clear its saved event.",
    ),
    (
        "ISSUE_RENTAL_HOME_AT_RISK_SALE_CARRY",
        "A prior at-risk property-sale loss on a rental with personal use needs the separate home and former-activity allocation, which is not calculated.",
    ),
    (
        "ACTIVITY_QBI_HISTORY_UNAVAILABLE",
        "Released prior at-risk losses need their original-year qualified business income status before completing the QBI deduction.",
    ),
    (
        "RENTAL_ASSET_NO_SECTION_179",
        "A rental asset cannot claim a section 179 deduction in this estimate.",
    ),
    (
        "ISSUE_RENTAL_SALE_DUAL_BASIS",
        "Selling a rental converted from personal use or received as a gift needs the separate gain and loss bases, which are not calculated.",
    ),
    ("DISPLAY_ROYALTY_INCOME", "Property and royalties"),
    ("DISPLAY_ROYALTY_DEPLETION", "Depletion and basis"),
    ("DISPLAY_RENTAL_PROPERTY", "Property and income"),
    ("DISPLAY_RENTAL_ASSETS", "Buildings and assets"),
    ("DISPLAY_RENTAL_USE", "Personal use and limits"),
    (
        "SELLER_MORTGAGE_BUYER_INVALID",
        "Enter the buyer's name, nine-digit taxpayer identification number without punctuation, and address for seller-financed home interest.",
    ),
    (
        "WARN_SAVINGS_BONDS_PRIOR_ALLOCATION",
        "The linked source combines interest from different bonds and does not identify how much previously reported interest belongs to these education bonds. The education exclusion is left out; the source's known current taxable interest remains included. Enter separate documented annual instruments to establish the allocation.",
    ),
    (
        "DEBT_EXEMPT_PREMIUM_HISTORY",
        "Tax-exempt premium creates no taxable-premium deduction. Leave taxable-premium history absent on an exempt instrument.",
    ),
    (
        "DEBT_NET_OFFSET_CHARACTER",
        "The applied premium offset must belong to this instrument’s matching ordinary or Treasury income; exempt income has no taxable-premium offset.",
    ),
    (
        "DEBT_OFFSET_ONLY_FOR_NET",
        "An already-applied taxable-premium offset belongs only to a net report. Leave it absent on a gross report.",
    ),
    (
        "BOND_PREMIUM_SOURCE_TWICE",
        "The same accepted instrument is entered more than once as a manual bond-premium deduction.",
    ),
    (
        "BOND_PREMIUM_DESCRIPTION_NEEDED",
        "Name the different documented bond whose manual deduction this is.",
    ),
    (
        "BOND_PREMIUM_SOURCE_MISSING",
        "This deduction names an annual instrument no longer on the return. Reopen and select its actual source or remove the deduction.",
    ),
    (
        "BOND_PREMIUM_COMPUTED_SOURCE_TWICE",
        "This instrument already supplies its computed annual excess-premium deduction. Remove the duplicate manual deduction.",
    ),
    (
        "WARN_DEBT_NET_CARRY_LEFT_OUT",
        "The annual amount is already net. Any separate uncomputed carry-premium relief is omitted; documented net interest remains unchanged.",
    ),
    (
        "DEBT_PRIVATE_ABOVE_EXEMPT",
        "The private-activity component cannot exceed its matching exempt interest.",
    ),
    (
        "DEBT_OUTSIDE_COMPONENT_OVERLAP",
        "Documented outside-owner components overlap or exceed this instrument’s matching reported component.",
    ),
    (
        "DEBT_PRIOR_ABOVE_RETAINED",
        "Previously included savings-bond interest cannot exceed retained U.S.-obligation interest.",
    ),
    (
        "DEBT_HISTORY_DEDUCTIONS_ABOVE_INCLUSIONS",
        "Prior premium deductions cannot exceed this instrument’s prior net interest inclusions.",
    ),
    (
        "DEBT_PRIVATE_AFTER_PREMIUM_INVALID",
        "Enter the documented retained private-activity amount after mandatory premium, within retained exempt interest. Do not estimate a ratio.",
    ),
    (
        "DEBT_INSTRUMENT_INVALID",
        "Enter one to 100 identified annual instrument worksheets with valid identities.",
    ),
    (
        "DEBT_COMPONENT_RANGE",
        "Annual debt components must be within their displayed ranges; only Treasury OID may be negative.",
    ),
    (
        "DEBT_MIXED_INSTRUMENT_CHARACTERS",
        "Keep ordinary, Treasury and tax-exempt instruments in separate annual worksheet rows.",
    ),
    (
        "DEBT_EXEMPT_REDUCTION_ABOVE_AMOUNT",
        "Exempt premium cannot exceed this instrument’s retained gross exempt interest.",
    ),
    ("DEBT_DISPOSAL_DATE_INVALID", "Enter a real sale, retirement or redemption date in 2026."),
    (
        "DEBT_PAYER_ALLOCATION_OVERLAP",
        "With annual instrument worksheets, enter ownership, prior inclusion and gross/net facts on each instrument only; leave payer allocations absent and payer reporting Gross with no prior inclusion.",
    ),
    (
        "DEBT_INSTRUMENT_TOTAL_MISMATCH",
        "Annual instrument components must reconcile separately to this payer’s reported boxes.",
    ),
    (
        "DEBT_WRONG_STATEMENT_COMPONENT",
        "An annual worksheet contains a component belonging to another form type.",
    ),
    (
        "NOTE_DEBT_AFTER_DEATH",
        "Documented annual instrument components belonging to the recipient after death are excluded from this final return.",
    ),
    (
        "WARN_DEBT_PREMIUM_ELECTION_LEFT_OUT",
        "The taxable-premium election is No or not confirmed. Known interest remains and taxable-premium relief is omitted; tax may be lower if you establish the election.",
    ),
    (
        "WARN_DEBT_PREMIUM_HISTORY_LEFT_OUT",
        "Same-instrument premium history is missing. Current documented interest offset remains; any uncomputed excess deduction or carry relief is omitted.",
    ),
    (
        "WARN_DEBT_TERMINAL_PREMIUM_LEFT_OUT",
        "Terminal unused-premium relief is not computed. Known income and established current premium treatment remain; an additional documented deduction may lower tax.",
    ),
    (
        "QOF_EVENT_NEEDED",
        "Tell us what happened to this opportunity fund investment during the return year.",
    ),
    ("QOF_NO_DEFERRED_GAIN", "Enter the short-term or long-term gain still deferred."),
    (
        "QOF_SECTION_1231_LONG_TERM",
        "A section 1231 gain (special gain code B) is long-term; leave the short-term gain empty.",
    ),
    (
        "QOF_COLLECTIBLES_LONG_TERM",
        "A collectibles gain (special gain code D) is long-term; leave the short-term gain empty.",
    ),
    ("QOF_SOLD_BEFORE_INVESTED", "The date sold must be after the date you invested."),
    ("QOF_DEFERRAL_NO_GAIN", "Enter the short-term or long-term gain you deferred."),
    ("INVESTMENT_PORTFOLIO_SOURCE", "Taxable portfolio investments"),
    ("INVESTMENT_ROYALTY_SOURCE", "Royalty property {0,number,0}"),
    ("INVESTMENT_LOAN_LABEL_TWICE", "Give each borrowing source a different label."),
    (
        "WARN_INTEREST_BOND_REPORTED_EARLIER_LEFT_OUT",
        "{0}: savings bond interest you reported on an earlier return, which Schedule B subtracts, isn't supported, so this statement is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_INTEREST_NOMINEE_LEFT_OUT",
        "{0}: interest you received as a nominee for someone else isn't supported, so this statement is left out of this estimate; your tax may be higher or lower.",
    ),
    (
        "WARN_INTEREST_TREATY_LEFT_OUT",
        "{0}: interest taxed under a tax treaty isn't supported, so this statement is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_QSBS_EXCLUSION_LEFT_OUT",
        "{0}: the issuer's section 1202 history isn't complete, so no exclusion is applied and the whole gain is taxed; your tax may be lower.",
    ),
    (
        "WARN_QSBS_ZONE_ASSUMED_NO",
        "{0}: assumed the empowerment-zone exclusion doesn't apply because its answer or zone gain isn't known, so the 50% exclusion is used; if it applies, your tax may be lower.",
    ),
    (
        "WARN_DIVIDEND_SECTION_897_LEFT_OUT",
        "{0}: dividends with section 897 gain (box 2e or 2f) aren't supported, so this statement is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_DIVIDEND_NOMINEE_LEFT_OUT",
        "{0}: dividends you received as a nominee for someone else aren't supported, so this statement is left out of this estimate; your tax may be higher or lower.",
    ),
    (
        "WARN_DIVIDEND_FOREIGN_LEFT_OUT",
        "{0}: foreign dividend treatment beyond the foreign tax credit election, such as a treaty rate, isn't supported, so this statement is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_DIVIDEND_OTHER_LEFT_OUT",
        "{0}: a dividend treatment this page doesn't calculate, such as an unlisted REIT or publicly traded partnership treatment, isn't supported, so this statement is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_OID_INVESTMENT_EXPENSES_LEFT_OUT",
        "{0}: original issue discount with investment expenses (box 9) isn't supported, so this statement is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_OID_NOMINEE_LEFT_OUT",
        "{0}: original issue discount you received as a nominee for someone else isn't supported, so this statement is left out of this estimate; your tax may be higher or lower.",
    ),
    (
        "WARN_OID_OBLIGATION_LEFT_OUT",
        "{0}: original issue discount on a stripped or contingent-payment obligation isn't supported, so this statement is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_OID_OTHER_LEFT_OUT",
        "{0}: an original issue discount treatment this page doesn't calculate isn't supported, so this statement is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_SALE_STOCK_COMPENSATION_LEFT_OUT",
        "{0}: a sale of stock whose compensation isn't resolved isn't supported, so it is left out of this estimate; an ordinary option, RSU, ESPP or NUA sale can be entered as Employee stock shares sold, ESPP shares sold, ISO shares sold or NUA shares sold. Your tax may be higher, or lower if it had a loss.",
    ),
    (
        "WARN_SALE_BUSINESS_PROPERTY_LEFT_OUT",
        "{0}: a sale of business or rental property is entered on that asset's or rental's own record, so this entry is left out of the estimate to avoid counting it twice.",
    ),
    (
        "WARN_SALE_HOME_LEFT_OUT",
        "{0}: the sale of your home isn't supported on this page, so it is left out of this estimate; your tax may be higher, or lower if it had a loss.",
    ),
    (
        "WARN_SALE_OTHER_LEFT_OUT",
        "{0}: a sale treatment this page doesn't calculate (a contingent payment debt instrument, a rollover of gain, the DC Zone or community asset exclusion, a trader's mark-to-market election, a carried-interest gain, or a loss your broker says the proceeds can't support) isn't supported, so it is left out of this estimate; your tax may be higher, or lower if it had a loss.",
    ),
    (
        "NOTE_SAVINGS_BONDS_CREDIT_EXPENSES",
        "An education credit entry uses tuition of a person on the savings bond record, but its credit is zero, so that tuition is not available for the exclusion. Remove the education credit entry to free the expenses for the exclusion.",
    ),
    (
        "NOTE_SAVINGS_BONDS_EXCLUSION_MFS",
        "No savings bond interest is excluded on a married-filing-separately return, so all of it stays taxable (section 135(d)(3)).",
    ),
    (
        "NOTE_SAVINGS_BONDS_OWNER_UNDER_24",
        "No savings bond interest is excluded: the bond owner had not turned 24 before the earliest bond was issued (section 135(c)(1)(B)). Check the owner's birth date and the issue date.",
    ),
    (
        "ISSUE_SECTION1231_ADD__BUSINESS_PROPERTY_LOSSES_FROM_PRIOR_FIVE_YEARS__ONCE_F5B687DD",
        "Add \"Business-property losses from 2021–2025\" once for the return: choose \"No net business-property loss on a 2021–2025 return\" or enter the losses.",
    ),
    (
        "ISSUE_SECTION1231_AN_INVESTMENT_INTEREST_PROPERTY_RESULT_HAS_NO_5284D6A5",
        "An investment-interest property result has no matching source business.",
    ),
    (
        "ISSUE_SECTION1231_A_BUSINESS_PROPERTY_ACTIVITY_IS_MISSING_ITS_A7B2C775",
        "A business-property activity is missing its resolved loss-limit result.",
    ),
    (
        "ISSUE_SECTION1231_A_PROPERTY_INVESTMENT_TAX_RESULT_IS_MISSING_A060069C",
        "A property investment-tax result is missing its source business.",
    ),
    (
        "ISSUE_SECTION1231_COMPLETE_AND_RECONCILE_THE_SECTION_1231_50E2A327",
        "Complete and reconcile the section 1231 lookback records and current sources for these filers.",
    ),
    (
        "ISSUE_SECTION1231_COMPLETE_REGULAR_SECTION_1231_LOOKBACK_HISTORY_400E96BC",
        "Complete regular section 1231 lookback history before computing rental MAGI from nonpassive property gains.",
    ),
    (
        "ISSUE_SECTION1231_COMPLETE_RETURN_SETUP_BEFORE_RENTAL_MAGI_35900792",
        "Complete return setup before rental MAGI.",
    ),
    (
        "ISSUE_SECTION1231_DIRECT_BUSINESS_PROPERTY_SALE_BUSINESS_IS_MISSING_4887B9F5",
        "Direct business-property sale business is missing.",
    ),
    (
        "ISSUE_SECTION1231_LOSS_GAIN_ALLOCATION_REQUIRES_ONE_ORDINARY_NONPASSIVE_39E47853",
        "Loss/gain allocation requires one ordinary nonpassive loss character and no loss history in the gain character.",
    ),
    (
        "ISSUE_SECTION1231_MIXED_INVESTMENT_TAX_TREATMENT_WITHIN_A_PROPERTY_E12DAE20",
        "Mixed investment-tax treatment within a property source needs traced ordinary/capital allocation.",
    ),
    (
        "ISSUE_SECTION1231_NONPASSIVE_PROPERTY_CHARACTER_IS_UNAVAILABLE_FOR_922C25EC",
        "Nonpassive property character is unavailable for rental MAGI.",
    ),
    (
        "ISSUE_SECTION1231_PROPERTY_EXCESS_BUSINESS_LOSS_REQUIRES_RESOLVED_975FCF28",
        "Property excess business loss requires resolved operating/property source allocation.",
    ),
    (
        "ISSUE_SECTION1231_PROVIDE_INDEPENDENT_AMT_PROPERTY_RECORDS_FOR_50C0DB6F",
        "Provide independent AMT property records for investment interest.",
    ),
    (
        "ISSUE_SECTION1231_RELEASED_FORMER_PASSIVE_PROPERTY_LOSSES_NEED_SEPARATE_EB3B9611",
        "Released former-passive property losses need separate investment-interest attribution before combining trading property amounts.",
    ),
    (
        "ISSUE_SECTION1231_RESOLVE_ALL_PASSIVE_POOL_INPUT_AND_ACTIVITY_LIMIT_055CB3FF",
        "Resolve all passive-pool input and activity-limit issues before calculating the section 1231 lookback.",
    ),
    (
        "ISSUE_SECTION1231_RESOLVE_AND_RECONCILE_THE_PROPERTY_ACTIVITY_FFC5EB09",
        "Resolve and reconcile the property activity pool before investment-tax classification.",
    ),
    (
        "ISSUE_SECTION1231_RESOLVE_FARM_LAND_SALE_ELIGIBILITY_BEFORE_RENTAL_10544F0A",
        "Resolve farm-land sale eligibility before rental MAGI.",
    ),
    (
        "ISSUE_SECTION1231_RESOLVE_PARTICIPATION_BEFORE_CALCULATING_NONPASSIVE_PROPERTY_39FB4535",
        "Resolve participation before calculating nonpassive property income for rental MAGI.",
    ),
    (
        "ISSUE_SECTION1231_RESOLVE_SOURCE_RECAPTURE_BEFORE_INVESTMENT_TAX_CLASSIFICATION_031FC7BF",
        "Resolve source recapture before investment-tax classification.",
    ),
    (
        "ISSUE_SECTION1231_RESOLVE_THE_PROPERTY_ACTIVITY_POOL_BEFORE_05F7B354",
        "Resolve the property activity pool before the investment-interest worksheet.",
    ),
    (
        "ISSUE_SECTION1231_SECTION_1231_LOOKBACK_RECAPTURE_NEEDS_BUSINESS_LEVEL_F77D46EA",
        "Section 1231 lookback recapture needs business-level gain allocation before source character can be determined.",
    ),
    (
        "ISSUE_SECTION1231_THE_NONPASSIVE_PROPERTY_PROJECTION_CANNOT_BE_59B193F5",
        "The nonpassive property projection cannot be reconciled for rental MAGI.",
    ),
    (
        "ISSUE_SECTION1231_THE_SECTION_1231_HISTORY_OR_ACTIVITY_33E7BDF7",
        "The section 1231 history or activity amounts cannot be reconciled.",
    ),
    (
        "NOTE_1231_REGULAR_WORKSHEET",
        "Section 1231 regular {0,number,0} worksheet: net after activity limits {1,number,currency}; prior five-year nonrecaptured losses {2,number,currency}; ordinary amount {3,number,currency}; long-term amount {4,number,currency}; business-asset section 1250 component {5,number,currency}. Intermediate Form 4797/character worksheet, not final return totals. Later QBI, investment-tax and return-level loss rules remain separate.",
    ),
    (
        "NOTE_1231_AMT_WORKSHEET",
        "Section 1231 AMT {0,number,0} worksheet: net after activity limits {1,number,currency}; prior five-year nonrecaptured losses {2,number,currency}; ordinary amount {3,number,currency}; long-term amount {4,number,currency}; business-asset section 1250 component {5,number,currency}. Intermediate Form 4797/character worksheet, not final return totals. Later QBI, investment-tax and return-level loss rules remain separate.",
    ),
    (
        "NOTE_1231_REGULAR_LOSS_YEAR",
        "Section 1231 regular loss year {0,number,0}: original deductible loss {1,number,currency}; opening nonrecaptured balance {2,number,currency}; recaptured this year {3,number,currency}; expired {4,number,currency}; eligible balance for next year {5,number,currency}. Provisional until all current-year source amounts and return calculations are final.",
    ),
    (
        "NOTE_1231_AMT_LOSS_YEAR",
        "Section 1231 AMT loss year {0,number,0}: original deductible loss {1,number,currency}; opening nonrecaptured balance {2,number,currency}; recaptured this year {3,number,currency}; expired {4,number,currency}; eligible balance for next year {5,number,currency}. Provisional until all current-year source amounts and return calculations are final.",
    ),
    (
        "NOTE_1231_REGULAR_CURRENT_LOSS",
        "Section 1231 regular current net loss {0,number,currency}: a new lookback balance cannot be finalized until the loss taken into account in taxable income is established. Do not automatically carry the full intermediate loss as a new history entry.",
    ),
    (
        "NOTE_1231_AMT_CURRENT_LOSS",
        "Section 1231 AMT current net loss {0,number,currency}: a new lookback balance cannot be finalized until the loss taken into account in taxable income is established. Do not automatically carry the full intermediate loss as a new history entry.",
    ),
    (
        "NOTE_1231_REGULAR_REMAINING",
        "Section 1231 regular prior-loss balances remaining eligible for {0,number,0}: {1,number,currency}. Retain the loss-year detail after final review; this is not a deduction carried forward.",
    ),
    (
        "NOTE_1231_AMT_REMAINING",
        "Section 1231 AMT prior-loss balances remaining eligible for {0,number,0}: {1,number,currency}. Retain the loss-year detail after final review; this is not a deduction carried forward.",
    ),
    (
        "NOTE_1231_SCORP_POTENTIAL_QBI",
        "S-corporation document {0,number,integer}, section 1231 potential QBI after capital exclusion: gain {1,number,currency}; loss {2,number,currency}. Current-source qualified loss {3,number,currency}; released historical qualified loss {4,number,currency}. Current-year SSTB eligibility and return-level limits remain unresolved; historical loss eligibility is retained. These are intermediate business items, not a final QBI deduction.",
    ),
    (
        "NOTE_1231_REGULAR_FINAL_HISTORY",
        "Finalized section 1231 regular history for {0,number,integer}: loss year {1,number,0}, original deductible loss {2,number,currency}, remaining balance {3,number,currency}. Keep the year and original amount with this balance; it is not a second loss deduction.",
    ),
    (
        "NOTE_1231_AMT_FINAL_HISTORY",
        "Finalized section 1231 AMT history for {0,number,integer}: loss year {1,number,0}, original deductible loss {2,number,currency}, remaining balance {3,number,currency}. Keep the year and original amount with this balance; it is not a second loss deduction.",
    ),
    (
        "ISSUE_INSTALLMENT_PAYMENTS_EXCEED_PRICE",
        "Installment sale: the loans the buyer took over and the principal received in 2026 cannot exceed the sale price (Form 6252 lines 5, 6 and 21).",
    ),
    (
        "ISSUE_INSTALLMENT_SPECIAL_TRANSACTION",
        "An installment sale to a related party, by a dealer, with a price that depends on later events, or with a note pledged as security or disposed of in 2026 is not calculated (Form 6252 lines 3 and 4 and Part III; sections 453(e), 453(g), 453A(d) and 453B).",
    ),
    (
        "ISSUE_INSTALLMENT_UNSTATED_INTEREST",
        "An installment note charging less than the applicable federal rate has unstated interest: part of the sale price is interest income (Form 6252 line 5; sections 483 and 1274, Publication 537), which is not calculated.",
    ),
    (
        "ISSUE_ASSET_DISPOSAL_DATE_BEFORE_SERVICE",
        "The date sold or stopped using must be after the date placed in service.",
    ),
    (
        "ISSUE_RENTAL_SALE_DATE_OR_LAND_PRICE_INVALID",
        "Enter the date you sold the rental in 2026 and a land price no larger than the sale price.",
    ),
    (
        "ISSUE_RENTAL_SALE_BEFORE_BUILDING_SERVICE",
        "The date sold or exchanged must be after the date each building and improvement was first available to rent.",
    ),
    (
        "ISSUE_SAVINGS_BOND_STUDENT_EXPENSES_ENTERED_TWICE",
        "This person's tuition is entered on Education credits and again on the savings bond record. Enter it once: choose the education credit entry here, or remove the credit entry.",
    ),
    (
        "ISSUE_SAVINGS_BOND_STUDENT_HAS_NO_EDUCATION_CREDIT_ENTRY",
        "A savings bond student's expenses come from an education credit entry, but this person has none. Add the entry on Education credits or enter the expenses on the savings bond record.",
    ),
    (
        "BROKER_PRINTED_VALUES_IN_DOCUMENT",
        "Correct the separate printed aggregation checkboxes in the broker document.",
    ),
    ("DISPLAY_SALE_MORE", "More about this sale"),
    ("DISPLAY_IMPROVEMENTS", "Improvements"),
    ("DISPLAY_CARRYOVERS", "Carryovers"),
    ("DISPLAY_EXCHANGE_DATES", "Dates"),
    ("DISPLAY_EXCHANGE_RECEIVED", "What you received"),
    ("DISPLAY_EXCHANGE_GIVEN", "What you added"),
    ("DISPLAY_EXCHANGE_OTHER_SIDE", "The other side"),
    ("DISPLAY_QSBS_SALES", "Issuer and sales"),
    ("DISPLAY_INT1099_PAYER", "Payer, recipient and box 1"),
    ("DISPLAY_INT1099_OTHER", "FATCA, account, box 14 and state"),
    ("DISPLAY_OID1099_PAYER", "Payer, recipient and boxes 1–2"),
    ("DISPLAY_OID1099_OTHER", "FATCA, account and state"),
    ("DISPLAY_INT_BOXES_2_4", "Boxes 2–4"),
    ("DISPLAY_INT_BOXES_6_9", "Boxes 6–9 foreign and tax-exempt"),
    ("DISPLAY_INT_BOXES_10_13", "Boxes 10–13 bond premium"),
    ("DISPLAY_DIV_BOXES_2_4", "Boxes 2a–4 gains and withholding"),
    ("DISPLAY_DIV_BOXES_5_13", "Boxes 5–13"),
    ("DISPLAY_OID_BOXES_3_6", "Boxes 3–6"),
    ("DISPLAY_OID_BOXES_8_11", "Boxes 8–11"),
    (
        "ISSUE_ALL_ENTRIES_FOR_ONE_ISSUING_CORPORATION_MUST_USE_ONE_RECONCILED_PRI_9F0EB02F",
        "All entries for one issuing corporation must use one reconciled prior eligible-gain amount; the exclusion cap is shared across its current-year lots.",
    ),
    (
        "ISSUE_A_BINDING_ISSUER_WIDE_SECTION_1202_ELIGIBLE_GAIN_CAP_WITH_50_75_OR_78E873C5",
        "A binding issuer-wide section 1202 eligible-gain cap with 50%, 75% or 100% acquisition cohorts needs an explicit allocation of that cap across cohorts.",
    ),
    (
        "ISSUE_A_BINDING_ISSUER_WIDE_SECTION_1202_LIMIT_WITH_BOTH_EMPOWERMENT_ZONE_0C92F677",
        "A binding issuer-wide section 1202 limit with both empowerment-zone and ordinary gain needs a verified allocation between 60% and 50% eligible gain.",
    ),
    (
        "ISSUE_ENTER_A_RETURN_YEAR_DATE_SOLD_OR_STOPPED_USING_THE_SALE_PRICE_AND_SELLING_A990EE59",
        "Enter a 2026 date sold or stopped using, the sale price and selling expenses within the supported limits, and installment amounts no larger than the sale price (Form 6252 lines 5, 6 and 21).",
    ),
    (
        "ISSUE_ENTER_A_SALE_DATE_BEFORE_RETURN_YEAR_A_GROSS_PROFIT_NO_LARGER_THAN_THE_CON_7BB8E34C",
        "Enter a sale date before 2026, a gross profit no larger than the contract price, and payments that do not exceed the contract price (Form 6252 lines 16, 18, 21 and 23).",
    ),
    (
        "NOTE_ASSET_STOPPED_BUSINESS_USE",
        "{0}: stopped business use in 2026. Depreciation stops at the date of the change under the convention; no gain or loss is recognized, and the property keeps its adjusted basis.",
    ),
    (
        "NOTE_ASSET_STOPPED_BUSINESS_USE_VEHICLE",
        "{0}: stopped business use in 2026. Depreciation stops at the date of the change under the convention; no gain or loss is recognized, and the property keeps its adjusted basis. If this vehicle is used 50% or less for business in a later year of its recovery period, excess depreciation must be recaptured on that year's return (Form 4797 Part IV).",
    ),
    (
        "NOTE_ASSET_PERSONAL_SALE_LOSS",
        "{0}: the personal-use part of the sale is a {1,number,currency} loss, which is not deductible.",
    ),
    (
        "ISSUE_INSTALLMENT_GAIN_FROM_AN_EARLIER_SALE_OF_RENTAL_OR_PASSIVE_ACTIVITY_13F232BB",
        "Installment gain from an earlier sale of rental or passive-activity property is passive income that allows part of this return's unallowed passive losses (Form 8582 lines 1 through 4), which is not calculated.",
    ),
    (
        "ISSUE_INSTALLMENT_NOTES_FROM_RETURN_YEAR_SALES_ABOVE_150_000_TOTAL_MORE_THAN_5_0_5149A91B",
        "Installment notes from 2026 sales above $150,000 total more than $5,000,000 at the end of 2026, so interest is owed on the deferred tax (section 453A, Form 6252 and Publication 537), which is not calculated.",
    ),
    (
        "RENTAL_BUILDING_SALE_HISTORY_UNAVAILABLE",
        "The sold rental's mixed-use building needs its actual business basis and regular/AMT business depreciation through the sale date.",
    ),
    (
        "RENTAL_PROPERTY_CHARACTER_UNAVAILABLE",
        "Resolve the sold rental's section 1231 character before completing its qualified business income.",
    ),
    (
        "RENTAL_PROPERTY_QBI_ACTIVITY_UNAVAILABLE",
        "Resolve the sold rental's single qualified activity before adding its ordinary property income.",
    ),
    (
        "ISSUE_THIS_EARLIER_YEAR_INSTALLMENT_SALE_IS_NOT_CALCULATED_A_RELATED_PART_7D8BAB22",
        "This earlier-year installment sale is not calculated: a related-party sale, a dealer sale, a contingent price, a pledged or disposed note, remaining recapture, collectibles, small business stock, unstated interest, a different AMT gain or suspended passive losses need their own worksheets (Form 6252 lines 3, 4, 25 and Part III; Publication 537).",
    ),
    // Complete save warnings and the localized row-kind phrases they use.
    ("SAVINGS_BONDS_INTEREST_ABOVE_PROCEEDS", "The interest cannot be more than the proceeds."),
    (
        "SAVINGS_BONDS_ISSUED_AFTER_2026",
        "Bonds cashed in 2026 were issued by the end of 2026. Check the earliest issue date.",
    ),
    (
        "SAVINGS_BONDS_ISSUED_BEFORE_1990",
        "Only bonds issued after 1989 qualify. Enter the earliest issue date from Form 8818.",
    ),
    ("DIV_QUALIFIED_ABOVE_ORDINARY", "Box 1b cannot exceed box 1a ordinary dividends."),
    ("DIV_2B_ABOVE_2A", "Box 2b cannot exceed box 2a total capital gain distributions."),
    ("DIV_2D_ABOVE_2A", "Box 2d cannot exceed box 2a total capital gain distributions."),
    (
        "INT_OUTSIDE_TAXABLE_ABOVE_BOX",
        "The combined nominee and after-death taxable cannot exceed the statement component.",
    ),
    (
        "INT_OUTSIDE_US_OBLIGATIONS_ABOVE_BOX",
        "The combined nominee and after-death us obligations cannot exceed the statement component.",
    ),
    (
        "INT_OUTSIDE_EXEMPT_ABOVE_BOX",
        "The combined nominee and after-death exempt cannot exceed the statement component.",
    ),
    (
        "INT_OUTSIDE_PRIVATE_ACTIVITY_ABOVE_BOX",
        "The combined nominee and after-death private activity cannot exceed the statement component.",
    ),
    (
        "INT_OUTSIDE_FOREIGN_TAX_ABOVE_BOX",
        "The combined nominee and after-death foreign tax cannot exceed the statement component.",
    ),
    (
        "INT_OUTSIDE_BOND_PREMIUM_ABOVE_BOX",
        "The combined nominee and after-death bond premium cannot exceed the statement component.",
    ),
    (
        "INT_OUTSIDE_TREASURY_BOND_PREMIUM_ABOVE_BOX",
        "The combined nominee and after-death treasury bond premium cannot exceed the statement component.",
    ),
    (
        "INT_OUTSIDE_TAX_EXEMPT_BOND_PREMIUM_ABOVE_BOX",
        "The combined nominee and after-death tax exempt bond premium cannot exceed the statement component.",
    ),
    (
        "INT_OUTSIDE_ACCRUED_INTEREST_ABOVE_BOX",
        "The combined nominee and after-death accrued interest cannot exceed the statement component.",
    ),
    (
        "INT_OUTSIDE_MARKET_DISCOUNT_ABOVE_BOX",
        "The combined nominee and after-death market discount cannot exceed the statement component.",
    ),
    (
        "INT_NOMINEE_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
        "The nominee private activity must be included in its exempt component.",
    ),
    (
        "INT_AFTER_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
        "The after private activity must be included in its exempt component.",
    ),
    (
        "INT_RETAINED_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
        "The retained private activity must be included in its exempt component.",
    ),
    (
        "DIV_OUTSIDE_ORDINARY_ABOVE_BOX",
        "The combined nominee and after-death ordinary cannot exceed the statement component.",
    ),
    (
        "DIV_OUTSIDE_QUALIFIED_ABOVE_BOX",
        "The combined nominee and after-death qualified cannot exceed the statement component.",
    ),
    (
        "DIV_OUTSIDE_CAPITAL_ABOVE_BOX",
        "The combined nominee and after-death capital cannot exceed the statement component.",
    ),
    (
        "DIV_OUTSIDE_UNRECAPTURED_ABOVE_BOX",
        "The combined nominee and after-death unrecaptured cannot exceed the statement component.",
    ),
    (
        "DIV_OUTSIDE_COLLECTIBLE_ABOVE_BOX",
        "The combined nominee and after-death collectible cannot exceed the statement component.",
    ),
    (
        "DIV_OUTSIDE_SECTION_1202_ABOVE_BOX",
        "The combined nominee and after-death section 1202 cannot exceed the statement component.",
    ),
    (
        "DIV_OUTSIDE_SECTION_199A_ABOVE_BOX",
        "The combined nominee and after-death section 199a cannot exceed the statement component.",
    ),
    (
        "DIV_OUTSIDE_EXEMPT_ABOVE_BOX",
        "The combined nominee and after-death exempt cannot exceed the statement component.",
    ),
    (
        "DIV_OUTSIDE_PRIVATE_ACTIVITY_ABOVE_BOX",
        "The combined nominee and after-death private activity cannot exceed the statement component.",
    ),
    (
        "DIV_OUTSIDE_FOREIGN_TAX_ABOVE_BOX",
        "The combined nominee and after-death foreign tax cannot exceed the statement component.",
    ),
    (
        "DIV_OUTSIDE_SECTION_897_ORDINARY_ABOVE_BOX",
        "The combined nominee and after-death section 897 ordinary cannot exceed the statement component.",
    ),
    (
        "DIV_OUTSIDE_SECTION_897_CAPITAL_ABOVE_BOX",
        "The combined nominee and after-death section 897 capital cannot exceed the statement component.",
    ),
    (
        "DIV_NOMINEE_QUALIFIED_ABOVE_ORDINARY",
        "The nominee qualified must be included in its ordinary component.",
    ),
    (
        "DIV_AFTER_QUALIFIED_ABOVE_ORDINARY",
        "The after qualified must be included in its ordinary component.",
    ),
    (
        "DIV_RETAINED_QUALIFIED_ABOVE_ORDINARY",
        "The retained qualified must be included in its ordinary component.",
    ),
    (
        "DIV_NOMINEE_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
        "The nominee private activity must be included in its exempt component.",
    ),
    (
        "DIV_AFTER_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
        "The after private activity must be included in its exempt component.",
    ),
    (
        "DIV_RETAINED_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
        "The retained private activity must be included in its exempt component.",
    ),
    (
        "DIV_NOMINEE_SECTION_199A_ABOVE_ORDINARY",
        "The nominee section 199a must be included in its ordinary component.",
    ),
    (
        "DIV_AFTER_SECTION_199A_ABOVE_ORDINARY",
        "The after section 199a must be included in its ordinary component.",
    ),
    (
        "DIV_RETAINED_SECTION_199A_ABOVE_ORDINARY",
        "The retained section 199a must be included in its ordinary component.",
    ),
    (
        "DIV_NOMINEE_SECTION_897_ORDINARY_ABOVE_ORDINARY",
        "The nominee section 897 ordinary must be included in its ordinary component.",
    ),
    (
        "DIV_AFTER_SECTION_897_ORDINARY_ABOVE_ORDINARY",
        "The after section 897 ordinary must be included in its ordinary component.",
    ),
    (
        "DIV_RETAINED_SECTION_897_ORDINARY_ABOVE_ORDINARY",
        "The retained section 897 ordinary must be included in its ordinary component.",
    ),
    (
        "DIV_NOMINEE_SECTION_897_CAPITAL_ABOVE_CAPITAL",
        "The nominee section 897 capital must be included in its capital component.",
    ),
    (
        "DIV_AFTER_SECTION_897_CAPITAL_ABOVE_CAPITAL",
        "The after section 897 capital must be included in its capital component.",
    ),
    (
        "DIV_RETAINED_SECTION_897_CAPITAL_ABOVE_CAPITAL",
        "The retained section 897 capital must be included in its capital component.",
    ),
    (
        "DIV_NOMINEE_UNRECAPTURED_ABOVE_CAPITAL",
        "The nominee unrecaptured must be included in its capital component.",
    ),
    (
        "DIV_AFTER_UNRECAPTURED_ABOVE_CAPITAL",
        "The after unrecaptured must be included in its capital component.",
    ),
    (
        "DIV_RETAINED_UNRECAPTURED_ABOVE_CAPITAL",
        "The retained unrecaptured must be included in its capital component.",
    ),
    (
        "DIV_NOMINEE_COLLECTIBLE_ABOVE_CAPITAL",
        "The nominee collectible must be included in its capital component.",
    ),
    (
        "DIV_AFTER_COLLECTIBLE_ABOVE_CAPITAL",
        "The after collectible must be included in its capital component.",
    ),
    (
        "DIV_RETAINED_COLLECTIBLE_ABOVE_CAPITAL",
        "The retained collectible must be included in its capital component.",
    ),
    (
        "DIV_NOMINEE_SECTION_1202_ABOVE_CAPITAL",
        "The nominee section 1202 must be included in its capital component.",
    ),
    (
        "DIV_AFTER_SECTION_1202_ABOVE_CAPITAL",
        "The after section 1202 must be included in its capital component.",
    ),
    (
        "DIV_RETAINED_SECTION_1202_ABOVE_CAPITAL",
        "The retained section 1202 must be included in its capital component.",
    ),
    (
        "OID_NOMINEE_OID_ABOVE_BOX",
        "The nominee oid cannot exceed the matching OID statement component.",
    ),
    (
        "OID_NOMINEE_OTHER_PERIODIC_INTEREST_ABOVE_BOX",
        "The nominee other periodic interest cannot exceed the matching OID statement component.",
    ),
    (
        "OID_NOMINEE_MARKET_DISCOUNT_ABOVE_BOX",
        "The nominee market discount cannot exceed the matching OID statement component.",
    ),
    (
        "OID_NOMINEE_ACQUISITION_PREMIUM_ABOVE_BOX",
        "The nominee acquisition premium cannot exceed the matching OID statement component.",
    ),
    (
        "OID_NOMINEE_BOND_PREMIUM_ABOVE_BOX",
        "The nominee bond premium cannot exceed the matching OID statement component.",
    ),
    (
        "OID_NOMINEE_EXEMPT_ABOVE_BOX",
        "The nominee exempt cannot exceed the matching OID statement component.",
    ),
    (
        "OID_NOMINEE_TAX_EXEMPT_BOND_PREMIUM_ABOVE_BOX",
        "The nominee tax exempt bond premium cannot exceed the matching OID statement component.",
    ),
    (
        "OID_NOMINEE_PRIVATE_ACTIVITY_ABOVE_BOX",
        "The nominee private activity cannot exceed the matching OID statement component.",
    ),
    (
        "INVESTMENT_SPLIT_RANGE",
        "Ownership component amounts must be within their permitted ranges.",
    ),
    (
        "INTEREST_PRIOR_ABOVE_RETAINED",
        "Previously reported interest cannot exceed the retained U.S. savings-bond interest.",
    ),
    (
        "OID_NOMINEE_TREASURY_SIGN_OR_BOUND",
        "A Treasury OID allocation must preserve the sign and not exceed the reported amount.",
    ),
    (
        "OID_NOMINEE_PRIVATE_ABOVE_EXEMPT",
        "Nominee private-activity OID must be included in nominee exempt OID.",
    ),
    (
        "OID_RETAINED_PRIVATE_ABOVE_EXEMPT",
        "Retained private-activity OID must be included in retained exempt OID.",
    ),
    (
        "OID_PRIOR_IS_NOT_BOX3",
        "Previously taxed savings-bond interest belongs on a 1099-INT box 3 source, not an OID source.",
    ),
    (
        "NOTE_INVESTMENT_NOMINEE",
        "Income belonging to someone outside this return is left off. A nominee may need to furnish and file Forms 1099; this App does not file them.",
    ),
    (
        "DIV_OUTSIDE_QUALIFIED_REIT_OVERLAP",
        "Qualified and section 199A dividends must be distinct parts of ordinary dividends for each owner.",
    ),
    (
        "DIV_OUTSIDE_CAPITAL_PARTS_OVERLAP",
        "Unrecaptured and collectible distributions must be distinct parts of capital distributions for each owner.",
    ),
    ("DIV_13_ABOVE_12", "Box 13 cannot exceed box 12 exempt-interest dividends."),
    (
        "NOTE_SECTION_1256_CARRYBACK",
        "{0,number,currency} of your net section 1256 contracts loss is carried back to 2023–2025 and left out of 2026. The refund for those years is claimed on Form 1045 or Form 1040-X and is not part of this estimate.",
    ),
    (
        "ISSUE_SECTION_1256_CARRYBACK_WITHOUT_LOSS",
        "A section 1256 loss carryback needs a net loss on Form 6781 line 5; your 1099-B contracts show a net gain. Remove the carryback or enter the contracts that produced the loss.",
    ),
    (
        "ISSUE_SECTION_1256_CARRYBACK_ABOVE_NET_LOSS",
        "The section 1256 loss carryback of {0,number,currency} is more than {1,number,currency}, the most your net section 1256 contracts loss allows. Reduce it.",
    ),
    (
        "QSBS_LOT_PROCEEDS_NOT_ABOVE_BASIS",
        "Qualified small business stock: the proceeds must be more than the basis. A sale at a loss or break-even is an ordinary Form 1099-B sale on Investment sales, not a section 1202 gain.",
    ),
    (
        "QSBS_LOT_ZONE_GAIN_OUTSIDE_THE_LOT_GAIN",
        "Qualified small business stock: the gain through 2018 for the empowerment-zone exclusion cannot be negative or more than this lot's gain (proceeds less basis).",
    ),
    (
        "QSBS_LOT_ZONE_GAIN_OUTSIDE_ZONE_DATES",
        "Qualified small business stock: the empowerment-zone gain applies only to shares acquired December 22, 2000 through February 17, 2009.",
    ),
    (
        "QSBS_LOT_OUTSIDE_SECTION_1202_DATES",
        "Qualified small business stock: each sale needs stock issued from August 11, 1993 through July 4, 2025, sold in 2026 more than five years after it was issued. Stock issued later cannot meet its minimum holding period in 2026.",
    ),
    (
        "SECTION_1244_ISSUED_AFTER_RETURN_YEAR",
        "Section 1244 stock: the date the stock was issued cannot be after 2026.",
    ),
    (
        "SECTION_1244_SOLD_BEFORE_ISSUED",
        "Section 1244 stock: the date sold cannot be before the date the stock was issued.",
    ),
    ("SECTION_1244_SALE_NOT_IN_RETURN_YEAR", "Section 1244 stock: enter a sale made in 2026."),
    (
        "SECTION_1244_PROCEEDS_ABOVE_BASIS",
        "Section 1244 stock: the proceeds cannot be more than your basis; a sale at a gain is not a loss to enter here.",
    ),
    (
        "SECTION_1244_LIABILITIES_ABOVE_BASIS",
        "Section 1244 stock: the liabilities the corporation assumed cannot be more than the property's basis.",
    ),
    (
        "SECTION_1244_PROPERTY_AFTER_ISSUE",
        "Section 1244 stock: the property cannot have been acquired after the stock was issued for it.",
    ),
    (
        "WORTHLESS_ACQUIRED_AFTER_2026",
        "Worthless security: the acquisition date cannot be after 2026.",
    ),
    (
        "BAD_DEBT_RECOVERED_ABOVE_ADVANCED",
        "Bad debt: the amount recovered or reimbursed cannot be more than the amount you advanced, paid or deposited.",
    ),
    (
        "BAD_DEBT_COLLECTED_FACE",
        "Bad debt: principal collected on a purchased debt must be less than its face amount; a fully collected debt has no loss.",
    ),
    (
        "BAD_DEBT_COLLECTED_ABOVE_COST",
        "Bad debt: principal collected on a purchased debt cannot be more than the price you paid plus the acquisition costs.",
    ),
    (
        "BAD_DEBT_COLLECTED_NEEDS_FACE_PRICE",
        "Bad debt: a purchased debt with principal collected is calculated only when you bought it at its face amount. Check the price paid and the face amount.",
    ),
    (
        "BAD_DEBT_PERSONAL_CLAIM_NOT_INVESTMENT",
        "Bad debt: a contractor deposit or a former-home mortgage shortfall is a personal claim, never a debt held to earn investment income. Answer No to the investment question.",
    ),
    (
        "BAD_DEBT_PURCHASED_AFTER_2026",
        "Bad debt: the date you bought the debt cannot be after 2026.",
    ),
    (
        "CAPITAL_RETURN_PROPERTY_NOT_LIQUIDATION",
        "1099-DIV return of capital: property in box 10 belongs to a liquidation; choose an interim or final liquidation, or Not sure.",
    ),
    (
        "FORM_2439_PARTS_ABOVE_1A",
        "Form 2439: boxes 1b, 1c and 1d are parts of box 1a and cannot add up to more than it.",
    ),
    (
        "FORM_2439_TAX_ABOVE_1A",
        "Form 2439: the tax paid in box 2 cannot be more than the gain in box 1a.",
    ),
    (
        "RENTAL_SOLD_BEFORE_IN_SERVICE",
        "The date sold must be after the date the building and each improvement were first available to rent.",
    ),
    (
        "INSTALLMENT_PAID_ABOVE_PRICE",
        "Installment sale: the loans the buyer took over and the principal received in 2026 cannot exceed the sale price.",
    ),
    (
        "INSTALLMENT_LIVESTOCK_CHARACTER",
        "Livestock cannot have building depreciation (unrecaptured section 1250 gain). Check the original property classification and Form 6252.",
    ),
    (
        "RENTAL_LAND_PRICE_ABOVE_PRICE",
        "The part of the sale price for the land cannot exceed the sale price.",
    ),
    ("RENTAL_DAYS_ABOVE_365", "Fair rental days plus personal-use days cannot exceed 365."),
    (
        "RENTAL_OVERLAP_DAYS_ABOVE_RENTAL_DAYS",
        "Rented days that were also personal-use days cannot exceed the days rented at a fair price.",
    ),
    (
        "RENTAL_STATEMENT_DATE_DIFFERS",
        "The Form 1099-S box 1 date of closing must equal the date sold or the date you gave up this rental.",
    ),
    (
        "RENTAL_STATEMENT_PROCEEDS_DIFFER",
        "The Form 1099-S box 2a total gross proceeds must equal the sale price.",
    ),
    (
        "RENTAL_CREDIT_REDUCTION_ABOVE_BASIS",
        "The credit basis reduction cannot exceed the building or improvement cost less land.",
    ),
    ("RENTAL_LAND_ABOVE_COST", "The land part cannot exceed the cost."),
    ("INSTALLMENT_PROPERTY_REQUIRED", "Enter the property sold."),
    ("INSTALLMENT_SOLD_BEFORE_ACQUIRED", "Enter a valid acquisition date on or before the sale."),
    ("INSTALLMENT_SALE_DATE_INVALID", "Enter a valid sale date in 2026."),
    ("INSTALLMENT_AMOUNT_NEGATIVE", "Enter zero or a positive amount."),
    ("INSTALLMENT_PRICE_REQUIRED", "Enter a positive selling price."),
    (
        "INSTALLMENT_RESALE_BEFORE_SALE",
        "Enter a valid date in 2026 on or after your sale for the related buyer’s disposition.",
    ),
    (
        "ISSUE_INSTALLMENT_QSBS",
        "Installment sales of qualified small business stock require the exclusion history; this sale is not calculated.",
    ),
    (
        "ISSUE_INSTALLMENT_NO_MAXIMUM_PRICE",
        "A contingent installment price without a stated maximum requires a separate basis recovery schedule; this sale is not calculated.",
    ),
    (
        "ISSUE_INSTALLMENT_DEALER_SALE",
        "Dealer sales of property held for customers cannot use this capital installment-sale worksheet.",
    ),
    ("SCHEDULE_B_YES", "Yes"),
    ("SCHEDULE_B_NO", "No"),
    ("SCHEDULE_B_NOT_APPLICABLE", "Not applicable"),
    ("SCHEDULE_B_PAYER", "Reported payer amount"),
    ("SCHEDULE_B_NOMINEE", "Nominee distribution"),
    ("SCHEDULE_B_ESTATE", "After-death amount belonging to the estate or another recipient"),
    ("SCHEDULE_B_PRIOR", "U.S. savings-bond interest previously reported"),
    ("SCHEDULE_B_ACCRUED", "Accrued interest"),
    (
        "SCHEDULE_B_ADJUSTMENT",
        "ABP/OID and source allocation adjustment, including any allocation rounding; uses the retained source calculation",
    ),
    ("SCHEDULE_B_OTHER", "Other reported sources and whole-dollar return rounding"),
    (
        "NOTE_SCHEDULE_B_PART_III",
        "Schedule B Part III: foreign account or signature authority (line 7a first answer): {0}; required to file FBAR (line 7a second answer): {1}; foreign trust (line 8): {2}. An unresolved answer is not a completed filing answer.",
    ),
    ("NOTE_SCHEDULE_B_COUNTRY", "Schedule B line 7b country: {0}."),
    (
        "DEBT_ORIGINAL_ISSUE_INVALID",
        "Use actual ordinary taxable fixed-principal original-issue terms: issue price cannot exceed redemption price. Stripped, tax-exempt, inflation-indexed and installment-principal debt need their own source computation.",
    ),
    (
        "DEBT_DE_MINIMIS_OID_CONFLICT",
        "The original discount is below .25% of redemption price per complete original year. Without an actual include-OID election, retained current originalOID is zero; correct the annual source amounts or election. Coupon interest and market discount keep their separate treatment.",
    ),
    (
        "NOTE_DEBT_DE_MINIMIS_OID",
        "This ordinary fixed-principal original discount is de minimis. No current originalOID is included without an actual include-OID election. Gain at disposition and basis remain governed by their own source records.",
    ),
    (
        "ISSUE_K1_SECTION1231_QBI_SPLIT",
        "The section 199A statement and loss records do not determine the qualified share of the section 1231 lookback recapture. Provide a business-level qualified allocation.",
    ),
    (
        "EXCHANGE_IDENTIFIED_AFTER_RECEIVED",
        "The identification date can't be after the date you received the new property.",
    ),
    (
        "EXCHANGE_LAND_ABOVE_VALUE",
        "The land part can't be more than the value of the property you received.",
    ),
    (
        "EXCHANGE_RENTAL_WITHOUT_BUILDING",
        "A property you rent out needs a building: its land part must be less than its value.",
    ),
    (
        "EXCHANGE_RENTED_BEFORE_RECEIVED",
        "The new property can't be available to rent before you received it.",
    ),
    (
        "EXCHANGE_ENTER_THE_RENTAL_DATE",
        "Enter the 2026 or 2027 date the new property was first available to rent.",
    ),
    (
        "EXCHANGE_LAND_VALUE_ABOVE_TOTAL",
        "The land part of this rental's value can't be more than everything you received for it.",
    ),
    (
        "EXCHANGE_SAY_WHETHER_YOU_EXTEND",
        "You received the new property after April 15, 2027: say whether you extend your 2026 return.",
    ),
    (
        "RENTAL_EXCHANGED_BEFORE_IN_SERVICE",
        "The date you gave up the rental must be after the date it was first available to rent.",
    ),
    (
        "RELATED_EXCHANGE_DISPOSED_BEFORE_TRANSFER",
        "The property was disposed of before the exchange ended; check both dates.",
    ),
    (
        "ISSUE_EXCHANGE_SOMETHING_ELSE",
        "This like-kind exchange is not calculated: other property given up, more than one property, a reverse or build-to-suit exchange, an intermediary's default or a note received need their own statements (Form 8824 lines 12–18, Publication 544).",
    ),
    (
        "ISSUE_EXCHANGE_RECEIVED_BEFORE_TRANSFER",
        "Receiving the new property before giving this rental up is a reverse exchange through an accommodation titleholder, which is not calculated (Revenue Procedure 2000-37).",
    ),
    (
        "ISSUE_EXCHANGE_REPLACEMENT_DISPOSED",
        "You sold or gave away in 2026 the property you received from a related person; that sale and the exchange gain it triggers are not calculated (section 1031(f), Form 8824 line 10).",
    ),
    (
        "ISSUE_EXCHANGE_REPLACEMENT_IN_BUSINESS",
        "The new property is used in your own business; its depreciation belongs to that business and is not calculated (Form 8824, Form 4562).",
    ),
    (
        "ISSUE_EXCHANGE_DATES_OR_AMOUNTS_INVALID",
        "Enter the exchange dates in order, amounts within the supported limits, and a land part no larger than the value you received for this rental.",
    ),
    (
        "ISSUE_RELATED_EXCHANGE_SOMETHING_ELSE",
        "This related-party exchange is not calculated: your own disposition of the property you received, a reduced risk of loss or ordinary depreciation recapture need their own worksheets (Form 8824 Parts II and III, section 1031(f) and (g)).",
    ),
    (
        "ISSUE_RELATED_EXCHANGE_DATES_OR_AMOUNTS_INVALID",
        "Enter the last transfer date in 2024 or 2025, the date of the disposition in 2026 and a deferred gain within the supported limits (Form 8824 lines 4, 6, 9 and 24).",
    ),
    (
        "ISSUE_RELATED_EXCHANGE_GAIN_FROM_A_RENTAL_WITH_UNALLOWED_PASSIVE_LOSSES",
        "Gain from an earlier related-party exchange of rental or passive-activity property is passive income that allows part of this return's unallowed passive losses (Form 8582 lines 1 through 4), which is not calculated.",
    ),
    (
        "ISSUE_RELATED_EXCHANGE_BUSINESS_GAIN_WITH_SECTION_1231_LOOKBACK",
        "Business-property losses from 2021–2025 make part of this year's section 1231 gain ordinary. The part from an earlier related-party exchange of business property needs that business's qualified business income allocation (Form 8995 line 1), which is not calculated.",
    ),
    (
        "WARN_EXCHANGE_TAXED_AS_SALE",
        "{0}: the exchange does not qualify ({1}), so it is taxed as a sale and the property you received has a basis equal to its value.",
    ),
    (
        "EXCHANGE_REASON_IDENTIFIED_LATE",
        "the new property was identified more than 45 days after the transfer",
    ),
    (
        "EXCHANGE_REASON_RECEIVED_LATE",
        "the new property was received more than 180 days after the transfer or after your return's due date",
    ),
    ("EXCHANGE_REASON_PERSONAL", "you use the new property as a home"),
    ("EXCHANGE_REASON_FOREIGN", "U.S. and foreign real property are not like kind"),
    (
        "EXCHANGE_REASON_RELATED",
        "a related person sold you the new property, or disposed of the property you gave them, within two years",
    ),
    (
        "WARN_EXCHANGE_NO_TAX_AVOIDANCE",
        "{0}: this estimate accepts your statement that tax avoidance was not a main purpose (Form 8824 line 11c); attach your explanation. If the IRS does not agree, the deferred gain is taxable.",
    ),
    (
        "NOTE_RELATED_EXCHANGE_TWO_YEARS",
        "{0}: file Form 8824 Parts I and II for 2027 and 2028 too. If you or the related person dispose of the exchanged property before two years after the exchange, the deferred gain becomes taxable that year.",
    ),
    (
        "NOTE_EXCHANGE_LOSSES_STAY_SUSPENDED",
        "{0}: an exchange is not a fully taxable disposition, so this rental's suspended passive losses stay suspended.",
    ),
    (
        "ISSUE_RENTAL_SALE_WITH_REHABILITATION_CREDIT",
        "A rental with a rehabilitation credit that is exchanged for other property or sold by installment is not calculated: the credit's recapture and basis reduction (Form 4255, Form 4797 line 28a) are not carried through Form 8824 or Form 6252.",
    ),
    (
        "REHABILITATION_NEEDS_AN_IMPROVEMENT_ROW",
        "The rehabilitation credit applies only to an improvement placed in service in 2018 or later.",
    ),
    (
        "RENTAL_MIXED_COMPONENT_HISTORY_UNAVAILABLE",
        "A separately disposed component of a mixed-use rental needs its own business basis, regular/AMT depreciation and personal-sale allocation. Whole-building sale records do not establish these component amounts.",
    ),
    (
        "ISSUE_RENTAL_PARTIAL_WITH_DISPOSITION",
        "A part of a building replaced or removed must come before the date the whole property was sold or condemned, and a rental exchanged for other property cannot also have a part disposed of (Form 4797 line 1b and Form 8824).",
    ),
    (
        "ISSUE_RENTAL_SALE_PERSONAL_SHARE_NEEDED",
        "Enter the personal-use share of a property you or your family also used (a percent of the years you owned it).",
    ),
    (
        "NOTE_ASSET_PERSONAL_PART_IN_FULL",
        "{0}: the gain on the personal-use part is counted in full this year; it can also be reported as the payments arrive.",
    ),
    (
        "NOTE_CONDEMNATION_RECAPTURE_LIMITED",
        "{0}: the depreciation recapture is limited to the gain taxed; if the replacement property is land, the recapture may be higher.",
    ),
    (
        "NOTE_PARTIAL_DISPOSITION_REDUCE_COST",
        "{0}: next year, reduce this row's cost by {1,number,currency}, the part disposed of.",
    ),
    (
        "NOTE_RENTAL_SALE_IMPROVEMENT_ALLOCATION",
        "{0}: the building part of the price was divided among the building and its improvements in proportion to their cost.",
    ),
    (
        "NOTE_RENTAL_SALE_PERSONAL_LOSS",
        "{0}: the personal-use part of the sale is a {1,number,currency} loss, which is not deductible.",
    ),
    (
        "RENTAL_SALE_PERSONAL_SHARE_NEEDED",
        "Enter the personal-use share of a property you also used.",
    ),
    ("CONDEMNATION_LAND_ABOVE_AWARD", "The land part of the award cannot be more than the award."),
    (
        "ISSUE_RENTAL_EXCHANGE_WITH_PERSONAL_USE",
        "The like-kind exchange of a rental property you or your family also used is not calculated (Form 8824 with the home-use rules of Publication 523).",
    ),
    ("DISPLAY_DEBT_INSTRUMENT", "Instrument"),
    ("DISPLAY_DEBT_AMOUNTS", "Amounts and other owners"),
    ("DISPLAY_DEBT_HISTORY", "History and disposal"),
    ("DISPLAY_DEBT_COMP_INTEREST", "Interest and OID"),
    ("DISPLAY_DEBT_COMP_EXEMPT", "Exempt interest, foreign tax and discount"),
    ("DISPLAY_DEBT_COMP_PREMIUM", "Premium and acquisition premium"),
    ("DISPLAY_DEBT_COMP_ACCRUED", "Accrued interest"),
    ("DISPLAY_DIV1099_PAYER", "Payer and boxes 1a–1b"),
    ("DISPLAY_DIV1099_OTHER", "Other boxes and state boxes 14–16"),
    ("DISPLAY_OID_OWNER_INCOME", "OID, Treasury OID and discount"),
    ("DISPLAY_OID_OWNER_PREMIUM", "Bond premium and exempt OID"),
    ("DISPLAY_SALE_B_PROCEEDS", "Sale, proceeds and basis"),
    ("DISPLAY_SALE_B_KIND", "What was sold and withholding"),
    ("DISPLAY_E_LINES_5_9", "Lines 5–9"),
    ("DISPLAY_E_LINES_10_13", "Lines 10–13"),
    ("DISPLAY_E_LINES_14_19", "Lines 14–19"),
];
