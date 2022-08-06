//! Server side entry point
//! This to be conversted into server

use base_engine::prelude::*;
use toml::Value;

use std::process;
use std::sync::Arc;
use log::info;
use serde::{Serialize, Deserialize};
use std::time::Instant;
#[cfg(feature = "FRTB")]
use frtb_engine::prelude::*;
#[cfg(target_os = "linux")]
use jemallocator::Jemalloc;
#[cfg(not(target_os = "linux"))]
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
#[cfg(feature = "FRTB")]
type DataSetType = frtb_engine::FRTBDataSet;
#[cfg(not(feature = "FRTB"))]
type DataSetType = base_engine::DataSetBase;

fn main() {
    // Read .env
    dotenv::dotenv().ok();
    // Allow pretty logs
    pretty_env_logger::init();
    // Read Config
    let conf = read_toml2::<DataSourceConfig>(SETUP).expect("Can not proceed without valid Data Set Up"); //Unrecovarable error
    info!("Data SetUp: {:?}", conf);

    // Build data
    let mut data = DataSetType::build(conf);
    // Pre build some columns, which you wish to store in memory alongside the original data
    data.prepare();

    ////owner of column names
    let numer_cols = data.numeric_columns_owned(vec![]); 
    println!("numeric columns: {:?}", numer_cols);

    //### Measures Struct ###
    //Owner of the measures which point to numer_cols
    let measures_vec = derive_basic_measures_vec(data.measure_cols());
    //Vec of pointers to owner(s) of the measures
    let mut ptrs_measures_vecs = vec![&measures_vec];
    #[cfg(feature = "FRTB")]
    if cfg!(feature = "FRTB") {
        //Extend the Vec with another pointer to an owner
        ptrs_measures_vecs.push(&*FRTB_MEASURE_VEC)
    }
    //measures_map holds pointers to data owned by measures_vec and frtb_engine::MEASURE_VEC
    let measures_map = derive_measures_map(ptrs_measures_vecs);
    let arc_measures_map = Arc::new(measures_map);
    //### ### ###

    let message: Message = serde_json::from_str(JSON).unwrap();
    info!("{:?}", message);
    let now = Instant::now();
    match message {
        Message::Request{ params: conf, ..} => {
            match base_engine::execute(conf, &data, Arc::clone(&arc_measures_map)){
                Err(e) =>{ // eventually will be tokio::spawn_blocking
                    eprintln!("Application error: {:#?}", e);
                    process::exit(1);
                },
                Ok(df) => {
                    let elapsed = now.elapsed();
                    println!("result: {:?}", df);
                    println!("Elapsed: {:.6?}", elapsed);}
            }
        },
        _ => ()
    };
}


// public params: Request
// bespoke params: FRTBRequest
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Message {
    Request { id: String, method: String, params: DataRequestS },
    Response { id: String, result: PlaceHolder },
}

#[derive(Serialize, Deserialize, Debug)]
struct PlaceHolder(u8);

