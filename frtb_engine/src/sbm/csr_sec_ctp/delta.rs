//! CSR Sec CTP Delta Calculations
//!
use crate::helpers::*;
use base_engine::prelude::*;
use sbm::csr_nonsec::delta::csr_nonsec_delta_charge;

use crate::prelude::*;
use polars::prelude::*;

pub fn total_csr_sec_ctp_delta_sens(_: &OCP) -> Expr {
    rc_rcat_sens("Delta", "CSR_Sec_CTP", total_delta_sens())
}
/// Helper functions

fn csr_sec_ctp_delta_sens_weighted_05y_bcbs() -> Expr {
    rc_tenor_weighted_sens("Delta", "CSR_Sec_CTP", "Sensitivity_05Y", "SensWeights", 0)
}
fn csr_sec_ctp_delta_sens_weighted_1y_bcbs() -> Expr {
    rc_tenor_weighted_sens("Delta", "CSR_Sec_CTP", "Sensitivity_1Y", "SensWeights", 1)
}
fn csr_sec_ctp_delta_sens_weighted_3y_bcbs() -> Expr {
    rc_tenor_weighted_sens("Delta", "CSR_Sec_CTP", "Sensitivity_3Y", "SensWeights", 2)
}
fn csr_sec_ctp_delta_sens_weighted_5y_bcbs() -> Expr {
    rc_tenor_weighted_sens("Delta", "CSR_Sec_CTP", "Sensitivity_5Y", "SensWeights", 3)
}
fn csr_sec_ctp_delta_sens_weighted_10y_bcbs() -> Expr {
    rc_tenor_weighted_sens("Delta", "CSR_Sec_CTP", "Sensitivity_10Y", "SensWeights", 4)
}

//CRR2
#[cfg(feature = "CRR2")]
fn csr_sec_ctp_delta_sens_weighted_05y_crr2() -> Expr {
    rc_tenor_weighted_sens(
        "Delta",
        "CSR_Sec_CTP",
        "Sensitivity_05Y",
        "SensWeightsCRR2",
        0,
    )
}
#[cfg(feature = "CRR2")]
fn csr_sec_ctp_delta_sens_weighted_1y_crr2() -> Expr {
    rc_tenor_weighted_sens(
        "Delta",
        "CSR_Sec_CTP",
        "Sensitivity_1Y",
        "SensWeightsCRR2",
        1,
    )
}
#[cfg(feature = "CRR2")]
fn csr_sec_ctp_delta_sens_weighted_3y_crr2() -> Expr {
    rc_tenor_weighted_sens(
        "Delta",
        "CSR_Sec_CTP",
        "Sensitivity_3Y",
        "SensWeightsCRR2",
        2,
    )
}
#[cfg(feature = "CRR2")]
fn csr_sec_ctp_delta_sens_weighted_5y_crr2() -> Expr {
    rc_tenor_weighted_sens(
        "Delta",
        "CSR_Sec_CTP",
        "Sensitivity_5Y",
        "SensWeightsCRR2",
        3,
    )
}
#[cfg(feature = "CRR2")]
fn csr_sec_ctp_delta_sens_weighted_10y_crr2() -> Expr {
    rc_tenor_weighted_sens(
        "Delta",
        "CSR_Sec_CTP",
        "Sensitivity_10Y",
        "SensWeightsCRR2",
        4,
    )
}

/// Total weighted CSR non-Sec Delta
/// Not used in calculation
pub(crate) fn csr_sec_ctp_delta_sens_weighted(op: &OCP) -> Expr {
    let juri: Jurisdiction = get_jurisdiction(op);

    match juri {
        #[cfg(feature = "CRR2")]
        Jurisdiction::CRR2 => {
            csr_sec_ctp_delta_sens_weighted_05y_crr2().fill_null(0.)
                + csr_sec_ctp_delta_sens_weighted_1y_crr2().fill_null(0.)
                + csr_sec_ctp_delta_sens_weighted_3y_crr2().fill_null(0.)
                + csr_sec_ctp_delta_sens_weighted_5y_crr2().fill_null(0.)
                + csr_sec_ctp_delta_sens_weighted_10y_crr2().fill_null(0.)
        }
        Jurisdiction::BCBS => {
            csr_sec_ctp_delta_sens_weighted_05y_bcbs().fill_null(0.)
                + csr_sec_ctp_delta_sens_weighted_1y_bcbs().fill_null(0.)
                + csr_sec_ctp_delta_sens_weighted_3y_bcbs().fill_null(0.)
                + csr_sec_ctp_delta_sens_weighted_5y_bcbs().fill_null(0.)
                + csr_sec_ctp_delta_sens_weighted_10y_bcbs().fill_null(0.)
        }
    }
}

