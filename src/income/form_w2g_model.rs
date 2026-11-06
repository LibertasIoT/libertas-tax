#[allow(unused_imports)]
use super::*;

/// Casual gambling records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalGambling {
    /// Winning amounts from reconciled gambling records
    /// Include reported and unreported cash winnings and the fair value of noncash prizes. Reconcile Forms W-2G with the wager/session records; do not count a form and its underlying winnings twice or net losing sessions against winning sessions here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub winnings: i64,
    /// Documented losing wagers or losing sessions, before the deduction limit
    /// Do not include a wager cost already netted in the winning amount, or travel, meals and other personal expenses. The App applies the 2026 loss limit and deduction choice.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub losses: i64,
    /// Federal tax withheld on Form W-2G
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// These are your own records, not a Form W-2G
    /// Losses and other winnings kept without a W-2G; false is a Form W-2G.
    pub own_records: bool,
    /// Ordinary casual gambling; complete records and no special treatment
    /// No professional gambling business, treaty/nonresident treatment, installment prize sale or unresolved session/W-2G discrepancy. Keep the gambling diary and supporting records, and reconcile your Forms W-2G with them. No leaves these records out of the estimate with a notice; that treatment is not supported.
    pub ordinary: bool,
}

/// Form W-2G Certain Gambling Winnings
/// The printed boxes of Form W-2G in the form's order, exactly as the payer printed them. Winnings, losses and treatment answers that are not printed are asked on the gambling page.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into payer, winner: the payer name, address and PAYER'S TIN into payer, the winner name, address and box 9 WINNER'S TIN into winner. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the payer telephone number exactly. Read the CORRECTED checkbox into corrected. Read the W-2G exactly. Check its tax year against the supplied year; report a mismatch. Box 1 must show an amount; report a blank box 1 as an error. Return box 1 reportable winnings, including the value of a noncash prize as shown. Return null for a clearly blank box 4 or box 7, preserving a printed zero as zero. Read box 2 date won as a date, and null when blank. Read boxes 3 type of wager, 5 transaction, 6 race, 8 cashier, 10 window, 11 first identification no. and 12 second identification no. as printed text, with empty text when blank. Report illegible or conflicting values as errors. Do not infer whether the winner gambles as a business, is a nonresident, claims a treaty or sold future prize payments, or other facts absent from the W-2G. The file may hold several copies of this form, such as Copy B, Copy C and Copy 2; copies of the same form are one form. Read state boxes 13 through 15 and local boxes 16 through 18: copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return null for a blank state or local income or withholding box; preserve a printed zero as zero. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows. Read the printed calendar or tax year at the top of the form as an integer number such as 2026, or zero when it is clearly blank; never substitute the form revision year.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.winner.federal_id,@.date_won,@.transaction,@.winnings,@.withholding,@.identical_wagers",
    form_date_order = "MonthDayYear"
)]
pub struct FormW2GPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// PAYER'S name, address and TIN
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// PAYER'S telephone no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_telephone: String,
    /// Box 1 Reportable winnings
    /// Include the value of a noncash prize as shown.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub winnings: i64,
    /// Box 2 Date won
    /// The date of the winning event, which is not the date the money was paid if that was later.
    /// [NullText]
    /// Box 2 is blank
    /// [SetText]
    /// Date won
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub date_won: Option<u32>,
    /// Box 3 Type of wager
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub type_of_wager: String,
    /// Box 4 Federal income tax withheld
    /// [NullText]
    /// Box 4 Federal income tax withheld is blank
    /// [SetText]
    /// Box 4 Federal income tax withheld
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
    /// Box 5 Transaction
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub transaction: String,
    /// Box 6 Race
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub race: String,
    /// Box 7 Winnings from identical wagers
    /// The payer's additional winnings from identical wagers. The App does not add them to box 1.
    /// [NullText]
    /// Box 7 Winnings from identical wagers is blank
    /// [SetText]
    /// Box 7 Winnings from identical wagers
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub identical_wagers: Option<i64>,
    /// Box 8 Cashier
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub cashier: String,
    /// WINNER'S name, address and box 9 WINNER'S TIN
    #[libertas_llm_doc_field]
    pub winner: crate::PrintedParty,
    /// Box 10 Window
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub window: String,
    /// Box 11 First identification no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub first_identification: String,
    /// Box 12 Second identification no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub second_identification: String,
    /// Boxes 13 to 15 State, payer's state identification no., state winnings and state income tax withheld
    #[libertas_llm_doc_field]
    /// ----
    /// State income row
    pub states: Vec<crate::StateIncomeTax>,
    /// Boxes 16 to 18 Local winnings, local income tax withheld and name of locality
    #[libertas_llm_doc_field]
    /// ----
    /// Local income row
    pub localities: Vec<crate::LocalIncomeTax>,
}

