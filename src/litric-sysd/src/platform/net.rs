//! Network facts: interfaces, addresses, routes, DNS.
//!
//! # What this reads, and what it deliberately does not
//!
//! `systemd-networkd` is the manager and `iwd` owns Wi-Fi (ADR-0023 decision 2).
//! Neither is spoken to here: this module only reports what the *kernel* says,
//! which is the same information both of them act on, and which is available
//! whether or not either daemon is running.
//!
//! Three surfaces, each chosen because it is the kernel's own:
//!
//! | Fact | Surface | Why this one |
//! |---|---|---|
//! | interface state, MAC, MTU | `/sys/class/net/<if>/` | one file per attribute, no parsing |
//! | byte counters | `/proc/net/dev` | the only place counters are published |
//! | IPv4 routes | `/proc/net/route` | the routing table, in fixed fields |
//! | IPv6 addresses | `/proc/net/if_inet6` | fixed-width hex, one address per line |
//! | IPv4 addresses | `ip -o -4 addr show` | see below |
//! | DNS servers | `/run/systemd/resolve/resolv.conf` | what `resolved` actually uses |
//!
//! ## The one command, and why an address needs one
//!
//! The kernel publishes *routes* as text and *addresses* only through netlink.
//! Reading netlink means a raw socket, which means `unsafe`, which this crate
//! forbids outright — so the choice is between a documented external command and
//! no IPv4 address at all. `iproute2` is a hard dependency of `systemd` on this
//! distribution, so `ip` is present in every profile including `nano`, and the
//! reader below runs it with a fixed argument vector and no shell. If it is
//! genuinely absent the reply says `net.addr.v4=none` rather than reporting an
//! empty address list as if the machine had none.
//!
//! ## Not read here
//!
//! IPv6 *routes* (`/proc/net/ipv6_route` is a fixed-width hex dump whose field
//! set differs between kernels, and guessing it produces plausible wrong
//! addresses rather than an error), and any Wi-Fi information at all: scanning
//! needs `iwd`'s D-Bus API, which is Phase 2 (`docs/design/wave1/00-network.md`
//! §2). The Settings pane shows known networks and says why there is no list.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use super::{read_attr, read_dir, read_file};

/// One network interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Iface {
    /// Kernel name, e.g. `eth0`.
    pub name: String,
    /// `up`, `down`, or `unknown` — from `operstate`, not `flags`.
    pub operstate: String,
    /// Whether the link has carrier. `None` when the file was unreadable,
    /// which is different from `Some(false)` ("nothing is plugged in").
    pub carrier: Option<bool>,
    /// MAC address, lower-case.
    pub mac: Option<String>,
    /// MTU in bytes.
    pub mtu: Option<u64>,
    /// Link speed in Mb/s, when the driver publishes one (it does not for
    /// virtual interfaces, which is not an error).
    pub speed_mbps: Option<i64>,
    /// Bytes received.
    pub rx_bytes: u64,
    /// Bytes sent.
    pub tx_bytes: u64,
}

/// One address on one interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addr {
    /// Interface name.
    pub name: String,
    /// 4 or 6.
    pub family: u8,
    /// The address, without a prefix length.
    pub addr: String,
    /// Prefix length in bits.
    pub prefix: u8,
    /// `global`, `link`, `host`, `site`, or `unknown`.
    pub scope: String,
}

/// One route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    /// Destination in CIDR form, e.g. `0.0.0.0/0`.
    pub destination: String,
    /// Next hop, or `-` for a directly connected route.
    pub via: String,
    /// Interface name.
    pub dev: String,
    /// Route metric.
    pub metric: u64,
}

/// What `resolved` says about DNS.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dns {
    /// Nameservers, in file order.
    pub nameservers: Vec<String>,
    /// Search domains.
    pub search: Vec<String>,
    /// Whether the file was there at all. A machine without `resolved` running
    /// has unknown DNS, not no DNS, and the reply says which.
    pub source_present: bool,
}

/// Everything this module reports about the network.
#[derive(Debug, Clone, Default)]
pub struct NetFacts {
    /// Interfaces, sorted by name.
    pub ifaces: Vec<Iface>,
    /// Addresses, in the order the kernel listed them per interface.
    pub addrs: Vec<Addr>,
    /// IPv4 routes.
    pub routes: Vec<Route>,
    /// DNS state.
    pub dns: Dns,
    /// Whether the IPv4 address reader ran. `Some(false)` means `ip` was not
    /// found, which is a fact about the image and is reported as one.
    pub ip_available: Option<bool>,
}

