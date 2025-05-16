# Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
# GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

use std::io;
use virt::connect::Connect;
use virt::domain::Domain;
use virt::sys::VIR_DOMAIN_UNDEFINE_NVRAM;
use std::process::Command;
use crate::api::ovs;
use std::fs;
use indoc::indoc;
use std::error::Error;


static LIBVIRT_STORAGE_PATH: &str = "/var/lib/libvirt/images";

pub struct VmDomain {}

impl VmDomain {
    async fn generate_domain_xml(
      name: &str, 
      memory: &u64, 
      cpu: &u32, 
      tenant: &str, 
      mac_addr: &str, 
      arch: &str) -> String {
        
      let mut domain_xml = String::new();
      if arch == "aarch64" {
        domain_xml = format!(r"
        <domain type='kvm'>
          <name>{}-{}</name>
          <memory unit='KiB'>{}</memory>
          <vcpu placement='static'>{}</vcpu>
          <os firmware='efi'>
            <type arch='aarch64' machine='virt-7.2'>hvm</type>
            <firmware>
              <feature enabled='no' name='secure-boot'/>
            </firmware>
            <boot dev='hd'/>
          </os>
          <cpu mode='host-passthrough'/>
          <features>
            <acpi/>
            <gic version='2'/>
          </features>
          <devices>
            <disk type='file' device='disk'>
              <driver name='qemu' type='qcow2'/>
              <source file='{}/{}/{}.qcow2'/>
              <target dev='vda' bus='virtio'/>
              <address type='pci' slot='0x04'/>
            </disk>
            <disk type='file' device='cdrom'>
              <driver name='qemu' type='raw'/>
              <source file='{}/{}/seed.iso' index='1'/>
              <backingStore/>
              <target dev='sda' bus='sata'/>
              <readonly/>
              <alias name='sata0-0-0'/>
              <address type='drive' controller='0' bus='0' target='0' unit='0'/>
            </disk>
            <interface type='ethernet'>
                <mac address='{}'/>
                <target dev='{}-{}'/>
                <model type='virtio'/>
            </interface>
            <console type='pty'>
            <target type='serial' port='0'/>
            </console>
            <rng model='virtio'>
              <backend model='random'>/dev/urandom</backend>
              <alias name='rng0'/>
              <address type='pci' domain='0x0000' bus='0x05' slot='0x00' function='0x0'/>
            </rng>
            <channel type='unix'>
              <target type='virtio' name='org.qemu.guest_agent.0'/>
              <address type='virtio-serial' controller='0' bus='0' port='1'/>
            </channel>
          </devices>
        </domain>
        ", tenant, name, memory * 1024, cpu, LIBVIRT_STORAGE_PATH, name, name, LIBVIRT_STORAGE_PATH, name, mac_addr, tenant, name);
      } else if arch == "x86_64" {
        domain_xml = format!(r"
        <domain type='kvm'>
          <name>{}-{}</name>
          <memory unit='KiB'>{}</memory>
          <vcpu placement='static'>{}</vcpu>
          <os>
            <type arch='x86_64' machine='q35'>hvm</type>
          </os>
          <cpu mode='host-passthrough'/>
          <features>
            <acpi/>
          </features>
          <devices>
            <disk type='file' device='disk'>
              <driver name='qemu' type='qcow2'/>
              <source file='{}/{}/{}.qcow2'/>
              <target dev='vda' bus='virtio'/>
              <address type='pci' slot='0x04'/>
            </disk>
            <disk type='file' device='cdrom'>
              <driver name='qemu' type='raw'/>
              <source file='{}/{}/seed.iso' index='1'/>
              <backingStore/>
              <target dev='sda' bus='sata'/>
              <readonly/>
              <alias name='sata0-0-0'/>
              <address type='drive' controller='0' bus='0' target='0' unit='0'/>
            </disk>
            <interface type='ethernet'>
                <mac address='{}'/>
                <target dev='{}-{}'/>
                <model type='virtio'/>
            </interface>
            <console type='pty'>
            <target type='serial' port='0'/>
            </console>
            <rng model='virtio'>
              <backend model='random'>/dev/urandom</backend>
              <alias name='rng0'/>
              <address type='pci' domain='0x0000' bus='0x05' slot='0x00' function='0x0'/>
            </rng>
            <channel type='unix'>
              <target type='virtio' name='org.qemu.guest_agent.0'/>
              <address type='virtio-serial' controller='0' bus='0' port='1'/>
            </channel>
          </devices>
        </domain>
        ", tenant, name, memory * 1024, cpu, LIBVIRT_STORAGE_PATH, name, name, LIBVIRT_STORAGE_PATH, name, mac_addr, tenant, name);
      }

      return domain_xml;
    }

    pub async fn create_vm(
        name: String, 
        memory: u64, 
        cpu: u32, 
        os: String, 
        pub_key: String,
        disk_size: u32, 
        tenant: String, 
        mac_addr: String, 
        networking: String, 
        network: Option<String>
    ) -> Result<Domain, Box<dyn Error>> {
        let conn: Connect = Connect::open(Some("qemu:///system"))?;

        let arch = std::env::consts::ARCH.to_string();
        let domain_xml = VmDomain::generate_domain_xml(&name, &memory, &cpu, &tenant, &mac_addr, &arch).await;

        VmDomain::create_disk(&os, &name, &disk_size)?;
        VmDomain::generate_seed(&pub_key, &name)?;
        let domain: Domain = Domain::define_xml(&conn, &domain_xml)?;
        domain.create()?;
        domain.set_autostart(true)?;

        if networking == "l2-tenant" {
            VmDomain::create_vm_nic(&name, tenant).await?;
        } else if networking == "l2-bridged" {
          match network {
              Some(network) => {
                  let bridge_name = format!("br-vlan{}", network);
                  VmDomain::add_vnet_to_provider_bridge(&name, &bridge_name, &tenant).await?;
            }
            None => {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Network name is required for l2-bridged networking").into());
            }
          }
        }

        return Ok(domain)
    }

    async fn add_vnet_to_provider_bridge(name: &str, bridge_name: &str, tenant: &str) -> Result<(), io::Error> {
        ovs::OvsDbRequest::add_vnet_to_provider_bridge(name, bridge_name, tenant).await?;
        Ok(())
    }

    async fn create_vm_nic(name: &str, tenant: String) -> Result<(), io::Error> {
        ovs::OvsDbRequest::add_port(name, None, tenant).await?;
        Ok(())
    }

    fn generate_seed(pub_key: &str, name: &str) -> Result<(), io::Error> {
        let user_data = format!(indoc!{ r#"
        #cloud-config
        ssh_authorized_keys:
          - {}
        shell: /bin/bash
        chpasswd:
          list: |
            cloud-user:temppassword123
          expire: False
        "#}, pub_key);

        let meta_data: String = format!(indoc!{r#"
        instance-id: {}
        local-hostname: localhost.localdomain
        "#}, name);

        fs::write(format!("{}/{}/user-data", LIBVIRT_STORAGE_PATH, name), user_data)?;
        fs::write(format!("{}/{}/meta-data", LIBVIRT_STORAGE_PATH, name), meta_data)?;

        let create_seed = Command::new("xorriso")
            .args(&[
                "-as",
                "mkisofs",
                "-output", &format!("{}/{}/seed.iso", LIBVIRT_STORAGE_PATH, name),
                "-volid", "CIDATA",
                "-joliet",
                "-rock",
                &format!("{}/{}/user-data", LIBVIRT_STORAGE_PATH, name),
                &format!("{}/{}/meta-data", LIBVIRT_STORAGE_PATH, name),
            ])
            .output()?;

        if !create_seed.status.success() {
            return Err(io::Error::new(
              io::ErrorKind::Other,
              format!("xorriso failed with: {}", String::from_utf8_lossy(&create_seed.stderr)),
          ));
        }

        for file in vec!["user-data", "meta-data"] {
            let remove_file = fs::remove_file(format!("{}/{}/{}", LIBVIRT_STORAGE_PATH, name, file));
            if remove_file.is_err() {
                println!("Failed to remove user,meta-data: {}", remove_file.unwrap_err());
            }
        }  

        Ok(())
    }
    
    fn create_disk(os: &str, name: &str, size: &u32) -> Result<(), io::Error> {
        fs::create_dir(format!("{}/{}", LIBVIRT_STORAGE_PATH, name))?;
      
        let create_disk = Command::new("qemu-img")
            .args(&[
                "convert",
                "-f", "qcow2",
                "-O", "qcow2",
                &format!("{}/{}-base.qcow2", LIBVIRT_STORAGE_PATH, os.to_uppercase()),
                &format!("{}/{}/{}.qcow2", LIBVIRT_STORAGE_PATH, name, name),
            ])
            .output()?;
    
        if !create_disk.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("qemu-img convert failed: {}", String::from_utf8_lossy(&create_disk.stderr)),
            ));
        }
    
        let resize_disk = Command::new("qemu-img")
            .args(&[
                "resize",
                &format!("{}/{}/{}.qcow2", LIBVIRT_STORAGE_PATH, name, name),
                &format!("{}G", size),
            ])
            .output()?;
    
        if !resize_disk.status.success() {
          return Err(io::Error::new(
              io::ErrorKind::Other,
              format!("qemu-img resize failed: {}", String::from_utf8_lossy(&resize_disk.stderr)),
          ));
        }
    
        Ok(())
    }    

    fn remove_vm_dir(name: &str) -> Result<(), io::Error> {
        let remove_vm_dir = fs::remove_dir_all(
            format!("{}/{}", LIBVIRT_STORAGE_PATH, name)
        );

        if remove_vm_dir.is_err() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to remove VM directory: {}", remove_vm_dir.unwrap_err()),
            ));
        }

        Ok(())
    }

    pub async fn delete_vm(name: String, tenant: String) -> Result<(), virt::error::Error> {
      let conn = Connect::open(Some("qemu:///system"))?;
      let domain_name = format!("{}-{}", tenant, name);

      let domain = Domain::lookup_by_name(&conn, &domain_name)?;
  
      if let Err(e) = VmDomain::remove_vm_dir(&name) {
          eprintln!("Warning: Failed to remove VM directory: {:?}", e);
      }
  
      domain.destroy()?;  
      domain.undefine_flags(VIR_DOMAIN_UNDEFINE_NVRAM)?;
  
      if let Err(e) = ovs::OvsDbRequest::delete_port(name, None, tenant).await {
          eprintln!("Warning: Failed to delete OVS port: {:?}", e);
      }
  
      Ok(())
    }
}
