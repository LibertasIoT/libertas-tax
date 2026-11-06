use crate::{
    Business, CommunityShare, Contracts1099B, EquipmentRent, ForeignAccountOrTrust, Form1099GPaper,
    Form8606, FormW2GPaper, GamblingLosses, InterestDividendDocument, JuryPay, Owner,
    Patr1099Paper, QbiLossCarryforward, QcdGift, Rental, RetirementDocument, RoyaltyProperty,
    Sale1099B, SelfEmploymentDocument, WageDocument,
};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Available income record kinds
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_read_only]
pub struct IncomeSourceChoices {
    /// Amount entry available when adding a business without a linked printed source
    #[libertas_output_only]
    pub manual_work_amount_kinds: Vec<i32>,
    /// Current printed earlier-year Social Security payments
    #[libertas_output_only]
    pub printed_lump_years: Vec<crate::PrintedSocialBenefitYearPage>,
    /// Records offered by this topic
    #[libertas_output_only]
    pub record: Vec<i32>,
}
impl IncomeSourceChoices {
    pub(crate) fn from_kinds(kinds: &[i32]) -> Self {
        let mut choices = Self { manual_work_amount_kinds: alloc::vec![1], ..Self::default() };
        macro_rules! admit {
            ($($kind:ident),+ $(,)?) => {
                $(if kinds.contains(&(libertas_macros::variant_index!(IncomeSourceKind::$kind) as i32)) {
                    choices.record.push(libertas_macros::variant_index!(IncomeRecord::$kind) as i32);
                })+
            };
        }
        admit!(
            SellerMortgageInterest,
            Form8606,
            Qcd,
            Business,
            Rental,
            Royalties,
            EquipmentRental,
            ForeignAccountOrTrust,
            TaxableBondPremium,
            CapitalLossCarryover,
            QbiLossCarryforward,
            BusinessPropertyLosses,
            GamblingLosses,
            Farm,
            FarmAveraging,
            FarmLoss,
            Form8915,
            InstallmentPayments,
            SocialLumpSums,
            MissedRmd,
            NetOperatingLoss,
            AlimonyReceived,
            RothHistory,
            Form8814,
            Section1256Carryback,
            RelatedExchange,
            EducationAccountExpenses,
            FarmRental,
            SmallBusinessStockLoss,
            WorthlessSecurity,
            BadDebt,
            HealthLoanRepayment,
            Scholarship,
            JuryPay,
            HobbySales,
            CasualtyLivingInsurance,
            DisasterMitigation,
            InheritedIra,
            Form2555,
            ForeignPay,
            AbleContributions,
            OtherIncome,
            OpportunityFundDeferral,
            Clergy,
            NonfarmOptionalMethod,
            SpouseSection179,
            InstallmentSale,
            LtcInsured,
            CashTipsWithoutW2,
            PrivateLoanInterest,
            MainHomeSale
        );
        choices
    }
}

// Shared native source settlement receives rows from one declared owner at a time.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IncomeEdits {
    pub documents: Vec<IncomeDocumentItem>,
    pub qbi_aggregations: Vec<crate::QbiAggregationPage>,
}

/// Income records
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IncomeRecords {
    /// Records for this topic
    #[libertas_unordered]
    /// ----
    /// Record
    pub documents: Vec<IncomeRecordItem>,
    /// QBI aggregation elections across your return
    pub qbi_aggregations: Vec<crate::QbiAggregationPage>,
}
impl IncomeRecords {
    pub(crate) fn into_edits(self) -> IncomeEdits {
        IncomeEdits {
            documents: self.documents.into_iter().map(IncomeRecordItem::into_income).collect(),
            qbi_aggregations: self.qbi_aggregations,
        }
    }
}

/// Native applicability for one current income-editor row
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IncomeRefundManagementDerived {
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    pub refund_management_needed: Option<bool>,
}

