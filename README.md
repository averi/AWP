# AWP - The Awesome Weekend Project

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

AWP is a private cloud infrastructure project designed to spawn multi-arch Virtual Machines (VMs), configure them using cloud-init, and manage them through an API/web UI. The primary motivation behind AWP was to learn Rust and its ecosystem while building a private cloud that could be run to power an homelab environment.

## Core Technologies

* **Backend:** Rust
    * **Web Framework:** Axum
    * **Virtualization Management:** libvirt (via `virt` crate)
    * **Networking:** OVN/OVS (custom XML-RPC client), rtnetlink
    * **Async Runtime:** Tokio
    * **Serialization:** Serde
    * **Database Interaction:** SQLx
* **Frontend:** ReactJS
    * **Styling:** TailwindCSS
* **Virtualization:** QEMU/KVM, Libvirt
* **Networking:** Open vSwitch (OVS), Open Virtual Network (OVN)
* **Operating Systems:** Primarily developed and tested on Linux (with examples for RHEL-based systems and Raspberry Pi OS)

## MVP & Key Features

AWP has reached its initial Minimal Viable Product (MVP) stage. Key features include:

1.  **Multi-Arch VM Management:** Schedule VMs of different architectures via an API and the web UI.
2.  **OS Selection:** Support for deploying multiple different operating systems.
3.  **Cloud-Init Integration:** VMs boot using cloud-init for automated setup (default password, networking).
4.  **Flexible Networking:**
    * Internet connectivity with trunked VLANs and provider networks.
    * L2 network configuration for VM-to-VM communication.
    * IP address assignment for VMs on flat networks via OVN's DHCP.
5.  **Web UI:** Display VM specifications, status, and other tenant components. Schedule new VMs and perform actions.
6.  **Metrics & Scheduling:** A hypervisor agent collects metrics for basic scheduling logic.
7.  **Deployment:** Designed with Ansible in mind for easier deployment and testing.

## Setup Instructions

The full preliminary setup steps, including package installation and configuration for the control plane and hypervisors (Raspberry Pi 5 and x86_64 systems), are detailed in the [introductory blog post](https://www.dragonsreach.it/2025/05/17/awp-the-awesome-weekend-project#preliminary-steps).

## Contributing

Feel free to open an issue for discussions, bug reports, or feature requests! Pull requests are welcome.
