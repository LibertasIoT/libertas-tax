#[allow(unused_imports)]
use super::*;
pub use crate::vehicle_loan_source::Form1098VliPaper;

/// Qualified tips and overtime on this W-2
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalWorkDeductions {
    /// Qualified tips to evaluate
    pub tips_claim: TipsClaim,
    /// Qualified overtime to evaluate
    pub overtime_claim: OvertimeClaim,
}
impl FederalWorkDeductions {
    /// No qualified tips and no qualified overtime.
    pub(crate) const NONE: Self =
        Self { tips_claim: TipsClaim::No, overtime_claim: OvertimeClaim::No };
}

/// Vehicle loan
/// Form 1098-VLI, or your lender's statement of 2026 interest
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct FederalVehicleInterest {
    /// RECIPIENT'S/LENDER'S name, TIN and address
    pub lender: crate::PrintedParty,
    /// PAYER'S/BORROWER'S name, TIN and address
    pub borrower: crate::PrintedParty,
    /// Interest paid in 2026 (Form 1098-VLI box 1)
    /// Interest only, not principal or fees. If the lender sent no form, use its statement.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub interest: i64,
    /// Box 5 refunds of overpaid interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub refund: i64,
    /// Box 6 vehicle original use commenced with the borrower
    #[libertas_default(false)]
    pub original_use: bool,
    /// Box 7 vehicle final assembly occurred in the United States
    #[libertas_default(false)]
    pub assembled_in_us: bool,
}

impl From<&Form1098VliPaper> for FederalVehicleInterest {
    fn from(paper: &Form1098VliPaper) -> Self {
        let Form1098VliPaper {
            // These printed facts are retained for display and import, not loan calculations.
            calendar_year: _,
            corrected: _,
            lender,
            lender_telephone: _,
            payer_of_record,
            account_number: _,
            box_1,
            box_2a: _,
            box_2b: _,
            box_2c: _,
            box_2d: _,
            box_3a: _,
            box_3b: _,
            box_4: _,
            box_5,
            box_6,
            box_7,
        } = paper;
        Self {
            lender: lender.clone(),
            borrower: payer_of_record.clone(),
            interest: *box_1,
            refund: box_5.unwrap_or(0),
            original_use: *box_6,
            assembled_in_us: *box_7,
        }
    }
}

/// Qualified tips to evaluate
/// Choose Yes to enter the amounts and eligibility facts; No has no additional fields.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum TipsClaim {
    /// No
    No,
    /// Yes
    Yes {
        /// Qualified tips in this employer's W-2 box 12 code TP
        /// Include only voluntary cash/charged or shared tips in an IRS-listed tipped occupation. These tips are already in W-2 wages; do not add them as new income.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        w2_tips: i64,
        /// Qualified tips included in this employer's Form 4137 line 1 column c
        /// This column includes reported and unreported cash/charged tips. Enter only its qualified portion; the corresponding employee-tip source above supplies the unreported wage income. The App takes the larger of this amount and W-2 code TP, not their sum.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        form4137_tips: i64,
        /// Occupation, reporting and applicable 2026 specified-service-business requirements are satisfied
        /// W-2 box 12 code TP and the employer-reported Treasury occupation code show it. No takes no deduction for these tips.
        tips_eligible: Option<bool>,
    },
}

impl FederalWorkDeductions {
    pub(crate) fn tips(&self) -> i64 {
        match &self.tips_claim {
            TipsClaim::No => 0,
            TipsClaim::Yes { w2_tips, form4137_tips, .. } => phase::floor(*w2_tips, *form4137_tips),
        }
    }

    /// The qualified tips W-2 box 12 code TP reports.
    pub(crate) fn w2_tips(&self) -> i64 {
        match &self.tips_claim {
            TipsClaim::No => 0,
            TipsClaim::Yes { w2_tips, .. } => *w2_tips,
        }
    }

    pub(crate) fn form4137_tips(&self) -> i64 {
        match &self.tips_claim {
            TipsClaim::No => 0,
            TipsClaim::Yes { form4137_tips, .. } => *form4137_tips,
        }
    }

    pub(crate) fn tips_valid(&self) -> bool {
        match &self.tips_claim {
            TipsClaim::No => true,
            TipsClaim::Yes { w2_tips, form4137_tips, .. } => {
                (0..=100000000000).contains(w2_tips) && (0..=100000000000).contains(form4137_tips)
            }
        }
    }

    pub(crate) fn tips_eligible(&self) -> bool {
        match &self.tips_claim {
            TipsClaim::No => false,
            TipsClaim::Yes { tips_eligible, .. } => *tips_eligible == Some(true),
        }
    }
    #[cfg(test)]
    pub(crate) fn set_tips_eligible(&mut self, value: bool) {
        if matches!(self.tips_claim, TipsClaim::No) {
            self.tips_claim =
                TipsClaim::Yes { w2_tips: 0, form4137_tips: 0, tips_eligible: Some(false) };
        }
        if let TipsClaim::Yes { tips_eligible, .. } = &mut self.tips_claim {
            *tips_eligible = Some(value);
        }
    }
}

/// Qualified overtime to evaluate
/// Choose Yes to enter the amounts and eligibility facts; No has no additional fields.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum OvertimeClaim {
    /// No
    No,
    /// Yes
    Yes {
        /// Qualified FLSA overtime premium reported by this employer for 2026
        /// Enter only the qualifying premium above the regular rate, not total overtime wages.
        /// The amount must already be included in this W-2's wages.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        overtime: i64,
        /// Employee and premium qualify under federal FLSA rules and the 2026 reporting requirements
        /// W-2 box 12 code TT shows the premium the employer reports as FLSA-required. No takes no deduction for this premium.
        overtime_eligible: Option<bool>,
    },
}

impl FederalWorkDeductions {
    pub(crate) fn overtime(&self) -> i64 {
        match &self.overtime_claim {
            OvertimeClaim::No => 0,
            OvertimeClaim::Yes { overtime, .. } => *overtime,
        }
    }
    pub(crate) fn overtime_eligible(&self) -> bool {
        match &self.overtime_claim {
            OvertimeClaim::No => false,
            OvertimeClaim::Yes { overtime_eligible, .. } => *overtime_eligible == Some(true),
        }
    }
}
