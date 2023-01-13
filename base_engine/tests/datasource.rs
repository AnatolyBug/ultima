use base_engine::{read_toml2, DataSet, DataSetBase, DataSourceConfig};

#[test]
fn toml2config() {
    let conf_path = r"./tests/data/test_config.toml";
    read_toml2::<DataSourceConfig>(conf_path).unwrap();
}

#[test]
#[should_panic(expected = "Error reading file")]
fn config_build() {
    let path = String::from(env!("CARGO_MANIFEST_DIR"));
    let conf_path = path + "/tests/data/bad_config.toml";

    let conf = read_toml2::<DataSourceConfig>(conf_path.as_str())
        .expect("Can not proceed without valid Data Set Up");
    let mut _data: DataSetBase = DataSet::from_config(conf);
}

/// In this config, files_join_attributes was provided but no such column is present
#[test]
#[should_panic(expected = "Couldn't build")]
fn config_build2() {
    let path = String::from(env!("CARGO_MANIFEST_DIR"));
    let conf_path = path + "/tests/data/test_config2.toml";
    // TODO: test_config2.toml didn't exist
    let conf = read_toml2::<DataSourceConfig>(conf_path.as_str())
        .expect("Can not proceed without valid Data Set Up");
    let (lf, _, _) = conf.build();
    lf.collect().expect("Couldn't build");
}