impl FormW2GPaper {
    /// The internal W-2G model, with the person's special-treatment answer.
    pub(crate) fn native(&self, special: bool) -> W2g {
        let Self {
            calendar_year: _,
            corrected: _,
            payer,
            // No figure reads these boxes; the parties and state boxes reach the
            // accepted entry's metadata.
            payer_telephone: _,
            winnings,
            date_won: _,
            type_of_wager: _,
            withholding,
            transaction: _,
            race: _,
            identical_wagers: _,
            cashier: _,
            winner,
            window: _,
            first_identification: _,
            second_identification: _,
            states,
            localities,
        } = self;
        W2g {
            payer: payer.clone(),
            winner: winner.clone(),
            winnings: *winnings,
            withholding: withholding.unwrap_or(0),
            special,
            state_details: crate::StateLocalFields {
                states: states.clone(),
                localities: localities.clone(),
            },
        }
    }
    /// The state and local boxes the state-return owner reads.
    pub(crate) fn state_details(&self) -> crate::StateLocalFields {
        self.native(false).state_details
    }
    /// Replace the state and local boxes from the state-return owner.
    pub(crate) fn set_state_details(&mut self, details: crate::StateLocalFields) {
        let crate::StateLocalFields { states, localities } = details;
        self.states = states;
        self.localities = localities;
    }
}
impl From<&FormW2GPaper> for W2g {
    fn from(paper: &FormW2GPaper) -> Self {
        paper.native(false)
    }
}

/// Accepted Form W-2G
/// The printed form and the topic answer it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2gSource {
    /// Form W-2G as printed
    pub form: FormW2GPaper,
    /// Something else applies
    /// You gamble as a business, you are a nonresident or a treaty applies, or you sold the right to future prize payments.
    #[libertas_default(false)]
    pub special: bool,
}
impl W2gSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: FormW2GPaper) -> Self {
        Self { form, special: false }
    }
    /// The topic answer of an accepted source survives a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.special = accepted.special;
    }
}

/// W-2G
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2g {
    /// PAYER'S name, identification number and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// WINNER'S name, identification number and address
    #[libertas_llm_doc_field]
    pub winner: crate::PrintedParty,
    /// Box 1 reportable winnings
    /// Include the value of a noncash prize as shown.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub winnings: i64,
    /// Box 4 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Something else applies
    /// You gamble as a business, you are a nonresident or a treaty applies, or you sold the right to future prize payments.
    #[libertas_default(false)]
    pub special: bool,
    /// Payer and state/local boxes 13–18
    #[libertas_llm_doc_field]
    pub state_details: crate::StateLocalFields,
}

/// Gambling losses and winnings without a W-2G
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct GamblingLosses {
    /// Winnings not on a Form W-2G
    /// Cash and the value of prizes from every wager, session or drawing, even small ones. Do not repeat a W-2G's box 1.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_winnings: i64,
    /// Gambling losses
    /// Losing wagers and sessions from your diary, tickets and statements, before any limit. Not travel or meals. They count only if you itemize, at 90% and no more than your winnings.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub losses: i64,
    /// Something else applies
    /// You gamble as a business, you are a nonresident or a treaty applies, or you sold the right to future prize payments.
    #[libertas_default(false)]
    pub special: bool,
}

/// Accepted casual gambling source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum GamblingSource {
    /// Form W-2G and its treatment answer
    W2g(alloc::boxed::Box<W2gSource>),
    /// Winnings and losses from personal records
    OwnRecords(GamblingLosses),
}

impl From<&GamblingSource> for FederalGambling {
    fn from(source: &GamblingSource) -> Self {
        match source {
            GamblingSource::W2g(source) => {
                let W2gSource { form, special } = source.as_ref();
                let W2g { payer: _, winner: _, winnings, withholding, special, state_details: _ } =
                    form.native(*special);
                // Printed parties and state boxes also reach their accepted-entry metadata.
                Self { winnings, losses: 0, withholding, own_records: false, ordinary: !special }
            }
            GamblingSource::OwnRecords(records) => {
                let GamblingLosses { other_winnings, losses, special } = records;
                Self {
                    winnings: *other_winnings,
                    losses: *losses,
                    withholding: 0,
                    own_records: true,
                    ordinary: !*special,
                }
            }
        }
    }
}

/// Gambling as a business
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ProfessionalGambling {
    /// Losing wagers in 2026
    /// From your wagering records, before the 90% limit. Do not also enter them as Part II expenses or subtract them from line 1.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub losses: i64,
    /// Federal income tax withheld on your W-2Gs for this activity
    /// W-2G box 4, added across forms.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Gambling is your trade or business, and your records are complete
    /// Yes when you gamble regularly and continuously to make a profit and line 1 holds only this activity's winnings. No leaves this business out of the estimate with a notice; enter casual gambling on the gambling page instead.
    pub eligible: bool,
}
