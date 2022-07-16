//! Commodity Delta Risk Charge
//! TODO Commodity RiskFactor should be of the form ...CCY (same as FX, where CCY is the reporting CCY)
//! 

use base_engine::prelude::*;
use ndarray::Order;
use ndarray::parallel::prelude::IntoParallelRefIterator;
use crate::sbm::common::*;
use crate::helpers::*;
use crate::prelude::*;

use polars::prelude::*;
use ndarray::{prelude::*, Zip, concatenate};
use ndarray::parallel::prelude::ParallelIterator;

pub fn total_commodity_delta_sens (_: &OCP) -> Expr {
    rc_delta_sens("Commodity")
}
/// Helper functions
pub(crate) fn commodity_delta_sens_weighted_spot() -> Expr {
    rc_tenor_weighted_sens("Delta", "Commodity", "SensitivitySpot","SensWeights", 0)
}
pub(crate) fn commodity_delta_sens_weighted_025y() -> Expr {
    rc_tenor_weighted_sens("Delta","Commodity", "Sensitivity_025Y","SensWeights", 1)
}
pub(crate) fn commodity_delta_sens_weighted_05y() -> Expr {
    rc_tenor_weighted_sens("Delta","Commodity", "Sensitivity_05Y","SensWeights", 2)
}
pub(crate) fn commodity_delta_sens_weighted_1y() -> Expr {
    rc_tenor_weighted_sens("Delta","Commodity", "Sensitivity_1Y","SensWeights", 3)
}
pub(crate) fn commodity_delta_sens_weighted_2y() -> Expr {
    rc_tenor_weighted_sens("Delta","Commodity", "Sensitivity_2Y","SensWeights", 4)
}
pub(crate) fn commodity_delta_sens_weighted_3y() -> Expr {
    rc_tenor_weighted_sens("Delta","Commodity", "Sensitivity_3Y","SensWeights", 5)
}
pub(crate) fn commodity_delta_sens_weighted_5y() -> Expr {
    rc_tenor_weighted_sens("Delta","Commodity", "Sensitivity_5Y","SensWeights", 6)
}
pub(crate) fn commodity_delta_sens_weighted_10y() -> Expr {
    rc_tenor_weighted_sens("Delta","Commodity", "Sensitivity_10Y","SensWeights", 7)
}
pub(crate) fn commodity_delta_sens_weighted_15y() -> Expr {
    rc_tenor_weighted_sens("Delta","Commodity", "Sensitivity_15Y","SensWeights", 8)
}
pub(crate) fn commodity_delta_sens_weighted_20y() -> Expr {
    rc_tenor_weighted_sens("Delta","Commodity", "Sensitivity_20Y", "SensWeights",9)
}
pub(crate) fn commodity_delta_sens_weighted_30y() -> Expr {
    rc_tenor_weighted_sens("Delta","Commodity", "Sensitivity_30Y", "SensWeights",10)
}
/// Total Commodity Delta
pub(crate) fn commodity_delta_sens_weighted(_: &OCP) -> Expr {
    commodity_delta_sens_weighted_spot().fill_null(0.)
    + commodity_delta_sens_weighted_025y().fill_null(0.)
    + commodity_delta_sens_weighted_05y().fill_null(0.)
    + commodity_delta_sens_weighted_1y().fill_null(0.)
    + commodity_delta_sens_weighted_2y().fill_null(0.)
    + commodity_delta_sens_weighted_3y().fill_null(0.)
    + commodity_delta_sens_weighted_5y().fill_null(0.)
    + commodity_delta_sens_weighted_10y().fill_null(0.)
    + commodity_delta_sens_weighted_15y().fill_null(0.)
    + commodity_delta_sens_weighted_20y().fill_null(0.)
    + commodity_delta_sens_weighted_30y().fill_null(0.)
}

///calculate commodity Delta Low Capital charge
pub(crate) fn commodity_delta_charge_low(op: &OCP) -> Expr {
    commodity_delta_charge_distributor(op, &*LOW_CORR_SCENARIO)  
}

///calculate commodity Delta Medium Capital charge
pub(crate) fn commodity_delta_charge_medium(op: &OCP) -> Expr {
    commodity_delta_charge_distributor(op, &*MEDIUM_CORR_SCENARIO)  
}