/// Income document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode)]
pub struct IncomeDocumentItem {
    /// Calculated carryover bounds for this edited source document
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Accepted record identity
    pub id: i64,
    /// Whose document
    pub owner: crate::DocumentOwner,
    /// Document
    pub document: IncomeSource,
    /// Community or separate
    /// Community: pay earned while married and domiciled in a community-property state, and income from community property, including property you can't show is separate. Separate: income from property owned before the marriage, received as a gift or inheritance or bought with separate funds, pay earned while domiciled elsewhere, or property made separate by a written agreement. Partly: an item that is both, such as a pension earned partly during the marriage. IRA and Coverdell distributions are always the account owner's separate income. Your state's law and any agreement between you decide this (Pub. 555). A rental, farm rental or Schedule K-1 is entered for the whole interest as its documents show it, carried losses and basis included; this return takes its share of every amount, and your spouse's return the rest. If your 2025 separate returns already divided it, enter the two returns' carried amounts added together.
    pub community: CommunityShare,
    /// Amount on the return
    pub preview: Option<i64>,
    /// Management of the funds from which this withholding was paid
    /// For Texas community-property refund allocation, use your ownership and any valid management agreement. This does not change who earned the wages or whose form reports the withholding.
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    pub refund_management: Option<crate::RefundManagement>,
    /// Texas source-management answer is needed
    /// The service derives this from the settled native payment source. Null
    /// means that source is not yet available; it is not a negative answer.
    pub refund_management_needed: Option<bool>,
    /// Allowed community-property answers for this row
    /// A row the person adds starts with the page's answers until its
    /// document's own answers arrive.
    pub allowed_community: Vec<i32>,
}

/// Income document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root(
    "PreviewDocument,BusinessRelatedOptions,FarmRelatedOptions,FarmRentalRelatedOptions,RentalRelatedOptions,HobbyRelatedOptions,EquipmentRelatedOptions,RoyaltyRelatedOptions,DocumentPropertyOptions"
)]
pub struct IncomeRecordItem {
    /// Calculated carryover bounds for this edited source document
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Accepted record identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_calculation_request("PreviewDocument")]
    pub id: i64,
    /// Whose document
    #[libertas_constrained_by("$.allowed_owners")]
    #[libertas_calculation_request(
        "BusinessRelatedOptions,FarmRelatedOptions,FarmRentalRelatedOptions,RentalRelatedOptions,HobbyRelatedOptions,EquipmentRelatedOptions,RoyaltyRelatedOptions,PreviewDocument"
    )]
    pub owner: Owner,
    /// Document
    #[libertas_constrained_by("$.source_choices.record")]
    #[libertas_calculation_request(
        "BusinessRelatedOptions,FarmRelatedOptions,FarmRentalRelatedOptions,RentalRelatedOptions,HobbyRelatedOptions,EquipmentRelatedOptions,RoyaltyRelatedOptions,DocumentPropertyOptions,PreviewDocument"
    )]
    #[libertas_ask_on_add]
    pub document: IncomeRecord,
    /// Community or separate
    /// Community: pay earned while married and domiciled in a community-property state, and income from community property, including property you can't show is separate. Separate: income from property owned before the marriage, received as a gift or inheritance or bought with separate funds, pay earned while domiciled elsewhere, or property made separate by a written agreement. Partly: an item that is both, such as a pension earned partly during the marriage. IRA and Coverdell distributions are always the account owner's separate income. Your state's law and any agreement between you decide this (Pub. 555). A rental, farm rental or Schedule K-1 is entered for the whole interest as its documents show it, carried losses and basis included; this return takes its share of every amount, and your spouse's return the rest. If your 2025 separate returns already divided it, enter the two returns' carried amounts added together.
    #[libertas_constrained_by("^<IncomeRecordItem>.allowed_community")]
    #[libertas_calculation_request("PreviewDocument")]
    pub community: CommunityShare,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Management of the funds from which this withholding was paid
    /// For Texas community-property refund allocation, use your ownership and any valid management agreement. This does not change who earned the wages or whose form reports the withholding.
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    pub refund_management: Option<crate::RefundManagement>,
    /// Texas source-management answer is needed
    /// The service derives this from the settled native payment source. Null
    /// means that source is not yet available; it is not a negative answer.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
    /// Allowed community-property answers for this row
    /// A row the person adds starts with the page's answers until its
    /// document's own answers arrive.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.allowed_community")]
    #[libertas_output_only]
    pub allowed_community: Vec<i32>,
}

