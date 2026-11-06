// Ministerial housing and approved religious-exemption source facts.
// https://www.irs.gov/publications/p517
use crate::common::Owner;
use crate::kernel::phase;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Status for self-employment tax
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ClergyRole {
    /// Minister of a church (ordained, commissioned or licensed), including a cantor or Christian Science reader
    /// Your pay for ministerial services, including as the church's employee, is self-employment income (Schedule SE line 2) unless your Form 4361 exemption is approved.
    Minister {
        /// Housing the church provides or designates
        #[libertas_default(None)]
        housing: MinisterHousing,
        /// Your Form 4361 exemption from self-employment tax is approved
        /// The IRS returned your Form 4361 marked approved; it is irrevocable and covers only ministerial earnings. An application still pending or denied: No (the tax is due).
        form_4361: bool,
        /// Unreimbursed expenses of your ministerial work as a church employee
        /// Not deductible for income tax; subtracted from your self-employment earnings. Leave zero if none or if your Form 4361 is approved.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        employee_expenses: i64,
    },
    /// Member of a religious order who has not taken a vow of poverty
    ReligiousOrderMember {
        /// Your Form 4361 exemption from self-employment tax is approved
        form_4361: bool,
        /// Unreimbursed expenses of your work for the order as its employee
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        employee_expenses: i64,
    },
    /// Christian Science practitioner
    ChristianSciencePractitioner {
        /// Your Form 4361 exemption from self-employment tax is approved
        form_4361: bool,
        /// Unreimbursed employee expenses of your practice
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        employee_expenses: i64,
    },
    /// Member of a recognized religious sect with an approved Form 4029
    /// No self-employment tax on any of your self-employment income. Wages are free of Social Security and Medicare tax only when the employer also holds an approved Form 4029 (its W-2 shows "Form 4029" in box 14 and no boxes 3 to 6).
    ReligiousSect,
}

/// Housing from the church
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MinisterHousing {
    /// None
    None,
    /// A parsonage the church provides
    Parsonage {
        /// Fair rental value of the parsonage for 2026, furnished
        /// What it would cost to rent a comparable furnished home in a similar location. Excluded from income; added to your self-employment earnings.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        fair_rental_value: i64,
        /// Allowance designated for utilities
        /// The amount the church designated in advance for utility costs; W-2 box 1 leaves it out. Zero if none.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        utility_allowance: i64,
        /// Utilities you actually paid
        /// Any allowance above this is taxable (Form 1040 line 1h "Excess allowance").
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        utilities_paid: i64,
    },
    /// A housing or rental allowance the church designated
    Allowance {
        /// Amount officially designated in advance as a housing or rental allowance for 2026, including any utility part
        /// From the church's resolution, budget or your contract; W-2 box 1 leaves it out.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        designated: i64,
        /// Amount you used in 2026 to rent or provide your home, including utilities
        /// Rent or mortgage payments, repairs, furnishings, utilities and similar costs of the home.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        spent: i64,
        /// Fair rental value of the home for 2026, furnished, plus the cost of utilities
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        fair_rental_value: i64,
    },
}

/// Clergy record for one person on this return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ClergyClaim {
    /// Person whose ministerial services or approved religious exemption this records
    pub owner: Owner,
    /// Ministerial status and its applicable housing and exemption facts
    pub role: ClergyRole,
}

impl ClergyRole {
    pub(crate) fn housing_amounts(&self) -> [i64; 3] {
        match self {
            Self::Minister { housing, .. } => housing.amounts(),
            _ => [0; 3],
        }
    }

    pub(crate) fn valid(&self) -> bool {
        let amount = |v: i64| (0..=100000000000).contains(&v);
        amount(self.employee_expenses())
            && match self {
                Self::Minister {
                    housing:
                        MinisterHousing::Parsonage {
                            fair_rental_value,
                            utility_allowance,
                            utilities_paid,
                        },
                    ..
                } => [*fair_rental_value, *utility_allowance, *utilities_paid]
                    .into_iter()
                    .all(amount),
                Self::Minister {
                    housing: MinisterHousing::Allowance { designated, spent, fair_rental_value },
                    ..
                } => [*designated, *spent, *fair_rental_value].into_iter().all(amount),
                _ => true,
            }
    }
    pub(crate) fn exempt(&self) -> bool {
        match self {
            Self::Minister { form_4361, .. }
            | Self::ReligiousOrderMember { form_4361, .. }
            | Self::ChristianSciencePractitioner { form_4361, .. } => *form_4361,
            Self::ReligiousSect => false,
        }
    }
    pub(crate) fn employee_expenses(&self) -> i64 {
        match self {
            Self::Minister { employee_expenses, .. }
            | Self::ReligiousOrderMember { employee_expenses, .. }
            | Self::ChristianSciencePractitioner { employee_expenses, .. } => *employee_expenses,
            Self::ReligiousSect => 0,
        }
    }
}
impl MinisterHousing {
    // All source and intermediate amounts are cents. Each existing downstream
    // form applies its own whole-dollar boundary, never an extra rounding here.
    fn amounts(&self) -> [i64; 3] {
        match *self {
            Self::None => [0; 3],
            Self::Parsonage { fair_rental_value, utility_allowance, utilities_paid } => {
                let utilities = phase::cap(utility_allowance, [utilities_paid]);
                [
                    fair_rental_value + utilities,
                    utility_allowance - utilities,
                    fair_rental_value + utility_allowance,
                ]
            }
            Self::Allowance { designated, spent, fair_rental_value } => {
                let excluded = phase::cap(designated, [spent, fair_rental_value]);
                [excluded, designated - excluded, designated]
            }
        }
    }
}
