MASTER_COUNT = 1
WORKER_COUNT = 2
IMAGE = "ubuntu/focal64"

$install_riklet = <<-SCRIPT
. /etc/os-release
echo "deb https://download.opensuse.org/repositories/devel:/kubic:/libcontainers:/stable/xUbuntu_${VERSION_ID}/ /" | sudo tee /etc/apt/sources.list.d/devel:kubic:libcontainers:stable.list
curl -L https://download.opensuse.org/repositories/devel:/kubic:/libcontainers:/stable/xUbuntu_${VERSION_ID}/Release.key | sudo apt-key add -
apt-get update
apt-get -y upgrade
apt-get install -y runc skopeo
mv /tmp/umoci /usr/local/bin/umoci
chmod +x /usr/local/bin/umoci
dpkg -i /tmp/riklet.deb
echo "ARG1=--master-ip 10.0.0.11:4995" >> /tmp/.rikletconf
echo "ARG2=-v" >> /tmp/.rikletconf
systemctl start riklet.service
SCRIPT

$install_master = <<-SCRIPT
dpkg -i /tmp/scheduler.deb && dpkg -i /tmp/controller.deb

systemctl start rik-scheduler.service
systemctl start rik-controller.service
SCRIPT

Vagrant.configure("2") do |config|

  (1..MASTER_COUNT).each do |i|
    vm_name = "master-#{i}-rik"
    config.vm.define vm_name do |rikmasters|
      rikmasters.vm.box = IMAGE
      rikmasters.vm.hostname = vm_name
      rikmasters.vm.network  :private_network, ip: "10.0.0.#{i+10}"
      rikmasters.vm.provider :virtualbox do |vb|
        vb.name = vm_name
        vb.memory = 1024
        vb.cpus = 2
      end
      rikmasters.vm.provision :file, source: "./target/debian/rik-scheduler_1.0.0_amd64.deb", destination: "/tmp/scheduler.deb"
      rikmasters.vm.provision :file, source: "./target/debian/controller_1.0.0_amd64.deb", destination: "/tmp/controller.deb"
      rikmasters.vm.synced_folder ".", "/vagrant", type: "rsync", rsync__exclude: "target/"
      rikmasters.vm.provision :shell, inline: $install_master
    end
  end

  (1..WORKER_COUNT).each do |i|
    vm_name = "worker-#{i}-rik"
    config.vm.define vm_name do |riknodes|
      riknodes.vm.box = IMAGE
      riknodes.vm.hostname = vm_name
      riknodes.vm.network  :private_network, ip: "10.0.0.#{i+20}"
      riknodes.vm.provider :virtualbox do |vb|
        vb.name = vm_name
        vb.memory = 512
        vb.cpus = 1
      end
      riknodes.vm.provision :file, source: "./target/debian/riklet_1.0.0_amd64.deb", destination: "/tmp/riklet.deb"
      riknodes.vm.provision :file, source: "./riklet/fixtures/umoci.amd64", destination: "/tmp/umoci"
      riknodes.vm.provision :shell, inline: $install_riklet
      riknodes.vm.synced_folder ".", "/vagrant", type: "rsync", rsync__exclude: "target/"
    end
  end
end
