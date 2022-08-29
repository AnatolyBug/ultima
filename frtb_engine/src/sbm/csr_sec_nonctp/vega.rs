use base_engine::prelude::*;
use crate::{prelude::*, sbm::equity::vega::equity_vega_charge};

use polars::prelude::*;

pub fn total_csr_sec_nonctp_vega_sens (_: &OCP) -> Expr {
    rc_rcat_sens("Vega", "CSR_Sec_nonCTP", total_vega_curv_sens())
}

pub fn total_csr_sec_nonctp_vega_sens_weighted (op: &OCP) -> Expr {
    total_csr_sec_nonctp_vega_sens(op)*col("SensWeights").arr().get(0)
}
///Interm Result
pub(crate) fn csr_sec_nonctp_vega_sb(op: &OCP) -> Expr {
    csr_sec_nonctp_vega_charge_distributor(op, &*MEDIUM_CORR_SCENARIO, ReturnMetric::Sb)  
}
pub(crate) fn csr_sec_nonctp_vega_kb_low(op: &OCP) -> Expr {
    csr_sec_nonctp_vega_charge_distributor(op, &*LOW_CORR_SCENARIO, ReturnMetric::Kb)  
}

///calculate Sec nonCTP Vega Low Capital charge
pub(crate) fn csr_sec_nonctp_vega_charge_low(op: &OCP) -> Expr {
    csr_sec_nonctp_vega_charge_distributor(op, &*LOW_CORR_SCENARIO, ReturnMetric::CapitalCharge)  
}

///Interm Result
pub(crate) fn csr_sec_nonctp_vega_kb_medium(op: &OCP) -> Expr {
    csr_sec_nonctp_vega_charge_distributor(op, &*MEDIUM_CORR_SCENARIO, ReturnMetric::Kb)  
}

///calculate Sec nonCTP Vega Low Capital charge
pub(crate) fn csr_sec_nonctp_vega_charge_medium(op: &OCP) -> Expr {
    csr_sec_nonctp_vega_charge_distributor(op, &*MEDIUM_CORR_SCENARIO, ReturnMetric::CapitalCharge)  
}

///Interm Result
pub(crate) fn csr_sec_nonctp_vega_kb_high(op: &OCP) -> Expr {
    csr_sec_nonctp_vega_charge_distributor(op, &*HIGH_CORR_SCENARIO, ReturnMetric::Kb)  
}

///calculate Sec nonCTP Vega Low Capital charge
pub(crate) fn csr_sec_nonctp_vega_charge_high(op: &OCP) -> Expr {
    csr_sec_nonctp_vega_charge_distributor(op, &*HIGH_CORR_SCENARIO, ReturnMetric::CapitalCharge)  
}

/// Helper funciton
/// Extracts relevant fields from OptionalParams
fn csr_sec_nonctp_vega_charge_distributor(op: &OCP, scenario: &'static ScenarioConfig, rtrn: ReturnMetric) -> Expr {
    let _suffix = scenario.as_str();
    //TODO check
    let csr_sec_nonctp_gamma = get_optional_parameter_array(op, format!("csr_sec_nonctp_vega_gamma{_suffix}").as_str(), &scenario.csr_sec_nonctp_gamma);
    let csr_sec_nonctp_rho_bucket = get_optional_parameter(op, "base_csr_sec_nonctp_rho_diff_name_bucket", &scenario.base_csr_sec_nonctp_rho_diff_name);
    let csr_sec_nonctp_vega_rho = get_optional_parameter_array(op, "base_csr_sec_nonctp_opt_mat_vega_rho", &scenario.base_vega_rho);

    equity_vega_charge(csr_sec_nonctp_vega_rho, 
        csr_sec_nonctp_gamma, 
        csr_sec_nonctp_rho_bucket.to_vec(), 
    scenario.scenario_fn, rtrn, Some("25"), "CSR_Sec_nonCTP")
}
