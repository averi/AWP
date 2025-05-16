# Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
# GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

use sqlx::postgres::PgPoolOptions;
use sqlx::types::{Uuid,ipnetwork::IpNetwork};
use crate::api::{Port, Tenant, Vpc, SSHKey, HypervisorScheduler, VirtualMachine, ProviderNetwork};
use std::env;
use std::path::Path;

pub struct Database {}

#[derive(serde::Deserialize, Debug)]
struct Config {
    controlplane: ControlPlaneAPI,
}

#[derive(serde::Deserialize, Debug)]
struct ControlPlaneAPI {
    db_port: u16,
    db_host: String,
    db_name: String,
    db_user: String,
    db_password: String,
}


fn read_conf_file(config_file: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string(config_file)?;
    let config: Config = serde_yaml::from_str(&file)?;
    Ok(config)
}

impl Database {
    pub async fn new() -> Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> {
        let config_path = env::current_dir()?.join("config.yaml");

        println!("Config path: {}", config_path.display());

        let config = read_conf_file(&config_path).unwrap();
        let db_url = format!("postgres://{}:{}@{}:{}/{}", config.controlplane.db_user,
            config.controlplane.db_password, config.controlplane.db_host,
            config.controlplane.db_port, config.controlplane.db_name);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;


        Ok(pool)
    }

    pub async fn create_tenant(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("INSERT INTO tenants (name) VALUES ($1)", name)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn delete_tenant(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        id: &Uuid
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM tenants where id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_tenant_by_name(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let row = sqlx::query!("SELECT * FROM tenants where name = $1", name)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|r| r.id))
    }

    pub async fn get_tenant_by_id(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        id: &Uuid
    ) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query!("SELECT * FROM tenants where id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|r| r.name))
    }

