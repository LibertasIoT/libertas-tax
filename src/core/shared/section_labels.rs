use crate::common::Message;
use crate::navigation::TaxSection;

#[cfg(test)]
pub(crate) const INJURED_SPOUSE_SECTION: &str = "Injured spouse allocation (Form 8379)";
// The page's name, and the section of its refusals.
#[cfg(test)]
pub(crate) const PARENT_TAX_SECTION: &str = "Your parent's return (Form 8615)";

// The name of a section or topic as its rows, progress and refusals show it.
pub(crate) fn section_info(topic: TaxSection) -> (&'static str, &'static str) {
    match topic {
        TaxSection::IncomeWithoutForm => {
            (crate::fixture_english!("Income without a form"), "SECTION_INCOME_WITHOUT_FORM")
        }
        TaxSection::Documents => (crate::fixture_english!("Document center"), "SECTION_DOCUMENTS"),
        TaxSection::HomeOffices => {
            (crate::fixture_english!("Business use of your homes"), "SECTION_HOME_OFFICES")
        }
        TaxSection::Wages => (crate::fixture_english!("Wages"), "SECTION_WAGES"),
        TaxSection::InterestDividends => {
            (crate::fixture_english!("Interest and dividends"), "SECTION_INTEREST_DIVIDENDS")
        }
        TaxSection::InvestmentSales => {
            (crate::fixture_english!("Investment sales"), "SECTION_INVESTMENT_SALES")
        }
        TaxSection::Retirement => {
            (crate::fixture_english!("Retirement and Social Security"), "SECTION_RETIREMENT")
        }
        TaxSection::SelfEmployment => {
            (crate::fixture_english!("Self-employment"), "SECTION_SELF_EMPLOYMENT")
        }
        TaxSection::Rentals => (crate::fixture_english!("Rental property"), "SECTION_RENTALS"),
        TaxSection::GovernmentBenefits => (
            crate::fixture_english!("Unemployment and state refunds"),
            "SECTION_GOVERNMENT_BENEFITS",
        ),
        TaxSection::OtherIncome => (
            crate::fixture_english!(
                "Prizes, scholarships, relief payments, account payouts and other income"
            ),
            "SECTION_OTHER_INCOME",
        ),
        TaxSection::LongTermCareBenefits => {
            (crate::fixture_english!("Long-term care benefits"), "SECTION_LONG_TERM_CARE_BENEFITS")
        }
        TaxSection::Gambling => (crate::fixture_english!("Gambling winnings"), "SECTION_GAMBLING"),
        TaxSection::Farming => (crate::fixture_english!("Farming"), "SECTION_FARMING"),
        TaxSection::PassThrough => (
            crate::fixture_english!("Partnerships, S corporations, estates and trusts"),
            "SECTION_PASS_THROUGH",
        ),
        TaxSection::CanceledDebt => {
            (crate::fixture_english!("Canceled debt"), "SECTION_CANCELED_DEBT")
        }
        TaxSection::WorkAbroad => (crate::fixture_english!("Work abroad"), "SECTION_WORK_ABROAD"),
        TaxSection::Care => (crate::fixture_english!("Child and dependent care"), "SECTION_CARE"),
        TaxSection::Hsa => {
            (crate::fixture_english!("Health and medical savings accounts"), "SECTION_HSA")
        }
        TaxSection::SelfEmployedBenefits => (
            crate::fixture_english!("Self-employed health insurance and retirement plan"),
            "SECTION_SELF_EMPLOYED_BENEFITS",
        ),
        TaxSection::IraContributions => {
            (crate::fixture_english!("IRA contributions"), "SECTION_IRA_CONTRIBUTIONS")
        }
        TaxSection::Educator => (crate::fixture_english!("Educator expenses"), "SECTION_EDUCATOR"),
        TaxSection::StudentLoan => {
            (crate::fixture_english!("Student loan interest"), "SECTION_STUDENT_LOAN")
        }
        TaxSection::EmployeeExpenses => (
            crate::fixture_english!("Reservist, performing artist and fee-basis official expenses"),
            "SECTION_EMPLOYEE_EXPENSES",
        ),
        TaxSection::Medical => {
            (crate::fixture_english!("Medical and dental expenses"), "SECTION_MEDICAL")
        }
        TaxSection::Taxes => (crate::fixture_english!("Taxes you paid"), "SECTION_TAXES"),
        TaxSection::Mortgage => {
            (crate::fixture_english!("Home mortgage interest"), "SECTION_MORTGAGE")
        }
        TaxSection::GiftCarryovers => {
            (crate::fixture_english!("Gifts of property and carryovers"), "SECTION_GIFT_CARRYOVERS")
        }
        TaxSection::OtherItemized => (
            crate::fixture_english!("Investment interest and other deductions"),
            "SECTION_OTHER_ITEMIZED",
        ),
        TaxSection::Itemized => {
            (crate::fixture_english!("Itemized deductions"), "SECTION_ITEMIZED")
        }
        TaxSection::Education => {
            (crate::fixture_english!("Education credits"), "SECTION_EDUCATION")
        }
        TaxSection::Saver => {
            (crate::fixture_english!("Retirement savings credit"), "SECTION_SAVER")
        }
        TaxSection::Marketplace => {
            (crate::fixture_english!("Health insurance marketplace"), "SECTION_MARKETPLACE")
        }
        TaxSection::MoreCredits => {
            (crate::fixture_english!("More credits"), "SECTION_MORE_CREDITS")
        }
        TaxSection::MoreAdjustments => {
            (crate::fixture_english!("More adjustments"), "SECTION_MORE_ADJUSTMENTS")
        }
        TaxSection::OtherTaxes => (crate::fixture_english!("Other taxes"), "SECTION_OTHER_TAXES"),
        TaxSection::InjuredSpouse => {
            (crate::fixture_english!(INJURED_SPOUSE_SECTION), "SECTION_INJURED_SPOUSE")
        }
        TaxSection::ParentTax => {
            (crate::fixture_english!(PARENT_TAX_SECTION), "SECTION_PARENT_TAX")
        }
        TaxSection::Adjustments => {
            (crate::fixture_english!("Adjustments to income"), "SECTION_ADJUSTMENTS")
        }
        TaxSection::Deductions => (crate::fixture_english!("Deductions"), "SECTION_DEDUCTIONS"),
        TaxSection::Credits => (crate::fixture_english!("Credits"), "SECTION_CREDITS"),
        TaxSection::Payments => (crate::fixture_english!("Federal payments"), "SECTION_PAYMENTS"),
        TaxSection::Checklist => {
            (crate::fixture_english!("Beyond your documents"), "SECTION_CHECKLIST")
        }
        TaxSection::OftenMissed => {
            (crate::fixture_english!("Situations people often miss"), "SECTION_OFTEN_MISSED")
        }
        TaxSection::Review => (crate::fixture_english!("Review"), "SECTION_REVIEW"),
        TaxSection::Setup => (crate::fixture_english!("Your 2026 return"), "SECTION_SETUP"),
        TaxSection::People => (crate::fixture_english!("About you"), "SECTION_PEOPLE"),
        TaxSection::Spouse => (crate::fixture_english!("About your spouse"), "SECTION_SPOUSE"),
        TaxSection::Children => {
            (crate::fixture_english!("Children and people you support"), "SECTION_CHILDREN")
        }
        TaxSection::IncomeByPeriod => {
            (crate::fixture_english!("Income by period"), "SECTION_INCOME_BY_PERIOD")
        }
        TaxSection::Income => (crate::fixture_english!("Income"), "SECTION_INCOME"),
    }
}
#[cfg(test)]
pub(crate) fn section_name(topic: TaxSection) -> &'static str {
    section_info(topic).0
}
pub(crate) fn section_message(topic: TaxSection) -> Message {
    Message::new(section_info(topic).1, alloc::vec![])
}
