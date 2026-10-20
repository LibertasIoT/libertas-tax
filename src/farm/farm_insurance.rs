use crate::FederalResult;
use crate::validation::ValidationIssue;
use alloc::vec::Vec;
use libertas_macros::*;

/// Crop insurance and federal crop-disaster payments
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FarmInsurance {
    /// None
    None,
    /// Payment and election records
    Payments {
        /// Current-year insurance and disaster income before any election
        /// Cash method: proceeds actually received in 2026. Accrual method: amounts recognized in 2026. Split a mixed revenue policy into its documented physical-loss and price-only portions. Include eligible federal physical-crop disaster/prevented-planting payments; do not duplicate these amounts in other farm receipts or subsidies.
        current: Vec<CropPayment>,
        /// 2025 proceeds previously elected into 2026 income
        /// Copy retained prior-year deferral records. This amount is included now and cannot be deferred again. Accrual returns with this prior cash-method election need an accounting-change worksheet.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        previous_deferred: i64,
        /// Election for this farming business
        /// One election covers every eligible crop payment of this trade or business. Do not split the same business into entries to defer only selected payments.
        election: CropDeferral,
        /// Federal income tax actually withheld in 2026
        /// Copy the 2026 payment statements, including voluntary disaster-payment withholding. Deferring income does not defer this payment credit. Exclude withholding already credited in 2025 or entered elsewhere.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        withholding: i64,
    },
}

/// Crop payment source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CropPayment {
    /// Physical crop loss or qualifying federal crop-disaster payment
    /// Insurance yield-loss portion, or federal payment for physical crop damage/destruction or inability to plant caused by a natural disaster. Exclude payments based solely on prices or a rainfall index without actual crop damage.
    Physical {
        /// Identified damaged crop from the insurer and farm records
        #[libertas_size(min = 1)]
        crop: alloc::string::String,
        /// Gross proceeds attributable to physical loss
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
        /// Year of physical crop damage or prevented planting
        // Tax structure: this source year must be no later than the 2026 return year.
        #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
        year: u32,
    },
    /// Nondeferrable insurance portion
    /// Price-only revenue losses, rainfall-index payments without physical crop damage, and livestock revenue insurance with no qualifying crop-damage deferral.
    Other {
        /// Gross proceeds
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
    },
}
impl CropPayment {
    fn amount(&self) -> i64 {
        match self {
            Self::Physical { amount, .. } | Self::Other { amount } => *amount,
        }
    }
}

