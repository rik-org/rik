output "rikconfig" {
  value = <<YAML
cluster:
  name: ${local.cluster_name}
  server: http://${google_compute_address.master_static.address}:5000/
  YAML
}