use mdns_sd::{ServiceDaemon, ServiceInfo};

pub fn mdns_broadcast(device_id: &str) {
    let mdns = ServiceDaemon::new().expect("Could not create service daemon");
    let airplay_service_type = "_airplay._tcp.local.";
    let roap_service_type = "_raop._tcp.local.";

    let host_name = hostname::get().unwrap();

    let instance_name = format!("OpeniTools@{}", host_name.to_str().unwrap());

    let my_addrs = "";

    let airplay_service_hostname = format!("{}{}", instance_name, &airplay_service_type);
    let roap_service_hostname = format!("{}{}", instance_name, &roap_service_type);

    let port = 5200;

    let airplay_properties = [
        ("model", "AppleTV3,2"),
        ("protovers", "1.1"),
        ("srcvers", "220.68"),
        ("features", &format!("0x{:08X},0x0", crate::FEATURES)),
        ("deviceid", device_id),
        ("pi", "2e388006-13ba-4041-9a67-25dd4a43d536"),
        ("vv", "2"),
        ("pw", "false"),
        ("flags", "0x4"),
    ];

    let roap_properties = [
        ("ch", "2"),
        ("cn", "0,1,2,3"),
        ("da", "true"),
        ("et", "0"),
        ("vv", "2"),
        ("ft", &format!("0x{:08X},0x0", crate::FEATURES)),
        ("am", "AppleTV3,2"),
        ("md", "0,1,2"),
        ("rhd", "5.6.0.0"),
        ("pw", "false"),
        ("sf", "0x4"),
        ("sr", "44100"),
        ("ss", "16"),
        ("sv", "false"),
        ("tp", "UDP"),
        ("txtvers", "1"),
        ("sf", "0x4"),
        ("vs", "220.68"),
        ("vn", "65537"),
    ];

    let airplay_service_info = ServiceInfo::new(
        airplay_service_type,
        &instance_name,
        &airplay_service_hostname,
        my_addrs,
        port,
        &airplay_properties[..],
    )
    .expect("valid service info")
    .enable_addr_auto();

    let roap_service_info = ServiceInfo::new(
        roap_service_type,
        &instance_name,
        &roap_service_hostname,
        my_addrs,
        port,
        &roap_properties[..],
    )
    .expect("valid service info")
    .enable_addr_auto();

    mdns.register(airplay_service_info)
        .expect("Failed to register mDNS service");

    mdns.register(roap_service_info)
        .expect("Failed to register mDNS service");
}