/*
/// Sample request
const JSON: &str = r#"
{"type": "Request",
    "id": "123", 
    "method": "None", 
    "params": {
        "cob": "2022-04-05",
        "measures": [["Delta", "sum"]],
        "groupby": ["Desk"],
        "filters": [{"Eq":[["LegalEntity", "EMEA"], ["Country", "UK"]]}]
    }
}"#;

/// Sample request 2
const JSON: &str = r#"
{"type": "Request",
    "id": "123", 
    "method": "None", 
    "params": {
        "cob": "2022-04-05",
        "measures": [["Delta", "sum"]],
        "groupby": ["Desk"],
        "reporting_ccy": "USD",
        "filters": [{"In":[["LegalEntity", ["EMEA"]], ["Country", ["UK", "China"]]]}]
    }
}"#;

/// Sample request 3
const JSON: &str = r#"
{"type": "Request",
    "id": "123", 
    "method": "None", 
    "params": {
        "measures": [
                    ["SensitivitySpot", "sum"],
                    ["FXDeltaSens", "sum"]
                    ["Commodity_DeltaCharge_Medium", "quantile95low"]
                    ],
        "groupby": ["Country", "Desk"],
        "filters": [
                    {"Neq":[ ["LegalEntity", "Asia"], ["Country", "UK"] ]},
                    {"In":[["LegalEntity", ["EMEA"]], ["Country", ["UK", "China"]]]}
                    ]
    }
}"#;


/// Sample request 4
const JSON: &str = r#"
{"type": "Request",
    "id": "123", 
    "method": "None", 
    "params": {
        "measures": [
            ["FXDeltaSens", "sum"],
            ["FxDeltaSensWeighted", "sum"],
            ["FxDeltaChargeLow", "first"],
            ["FxDeltaChargeMedium", "first"],
            ["FxDeltaChargeHigh", "first"]],
        "groupby": ["Desk"],
        "filters": []
    }
}"#;
["SensWeights", "list"] , 
["TotalDeltaSens", "sum"],
["SensitivitySpot", "sum"], 
["FXDeltaSens", "sum"], 
["FxDeltaSensWeighted", "sum"],


const JSON: &str = r#"
{"type": "Request",
    "id": "123", 
    "method": "None", 
    "params": {
        "measures": [
            ["GIRRDeltaChargeLow", "first"],
            ["GIRRDeltaChargeMedium", "first"],
            ["GIRRDeltaChargeHigh", "first"]  
        ],
        "groupby": ["Desk"],
        "filters": []
    }
}"#;

["Equity_DeltaSens", "sum"],
["Equity_DeltaSens_Weighted", "sum"],

["Commodity_DeltaSens", "sum"],
["Commodity_DeltaSens_Weighted", "sum"],



["CSR_Sec_nonCTP_DeltaSens", "sum"],
["CSR_Sec_nonCTP_DeltaSens_Weighted", "sum"],

["CSR_secCTP_DeltaSens", "sum"],
["CSR_secCTP_DeltaSens_Weighted", "sum"],

["CSR_nonSec_DeltaSens", "sum"],
["CSR_nonSec_DeltaSens_Weighted", "sum"],



["Commodity_DeltaCharge_Low", "first"],
["Commodity_DeltaCharge_Medium", "first"],
["Commodity_DeltaCharge_High", "first"],

["Equity_DeltaCharge_Low", "first"],
["Equity_DeltaCharge_Medium", "first"],
["Equity_DeltaCharge_High", "first"],

["CSR_nonSec_DeltaCharge_Low", "first"],
["CSR_nonSec_DeltaCharge_Medium", "first"],
["CSR_nonSec_DeltaCharge_High", "first"],

["CSR_secCTP_DeltaCharge_Low", "first"],
["CSR_secCTP_DeltaCharge_Medium", "first"],
["CSR_secCTP_DeltaCharge_High", "first"],

["CSR_Sec_nonCTP_DeltaCharge_Low", "first"],
["CSR_Sec_nonCTP_DeltaCharge_Medium", "first"],
["CSR_Sec_nonCTP_DeltaCharge_High", "first"],

["FX_DeltaSens", "sum"],
["FX_DeltaSens_Weighted", "sum"],
["FX_DeltaSb", "first"],
["FX_DeltaKb", "first"],
["FX_DeltaCharge_Low", "first"],
["FX_DeltaCharge_Medium", "first"],
["FX_DeltaCharge_High", "first"],

["FX_VegaSens", "sum"],
["FX_VegaSens_Weighted", "sum"],
["FX_VegaSb", "first"],
["FX_VegaKb_Low", "first"],
["FX_VegaKb_Medium", "first"],
["FX_VegaKb_High", "first"],
["FX_VegaCharge_Low", "first"],
["FX_VegaCharge_Medium", "first"],
["FX_VegaCharge_High", "first"],

["FX_CurvatureDelta", "sum"],
["FX_CurvatureDelta_Weighted", "sum"],
["FX_PnLup", "sum"],
["FX_PnLdown", "sum"],
["FX_CVRup", "sum"],
["FX_CVRdown", "sum"],
["FX_Curvature_KbPlus", "first"],
["FX_Curvature_KbMinus", "first"],
["FX_Curvature_Kb", "first"],
["FX_Curvature_Sb", "first"],
["FX_CurvatureCharge_Low", "first"],
["FX_CurvatureCharge_Medium", "first"],
["FX_CurvatureCharge_High", "first"]


["GIRR_DeltaSens", "sum"],
["GIRR_DeltaSens_Weighted", "sum"],
["GIRR_DeltaSb", "sum"],
["GIRR_DeltaKb_Low", "first"],
["GIRR_DeltaKb_Medium", "first"],
["GIRR_DeltaKb_High", "first"],
["GIRR_DeltaCharge_Low", "first"],
["GIRR_DeltaCharge_Medium", "first"],
["GIRR_DeltaCharge_High", "first"],

["GIRR_VegaSens", "sum"],
["GIRR_VegaSens_Weighted", "sum"],
["GIRR_VegaSb", "first"],
["GIRR_VegaKb_Low", "first"],
["GIRR_VegaKb_Medium", "first"],
["GIRR_VegaKb_High", "first"],
["GIRR_VegaCharge_Low", "first"],
["GIRR_VegaCharge_Medium", "first"],
["GIRR_VegaCharge_High", "first"],

["GIRR_CurvatureDelta", "sum"],
["GIRR_PnLup", "sum"],
["GIRR_PnLdown", "sum"],
["GIRR_CurvatureDelta_Weighted", "sum"],
["GIRR_CVRup", "sum"],
["GIRR_CVRdown", "sum"],
["GIRR_Curvature_KbPlus", "first"],
["GIRR_Curvature_KbMinus", "first"],
["GIRR_Curvature_Kb", "first"],
["GIRR_Curvature_Sb", "first"],
["GIRR_CurvatureCharge_Low", "first"],
["GIRR_CurvatureCharge_Medium", "first"],
["GIRR_CurvatureCharge_High", "first"]

["PnL_Up", "sum"],
["PnL_Down", "sum"]


"reporting_ccy": "USD"
"filters": [{"Eq":[["Desk", "FXOptions"]]}],
"base_csr_nonsec_tenor_rho": "{\"v\":1,\"dim\":[5,5],\"data\":[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0]}"
"base_csr_nonsec_diff_name_rho_per_bucket": "[1.0,2.0]" <- Example of bad input. Parsing would go for a default  
["Desk","FXCash"],["Desk","RatesEM"]
*/

const JSON: &str = r#"
{"type": "Request",
    "id": "123", 
    "method": "SEND", 
    "params": {
        "measures": [
            ["GIRR_CurvatureDelta", "sum"],
["GIRR_PnLup", "sum"],
["GIRR_PnLdown", "sum"],
["GIRR_CurvatureDelta_Weighted", "sum"],
["GIRR_CVRup", "sum"],
["GIRR_CVRdown", "sum"],
["GIRR_Curvature_KbPlus", "first"],
["GIRR_Curvature_KbMinus", "first"],
["GIRR_Curvature_Kb", "first"],
["GIRR_Curvature_Sb", "first"],
["GIRR_CurvatureCharge_Low", "first"],
["GIRR_CurvatureCharge_Medium", "first"],
["GIRR_CurvatureCharge_High", "first"]
        ],
        "groupby": ["Desk"],
        "filters": [{"Eq": [["Desk","RatesEM"]]}],
        "optional_params": {
            "hide_zeros": true,
            "calc_params": {"jurisdiction": "BCBS",
            "apply_fx_curv_div": "true"}
        }
    }
}"#;

//to be passed as a command line argument
const SETUP: &str = r"frtb_engine/examples/data/datasource_config.toml";
