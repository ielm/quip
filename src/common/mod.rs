pub mod deal;
pub mod fetch;
pub mod problem;

const PROBLEMS_URL: &str = "https://leetcode.com/api/problems/algorithms/";
const GRAPHQL_URL: &str = "https://leetcode.com/graphql";
const QUESTION_QUERY_STRING: &str = r#"
query questionData($titleSlug: String!) {
    question(titleSlug: $titleSlug) {
        content
        stats
        codeDefinition
        sampleTestCase
        metaData
    }
}"#;
const QUESTION_QUERY_OPERATION: &str = "questionData";