impl NetFacts {
    /// The facts as `key=value` lines.
    #[must_use]
    pub fn to_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        out.push(format!("net.iface.count={}", self.ifaces.len()));
        for i in &self.ifaces {
            out.push(format!(
                "net.iface {} state={} carrier={} mac={} mtu={} rx={} tx={}",
                i.name,
                i.operstate,
                i.carrier
                    .map_or_else(|| "-".to_owned(), |c| u8::from(c).to_string()),
                i.mac.as_deref().unwrap_or("-"),
                i.mtu.map_or_else(|| "-".to_owned(), |m| m.to_string()),
                i.rx_bytes,
                i.tx_bytes
            ));
        }
        for a in &self.addrs {
            out.push(format!(
                "net.addr {} family={} addr={} prefix={} scope={}",
                a.name, a.family, a.addr, a.prefix, a.scope
            ));
        }
        for r in &self.routes {
            out.push(format!(
                "net.route destination={} via={} dev={} metric={}",
                r.destination, r.via, r.dev, r.metric
            ));
        }
        for ns in &self.dns.nameservers {
            out.push(format!("net.dns nameserver={ns}"));
        }
        for d in &self.dns.search {
            out.push(format!("net.dns search={d}"));
        }
        out.push(format!(
            "net.dns.source={}",
            if self.dns.source_present {
                "resolved"
            } else {
                "none"
            }
        ));
        out.push(format!(
            "net.addr.v4={}",
            match self.ip_available {
                Some(true) => "iproute2",
                Some(false) => "none",
                None => "unknown",
            }
        ));
        out
    }
}

/// Read everything from a filesystem root.
#[must_use]
pub fn read(root: &Path) -> NetFacts {
    let counters = read_file(root, "proc/net/dev")
        .as_deref()
        .map_or_else(BTreeMap::new, |t| parse_net_dev(t).into_iter().collect());

    let mut ifaces = Vec::new();
    for name in read_dir(root, "sys/class/net") {
        let base = format!("sys/class/net/{name}");
        let (rx_bytes, tx_bytes) = counters.get(&name).copied().unwrap_or((0, 0));
        ifaces.push(Iface {
            operstate: read_attr(root, &format!("{base}/operstate"))
                .as_deref()
                .map_or_else(|| "unknown".to_owned(), parse_operstate),
            carrier: read_attr(root, &format!("{base}/carrier"))
                .as_deref()
                .map(parse_flag),
            mac: read_attr(root, &format!("{base}/address")).map(|s| s.to_ascii_lowercase()),
            mtu: read_attr(root, &format!("{base}/mtu")).and_then(|s| s.parse().ok()),
            speed_mbps: read_attr(root, &format!("{base}/speed"))
                .and_then(|s| s.parse().ok())
                .filter(|v: &i64| *v >= 0),
            rx_bytes,
            tx_bytes,
            name,
        });
    }

    let mut addrs = Vec::new();
    if let Some(text) = read_file(root, "proc/net/if_inet6") {
        addrs.extend(parse_if_inet6(&text));
    }
    let ip_available = match read_ip_addr() {
        Some(text) => {
            addrs.extend(parse_ip_addr(&text));
            Some(true)
        }
        None => Some(false),
    };
    addrs.sort_by(|a, b| (&a.name, a.family, &a.addr).cmp(&(&b.name, b.family, &b.addr)));

    let routes = read_file(root, "proc/net/route")
        .as_deref()
        .map_or_else(Vec::new, parse_route);

    let dns_text = read_file(root, "run/systemd/resolve/resolv.conf");
    let dns = dns_text
        .as_deref()
        .map_or_else(Dns::default, parse_resolv_conf);

    NetFacts {
        ifaces,
        addrs,
        routes,
        dns,
        ip_available,
    }
}

/// Parse the contents of `/sys/class/net/<if>/operstate`.
///
/// Anything that is not `up` or `down` becomes `unknown`, including the
/// `unknown` the kernel itself writes and the empty read a driver that has no
/// state yet produces. Mapping a missing state onto `down` would report a
/// working interface as broken, which is the mistake this function exists to
/// not make.
#[must_use]
pub fn parse_operstate(text: &str) -> String {
    match text.trim() {
        "up" => "up".to_owned(),
        "down" => "down".to_owned(),
        _ => "unknown".to_owned(),
    }
}

