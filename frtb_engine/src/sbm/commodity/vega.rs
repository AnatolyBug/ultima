use crate::{prelude::*, sbm::equity::vega::equity_vega_charge};
use base_engine::prelude::*;

use polars::prelude::*;

pub fn total_com_vega_sens(_: &OCP) -> Expr {
    rc_rcat_sens("Vega", "Commodity", total_vega_curv_sens())
}

pub fn total_com_vega_sens_weighted(op: &OCP) -> Expr {
    total_com_vega_sens(op) * col("SensWeights").arr().get(0)
}
///Interm Result
pub(crate) fn com_vega_sb(op: &OCP) -> Expr {
    com_vega_charge_distributor(op, &*MEDIUM_CORR_SCENARIO, ReturnMetric::Sb)
}
pub(crate) fn com_vega_kb_low(op: &OCP) -> Expr {
    com_vega_charge_distributor(op, &*LOW_CORR_SCENARIO, ReturnMetric::Kb)
}

///calculate Equity Vega Low Capital charge
pub(crate) fn com_vega_charge_low(op: &OCP) -> Expr {
    com_vega_charge_distributor(op, &*LOW_CORR_SCENARIO, ReturnMetric::CapitalCharge)
}

///Interm Result
pub(crate) fn com_vega_kb_medium(op: &OCP) -> Expr {
    com_vega_charge_distributor(op, &*MEDIUM_CORR_SCENARIO, ReturnMetric::Kb)
}

///calculate Equity Vega Low Capital charge
pub(crate) fn com_vega_charge_medium(op: &OCP) -> Expr {
    com_vega_charge_distributor(op, &*MEDIUM_CORR_SCENARIO, ReturnMetric::CapitalCharge)
}

///Interm Result
pub(crate) fn com_vega_kb_high(op: &OCP) -> Expr {
    com_vega_charge_distributor(op, &*HIGH_CORR_SCENARIO, ReturnMetric::Kb)
}

///calculate Equity Vega Low Capital charge
pub(crate) fn com_vega_charge_high(op: &OCP) -> Expr {
    com_vega_charge_distributor(op, &*HIGH_CORR_SCENARIO, ReturnMetric::CapitalCharge)
}

/// Helper funciton
/// Extracts relevant fields from OptionalParams
/// TODO test
fn com_vega_charge_distributor(
    op: &OCP,
    scenario: &'static ScenarioConfig,
    rtrn: ReturnMetric,
) -> Expr {
    let _suffix = scenario.as_str();

    let com_gamma = get_optional_parameter_array(
        op,
        format!("commodity_delta_gamma{_suffix}").as_str(),
        &scenario.com_gamma,
    );
    let com_rho_bucket = get_optional_parameter(
        op,
        format!("commodity_base_delta_rho_bucket{_suffix}").as_str(),
        &scenario.base_com_delta_rho_cty,
    );
    let com_vega_rho = get_optional_parameter_array(
        op,
        format!("commodity_base_opt_mat_vega_rho{_suffix}").as_str(),
        &scenario.base_vega_rho,
    );

    // The approach is identical to Equity
    equity_vega_charge(
        com_vega_rho,
        com_gamma,
        com_rho_bucket.to_vec(),
        scenario.scenario_fn,
        rtrn,
        None,
        "Commodity",
    )
}

/// Exporting Measures
pub(crate) fn com_vega_measures() -> Vec<Measure<'static>> {
    vec![
        Measure {
            name: "Commodity_VegaSens".to_string(),
            calculator: Box::new(total_com_vega_sens),
            aggregation: None,
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Vega"))
                    .and(col("RiskClass").eq(lit("Commodity"))),
            ),
        },
        Measure {
            name: "Commodity_VegaSens_Weighted".to_string(),
            calculator: Box::new(total_com_vega_sens_weighted),
            aggregation: None,
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Vega"))
                    .and(col("RiskClass").eq(lit("Commodity"))),
            ),
        },
        Measure {
            name: "Commodity_VegaSb".to_string(),
            calculator: Box::new(com_vega_sb),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Vega"))
                    .and(col("RiskClass").eq(lit("Commodity"))),
            ),
        },
        Measure {
            name: "Commodity_VegaKb_Low".to_string(),
            calculator: Box::new(com_vega_kb_low),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Vega"))
                    .and(col("RiskClass").eq(lit("Commodity"))),
            ),
        },
        Measure {
            name: "Commodity_VegaCharge_Low".to_string(),
            calculator: Box::new(com_vega_charge_low),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Vega"))
                    .and(col("RiskClass").eq(lit("Commodity"))),
            ),
        },
        Measure {
            name: "Commodity_VegaKb_Medium".to_string(),
            calculator: Box::new(com_vega_kb_medium),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Vega"))
                    .and(col("RiskClass").eq(lit("Commodity"))),
            ),
        },
        Measure {
            name: "Commodity_VegaCharge_Medium".to_string(),
            calculator: Box::new(com_vega_charge_medium),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Vega"))
                    .and(col("RiskClass").eq(lit("Commodity"))),
            ),
        },
        Measure {
            name: "Commodity_VegaKb_High".to_string(),
            calculator: Box::new(com_vega_kb_high),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Vega"))
                    .and(col("RiskClass").eq(lit("Commodity"))),
            ),
        },
        Measure {
            name: "Commodity_VegaCharge_High".to_string(),
            calculator: Box::new(com_vega_charge_high),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Vega"))
                    .and(col("RiskClass").eq(lit("Commodity"))),
            ),
        },
    ]
}
