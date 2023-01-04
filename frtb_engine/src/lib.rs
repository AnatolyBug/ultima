//! FRTB job entry point
#![allow(clippy::unnecessary_lazy_evaluations)]
#![doc(html_no_source)]

mod drc;
mod rrao;
mod sbm;

pub mod calc_params;
pub mod docs;
mod helpers;
pub mod measures;
pub mod prelude;
mod risk_weights;
#[cfg(feature = "CRR2")]
mod risk_weights_crr2;
pub mod statics;

//use crate::drc::drc_weights;
use base_engine::polars::prelude::{when, AnyValue, LazyFrame, LiteralValue, NamedFrom, Series};
use base_engine::prelude::*;
//use polars:: series::Series, lazy::dsl::when};
use prelude::{calc_params::frtb_calc_params, drc::common::drc_scalinng, frtb_measure_vec};
use risk_weights::*;
use sbm::buckets;

use std::collections::HashMap;

pub struct FRTBDataSet {
    pub frame: LazyFrame,
    pub measures: MeasuresMap,
    pub build_params: HashMap<String, String>,
    //pub calc_params: Vec<CalcParameter>
}
impl FRTBDataSet {
    /// Helper function which appends bespoke measures to self.measures
    fn with_measures(&mut self, bespoke: Vec<Measure>)
    where
        Self: Sized,
    {
        let self_measures = &mut self.measures;
        self_measures.extend(bespoke.into_iter().map(|m| (m.name.clone(), m)));
    }
}

impl DataSet for FRTBDataSet {
    fn get_lazyframe(&self) -> &LazyFrame {
        &self.frame
    }
    fn get_lazyframe_owned(self) -> LazyFrame {
        self.frame
    }
    /// Modify lf in place
    fn set_lazyframe_inplace(&mut self, lf: LazyFrame) {
        self.frame = lf;
    }
    fn set_lazyframe(self, lf: LazyFrame) -> Self
    where
        Self: Sized,
    {
        Self {
            frame: lf,
            measures: self.measures,
            build_params: self.build_params,
        }
    }
    fn get_measures(&self) -> &MeasuresMap {
        &self.measures
    }
    fn get_measures_owned(self) -> MeasuresMap {
        self.measures
    }
    fn calc_params(&self) -> Vec<CalcParameter> {
        frtb_calc_params()
    }

    fn new(frame: LazyFrame, mm: MeasuresMap, build_params: HashMap<String, String>) -> Self {
        let mut res = Self {
            frame,
            measures: mm,
            build_params,
        };
        res.with_measures(frtb_measure_vec());
        res
    }

