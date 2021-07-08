use node_metrics::metrics_manager::MetricsManager;

fn main() {
    let mut metrics_manager = MetricsManager::new();
    let metrics = metrics_manager.fetch();
    let json = metrics.to_json().unwrap();
    println!("{}", json);
}
