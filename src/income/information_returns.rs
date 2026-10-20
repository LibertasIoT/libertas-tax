use crate::validation::ValidationIssue;
// Coverage: Forms 1099-NEC, 1099-MISC and 1099-K, each entered once as its own document (unit b9-2,
//   2026-10-10; Forms 1099-NEC, 1099-MISC and 1099-K Rev. December 2026 and their Instructions
//   for Recipient/Payee, i1099mec, i1099k, i1040sc (2026 draft) line 1, Schedule 1 (2026 draft)
//   top-of-form 1099-K entry, i1040gi (2025) "Form(s) 1099-K", Pub. 525).
//   Lines: every nonzero box is sent by the person's answer to the page that owns its tax
//   treatment through RecordChoice catalogs (the Form 4835 `$.farms` pattern): a business
//   (Schedule C line 1: 1099-NEC box 1a; 1099-MISC boxes 1, 2 and 3 on a business choice, and
//   boxes 5, 6, 10 less the part paid on to clients, and 11; a 1099-K part less its refunds), a
//   rental property (Schedule E line 3: 1099-MISC box 1, a 1099-K part), a royalty entry
//   (Schedule E line 4, box 2), equipment rented out (line 8l, box 1), a hobby (line 8j: a
//   1099-K part, and a 1099-NEC/MISC amount marked not for profit) or an Other income kind
//   (1099-NEC box 1a, 1099-MISC box 3). 1099-MISC box 8 substitute payments are line 8z and net
//   investment income. Box 4 of each form is Form 1040 line 25b. A 1099-K part of money that
//   was not income (gifts, shared costs, personal items sold at a loss) is the top-of-Schedule 1
//   entry; a personal item sold at a gain is a capital gain entered on Investment sales (a
//   review note when no sale is entered). The 1099-K box 1a parts must add to box 1a.
//   `route` runs on the supported working copy and adds the routed amounts to the destination
//   records; a form whose destination is left out follows it with
//   WARN_FORM_1099_DESTINATION_LEFT_OUT. NEC box 3 is included in box 1a and adds only
//   section 4999 tax (20%) to 2026 Schedule 2 line 13k. W-2 code K supplies tax itself;
//   see forms/w2.rs. Refused by name: MISC box 15 section 409A tax and premium interest. Not read: the cash tips, tips-occupation and overtime boxes
//   (Schedule 1-A, with RareSituation::TipsNotOnW2), boxes 7 and 12 and the FATCA checkbox;
//   crop insurance (box 9) and farm 1099 amounts stay on the Farming lines.
//   Evidence: this module, other_income.rs, federal_rules.rs, unknowns.rs, state_return.rs;
//   tests/information_returns.rs (fixtures A, B, C, H and the box routing).
//   Unit18 source validation: NEC box3 is included in box1a (i1099mec,
//   Rev.December2026, p.11); both the excess subset and box4 withholding
//   are bounded by total compensation, including on restored records.
use crate::{
    BusinessActivity, FederalDraft, FederalIncome, FederalIncomeEntry, OtherIncomeItem,
    OtherIncomeKind, Owner,
};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// 1099-NEC
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-NEC exactly. Check the form's tax year against the supplied year and report a mismatch as an error. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Read box 1a nonemployee compensation; report an error when box 1a is blank. Read box 3 excess golden parachute payments and box 4 federal income tax withheld, returning zero for each when it is clearly blank. Box 3 is already included in box 1a; do not add it to compensation or compute its tax. Do not read boxes 1b, 1c or 1d or the box 2 checkbox. Do not say where the income belongs or which business earned it; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the payer name, federal ID and address and state boxes 5 through 7. For the state_details fields, copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return an empty string for absent payer identity text fields and null for a blank state or local income box; return zero for a clearly blank withholding box. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct Form1099Nec {
    /// Box 1a nonemployee compensation
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub compensation: i64,
    /// Box 4 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 3 excess golden parachute payments
    /// Already included in box 1a. The App calculates the 20% excise tax on this excess amount.
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub golden_parachute: i64,
    /// Where box 1a belongs
    /// Self-employment income is taxed with self-employment tax and can be reduced by the costs of the business. Income from a hobby or a one-time job has no self-employment tax and no deduction of costs.
    pub belongs: IncomeUse,
    /// Business that earned it
    /// When box 1a belongs to a business, choose it. Empty uses your only business; one entered on this same page is chosen after you save it.
    /// [NullText]
    /// Use my only applicable business
    /// [SetText]
    /// The business is
    #[libertas_enum_source("$.businesses")]
    #[libertas_default($null)]
    pub business: Option<i64>,
    /// Payer and state boxes 5–7
    #[libertas_llm_doc_field]
    pub state_details: crate::StateIncomeFields,
}

/// Where this income belongs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IncomeUse {
    /// A business you run
    /// Self-employment income on that business's Schedule C, with self-employment tax. Choose the business below.
    Business,
    /// An activity you don't run to make a profit
    /// A hobby: the income goes on Schedule 1 line 8j, with no self-employment tax and no deduction for its costs.
    NotForProfit,
    /// Income that is not from a business or a hobby
    /// It goes on its own Schedule 1 line, with no self-employment tax.
    Income {
        /// What it is
        kind: OtherIncomeKind,
    },
}

