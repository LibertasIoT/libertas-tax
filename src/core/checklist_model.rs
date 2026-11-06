use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport, libertas_bitflags};

// Beyond your documents: three questions about what no document shows,
// asked after the household. This is the stored
// FederalDraft.checklist: every box the person checked (walk-owned data: a
// topic's entries are owned by its box, and every save but this page's keeps
// a box with entries checked). A box a document evidences opens checked and
// is saved unchecked, so a document never becomes the person's answer.
/// Beyond your documents
/// Some things never come with a form. Three quick questions cover them.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Checklist {
    /// What did you do in 2026?
    /// On a joint return, include what your spouse did. A share in a partnership, S corporation, estate or trust is not asked here: its Schedule K-1 brings it in.
    #[libertas_bitflags(ActivityTopics)]
    pub activities: u32,
    /// What did you pay or put away in 2026?
    /// On a joint return, include what your spouse paid.
    #[libertas_bitflags(PaidTopics)]
    pub paid: u32,
    /// Did your 2025 return carry anything into 2026?
    /// These amounts are only on last year's return.
    #[libertas_bitflags(CarryoverTopics)]
    pub carryovers: u32,
}
impl Checklist {
    pub(crate) const NONE: Self = Self { activities: 0, paid: 0, carryovers: 0 };
}

/// Beyond your documents
/// Some things never come with a form. Three quick questions cover them.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct ChecklistAnswers {
    /// What did you do in 2026?
    /// On a joint return, include what your spouse did. A share in a partnership, S corporation, estate or trust is not asked here: its Schedule K-1 brings it in.
    #[libertas_bitflags(ChecklistActivities)]
    pub activities: u32,
    /// What did you pay or put away in 2026?
    /// On a joint return, include what your spouse paid.
    #[libertas_bitflags(PaidTopics)]
    pub paid: u32,
    /// Did your 2025 return carry anything into 2026?
    /// These amounts are only on last year's return.
    #[libertas_bitflags(CarryoverTopics)]
    pub carryovers: u32,
}
impl From<Checklist> for ChecklistAnswers {
    fn from(value: Checklist) -> Self {
        Self {
            activities: value.activities & !ActivityTopics::BUSINESS.0,
            paid: value.paid,
            carryovers: value.carryovers,
        }
    }
}
impl ChecklistAnswers {
    pub(crate) fn with_preserved_business(self, accepted: Checklist) -> Checklist {
        Checklist {
            activities: (self.activities & !ActivityTopics::BUSINESS.0)
                | (accepted.activities & ActivityTopics::BUSINESS.0),
            paid: self.paid,
            carryovers: self.carryovers,
        }
    }
}

/// What did you do in 2026?
#[libertas_bitflags]
pub struct ChecklistActivities(pub u32);
impl ChecklistActivities {
    /// None of these
    pub const NONE: Self = Self(0);
    /// Rental property, including equipment you rented out and royalties not on a 1099-MISC
    pub const RENTALS: Self = Self(ActivityTopics::RENTALS.0);
    /// A farm or ranch, or farmland rented out for a crop or livestock share
    /// Checked when a 1099-PATR or a 1099-G farm payment is entered.
    pub const FARM: Self = Self(ActivityTopics::FARM.0);
    /// A household employee, such as a nanny, caregiver or housekeeper
    pub const HOUSEHOLD_EMPLOYEE: Self = Self(ActivityTopics::HOUSEHOLD_EMPLOYEE.0);
    /// Income, a job or accounts outside the U.S.
    pub const ABROAD: Self = Self(ActivityTopics::ABROAD.0);
}

/// What did you do in 2026?
#[libertas_bitflags]
pub struct ActivityTopics(pub u32);
impl ActivityTopics {
    /// None of these
    pub const NONE: Self = Self(0);
    /// A business or side work of your own, including gig and app work and a single-member LLC
    /// Checked when a 1099-NEC or 1099-K is entered.
    pub const BUSINESS: Self = Self(1 << 0);
    /// Rental property, including equipment you rented out and royalties not on a 1099-MISC
    pub const RENTALS: Self = Self(1 << 1);
    /// A farm or ranch, or farmland rented out for a crop or livestock share
    /// Checked when a 1099-PATR or a 1099-G farm payment is entered.
    pub const FARM: Self = Self(1 << 2);
    /// A household employee, such as a nanny, caregiver or housekeeper
    pub const HOUSEHOLD_EMPLOYEE: Self = Self(1 << 3);
    /// Income, a job or accounts outside the U.S.
    pub const ABROAD: Self = Self(1 << 4);
}