impl IncomeRecordItem {
    pub(crate) fn into_income(self) -> IncomeDocumentItem {
        IncomeDocumentItem {
            investment_caps: self.investment_caps,
            id: self.id,
            owner: crate::DocumentOwner::from(self.owner),
            document: IncomeSource::Record { value: self.document },
            community: self.community,
            preview: self.preview,
            refund_management: self.refund_management,
            refund_management_needed: self.refund_management_needed,
            allowed_community: self.allowed_community,
        }
    }
    pub(crate) fn from_income(row: IncomeDocumentItem) -> Option<Self> {
        let IncomeSource::Record { value } = row.document else {
            return None;
        };
        Some(Self {
            investment_caps: row.investment_caps,
            id: row.id,
            owner: row.owner.owned(),
            document: value,
            community: row.community,
            preview: row.preview,
            refund_management: row.refund_management,
            refund_management_needed: row.refund_management_needed,
            allowed_community: row.allowed_community,
        })
    }
}

// Native source-kind identifiers used by the topic catalog and admission checks.
#[derive(libertas_macros::VariantIndex)]
#[allow(dead_code)]
pub enum IncomeSourceKind {
    W2,
    Int1099,
    SellerMortgageInterest,
    Div1099,
    Oid1099,
    Sale1099B,
    R1099,
    Ssa1099,
    Form8606,
    Qcd,
    Business,
    Rental,
    Form1099G,
    Royalties,
    EquipmentRental,
    ForeignAccountOrTrust,
    TaxableBondPremium,
    CapitalLossCarryover,
    QbiLossCarryforward,
    BusinessPropertyLosses,
    Patr1099,
    W2g,
    GamblingLosses,
    Farm,
    FarmAveraging,
    FarmLoss,
    K1EstateTrust,
    K1Scorp,
    K1Partnership,
    Form3921,
    Form3922,
    Rrb1099,
    Form8915,
    IsoShareSale,
    InstallmentPayments,
    Cancel1099C,
    Acquisition1099A,
    SocialLumpSums,
    MissedRmd,
    NetOperatingLoss,
    AlimonyReceived,
    RothHistory,
    MainHomeSale,
    SavingsBonds,
    Form8814,
    Contracts1099B,
    Section1256Carryback,
    RelatedExchange,
    Q1099,
    EducationAccountExpenses,
    FarmRental,
    EmployeeShareSale,
    EsppShareSale,
    NuaShareSale,
    QsbsSale,
    SmallBusinessStockLoss,
    WorthlessSecurity,
    BadDebt,
    Form2439,
    Form8919,
    HealthLoanRepayment,
    Scholarship,
    JuryPay,
    HobbySales,
    WildfireRelief,
    DisasterRelief,
    CasualtyLivingInsurance,
    DisasterMitigation,
    Rrb1099R,
    InheritedIra,
    Form2555,
    ForeignPay,
    Sa1099,
    Ltc1099,
    LtcInsured,
    Qa1099,
    AbleContributions,
    OtherIncome,
    Form1099Nec,
    Form1099K,
    Form1099Misc,
    OpportunityFund,
    OpportunityFundDeferral,
    Clergy,
    NonfarmOptionalMethod,
    SpouseSection179,
    K1PartnershipActivities,
    K1ScorpActivities,
    InstallmentSale,
    CashTipsWithoutW2,
    PrivateLoanInterest,
    Form1099B,
    Form1099DA,
    HsaTrusteeStatement,
    Form8818,
    Form1099S,
}