//Interm Results
///Sb is same for each scenario
pub(crate) fn csr_sec_ctp_delta_sb(op: &OCP) -> Expr {
    csr_sec_ctp_delta_charge_distributor(op, &*LOW_CORR_SCENARIO, ReturnMetric::Sb)
}

pub(crate) fn csr_sec_ctp_delta_kb_low(op: &OCP) -> Expr {
    csr_sec_ctp_delta_charge_distributor(op, &*LOW_CORR_SCENARIO, ReturnMetric::Kb)
}

pub(crate) fn csr_sec_ctp_delta_kb_medium(op: &OCP) -> Expr {
    csr_sec_ctp_delta_charge_distributor(op, &*MEDIUM_CORR_SCENARIO, ReturnMetric::Kb)
}

pub(crate) fn csr_sec_ctp_delta_kb_high(op: &OCP) -> Expr {
    csr_sec_ctp_delta_charge_distributor(op, &*HIGH_CORR_SCENARIO, ReturnMetric::Kb)
}

///calculate CSR non-Sec Delta Low Capital charge
pub(crate) fn csr_sec_ctp_delta_charge_low(op: &OCP) -> Expr {
    csr_sec_ctp_delta_charge_distributor(op, &*LOW_CORR_SCENARIO, ReturnMetric::CapitalCharge)
}

///calculate CSR non-Sec Delta Medium Capital charge
pub(crate) fn csr_sec_ctp_delta_charge_medium(op: &OCP) -> Expr {
    csr_sec_ctp_delta_charge_distributor(op, &*MEDIUM_CORR_SCENARIO, ReturnMetric::CapitalCharge)
}

///calculate CSR non-Sec Delta High Capital charge
pub(crate) fn csr_sec_ctp_delta_charge_high(op: &OCP) -> Expr {
    csr_sec_ctp_delta_charge_distributor(op, &*HIGH_CORR_SCENARIO, ReturnMetric::CapitalCharge)
}

/// Helper funciton
/// Extracts relevant fields from OptionalParams
/// And pass them to the main Delta Charge calculator accordingly
/// calls csr_nonsec_delta_charge because the calculation is identical
fn csr_sec_ctp_delta_charge_distributor(
    op: &OCP,
    scenario: &'static ScenarioConfig,
    rtrn: ReturnMetric,
) -> Expr {
    //unimplemented!();
    let juri: Jurisdiction = get_jurisdiction(op);

    // First, obtaining parameters specific to jurisdiciton
    let (weight, bucket_col, name_rho_vec, gamma, n_buckets, special_bucket) = match juri {
        #[cfg(feature = "CRR2")]
        Jurisdiction::CRR2 => (
            [col("SensWeightsCRR2").arr().get(0),
            col("SensWeightsCRR2").arr().get(1),
            col("SensWeightsCRR2").arr().get(2),
            col("SensWeightsCRR2").arr().get(3),
            col("SensWeightsCRR2").arr().get(4),],
            col("BucketCRR2"),
            Vec::from(scenario.base_csr_nonsec_rho_name_crr2),
            &scenario.csr_ctp_gamma_crr2,
            18usize,
            Option::<usize>::None,
        ),
        Jurisdiction::BCBS => (
            [col("SensWeights").arr().get(0),
            col("SensWeights").arr().get(1),
            col("SensWeights").arr().get(2),
            col("SensWeights").arr().get(3),
            col("SensWeights").arr().get(4),],
            col("BucketBCBS"),
            Vec::from(scenario.base_csr_ctp_rho_name_bcbs),
            &scenario.csr_ctp_gamma,
            16usize,
            Option::<usize>::None,
        ),
    };

    // Checking if request contains overrides
    let base_csr_ctp_rho_tenor = get_optional_parameter(
        op,
        "base_csr_ctp_tenor_rho",
        &scenario.base_csr_ctp_rho_tenor,
    );

    let name_rho_vec =
        get_optional_parameter_vec(op, "base_csr_ctp_diff_name_rho_per_bucket", &name_rho_vec);

    let base_csr_ctp_rho_basis = get_optional_parameter(
        op,
        "base_csr_ctp_diff_basis_rho",
        &scenario.base_csr_nonsec_rho_basis,
    );

    let gamma = get_optional_parameter_array(op, "base_csr_ctp_rating_gamma", gamma);

    // CTP calc is identical to nonSec, with the only exception on rho, gamma and number of buckets
    csr_nonsec_delta_charge(
        weight,
        base_csr_ctp_rho_tenor,
        name_rho_vec,
        base_csr_ctp_rho_basis,
        bucket_col,
        scenario.scenario_fn,
        gamma,
        n_buckets,
        special_bucket,
        "CSR_Sec_CTP",
        "Delta",
        rtrn,
    )
}