/// 1099-MISC
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-MISC exactly. Check the form's tax year against the supplied year and report a mismatch as an error. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Read box 1 rents, box 2 royalties, box 3 other income, box 4 federal income tax withheld, box 5 fishing boat proceeds, box 6 medical and health care payments, box 8 substitute payments in lieu of dividends or interest, box 10 gross proceeds paid to an attorney, box 11 fish purchased for resale and box 15 nonqualified deferred compensation, returning an empty string for each box that is clearly blank; report an error when boxes 1, 2, 3, 5, 6, 8, 10, 11 and 15 are all blank. Do not read box 9 crop insurance proceeds, box 12, boxes 13a, 13b or 14, or the box 7 and FATCA checkboxes. Do not say where an amount belongs, which business, rental or property received it, or what part of box 10 was paid on to clients; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the payer name, federal ID and address and state boxes 16 through 18. For the state_details fields, copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return an empty string for absent payer identity text fields and null for a blank state or local income box; return zero for a clearly blank withholding box. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct Form1099Misc {
    /// Box 1 rents
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub rents: i64,
    /// Where box 1 belongs
    /// Rent for real estate goes on that rental property; rent for equipment or other movable property you rent out for profit, but not as a business, goes on that entry under Other income; rent with substantial services or from a rental business is business income.
    /// [NullText]
    /// Box 1 has no income to route
    /// [SetText]
    /// The box 1 rent income belongs to
    pub rents_use: Option<RentUse>,
    /// Box 2 royalties
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub royalties: i64,
    /// Where box 2 belongs
    /// Royalties from property you own go on that royalty entry; royalties for your own current work as a writer, inventor or artist are business income.
    /// [NullText]
    /// Box 2 has no income to route
    /// [SetText]
    /// The box 2 royalty income belongs to
    pub royalties_use: Option<RoyaltyUse>,
    /// Box 3 other income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other: i64,
    /// Where box 3 belongs
    /// Prizes, damages, research-study pay and similar amounts are Other income; payments for your services or goods in a business are business income; a hobby's are not-for-profit income.
    /// [NullText]
    /// Box 3 has no income to route
    /// [SetText]
    /// The box 3 other income belongs to
    pub other_use: Option<IncomeUse>,
    /// Box 4 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 5 fishing boat proceeds
    /// Your share as a crew member; income of your fishing business.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub fishing: i64,
    /// Box 6 medical and health care payments
    /// Paid to you as a provider of medical or health care services; income of your practice.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub medical: i64,
    /// Box 8 substitute payments in lieu of dividends or interest
    /// Paid by your broker while your shares were on loan. Other income on Schedule 1 line 8z, not qualified dividends or tax-exempt interest.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub substitute: i64,
    /// Box 10 gross proceeds paid to an attorney
    /// Paid to you as an attorney; only your fee is your practice's income.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub attorney: i64,
    /// Part of box 10 you paid on to clients or others
    /// Settlement money that was not your fee. The rest of box 10 is added to your practice's receipts.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub client_funds: i64,
    /// Box 11 fish purchased for resale
    /// Cash a buyer paid you for fish you caught in your fishing business.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub fish_sales: i64,
    /// Box 15 nonqualified deferred compensation
    /// The App does not figure the section 409A additional tax and interest on this income; a form with box 15 is named in the estimate as not calculated.
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub deferred: i64,
    /// Business for the amounts that belong to a business
    /// Boxes 5, 6, 10 and 11 always do, and boxes 1–3 when you chose a business above. Empty uses your only business on Self-employment.
    /// [NullText]
    /// Use my only applicable business
    /// [SetText]
    /// The business is
    #[libertas_enum_source("$.businesses")]
    #[libertas_default($null)]
    pub business: Option<i64>,
    /// Payer and state boxes 16–18
    #[libertas_llm_doc_field]
    pub state_details: crate::StateIncomeFields,
}

/// Where box 1 belongs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentUse {
    /// A rental property
    /// Rent for real estate: Schedule E line 3 of the property you choose on Rental property.
    RealEstate {
        /// Rental property
        /// Empty uses your only rental property.
        /// [NullText]
        /// Use my only applicable rental property
        /// [SetText]
        /// The rental property is
        #[libertas_enum_source("$.rentals")]
        #[libertas_default($null)]
        rental: Option<i64>,
    },
    /// Equipment or other property you rent out, not as a business
    /// Schedule 1 line 8l, on the equipment entry you choose under Other income.
    Equipment {
        /// Equipment you rent out
        /// Empty uses your only such entry.
        /// [NullText]
        /// Use my only applicable equipment rental entry
        /// [SetText]
        /// The equipment rental entry is
        #[libertas_enum_source("$.equipment_rentals")]
        #[libertas_default($null)]
        rental: Option<i64>,
    },
    /// A business you run
    /// A business of renting property, or real estate rented with substantial services: Schedule C of the business chosen below.
    Business,
}

/// Where box 2 belongs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RoyaltyUse {
    /// Royalties from property you own
    /// Oil, gas or mineral royalties, or royalties from a copyright or patent that is not your own current work: Schedule E, on the royalty entry you choose under Other income.
    Property {
        /// Royalty property
        /// Empty uses your only royalty entry.
        /// [NullText]
        /// Use my only applicable royalty entry
        /// [SetText]
        /// The royalty entry is
        #[libertas_enum_source("$.royalty_properties")]
        #[libertas_default($null)]
        property: Option<i64>,
    },
    /// A business you run
    /// Royalties from your work as a self-employed writer, inventor or artist: Schedule C of the business chosen below.
    Business,
}

/// 1099-K
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-K exactly. Check the form's tax year against the supplied year and report a mismatch as an error. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Read box 1a gross amount of payment card and third party network transactions; report an error when box 1a is blank. Read box 4 federal income tax withheld, returning an empty string when it is clearly blank. Do not read boxes 1b, 1c, 1d, 2, 3 or 5a through 5l. Do not split box 1a or say where its payments belong; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the filer name, federal ID and address and state boxes 6 through 8. For the state_details fields, copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return an empty string for absent payer identity text fields and null for a blank state or local income box; return zero for a clearly blank withholding box. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct Form1099K {
    /// Box 1a gross amount
    /// Before refunds, fees and shipping.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub gross: i64,
    /// Box 4 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Where the payments in box 1a belong
    /// The parts add up to box 1a: one for each business, rental or hobby, and one for payments that were not income to you.
    /// ----
    /// Part
    #[libertas_size(min = 1)]
    pub parts: alloc::vec::Vec<KPart>,
    /// Filer and state boxes 6–8
    #[libertas_llm_doc_field]
    pub state_details: crate::StateWithholdingFields,
}