/// Parse a sysfs boolean attribute (`1` / `0`) and not a word.
#[must_use]
pub fn parse_flag(text: &str) -> bool {
    matches!(text.trim(), "1" | "yes" | "true")
}

/// Parse `/proc/net/dev` into `name -> (rx_bytes, tx_bytes)`.
///
/// The file has two header lines and each row is `name:` followed by sixteen
/// counters. The indices are used rather than a search for a named field
/// because the format is positional; `rx_bytes` is field 0 and `tx_bytes` is
/// field 8, and a row with fewer fields is skipped rather than guessed at.
#[must_use]
pub fn parse_net_dev(text: &str) -> Vec<(String, (u64, u64))> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some((name, rest)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() || name.contains(char::is_whitespace) {
            continue;
        }
        let fields: Vec<&str> = rest.split_whitespace().collect();
        if fields.len() < 16 {
            continue;
        }
        // `get` rather than `[]`: the length check above is the reason the
        // indices are in range, and a parser that turns a wrong assumption
        // about untrusted input into a panic is a daemon that stops answering.
        let rx = fields.first().and_then(|f| f.parse::<u64>().ok());
        let tx = fields.get(8).and_then(|f| f.parse::<u64>().ok());
        let (Some(rx), Some(tx)) = (rx, tx) else {
            continue;
        };
        out.push((name.to_owned(), (rx, tx)));
    }
    out
}

/// Parse `/proc/net/route` into routes.
///
/// The addresses are 32-bit values written in **little-endian byte order**, so
/// the hex digits of `0102A8C0` mean 192.168.2.1, not 1.2.168.192. Reading them
/// the other way round produces addresses that look entirely plausible and are
/// wrong, which is why the bytes are reversed here and a test asserts it.
#[must_use]
pub fn parse_route(text: &str) -> Vec<Route> {
    let mut out = Vec::new();
    for line in text.lines().skip(1) {
        let f: Vec<&str> = line.split_whitespace().collect();
        let (Some(iface), Some(destination), Some(gateway), Some(mask), Some(metric)) =
            (f.first(), f.get(1), f.get(2), f.get(7), f.get(6))
        else {
            continue;
        };
        let (Some(dest), Some(prefix)) = (hex_v4(destination), netmask_to_prefix(mask)) else {
            continue;
        };
        out.push(Route {
            destination: format!("{dest}/{prefix}"),
            // The kernel writes 00000000 for "no next hop", which is what a
            // directly connected route has. Printing `0.0.0.0` would be a
            // gateway the machine does not have, so the sentinel becomes the
            // same `-` every other absent field gets.
            via: match hex_v4(gateway) {
                Some(g) if !g.is_unspecified() => g.to_string(),
                _ => "-".to_owned(),
            },
            dev: (*iface).to_owned(),
            metric: metric.parse().unwrap_or(0),
        });
    }
    out
}

/// Parse one `/proc/net/route` field as a dotted-quad IPv4 address.
#[must_use]
pub fn hex_v4(field: &str) -> Option<std::net::Ipv4Addr> {
    if field.len() != 8 {
        return None;
    }
    let raw = u32::from_str_radix(field, 16).ok()?;
    // The kernel writes the address in host byte order on a little-endian
    // machine, i.e. least significant byte first.
    let bytes = raw.to_le_bytes();
    Some(std::net::Ipv4Addr::from(bytes))
}

/// The number of leading ones in a netmask, e.g. `FFFFFF00` is 24.
#[must_use]
pub fn netmask_to_prefix(mask: &str) -> Option<u8> {
    let addr = hex_v4(mask)?;
    let bits = u32::from(addr);
    // A netmask is a run of ones followed by zeros; anything else is not a
    // netmask and its "prefix length" would be an invention.
    let prefix = bits.count_ones();
    // A prefix of 0 shifts by 32, which is not a shift at all — `checked_shl`
    // says so rather than wrapping, and a wrapped shift here would make a
    // netmask of all zeros look like a netmask of ones.
    let expected = u32::MAX
        .checked_shl(32u32.saturating_sub(prefix))
        .unwrap_or(0);
    if bits == expected {
        u8::try_from(prefix).ok()
    } else {
        None
    }
}

