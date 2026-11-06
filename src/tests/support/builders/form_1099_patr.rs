extern crate std;

use crate::tests::*;

pub(in crate::tests) fn cooperative_paper(form: Patr1099) -> Patr1099Source {
    let Patr1099 {
        mut payer,
        recipient,
        patronage,
        nonpatronage,
        per_unit_retains,
        withholding,
        redeemed,
        more,
    } = form;
    let PatrMore {
        section_199a_g,
        qualified_payments,
        qualified_items,
        sstb_items,
        specified_cooperative,
        allocations,
        credit_facts,
        other_credits_and_deductions,
    } = more;
    let CoopCreditFactsInput {
        payer_ein,
        investment,
        work_opportunity,
        investment_shares,
        other,
        participation,
        activity,
        notice,
    } = credit_facts;
    if payer.federal_id.is_empty() {
        payer.federal_id = payer_ein;
    } else {
        assert!(
            payer_ein.is_empty() || payer.federal_id == payer_ein,
            "a cooperative has one actual printed payer EIN"
        );
    }
    Patr1099Source {
        form: Patr1099Paper {
            payer,
            recipient,
            patronage,
            nonpatronage,
            per_unit_retains,
            withholding,
            redeemed,
            section_199a_g,
            qualified_payments,
            qualified_items,
            sstb_items,
            specified_cooperative,
            investment,
            work_opportunity,
            other_credits_and_deductions,
            calendar_year: 2026,
            ..Default::default()
        },
        more: Patr1099Answers {
            allocations: allocations.into_iter().map(Into::into).collect(),
            credit_facts: Patr1099CreditAnswers {
                investment_shares,
                other: other.into_iter().map(Into::into).collect(),
                participation,
                activity,
                notice,
            },
        },
    }
}

pub(in crate::tests) fn cooperative_source(data: FederalCooperative) -> Patr1099Source {
    let FederalCooperative {
        patronage,
        nonpatronage,
        per_unit_retains,
        withholding,
        redeemed,
        section_199a_g,
        qualified_payments,
        qualified_items,
        sstb_items,
        specified_cooperative,
        allocations,
        credit_facts,
        other_credits_and_deductions,
    } = data;
    cooperative_paper(Patr1099 {
        payer: Default::default(),
        recipient: Default::default(),
        patronage,
        nonpatronage,
        per_unit_retains,
        withholding,
        redeemed,
        more: PatrMore {
            section_199a_g,
            qualified_payments,
            qualified_items,
            sstb_items,
            specified_cooperative,
            allocations: allocations.into_iter().map(Into::into).collect(),
            credit_facts: credit_facts.into(),
            other_credits_and_deductions,
        },
    })
}

std::thread_local! {
    static COOPERATIVE_ANSWERS: core::cell::RefCell<Vec<(Patr1099Paper, Patr1099Answers)>> =
        const { core::cell::RefCell::new(Vec::new()) };
}

pub(super) fn remember_cooperative_answers(paper: Patr1099Paper, answers: Patr1099Answers) {
    COOPERATIVE_ANSWERS.with(|rows| {
        let mut rows = rows.borrow_mut();
        rows.retain(|(source, _)| source != &paper);
        rows.push((paper, answers));
    });
}

pub(in crate::tests) fn fixture_cooperative_answers(
    paper: &Patr1099Paper,
) -> Option<Patr1099Answers> {
    COOPERATIVE_ANSWERS.with(|rows| {
        rows.borrow()
            .iter()
            .rev()
            .find(|(source, _)| source == paper)
            .map(|(_, answers)| answers.clone())
    })
}

pub(in crate::tests) trait CooperativePaperFixture {
    fn cooperative_document(self) -> Patr1099Paper;
}

impl CooperativePaperFixture for Patr1099Paper {
    fn cooperative_document(self) -> Patr1099Paper {
        self
    }
}

impl CooperativePaperFixture for Patr1099 {
    fn cooperative_document(self) -> Patr1099Paper {
        cooperative_document(cooperative_paper(self))
    }
}

impl CooperativePaperFixture for Patr1099Source {
    fn cooperative_document(self) -> Patr1099Paper {
        remember_cooperative_answers(self.form.clone(), self.more);
        self.form
    }
}

pub(in crate::tests) fn cooperative_document(
    source: impl CooperativePaperFixture,
) -> Patr1099Paper {
    source.cooperative_document()
}
