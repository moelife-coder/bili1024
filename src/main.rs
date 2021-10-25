use std::io::{Read, Write};
#[macro_use]
extern crate log;

fn main() {
    pretty_env_logger::init();
    let mut parsed: Vec<String> = std::fs::read_to_string("list_of_ip")
        .unwrap()
        .split('\n')
        .map(|x| x.trim_matches('"').to_string())
        .collect();
    let mut total_sum = parsed.len();
    let args = std::env::args().collect::<Vec<_>>();
    let mut list = args[1]
        .split(',')
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let client = isahc::HttpClientBuilder::new()
        .default_headers(&[
            (
                isahc::http::header::COOKIE.as_str(),
                format!("SESSDATA={}", std::env::var("SESSDATA").unwrap()).as_str(),
            ),
            (
                isahc::http::header::CONTENT_TYPE.as_str(),
                r#"application/json;charset=utf-8"#,
            ),
        ])
        .build()
        .unwrap();
    let mut first_trial = String::new();
    client
        .post(
            "https://security.bilibili.com/sec1024/api/v1/submit_flag",
            format!(r#"{{"flag":"{}","qid":"7"}}"#, list.join(",")),
        )
        .unwrap()
        .into_body()
        .read_to_string(&mut first_trial)
        .unwrap();
    if !first_trial.contains("再接再厉") {
        panic!("输入有问题! {}", first_trial);
    }
    let started = std::time::Instant::now();
    let mut history = std::fs::OpenOptions::new().create(true).write(true).append(true).open("history").unwrap();
    let prev_history: std::collections::HashSet<String> = std::fs::read_to_string("history").unwrap().split("\n").map(|x| x.to_string()).collect();
    while list.len() < 28 && parsed.len() > 0 {
        list.push(parsed.remove(0));
        if prev_history.get(&list[list.len() - 1]).is_some() {
            list.remove(list.len() - 1);
            total_sum -= 1;
            continue;
        }
        let mut result = String::new();
        client
            .post(
                "https://security.bilibili.com/sec1024/api/v1/submit_flag",
                format!(r#"{{"flag":"{}","qid":"7"}}"#, list.join(",")),
            )
            .unwrap()
            .into_body()
            .read_to_string(&mut result)
            .unwrap();
        info!(
            "{}% ({} / {} completed), missing {}, ETA {:.2}s",
            (total_sum - parsed.len() - 1) as f64 / total_sum as f64 * 100.0,
            (total_sum - parsed.len() - 1),
            total_sum,
            28 - list.len() as i128,
            (started.elapsed().as_secs_f64() * parsed.len() as f64 / (total_sum - parsed.len() - 1) as f64)
        );
        if result.contains("再接再厉") {
            info!("YESSSS! {} out of 28", list.len());
            warn!("Updated output: {}", list.join(","));
        } else if result.contains("您提交的答案不正确") {
            info!("Failed {}", list[list.len() - 1]);
            history.write_all(format!("{}\n", list.remove(list.len() - 1)).as_bytes()).unwrap();
            history.flush().unwrap();
        } else if result.contains("频繁") | result.contains("拒绝") {
            info!("Access denied. Sleeping");
            std::thread::sleep(std::time::Duration::from_secs(10));
            parsed.push(list.remove(list.len() - 1));
        } else {
            error!("Unknown result: {}", result);
            std::thread::sleep(std::time::Duration::from_secs(5));
            parsed.push(list.remove(list.len() - 1));
        }
        std::thread::sleep(std::time::Duration::from_secs_f32(0.25));
    }
    println!("Output: {}", list.join(","));
}
