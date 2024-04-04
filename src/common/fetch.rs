use regex::Regex;
use serde_json::Value;
use std::fmt::Error;
use std::fs;

use crate::common::problem::{Problem, Query, RawProblem, UserProblems};
use crate::common::{GRAPHQL_URL, PROBLEMS_URL};

use super::problem::StatWithStatus;

pub fn get_initialized_problems() -> Vec<u32> {
    let content = fs::read_to_string("./src/problem/mod.rs").unwrap();
    let id_pattern = Regex::new(r"p(\d{4})_").unwrap();
    id_pattern
        .captures_iter(&content)
        .map(|x| x.get(1).unwrap().as_str().parse().unwrap())
        .collect()
}

async fn init_client() -> Result<(reqwest::Client, reqwest::header::HeaderMap), Error> {
    let client = reqwest::Client::builder()
        .build()
        .expect("Failed to build client");
    let mut headers = reqwest::header::HeaderMap::new();
    let cookie = match std::env::var("LEETCODE_COOKIE") {
        Ok(val) => val,
        Err(_) => "".to_string(),
    };
    headers.insert(
        "Cookie",
        reqwest::header::HeaderValue::from_str(&cookie).unwrap(),
    );
    headers.insert("Content-Type", "application/json".parse().unwrap());
    Ok((client, headers))
}

pub async fn get_problem(_question_id: u32) -> Option<Problem> {
    let problems = get_user_problems().await.unwrap();

    for problem_stat in problems.stat_status_pairs.iter() {
        if problem_stat.stat.frontend_question_id == _question_id {
            // return Some(problem.stat.clone());
            if let Some(problem) = get_problem_request(problem_stat).await.unwrap() {
                return Some(problem);
            }
        }
    }
    None
}

async fn get_problem_request(
    problem: &StatWithStatus,
) -> Result<Option<Problem>, Box<dyn std::error::Error>> {
    let (client, headers) = init_client().await?;

    let resp: RawProblem = client
        .post(GRAPHQL_URL)
        .headers(headers)
        .json(&Query::question_query(
            problem.stat.question_title_slug.as_ref().unwrap(),
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    Ok(Some(Problem {
        title: problem.stat.question_title.clone().unwrap(),
        title_slug: problem.stat.question_title_slug.clone().unwrap(),
        code_definition: serde_json::from_str(&resp.data.question.code_definition).unwrap(),
        content: resp.data.question.content,
        sample_test_case: resp.data.question.sample_test_case,
        difficulty: problem.difficulty.to_string(),
        question_id: problem.stat.frontend_question_id,
        return_type: {
            let v: Value = serde_json::from_str(&resp.data.question.meta_data).unwrap();
            v["returnType"].to_string().replace('\"', "")
        },
    }))
}

pub async fn get_user_problems() -> Option<UserProblems> {
    // let res = reqwest::get(PROBLEMS_URL).await.unwrap();
    let res = get_problems_request().await.unwrap();

    let problems = serde_json::from_str::<UserProblems>(&res).unwrap();
    // println!("{:?}", problems);

    Some(problems)
}

async fn get_problems_request() -> Result<String, Box<dyn std::error::Error>> {
    let (client, headers) = init_client().await?;

    let request = client
        .request(reqwest::Method::GET, PROBLEMS_URL)
        .headers(headers);

    let response = request.send().await?;
    let body = response.text().await?;

    Ok(body)
}
