use node_metrics::Metrics;

fn main() {
    let metrics = Metrics::new();
    metrics.log();
    let json = metrics.to_json().unwrap();
    println!("{}", json);
}