/// Parse `/proc/net/if_inet6`.
///
/// Each line is the address as 32 hex digits, then the interface index, prefix
/// length, scope and flags — all in hex — then the device name.
#[must_use]
pub fn parse_if_inet6(text: &str) -> Vec<Addr> {
    let mut out = Vec::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let (Some(address), Some(prefix_field), Some(scope_field), Some(name)) =
            (f.first(), f.get(2), f.get(3), f.get(5))
        else {
            continue;
        };
        if address.len() != 32 {
            continue;
        }
        let Ok(prefix) = u8::from_str_radix(prefix_field, 16) else {
            continue;
        };
        let scope = u8::from_str_radix(scope_field, 16).unwrap_or(0xff);
        let Some(addr) = hex_v6(address) else {
            continue;
        };
        out.push(Addr {
            name: (*name).to_owned(),
            family: 6,
            addr,
            prefix,
            scope: scope_word(scope).to_owned(),
        });
    }
    out
}

/// Parse 32 hex digits as an IPv6 address in RFC 5952 form.
#[must_use]
pub fn hex_v6(field: &str) -> Option<String> {
    if field.len() != 32 {
        return None;
    }
    let mut groups = [0u16; 8];
    for (i, g) in groups.iter_mut().enumerate() {
        let start = i.saturating_mul(4);
        *g = u16::from_str_radix(field.get(start..start.saturating_add(4))?, 16).ok()?;
    }
    Some(format_v6(&groups))
}

/// Format eight groups as an IPv6 address, compressing the longest run of
/// zero groups as `::` — RFC 5952's rule, including its minimum of two.
///
/// Two groups is the minimum because `::` standing for a single zero group is
/// legal but is not what any other tool prints, and a facts line that disagrees
/// with `ip` about the same address is a fact nobody can diff.
#[must_use]
pub fn format_v6(groups: &[u16; 8]) -> String {
    // The longest run of zero groups, as `(start, len)`. Iterated rather than
    // indexed: the array's length is a type-level fact here, and a loop that
    // reads by position hides that from the compiler's bounds analysis.
    let mut best = (0usize, 0usize);
    let mut run: Option<usize> = None;
    for (i, g) in groups.iter().enumerate() {
        if *g == 0 {
            let start = run.unwrap_or(i);
            run = Some(start);
            let len = i.saturating_sub(start).saturating_add(1);
            if len > best.1 {
                best = (start, len);
            }
        } else {
            run = None;
        }
    }
    // Two groups is the minimum: `::` standing for a single zero group is
    // legal and is not what any other tool prints.
    let (zero_start, zero_len) = if best.1 >= 2 { best } else { (0, 0) };

    let mut out = String::new();
    let mut i = 0usize;
    while let Some(g) = groups.get(i) {
        if zero_len > 0 && i == zero_start {
            out.push_str("::");
            i = i.saturating_add(zero_len);
            continue;
        }
        if !out.is_empty() && !out.ends_with(':') {
            out.push(':');
        }
        out.push_str(&format!("{g:x}"));
        i = i.saturating_add(1);
    }
    if out.is_empty() {
        out.push_str("::");
    }
    out
}

/// The word for an `if_inet6` scope value.
fn scope_word(scope: u8) -> &'static str {
    match scope {
        0x00 => "global",
        0x10 => "host",
        0x20 => "link",
        0x40 => "site",
        _ => "unknown",
    }
}

/// Parse `ip -o -4 addr show` output.
///
/// One address per line, e.g.
///
/// ```text
/// 2: eth0    inet 192.168.2.5/24 brd 192.168.2.255 scope global dynamic eth0\
///        valid_lft 86362sec preferred_lft 86362sec
/// ```
///
/// The interface is taken from the `N: name` prefix rather than from the
/// trailing device field, because the trailing field is absent for a peer
/// address; the prefix is always there.
#[must_use]
pub fn parse_ip_addr(text: &str) -> Vec<Addr> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some((head, rest)) = line.split_once(':') else {
            continue;
        };
        if head.trim().parse::<u32>().is_err() {
            continue;
        }
        let mut fields = rest.split_whitespace();
        let name = match fields.next() {
            Some(n) => n.to_owned(),
            None => continue,
        };
        let tokens: Vec<&str> = rest.split_whitespace().skip(1).collect();
        let Some(pos) = tokens.iter().position(|t| *t == "inet") else {
            continue;
        };
        let Some(cidr) = tokens.get(pos.saturating_add(1)) else {
            continue;
        };
        let Some((addr, prefix)) = cidr.split_once('/') else {
            continue;
        };
        let Ok(prefix) = prefix.parse::<u8>() else {
            continue;
        };
        if addr.parse::<std::net::Ipv4Addr>().is_err() {
            continue;
        }
        let scope = tokens
            .iter()
            .position(|t| *t == "scope")
            .and_then(|i| tokens.get(i.saturating_add(1)))
            .map_or("global", |s| match *s {
                "global" => "global",
                "link" => "link",
                "host" => "host",
                _ => "unknown",
            });
        out.push(Addr {
            name,
            family: 4,
            addr: addr.to_owned(),
            prefix,
            scope: scope.to_owned(),
        });
    }
    out
}

