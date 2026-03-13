use std::fs;

fn compile_fixture(name: &str) -> serde_json::Value {
    let path = format!("tests/fixtures/{}.wdsl", name);
    let input = fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {}", path, e));
    let file_path = std::path::PathBuf::from(&path);
    wavedsl::compile(&input, Some(&file_path)).unwrap_or_else(|errs| {
        for err in &errs {
            eprintln!("{}", err);
        }
        panic!("compilation failed for {}", name);
    })
}

fn compile_fixture_full(name: &str) -> (serde_json::Value, Option<String>) {
    let path = format!("tests/fixtures/{}.wdsl", name);
    let input = fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {}", path, e));
    let file_path = std::path::PathBuf::from(&path);
    wavedsl::compile_full(&input, Some(&file_path)).unwrap_or_else(|errs| {
        for err in &errs {
            eprintln!("{}", err);
        }
        panic!("compilation failed for {}", name);
    })
}

#[test]
fn test_simple_spi() {
    let result = compile_fixture("simple_spi");
    insta::assert_json_snapshot!(result);
}

#[test]
fn test_burst() {
    let result = compile_fixture("burst");
    insta::assert_json_snapshot!(result);
}

#[test]
fn test_axi_group() {
    let result = compile_fixture("axi_group");
    insta::assert_json_snapshot!(result);
}

#[test]
fn test_falling_clock() {
    let result = compile_fixture("falling_clock");
    insta::assert_json_snapshot!(result);
}

#[test]
fn test_complete_example() {
    let result = compile_fixture("complete_example");
    insta::assert_json_snapshot!(result);
}

#[test]
fn test_ddr_timing() {
    let result = compile_fixture("ddr_timing");
    insta::assert_json_snapshot!(result);
}

#[test]
fn test_head_foot_config() {
    let result = compile_fixture("head_foot_config");
    insta::assert_json_snapshot!(result);
}

#[test]
fn test_const_basic() {
    let result = compile_fixture("const_basic");
    insta::assert_json_snapshot!(result);
}

#[test]
fn test_include_basic() {
    let result = compile_fixture("include_basic");
    insta::assert_json_snapshot!(result);
}

#[test]
fn test_const_include() {
    let result = compile_fixture("const_include");
    insta::assert_json_snapshot!(result);
}

#[test]
fn test_assert_wave_json() {
    // assert wave block should appear as a named group in JSON
    let (json, sv) = compile_fixture_full("assert_wave");
    insta::assert_json_snapshot!(json);
    let sv_text = sv.expect("SV should be generated for assert_wave fixture");
    assert!(sv_text.contains("property burst_protocol"));
    assert!(sv_text.contains("$rose(valid)"));
    assert!(sv_text.contains("assert property (burst_protocol)"));
}

#[test]
fn test_assert_cond_json() {
    // when/then assert block: signals appear in JSON, assert block itself doesn't
    let (json, sv) = compile_fixture_full("assert_cond");
    insta::assert_json_snapshot!(json);
    let sv_text = sv.expect("SV should be generated for assert_cond fixture");
    assert!(sv_text.contains("property valid_data_combo"));
    assert!(sv_text.contains("|->"));
    assert!(sv_text.contains("##2"));
}
