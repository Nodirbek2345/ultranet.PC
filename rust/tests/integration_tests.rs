use rust_lib_ultranet::plugins::dns::run_dns_benchmark;
use rust_lib_ultranet::plugins::ping::run_multi_ping;
use rust_lib_ultranet::db::init_db;

#[tokio::test]
async fn test_dns_benchmark_returns_results() {
    let results = run_dns_benchmark().await;
    assert!(!results.is_empty(), "DNS benchmark should return results");
    assert!(results.iter().any(|r| r.target == "1.1.1.1"), "Should benchmark 1.1.1.1");
}

#[tokio::test]
async fn test_ping_engine_works() {
    let pings = run_multi_ping(vec!["1.1.1.1".to_string()], 2).await;
    assert_eq!(pings.len(), 1, "Should return 1 ping result");
    assert!(pings[0].average_ms > 0.0, "Ping should be greater than 0");
}

#[test]
fn test_database_initialization() {
    let init_result = init_db();
    assert!(init_result.is_ok(), "Database should initialize without errors");
}
