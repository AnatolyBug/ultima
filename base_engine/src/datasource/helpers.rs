use std::collections::HashMap;

use polars::{
    prelude::{
        col, concat, DataFrame, DataType, Expr, Field, JoinType, LazyCsvReader, LazyFrame, Literal,
        NamedFrom, PolarsResult, Schema, NULL,
    },
    series::Series,
};

use crate::{derive_basic_measures_vec, numeric_columns, Measure};

/// creates an empty frame with columns
pub fn empty_frame(with_columns: &[String]) -> DataFrame {
    let mut x: Vec<Series> = Vec::with_capacity(with_columns.len());
    let y: [String; 0] = [];
    for c in with_columns {
        x.push(Series::new(c, &y));
    }
    DataFrame::new_no_checks(x)
}

/// reads DataFrame from path, casts cols to str and numeric cols to f64
pub fn path_to_lf(path: &str, cast_to_str: &[String], cast_to_f64: &[String]) -> LazyFrame {
    let mut vc = Vec::with_capacity(cast_to_str.len() + cast_to_f64.len());
    for str_col in cast_to_str {
        vc.push(Field::new(str_col, DataType::Utf8))
    }
    for f64_col in cast_to_f64 {
        vc.push(Field::new(f64_col, DataType::Float64))
    }

    let schema = Schema::from_iter(vc);

    // if path provided, then we expect it to be of the correct format
    // unrecoverable. Panic if failed to read file
    let lf = LazyCsvReader::new(path)
        .has_header(true)
        .with_parse_dates(true)
        .with_dtype_overwrite(Some(&schema))
        //.with_ignore_parser_errors(ignore)
        .finish()
        .unwrap_or_else(|_| panic!("Error reading file: {path}"));

    lf
}

pub fn finish(
    a2h: Vec<String>,
    f2a: Vec<String>,
    measures: Vec<String>,
    mut df_attr: LazyFrame,
    df_hms: LazyFrame,
    mut concatinated_frame: LazyFrame,
    build_params: HashMap<String, String>,
) -> (LazyFrame, Vec<Measure>, HashMap<String, String>) {
    // join with hms if a2h was provided
    if !a2h.is_empty() {
        let a2h_expr = a2h.iter().map(|c| col(c)).collect::<Vec<Expr>>();
        df_attr = df_attr.join(df_hms, a2h_expr.clone(), a2h_expr, JoinType::Left)
        //.collect()
        //.expect("Could not join attributes to hms. Review attributes_join_hierarchy field in the setup");
    }
    // if files to attributes was provided
    if !f2a.is_empty() {
        let f2a_expr = f2a.iter().map(|c| col(c)).collect::<Vec<Expr>>();
        concatinated_frame =
            concatinated_frame.join(df_attr, f2a_expr.clone(), f2a_expr, JoinType::Outer)
        //.collect()
        //.expect("Could not join files with attributes-hms. Review files_join_attributes field in the setup");
    }

    // if measures were provided
    let measures = if !measures.is_empty() {
        let schema = concatinated_frame
            .schema()
            .expect("Could not extract Schema");
        let fields = schema
            .iter_fields()
            .map(|f| f.name)
            .collect::<Vec<String>>();

        // Checking if each measure is present in DF
        measures.iter().for_each(|col| {
            if !fields.contains(col) {
                panic!("Measure: {}, is not part of the fields: {:?}", col, fields)
            }
        });
        derive_basic_measures_vec(measures)
    }
    // If not provided return all numeric columns
    // TODO move inside the DataSet see FRTBDataSetWrapper::new
    else {
        let num_cols = concatinated_frame.schema()
            .and_then(|schema| Ok(numeric_columns(schema)))
            .unwrap_or_default();
        derive_basic_measures_vec(num_cols)
    };

    (concatinated_frame, measures, build_params)
}

/// TODO contribute to Polars
/// Concat [LazyFrame]s diagonally.
/// Calls [concat] internally.
pub fn diag_concat_lf<L: AsRef<[LazyFrame]>>(
    lfs: L,
    rechunk: bool,
    parallel: bool,
) -> PolarsResult<LazyFrame> {
    let lfs = lfs.as_ref().to_vec();
    let schemas = lfs
        .iter()
        .map(|lf| lf.schema())
        .collect::<PolarsResult<Vec<_>>>()?;

    let upper_bound_width = schemas.iter().map(|sch| sch.len()).sum();

    // Use Vec to preserve order
    let mut column_names = Vec::with_capacity(upper_bound_width);
    let mut total_schema = Vec::with_capacity(upper_bound_width);

    for sch in schemas.iter() {
        sch.iter().for_each(|(name, dtype)| {
            if !column_names.contains(name) {
                column_names.push(name.clone());
                total_schema.push((name.clone(), dtype.clone()));
            }
        });
    }

    //Append column if missing
    let lfs_with_all_columns = lfs
        .into_iter()
        // Zip Frames with their Schemas
        .zip(schemas.into_iter())
        .map(|(mut lf, lf_schema)| {
            for (name, dtype) in total_schema.iter() {
                // If a name from Total Schema is not present - append
                if lf_schema.get_field(name).is_none() {
                    lf = lf.with_column(NULL.lit().cast(dtype.clone()).alias(name))
                }
            }

            // Now, reorder to match schema
            let reordered_lf = lf.select(
                column_names
                    .iter()
                    .map(|col_name| col(col_name))
                    .collect::<Vec<Expr>>(),
            );

            Ok(reordered_lf)
        })
        .collect::<PolarsResult<Vec<_>>>()?;

    concat(lfs_with_all_columns, rechunk, parallel)
}