/// What did you pay or put away in 2026?
#[libertas_bitflags]
pub struct PaidTopics(pub u32);
impl PaidTopics {
    /// None of these
    pub const NONE: Self = Self(0);
    /// Estimated tax payments, a payment with an extension, or last year's refund applied to 2026
    pub const PAYMENTS: Self = Self(1 << 0);
    /// State or local taxes you paid yourself, such as property tax on a home without a mortgage or state estimated payments
    pub const TAXES: Self = Self(1 << 1);
    /// Gifts to charity (cash or property)
    /// From 2026, cash gifts are deductible even without itemizing.
    pub const GIFTS: Self = Self(1 << 2);
    /// IRA contributions
    /// Form 5498 usually arrives after the filing deadline: use your own records.
    pub const IRA_CONTRIBUTIONS: Self = Self(1 << 3);
    /// HSA contributions not made through payroll
    pub const HSA: Self = Self(1 << 4);
    /// Child or dependent care
    pub const CARE: Self = Self(1 << 5);
}

/// Did your 2025 return carry anything into 2026?
#[libertas_bitflags]
pub struct CarryoverTopics(pub u32);
impl CarryoverTopics {
    /// None of these
    pub const NONE: Self = Self(0);
    /// A capital loss carryover
    /// It is deductible even with no sale in 2026.
    pub const CAPITAL_LOSS: Self = Self(1 << 0);
    /// A net operating loss
    pub const NET_OPERATING_LOSS: Self = Self(1 << 1);
    /// A qualified business income (QBI) loss carryforward
    pub const QBI_LOSS: Self = Self(1 << 2);
    /// A charitable contribution carryover
    pub const CHARITY: Self = Self(1 << 3);
    /// A minimum tax credit (Form 8801)
    pub const MINIMUM_TAX_CREDIT: Self = Self(1 << 4);
    /// Basis in nondeductible IRA contributions (Form 8606)
    pub const IRA_BASIS: Self = Self(1 << 5);
    /// Home-use rental expenses carried from your 2025 Form 1040 Schedule E worksheet (vacation home or rented home)
    pub const HOME_USE_RENTAL: Self = Self(1 << 6);
}

// The last page before the review (the walk's step 5). Its income and benefits
// answers plan the pages named by `MISSED_INCOME` and `MISSED_BENEFITS`
// (interview map); its situations are the whole answer of what the App does
// not prepare, stored with them.
/// Situations people often miss
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct OftenMissed {
    /// Income and taxes people often miss
    /// Which of these did you have in 2026, with no form to enter?
    #[libertas_bitflags(MissedIncome)]
    pub income: u32,
    /// Situations this App doesn't prepare
    /// Add each that applies to 2026. Leave the list empty if none does. Each one is left out of the estimate and named as not calculated.
    #[libertas_unordered]
    /// ----
    /// Situation
    pub unprepared: Vec<Unprepared>,
    /// Deductions and credits people often miss
    /// Did any of these apply to you in 2026? Optional: every other deduction and credit is under See all topics.
    #[libertas_bitflags(MissedBenefits)]
    pub benefits: u32,
}

impl OftenMissed {
    // The options checked and the situations listed.
    pub(crate) fn chosen(&self) -> usize {
        (self.income.count_ones() + self.benefits.count_ones()) as usize + self.unprepared.len()
    }
}

