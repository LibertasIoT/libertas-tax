// form_8880 items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// W-2 box 12 elective deferrals and designated Roth contributions (codes D,
// E, F, G, S, AA, BB and EE) of one owner: Form 8880 line 2.
pub(super) fn saver_box12(d: &FederalDraft, owner: Owner) -> i64 {
    d.income
        .iter()
        .filter(|e| e.owner == crate::DocumentOwner::from(owner))
        .map(|e| match &e.income {
            FederalIncome::Wage { data } => crate::FederalWage::from(data)
                .other_box12
                .iter()
                .filter(|c| {
                    matches!(
                        c.code,
                        W2Code::D
                            | W2Code::E
                            | W2Code::F
                            | W2Code::G
                            | W2Code::S
                            | W2Code::AA
                            | W2Code::BB
                            | W2Code::EE
                    )
                })
                .map(|c| c.amount)
                .sum(),
            _ => 0,
        })
        .sum()
}
pub(super) fn saver_from_w2(d: &FederalDraft) -> Vec<SaverContribution> {
    [Owner::Taxpayer, Owner::Spouse]
        .into_iter()
        .map(|owner| SaverContribution { owner, amount: saver_box12(d, owner) })
        .filter(|c| c.amount != 0)
        .collect()
}
// The saver's credit is offered whenever the return's facts let it apply:
// the AGI inside Form 8880's bands for the filing status (a recorded credit
// stays offered so it can be changed). A contribution is not a gate: the
// page itself asks for contributions no other page records (ABLE, plans
// outside W-2 box 12). It is suggested (derived) only when a contribution
// is already on the return (box 12 deferrals or IRA contributions).
pub(super) fn saver_offered(d: &FederalDraft) -> bool {
    d.credits.as_ref().is_some_and(|c| c.saver.is_some()) || saver_band(d)
}
pub(super) fn saver_hint(d: &FederalDraft) -> bool {
    let contribution = saver_box12(d, Owner::Taxpayer) != 0
        || saver_box12(d, Owner::Spouse) != 0
        || d.adjustments.as_ref().is_some_and(|a| a.entered_iras().is_some());
    contribution && saver_band(d)
}
pub(super) fn saver_band(d: &FederalDraft) -> bool {
    let Some(status) = d.setup.as_ref().map(|s| s.filing_status()) else {
        return false;
    };
    // An AGI the estimate cannot state yet does not hide the topic.
    estimate(d).lines.iter().find(|l| l.id == "agi").is_none_or(|l| {
        crate::federal_rules::saver_rate(crate::kernel::ratio::trunc_i64(l.amount, 100), status) > 0
    })
}
// The saver page's value for the stored credit: the workplace rows derived
// from box 12, the other contributions as what the stored total adds to them.
// None for a record this page does not write (unconfirmed answers, spouse
// contributions not from a W-2).
pub(super) fn saver_item(d: &FederalDraft) -> Option<SaverCredit> {
    let from_w2 = saver_from_w2(d);
    let Some(facts) = d.credits.as_ref().and_then(|c| c.saver.as_ref()) else {
        return Some(SaverCredit {
            from_w2,
            distributions: Vec::new(),
            more: SaverMore::default(),
        });
    };
    let other_contributions = facts.taxpayer_contributions - saver_box12(d, Owner::Taxpayer);
    if other_contributions < 0 || facts.spouse_contributions != saver_box12(d, Owner::Spouse) {
        return None;
    }
    let spouse_able = facts.spouse_able;
    Some(SaverCredit {
        from_w2,
        distributions: facts.distributions.clone(),
        more: SaverMore { your_able: facts.taxpayer_able, spouse_able, other_contributions },
    })
}
// The page onto FederalSaver: contributions are box 12 deferrals plus the
// other contributions; choosing the page is the eligibility, ABLE and
// distribution-lookback confirmation.
pub(super) fn saver_record(d: &FederalDraft, value: &SaverCredit) -> FederalSaver {
    let SaverMore { your_able, spouse_able, other_contributions } = value.more;
    FederalSaver {
        taxpayer_contributions: saver_box12(d, Owner::Taxpayer) + other_contributions,
        spouse_contributions: saver_box12(d, Owner::Spouse),
        taxpayer_able: your_able,
        spouse_able,
        distributions: value.distributions.clone(),
    }
}
// A changed W-2 re-derives the saver's workplace contributions: the part of
// each stored total not from box 12 is kept, the box 12 part follows the
// W-2s (the wages page erases nothing).
pub(super) fn follow_saver(before: &FederalDraft, d: &mut FederalDraft) {
    let (taxpayer, spouse) = (saver_box12(d, Owner::Taxpayer), saver_box12(d, Owner::Spouse));
    let (was_taxpayer, was_spouse) =
        (saver_box12(before, Owner::Taxpayer), saver_box12(before, Owner::Spouse));
    if let Some(facts) = d.credits.as_mut().and_then(|c| c.saver.as_mut()) {
        facts.taxpayer_contributions =
            taxpayer + phase::floor(facts.taxpayer_contributions - was_taxpayer, 0);
        facts.spouse_contributions =
            spouse + phase::floor(facts.spouse_contributions - was_spouse, 0);
    }
}

impl Interview {
    // The retirement savings credit page, opened with its complete value:
    // the workplace contributions derived from box 12, the stored
    // distributions and other amounts, or the defaults (M3-P11).
    pub(super) fn saver_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let Some(value) = saver_item(d) else {
            return problem_key("PROBLEM_UNDISPLAYABLE_SAVER_S_CREDIT_RECORD");
        };
        Reply::Edit(P::SaveSaver {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: CREDITS_PAGE,
            previous: CREDITS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::Saver),
            warnings: Vec::new(),
            progress: self.topic_progress(CREDITS_PAGE, Some(TaxSection::Saver)),
            value,
        })
    }
    // The retirement savings credit (Form 8880) onto FederalSaver.
    pub(super) fn saver_candidate(&self, value: &SaverCredit) -> Result<FederalDraft, Vec<u8>> {
        if !saver_offered(&self.draft) {
            return Err(crate::interview::status("PROBLEM_SECTION_UNAVAILABLE_YET"));
        }
        let mut draft = self.draft.clone();
        let facts = saver_record(&draft, value);
        let Some(credits) = draft.credits.as_mut() else {
            return Err(crate::interview::status("PROBLEM_SECTION_UNAVAILABLE_YET"));
        };
        credits.saver = Some(facts).into();
        finish_section(&mut draft, TaxSection::Saver);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_saver(
        &self,
        value: &SaverCredit,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != CREDITS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.saver_candidate(value).map_err(Into::into)
    }
}