/// Part
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct KPart {
    /// Amount of box 1a
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
    /// Where it belongs
    pub belongs: KUse,
    /// Refunds you gave back out of this part
    /// Money you returned to customers or guests for these payments. Platform fees are not refunds; enter them as that business's or rental's expenses.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub refunded: i64,
}

/// Where it belongs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum KUse {
    /// A business you run
    /// Schedule C line 1 of the business you choose, less this part's refunds.
    Business {
        /// Business
        /// Empty uses your only business; one entered on this same page is chosen after you save it.
        /// [NullText]
        /// Use my only applicable business
        /// [SetText]
        /// The business is
        #[libertas_enum_source("$.businesses")]
        #[libertas_default($null)]
        business: Option<i64>,
    },
    /// A rental property
    /// Schedule E line 3 rents of the property you choose, such as short-term rental bookings, less this part's refunds.
    Rental {
        /// Rental property
        /// Empty uses your only rental property.
        /// [NullText]
        /// Use my only applicable rental property
        /// [SetText]
        /// The rental property is
        #[libertas_enum_source("$.rentals")]
        #[libertas_default($null)]
        rental: Option<i64>,
    },
    /// Sales from a hobby
    /// Added to the hobby entry you choose under Other income (Schedule 1 line 8j), less this part's refunds.
    Hobby {
        /// Hobby
        /// Empty uses your only hobby entry.
        /// [NullText]
        /// Use my only applicable hobby entry
        /// [SetText]
        /// The hobby entry is
        #[libertas_enum_source("$.hobbies")]
        #[libertas_default($null)]
        hobby: Option<i64>,
    },
    /// Not income to you
    /// Payments included in error, such as money from friends or family for shared costs, gifts or reimbursements, and personal items you sold for less than you paid. Shown at the top of Schedule 1; not taxed.
    NotIncome,
    /// Personal items you sold for more than you paid
    /// Each such item is a capital gain: enter it as a sale on Investment sales (no 1099-B). This part adds nothing by itself.
    PersonalGain,
}

// ---- The stored forms ----
//
// The page types above choose their records through `$.` catalogs of the page's
// request, so they are not stored: the stored forms name the same records by
// income identity.

/// Form 1099-NEC
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalForm1099Nec {
    /// Box 1a nonemployee compensation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub compensation: i64,
    /// Box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub withholding: i64,
    /// Box 3 excess golden parachute payments
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub golden_parachute: i64,
    /// Where box 1a belongs
    pub belongs: IncomeUse,
    /// Business that earned it; empty is the owner's only business
    #[libertas_number(min = 1)]
    #[libertas_default($null)]
    pub business: Option<i64>,
}

/// Form 1099-MISC
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalForm1099Misc {
    /// Box 1 rents
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub rents: i64,
    /// Where box 1 belongs
    #[libertas_default($null)]
    pub rents_use: Option<FederalRentUse>,
    /// Box 2 royalties
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub royalties: i64,
    /// Where box 2 belongs
    #[libertas_default($null)]
    pub royalties_use: Option<FederalRoyaltyUse>,
    /// Box 3 other income
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub other: i64,
    /// Where box 3 belongs
    #[libertas_default($null)]
    pub other_use: Option<IncomeUse>,
    /// Box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub withholding: i64,
    /// Box 5 fishing boat proceeds
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub fishing: i64,
    /// Box 6 medical and health care payments
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub medical: i64,
    /// Box 8 substitute payments in lieu of dividends or interest
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub substitute: i64,
    /// Box 10 gross proceeds paid to an attorney
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub attorney: i64,
    /// Part of box 10 paid on to clients
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub client_funds: i64,
    /// Box 11 fish purchased for resale
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub fish_sales: i64,
    /// Box 15 nonqualified deferred compensation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub deferred: i64,
    /// Business for the amounts that belong to a business
    #[libertas_number(min = 1)]
    #[libertas_default($null)]
    pub business: Option<i64>,
}

/// Where box 1 belongs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalRentUse {
    /// A rental property
    RealEstate {
        /// The rental's income identity; empty is the only one
        #[libertas_number(min = 1)]
        #[libertas_default($null)]
        rental: Option<i64>,
    },
    /// Equipment rented out
    Equipment {
        /// The equipment entry's income identity; empty is the only one
        #[libertas_number(min = 1)]
        #[libertas_default($null)]
        rental: Option<i64>,
    },
    /// A business
    Business,
}

/// Where box 2 belongs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalRoyaltyUse {
    /// A royalty entry
    Property {
        /// The royalty entry's income identity; empty is the only one
        #[libertas_number(min = 1)]
        #[libertas_default($null)]
        property: Option<i64>,
    },
    /// A business
    Business,
}

/// Form 1099-K
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalForm1099K {
    /// Box 1a gross amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub gross: i64,
    /// Box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub withholding: i64,
    /// Where the payments in box 1a belong
    #[libertas_size(min = 1)]
    pub parts: alloc::vec::Vec<FederalKPart>,
}

/// Part of box 1a
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalKPart {
    /// Amount of box 1a
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
    /// Where it belongs
    pub belongs: FederalKUse,
    /// Refunds given back out of this part
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub refunded: i64,
}

