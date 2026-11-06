use crate::tests::*;

// A stored situation refuses the return by name (every needle appears in one
// issue, under the page it is chosen on) with its milestone, and the draft
// without it keeps its own result.
// One chosen item the estimate refuses.
pub(crate) type Chosen = Unprepared;
impl From<RareSituation> for Unprepared {
    fn from(situation: RareSituation) -> Self {
        Self::Situation { situation }
    }
}
impl Unprepared {
    // Adds the item to the stored list.
    pub(crate) fn choose(&self, d: &mut FederalDraft) {
        d.often_missed.unprepared.push(self.clone());
    }
}

// Every refused item.
pub(crate) fn every_refusal_item() -> Vec<Chosen> {
    Unprepared::all()
}

pub(in crate::tests) fn survivor_with_adult_son(gross_income: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Surviving);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::WidowedPriorYear { could_file_joint: true };
    add_child(&mut d);
    let son = &mut d.dependents[0];
    son.birth_date = 20020301;
    son.situation = DependentSituation::Supported {
        your_support_over_half: son.your_support_over_half(),
        gross_income,
    };
    d
}
