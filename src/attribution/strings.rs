pub(crate) const ATTRIBUTION_STRINGS: &[(&str, &str)] = &[
    ("RENTAL_ENERGY_BUILDING_CHOICE", "{0}"),
    (
        "EXPENSE_PERSON_OWNER_REQUIRED",
        "Supply the actual taxpayer, spouse or both ownership for this expense or retained carryover. An originating entity does not identify the individual owner.",
    ),
    ("COMMUNITY_PERCENT_OUT_OF_RANGE", "Enter a community part from 1 to 99 percent."),
    (
        "ISSUE_8379_JOINT_RETURN_REQUIRED",
        "Form 8379 line 2: an injured spouse allocation needs a joint return; you are not an injured spouse.",
    ),
    (
        "ISSUE_8379_SPOUSE_ONLY_DEBT_REQUIRED",
        "Form 8379 line 3: only a joint refund applied to past-due federal tax, state income tax, state unemployment compensation, child support, spousal support or a federal nontax debt owed by the other spouse alone can be allocated; you are not an injured spouse.",
    ),
    (
        "ISSUE_8379_INJURED_SPOUSE_OWES_DEBT",
        "Form 8379 line 4: an injured spouse who is legally obligated to pay the past-due amount is not an injured spouse. For a federal tax debt you both owe, see innocent spouse relief (Form 8857).",
    ),
    (
        "ISSUE_8379_W2_WAGES_UNATTRIBUTABLE",
        "Form 8379 line 13a: the W-2 wages on Form 1040 line 1a cannot be attributed between you.",
    ),
    (
        "ISSUE_8379_SOCIAL_SECURITY_UNATTRIBUTABLE",
        "Form 8379 line 13b: the App cannot attribute the final taxable Social Security benefits ({0,number,currency}) between spouses. The ordinary federal estimate includes this income; the Form 8379 refund allocation is unavailable.",
    ),
    (
        "ISSUE_8379_CAPITAL_UNATTRIBUTABLE",
        "Form 8379 line 13b: the App cannot attribute the final capital gain or loss ({0,number,currency}) between spouses after the return limits. The ordinary federal estimate includes this amount; the Form 8379 refund allocation is unavailable.",
    ),
    (
        "ISSUE_8379_BUSINESS_UNATTRIBUTABLE",
        "Form 8379 line 13b: the App cannot attribute the final business or farm income ({0,number,currency}) between spouses after the return limits. The ordinary federal estimate includes this amount; the Form 8379 refund allocation is unavailable.",
    ),
    (
        "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE",
        "Form 8379 line 13b: the App cannot attribute the final other income ({0,number,currency}) between spouses after the return limits. The ordinary federal estimate includes this amount; the Form 8379 refund allocation is unavailable.",
    ),
    (
        "ISSUE_8379_WITHHOLDING_UNTRACED",
        "Form 8379 line 19: federal income tax withheld on Form 1040 line 25d includes withholding the App cannot trace to either spouse's document.",
    ),
    (
        "ISSUE_8379_EIC_UNATTRIBUTABLE",
        "Form 8379: the earned income credit on Form 1040 line 27a cannot be allocated, because neither spouse's separate earned income or loss explains it.",
    ),
    (
        "ISSUE_8379_NO_ELIGIBLE_REFUND_SHARE",
        "Form 8379 lines 6-9: the injured spouse made no payments, had no withholding, claims no earned income credit or additional child tax credit and no refundable credit; you are not an injured spouse.",
    ),
    (
        "NOTE_INJURED_SPOUSE_COMMUNITY_ESTIMATE",
        "Form 8379 estimate: the injured spouse's share of the joint refund is about {0,number,currency}. The IRS makes the final computation after you file: in a community property state it applies the state's rules; for a debt other than federal tax, half the refund other than the earned income credit, plus the injured spouse's part of the earned income credit, is generally the injured spouse's.",
    ),
    (
        "NOTE_INJURED_SPOUSE_IRS_ESTIMATE",
        "Form 8379 estimate: the injured spouse's share of the joint refund is about {0,number,currency}. The IRS makes the final computation after you file: it figures each spouse's married filing separately tax from this allocation, shares the joint tax in that proportion, and refunds the injured spouse's withholding, payments and credits less their share.",
    ),
    ("PAYMENT_AMOUNT_INVALID", "Enter a positive amount for each dated estimated payment."),
    ("INJURED_COMPONENT_ACTC", "Additional child tax credit: claimants of eligible children"),
    (
        "INJURED_COMPONENT_EXCESS_BUSINESS_LOSS",
        "Excess business loss addback (Schedule 1, line 8p)",
    ),
    (
        "INJURED_COMPONENT_NIIT_ITEMIZED_DEDUCTION",
        "Investment-income portion of shared itemized expenses",
    ),
    (
        "INJURED_COMPONENT_NIIT_RETAINED_USE",
        "Retained Form 8960 excluded-loss difference used this year",
    ),
    (
        "WARN_8379_EBL_CAPITAL_OWNER_UNAVAILABLE",
        "Form 8379 needs actual ownership of the native Form 461 business-capital allowance before allocating its excess business loss. The ordinary federal estimate is unchanged.",
    ),
    (
        "INJURED_COMPONENT_CAPITAL_LOSS",
        "Actual joint capital-loss deduction: permitted shared part",
    ),
    (
        "INJURED_COMPONENT_INVESTMENT_ELECTED_CAPITAL",
        "Investment capital gains elected into ordinary income",
    ),
    (
        "INJURED_COMPONENT_INVESTMENT_ELECTED_DIVIDENDS",
        "Qualified dividends elected into ordinary income",
    ),
    (
        "WARN_8379_INVESTMENT_ELECTION_OWNER_UNAVAILABLE",
        "The actual eligible investment sources of the Form 4952 election must be established before the Form 8379 liability estimate can be calculated.",
    ),
    (
        "INJURED_COMPONENT_CTC_ODC",
        "Child and other-dependent credit: claimants of eligible dependents",
    ),
    ("INJURED_COMPONENT_ADJUSTMENT", "Joint adjustment: permitted shared part"),
    ("INJURED_COMPONENT_EDUCATOR", "Educator deduction"),
    ("INJURED_COMPONENT_IRA", "Actual allowed IRA deduction"),
    (
        "INJURED_COMPONENT_STUDENT_LOAN",
        "Student-loan deduction for loans the injured spouse is legally obligated to pay",
    ),
    ("INJURED_COMPONENT_ITEMIZED", "Itemized deductions attributable to the injured spouse"),
    ("INJURED_COMPONENT_CHARITY", "Supplemental non-itemizer charitable deduction"),
    ("INJURED_COMPONENT_VEHICLE_INTEREST", "Supplemental vehicle-loan interest deduction"),
    ("INJURED_COMPONENT_SCHEDULE_1A_QBI", "Supplemental Schedule 1-A or QBI deduction"),
    (
        "INJURED_COMPONENT_QBI_DPAD",
        "Supplemental cooperative deduction attributable to the injured spouse",
    ),
    (
        "INJURED_COMPONENT_NONREFUNDABLE",
        "Other nonrefundable credits attributable to the injured spouse",
    ),
    ("INJURED_COMPONENT_REFUNDABLE", "Other refundable credits attributable to the injured spouse"),
    ("INJURED_COMPONENT_OTHER_TAX", "Joint additional tax attributable to the injured spouse"),
    (
        "INJURED_COMPONENT_TAXABLE_SOCIAL_SECURITY",
        "Actual joint taxable Social Security attributable to the injured spouse",
    ),
    (
        "INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT",
        "Allocate each current component once, within its permitted amount. Refresh the page after changing ownership facts.",
    ),
    (
        "INJURED_SPOUSE_CLAIMANTS_NOT_CURRENT",
        "Identify the separate-return claimant for each current dependent exactly once.",
    ),
    (
        "INJURED_SPOUSE_COMBINED_CHILD_CREDIT_ABOVE_CAP",
        "The combined child and additional child credits cannot exceed the ceiling for that spouse's eligible dependents.",
    ),
    (
        "INJURED_SPOUSE_COMMUNITY_FACTS_NOT_CURRENT",
        "Choose the applicable state and debt facts from the current choices, without repeating a state.",
    ),
    (
        "INJURED_SPOUSE_LOAN_BORROWER_REQUIRED",
        "Identify who is legally obligated on the loans whose interest is deducted.",
    ),
    ("INJURED_COMPONENT_LTC", "Taxable benefits from this long-term-care period"),
    (
        "INJURED_COMPONENT_CARE_CREDIT",
        "Allowed care credit attributable to the injured spouse’s qualifying people",
    ),
    ("INJURED_COMPONENT_EDUCATION_CREDIT", "Education credit within the joint tax limit"),
    ("INJURED_COMPONENT_ADOPTION_REFUND", "Refundable adoption credit"),
    ("INJURED_COMPONENT_HOME_FUEL", "Home fuel tax credit"),
    ("INJURED_COMPONENT_REGULAR_NOL", "Jointly limited regular loss from this retained year"),
    (
        "INJURED_LTC_PAYEE_TOTAL",
        "The actual source payments in this period must total its periodic benefits.",
    ),
    ("INJURED_LTC_SOURCE_TWICE", "Enter each existing statement once in this period."),
    (
        "INJURED_LTC_SOURCE_UNAVAILABLE",
        "Choose an existing periodic-payment statement linked to this insured person.",
    ),
    (
        "INJURED_LTC_SOURCE_TOTAL",
        "Payments attributed across the periods must reconcile to each linked statement’s actual periodic payments.",
    ),
    (
        "INJURED_PRIOR_CARE_PERSON_INVALID",
        "Identify each actual prior-year qualifying person and their separate-return claimant; choose an existing person or name someone not on this return.",
    ),
    ("INJURED_COMPONENT_NONREFUNDABLE_REMAINDER", "Other nonrefundable credits"),
    ("INJURED_COMPONENT_REFUNDABLE_REMAINDER", "Other refundable credits"),
    ("INJURED_COMPONENT_FOREIGN_CREDIT", "Foreign tax credit"),
    (
        "WARN_8379_FOREIGN_CREDIT_OWNER_UNAVAILABLE",
        "The Form 8379 refund estimate needs the foreign credit's final source-owner amounts. Your ordinary federal estimate and known Form 8379 amounts are retained.",
    ),
    (
        "WARN_8379_BUSINESS_CREDIT_OWNER_UNAVAILABLE",
        "The Form 8379 refund estimate needs each business credit owner's final allowed and used amounts. Your ordinary federal estimate and known Form 8379 amounts are retained.",
    ),
    (
        "WARN_8379_REFUNDABLE_OWNER_UNAVAILABLE",
        "The Form 8379 refund estimate needs final owner amounts for business or fund refundable credits. Your ordinary federal estimate and known Form 8379 amounts are retained.",
    ),
    (
        "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE",
        "The Form 8379 refund estimate needs the final source-owner allocation of additional tax. Your ordinary federal estimate and known Form 8379 amounts are retained.",
    ),
    (
        "WARN_8379_QBI_OWNER_UNAVAILABLE",
        "The Form 8379 refund estimate needs final QBI amounts by owner, including activities, REIT/PTP income and loss history. Your ordinary QBI deduction and federal estimate are retained.",
    ),
    (
        "WARN_8379_ADJUSTMENT_OWNER_UNAVAILABLE",
        "The Form 8379 refund estimate is unavailable because a final adjustment cannot be traced to its owner. Your ordinary federal estimate is retained.",
    ),
    (
        "WARN_8379_ACCOUNT_OWNER_UNAVAILABLE",
        "The account income allocation cannot be represented in the Form 8379 result. The ordinary federal estimate remains available.",
    ),
    (
        "WARN_8379_RETIREMENT_OWNER_UNAVAILABLE",
        "The Form 8379 refund estimate is unavailable because final retirement income does not reconcile to its owner amounts. Your ordinary federal estimate is retained.",
    ),
    (
        "WARN_8379_ALLOCATION_CHOICES_NEEDED",
        "Complete the permitted shared allocations and borrower or claimant facts before relying on the Form 8379 refund estimate. Your ordinary federal estimate is retained.",
    ),
    (
        "WARN_8379_COMMUNITY_ESTIMATE_UNAVAILABLE",
        "The Form 8379 refund estimate needs a verified governing state, debt regime and complete property sources. Your ordinary federal estimate is retained.",
    ),
    (
        "WARN_8379_DEPENDENT_CREDIT_OWNER_UNAVAILABLE",
        "The Form 8379 refund estimate needs final credit amounts for the differently assigned qualifying people. Your ordinary federal estimate is retained.",
    ),
    (
        "WARN_8379_FARMLAND_ESTIMATE_UNAVAILABLE",
        "Form 1062 remains in your ordinary federal estimate. The combined Form 8379 refund estimate is unavailable; known Part III amounts remain available.",
    ),
    (
        "WARN_8379_JOINT_PAYMENT_SPLIT_NEEDED",
        "Both separate tax amounts are zero. Enter the spouses' agreed division of joint payments before relying on the Form 8379 refund estimate.",
    ),
    (
        "WARN_8379_PAYMENT_OWNER_UNAVAILABLE",
        "The Form 8379 refund estimate needs each payment's actual payer, including any estate or trust allocation. Your ordinary federal estimate is retained.",
    ),
    ("INJURED_STATE_ARIZONA", "Arizona"),
    ("INJURED_STATE_CALIFORNIA", "California"),
    ("INJURED_STATE_IDAHO", "Idaho"),
    ("INJURED_STATE_LOUISIANA", "Louisiana"),
    ("INJURED_STATE_NEVADA", "Nevada"),
    ("INJURED_STATE_NEW_MEXICO", "New Mexico"),
    ("INJURED_STATE_TEXAS", "Texas"),
    ("INJURED_STATE_WASHINGTON", "Washington"),
    ("INJURED_STATE_WISCONSIN", "Wisconsin"),
    (
        "INJURED_SPOUSE_ESTIMATED_ABOVE_PAYMENTS",
        "The injured spouse's part cannot exceed your joint estimated tax and extension payments.",
    ),
    (
        "PAYMENT_DATE_OUTSIDE_2026",
        "Each 2026 estimated payment must be dated from January 1, 2026 through April 15, 2027.",
    ),
    (
        "WARN_8379_MORTGAGE_TAX_OWNER_UNAVAILABLE",
        "The injured-spouse estimate needs the actual source owner of mortgage subsidy recapture tax. The ordinary joint tax is unchanged.",
    ),
    ("DISPLAY_INJURED_DEBT", "Debt and residence"),
    ("DISPLAY_INJURED_JOINT", "Joint payments and allocations"),
    ("DISPLAY_INJURED_ROWS", "Allocation rows"),
    (
        "NOTE_COMMUNITY_CARRY_IS_THIS_RETURNS_SHARE",
        "{0}: the amounts carried to 2027 for this item are this return's share of the whole interest; your spouse's return carries the rest.",
    ),
];