/// Other income document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[allow(clippy::large_enum_variant)]
pub enum OtherIncomeDocument {
    /// 1099-MISC
    /// One for each Form 1099-MISC addressed to you or your spouse. Say where each box with an amount belongs; the App adds it there, so leave it out of that page's own amounts. Crop insurance (box 9) is entered on your farm.
    Form1099Misc(crate::Form1099MiscPaper),
    /// 1099-Q
    /// Payments from a 529 plan (qualified tuition program) or a Coverdell education savings account. One for each Form 1099-Q addressed to you or your spouse.
    Q1099(crate::Form1099QPaper),
    /// 1099-SA
    /// From your own or inherited HSA, Archer MSA or Medicare Advantage MSA. Keep each actual Form 1099-SA as one record, including its dated nonmedical payments. Enter a trustee statement under its own document kind. Contributions and contribution corrections stay on Health and medical savings accounts.
    Sa1099(crate::Form1099SaPaper),
    /// 1099-LTC
    /// Long-term care insurance or accelerated death benefits. One for each Form 1099-LTC addressed to you or your spouse as policyholder.
    Ltc1099(crate::Ltc1099Paper),
    /// 1099-QA
    /// Distributions from an ABLE account. One for each Form 1099-QA addressed to you or your spouse.
    Qa1099(crate::Form1099QaPaper),
    /// Wildfire relief payment
    /// Compensation for losses from a federally declared forest or range fire, excluded under section 139M. One for each payment.
    WildfireRelief(crate::FederalWildfireRelief),
    /// Disaster relief payment
    /// Help with personal, living, funeral or home-repair expenses after a qualified disaster, excluded under section 139. One for each payment. East Palestine train derailment payments are not taxable and need not be entered.
    DisasterRelief(crate::FederalDisasterRelief),
    /// HSA/MSA trustee statement
    /// A trustee statement for an HSA, Archer MSA or Medicare Advantage MSA that is not a Form 1099-SA. Keep each statement as one record. Contributions and contribution corrections stay on Health and medical savings accounts.
    HsaTrusteeStatement(crate::HsaTrusteeStatementPaper),
}

/// K1 document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[allow(clippy::large_enum_variant)]
pub enum K1Document {
    /// Schedule K-1 (Form 1065)
    /// From a partnership, or an LLC taxed as one, including a fiscal-year K-1. Keep attached entity totals and section 199A / QBI statements here; amounts in boxes 1, 2 and 3 do not by themselves identify a distinct-activity statement.
    K1Partnership(crate::K1PartnershipDocument),
    /// Partnership K-1 with a distinct-activity statement.
    /// Only when an attached statement separately identifies activities and allocates K-1 box amounts among those activities, such as two named businesses sharing box 1. An entity-total summary or section 199A / QBI breakdown alone belongs with Schedule K-1 (Form 1065), even when it reports ordinary business and rental amounts. The printed form year and fiscal-year dates do not distinguish these kinds.
    K1PartnershipActivities(alloc::boxed::Box<crate::K1PartnershipActivitiesPage>),
    /// Schedule K-1 (Form 1120-S)
    /// From an S corporation, including a fiscal-year K-1. Keep attached entity totals and section 199A / QBI statements here; amounts in boxes 1, 2 and 3 do not by themselves identify a distinct-activity statement.
    K1Scorp(crate::K1ScorpDocument),
    /// S-corporation K-1 with a distinct-activity statement.
    /// Only when an attached statement separately identifies activities and allocates K-1 box amounts among those activities, such as two named businesses sharing box 1. An entity-total summary or section 199A / QBI breakdown alone belongs with Schedule K-1 (Form 1120-S), even when it reports ordinary business and rental amounts. The printed form year and fiscal-year dates do not distinguish these kinds.
    K1ScorpActivities(crate::K1ScorpActivitiesPage),
    /// Schedule K-1 (Form 1041)
    /// From an estate or trust.
    K1EstateTrust(crate::K1EstateTrustDocument),
}

/// Canceled debt document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[allow(clippy::large_enum_variant)]
pub enum CanceledDebtDocument {
    /// 1099-C
    /// Cancellation of debt.
    Cancel1099C(crate::Form1099C),
    /// 1099-A
    /// A lender's foreclosure or repossession of property.
    Acquisition1099A(crate::Form1099A),
}