/// Income and taxes people often miss
#[libertas_bitflags]
pub struct MissedIncome(pub u32);
impl MissedIncome {
    /// None of these
    pub const NONE: Self = Self(0);
    /// Pay for jury duty
    pub const JURY: Self = Self(1 << 0);
    /// A prize or award, with no form
    pub const PRIZE: Self = Self(1 << 1);
    /// Gambling winnings, with no W-2G
    pub const GAMBLING: Self = Self(1 << 2);
    /// A debt forgiven with no 1099-C, or a student loan repaid by a health-service program
    pub const CANCELED_DEBT: Self = Self(1 << 3);
    /// A home or other property sold, with no 1099-S
    pub const PROPERTY_SOLD: Self = Self(1 << 4);
    /// Property sold in 2026 or earlier that is paid over time (installment payments, interest from financing the buyer)
    pub const INSTALLMENT: Self = Self(1 << 5);
    /// Sales from a hobby, with no 1099-K
    pub const HOBBY: Self = Self(1 << 6);
    /// A scholarship or fellowship, with no 1098-T
    pub const SCHOLARSHIP: Self = Self(1 << 7);
    /// Alimony received under an agreement made before 2019
    pub const ALIMONY: Self = Self(1 << 8);
    /// A casualty or disaster: insurance for living expenses, a hazard mitigation grant or a casualty loss
    pub const CASUALTY: Self = Self(1 << 9);
    /// Income from an earlier disaster distribution (Form 8915-F)
    pub const DISASTER_DISTRIBUTION: Self = Self(1 << 10);
    /// A required minimum distribution you did not take
    pub const MISSED_RMD: Self = Self(1 << 11);
    /// Excess contributions to, or taxable withdrawals from, an IRA, Coverdell, ABLE or Trump account
    pub const ACCOUNT_TAXES: Self = Self(1 << 12);
    /// A credit you must pay back (recapture)
    pub const RECAPTURE: Self = Self(1 << 13);
    /// A property exchange from 2024 or 2025 with a related party
    pub const RELATED_EXCHANGE: Self = Self(1 << 14);
    /// Health insurance from the Marketplace (1095-A)
    pub const MARKETPLACE: Self = Self(1 << 15);
    /// Other income that is not on a form
    pub const OTHER_INCOME: Self = Self(1 << 16);
    /// A credit was denied or reduced on an earlier return (Form 8862)
    /// The child tax credit, credit for other dependents, American opportunity credit or earned income credit.
    pub const DISALLOWED_CREDIT: Self = Self(1 << 17);
    /// Cash or charged tips not reported on a W-2
    pub const CASH_TIPS: Self = Self(1 << 18);
    /// Interest on a private loan you made, with no 1099-INT
    pub const PRIVATE_LOAN_INTEREST: Self = Self(1 << 19);
}

/// Income and taxes people often miss
#[libertas_bitflags]
pub struct MissedIncomeTail(pub u32);
impl MissedIncomeTail {
    /// None of these
    pub const NONE: Self = Self(MissedIncome::NONE.0);
    /// Gambling winnings, with no W-2G
    pub const GAMBLING: Self = Self(MissedIncome::GAMBLING.0);
    /// A debt forgiven with no 1099-C, or a student loan repaid by a health-service program
    pub const CANCELED_DEBT: Self = Self(MissedIncome::CANCELED_DEBT.0);
    /// A home or other property sold, with no 1099-S
    pub const PROPERTY_SOLD: Self = Self(MissedIncome::PROPERTY_SOLD.0);
    /// Property sold in 2026 or earlier that is paid over time (installment payments, interest from financing the buyer)
    pub const INSTALLMENT: Self = Self(MissedIncome::INSTALLMENT.0);
    /// A scholarship or fellowship, with no 1098-T
    pub const SCHOLARSHIP: Self = Self(MissedIncome::SCHOLARSHIP.0);
    /// Alimony received under an agreement made before 2019
    pub const ALIMONY: Self = Self(MissedIncome::ALIMONY.0);
    /// A casualty or disaster: insurance for living expenses, a hazard mitigation grant or a casualty loss
    pub const CASUALTY: Self = Self(MissedIncome::CASUALTY.0);
    /// Income from an earlier disaster distribution (Form 8915-F)
    pub const DISASTER_DISTRIBUTION: Self = Self(MissedIncome::DISASTER_DISTRIBUTION.0);
    /// A required minimum distribution you did not take
    pub const MISSED_RMD: Self = Self(MissedIncome::MISSED_RMD.0);
    /// Excess contributions to, or taxable withdrawals from, an IRA, Coverdell, ABLE or Trump account
    pub const ACCOUNT_TAXES: Self = Self(MissedIncome::ACCOUNT_TAXES.0);
    /// A credit you must pay back (recapture)
    pub const RECAPTURE: Self = Self(MissedIncome::RECAPTURE.0);
    /// A property exchange from 2024 or 2025 with a related party
    pub const RELATED_EXCHANGE: Self = Self(MissedIncome::RELATED_EXCHANGE.0);
    /// Health insurance from the Marketplace (1095-A)
    pub const MARKETPLACE: Self = Self(MissedIncome::MARKETPLACE.0);
    /// A credit was denied or reduced on an earlier return (Form 8862)
    /// The child tax credit, credit for other dependents, American opportunity credit or earned income credit.
    pub const DISALLOWED_CREDIT: Self = Self(MissedIncome::DISALLOWED_CREDIT.0);
    /// Interest on a private loan you made, with no 1099-INT
    pub const PRIVATE_LOAN_INTEREST: Self = Self(MissedIncome::PRIVATE_LOAN_INTEREST.0);
}

