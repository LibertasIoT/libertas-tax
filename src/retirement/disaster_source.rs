use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Per-disaster limit for 2021 and later disasters (Form 8915-F line 1a): owned by year_constants.rs.
pub(crate) const LIMIT: i64 = crate::year_constants::DISASTER_DISTRIBUTION_LIMIT_CENTS;

/// This was a qualified disaster distribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum DisasterDistribution {
    /// No
    No,
    /// Yes: my main home was in a FEMA major disaster area and the disaster caused me an economic loss
    /// Loss of or damage to property, displacement from your home, or lost work. No 10% additional tax applies to the part you treat as a qualified disaster distribution.
    Qualified {
        /// FEMA disaster number
        /// The digits of the DR number at fema.gov/disaster/declarations, for example 4834 for DR-4834-FL.
        #[libertas_number(min = 1)]
        fema: u32,
        /// Disaster beginning date
        /// The first day of the incident period.
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20210126, max = 20261231)]
        began: u32,
        /// Disaster declaration date
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20200101, max = 20261231)]
        declared: u32,
        /// Date of this distribution
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20260101, max = 20261231)]
        distributed: u32,
        /// Amount you treat as a qualified disaster distribution
        /// Up to box 1. For one disaster, all your plans and years together allow at most $22,000.
        #[libertas_money("USD", 2)]
        // IRC 72(t)(11): qualified disaster distributions and their repayments share the $22,000 limit.
        #[libertas_number(min = 1, max = $DISASTER_DISTRIBUTION_LIMIT_CENTS)]
        amount: i64,
        /// Qualified disaster distributions for this disaster in earlier years
        /// Form 8915-F line 1b. Enter the same amount on each 1099-R for this disaster.
        #[libertas_money("USD", 2)]
        // IRC 72(t)(11): qualified disaster distributions and their repayments share the $22,000 limit.
        #[libertas_number(min = 0, max = $DISASTER_DISTRIBUTION_LIMIT_CENTS)]
        #[libertas_default(0)]
        earlier: i64,
        /// When to include the taxable part
        #[libertas_default(OverThreeYears)]
        spread: DisasterSpread,
        /// Repaid to a retirement plan or IRA by the time you file
        /// Within 3 years of the distribution. Not a required minimum distribution or a beneficiary's distribution.
        #[libertas_default(No)]
        repaid: DisasterRepayment,
    },
    /// Yes, to buy or build a main home in a disaster area
    /// A hardship distribution from a 401(k) or 403(b) plan, or an IRA first-home distribution, for a main home in the disaster area that was not bought or built because of the disaster (Form 8915-F Part IV).
    HomePurchase {
        /// FEMA disaster number
        /// The digits of the DR number at fema.gov/disaster/declarations, for example 4834 for DR-4834-FL.
        #[libertas_number(min = 1)]
        fema: u32,
        /// Disaster beginning date
        /// The first day of the incident period.
        #[libertas_date_only]
        // Form 8915-F home-purchase relief: the incident and declaration window includes 2027.
        #[libertas_number(min = 20210126, max = 20271231)]
        began: u32,
        /// Disaster ending date
        /// The last day of the incident period.
        #[libertas_date_only]
        // Form 8915-F home-purchase relief: the incident and declaration window includes 2027.
        #[libertas_number(min = 20210126, max = 20271231)]
        ended: u32,
        /// Disaster declaration date
        #[libertas_date_only]
        // Form 8915-F home-purchase relief: the incident and declaration window includes 2027.
        #[libertas_number(min = 20210126, max = 20271231)]
        declared: u32,
        /// Date of this distribution
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20260101, max = 20261231)]
        distributed: u32,
        /// Taxable part repaid to a retirement plan or IRA
        /// Repaid by 180 days after the later of the beginning and declaration dates, and before you file. Repayments after you file need an amended return. Do not include a repayment of an after-tax amount.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        taxable_repaid: i64,
    },
    /// Yes, but the disaster has no FEMA DR number or I am not sure
    NotDeclared,
}

/// Repaid to a retirement plan or IRA by the time you file
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum DisasterRepayment {
    /// Nothing repaid
    No,
    /// Repaid
    Yes {
        /// Total repaid by the time you file
        #[libertas_money("USD", 2)]
        // IRC 72(t)(11): qualified disaster distributions and their repayments share the $22,000 limit.
        #[libertas_number(min = 1, max = $DISASTER_DISTRIBUTION_LIMIT_CENTS)]
        total: i64,
        /// Of that, repaid during 2026
        /// Only the deposits made on or before December 31, 2026. For an IRA distribution this amount is already in your December 31 IRA value, so Form 8606 line 6 subtracts it.
        #[libertas_money("USD", 2)]
        // IRC 72(t)(11): qualified disaster distributions and their repayments share the $22,000 limit.
        #[libertas_number(min = 0, max = $DISASTER_DISTRIBUTION_LIMIT_CENTS)]
        #[libertas_default(0)]
        in_return_year: i64,
    },
}
impl DisasterRepayment {
    pub(crate) fn total(&self) -> i64 {
        match self {
            Self::No => 0,
            Self::Yes { total, .. } => *total,
        }
    }
    pub(crate) fn in_return_year(&self) -> i64 {
        match self {
            Self::No => 0,
            Self::Yes { in_return_year, .. } => *in_return_year,
        }
    }
}

/// When to include the taxable part
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum DisasterSpread {
    /// Over 2026, 2027 and 2028
    OverThreeYears,
    /// All in 2026
    AllInReturnYear,
}