/// Income records
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[allow(clippy::large_enum_variant)]
pub enum IncomeRecord {
    /// Interest from financing the buyer of your home
    SellerMortgageInterest(crate::SellerMortgageInterest),
    /// Last year's Form 8606
    Form8606(Form8606),
    /// Gift from your IRA directly to a charity
    Qcd(QcdGift),
    /// Business
    /// Schedule C. A farm goes on the Farming topic.
    Business(Business),
    /// Rental property
    Rental(Rental),
    /// Royalties
    /// Royalties from oil, gas or mineral property, or from a copyright or patent you own that is not your own current work (Schedule E). One for each property; enter its Form 1099-MISC in Documents, then add its box 2 royalties under Related forms here.
    Royalties(RoyaltyProperty),
    /// Equipment you rent out
    /// Equipment or other movable property you rent out for profit but not as a business (Schedule 1 lines 8l and 24b). Rent for real estate belongs under Rental property. Enter its Form 1099-MISC in Documents, then add its box 1 rents under Related forms here.
    EquipmentRental(EquipmentRent),
    /// Foreign account or trust
    ForeignAccountOrTrust(ForeignAccountOrTrust),
    /// Election to amortize taxable bond premium
    /// One choice for all taxable bonds. Check your retained amortization election and annual broker statements. Yes claims documented annual amortization; No keeps known income and omits only unclaimed premium relief. A net broker amount is never reduced twice.
    TaxableBondPremium(bool),
    /// Last year's capital loss carryover
    CapitalLossCarryover(crate::PriorCapitalReturnPage),
    /// Last year's QBI loss carryforward
    QbiLossCarryforward(QbiLossCarryforward),
    /// Business-property losses from 2021–2025
    /// Needed when you sold business or rental property in 2026: say whether a 2021–2025 return deducted a net loss from business-property sales (Form 4797 line 7, carried to line 11). Enter it once for the whole return.
    BusinessPropertyLosses(crate::BusinessPropertyLosses),
    /// Gambling losses and winnings without a W-2G
    /// Enter it once for the whole return.
    GamblingLosses(GamblingLosses),
    /// Farm (Schedule F)
    /// One for each farm you run. Crop or livestock shares from land you rent out without taking part are a Farm rental.
    Farm(crate::Farm),
    /// Farm income averaging (Schedule J)
    /// Enter it once for the return. It can lower your tax when this year's farm income is high; the review shows your tax with and without it.
    FarmAveraging(crate::FarmAveragingElection),
    /// 2026 farm loss: carryback choice
    /// Enter it once for the return.
    FarmLoss(crate::FarmLossWaiver),
    /// Earlier Form 8915-F
    /// Your 2024 or 2025 qualified disaster distributions still being included in income or repaid. One for each form.
    Form8915(crate::EarlierForm8915),
    /// Installment payments from an earlier-year sale
    /// Principal received in 2026 on a sale before 2026 reported on the installment method. Use Installment sale in 2026 for investment or personal-use property sold this year. Enter a business asset, vehicle, rental or farm land installment sale on that asset.
    InstallmentPayments(crate::InstallmentSale),
    /// Social Security paid in 2026 for earlier years
    /// A lump sum shown in SSA-1099 box 3 as paid in 2026 for an earlier year. Enter it once for the return; the App uses the lump-sum election only when it lowers your taxable benefits.
    SocialLumpSums(crate::LumpSumElection),
    /// Missed required minimum distribution
    /// A 2026 required distribution from an IRA or plan that was not fully taken by its deadline (Form 5329 Part IX). One for each plan or group of IRAs.
    MissedRmd(crate::MissedDistribution),
    /// Net operating loss from an earlier year
    /// Business losses from earlier years still unused at the start of 2026 (Form 172). Enter it once for the return.
    NetOperatingLoss(crate::LossCarryforward),
    /// Alimony received
    /// Under a divorce or separation agreement signed before 2019. One for each agreement; choose whose it is by who received it.
    AlimonyReceived(crate::AlimonyPayment),
    /// Roth IRA history
    /// Needed for a 1099-R with code J or T: your first Roth IRA year and the contributions and conversions not yet taken out. One for each owner.
    RothHistory(crate::RothIraHistory),
    /// Child's interest and dividends (Form 8814)
    /// Report a child's interest and dividends on your return instead of filing a return for the child. One for each child.
    Form8814(crate::Form8814),
    /// 2026 section 1256 loss: carryback choice
    /// Only if you elect to carry back a net loss from futures or section 1256 options to 2023–2025 (Form 6781 box D). Enter it once for the return.
    Section1256Carryback(crate::Section1256Carryback),
    /// Related-party exchange from 2024 or 2025
    /// A like-kind exchange with a relative or a related company whose property was sold or given away in 2026, within two years of the exchange (Form 8824 Part II). Enter a 2026 exchange on the rental you gave up.
    RelatedExchange(crate::RelatedExchange),
    /// 529 and Coverdell expenses
    /// The costs those payments paid for, one row for each student on this return. Enter it once for the whole return; the taxable earnings of every Form 1099-Q come from it.
    EducationAccountExpenses(crate::EducationAccountExpenses),
    /// Farm rental (Form 4835)
    /// Crop or livestock shares from land you rent out without taking part in the farming. One for each rental.
    FarmRental(crate::FarmRental),
    /// Small business stock loss (section 1244)
    /// Stock you bought at original issue from a small domestic corporation, sold at a loss or worthless in 2026. Up to $50,000 of the loss ($100,000 on a joint return) is an ordinary loss.
    SmallBusinessStockLoss(crate::Section1244Stock),
    /// Security that became worthless
    /// Stock or a bond held as an investment that became completely worthless in 2026. It counts as sold for nothing on December 31.
    WorthlessSecurity(crate::WorthlessSecurity),
    /// Personal loan or debt that became worthless
    /// Money you lent, a guarantee you paid or a deposit you cannot get back, now completely uncollectible. It is a short-term capital loss.
    BadDebt(crate::NonbusinessBadDebt),
    /// Student loan repaid by a health-service program
    /// National Health Service Corps Loan Repayment Program, or a state program for underserved or health professional shortage areas. One for each program.
    HealthLoanRepayment(crate::HealthLoanAssistance),
    /// Scholarship or fellowship
    /// A grant not on your W-2. The part not spent on tuition, required fees and course books, supplies and equipment is taxable (Schedule 1 line 8r). One for each award.
    Scholarship(crate::Scholarship),
    /// Jury duty pay
    /// Enter it once for each person who served.
    JuryPay(JuryPay),
    /// Sales from a hobby
    /// Goods sold from an activity you don't run to make a profit (Schedule 1 line 8j). A business goes on Self-employment.
    HobbySales(crate::HobbySales),
    /// Insurance for living expenses after losing your home
    /// An insurer's payments for temporary housing and other extra living costs while you could not live in your main home. One for each event.
    CasualtyLivingInsurance(crate::FederalCasualtyLivingInsurance),
    /// Hazard mitigation grant
    /// A FEMA or flood-insurance program payment to make your property safer from future disasters, excluded under section 139(g). One for each grant.
    DisasterMitigation(crate::FederalDisasterMitigation),
    /// Inherited IRA
    /// An IRA or Roth IRA you inherited from someone other than your spouse, or from your spouse without making it your own. One for each person you inherited from and each kind of IRA.
    InheritedIra(crate::InheritedIraAccount),
    /// Form 2555: foreign earned income exclusion
    /// One for each person who lived and worked abroad with a foreign tax home. It holds the qualifying test, your elections and housing; mark W-2 pay for work abroad on its W-2 under Wages and a business run abroad under More about this business on Self-employment.
    Form2555(crate::Form2555),
    /// Pay from a foreign employer
    /// Pay for work abroad that no W-2 reports. One for each employer.
    ForeignPay(crate::ForeignPay),
    /// ABLE contributions
    /// Contributions to the ABLE accounts of one beneficiary on this return for 2026, to figure the 6% tax on contributions above the limit (Form 5329 Part VIII). One for each beneficiary.
    AbleContributions(crate::AbleContributionsPage),
    /// Other income
    /// Income with no form of its own here: an Alaska Permanent Fund dividend, a prize without a 1099-MISC, digital assets received as income, a refund of an earlier year's itemized deduction, or other taxable income. One for each item. A Form 1099-MISC or 1099-NEC is entered as that form.
    OtherIncome(crate::OtherIncomeItem),
    /// 2026 gain invested in an opportunity fund in 2027
    /// A capital gain from a 2026 sale that you invested in a qualified opportunity fund in 2027, within 180 days of the sale, and elect to defer (Form 8997 Part II, Form 8949 code Z). One for each fund and investment date.
    OpportunityFundDeferral(crate::QofDeferral),
    /// Minister, religious worker or Form4029 exemption
    /// One per person. Mark ministerial W2s and business fees separately. Earnings under a vow of poverty belong to the order; do not enter them. A retired minister's pension housing designation is not excluded here; its otherwise taxable pension amount remains taxable. Enter foreign church pay without a W2 as a ministerial business.
    Clergy(crate::ClergyRecord),
    /// Nonfarm optional method (ScheduleSE PartII)
    /// One per person. This optional coverage election can increase self-employment tax and never changes actual business profit.
    NonfarmOptionalMethod(crate::NonfarmSeHistory),
    /// Spouse's section 179 purchases
    /// Asked when you file separately and elect section 179: your spouse's purchases and the allocation of the shared dollar limit. Enter it once for the return.
    SpouseSection179(crate::SpouseSection179),
    /// Installment sale in 2026
    /// Investment or personal-use property; enter actual interest separately as interest income. Business, rental and farm assets use their own sale entries.
    InstallmentSale(crate::InvestmentInstallmentSale),
    /// Long-term care periods
    /// For per diem payments: the days of care, the care costs and reimbursements for one insured person. One for each insured person.
    LtcInsured(crate::LtcInsured),
    /// Cash tips not reported on a W-2
    /// One record per employer. Do not repeat tips already entered on that employer's W-2 tip records.
    CashTipsWithoutW2(crate::CashTipsWithoutW2),
    /// Interest on a private loan you made
    /// Personal or seller-financed loan interest without a 1099-INT. Exclude principal and interest already entered elsewhere.
    PrivateLoanInterest(crate::PrivateLoanInterest),
    /// Sale of your main home without a 1099-S
    /// Use your closing statement and home records. Enter a printed Form 1099-S in Documents instead.
    MainHomeSale {
        /// The sale from your records
        sale: crate::Form1099S,
        /// The facts about the home you sold
        answers: crate::HomeSaleAnswers,
    },
}