/// Where a part of box 1a belongs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalKUse {
    /// A business
    Business {
        /// The business's income identity; empty is the owner's only one
        #[libertas_number(min = 1)]
        #[libertas_default($null)]
        business: Option<i64>,
    },
    /// A rental property
    Rental {
        /// The rental's income identity; empty is the only one
        #[libertas_number(min = 1)]
        #[libertas_default($null)]
        rental: Option<i64>,
    },
    /// A hobby
    Hobby {
        /// The hobby's income identity; empty is the only one
        #[libertas_number(min = 1)]
        #[libertas_default($null)]
        hobby: Option<i64>,
    },
    /// Not income to the person
    NotIncome,
    /// Personal items sold for more than they cost
    PersonalGain,
}

/// Who received a Form 1099-G box 6 grant
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalGrantUse {
    /// The person, not for a business
    You,
    /// A business
    Business {
        /// The business's income identity; empty is the owner's only one
        #[libertas_number(min = 1)]
        #[libertas_default($null)]
        business: Option<i64>,
    },
}

impl Form1099Nec {
    pub(crate) fn record(&self) -> FederalForm1099Nec {
        FederalForm1099Nec {
            compensation: self.compensation,
            withholding: self.withholding,
            golden_parachute: self.golden_parachute,
            belongs: self.belongs.clone(),
            business: self.business,
        }
    }
}
impl FederalForm1099Nec {
    // The page of a stored form; the state boxes come back from the entry's state source.
    pub(crate) fn page(&self) -> Form1099Nec {
        Form1099Nec {
            compensation: self.compensation,
            withholding: self.withholding,
            golden_parachute: self.golden_parachute,
            belongs: self.belongs.clone(),
            business: self.business,
            state_details: Default::default(),
        }
    }
    pub(crate) fn valid(&self) -> bool {
        [self.compensation, self.withholding, self.golden_parachute].iter().all(|v| money(*v))
            && self.golden_parachute <= self.compensation
            && self.withholding <= self.compensation
            && self.business.is_none_or(|id| id >= 1)
            && use_valid(&self.belongs, self.compensation)
    }
}
impl Form1099Misc {
    pub(crate) fn record(&self) -> FederalForm1099Misc {
        FederalForm1099Misc {
            rents: self.rents,
            rents_use: self.rents_use.as_ref().map(|u| match u {
                RentUse::RealEstate { rental } => FederalRentUse::RealEstate { rental: *rental },
                RentUse::Equipment { rental } => FederalRentUse::Equipment { rental: *rental },
                RentUse::Business => FederalRentUse::Business,
            }),
            royalties: self.royalties,
            royalties_use: self.royalties_use.as_ref().map(|u| match u {
                RoyaltyUse::Property { property } => {
                    FederalRoyaltyUse::Property { property: *property }
                }
                RoyaltyUse::Business => FederalRoyaltyUse::Business,
            }),
            other: self.other,
            other_use: self.other_use.clone(),
            withholding: self.withholding,
            fishing: self.fishing,
            medical: self.medical,
            substitute: self.substitute,
            attorney: self.attorney,
            client_funds: self.client_funds,
            fish_sales: self.fish_sales,
            deferred: self.deferred,
            business: self.business,
        }
    }
}
impl FederalForm1099Misc {
    pub(crate) fn page(&self) -> Form1099Misc {
        Form1099Misc {
            rents: self.rents,
            rents_use: self.rents_use.as_ref().map(|u| match u {
                FederalRentUse::RealEstate { rental } => RentUse::RealEstate { rental: *rental },
                FederalRentUse::Equipment { rental } => RentUse::Equipment { rental: *rental },
                FederalRentUse::Business => RentUse::Business,
            }),
            royalties: self.royalties,
            royalties_use: self.royalties_use.as_ref().map(|u| match u {
                FederalRoyaltyUse::Property { property } => {
                    RoyaltyUse::Property { property: *property }
                }
                FederalRoyaltyUse::Business => RoyaltyUse::Business,
            }),
            other: self.other,
            other_use: self.other_use.clone(),
            withholding: self.withholding,
            fishing: self.fishing,
            medical: self.medical,
            substitute: self.substitute,
            attorney: self.attorney,
            client_funds: self.client_funds,
            fish_sales: self.fish_sales,
            deferred: self.deferred,
            business: self.business,
            state_details: Default::default(),
        }
    }
    // The income the form adds to the return: every routed or local box at its
    // taxable amount (a prize under its disposition, box 10 less the part paid
    // on to clients, box 8 as other income). Box 15 is refused, not income. Cents.
    pub(crate) fn income(&self) -> i64 {
        let other = match &self.other_use {
            Some(IncomeUse::Income { kind }) => {
                OtherIncomeItem { kind: kind.clone(), amount: self.other, withholding: 0 }.preview()
            }
            _ => self.other,
        };
        self.rents
            + self.royalties
            + other
            + self.fishing
            + self.medical
            + self.substitute
            + (self.attorney - self.client_funds)
            + self.fish_sales
    }
    pub(crate) fn valid(&self) -> bool {
        [
            self.rents,
            self.royalties,
            self.other,
            self.withholding,
            self.fishing,
            self.medical,
            self.substitute,
            self.attorney,
            self.client_funds,
            self.fish_sales,
            self.deferred,
        ]
        .iter()
        .all(|v| money(*v))
            && self.business.is_none_or(|id| id >= 1)
            && self.other_use.as_ref().is_none_or(|u| use_valid(u, self.other))
    }
}
impl Form1099K {
    pub(crate) fn record(&self) -> FederalForm1099K {
        FederalForm1099K {
            gross: self.gross,
            withholding: self.withholding,
            parts: self
                .parts
                .iter()
                .map(|part| FederalKPart {
                    amount: part.amount,
                    belongs: match &part.belongs {
                        KUse::Business { business } => {
                            FederalKUse::Business { business: *business }
                        }
                        KUse::Rental { rental } => FederalKUse::Rental { rental: *rental },
                        KUse::Hobby { hobby } => FederalKUse::Hobby { hobby: *hobby },
                        KUse::NotIncome => FederalKUse::NotIncome,
                        KUse::PersonalGain => FederalKUse::PersonalGain,
                    },
                    refunded: part.refunded,
                })
                .collect(),
        }
    }
}
impl FederalForm1099K {
    pub(crate) fn page(&self) -> Form1099K {
        Form1099K {
            gross: self.gross,
            withholding: self.withholding,
            parts: self
                .parts
                .iter()
                .map(|part| KPart {
                    amount: part.amount,
                    belongs: match &part.belongs {
                        FederalKUse::Business { business } => {
                            KUse::Business { business: *business }
                        }
                        FederalKUse::Rental { rental } => KUse::Rental { rental: *rental },
                        FederalKUse::Hobby { hobby } => KUse::Hobby { hobby: *hobby },
                        FederalKUse::NotIncome => KUse::NotIncome,
                        FederalKUse::PersonalGain => KUse::PersonalGain,
                    },
                    refunded: part.refunded,
                })
                .collect(),
            state_details: Default::default(),
        }
    }
    // The income parts of box 1a less their refunds. Cents.
    pub(crate) fn income(&self) -> i64 {
        self.parts
            .iter()
            .filter(|part| {
                !matches!(part.belongs, FederalKUse::NotIncome | FederalKUse::PersonalGain)
            })
            .map(|part| part.amount - part.refunded)
            .sum()
    }
    pub(crate) fn valid(&self) -> bool {
        money(self.gross)
            && money(self.withholding)
            && !self.parts.is_empty()
            && self.parts.iter().all(|part| {
                money(part.amount)
                    && money(part.refunded)
                    && match &part.belongs {
                        FederalKUse::Business { business: Some(id) }
                        | FederalKUse::Rental { rental: Some(id) }
                        | FederalKUse::Hobby { hobby: Some(id) } => *id >= 1,
                        _ => true,
                    }
            })
            // Reuse Save's exact gross/refund owner for stored records too: every
            // nonnegative routed/net subset is bounded by this form's gross.
            && k_issue(&self.page()).is_none()
    }
}

