locals {
  username = "rik"
}

# Create the private key which will be
# used to access our cluster nodes
resource "tls_private_key" "ssh" {
  algorithm = "RSA"
  rsa_bits  = 8192
}

# Creates the compute network
resource "google_compute_network" "rik" {
  name                    = "rik-vpc"
  auto_create_subnetworks = true
}

# Create the master static ip
resource "google_compute_address" "master_static" {
  name = "rik-master"
}

# Create the worker static ip
resource "google_compute_address" "worker_static" {
  count = var.workers_count
  name  = "rik-worker-${count.index}"
}

# Authorize SSH
resource "google_compute_firewall" "ssh" {
  name    = "rik-allow-ssh"
  network = google_compute_network.rik.name

  allow {
    protocol = "tcp"
    ports    = ["22"]
  }

  priority      = 65534
  source_ranges = ["0.0.0.0/0"]
}

# Authorize external access to the RIK API
resource "google_compute_firewall" "api_server" {
  name    = "rik-allow-api-server"
  network = google_compute_network.rik.name

  allow {
    protocol = "tcp"
    ports    = ["5000"]
  }

  priority      = 1000
  source_ranges = ["0.0.0.0/0"]
}

# Authorize external access to the RIK API
resource "google_compute_firewall" "workers" {
  name    = "rik-allow-internal-workers"
  network = google_compute_network.rik.name

  allow {
    protocol = "tcp"
    ports    = ["4995"]
  }

  priority      = 1000
  source_ranges = [for i in range(var.workers_count): "${google_compute_address.worker_static[i].address}/32"]
  source_tags = []
}

# Creates the rik master node
resource "google_compute_instance" "master" {
  name         = "rik-master"
  machine_type = "e2-micro"
  zone         = "europe-west1-b"

  metadata = {
    ssh-keys = "${local.username}:${tls_private_key.ssh.public_key_openssh}"
  }

  network_interface {
    network = google_compute_network.rik.name
    access_config {
      nat_ip = google_compute_address.master_static.address
    }
  }

  boot_disk {
    initialize_params {
      image = "debian-cloud/debian-11"
    }
  }

  connection {
    type        = "ssh"
    user        = "rik"
    host        = google_compute_address.master_static.address
    private_key = tls_private_key.ssh.private_key_pem
  }

  provisioner "file" {
    source      = "utils/controller.deb"
    destination = "/tmp/controller.deb"
  }

  provisioner "file" {
    source      = "utils/scheduler.deb"
    destination = "/tmp/scheduler.deb"
  }

  provisioner "remote-exec" {
    inline = [
      "sudo dpkg -i /tmp/scheduler.deb",
      "sudo dpkg -i /tmp/controller.deb",
      "sudo systemctl start scheduler.service",
      "sudo systemctl start rik-controller.service"
    ]
  }
}

# Creates the workers instances
resource "google_compute_instance" "worker" {
  count        = var.workers_count
  name         = "rik-worker-${count.index}"
  machine_type = "e2-micro"
  zone         = "europe-west1-b"
  metadata = {
    ssh-keys = "${local.username}:${tls_private_key.ssh.public_key_openssh}"
  }

  network_interface {
    network = google_compute_network.rik.name
    access_config {
      nat_ip = google_compute_address.worker_static[count.index].address
    }
  }

  boot_disk {
    initialize_params {
      image = "debian-cloud/debian-11"
    }
  }

  connection {
    type        = "ssh"
    user        = "rik"
    host        = google_compute_address.worker_static[count.index].address
    private_key = tls_private_key.ssh.private_key_pem
  }

  provisioner "file" {
    source      = "utils/riklet.deb"
    destination = "/tmp/riklet.deb"
  }

  provisioner "remote-exec" {
    inline = [
      "sudo apt-get update",
      "sudo apt-get install -y runc skopeo umoci",
      "sudo dpkg -i /tmp/riklet.deb",
      "echo 'ARG1=--master-ip ${google_compute_address.master_static.address}:4995' >> /tmp/.rikletconf",
      "echo 'ARG2=-v' >> /tmp/.rikletconf",
      "sudo systemctl start riklet.service"
    ]
  }
}
