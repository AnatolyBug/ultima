use base_engine::polars::prelude::Expr;
use base_engine::{Measure, PolarsResult, CPM, DependantMeasure};
use base_engine::polars::lazy::dsl::col;

// TODO NOTE: add DRC Sec CTP - currently missing
pub(crate) fn drc_charge(_: &CPM) -> PolarsResult<Expr> {
    Ok(col("DRC nonSec CapitalCharge") + col("DRC Sec nonCTP CapitalCharge"))
}

pub(crate) fn drc_total_measures() -> Vec<Measure> {
    vec![
    
    DependantMeasure {
        name: "DRC Charge".to_string(),
        calculator: Box::new(drc_charge),
        depends_upon: vec![
            ("DRC nonSec CapitalCharge".to_string(), "scalar".to_string()),
            ("DRC Sec nonCTP CapitalCharge".to_string(), "scalar".to_string()),
        ],
    }.into()
    
    ]
}
