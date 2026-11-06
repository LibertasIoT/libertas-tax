// Broker paper stays intact. These runtime views forward each supported
// component to the same native sale and section 1256 owners as manual records.
use crate::broker_document::BrokerProjection;
use crate::{
    BrokerDocument, FederalDraft, FederalIncome, FederalIncomeEntry, FederalSale,
    FederalSection1256,
};
use alloc::borrow::Cow;

pub(crate) fn projection(document: &BrokerDocument) -> BrokerProjection {
    let mut view = document.projection();
    view.sale = view.sale.filter(|sale| sale.special.notice().is_none());
    view.contracts = view.contracts.filter(|contracts| !contracts.special_treatment);
    view
}

pub(crate) fn sale(income: &FederalIncome) -> Option<Cow<'_, FederalSale>> {
    match income {
        FederalIncome::Sale { data } => Some(Cow::Owned(FederalSale::from(data))),
        FederalIncome::Broker { data } => projection(data).sale.map(Cow::Owned),
        _ => None,
    }
}

pub(crate) fn contracts(income: &FederalIncome) -> Option<Cow<'_, FederalSection1256>> {
    match income {
        FederalIncome::Section1256 { data } => Some(Cow::Owned(FederalSection1256::from(data))),
        FederalIncome::Broker { data } => projection(data).contracts.map(Cow::Owned),
        _ => None,
    }
}

// Existing consumer scopes describe which native components they support.
// A broker wrapper cannot hide an unknown or differently classified receipt.
pub(crate) fn known_components(
    document: &BrokerDocument,
    allow_sale: bool,
    allow_contracts: bool,
) -> bool {
    let view = projection(document);
    view.sale_boundary.is_none()
        && view.contracts_boundary.is_none()
        && view.receipt.is_none_or(|receipt| receipt.amount == 0)
        && (allow_sale || view.sale.is_none())
        && (allow_contracts || view.contracts.is_none())
}

// Form 6781 runs before the ordinary per-entry collector's community pass.
// Project the new document's native amount through the existing ownership
// fraction here; never divide or rewrite its printed box 11.
pub(crate) fn contracts_amount(draft: &FederalDraft, entry: &FederalIncomeEntry) -> Option<i64> {
    let amount = contracts(&entry.income)?.aggregate;
    if !matches!(entry.income, FederalIncome::Broker { .. }) {
        return Some(amount);
    }
    let share = draft.setup.as_ref().and_then(|setup| {
        crate::community::share(setup, entry.owner.owned(), entry.community, &entry.income)
    });
    Some(share.map_or(amount, |share| share.gain(amount)))
}

