pub fn get_current_wifi_ssid() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        get_windows_wifi()
    }

    #[cfg(target_os = "macos")]
    {
        get_macos_wifi()
    }

    #[cfg(target_os = "linux")]
    {
        get_linux_wifi()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

// WINDOWS

#[cfg(target_os = "windows")]
fn get_windows_wifi() -> Option<String> {
    use windows::Win32::Networking::NetworkListManager::{
        CoCreateInstance, INetworkListManager, NLM_ENUM_NETWORK_CONNECTED, NetworkListManager,
    };

    use windows::Win32::System::Com::{
        CLSCTX_ALL, COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize,
    };

    unsafe {
        // COM-Bibliothek für den aktuellen Thread initialisieren.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let result = get_windows_wifi_inner();

        CoUninitialize();

        result
    }
}

#[cfg(target_os = "windows")]
unsafe fn get_windows_wifi_inner() -> Option<String> {
    use windows::Win32::Networking::NetworkListManager::{
        CoCreateInstance, INetworkListManager, NLM_ENUM_NETWORK_CONNECTED, NetworkListManager,
    };

    let manager = match CoCreateInstance::<_, INetworkListManager>(
        &NetworkListManager,
        None,
        windows::Win32::System::Com::CLSCTX_ALL,
    ) {
        Ok(manager) => manager,
        Err(error) => {
            eprintln!("Network List Manager konnte nicht erstellt werden: {error}");

            return None;
        }
    };

    let networks = match manager.GetNetworks(NLM_ENUM_NETWORK_CONNECTED) {
        Ok(networks) => networks,
        Err(error) => {
            eprintln!("Verbundene Netzwerke konnten nicht abgefragt werden: {error}");

            return None;
        }
    };

    loop {
        let mut fetched = 0;
        let mut network = None;

        match networks.Next(&mut network, &mut fetched) {
            Ok(_) if fetched == 1 => {
                if let Some(network) = network {
                    if let Ok(name) = network.GetName() {
                        let ssid = name.to_string();

                        if !ssid.is_empty() {
                            return Some(ssid);
                        }
                    }
                }
            }

            _ => {
                break;
            }
        }
    }

    None
}

// NACOS

#[cfg(target_os = "macos")]
fn get_macos_wifi() -> Option<String> {
    use objc2_core_wlan::CWWiFiClient;

    // Gemeinsamen CoreWLAN-Client holen.
    let client = CWWiFiClient::sharedWiFiClient();

    // Aktives WLAN-Interface ermitteln.
    let interface = client.interface()?;

    // SSID des Interfaces abfragen.
    //
    // API ist unsafe
    let ssid = unsafe { interface.ssid() }?;

    let ssid = ssid.to_string();

    if ssid.is_empty() { None } else { Some(ssid) }
}

// LINUX

#[cfg(target_os = "linux")]
fn get_linux_wifi() -> Option<String> {
    use zbus::blocking::Connection;
    use zbus::zvariant::OwnedObjectPath;

    // Verbindung zum System-D-Bus herstellen.
    let connection = Connection::system().ok()?;

    // Aktive Netzwerkverbindungen abfragen

    let active_connections: Vec<OwnedObjectPath> = connection
        .call_method(
            Some("org.freedesktop.NetworkManager"),
            "/org/freedesktop/NetworkManager",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.NetworkManager", "ActiveConnections"),
        )
        .ok()?
        .body()
        .deserialize()
        .ok()?;

    // Aktive Verbindungen durchgehen

    for path in active_connections {
        // Typ der aktiven Verbindung abfragen
        let connection_type: String = match connection.call_method(
            Some("org.freedesktop.NetworkManager"),
            &path,
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.NetworkManager.Connection.Active", "Type"),
        ) {
            Ok(reply) => match reply.body().deserialize() {
                Ok(value) => value,
                Err(_) => continue,
            },

            Err(_) => continue,
        };

        // Nur WLAN-Verbindungen
        if connection_type != "802-11-wireless" {
            continue;
        }

        // Namen/SSID der aktiven WLAN-Verbindung auslesen

        let ssid: String = match connection.call_method(
            Some("org.freedesktop.NetworkManager"),
            &path,
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.NetworkManager.Connection.Active", "Id"),
        ) {
            Ok(reply) => match reply.body().deserialize() {
                Ok(value) => value,
                Err(_) => continue,
            },

            Err(_) => continue,
        };

        if !ssid.is_empty() {
            return Some(ssid);
        }
    }

    None
}
