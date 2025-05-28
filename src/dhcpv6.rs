// SPDX-License-Identifier: Apache-2.0

use mozim::{DhcpV6Client, DhcpV6Config, DhcpV6IaType, DhcpV6Lease};

use crate::IFNAME;

pub(crate) const ARG_IS_PD: &str = "pd";
pub(crate) const ARG_IS_TA: &str = "temp";
pub(crate) const ARG_DISPATCH: &str = "dispatch";

const DEFAULT_TIMEOUT: u32 = 480;
const POLL_WAIT_TIME: u32 = 5;

pub(crate) fn run_dhcpv6_cli(matches: &clap::ArgMatches) {
    let iface_name: &String = matches.get_one(IFNAME).unwrap();
    let ia_type = if matches.get_flag(ARG_IS_PD) {
        DhcpV6IaType::PrefixDelegation
    } else if matches.get_flag(ARG_IS_TA) {
        purge_dhcpv6_addr(iface_name.as_str());
        DhcpV6IaType::TemporaryAddresses
    } else {
        purge_dhcpv6_addr(iface_name.as_str());
        DhcpV6IaType::NonTemporaryAddresses
    };
    println!("HAHA {:?}", ia_type);

    let dispatch_path: String = matches
        .get_one(ARG_DISPATCH)
        .map(|s: &String| s.to_string())
        .unwrap_or_default();

    let mut config = DhcpV6Config::new(iface_name, ia_type);
    config.set_timeout(DEFAULT_TIMEOUT);
    let mut cli = DhcpV6Client::init(config, None).unwrap();
    loop {
        for event in cli.poll(POLL_WAIT_TIME).unwrap() {
            match cli.process(event) {
                Ok(Some(lease)) => {
                    log::info!("Got DHCPv6 lease {:?}", lease);
                    apply_lease(iface_name, lease, dispatch_path.as_str());
                }
                Ok(None) => (),
                Err(e) => log::error!("Got error {}", e),
            }
        }
    }
}

fn purge_dhcpv6_addr(iface_name: &str) {
    // TODO
}

fn apply_lease(iface_name: &str, lease: DhcpV6Lease, dispatch_path: &str) {
    match lease.ia_type {
        DhcpV6IaType::PrefixDelegation => {
            std::process::Command::new(dispatch_path)
                .arg(iface_name)
                .arg(lease.addr.to_string())
                .arg(format!("{}", lease.prefix_len))
                .arg(format!("{}", lease.preferred_life))
                .arg(format!("{}", lease.valid_life))
                .status()
                .expect(
                    format!("Command {} failed to start", dispatch_path)
                        .as_str(),
                );
        }
        DhcpV6IaType::TemporaryAddresses => {
            if !dispatch_path.is_empty() {
                std::process::Command::new(dispatch_path)
                    .arg(iface_name)
                    .arg(lease.addr.to_string())
                    .arg(format!("{}", lease.prefix_len))
                    .arg(format!("{}", lease.preferred_life))
                    .arg(format!("{}", lease.valid_life))
                    .status()
                    .expect(
                        format!("Command {} failed to start", dispatch_path)
                            .as_str(),
                    );
            }
        }
        DhcpV6IaType::NonTemporaryAddresses => {
            if !dispatch_path.is_empty() {
                std::process::Command::new(dispatch_path)
                    .arg(iface_name)
                    .arg(lease.addr.to_string())
                    .arg(format!("{}", lease.prefix_len))
                    .arg(format!("{}", lease.preferred_life))
                    .arg(format!("{}", lease.valid_life))
                    .status()
                    .expect(
                        format!("Command {} failed to start", dispatch_path)
                            .as_str(),
                    );
            }
        }
        _ => {
            log::error!(
                "BUG: apply_lease() unsupported lease IA type {}: {:?} ",
                lease.ia_type, lease
            );
        }
    }
}