/// Returns max of three scenarios
/// !Note This is not a real measure, as MAX should be taken as
/// MAX(ir_delta_low+ir_vega_low+eq_curv_low, ..._medium, ..._high).
/// This is for convienience view only.
fn csrsecctp_delta_max(op: &OCP) -> Expr {
    max_exprs(
    &[csr_sec_ctp_delta_charge_low(op), 
    csr_sec_ctp_delta_charge_medium(op),
    csr_sec_ctp_delta_charge_high(op)
    ]
)
}

/// Exporting Measures
pub(crate) fn csrsecctp_delta_measures() -> Vec<Measure<'static>> {
    vec![
        Measure {
            name: "CSR_secCTP_DeltaSens".to_string(),
            calculator: Box::new(total_csr_sec_ctp_delta_sens),
            aggregation: None,
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Delta"))
                    .and(col("RiskClass").eq(lit("CSR_Sec_CTP"))),
            ),
        },
        Measure {
            name: "CSR_secCTP_DeltaSens_Weighted".to_string(),
            calculator: Box::new(csr_sec_ctp_delta_sens_weighted),
            aggregation: None,
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Delta"))
                    .and(col("RiskClass").eq(lit("CSR_Sec_CTP"))),
            ),
        },
        Measure {
            name: "CSR_secCTP_DeltaSb".to_string(),
            calculator: Box::new(csr_sec_ctp_delta_sb),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Delta"))
                    .and(col("RiskClass").eq(lit("CSR_Sec_CTP"))),
            ),
        },
        Measure {
            name: "CSR_secCTP_DeltaKb_Low".to_string(),
            calculator: Box::new(csr_sec_ctp_delta_kb_low),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Delta"))
                    .and(col("RiskClass").eq(lit("CSR_Sec_CTP"))),
            ),
        },
        Measure {
            name: "CSR_secCTP_DeltaKb_Medium".to_string(),
            calculator: Box::new(csr_sec_ctp_delta_kb_medium),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Delta"))
                    .and(col("RiskClass").eq(lit("CSR_Sec_CTP"))),
            ),
        },
        Measure {
            name: "CSR_secCTP_DeltaKb_High".to_string(),
            calculator: Box::new(csr_sec_ctp_delta_kb_high),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Delta"))
                    .and(col("RiskClass").eq(lit("CSR_Sec_CTP"))),
            ),
        },
        Measure {
            name: "CSR_secCTP_DeltaCharge_Low".to_string(),
            calculator: Box::new(csr_sec_ctp_delta_charge_low),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Delta"))
                    .and(col("RiskClass").eq(lit("CSR_Sec_CTP"))),
            ),
        },
        Measure {
            name: "CSR_secCTP_DeltaCharge_Medium".to_string(),
            calculator: Box::new(csr_sec_ctp_delta_charge_medium),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Delta"))
                    .and(col("RiskClass").eq(lit("CSR_Sec_CTP"))),
            ),
        },
        Measure {
            name: "CSR_secCTP_DeltaCharge_High".to_string(),
            calculator: Box::new(csr_sec_ctp_delta_charge_high),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Delta"))
                    .and(col("RiskClass").eq(lit("CSR_Sec_CTP"))),
            ),
        },
        Measure {
            name: "CSR_secCTP_DeltaCharge_MAX".to_string(),
            calculator: Box::new(csrsecctp_delta_max),
            aggregation: Some("first"),
            precomputefilter: Some(
                col("RiskCategory")
                    .eq(lit("Delta"))
                    .and(col("RiskClass").eq(lit("CSR_Sec_CTP"))),
            ),
        },
    ]
}
