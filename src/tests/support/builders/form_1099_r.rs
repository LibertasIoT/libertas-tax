use super::paper_documents::remember;
use crate::tests::*;

fn retirement_paper_native(form: R1099) -> Form1099RPaper {
    let R1099 {
        payer,
        payer_telephone,
        account_number,
        recipient,
        gross,
        taxable,
        taxable_not_determined,
        total_distribution,
        withholding,
        employee_contributions,
        box7,
        ira,
        trump,
        excess_contribution_earnings,
        box_3,
        box_6,
        box_8a,
        box_8b_percent,
        box_9a_percent,
        box_9b,
        box_10,
        first_roth_year,
        fatca,
        payment_date,
        state_details,
    } = form;
    Form1099RPaper {
        corrected: false,
        calendar_year: Default::default(),
        payer,
        payer_telephone,
        account_number,
        recipient,
        gross,
        taxable,
        taxable_not_determined,
        total_distribution,
        withholding,
        employee_contributions,
        box7,
        ira,
        trump,
        excess_contribution_earnings,
        box_3,
        box_6,
        box_8a,
        box_8b_percent,
        box_9a_percent,
        box_9b,
        box_10,
        first_roth_year: (first_roth_year != 0).then_some(first_roth_year),
        fatca,
        payment_date,
        state_details,
    }
}

paper_adapter!(
    retirement_paper,
    retirement_paper_native,
    RetirementBoxFixture,
    R1099,
    Form1099RPaper
);

