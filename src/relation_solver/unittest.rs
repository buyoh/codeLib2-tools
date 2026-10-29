use super::{solve_tested_by, CodeNode};
use crate::CodeInfo;

fn code_info(deps: &[&str]) -> CodeInfo {
    CodeInfo {
        filepath_dependencies: deps.iter().map(|s| s.to_string()).collect(),
    }
}

fn to_nodes<'a>(v: &'a [(&'static str, CodeInfo)]) -> Vec<CodeNode<'a>> {
    v.iter()
        .map(|(path, code_info)| CodeNode { path, code_info })
        .collect()
}

#[test]
fn test() {
    let srcs = [
        ("/src/a/x.hpp", code_info(&[])),
        ("/src/a/y.hpp", code_info(&["/src/a/x.hpp"])),
        ("/src/b/x.hpp", code_info(&["/src/a/x.hpp"])),
        ("/src/b/y.hpp", code_info(&["/src/a/x.hpp", "/src/b/x.hpp"])),
    ];
    let tests = [
        (
            "/test/1.cpp",
            code_info(&["/src/a/x.hpp", "/test/common/util.hpp"]),
        ),
        ("/test/2.cpp", code_info(&["/src/b/y.hpp"])),
    ];

    let tested_by = solve_tested_by(&to_nodes(&srcs), &to_nodes(&tests));

    assert_eq!(tested_by.get("/src/a/y.hpp"), None);
    assert_eq!(tested_by["/src/b/x.hpp"], vec!["/test/2.cpp"]);
    assert_eq!(tested_by["/src/b/y.hpp"], vec!["/test/2.cpp"]);
    assert_eq!(
        tested_by["/src/a/x.hpp"],
        vec!["/test/1.cpp", "/test/2.cpp"]
    );
    assert_eq!(tested_by.len(), 3);
}

#[test]
fn test_cyclic_dependency() {
    let srcs = [
        ("/src/a.hpp", code_info(&["/src/b.hpp"])),
        ("/src/b.hpp", code_info(&["/src/a.hpp"])),
    ];
    let tests = [("/test/t.cpp", code_info(&["/src/a.hpp"]))];

    let tested_by = solve_tested_by(&to_nodes(&srcs), &to_nodes(&tests));

    assert_eq!(tested_by["/src/a.hpp"], vec!["/test/t.cpp"]);
    assert_eq!(tested_by["/src/b.hpp"], vec!["/test/t.cpp"]);
}
