use regex::Regex;
use std::path::Path;
use std::{fs, io::Write};

use syn::{parse_file, ImplItem, Item, ItemFn, ReturnType, Type};

use super::problem::{CodeDefinition, Problem};

pub fn deal_problem(problem: &Problem, code: &CodeDefinition, write_mod_file: bool) {
    let file_name = format!(
        "p{:04}_{}",
        problem.question_id,
        problem.title_slug.replace('-', "_")
    );

    let file_path = Path::new("./src/problem").join(format!("{}.rs", file_name));
    if file_path.exists() {
        println!("Problem {} already exists", file_name);
        return;
    }

    let fixed_code = insert_return_type(&code.default_code);
    // println!("{}", res);

    let template = fs::read_to_string("./template.rs").unwrap();
    let source = template
        .replace("__PROBLEM_TITLE__", &problem.title)
        .replace("__PROBLEM_DESC__", &build_desc(&problem.content))
        .replace(
            "__PROBLEM_DEFAULT_CODE__",
            &insert_return_in_code(&problem.return_type, &code.default_code),
        )
        .replace("__PROBLEM_ID__", &format!("{}", problem.question_id))
        .replace("__EXTRA_USE__", &parse_extra_use(&code.default_code))
        .replace("__PROBLEM_LINK__", &parse_problem_link(problem))
        .replace("__DISCUSS_LINK__", &parse_discuss_link(problem));

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file_path)
        .unwrap();

    file.write_all(source.as_bytes()).unwrap();
    drop(file);

    if write_mod_file {
        let mut lib_file = fs::OpenOptions::new()
            .append(true)
            .open("./src/problem/mod.rs")
            .unwrap();
        writeln!(lib_file, "\nmod {};\n", file_name).unwrap();
    }
}

fn parse_extra_use(code: &str) -> String {
    let mut extra_use_line = String::new();
    // a linked-list problem
    if code.contains("pub struct ListNode") {
        extra_use_line.push_str("\nuse crate::util::linked_list::{ListNode, to_list};")
    }
    if code.contains("pub struct TreeNode") {
        extra_use_line.push_str("\nuse crate::util::tree::{TreeNode, to_tree};")
    }
    if code.contains("pub struct Point") {
        extra_use_line.push_str("\nuse crate::util::point::Point;")
    }
    extra_use_line
}

fn parse_problem_link(problem: &Problem) -> String {
    format!("https://leetcode.com/problems/{}/", problem.title_slug)
}

fn parse_discuss_link(problem: &Problem) -> String {
    format!(
        "https://leetcode.com/problems/{}/discuss/?currentPage=1&orderBy=most_votes&query=",
        problem.title_slug
    )
}

fn build_desc(content: &str) -> String {
    // TODO: fix this shit
    content
        .replace("<strong>", "")
        .replace("</strong>", "")
        .replace("<em>", "")
        .replace("</em>", "")
        .replace("</p>", "")
        .replace("<p>", "")
        .replace("<b>", "")
        .replace("</b>", "")
        .replace("<pre>", "")
        .replace("</pre>", "")
        .replace("<ul>", "")
        .replace("</ul>", "")
        .replace("<li>", "")
        .replace("</li>", "")
        .replace("<code>", "")
        .replace("</code>", "")
        .replace("<i>", "")
        .replace("</i>", "")
        .replace("<sub>", "")
        .replace("</sub>", "")
        .replace("</sup>", "")
        .replace("<sup>", "^")
        .replace("&nbsp;", " ")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&quot;", "\"")
        .replace("&minus;", "-")
        .replace("&#39;", "'")
        .replace("\n\n", "\n")
        .replace('\n', "\n * ")
        .replace('\t', "  ")
}

// pub enum SolutionReturnType {
//     Integer,
//     Double,
//     String,
//     Boolean,
//     NoReturn,
// }

fn insert_return_type(code: &str) -> String {
    let type_re = Regex::new(r"\s+->\s+([a-zA-Z0-9<>_]+)\s+\{[\s*\n*.*]*}").unwrap();

    // println!("Code: {}", code);

    let rtype = type_re
        .captures(code)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string();

    // println!("{}", rtype);

    let sblock_re = Regex::new(r"\{[\s+\n]+}").unwrap();

    // match on rtypes and insert the correct return value

    let syntax_tree = parse_file(code).unwrap();

    // for item in syntax_tree.items {
    //     extract_block_details(item.clone());
    //     if let syn::Item::Fn(item_fn) = item {
    //         if let Some(return_type) = extract_return_type(&item_fn) {
    //             println!("Function: {}", item_fn.sig.ident);
    //         }
    //     }
    // }
    //
    code.to_string()
}

fn extract_block_details(item: Item) {
    if let Item::Impl(imp) = item {
        for item in imp.items {
            if let ImplItem::Fn(item_fn) = item {
                // println!("\nFunction: {:#?}", item_fn);
                let sig = &item_fn.sig;
                let out = &sig.output;
                // println!("Function: {:#?}", out);
                match out {
                    ReturnType::Default => {}
                    ReturnType::Type(_, ty) => {
                        println!("Return Type: {:#?}", ty);

                        match **ty {
                            Type::Path(ref path) => {
                                // println!("Return Type: {:#?}", path);
                                for seg in &path.path.segments {
                                    // println!("Return Type: {:#?}", seg.ident);
                                    match seg.ident.to_string().as_str() {
                                        "Option" => {
                                            println!("Option");
                                        }

                                        "Vec" => {
                                            println!("Vec");
                                        }

                                        _ => {
                                            println!("{}", seg.ident);
                                        }
                                    }
                                }
                            }
                            _ => {
                                todo!()
                            }
                        }

                        // match ty {
                        //     Type::Path(path) => {
                        //         println!("Return Type: {:#?}", path);
                        //     }
                        //     _ => {}
                        // }
                        //
                        // if ty.segments.len() == 1 {
                        //     println!("Return Type: {:#?}", ty.segments[0].ident);
                        // }
                    }
                }
            }
        }
    }
}

fn extract_return_type(item_fn: &ItemFn) -> Option<&Type> {
    match &item_fn.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(ty),
    }
}

fn insert_return_in_code(return_type: &str, code: &str) -> String {
    // let tre = Regex::new(r"([a-zA-Z0-9]+)\s\{[ \n]+}").unwrap();

    let re = Regex::new(r"\{[ \n]+}").unwrap();

    match return_type {
        "ListNode" => re
            .replace(code, "{\n        Some(Box::new(ListNode::new(0)))\n    }")
            .to_string(),
        "ListNode[]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "TreeNode" => re
            .replace(
                code,
                "{\n        Some(Rc::new(RefCell::new(TreeNode::new(0))))\n    }",
            )
            .to_string(),
        "boolean" => re.replace(code, "{\n        false\n    }").to_string(),
        "character" => re.replace(code, "{\n        '0'\n    }").to_string(),
        "character[][]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "double" => re.replace(code, "{\n        0f64\n    }").to_string(),
        "double[]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "int[]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "integer" => re.replace(code, "{\n        0\n    }").to_string(),
        "integer[]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "integer[][]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<String>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<TreeNode>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<boolean>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<double>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<integer>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<list<integer>>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<list<string>>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<string>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "null" => code.to_string(),
        "string" => re
            .replace(code, "{\n        String::new()\n    }")
            .to_string(),
        "string[]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "void" => code.to_string(),
        "NestedInteger" => code.to_string(),
        "Node" => code.to_string(),
        _ => code.to_string(),
    }
}