pub(in crate::tests) fn r1099_source(data: FederalRetirement) -> R1099Source {
    let FederalRetirement {
        form_boxes,
        trump,
        box_2a,
        box_3,
        box_6,
        box_8a,
        box_8b_percent,
        box_9a_percent,
        ira,
        box_9b,
        box_10,
        box_10_basis,
        disability,
        disability_wages,
        kind,
        gross,
        taxable,
        withholding,
        box7,
        ordinary_distribution,
        disaster,
        after_death,
    } = data;
    let mut property = None;
    let mut inherited = None;
    let mut ira_basis = false;
    let taxable_method = taxable.map(|taxable| match taxable {
        RetirementTaxable::Payer => TaxableMethod::BoxTwoA,
        RetirementTaxable::IraBasis => {
            ira_basis = true;
            TaxableMethod::BoxTwoA
        }
        RetirementTaxable::Inherited { account } => {
            inherited = Some(account);
            TaxableMethod::BoxTwoA
        }
        RetirementTaxable::Simplified { data } => TaxableMethod::Simplified { data },
        RetirementTaxable::GeneralAnnuity { data } => TaxableMethod::GeneralRule { data },
        RetirementTaxable::VariableAnnuity { data } => TaxableMethod::VariablePayments { data },
        RetirementTaxable::FullDischarge { data } => TaxableMethod::WholeContract { data },
        RetirementTaxable::QualifiedWithdrawal { data } => TaxableMethod::PlanWithdrawal { data },
        RetirementTaxable::DeferredWithdrawal { data } => TaxableMethod::AnnuityWithdrawal { data },
        RetirementTaxable::LegacyAnnuity { data } => TaxableMethod::Pre1982Withdrawal { data },
        RetirementTaxable::CivilServiceDeath { data } => {
            TaxableMethod::CivilServiceLumpSum { data }
        }
        RetirementTaxable::ChildAnnuity { data } => TaxableMethod::ChildAnnuity { data },
        RetirementTaxable::PropertyProceeds { data } => {
            property = Some(data);
            TaxableMethod::BoxTwoA
        }
        RetirementTaxable::Railroad { .. } => panic!("railroad fixture uses retirement_source"),
    });
    let disposition = |amount, into| match property.clone() {
        Some(data) => Disposition::SoldAndRolledOver { data, amount, into },
        None => Disposition::RolledOver { amount, into },
    };
    let normal = |what_happened, early| {
        if early {
            DistributionCode::Early { what_happened, exception: EarlyException::None }
        } else {
            DistributionCode::Normal { what_happened }
        }
    };
    // A disability or death code carries its distribution date as the printed payment date.
    let mut exception_date = None;
    let mut simple_exception = false;
    let code = ordinary_distribution.map(|treatment| match treatment {
        RetirementTreatment::Ordinary => match inherited {
            Some(account) => DistributionCode::Inherited { account },
            None if kind == RetirementKind::Inherited => {
                DistributionCode::Inherited { account: None }
            }
            None => DistributionCode::Normal { what_happened: Disposition::Kept },
        },
        RetirementTreatment::Early => normal(Disposition::Kept, true),
        RetirementTreatment::EarlySimple => DistributionCode::EarlySimple {
            what_happened: SimpleDisposition::Kept { exception: EarlyException::None },
        },
        RetirementTreatment::Rollover { destination, amount, early } => {
            normal(disposition(amount, destination), early)
        }
        RetirementTreatment::Conversion { amount, early } => {
            normal(Disposition::Converted { amount }, early)
        }
        RetirementTreatment::Exception { distributed, facts, second, early_simple } => {
            simple_exception |= early_simple;
            if early_simple {
                DistributionCode::EarlySimple {
                    what_happened: SimpleDisposition::Kept {
                        exception: EarlyException::Applies {
                            distributed,
                            facts,
                            second,
                            repaid: None,
                        },
                    },
                }
            } else if let RetirementException::Disability { began } = &facts {
                exception_date = Some(distributed);
                DistributionCode::Disability { began: *began, wages: disability_wages }
            } else if let RetirementException::Death { died } = &facts {
                exception_date = Some(distributed);
                match inherited {
                    Some(account) => DistributionCode::Inherited { account },
                    None => DistributionCode::Death { died: *died },
                }
            } else {
                DistributionCode::Early {
                    what_happened: Disposition::Kept,
                    exception: EarlyException::Applies { distributed, facts, second, repaid: None },
                }
            }
        }
        RetirementTreatment::Unsupported => DistributionCode::Other,
        RetirementTreatment::LumpSum { data } => {
            if let Some(tax) = data.own_page() {
                DistributionCode::NormalLumpSum { tax }
            } else {
                let (died, tax) = data.decedent_page().expect("decedent election");
                DistributionCode::DeathLumpSum { died, tax }
            }
        }
        RetirementTreatment::Corrective { excess_deferral } => {
            DistributionCode::Corrective { excess_deferral }
        }
        RetirementTreatment::QualifiedCareCharge => DistributionCode::QualifiedCareCharge,
        RetirementTreatment::EmployerRothContribution => DistributionCode::EmployerRothContribution,
    });
    let RetirementPrintedBoxes {
        taxable_not_determined,
        total_distribution,
        employee_contributions,
        excess_contribution_earnings,
        first_roth_year,
        fatca,
        payment_date,
        medicare_premiums: _,
    } = form_boxes;
    // Native worksheet fixtures describe an amount the payer did not settle.
    // Literal paper fixtures go through retirement_paper_native unchanged.
    let taxable_not_determined =
        taxable_not_determined || !matches!(taxable_method, Some(TaxableMethod::BoxTwoA));
    let payment_date = payment_date.or(exception_date);
    let mut box7 = box7;
    // A SIMPLE IRA exception is printed as code S in box 7.
    if simple_exception {
        box7 = DistributionCodes::CODE_S.0;
    }
    // Native fixtures name the account category independently; retain it in its printed code.
    if kind == RetirementKind::Nonqualified
        && box7
            & (DistributionCodes::CODE_D.0
                | DistributionCodes::CODE_F.0
                | DistributionCodes::CODE_W.0)
            == 0
    {
        box7 |= DistributionCodes::CODE_D.0;
    }
    R1099Source {
        form: R1099 {
            payer: Default::default(),
            payer_telephone: alloc::string::String::new(),
            account_number: alloc::string::String::new(),
            recipient: Default::default(),
            gross,
            taxable: Some(box_2a),
            taxable_not_determined,
            total_distribution,
            withholding: Some(withholding),
            employee_contributions: Some(employee_contributions),
            box7,
            ira: ira || kind.ira(),
            trump,
            excess_contribution_earnings: Some(excess_contribution_earnings),
            box_3: Some(box_3),
            box_6: Some(box_6),
            box_8a: Some(box_8a),
            box_8b_percent,
            box_9a_percent,
            box_9b,
            box_10: Some(box_10),
            first_roth_year,
            fatca,
            payment_date,
            state_details: Default::default(),
        },
        code,
        taxable_method,
        disability: Some(disability),
        box_10_basis: box_10_basis.map(|amount| InPlanRothBasis::Known { amount }),
        disaster: Some(disaster),
        after_death,
        ira_basis,
    }
}

macro_rules! fixture_retirement_source_boxes {
    ($value:expr, $gross:expr) => {{
        let value = &$value;
        let mut native = native_retirement(
            RetirementKind::Pension,
            $gross,
            (RetirementTaxable::Payer, value.box_2a),
        );
        native.form_boxes = value.form_boxes.clone();
        native.box_3 = value.box_3;
        native.box_6 = value.box_6;
        native.box_8a = value.box_8a;
        native.box_8b_percent = value.box_8b_percent;
        native.box_9a_percent = value.box_9a_percent;
        native.box_9b = value.box_9b;
        native.box_10 = value.box_10;
        native.box7 = value.box7;
        native.ira = value.ira;
        native.trump = value.trump;
        native.withholding = value.withholding;
        r1099_source(native)
    }};
}

