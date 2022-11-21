use node_metrics::metrics::Metrics;

fn main() {
    let json_str = "{\"cpu\":{\"total\":8,\"free\":28.73024},\"memory\":{\"total\":34054199296,\"free\":27052634112},\"disks\":[{\"disk_name\":\"/dev/nvme0n1p3\",\"total\":496896393216,\"free\":44782456832},{\"disk_name\":\"/dev/nvme0n1p1\",\"total\":824180736,\"free\":765476864},{\"disk_name\":\"overlay\",\"total\":496896393216,\"free\":44782456832},{\"disk_name\":\"overlay\",\"total\":496896393216,\"free\":44782456832},{\"disk_name\":\"overlay\",\"total\":496896393216,\"free\":44782456832},{\"disk_name\":\"overlay\",\"total\":496896393216,\"free\":44782456832},{\"disk_name\":\"overlay\",\"total\":496896393216,\"free\":44782456832},{\"disk_name\":\"overlay\",\"total\":496896393216,\"free\":44782456832}]}";
    let metrics = Metrics::from_json(json_str.to_string());
    println!("{:?}", metrics);
}