impl crate::GrantUse {
    pub(crate) fn record(&self) -> FederalGrantUse {
        match self {
            Self::You => FederalGrantUse::You,
            Self::Business { business } => FederalGrantUse::Business { business: *business },
        }
    }
}
impl FederalGrantUse {
    pub(crate) fn page(&self) -> crate::GrantUse {
        match self {
            Self::You => crate::GrantUse::You,
            Self::Business { business } => crate::GrantUse::Business { business: *business },
        }
    }
}

fn money(value: i64) -> bool {
    (0..=100_000_000_000).contains(&value)
}

fn use_valid(belongs: &IncomeUse, amount: i64) -> bool {
    match belongs {
        IncomeUse::Income { kind } => {
            OtherIncomeItem { kind: kind.clone(), amount, withholding: 0 }.valid()
        }
        _ => true,
    }
}

pub(crate) fn nec_issue(v: &Form1099Nec) -> Option<ValidationIssue> {
    if v.golden_parachute > v.compensation {
        Some(ValidationIssue::new("NEC_EXCESS_ABOVE_TOTAL_PAY").at_field("golden_parachute"))
    } else if v.withholding > v.compensation {
        Some(ValidationIssue::new("NEC_WITHHOLDING_ABOVE_PAY").at_field("withholding"))
    } else {
        use_issue(&v.belongs).map(|issue| issue.in_field("belongs"))
    }
}
// A form never reports a recovery of an earlier year's deduction.
fn use_issue(belongs: &IncomeUse) -> Option<ValidationIssue> {
    matches!(belongs, IncomeUse::Income { kind: OtherIncomeKind::Recovery { .. } })
        .then(|| ValidationIssue::new("FORM_1099_USE_NOT_RECOVERY").at_field("kind"))
}
pub(crate) fn misc_issue(v: &Form1099Misc) -> Option<ValidationIssue> {
    if i128::from(v.withholding)
        > i128::from(v.rents)
            + i128::from(v.royalties)
            + i128::from(v.other)
            + i128::from(v.fishing)
            + i128::from(v.medical)
            + i128::from(v.substitute)
            + i128::from(v.attorney)
            + i128::from(v.fish_sales)
            + i128::from(v.deferred)
    {
        Some(ValidationIssue::new("MISC_WITHHOLDING_ABOVE_INCOME").at_field("withholding"))
    } else if v.rents != 0 && v.rents_use.is_none() {
        Some(ValidationIssue::new("MISC_RENTS_NEED_A_USE").at_field("rents_use"))
    } else if v.royalties != 0 && v.royalties_use.is_none() {
        Some(ValidationIssue::new("MISC_ROYALTIES_NEED_A_USE").at_field("royalties_use"))
    } else if v.other != 0 && v.other_use.is_none() {
        Some(ValidationIssue::new("MISC_OTHER_NEEDS_A_USE").at_field("other_use"))
    } else if v.client_funds > v.attorney {
        Some(ValidationIssue::new("MISC_CLIENT_FUNDS_ABOVE_PROCEEDS").at_field("client_funds"))
    } else {
        v.other_use.as_ref().and_then(use_issue).map(|issue| issue.in_field("other_use"))
    }
}
pub(crate) fn k_issue(v: &Form1099K) -> Option<ValidationIssue> {
    if v.parts.iter().map(|part| i128::from(part.amount)).sum::<i128>() != i128::from(v.gross) {
        return Some(ValidationIssue::new("K_PARTS_NOT_GROSS").at_field("parts"));
    }
    if v.withholding > v.gross {
        return Some(ValidationIssue::new("K_WITHHOLDING_ABOVE_GROSS").at_field("withholding"));
    }
    if let Some((i, _)) = v.parts.iter().enumerate().find(|(_, part)| part.refunded > part.amount) {
        return Some(
            ValidationIssue::new("K_REFUNDS_ABOVE_PART").at_field("refunded").in_list("parts", i),
        );
    }
    if let Some((i, _)) = v.parts.iter().enumerate().find(|(_, part)| {
        matches!(part.belongs, KUse::NotIncome | KUse::PersonalGain) && part.refunded != 0
    }) {
        return Some(
            ValidationIssue::new("K_REFUNDS_ONLY_ON_INCOME")
                .at_field("refunded")
                .in_list("parts", i),
        );
    }
    None
}