pub(in crate::tests) fn roth_source(value: RothWithdrawal) -> R1099Source {
    let mut source = fixture_retirement_source_boxes!(value, value.gross);
    // Roth withdrawals have no Schedule R disability-pay answer.
    source.disability = None;
    let roth = match value.code {
        RothCode::Qualified => RothIraCode::Qualified,
        RothCode::Early => RothIraCode::Early {
            distributed: value.distributed,
            exception: value.exception.clone(),
            repaid: value.repaid.clone(),
        },
        RothCode::ExceptionApplies => RothIraCode::ExceptionApplies {
            distributed: value.distributed,
            reason: match value.inherited {
                InheritedLink::Yes { account } => RothReason::Inherited { account },
                InheritedLink::No => match &value.exception {
                    Some(RetirementException::Disability { began }) => {
                        RothReason::Disabled { since: *began }
                    }
                    _ => RothReason::Age,
                },
            },
        },
    };
    source.code = Some(DistributionCode::RothIra {
        sequence: value.sequence,
        rolled_over: value.rolled_over,
        roth,
    });
    source.disaster = Some(value.disaster.clone());
    assert_eq!(
        RothWithdrawal::from(&source),
        value,
        "Roth fixture must preserve all native facts in its stored source"
    );
    source
}

pub(in crate::tests) fn trump_death_source(value: TrumpDeathAcquisition) -> R1099Source {
    let mut source = fixture_retirement_source_boxes!(value, value.value);
    source.code =
        Some(DistributionCode::TrumpDeath { estate: value.beneficiary_estate, basis: value.basis });
    assert_eq!(
        TrumpDeathAcquisition::from(&source),
        value,
        "Trump death fixture must preserve all native facts in its stored source"
    );
    source
}

pub(in crate::tests) fn trump_able_source(value: TrumpAbleRollover) -> R1099Source {
    let mut source = fixture_retirement_source_boxes!(value, value.amount);
    source.code = Some(DistributionCode::TrumpToAble { transferred: value.transferred });
    assert_eq!(
        TrumpAbleRollover::from(&source),
        value,
        "Trump ABLE fixture must preserve all native facts in its stored source"
    );
    source
}

pub(in crate::tests) trait RetirementPaperFixture {
    fn answered(self) -> R1099Answered;
}

impl RetirementPaperFixture for R1099Answered {
    fn answered(self) -> R1099Answered {
        self
    }
}

impl RetirementPaperFixture for R1099Source {
    fn answered(self) -> R1099Answered {
        R1099Answered::new(self, Vec::new())
    }
}

pub(in crate::tests) fn retirement_document(source: impl RetirementPaperFixture) -> Form1099RPaper {
    let source = source.answered();
    let public_safety = source.public_safety().to_vec();
    let source = source.into_source();
    use crate::DocumentAnswer as A;
    let form = retirement_paper(source.form);
    remember(
        IncomeSource::Retirement { value: RetirementDocument::R1099(form.clone()) },
        vec![
            A::RetirementTreatment { code: source.code.map(Into::into) },
            A::RetirementMethod { method: source.taxable_method },
            A::RetirementBasis { basis: source.box_10_basis },
            A::RetirementDisability { disability: source.disability },
            A::RetirementDisaster { value: source.disaster },
            A::RetirementPublicSafety { value: crate::PublicSafetyAnswer::of(public_safety) },
            A::RetirementAfterDeath { value: source.after_death },
        ],
    );
    form
}

/// Save a printed retirement statement and reopen its stored paper after a restart.
pub(in crate::tests) fn assert_retirement_paper_reopens(paper: RetirementDocument) {
    use crate::tests::support::assert::accepted;
    use crate::tests::support::builders::native_income::document;
    use crate::tests::support::client::income_pages::{
        open_documents, page_documents, save_documents,
    };
    use crate::tests::support::client::{restart, send};
    let mut session = restart(complete(FederalStatus::Single));
    let source = IncomeSource::Retirement { value: paper };
    let request = save_documents(&mut session, vec![document(0, source.clone())]);
    accepted(&send(&mut session, request));
    let mut session = restart(session.draft);
    let rows = page_documents(&open_documents(&mut session));
    let expected = expected_named_source(&session.draft, Owner::Taxpayer, source);
    assert_eq!(rows.iter().filter(|row| row.document == expected).count(), 1);
}
