use super::paper_documents::remember;
use crate::tests::*;
use alloc::string::String;

pub(in crate::tests) fn w2g_source(form: W2g) -> W2gSource {
    let W2g { payer, winner, winnings, withholding, special, state_details } = form;
    W2gSource {
        form: FormW2GPaper {
            corrected: false,
            calendar_year: Default::default(),
            payer,
            winner,
            winnings,
            withholding: Some(withholding),
            states: state_details.states,
            localities: state_details.localities,
            payer_telephone: String::new(),
            date_won: None,
            type_of_wager: String::new(),
            transaction: String::new(),
            race: String::new(),
            identical_wagers: Some(0),
            cashier: String::new(),
            window: String::new(),
            first_identification: String::new(),
            second_identification: String::new(),
        },
        special,
    }
}

pub(in crate::tests) fn gambling_source(data: FederalGambling) -> crate::GamblingSource {
    if data.own_records {
        assert_eq!(data.withholding, 0);
        crate::GamblingSource::OwnRecords(crate::GamblingLosses {
            other_winnings: data.winnings,
            losses: data.losses,
            special: !data.ordinary,
        })
    } else {
        assert_eq!(data.losses, 0);
        crate::GamblingSource::W2g(
            w2g_source(crate::W2g {
                payer: Default::default(),
                winner: Default::default(),
                winnings: data.winnings,
                withholding: data.withholding,
                special: !data.ordinary,
                state_details: Default::default(),
            })
            .into(),
        )
    }
}

pub(in crate::tests) fn gambling_losses(losses: i64) -> FederalIncomeEntry {
    income(FederalIncome::Gambling {
        data: crate::GamblingSource::OwnRecords(crate::GamblingLosses {
            other_winnings: 0,
            losses: losses * 100,
            special: false,
        }),
    })
}

pub(in crate::tests) fn w2g_document(form: W2g) -> FormW2GPaper {
    let source = w2g_source(form);
    let form = source.form;
    remember(
        IncomeSource::Gambling(form.clone()),
        vec![DocumentAnswer::GamblingSpecial { value: source.special }],
    );
    form
}
