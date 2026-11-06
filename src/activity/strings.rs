pub(crate) const ACTIVITY_STRINGS: &[(&str, &str)] = &[
    (
        "QBI_AGGREGATION_FACTS",
        "Review this selected QBI aggregation’s eligibility, election history and complete annual business disclosures. The current facts do not establish a supported aggregation, so its estimate cannot be completed.",
    ),
    (
        "QBI_AGGREGATION_HISTORY",
        "Review the retained QBI election history for these actual businesses. Keep or explicitly correct their history in the aggregation facts; clearing or changing a group name does not end a continuing election or erase unavailable or known IRS history.",
    ),
    (
        "BUSINESS_RISK_RECAPTURE_CHARACTER_REQUIRED",
        "Prior deducted business losses require their original income, self-employment and qualified-business-income character before at-risk recapture can be included.",
    ),
    (
        "BUSINESS_RISK_ORIGINAL_FORM_HISTORY_REQUIRED",
        "Prior business losses require their original schedule and tax character before the at-risk owner can release them.",
    ),
    (
        "RISK_RECAPTURE_HISTORY_REQUIRED",
        "Supply the prior deducted losses and recapture history needed for this negative closing at-risk balance.",
    ),
    (
        "WARN_QBI_SPECIFIED_SERVICE_ASSUMED",
        "Assumed a business you weren't sure about is a specified service business (for a loss, that it isn't one); if that's wrong, your qualified business income deduction may be higher.",
    ),
    (
        "ISSUE_QBI_SERVICE_BUSINESS_AGGREGATION",
        "A specified service trade or business cannot join an aggregation group (Form 8995-A Schedule B); remove its group above the income threshold.",
    ),
    (
        "NOTE_QBI_AGGREGATION_ELECTION",
        "Aggregation elected under Reg. 1.199A-4 for group {0}; attach Schedule B (Form 8995-A). Combined QBI {1,number,currency}, W-2 wages {2,number,currency}, UBIA {3,number,currency} share one wage/property limit.",
    ),
    ("DISPLAY_SPART_TESTS", "Participation tests"),
    ("DISPLAY_SPART_PRIOR", "Prior years and personal service"),
    ("DISPLAY_SPART_MANAGEMENT", "Management and other businesses"),
];