    pub async fn create_vpc(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str, 
        cidr: &str,
        nat: &bool,
        tenant: &Uuid
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("INSERT INTO vpcs (name, cidr, nat, tenant) VALUES ($1, $2, $3, $4)", name, cidr, nat, tenant)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn delete_vpc(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        id: &Uuid
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM vpcs where id = $1", id)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn list_ports(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        vpc: &Uuid
    ) -> Result<Vec<Port>, sqlx::Error> {
        let ports = sqlx::query_as!(Port, "SELECT * FROM ports WHERE vpc = $1", vpc)
            .fetch_all(pool)
            .await?;
        Ok(ports)
    }

    pub async fn list_tenants(
        pool: &sqlx::Pool<sqlx::Postgres>
    ) -> Result<Vec<Tenant>, sqlx::Error> {
        let tenants = sqlx::query_as!(Tenant, "SELECT * FROM tenants")
            .fetch_all(pool)
            .await?;
        Ok(tenants)
    }

    pub async fn list_vpcs_by_tenantid(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        tenant: &Uuid
    ) -> Result<Vec<Vpc>, sqlx::Error> {
        let vpcs = sqlx::query_as!(Vpc, "SELECT * FROM vpcs where tenant = $1", tenant)
            .fetch_all(pool)
            .await?;
        Ok(vpcs)
    }

    pub async fn list_vpcs_by_tenantname(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        tenant: &str
    ) -> Result<Vec<Vpc>, sqlx::Error> {
        let tenantname = sqlx::query!("SELECT id FROM tenants where name = $1", tenant)
            .fetch_one(pool)
            .await?;
        let tenant_id = tenantname.id;

        let vpcs = sqlx::query_as!(Vpc, "SELECT * FROM vpcs where tenant = $1", &tenant_id)
            .fetch_all(pool)
            .await?;
        Ok(vpcs)
    }

    pub async fn get_vpc_object(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str
    ) -> Result<Option<Vpc>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Vpc>("SELECT * FROM vpcs WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_vpc_cidr(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str,
        tenant: &Uuid
    ) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query!("SELECT cidr FROM vpcs where name = $1 AND tenant = $2", name, tenant)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|r| r.cidr))
    }

    pub async fn hypervisor_register(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        hostname: &str, 
        total_ram: &i32, 
        total_cpu: &i32, 
        used_memory: i32, 
        used_cpu: i32,
        arch: &str, 
        hosted_vms: i32
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO hypervisors (hostname, total_ram, total_cpu, used_ram, used_cpu, arch, hosted_vms) VALUES ($1, $2, $3, $4, $5, $6, $7)", 
            hostname, 
            total_ram, 
            total_cpu, 
            used_memory, 
            used_cpu, 
            arch, 
            hosted_vms)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn get_hypervisor_by_hostname(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        hostname: &str
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let row = sqlx::query!("SELECT * FROM hypervisors where hostname = $1", hostname)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|r| r.id))
    }

    pub async fn get_hypervisor_by_id(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        id: &Uuid
    ) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query!("SELECT * FROM hypervisors where id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|r| r.hostname))
    }

    pub async fn update_hypervisor(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        id: &Uuid, 
        used_memory: &i32, 
        used_cpu: &i32, 
        hosted_vms: i32
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE hypervisors SET used_ram = $1, used_cpu = $2, hosted_vms = $3 WHERE id = $4", 
            used_memory, 
            used_cpu, 
            hosted_vms, 
            id)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn get_vpc_by_name(
        pool: &sqlx::Pool<sqlx::Postgres>,
        name: &str,
        tenant: &Uuid
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let row = sqlx::query!("SELECT id FROM vpcs where name = $1 AND tenant = $2", name, tenant)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|r| r.id))
    }

    pub async fn get_vpc_by_id(
        pool: &sqlx::Pool<sqlx::Postgres>,
        id: &Uuid,
        tenant: &Uuid
    ) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query!("SELECT name FROM vpcs where id = $1 AND tenant = $2", id, tenant)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|r| r.name))
    }

    pub async fn create_ssh_pub_key(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str, 
        ssh_pub_key: &str,
        fingerprint: &str,
        tenant: &Uuid
        ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO ssh_pub_keys (name, ssh_pub_key, fingerprint, tenant) VALUES ($1, $2, $3, $4)", 
            name, ssh_pub_key, fingerprint, tenant)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn delete_ssh_pub_key(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM ssh_pub_keys where name = $1", name)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn list_ssh_pub_keys(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        id: &Uuid
    ) -> Result<Vec<SSHKey>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SSHKey>("SELECT * FROM ssh_pub_keys WHERE tenant = $1")
        .bind(id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_ssh_key(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let row = sqlx::query!("SELECT id FROM ssh_pub_keys where name = $1", name)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|r| r.id))
    }

    pub async fn list_hypervisors(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<Vec<HypervisorScheduler>, sqlx::Error> {
        let rows = sqlx::query_as::<_, HypervisorScheduler>("SELECT * FROM hypervisors")
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_hypervisors_min_hosted_vms(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        arch: &str
    ) -> Result<Vec<HypervisorScheduler>, sqlx::Error> {
        let rows = sqlx::query_as::<_, HypervisorScheduler>("SELECT * FROM hypervisors WHERE arch = $1 AND hosted_vms = (SELECT MIN(hosted_vms) FROM hypervisors WHERE arch = $1);")
        .bind(arch)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn create_virtual_machine(
        pool: &sqlx::Pool<sqlx::Postgres>,
        name: &str,
        cpu: &i32,
        ram: &i32,
        tenant: &Uuid,
        vpc: &Uuid,
        ssh_pub_key: &Uuid,
        disk_size: &i32,
        hypervisor: &Uuid,
        os: &str,
        state: &str,
        networking: &str,
        network: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO vms (name, cpu, ram, tenant, vpc, ssh_pub_key, disk_size, hypervisor, os, state, networking, network) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            name,
            cpu,
            ram,
            tenant,
            vpc,
            ssh_pub_key,
            disk_size,
            hypervisor,
            os,
            state,
            networking,
            network
        )
        .execute(pool)
        .await?;
    
        Ok(())
    }

    pub async fn get_virtual_machine_by_name(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str, 
        tenant: &Uuid
    ) -> Result<Option<VirtualMachine>, sqlx::Error> {
        let row = sqlx::query_as::<_, VirtualMachine>(
            "SELECT * FROM vms where name = $1 AND tenant = $2")
            .bind(name)
            .bind(tenant)
            .fetch_optional(pool)
            .await?;

        Ok(row)
    }

    pub async fn get_virtual_machine_by_tenant(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        tenant: &Uuid
    ) -> Result<Option<VirtualMachine>, sqlx::Error> {
        let row = sqlx::query_as::<_, VirtualMachine>(
            "SELECT * FROM vms where tenant = $1")
            .bind(tenant)
            .fetch_optional(pool)
            .await?;

        Ok(row)
    }

    pub async fn delete_virtual_machine(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM vms where name = $1", name)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn list_virtual_machines(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        tenant: &Uuid
    ) -> Result<Vec<VirtualMachine>, sqlx::Error> {
        let rows = sqlx::query_as::<_, VirtualMachine>("SELECT * FROM vms WHERE tenant = $1")
        .bind(tenant)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn update_vm_state(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str, 
        tenant: &Uuid, 
        state: &str
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE vms SET state = $1 WHERE name = $2 AND tenant = $3", state, name, tenant)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn get_provider_network(
        pool: &sqlx::Pool<sqlx::Postgres>,
        name: &str
    ) -> Result<Option<ProviderNetwork>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ProviderNetwork>(
        "SELECT * FROM provider_networks where NAME = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_provider_networks(
        pool: &sqlx::Pool<sqlx::Postgres>
    ) -> Result<Vec<ProviderNetwork>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ProviderNetwork>("SELECT * FROM provider_networks")
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn create_provider_network(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str,
        vlan: &i32, 
        subnet: &str
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("INSERT INTO provider_networks (name, vlan, subnet) VALUES ($1, $2, $3)", name, vlan, subnet)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn delete_provider_network(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM provider_networks where name = $1", name)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn update_vm_ip_addr(
        pool: &sqlx::Pool<sqlx::Postgres>, 
        name: &str, 
        tenant: &Uuid, 
        ip_addresses: &[IpNetwork]
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE vms SET ip_addresses = $1 WHERE name = $2 AND tenant = $3", ip_addresses, name, tenant)
            .execute(pool)
            .await?;
    
        Ok(())
    }
}