use crate::{BrokerDocument, BrokerPaper, IncomeSource, InvestmentSaleDocument};

impl BrokerDocument {
    /// The paper an accepted broker document reopens: the answers stay on the source.
    pub(crate) fn income_source(&self) -> IncomeSource {
        let value = match &self.paper {
            BrokerPaper::Form1099B(form) => InvestmentSaleDocument::Form1099B(form.clone()),
            BrokerPaper::Form1099DA(form) => InvestmentSaleDocument::Form1099DA(form.clone()),
        };
        IncomeSource::InvestmentSale { value }
    }
}

pub(crate) fn broker_document(
    source: &IncomeSource,
    accepted: Option<&BrokerDocument>,
) -> Option<BrokerDocument> {
    let paper = match source {
        IncomeSource::InvestmentSale { value: InvestmentSaleDocument::Form1099B(value) } => {
            BrokerPaper::Form1099B(value.clone())
        }
        IncomeSource::InvestmentSale { value: InvestmentSaleDocument::Form1099DA(value) } => {
            BrokerPaper::Form1099DA(value.clone())
        }
        _ => return None,
    };
    Some(paper.into_document(accepted))
}

pub(crate) fn year_issue(
    source: &IncomeSource,
    tax_year: i32,
) -> Option<crate::validation::ValidationIssue> {
    let printed = match source {
        IncomeSource::InvestmentSale { value: InvestmentSaleDocument::Form1099B(form) } => {
            form.year
        }
        IncomeSource::InvestmentSale { value: InvestmentSaleDocument::Form1099DA(form) } => {
            form.year
        }
        _ => return None,
    };
    crate::broker_document::printed_year_issue(printed, i64::from(tax_year))
}