/// Deductions and credits people often miss
#[libertas_bitflags]
pub struct MissedBenefits(pub u32);
impl MissedBenefits {
    /// None of these
    pub const NONE: Self = Self(0);
    /// Foreign taxes you paid that are not on a 1099
    pub const FOREIGN_TAX: Self = Self(1 << 0);
    /// Classroom expenses as a K-12 educator
    pub const EDUCATOR: Self = Self(1 << 1);
    /// Large medical and dental expenses
    pub const MEDICAL: Self = Self(1 << 2);
    /// Alimony you paid under an agreement made before 2019
    pub const ALIMONY_PAID: Self = Self(1 << 3);
    /// Investment interest
    pub const INVESTMENT_INTEREST: Self = Self(1 << 4);
    /// A security or loan that became worthless
    pub const WORTHLESS: Self = Self(1 << 5);
    /// Job expenses as an Armed Forces reservist, performing artist or fee-basis official (Form 2106)
    pub const EMPLOYEE_EXPENSES: Self = Self(1 << 6);
    /// A clean vehicle or other energy credit
    pub const ENERGY_CREDIT: Self = Self(1 << 7);
    /// Adoption expenses
    pub const ADOPTION: Self = Self(1 << 8);
    /// Your child's interest and dividends, reported on your return (Form 8814)
    pub const CHILD_INCOME: Self = Self(1 << 9);
    /// Pay a firm reported on a 1099 although you were its employee (Form 8919)
    pub const EMPLOYEE_PAY: Self = Self(1 << 10);
}

/// Situation this App doesn't prepare
// The one owner of what the estimate refuses by name, built from the
// definitions of each refused kind: every RareSituation, the payload-less
// kinds below and every RareAdditionalTax. The page offers exactly these
// types, so a new refusal joins the page when it joins its owner.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Unprepared {
    /// An income, property or deduction situation
    Situation {
        /// Situation
        situation: RareSituation,
    },
    /// Another adjustment to income not listed
    /// Such as jury pay you gave your employer or contributions to a section 501(c)(18)(D) pension plan.
    OtherAdjustments,
    /// Other alternative minimum tax adjustments and preferences (Form 6251)
    /// Incentive stock options you exercised are entered with Form 3921 on Investment sales.
    AlternativeMinimumTax,
    /// Recapture of another credit, or look-back interest on a long-term contract (Forms 8611, 8697, 8866)
    HousingCreditRecapture,
    /// A deferred section 965 tax
    Section965,
    /// Another additional tax from identified source records
    /// Select the actual situation. An uncomputed additional tax prevents the App from completing the federal estimate.
    OtherAdditionalTaxes {
        /// Tax
        kind: crate::RareAdditionalTax,
    },
}

/// Situation that applies to you
// Income and itemized-deduction situations; the other refused kinds are the
// remaining variants of Unprepared, and whole-return choices belong to
// the filers' pages (FederalPerson.whole_return). Each variant is refused by name.
// FederalDraft.often_missed stores the selected values; keep numeric values
// stable when a situation becomes supported.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum RareSituation {
    /// Compensation as a railroad employee representative (Form CT-2)
    RailroadRepresentative = 0,
    /// Residual interest in a REMIC (Schedule Q; Schedule E lines 38 and 39)
    RemicResidual = 1,
    /// Straddles: a loss on a position offset by another position you or your spouse held, or a mixed straddle election (Form 6781 Part II); not covered calls on stock you own or spreads made only of futures or index options
    Straddles = 3,
    /// Special depreciation for specified plants, reuse and recycling property, qualified production property, water utility or railroad property (Form 4562)
    SpecialDepreciation = 4,
    /// A sale of purchased goodwill or another section 197 intangible, or of section 1254 or 1255 property (Form 4797)
    AssetDisposed = 6,
    /// A like-kind exchange of business, farm or investment real property that is not a rental, or a conflict-of-interest sale (Form 8824)
    LikeKindExchange = 7,
    /// Income a tax treaty exempts or re-sources, or another treaty-based return position (Form 8833)
    TaxTreaty = 8,
    /// You are a debtor in a Chapter 11 bankruptcy case with self-employment income
    /// Business income belongs to the bankruptcy estate's Form 1041, while self-employment tax stays on your return. The App does not prepare that estate return.
    Chapter11Debtor = 9,
    /// Income from a foreign corporation or PFIC: section 951 and 951A inclusions (Schedule 1 lines 8n and 8o, Forms 5471 and 8992), section 962 tax, or other net investment income adjustments (Form 8960)
    ForeignCorporation = 10,
    /// Casualty or theft of business, rental or investment property, or a loss from an investment scam (Form 4684)
    Casualty = 11,
}