/// Crop-insurance income timing
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CropDeferral {
    /// Include current-year income
    Include,
    /// Elect eligible 2026 physical-loss proceeds into 2027
    /// The supported estimate uses retained records establishing that more than 50% of each affected crop's income normally belongs in a year after the damage year; exactly 50% is insufficient. Mixed practices retain current income pending their separate eligibility treatment. A section 451(f)/Regulations 1.451-6 election statement covering all eligible proceeds for this business is attached to the return. Retain crop, damage cause/date, insurer, receipt date and amount records. A made election is binding unless the IRS approves a change; the App does not file it.
    Defer {
        /// Actual normal sales practice for every identified damaged crop
        /// The current estimate computes the election when every affected crop has more than half its normal income in a following year. Mixed practices need their separate election eligibility treatment; the estimate keeps current income and leaves the deferral out.
        #[libertas_size(min = 1)]
        practice: Vec<crate::FarmCropPractice>,
    },
}
impl FarmInsurance {
    pub(crate) fn source_issue(&self) -> Option<ValidationIssue> {
        let Self::Payments { current, election, .. } = self else {
            return None;
        };
        for (i, payment) in current.iter().enumerate() {
            if matches!(payment, CropPayment::Physical { crop, .. } if crop.trim().is_empty()) {
                return Some(
                    ValidationIssue::new("CROP_PRACTICE_SOURCE")
                        .at_field("crop")
                        .in_list("current", i),
                );
            }
        }
        if let CropDeferral::Defer { practice } = election {
            if practice.is_empty() || practice.len() > 400 {
                return Some(
                    ValidationIssue::new("CROP_PRACTICE_SOURCE")
                        .at_field("practice")
                        .in_field("election"),
                );
            }
            for (i, record) in practice.iter().enumerate() {
                let field = if record.crop.trim().is_empty() {
                    Some("crop")
                } else if !(1..=100_000_000_000).contains(&record.normal_total) {
                    Some("normal_total")
                } else if !(0..=100_000_000_000).contains(&record.normal_following) {
                    Some("normal_following")
                } else if practice[..i].iter().any(|other| other.crop == record.crop) {
                    Some("crop")
                } else {
                    None
                };
                if let Some(field) = field {
                    return Some(
                        ValidationIssue::new("CROP_PRACTICE_SOURCE")
                            .at_field(field)
                            .in_list("practice", i)
                            .in_field("election"),
                    );
                }
            }
            for (i, payment) in current.iter().enumerate() {
                if matches!(payment, CropPayment::Physical { crop, .. } if !practice.iter().any(|record| &record.crop == crop))
                {
                    return Some(
                        ValidationIssue::new("CROP_PRACTICE_SOURCE")
                            .at_field("crop")
                            .in_list("current", i),
                    );
                }
            }
            if let Some((i, _)) = practice
                .iter()
                .enumerate()
                .find(|(_, record)| record.normal_following > record.normal_total)
            {
                return Some(
                    ValidationIssue::new("CROP_PRACTICE_ABOVE_TOTAL")
                        .at_field("normal_following")
                        .in_list("practice", i)
                        .in_field("election"),
                );
            }
        }
        None
    }
    pub(crate) fn source_rule(&self) -> Option<&'static str> {
        self.source_issue().map(|issue| issue.key)
    }
    fn practice_established(&self) -> bool {
        let Self::Payments { current, election: CropDeferral::Defer { practice }, .. } = self
        else {
            return false;
        };
        self.source_rule().is_none()
            && current.iter().all(|payment| match payment {
                CropPayment::Physical { crop, year, .. }
                    if *year == crate::year_constants::TAX_YEAR =>
                {
                    practice.iter().any(|record| {
                        &record.crop == crop
                            && record.normal_following
                                > crate::kernel::ratio::trunc_i64(record.normal_total, 2)
                    })
                }
                _ => true,
            })
    }
    pub(crate) fn valid(&self) -> bool {
        self.source_rule().is_none()
            && match self {
                Self::None => true,
                Self::Payments { current, previous_deferred, withholding, .. } => {
                    (0..=100000000000).contains(previous_deferred)
                        && (0..=100000000000).contains(withholding)
                        && current.len() <= 400
                        && current.iter().all(|p| {
                            (0..=100000000000).contains(&p.amount())
                                && match p {
                                    CropPayment::Physical { year, .. } => {
                                        (1900..=crate::year_constants::TAX_YEAR).contains(year)
                                    }
                                    _ => true,
                                }
                        })
                }
            }
    }
    fn eligible(&self) -> i64 {
        match self {
            Self::None => 0,
            Self::Payments { current, .. } => current
                .iter()
                .filter_map(|p| match p {
                    CropPayment::Physical { amount, year, .. }
                        if *year == crate::year_constants::TAX_YEAR =>
                    {
                        Some(*amount)
                    }
                    _ => None,
                })
                .sum(),
        }
    }
    fn deferred(&self, cash: bool) -> i64 {
        // Only a cash-method Defer election defers; otherwise every payment
        // is current income; `report` names the omitted optional deferral.
        if cash
            && self.practice_established()
            && matches!(self, Self::Payments { election: CropDeferral::Defer { .. }, .. })
        {
            self.eligible()
        } else {
            0
        }
    }
    pub(crate) fn income(&self, cash: bool) -> i64 {
        match self {
            Self::None => 0,
            Self::Payments { current, previous_deferred, .. } => {
                current.iter().map(CropPayment::amount).sum::<i64>() + previous_deferred
                    - self.deferred(cash)
            }
        }
    }
    pub(crate) fn withholding(&self) -> i64 {
        match self {
            Self::None => 0,
            Self::Payments { withholding, .. } => *withholding,
        }
    }
    pub(crate) fn report(
        &self,
        cash: bool,
        id: i64,
        form: crate::farm::FarmForm,
        r: &mut FederalResult,
    ) {
        let input = form.input("insurance");
        let Self::Payments { current, previous_deferred, election, withholding } = self else {
            return;
        };
        crate::require_localized!(
            r,
            cash || *previous_deferred == 0,
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Crop insurance accounting",
            "ISSUE_PRIOR_CASH_METHOD_CROP_DEFERRALS_ON_AN_ACCRUAL_METHOD_RETURN_REQUIR_B6A28F1E",
            "Prior cash-method crop deferrals on an accrual-method return require an accounting-change worksheet.",
        );
        if matches!(election, CropDeferral::Defer { .. }) {
            if !cash || self.eligible() == 0 || !self.practice_established() {
                // Current income is known. Omit only the unestablished optional
                // deferral, including mixed practices without an owner ruling.
                crate::federal_rules::note_once_scoped(
                    r,
                    msg!("WARN_CROP_PRACTICE_DEFERRAL_LEFT_OUT"),
                    crate::IssueContext::Section(crate::TaxSection::Farming),
                );
            } else {
                r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Crop-insurance deferral selected: attach the required section 451(f)/Regulations 1.451-6 statement and retain the 2027 income amount. The election covers all eligible proceeds of this farming business and is binding unless the IRS approves a change. The App does not file the statement.", msg!("NOTE_CROP_INSURANCE_DEFERRAL_SELECTED_ATTACH_THE_REQUIRED_SECTION_451_42E57F41")));
            }
        }
        for (name, value, note) in [
            (
                "crop_insurance_before_deferral",
                current.iter().map(CropPayment::amount).sum(),
                msg!("EXPLAIN_FARM_N_N_CROP_INSURANCE_BEFORE_DEFERRAL"),
            ),
            (
                "crop_insurance_deferred_next_year",
                self.deferred(cash),
                msg!("EXPLAIN_FARM_N_N_CROP_INSURANCE_DEFERRED_N"),
            ),
            (
                "crop_insurance_current_taxable",
                current.iter().map(CropPayment::amount).sum::<i64>() - self.deferred(cash),
                msg!("EXPLAIN_FARM_N_N_CROP_INSURANCE_CURRENT_TAXABLE"),
            ),
            (
                "crop_insurance_prior_included",
                *previous_deferred,
                msg!("EXPLAIN_FARM_N_N_CROP_INSURANCE_PRIOR_INCLUDED"),
            ),
            (
                "crop_insurance_taxable",
                self.income(cash),
                msg!("EXPLAIN_FARM_N_N_CROP_INSURANCE_TAXABLE"),
            ),
            (
                "crop_insurance_withholding",
                *withholding,
                msg!("EXPLAIN_FARM_N_N_CROP_INSURANCE_WITHHOLDING"),
            ),
        ] {
            // Schedule F lines 6a, 6b and 6d on the cash method; Part III has the
            // single line 41 for the accrual method, which cannot defer, so it
            // holds the current proceeds. Form 4835 lines 5a, 5b and 5d.
            let form_line = match (form, name) {
                (crate::farm::FarmForm::Form4835, "crop_insurance_before_deferral") => Some("5a"),
                (crate::farm::FarmForm::Form4835, "crop_insurance_current_taxable") => Some("5b"),
                (crate::farm::FarmForm::Form4835, "crop_insurance_prior_included") => Some("5d"),
                (crate::farm::FarmForm::ScheduleF, "crop_insurance_before_deferral") if cash => {
                    Some("6a")
                }
                (crate::farm::FarmForm::ScheduleF, "crop_insurance_current_taxable") => {
                    Some(if cash { "6b" } else { "41" })
                }
                (crate::farm::FarmForm::ScheduleF, "crop_insurance_prior_included") if cash => {
                    Some("6d")
                }
                _ => None,
            }
            .map(|line| crate::FormLine::new(form.tax_form(), line));
            crate::kernel::lines::push_line(
                r,
                form_line,
                &alloc::format!("farm_{id}_{name}"),
                value,
                &[&input],
                "https://www.irs.gov/publications/p225",
                note,
            );
        }
    }
}

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_enum!(FarmInsurance {
    records: [Payments { amounts: [current, previous_deferred, election, withholding], facts: [] }],
    units: [None],
    tuples: [],
});

crate::community::divide_enum!(CropDeferral {
    records: [Defer { amounts: [], facts: [practice] }],
    units: [Include],
    tuples: [],
});

crate::community::divide_enum!(CropPayment {
    records: [
        Physical { amounts: [amount], facts: [crop, year] },
        Other { amounts: [amount], facts: [] }
    ],
    units: [],
    tuples: [],
});