///calculate commodity Delta High Capital charge
pub(crate) fn commodity_delta_charge_high(op: &OCP) -> Expr {
    commodity_delta_charge_distributor(op, &*HIGH_CORR_SCENARIO)
}

/// Helper funciton
/// Extracts relevant fields from OptionalParams
fn commodity_delta_charge_distributor(_: &OCP, scenario: &'static ScenarioConfig) -> Expr {
    // TODO Accept optional parameters from op
    commodity_delta_charge( 
        scenario.base_com_rho_cty,
         &scenario.com_gamma,
        scenario.base_com_rho_basis_diff,
    &scenario.base_com_rho_tenor,
    scenario.scenario_fn)
}

fn commodity_delta_charge<F>(bucket_rho_basis: [f64; 11], com_gamma: &'static Array2<f64>, 
    com_rho_base_diff_loc: f64, rho_tenor: &'static Array2<f64>, scenario_fn: F)
    -> Expr 
    where F: Fn(f64) -> f64 + Sync + Send + 'static,{  
    
    let n_tenors = 11;

    apply_multiple( move |columns| {
        let df = df![
            "rc" =>   columns[0].clone(), 
            "rf" =>   columns[1].clone(),
            "loc" =>  columns[2].clone(),
            "b" =>    columns[3].clone(),
            "y0" =>   columns[4].clone(),
            "y025" => columns[5].clone(),
            "y05" =>  columns[6].clone(),
            "y1" =>   columns[7].clone(),
            "y2" =>   columns[8].clone(),
            "y3" =>   columns[9].clone(),
            "y5" =>   columns[10].clone(),
            "y10" =>  columns[11].clone(),
            "y15" =>  columns[12].clone(),
            "y20" =>  columns[13].clone(),
            "y30" =>  columns[14].clone(),
        ]?;

        let df = df.lazy()
            .filter(col("rc").eq(lit("Commodity")))
            .groupby([col("b"), col("rf"), col("loc")])
            .agg([
                col("y0").sum(),
                col("y025").sum(),
                col("y05").sum(),
                col("y1").sum(),
                col("y2").sum(),
                col("y3").sum(),
                col("y5").sum(),
                col("y10").sum(),
                col("y15").sum(),
                col("y20").sum(),
                col("y30").sum()            
            ])
            .collect()?;        
        //let k = a; <--can't do this, as a is captured in Fn , so can't move out from there

        // TODO if df is empty return 0s here, as for Rates for example we don't need to go inide buckets
        
        // 21.4.4
        let mut kbs: [f64; 11] = [0.;11];
        // sb = sum {ws_b}
        let mut sbs: [f64; 11] = [0.;11];

        for bucket_df in df.partition_by(["b"])? {
            let n_curves = bucket_df.height();
            let bucket = if let AnyValue::Utf8(b) = unsafe{ bucket_df["b"].get_unchecked(0) } { b } else { unreachable!() };
            let bucket_as_idx = bucket.parse::<usize>().unwrap_or(1);
            
            let comm_arr = bucket_df
            .select(
                ["y0", "y025", "y05", "y1", "y2", "y3", "y5", "y10", "y15", "y20", "y30"])?
            .to_ndarray::<Float64Type>()?;

            let comm_reshaped = comm_arr.to_shape(( n_tenors*n_curves, Order::RowMajor) )
            //.to_shape((1, n_tenors*n_curves) )
            .map_err(|_| PolarsError::ShapeMisMatch("Could not reshape commodity arr".into()) )?;

            // indexes to be removed
            let non_nan_zero_idxs_vec = non_nan_zero_idxs(comm_reshaped.view());
            
            // 21.83.1
            let bucket_rho_basis = bucket_rho_basis[bucket_as_idx-1];
            let rho_cty = build_basis_rho(n_tenors, &bucket_df["rf"], bucket_rho_basis, &non_nan_zero_idxs_vec)?;
            
            // 21.83.2
            let rho_tenor = build_tenor_rho( bucket_df.height(), rho_tenor.view(), &non_nan_zero_idxs_vec)?;

            // 21.83.3
            let rho_basis: Array2<f64> = build_basis_rho(n_tenors, &bucket_df["loc"], com_rho_base_diff_loc, &non_nan_zero_idxs_vec)?;

            //Rhos has been reduced already. Now, reduce weighted deltas by throwing away zeros and nans
            let comm_reshaped = comm_reshaped.select(Axis(0), &non_nan_zero_idxs_vec);
            // 21.83 final
            let mut rho = rho_cty*rho_tenor*rho_basis;
            
            //Apply Scenario rho
            rho.par_mapv_inplace(|el| {scenario_fn(el)});
            //21.4.4
            let a = comm_reshaped.t().dot(&rho);

            //21.4.4
            let kb = a.dot(&comm_reshaped)
                .max(0.)
                .sqrt();

            //21.4.5.a
            let sb = comm_reshaped.sum();

            sbs[bucket_as_idx-1] = sb;
            kbs[bucket_as_idx-1] = kb;
        }
        // If no buckets, early return zeros
        if kbs == [0.;11] && sbs == [0.;11] {
            return Ok( Series::from_vec("res", vec![0.; columns[0].len() ] as Vec<f64>) );
        }

        let sbs_arr = Array1::from_iter(sbs);
        let kbs_arr = Array1::from_iter(kbs);
        

        //21.4.5 sum{ sum {gamma*s_b*s_c} }
        let a = sbs_arr.t().dot(com_gamma);
        let b = a.dot(&sbs_arr);

        //21.4.5 sum{K-b^2}
        let c = kbs_arr.dot(&kbs_arr);

        let sum = c+b;

        let res = if sum < 0. {
            //21.4.5.b
            let mut sbs_alt = Array1::<f64>::zeros(kbs_arr.raw_dim());
            Zip::from(&mut sbs_alt)
                .and(&sbs_arr)
                .and(&kbs_arr)
                .par_for_each(|alt, &sb, &kb|{
                    let _min = sb.min(kb);
                    *alt = _min.max(-kb);
            });
            //now recalculate capital charge with alternative sb
            //21.4.5 sum{ sum {gamma*s_b*s_c} }
            let a = sbs_alt.t().dot(com_gamma);
            let b = a.dot(&sbs_alt);
            //21.4.5 sum{K-b^2}
            let c = kbs_arr.dot(&kbs_arr);
            let sum = c+b;
            sum.sqrt()
        } else {
            sum.sqrt()
        };

        // The function is supposed to return a series of same len as the input, hence we broadcast the result
        let res_arr = Array::from_elem(columns[0].len(), res);
        // if option panics on .unwrap() implement match and use .iter() and then Series from iter
        Ok( Series::new("res", res_arr.as_slice().unwrap() ) )
    }, 
    &[ col("RiskClass"), col("RiskFactor"), col("CommodityLocation"), col("BucketBCBS"), 
    commodity_delta_sens_weighted_spot(), commodity_delta_sens_weighted_025y(), commodity_delta_sens_weighted_05y(),
    commodity_delta_sens_weighted_1y(), commodity_delta_sens_weighted_2y(), commodity_delta_sens_weighted_3y(),
    commodity_delta_sens_weighted_5y(), commodity_delta_sens_weighted_10y(), commodity_delta_sens_weighted_15y(),
    commodity_delta_sens_weighted_20y(), commodity_delta_sens_weighted_30y()], 
        GetOutput::from_type(DataType::Float64))
}

#[deprecated(note="use helpers::build_tenor_rho instead")]
fn build_commodity_rho_matrix_tenor(n_tenors: usize, n_curves: usize, rho_diff_tenor: f64, idx_select: &[usize]) -> Result<Array2<f64>> {
    // 0.99
    // TODO rho_tenor should be pre computed
    let mut rho_tenor = Array2::from_elem((n_tenors, n_tenors), rho_diff_tenor );
    let ones = Array1::<f64>::ones(n_tenors);
    rho_tenor.diag_mut().assign(&ones);
    let vec_rho = vec![rho_tenor.view(); n_curves ];
    let arr_rho_row1 = ndarray::concatenate(Axis(1), vec_rho.as_slice())
        .and_then(|x| Ok(x.select(Axis(1), idx_select)))
        .map_err(|_| PolarsError::ShapeMisMatch("Could not build Commodity Tenor Rho. Invalid Shape".into()))?;

    let mut res = arr_rho_row1.clone();
    for j in 1..n_curves {
        //for each curve shift the block by 10 and then add to res as rows
        res = concatenate![Axis(0), res, shift_right_by(j*n_tenors, &arr_rho_row1)]
    };
    Ok(res.select(Axis(0), idx_select))    
}

