use super::paper_documents::remember;
use crate::tests::*;

fn railroad_paper_native(form: Rrb1099R) -> FormRrb1099RPaper {
    let Rrb1099R {
        issuer,
        recipient,
        claim_number_and_payee_code,
        box_3,
        box_4,
        box_5,
        box_6,
        gross,
        box_8,
        withholding,
        medicare_premiums,
    } = form;
    FormRrb1099RPaper {
        issuer,
        claim_number_and_payee_code,
        recipient,
        box_3,
        box_4,
        box_5,
        box_6,
        gross,
        box_8,
        withholding,
        medicare_premiums,
    }
}

paper_adapter!(
    railroad_paper,
    railroad_paper_native,
    RailroadBoxFixture,
    Rrb1099R,
    FormRrb1099RPaper
);

pub(in crate::tests) fn retirement_source(data: FederalRetirement) -> RetirementSource {
    if let Some(RetirementTaxable::Railroad {
        repaid,
        repayments,
        contributory,
        box_5,
        box_6,
        holder,
        basis,
    }) = &data.taxable
    {
        let recovery = basis
            .clone()
            .map_or(RailroadRecovery::None, |data| RailroadRecovery::SimplifiedMethod { data });
        return RetirementSource::Rrb1099R(RailroadSource {
            form: Rrb1099R {
                issuer: Default::default(),
                claim_number_and_payee_code: alloc::string::String::new(),
                recipient: Default::default(),
                box_3: data.box_9b,
                box_4: Some(*contributory),
                box_5: Some(*box_5),
                box_6: Some(*box_6),
                gross: data.gross,
                box_8: Some(*repaid),
                withholding: Some(data.withholding),
                medicare_premiums: Some(data.form_boxes.medicare_premiums),
            },
            // A death exception states the survivor: the stored holder is the source of it.
            recipient_kind: holder.map(|holder| {
                if let Some(RetirementTreatment::Exception {
                    distributed,
                    facts: RetirementException::Death { died },
                    ..
                }) = &data.ordinary_distribution
                {
                    return RailroadRecipient::Survivor {
                        died: *died,
                        distributed: *distributed,
                        recovery: recovery.clone(),
                    };
                }
                match holder {
                    RailroadHolder::Employee => {
                        RailroadRecipient::Employee { recovery: recovery.clone() }
                    }
                    RailroadHolder::Spouse => RailroadRecipient::Spouse,
                    RailroadHolder::Survivor { .. } => {
                        panic!("railroad survivor fixture needs its distribution date")
                    }
                }
            }),
            repayments: repayments.clone(),
        });
    }
    RetirementSource::R1099(r1099_source(data))
}

pub(in crate::tests) fn railroad_document(source: RailroadSource) -> FormRrb1099RPaper {
    use crate::DocumentAnswer as A;
    let form = railroad_paper(source.form);
    remember(
        IncomeSource::Retirement { value: RetirementDocument::Rrb1099R(form.clone()) },
        vec![
            A::RailroadRecipient { value: source.recipient_kind },
            A::RailroadRepayments { value: source.repayments },
        ],
    );
    form
}