    fn collect(self) -> PolarsResult<Self> {
        let lf = self.frame.collect()?.lazy();
        Ok(Self { frame: lf, ..self })
    }
    /// Adds: BCBS buckets, CRR2 Buckets
    /// Adds: SensWeights, CurvatureRiskWeight, SensWeightsCRR2, SeniorityRank
    fn prepare_frame(&self, _lf: Option<LazyFrame>) -> PolarsResult<LazyFrame> {
        let mut lf1 = if let Some(lf) = _lf {
            lf
        } else {
            self.get_lazyframe().clone()
        };

        //First, identify buckets
        lf1 = lf1.with_column(buckets::sbm_buckets(&self.build_params));

        // Then assign SensWeights(BCBS) based on buckets
        lf1 = weights_assign(lf1, &self.build_params)?;
        // TODO Remove after this issue
        // workaround for https://github.com/pola-rs/polars/issues/5812
        let a = Expr::Literal(
            LiteralValue::try_from(AnyValue::List(Series::new("NewVal", [0.]))).unwrap(),
        );
        lf1 = lf1.with_column(
            when(col("SensWeights").is_null())
                .then(a.list())
                .otherwise(col("SensWeights"))
                .alias("SensWeights"),
        );

        // Curvature risk weight
        //lf1 = tmp_frame.lazy()
        lf1 = lf1.with_column(
            when(
                col("PnL_Up")
                    .is_not_null()
                    .or(col("PnL_Down").is_not_null()),
            )
            .then(col("SensWeights").arr().max().alias("CurvatureRiskWeight"))
            .otherwise(NULL.lit()),
        );
        //dbg!(lf1.clone().select([col("*")]).collect());

        // Now,  ammend weights if required. ie has to be done after main assignment of risk weights
        let mut other_cols: Vec<Expr> = vec![];
        // 21.53 Footnote 17
        let csrnonsec_covered_bond_15 = self
            .build_params
            .get("csrnonsec_covered_bond_15")
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or_else(|| false);

        if csrnonsec_covered_bond_15 {
            other_cols.push(
                when(
                    col("RiskClass")
                        .eq(lit("CSR_nonSec"))
                        .and(col("RiskCategory").eq(lit("Delta")))
                        .and(col("BucketBCBS").eq(lit("8")))
                        .and(col("CoveredBondReducedWeight").eq(lit::<bool>(true))),
                )
                .then(Series::new("", &[0.015]).lit().list())
                .otherwise(col("SensWeights"))
                .alias("SensWeights"),
            )
        };

        if !other_cols.is_empty() {
            lf1 = lf1.with_columns(&other_cols)
        };

        // If CRR2 config, we need to derive SensWeightsCRR2
        #[cfg(feature = "CRR2")]
        if cfg!(feature = "CRR2") {
            lf1 = lf1.with_column(buckets::sbm_buckets_crr2());
            //other_cols.push(weights_assign_crr2().alias("SensWeightsCRR2"))
            lf1 = crate::risk_weights_crr2::weights_assign_crr2(lf1, &self.build_params)?;

            // Now, we need to also ammend CRR2 weights
            // Bucket 10 as per
            // https://www.eba.europa.eu/regulation-and-policy/single-rulebook/interactive-single-rulebook/108776
            let mut with_cols = vec![when(
                col("PnL_Up")
                    .is_not_null()
                    .or(col("PnL_Down").is_not_null()),
            )
            .then(
                col("SensWeightsCRR2")
                    .arr()
                    .max()
                    .alias("CurvatureRiskWeightCRR2"),
            )
            .otherwise(NULL.lit())];

            if csrnonsec_covered_bond_15 {
                with_cols.push(
                    when(
                        col("RiskClass")
                            .eq(lit("CSR_nonSec"))
                            .and(col("RiskCategory").eq(lit("Delta")))
                            .and(col("BucketCRR2").eq(lit("10")))
                            .and(col("CoveredBondReducedWeight").eq(lit::<bool>(true))),
                    )
                    .then(Series::new("", &[0.015]).lit().list())
                    .otherwise(col("SensWeightsCRR2"))
                    .alias("SensWeightsCRR2"),
                )
            }

            lf1 = lf1.with_columns(with_cols)
        }

        // Have to collect into a tmp df, as the code panics otherwise
        //let tmp_frame = lf1
        //    .collect()
        //    .expect("Failed to unwrap tmp_frame while .prepare()");
        //lf1 = tmp_frame.lazy()
        lf1 = lf1.with_columns(&[
            when(
                col("RiskClass")
                    .eq(lit("GIRR"))
                    .and(col("RiskCategory").eq(lit("Vega"))),
            )
            .then(col("GirrVegaUnderlyingMaturity").fill_null(col("RiskFactorType")))
            .otherwise(NULL.lit()),
            drc_scalinng(
                self.build_params
                    .get("DayCountConvention")
                    .and_then(|x| x.parse::<u8>().ok()),
                self.build_params.get("DateFormat"),
            )
            .alias("ScaleFactor"),
            //drc_seniority().alias("SeniorityRank"),
        ]);

        // DRC Seniority
        lf1 = drc::drc_weights::with_drc_seniority(lf1);
        //let tmp2_frame = lf1
        //    .collect()
        //    .expect("Failed to unwrap tmp2_frame while .prepare()");

        Ok(lf1)
    }

    // TODO Validate:
    // all required columns are present
    //
    // columns are of expected format
    //
    // if risk class == GIRR => Risk Factor Type is not empty and one of:
    // Yield, XCCY, Inflation
    //
    // Commodity bucket can be strictly an int in 1..11 inclusive (and non empty)
    // Commodity location empty replaced with "Default_Location"
    // Equity bucket can be strictly an int in 1..13 inclusive (and non empty)
    // Equity Risk Factor Type has to be one of: EqSpot, EqRepo (and non empty) for Delta
    //
    // CSR_nonSec BCBS buckets
    // CSR_nonSec CRR2 buckets
    // if csrnonsec_covered_bond_15 == true in build config then
    // If DRC validate CreditQuality
    // fn validate(self) -> Self {}
}