/// Income source
#[derive(
    Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, libertas_macros::VariantIndex,
)]
#[allow(clippy::large_enum_variant)]
pub enum IncomeSource {
    /// Wage
    Wage {
        /// Document
        value: WageDocument,
    },
    /// InterestDividend
    InterestDividend {
        /// Document
        value: InterestDividendDocument,
    },
    /// InvestmentSale
    InvestmentSale {
        /// Document
        value: InvestmentSaleDocument,
    },
    /// Retirement
    Retirement {
        /// Document
        value: RetirementDocument,
    },
    /// SelfEmployment
    SelfEmployment {
        /// Document
        value: SelfEmploymentDocument,
    },
    /// Farming
    Farming(Patr1099Paper),
    /// GovernmentPayment
    GovernmentPayment {
        /// Document
        value: Form1099GPaper,
        /// State that paid the box 2 refund
        /// Saving fills it once from the paying agency's printed address when that shows a state; a state already recorded is kept. Otherwise identify the refund-paying agency's state, not your current home state.
        /// [NullText]
        /// The paying agency's printed address shows no state
        refund_state: Option<crate::TaxState>,
        /// Locality that paid a local box 2 refund
        /// The local refund agency or locality; leave empty when none.
        refund_locality: alloc::string::String,
    },
    /// OtherIncome
    OtherIncome {
        /// Document
        value: OtherIncomeDocument,
    },
    /// Gambling
    Gambling(FormW2GPaper),
    /// K1
    K1 {
        /// Document
        value: K1Document,
    },
    /// CanceledDebt
    CanceledDebt {
        /// Document
        value: CanceledDebtDocument,
    },
    /// Topic records
    Record {
        /// Record
        value: IncomeRecord,
    },
}