// ---- Routing ----

// The page of a record kind, for a correction link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Business,
    Rental,
    Royalty,
    Equipment,
    Hobby,
}
impl Kind {
    pub(crate) fn section(self) -> crate::TaxSection {
        match self {
            Self::Business => crate::TaxSection::SelfEmployment,
            Self::Rental => crate::TaxSection::Rentals,
            Self::Royalty => crate::TaxSection::Rentals,
            Self::Equipment | Self::Hobby => crate::TaxSection::OtherIncome,
        }
    }
}

// Where on a form one routed amount sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Slot {
    Nec,
    Rents,
    Royalties,
    Other,
    Fishing,
    Medical,
    Attorney,
    FishSales,
    Part(usize),
    Grant,
    Refund,
}

// One amount (cents) a form sends to a record, and the record it names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Flow {
    pub slot: Slot,
    pub kind: Kind,
    pub choice: Option<i64>,
    pub amount: i64,
}

// The amounts of one income record that belong to another record.
pub(crate) fn flows(income: &FederalIncome) -> alloc::vec::Vec<Flow> {
    let mut out = alloc::vec::Vec::new();
    let mut push = |slot, kind, choice: Option<i64>, amount: i64| {
        if amount != 0 {
            out.push(Flow { slot, kind, choice, amount });
        }
    };
    match income {
        FederalIncome::Nec1099 { data } if data.belongs == IncomeUse::Business => {
            push(Slot::Nec, Kind::Business, data.business, data.compensation);
        }
        FederalIncome::Misc1099 { data } => {
            match &data.rents_use {
                Some(FederalRentUse::RealEstate { rental }) => {
                    push(Slot::Rents, Kind::Rental, *rental, data.rents)
                }
                Some(FederalRentUse::Equipment { rental }) => {
                    push(Slot::Rents, Kind::Equipment, *rental, data.rents)
                }
                Some(FederalRentUse::Business) => {
                    push(Slot::Rents, Kind::Business, data.business, data.rents)
                }
                None => {}
            }
            match &data.royalties_use {
                Some(FederalRoyaltyUse::Property { property }) => {
                    push(Slot::Royalties, Kind::Royalty, *property, data.royalties)
                }
                Some(FederalRoyaltyUse::Business) => {
                    push(Slot::Royalties, Kind::Business, data.business, data.royalties)
                }
                None => {}
            }
            if matches!(data.other_use, Some(IncomeUse::Business)) {
                push(Slot::Other, Kind::Business, data.business, data.other);
            }
            push(Slot::Fishing, Kind::Business, data.business, data.fishing);
            push(Slot::Medical, Kind::Business, data.business, data.medical);
            push(Slot::Attorney, Kind::Business, data.business, data.attorney - data.client_funds);
            push(Slot::FishSales, Kind::Business, data.business, data.fish_sales);
        }
        FederalIncome::K1099 { data } => {
            for (index, part) in data.parts.iter().enumerate() {
                let amount = part.amount - part.refunded;
                match &part.belongs {
                    FederalKUse::Business { business } => {
                        push(Slot::Part(index), Kind::Business, *business, amount)
                    }
                    FederalKUse::Rental { rental } => {
                        push(Slot::Part(index), Kind::Rental, *rental, amount)
                    }
                    FederalKUse::Hobby { hobby } => {
                        push(Slot::Part(index), Kind::Hobby, *hobby, amount)
                    }
                    FederalKUse::NotIncome | FederalKUse::PersonalGain => {}
                }
            }
        }
        FederalIncome::Unemployment { data } => {
            if let Some(crate::FederalGrantUse::Business { business }) = &data.grant_use {
                push(Slot::Grant, Kind::Business, *business, data.taxable_grants);
            }
        }
        FederalIncome::TaxRefund {
            amount,
            history: crate::TaxRefundHistory::BusinessExpense { business },
            ..
        } => push(Slot::Refund, Kind::Business, *business, *amount),
        _ => {}
    }
    out
}

// The record choice a slot names, for rewriting.
fn choice_mut(income: &mut FederalIncome, slot: Slot) -> Option<&mut Option<i64>> {
    match (income, slot) {
        (FederalIncome::Nec1099 { data }, Slot::Nec) => Some(&mut data.business),
        (FederalIncome::Misc1099 { data }, Slot::Rents) => match &mut data.rents_use {
            Some(FederalRentUse::RealEstate { rental })
            | Some(FederalRentUse::Equipment { rental }) => Some(rental),
            _ => Some(&mut data.business),
        },
        (FederalIncome::Misc1099 { data }, Slot::Royalties) => match &mut data.royalties_use {
            Some(FederalRoyaltyUse::Property { property }) => Some(property),
            _ => Some(&mut data.business),
        },
        (
            FederalIncome::Misc1099 { data },
            Slot::Other | Slot::Fishing | Slot::Medical | Slot::Attorney | Slot::FishSales,
        ) => Some(&mut data.business),
        (FederalIncome::K1099 { data }, Slot::Part(index)) => {
            match data.parts.get_mut(index).map(|part| &mut part.belongs) {
                Some(FederalKUse::Business { business }) => Some(business),
                Some(FederalKUse::Rental { rental }) => Some(rental),
                Some(FederalKUse::Hobby { hobby }) => Some(hobby),
                _ => None,
            }
        }
        (FederalIncome::Unemployment { data }, Slot::Grant) => match &mut data.grant_use {
            Some(crate::FederalGrantUse::Business { business }) => Some(business),
            _ => None,
        },
        (
            FederalIncome::TaxRefund {
                history: crate::TaxRefundHistory::BusinessExpense { business },
                ..
            },
            Slot::Refund,
        ) => Some(business),
        _ => None,
    }
}