impl DisasterDistribution {
    pub(crate) fn valid_with_source_alignment(&self, gross: i64, source_alignment: bool) -> bool {
        match self {
            Self::Qualified { amount, earlier, repaid, .. } => {
                (1..=LIMIT).contains(amount)
                    && (!source_alignment || *amount <= gross)
                    && (0..=LIMIT - amount).contains(earlier)
                    && (0..=*amount).contains(&repaid.total())
                    && (0..=repaid.total()).contains(&repaid.in_return_year())
                    && (matches!(repaid, DisasterRepayment::No) || repaid.total() > 0)
            }
            Self::HomePurchase { began, ended, declared, distributed, taxable_repaid, .. } => {
                [*began, *ended, *declared].iter().all(|d| {
                    (20210126..=20271231).contains(d) && crate::dates::source_valid_date(*d)
                }) && (20260101..=20261231).contains(distributed)
                    && crate::dates::source_valid_date(*distributed)
                    && (0..=100000000000).contains(taxable_repaid)
            }
            _ => true,
        }
    }
    // The client rules of the 1099-R page about the qualified disaster distribution period,
    // the home purchase window and what else happened to the money, checked again on save
    // (the keys are the client rules' keys), first failure first.
    pub(crate) fn code_issue(
        &self,
        v: &crate::R1099Answered,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::dates::add_days;
        let plain = matches!(
            v.code,
            Some(crate::DistributionCode::Normal { .. })
                | Some(crate::DistributionCode::Early { .. })
        );
        let roth_rolled = match v.code {
            Some(crate::DistributionCode::RothIra { rolled_over, .. }) => Some(rolled_over),
            _ => None,
        };
        match self {
            Self::Qualified { began, declared, distributed, amount, .. } => {
                // Form 8915-F Part I: from the incident's beginning through 179 days after
                // the later of the beginning and declaration dates.
                if !(*distributed >= *began
                    && (*distributed <= add_days(*began, 179)
                        || *distributed <= add_days(*declared, 179)))
                {
                    return Some(
                        crate::validation::ValidationIssue::new("DISASTER_OUTSIDE_PERIOD")
                            .at_field("distributed")
                            .in_field("disaster"),
                    );
                }
                if matches!(v.code, Some(crate::DistributionCode::DirectRollover { .. })) {
                    return Some(
                        crate::validation::ValidationIssue::new("DISASTER_DIRECT_ROLLOVER")
                            .at_field("code"),
                    );
                }
                if let Some(rolled) = roth_rolled
                    && amount + rolled > v.gross
                {
                    return Some(
                        crate::validation::ValidationIssue::new(
                            "DISASTER_AND_ROLLOVER_ABOVE_GROSS",
                        )
                        .at_field("amount")
                        .in_field("disaster"),
                    );
                }
                if plain {
                    let other = match v.code.as_ref().and_then(crate::retirement::disposition_of) {
                        Some(
                            crate::Disposition::RolledOver { amount, .. }
                            | crate::Disposition::Converted { amount }
                            | crate::Disposition::SoldAndRolledOver { amount, .. },
                        ) => *amount,
                        _ => 0,
                    };
                    if amount + other > v.gross {
                        return Some(
                            crate::validation::ValidationIssue::new(
                                "DISASTER_AND_ROLLOVER_ABOVE_GROSS",
                            )
                            .at_field("amount")
                            .in_field("disaster"),
                        );
                    }
                }
                None
            }
            Self::HomePurchase { began, ended, distributed, taxable_repaid, .. } => {
                // Form 8915-F Part IV: received no more than 180 days before the first day and
                // no later than 30 days after the last day of the disaster.
                if !(*distributed >= add_days(*began, -180) && *distributed <= add_days(*ended, 30))
                {
                    Some(
                        crate::validation::ValidationIssue::new("DISASTER_HOME_OUTSIDE_PERIOD")
                            .at_field("distributed")
                            .in_field("disaster"),
                    )
                } else if *ended < *began {
                    Some(
                        crate::validation::ValidationIssue::new("DISASTER_ENDED_BEFORE_BEGAN")
                            .at_field("ended")
                            .in_field("disaster"),
                    )
                } else if v.taxable.is_some_and(|taxable| *taxable_repaid > taxable) {
                    Some(
                        crate::validation::ValidationIssue::new(
                            "DISASTER_HOME_REPAID_ABOVE_TAXABLE",
                        )
                        .at_field("taxable_repaid")
                        .in_field("disaster"),
                    )
                } else if !plain {
                    Some(
                        crate::validation::ValidationIssue::new("DISASTER_HOME_KEPT")
                            .at_field("code"),
                    )
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    // The client rules of the 1099-R page, checked again on save.
    pub(crate) fn issue(&self, gross: i64) -> Option<crate::validation::ValidationIssue> {
        match self {
            Self::Qualified { amount, .. } if *amount > gross => Some(
                crate::validation::ValidationIssue::new("R_DISASTER_ABOVE_GROSS")
                    .at_field("amount"),
            ),
            Self::Qualified { amount, earlier, .. } if amount + earlier > LIMIT => Some(
                crate::validation::ValidationIssue::new("DISASTER_ABOVE_LIMIT").at_field("earlier"),
            ),
            Self::Qualified { amount, repaid, .. } if repaid.total() > *amount => Some(
                crate::validation::ValidationIssue::new("DISASTER_REPAID_ABOVE_AMOUNT")
                    .at_field("total")
                    .in_field("repaid"),
            ),
            Self::Qualified { repaid, .. } if repaid.in_return_year() > repaid.total() => Some(
                crate::validation::ValidationIssue::new("DISASTER_REPAID_2026_ABOVE_REPAID")
                    .at_field("in_return_year")
                    .in_field("repaid"),
            ),
            _ => None,
        }
    }
}