impl IncomeSource {
    pub(crate) fn has_value_field(&self) -> bool {
        !matches!(self, Self::Farming(_) | Self::Gambling(_))
    }
}

/// Investment sale document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[allow(clippy::large_enum_variant)]
pub enum InvestmentSaleDocument {
    /// Sale from your own records
    /// Stock, bonds, an option you bought or wrote, a digital asset or personal property with no broker form. One for each sale. Available printed Forms 1099-B and 1099-DA have their own entries.
    Sale1099B(Sale1099B),
    /// Futures and section 1256 options from your records
    /// Regulated futures, foreign currency contracts and section 1256 options when no printed Form 1099-B is available.
    Contracts1099B(Contracts1099B),
    /// Form 1099-S
    /// Proceeds from a real estate transaction, as printed: a home, land, a rental or other real estate. Investment sales asks what you sold. Without a Form 1099-S, a main-home sale is entered from your own records there.
    #[libertas_llm_doc_field]
    Form1099S(crate::Form1099SPaper),
    /// Form 3921
    /// Exercise of an incentive stock option (ISO).
    #[libertas_llm_doc_field]
    Form3921(crate::Form3921Paper),
    /// Form 3922 employee stock purchase plan acquisition
    #[libertas_llm_doc_field]
    Form3922(crate::Form3922Paper),
    /// ISO shares sold
    /// Shares from an incentive stock option sold in 2026, with their 1099-B and the employer's stock plan record, including shares exercised in an earlier year.
    #[libertas_llm_doc_field]
    IsoShareSale(crate::IsoShareSale),
    /// Employee stock shares sold
    /// Shares from a nonqualified stock option, restricted stock units or a stock award, sold in 2026, with their 1099-B and the employer's stock plan record.
    #[libertas_llm_doc_field]
    EmployeeShareSale(crate::EmployeeShareSale),
    /// ESPP shares sold
    /// Shares from an employee stock purchase plan sold in 2026, with their 1099-B and the employer's stock plan record or Form 3922.
    #[libertas_llm_doc_field]
    EsppShareSale(crate::EsppShareSale),
    /// NUA shares sold
    /// Employer shares a retirement plan distributed to you, sold in 2026, with their 1099-B and the plan's record of the distributed shares.
    #[libertas_llm_doc_field]
    NuaShareSale(crate::NuaShareSale),
    /// Stock sale closing statement or qualified small business stock (section 1202)
    /// A paying agent's or closing statement of a sale of one corporation's stock that is not a Form 1099-B, whether or not it names qualified small business stock, or the Form 1099-B of qualified small business stock; one for each issuing corporation. The printed statement does not establish eligibility; the topic asks the necessary stock records.
    #[libertas_llm_doc_field]
    QsbsSale(crate::QsbsSale),
    /// Qualified opportunity fund investment (Form 8997)
    /// An investment in a qualified opportunity fund made before 2026 that still defers a gain: one row of Part IV of your 2025 Form 8997. Its deferral ends on December 31, 2026, so the deferred gain is taxed this year, or when you sold the investment if that was earlier. One for each investment.
    #[libertas_llm_doc_field]
    OpportunityFund(crate::QofInvestmentPaper),
    /// Form 1099-B
    /// Actual broker or barter-exchange paper, including contracts reported in boxes 8–11.
    #[libertas_llm_doc_field]
    Form1099B(crate::Form1099B),
    /// Form 1099-DA
    /// Actual digital-asset broker paper.
    #[libertas_llm_doc_field]
    Form1099DA(crate::Form1099DA),
}
