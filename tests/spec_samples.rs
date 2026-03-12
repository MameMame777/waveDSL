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