// Actual source withholding, before owner/community projection. Group records
// do not repeat the underlying payment documents. Wide sums preserve source
// values while the financial owner checks its required native carrier.
pub(crate) fn source_withholding(entry: &FederalIncomeEntry) -> i128 {
    match &entry.income {
        FederalIncome::Broker { data } => i128::from(data.projection().withholding),
        FederalIncome::Wage { data } => i128::from(crate::FederalWage::from(data).withholding),
        FederalIncome::EmployeeWages { data } => i128::from(data.withholding),
        FederalIncome::Interest { data } => {
            i128::from(crate::FederalInterest::from(data).withholding)
        }
        FederalIncome::Oid { data } => i128::from(crate::FederalOid::from(data).withholding),
        FederalIncome::Dividend { data } => {
            i128::from(crate::FederalDividend::from(data).withholding)
        }
        FederalIncome::Partnership { data } => {
            data.separate.as_ref().map_or(0, |separate| i128::from(separate.backup_withholding))
        }
        FederalIncome::Scorp { data } => i128::from(crate::ScorpK1::from(data).withholding),
        FederalIncome::EstateTrustPortfolio { data } => {
            i128::from(crate::FederalEstateTrustPortfolioK1::from(data).backup_withholding)
        }
        FederalIncome::QualifiedSmallBusinessSale { data } => i128::from(data.withholding),
        FederalIncome::HomeSale { data } => i128::from(crate::HomeSale::from(data).firpta_credit()),
        FederalIncome::TrumpDeath { data } => i128::from(data.withholding.unwrap_or(0)),
        FederalIncome::WildfireRelief { data } => i128::from(data.withholding),
        FederalIncome::DisasterRelief { data } => i128::from(data.withholding),
        FederalIncome::CasualtyLivingInsurance { data } => data
            .payments
            .iter()
            .filter(|payment| payment.received_on >= crate::TAX_YEAR_START_DATE)
            .map(|payment| i128::from(payment.withholding))
            .sum(),
        FederalIncome::OtherIncome { data } => i128::from(data.withholding),
        FederalIncome::Nec1099 { data } => {
            i128::from(crate::FederalForm1099Nec::from(data).withholding)
        }
        FederalIncome::Misc1099 { data } => i128::from(crate::Form1099Misc::from(data).withholding),
        FederalIncome::K1099 { data } => {
            i128::from(crate::FederalForm1099K::from(data).withholding)
        }
        FederalIncome::NuaSale { data } => i128::from(data.withholding),
        FederalIncome::EmployeeStockSale { data } => i128::from(data.withholding),
        FederalIncome::EsppSale { data } => i128::from(data.withholding),
        FederalIncome::EsppSaleSource { data } => i128::from(data.withholding),
        FederalIncome::IsoSaleSource { data } => i128::from(data.withholding),
        FederalIncome::NuaSaleSource { data } => i128::from(data.withholding),
        FederalIncome::IsoSale { data } => i128::from(data.withholding),
        FederalIncome::Sale { data } => i128::from(crate::FederalSale::from(data).withholding),
        FederalIncome::Roth { data } => i128::from(data.withholding.unwrap_or(0)),
        FederalIncome::Retirement { data } => {
            i128::from(crate::FederalRetirement::from(data).withholding)
        }
        FederalIncome::SocialSecurity { data } => {
            i128::from(crate::FederalSocial::from(data).withholding)
        }
        FederalIncome::Gambling { data } => {
            i128::from(crate::FederalGambling::from(data).withholding)
        }
        FederalIncome::Section1256 { data } => {
            i128::from(crate::FederalSection1256::from(data).withholding)
        }
        FederalIncome::GovernmentPayment { data } => {
            i128::from(data.form.federal_income_tax_withheld)
        }
        FederalIncome::Cooperative { data } => i128::from(data.form.withholding),
        FederalIncome::Business { data } => {
            let own = i128::from(data.withholding);
            match &data.activity {
                crate::BusinessActivity::Gambling { withholding, .. } => {
                    own + i128::from(*withholding)
                }
                crate::BusinessActivity::Farming { data } => {
                    own + i128::from(data.insurance.withholding())
                        + data
                            .crp
                            .iter()
                            .map(|payment| i128::from(payment.withholding))
                            .sum::<i128>()
                        + data.ccc.as_ref().map_or(0, |ccc| i128::from(ccc.withholding))
                }
                _ => own,
            }
        }
        FederalIncome::FarmRental { data } => {
            i128::from(data.farm.insurance.withholding())
                + data.farm.ccc.as_ref().map_or(0, |ccc| i128::from(ccc.withholding))
        }
        FederalIncome::HealthLoanAssistance { .. }
        | FederalIncome::DisasterMitigation { .. }
        | FederalIncome::PersonalDebt { .. }
        | FederalIncome::InheritedIra { .. }
        | FederalIncome::LtcBenefits { .. }
        | FederalIncome::Able { .. }
        | FederalIncome::TrumpAbleRollover { .. }
        | FederalIncome::Royalty { .. }
        | FederalIncome::NonbusinessProperty { .. }
        | FederalIncome::SmallBusinessStock { .. }
        | FederalIncome::WorthlessSecurity { .. }
        | FederalIncome::NonbusinessBadDebt { .. }
        | FederalIncome::ReturnOfCapital { .. }
        | FederalIncome::UndistributedGain { .. }
        | FederalIncome::HobbySales { .. }
        | FederalIncome::Scholarship { .. }
        | FederalIncome::JuryDuty { .. }
        | FederalIncome::Alimony { .. }
        | FederalIncome::ForeignEmployment { .. }
        | FederalIncome::Rental { .. }
        | FederalIncome::PropertyAcquisition { .. }
        | FederalIncome::BondRedemption { .. }
        | FederalIncome::SavingsBonds { .. }
        | FederalIncome::ChildElection { .. }
        | FederalIncome::EducationDistribution { .. }
        | FederalIncome::EducationAccountExpenses { .. }
        | FederalIncome::MedicalDistribution { .. }
        | FederalIncome::LtcInsured { .. }
        | FederalIncome::OpportunityFund { .. }
        | FederalIncome::OpportunityFundDeferral { .. }
        | FederalIncome::PartnershipActivities { .. }
        | FederalIncome::ScorpActivities { .. }
        | FederalIncome::InstallmentSale { .. } => 0,
    }
}

pub(crate) fn financial_issue(draft: &FederalDraft) -> Option<(i64, &'static str)> {
    if !draft.income.iter().any(|entry| matches!(entry.income, FederalIncome::Broker { .. })) {
        return None;
    }
    let mut withheld = 0_i128;
    for entry in &draft.income {
        let Some(total) = withheld.checked_add(source_withholding(entry)) else {
            return Some((entry.id, "BROKER_WITHHOLDING_ARITHMETIC_UNAVAILABLE"));
        };
        if i64::try_from(total).is_err() {
            return Some((entry.id, "BROKER_WITHHOLDING_ARITHMETIC_UNAVAILABLE"));
        }
        withheld = total;
    }
    crate::section1256::financial_issue(draft)
}
