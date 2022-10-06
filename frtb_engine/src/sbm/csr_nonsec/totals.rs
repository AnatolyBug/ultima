use base_engine::Measure;
use base_engine::OCP;
use polars::prelude::*;

use super::curvature::*;
use super::delta::*;
use super::vega::*;
use crate::sbm::totals::total_sum;

pub(crate) fn csrnonsec_total_low(op: &OCP) -> Expr {
    total_sum(&[
        csr_nonsec_delta_charge_low(op),
        csr_nonsec_vega_charge_low(op),
        csrnonsec_curvature_charge_low(op),
    ])
}
pub(crate) fn csrnonsec_total_medium(op: &OCP) -> Expr {
    total_sum(&[
        csr_nonsec_delta_charge_medium(op),
        csr_nonsec_vega_charge_medium(op),
        csrnonsec_curvature_charge_medium(op),
    ])
}
pub(crate) fn csrnonsec_total_high(op: &OCP) -> Expr {
    total_sum(&[
        csr_nonsec_delta_charge_high(op),
        csr_nonsec_vega_charge_high(op),
        csrnonsec_curvature_charge_high(op),
    ])
}

/// Not a real measure. Used for analysis only
fn csrnonsec_total_max(op: &OCP) -> Expr {
    max_exprs(&[csrnonsec_total_low(op), csrnonsec_total_medium(op), csrnonsec_total_high(op)])
}

pub(crate) fn csrnonsec_total_measures() -> Vec<Measure> {
    vec![
        Measure {
            name: "CSR nonSec TotalCharge Low".to_string(),
            calculator: Box::new(csrnonsec_total_low),
            aggregation: Some("first"),
            precomputefilter: Some(col("RiskClass").eq(lit("CSR_nonSec"))),
        },
        Measure {
            name: "CSR nonSec TotalCharge Medium".to_string(),
            calculator: Box::new(csrnonsec_total_medium),
            aggregation: Some("first"),
            precomputefilter: Some(col("RiskClass").eq(lit("CSR_nonSec"))),
        },
        Measure {
            name: "CSR nonSec TotalCharge High".to_string(),
            calculator: Box::new(csrnonsec_total_high),
            aggregation: Some("first"),
            precomputefilter: Some(col("RiskClass").eq(lit("CSR_nonSec"))),
        },
        Measure {
            name: "CSR nonSec TotalCharge MAX".to_string(),
            calculator: Box::new(csrnonsec_total_max),
            aggregation: Some("first"),
            precomputefilter: Some(col("RiskClass").eq(lit("CSR_nonSec"))),
        },
    ]
}