/// Parse `resolved`'s `resolv.conf`.
#[must_use]
pub fn parse_resolv_conf(text: &str) -> Dns {
    let mut dns = Dns {
        source_present: true,
        ..Dns::default()
    };
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let mut f = line.split_whitespace();
        match f.next() {
            Some("nameserver") => {
                if let Some(v) = f.next() {
                    dns.nameservers.push(v.to_owned());
                }
            }
            Some("search") => dns.search.extend(f.map(str::to_owned)),
            _ => {}
        }
    }
    dns
}

/// Run `ip -o -4 addr show`, or `None` when it is not installed.
///
/// A fixed argument vector, no shell, and the binary looked up at the four
/// absolute paths it is installed at across this distribution's layouts. Using
/// `PATH` would let a caller's environment choose which `ip` runs, and a fact
/// reader that runs a program the caller named is a fact reader that can be
/// made to report anything.
fn read_ip_addr() -> Option<String> {
    for bin in ["/usr/bin/ip", "/usr/sbin/ip", "/bin/ip", "/sbin/ip"] {
        if !std::path::Path::new(bin).exists() {
            continue;
        }
        let out = Command::new(bin)
            .args(["-o", "-4", "addr", "show"])
            .output()
            .ok()?;
        if out.status.success() {
            return Some(String::from_utf8_lossy(&out.stdout).into_owned());
        }
    }
    None
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    #[test]
    fn operstate_keeps_the_three_answers_apart() {
        assert_eq!(parse_operstate("up\n"), "up");
        assert_eq!(parse_operstate("down\n"), "down");
        // The kernel writes "unknown" for an interface whose driver has not
        // reported yet. Reporting that as "down" would call a working link
        // broken; reporting it as "up" would call a broken one working.
        assert_eq!(parse_operstate("unknown\n"), "unknown");
        assert_eq!(parse_operstate(""), "unknown");
        assert_eq!(parse_operstate("dormant"), "unknown");
    }

    #[test]
    fn net_dev_counters_are_read_positionally() {
        let text = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo:       0       0    0    0    0     0          0         0        0       0    0    0    0     0       0          0
  eth0: 5359841   45123    0    0    0     0          0         0  1228374   14200    0    0    0     0       0          0
";
        let v = parse_net_dev(text);
        assert_eq!(
            v,
            vec![
                ("lo".to_owned(), (0, 0)),
                ("eth0".to_owned(), (5_359_841, 1_228_374)),
            ]
        );
    }

    #[test]
    fn a_short_net_dev_row_is_skipped_not_padded() {
        // A row with missing fields would otherwise read a later column as
        // bytes-sent and report a counter that belongs to something else.
        let text = "eth0: 5 1 2\n";
        assert!(parse_net_dev(text).is_empty());
    }

    #[test]
    fn route_hex_is_little_endian() {
        // The exact line a default gateway produces on this machine, and the
        // value it means. Reading the hex the other way gives 1.2.168.192.
        let text = "\
Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT
eth0\t00000000\t0102A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0
eth0\t0002A8C0\t00000000\t0001\t0\t0\t100\t00FFFFFF\t0\t0\t0
";
        let routes = parse_route(text);
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].destination, "0.0.0.0/0");
        assert_eq!(routes[0].via, "192.168.2.1");
        assert_eq!(routes[0].dev, "eth0");
        assert_eq!(routes[0].metric, 100);
        assert_eq!(routes[1].destination, "192.168.2.0/24");
        assert_eq!(routes[1].via, "-");
    }

    #[test]
    fn a_route_with_no_next_hop_says_so_instead_of_naming_address_zero() {
        // 0.0.0.0 in the gateway column is a sentinel, and a direct route
        // reported with a gateway of 0.0.0.0 is a gateway nobody can dial.
        let text =
            "Iface\tDestination\tGateway\tFlags\tRefCnt\tUse\tMetric\tMask\tMTU\tWindow\tIRTT\n\
                    eth0\t0002A8C0\t00000000\t0001\t0\t0\t100\t00FFFFFF\t0\t0\t0\n";
        let routes = parse_route(text);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].via, "-");

        // ...while a real gateway is printed as itself.
        let text =
            "Iface\tDestination\tGateway\tFlags\tRefCnt\tUse\tMetric\tMask\tMTU\tWindow\tIRTT\n\
                    eth0\t00000000\t0102A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0\n";
        let routes = parse_route(text);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].via, "192.168.2.1");
        assert_eq!(routes[0].destination, "0.0.0.0/0");
    }

    #[test]
    fn a_route_whose_mask_is_not_a_netmask_is_dropped_rather_than_invented() {
        // A prefix length is a claim; one cannot be derived from a mask that
        // is not a run of ones, and defaulting to /0 would describe a default
        // route that may not exist.
        let text =
            "Iface\tDestination\tGateway\tFlags\tRefCnt\tUse\tMetric\tMask\tMTU\tWindow\tIRTT\n\
                    eth0\t0002A8C0\t00000000\t0001\t0\t0\t100\t0F0F0F0F\t0\t0\t0\n";
        assert!(parse_route(text).is_empty());
    }

    #[test]
    fn a_mask_that_is_not_a_run_of_ones_has_no_prefix_length() {
        assert_eq!(netmask_to_prefix("00FFFFFF"), Some(24));
        assert_eq!(netmask_to_prefix("00000000"), Some(0));
        assert_eq!(netmask_to_prefix("FFFFFFFF"), Some(32));
        // Not a netmask: reporting a prefix length for it would be inventing a
        // number the machine never said.
        assert_eq!(netmask_to_prefix("0F0F0F0F"), None);
        assert_eq!(netmask_to_prefix("nonsense"), None);
    }

    #[test]
    fn ipv6_addresses_are_compressed_the_way_every_other_tool_prints_them() {
        let line = "00000000000000000000000000000001 01 80 10 80       lo\n\
                    fe800000000000000000000000000001 02 40 20 80     eth0\n\
                    20010db8000000000000000000000001 02 40 00 00     eth0\n";
        let addrs = parse_if_inet6(line);
        assert_eq!(addrs.len(), 3);
        assert_eq!(addrs[0].addr, "::1");
        assert_eq!(addrs[0].scope, "host");
        assert_eq!(addrs[0].prefix, 128);
        assert_eq!(addrs[1].addr, "fe80::1");
        assert_eq!(addrs[1].scope, "link");
        assert_eq!(addrs[2].addr, "2001:db8::1");
        assert_eq!(addrs[2].scope, "global");
    }

    #[test]
    fn a_single_zero_group_is_not_compressed() {
        // RFC 5952 4.2.2: `::` for one group is legal and no implementation
        // prints it. A facts line that disagrees with `ip` about the same
        // address is a fact that cannot be diffed.
        let groups = [0x2001, 0x0db8, 0, 1, 2, 3, 4, 5];
        assert_eq!(format_v6(&groups), "2001:db8:0:1:2:3:4:5");
        // Two is the minimum, and the LONGEST run wins.
        let groups = [1, 0, 0, 0, 5, 0, 0, 7];
        assert_eq!(format_v6(&groups), "1::5:0:0:7");
        let groups = [0, 0, 0, 0, 0, 0, 0, 1];
        assert_eq!(format_v6(&groups), "::1");
        let groups = [1, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(format_v6(&groups), "1::");
        let groups = [0; 8];
        assert_eq!(format_v6(&groups), "::");
    }

    #[test]
    fn ip_addr_output_is_parsed_per_line() {
        let text = "\
1: lo    inet 127.0.0.1/8 scope host lo\\       valid_lft forever preferred_lft forever
2: eth0    inet 10.0.2.15/24 brd 10.0.2.255 scope global dynamic eth0\\       valid_lft 86362sec preferred_lft 86362sec
2: eth0    inet 169.254.3.4/16 brd 169.254.255.255 scope link eth0\\       valid_lft forever preferred_lft forever
";
        let addrs = parse_ip_addr(text);
        assert_eq!(addrs.len(), 3);
        assert_eq!(addrs[0].name, "lo");
        assert_eq!(addrs[0].addr, "127.0.0.1");
        assert_eq!(addrs[0].prefix, 8);
        assert_eq!(addrs[0].scope, "host");
        assert_eq!(addrs[1].addr, "10.0.2.15");
        assert_eq!(addrs[1].scope, "global");
        assert_eq!(addrs[2].scope, "link");
    }

    #[test]
    fn ip_addr_garbage_yields_nothing_rather_than_a_guess() {
        for junk in [
            "",
            "Command 'ip' not found\n",
            "2: eth0    inet not-an-address/24 scope global eth0\n",
            "2: eth0    inet 10.0.2.15 scope global eth0\n",
        ] {
            assert!(parse_ip_addr(junk).is_empty(), "parsed {junk:?}");
        }
    }

    #[test]
    fn resolv_conf_reports_servers_search_and_whether_it_was_there() {
        let text = "\
# This is /run/systemd/resolve/resolv.conf
nameserver 10.0.2.3
nameserver 1.1.1.1
search lan
";
        let d = parse_resolv_conf(text);
        assert!(d.source_present);
        assert_eq!(d.nameservers, vec!["10.0.2.3", "1.1.1.1"]);
        assert_eq!(d.search, vec!["lan"]);

        // A root with no resolv.conf must not claim resolved is running.
        assert!(!Dns::default().source_present);
    }

    #[test]
    fn the_reply_says_why_there_are_no_ipv4_addresses() {
        let mut f = NetFacts {
            ip_available: Some(false),
            ..NetFacts::default()
        };
        assert!(f.to_lines().iter().any(|l| l == "net.addr.v4=none"));
        f.ip_available = None;
        assert!(f.to_lines().iter().any(|l| l == "net.addr.v4=unknown"));
    }

    #[test]
    fn fixture_tree_is_read_end_to_end() {
        // A whole fake machine on disk, so the reader path is exercised and not
        // only the parsers: sysfs attributes, proc files, resolv.conf.
        let root = std::env::temp_dir().join("litric-net-fixture");
        let _ = std::fs::remove_dir_all(&root);
        for (rel, content) in [
            ("sys/class/net/lo/operstate", "unknown\n"),
            ("sys/class/net/lo/address", "00:00:00:00:00:00\n"),
            ("sys/class/net/lo/mtu", "65536\n"),
            ("sys/class/net/eth0/operstate", "up\n"),
            ("sys/class/net/eth0/carrier", "1\n"),
            ("sys/class/net/eth0/address", "52:54:00:12:34:56\n"),
            ("sys/class/net/eth0/mtu", "1500\n"),
            ("sys/class/net/eth0/speed", "1000\n"),
            ("proc/net/dev", "eth0: 10 0 0 0 0 0 0 0 20 0 0 0 0 0 0 0\n"),
            (
                "proc/net/route",
                "Iface\tDestination\tGateway\tFlags\tRefCnt\tUse\tMetric\tMask\tMTU\tWindow\tIRTT\n\
                 eth0\t00000000\t0302A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0\n",
            ),
            ("run/systemd/resolve/resolv.conf", "nameserver 10.0.2.3\n"),
        ] {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
            std::fs::write(p, content).expect("write");
        }

        let facts = read(&root);
        assert_eq!(facts.ifaces.len(), 2);
        let eth0 = facts
            .ifaces
            .iter()
            .find(|i| i.name == "eth0")
            .expect("eth0");
        assert_eq!(eth0.operstate, "up");
        assert_eq!(eth0.carrier, Some(true));
        assert_eq!(eth0.mac.as_deref(), Some("52:54:00:12:34:56"));
        assert_eq!(eth0.mtu, Some(1500));
        assert_eq!(eth0.speed_mbps, Some(1000));
        assert_eq!((eth0.rx_bytes, eth0.tx_bytes), (10, 20));
        // lo has no carrier file, and no carrier is not "no carrier".
        let lo = facts.ifaces.iter().find(|i| i.name == "lo").expect("lo");
        assert_eq!(lo.carrier, None);
        assert_eq!(facts.routes.len(), 1);
        assert_eq!(facts.routes[0].via, "192.168.2.3");
        assert!(facts.dns.source_present);
        assert_eq!(facts.dns.nameservers, vec!["10.0.2.3"]);

        let _ = std::fs::remove_dir_all(&root);
    }
}