// Name the record a form sends an amount to, so a record removed later takes
// the amount with it.
pub(crate) fn name_record(income: &mut FederalIncome, slot: Slot, id: i64) {
    if let Some(choice) = choice_mut(income, slot) {
        *choice = Some(id);
    }
}

// Take the amount out of the form (the record left the estimate).
pub(crate) fn clear(income: &mut FederalIncome, slot: Slot) {
    match (income, slot) {
        (FederalIncome::Nec1099 { data }, Slot::Nec) => data.compensation = 0,
        (FederalIncome::Misc1099 { data }, Slot::Rents) => data.rents = 0,
        (FederalIncome::Misc1099 { data }, Slot::Royalties) => data.royalties = 0,
        (FederalIncome::Misc1099 { data }, Slot::Other) => data.other = 0,
        (FederalIncome::Misc1099 { data }, Slot::Fishing) => data.fishing = 0,
        (FederalIncome::Misc1099 { data }, Slot::Medical) => data.medical = 0,
        (FederalIncome::Misc1099 { data }, Slot::Attorney) => {
            data.attorney = 0;
            data.client_funds = 0;
        }
        (FederalIncome::Misc1099 { data }, Slot::FishSales) => data.fish_sales = 0,
        (FederalIncome::K1099 { data }, Slot::Part(index)) => {
            if let Some(part) = data.parts.get_mut(index) {
                part.amount = 0;
                part.refunded = 0;
            }
        }
        (FederalIncome::Unemployment { data }, Slot::Grant) => data.taxable_grants = 0,
        (
            FederalIncome::TaxRefund {
                amount,
                history: crate::TaxRefundHistory::BusinessExpense { .. },
                ..
            },
            Slot::Refund,
        ) => *amount = 0,
        _ => {}
    }
}

// Whether a form still has an amount that does not go to another record, or
// nothing at all to send: a form emptied of its routed amounts is removed.
pub(crate) fn nothing_left(income: &FederalIncome) -> bool {
    match income {
        FederalIncome::Nec1099 { data } => {
            data.golden_parachute == 0
                && (data.belongs == IncomeUse::Business && data.compensation == 0)
        }
        FederalIncome::Misc1099 { data } => {
            data.rents == 0
                && data.royalties == 0
                && data.fishing == 0
                && data.medical == 0
                && data.substitute == 0
                && data.attorney == 0
                && data.fish_sales == 0
                && data.deferred == 0
                && data.other == 0
        }
        FederalIncome::K1099 { data } => {
            data.parts.iter().all(|part| part.amount == 0 && part.refunded == 0)
        }
        FederalIncome::TaxRefund { amount, .. } => *amount == 0,
        _ => false,
    }
}

fn mine(d: &FederalDraft, owner: Owner, entry: &FederalIncomeEntry, business: bool) -> bool {
    if business {
        entry.owner == owner
    } else {
        entry.owner == owner
            || d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Joint)
    }
}

// The records of one kind a form of `owner` may name: the owner's own
// businesses (self-employment tax is per person), and any rental, royalty,
// equipment or hobby record of the owner, or either spouse's on a joint return.
pub(crate) fn candidates(d: &FederalDraft, owner: Owner, kind: Kind) -> alloc::vec::Vec<i64> {
    d.income
        .iter()
        .filter(|entry| match (&entry.income, kind) {
            (FederalIncome::Business { data }, Kind::Business) => {
                matches!(data.activity, BusinessActivity::Ordinary { .. })
                    && mine(d, owner, entry, true)
            }
            (FederalIncome::Rental { .. }, Kind::Rental) => mine(d, owner, entry, false),
            (FederalIncome::Royalty { .. }, Kind::Royalty) => mine(d, owner, entry, false),
            (
                FederalIncome::NonbusinessProperty {
                    kind: crate::NonbusinessPropertyKind::PersonalRental { .. },
                    ..
                },
                Kind::Equipment,
            ) => mine(d, owner, entry, false),
            (FederalIncome::HobbySales { .. }, Kind::Hobby) => mine(d, owner, entry, false),
            _ => false,
        })
        .map(|entry| entry.id)
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Lookup {
    Found(i64),
    // The owner has no record of the kind.
    Missing,
    // Several, and none chosen; or the one named is not one the form may name.
    NotChosen,
}

pub(crate) fn lookup(d: &FederalDraft, owner: Owner, kind: Kind, choice: Option<i64>) -> Lookup {
    let found = candidates(d, owner, kind);
    match (choice, found.as_slice()) {
        (Some(id), _) if found.contains(&id) => Lookup::Found(id),
        (Some(_), _) => Lookup::NotChosen,
        (None, []) => Lookup::Missing,
        (None, [only]) => Lookup::Found(*only),
        (None, _) => Lookup::NotChosen,
    }
}

// Collect a retained local NEC/MISC amount at its actual selected destination.
// A business amount has already been routed to its native business record.
pub(crate) fn collect_local(
    ctx: &mut crate::other_income::Ctx<'_>,
    belongs: &IncomeUse,
    amount: i64,
) {
    match belongs {
        IncomeUse::Business => {}
        IncomeUse::NotForProfit => {
            ctx.totals.not_for_profit += amount;
            ctx.totals.injured_other_amount(ctx.id, Some(amount));
        }
        IncomeUse::Income { kind } => crate::other_income::collect(ctx, kind, amount),
    }
}

// Add every routed amount to the record it names (or the owner's only one).
// Runs on the supported working copy after unsupported records and their
// forms have been left out, so a unresolved or ineligible destination is
// left for the estimate's issue, and nothing is added to it.
pub(crate) fn route(d: &mut FederalDraft) {
    let mut adds: alloc::vec::Vec<(Kind, i64, i64)> = alloc::vec::Vec::new();
    for entry in &d.income {
        for flow in flows(&entry.income) {
            if let Lookup::Found(id) = lookup(d, entry.owner, flow.kind, flow.choice) {
                adds.push((flow.kind, id, flow.amount));
            }
        }
    }
    for (kind, id, amount) in adds {
        let Some(entry) = d.income.iter_mut().find(|entry| entry.id == id) else {
            continue;
        };
        match (&mut entry.income, kind) {
            (FederalIncome::Business { data }, Kind::Business) => {
                if let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity {
                    *receipts += amount;
                }
            }
            (FederalIncome::Rental { data }, Kind::Rental) => data.rent += amount,
            (FederalIncome::NonbusinessProperty { receipts, .. }, Kind::Equipment) => {
                *receipts += amount
            }
            (FederalIncome::Royalty { data }, Kind::Royalty) => data.receipts += amount,
            (FederalIncome::HobbySales { data }, Kind::Hobby) => data.receipts += amount,
            _ => {}
        }
    }
}

// Cents of one form that `route` added to the records it names.
pub(crate) fn routed_from(d: &FederalDraft, entry: &FederalIncomeEntry) -> i64 {
    flows(&entry.income)
        .into_iter()
        .filter(|flow| matches!(lookup(d, entry.owner, flow.kind, flow.choice), Lookup::Found(_)))
        .map(|flow| flow.amount)
        .sum()
}

// Cents the forms of the working copy add to one record, for its review row.
pub(crate) fn routed_to(d: &FederalDraft, id: i64) -> i64 {
    d.income
        .iter()
        .flat_map(|entry| {
            flows(&entry.income)
                .into_iter()
                .filter(move |flow| {
                    lookup(d, entry.owner, flow.kind, flow.choice) == Lookup::Found(id)
                })
                .map(|flow| flow.amount)
        })
        .sum()
}

// The estimate's issue for a form amount no record can receive.
pub(crate) fn destination_issues(
    d: &FederalDraft,
    entry: &FederalIncomeEntry,
    result: &mut crate::FederalResult,
    context: &crate::IssueContext,
    #[cfg(test)] source: &str,
) {
    for flow in flows(&entry.income) {
        match lookup(d, entry.owner, flow.kind, flow.choice) {
            Lookup::Found(_) => {}
            Lookup::Missing => crate::issue_localized!(
                result,
                &context.clone().corrective(flow.kind.section()),
                source,
                "ISSUE_FORM_1099_DESTINATION_MISSING",
                "This Form 1099 amount belongs to a business, rental property, royalty entry, equipment entry or hobby that is not entered yet. Enter it, then choose it on the form.",
            ),
            Lookup::NotChosen => crate::issue_localized!(
                result,
                context,
                source,
                "ISSUE_FORM_1099_DESTINATION_NOT_CHOSEN",
                "This Form 1099 amount needs the record that receives it: choose one of your own businesses (or the rental property, royalty entry, equipment entry or hobby it belongs to) on the form.",
            ),
        }
    }
}

// Schedule C line 2 is bounded by all line 1 receipts, including linked source
// forms once. These are invariant accepted rows outside the edited topic;
// current-page source amounts and automatic destinations stay live in the
// IncomeDocuments rule. This metadata is never accepted as financial input.
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct BusinessReceiptContext {
    #[libertas_read_only]
    #[libertas_output_only]
    pub businesses: alloc::vec::Vec<BusinessReceiptCap>,
    #[libertas_read_only]
    #[libertas_output_only]
    pub sources: alloc::vec::Vec<BusinessReceiptSource>,
}
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessReceiptCap {
    pub id: i64,
    pub owner: Owner,
    pub receipts: i64,
    pub returns: i64,
}
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessReceiptSource {
    pub owner: Owner,
    pub business: Option<i64>,
    pub amount: i64,
}

pub(crate) fn receipt_context(d: &FederalDraft, excluded: &[i64]) -> BusinessReceiptContext {
    let mut context = BusinessReceiptContext::default();
    for entry in d.income.iter().filter(|entry| !excluded.contains(&entry.id)) {
        if let FederalIncome::Business { data } = &entry.income
            && let BusinessActivity::Ordinary { receipts, returns, .. } = data.activity
        {
            context.businesses.push(BusinessReceiptCap {
                id: entry.id,
                owner: entry.owner,
                receipts,
                returns,
            });
        }
        context.sources.extend(
            flows(&entry.income).into_iter().filter(|flow| flow.kind == Kind::Business).map(
                |flow| BusinessReceiptSource {
                    owner: entry.owner,
                    business: flow.choice,
                    amount: flow.amount,
                },
            ),
        );
    }
    context
}

pub(crate) fn receipt_rule(d: &FederalDraft) -> Option<&'static str> {
    receipt_issue(d).map(|(_, issue)| issue.key)
}
pub(crate) fn receipt_issue(d: &FederalDraft) -> Option<(i64, crate::validation::ValidationIssue)> {
    d.income.iter().find_map(|entry| {
        if let FederalIncome::Business { data } = &entry.income
            && let BusinessActivity::Ordinary { receipts, returns, .. } = data.activity
            && returns > receipts + routed_to(d, entry.id)
        {
            Some((
                entry.id,
                crate::validation::ValidationIssue::new("BUSINESS_RETURNS_ABOVE_RECEIPTS")
                    .at_field("returns")
                    .in_field("activity"),
            ))
        } else {
            None
        }
    })
}
