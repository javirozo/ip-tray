#![windows_subsystem = "windows"]

mod icon;

use std::io::Write;
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};

const REFRESH_EVERY: Duration = Duration::from_secs(300);
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

struct NetInfo {
    public_ip: String,
    private_ip: String,
    dns: Vec<String>,
}

impl NetInfo {
    fn tooltip(&self) -> String {
        let dns = if self.dns.is_empty() { "n/d".to_string() } else { self.dns.join(", ") };
        format!(
            "IP pública: {}\nIP privada: {}\nDNS: {}",
            self.public_ip, self.private_ip, dns
        )
    }
}

enum UserEvent {
    Info(NetInfo),
    Menu(MenuEvent),
}

fn fetch_ip() -> Result<String, String> {
    let body = ureq::get("https://api.ipify.org")
        .timeout(Duration::from_secs(10))
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())?;
    Ok(body.trim().to_string())
}

/// IP local de la interfaz con ruta a internet (el connect UDP no envía paquetes).
fn private_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| s.connect("8.8.8.8:80").and_then(|_| s.local_addr()))
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "n/d".to_string())
}

fn dns_servers() -> Vec<String> {
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetNetworkParams, FIXED_INFO_W2KSP1, IP_ADDR_STRING,
    };

    let mut len: u32 = 0;
    unsafe { GetNetworkParams(std::ptr::null_mut(), &mut len) };
    if len == 0 {
        return Vec::new();
    }
    // Vec<u64> garantiza la alineación que requiere FIXED_INFO.
    let mut buf = vec![0u64; (len as usize + 7) / 8];
    let info = buf.as_mut_ptr() as *mut FIXED_INFO_W2KSP1;
    if unsafe { GetNetworkParams(info, &mut len) } != 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut node: *const IP_ADDR_STRING = unsafe { &(*info).DnsServerList };
    while !node.is_null() {
        let s = unsafe { &(*node).IpAddress.String };
        let end = s.iter().position(|&c| c == 0).unwrap_or(s.len());
        let bytes: Vec<u8> = s[..end].iter().map(|&c| c as u8).collect();
        let ip = String::from_utf8_lossy(&bytes).to_string();
        if !ip.is_empty() {
            out.push(ip);
        }
        node = unsafe { (*node).Next };
    }
    out
}

fn copy_to_clipboard(text: &str) {
    if let Ok(mut child) = Command::new("clip")
        .stdin(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

/// Globo terráqueo: azul = online, rojo = sin conexión.
fn make_icon(online: bool) -> Icon {
    Icon::from_rgba(icon::rgba(online, 32), 32, 32).expect("icono inválido")
}

fn main() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |e| {
        let _ = proxy.send_event(UserEvent::Menu(e));
    }));

    let ip_item = MenuItem::new("IP: consultando…", true, None);
    let refresh_item = MenuItem::new("Actualizar", true, None);
    let quit_item = MenuItem::new("Salir", true, None);
    let menu = Menu::new();
    menu.append_items(&[
        &ip_item,
        &PredefinedMenuItem::separator(),
        &refresh_item,
        &quit_item,
    ])
    .unwrap();

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("IP pública: consultando…")
        .with_icon(make_icon(false))
        .build()
        .expect("no se pudo crear el icono de bandeja");

    // Hilo de red: consulta al inicio, cada 5 min, o cuando se pide "Actualizar".
    let (refresh_tx, refresh_rx) = mpsc::channel::<()>();
    let proxy = event_loop.create_proxy();
    thread::spawn(move || loop {
        let info = NetInfo {
            public_ip: fetch_ip().unwrap_or_else(|_| "sin conexión".to_string()),
            private_ip: private_ip(),
            dns: dns_servers(),
        };
        if proxy.send_event(UserEvent::Info(info)).is_err() {
            break;
        }
        let _ = refresh_rx.recv_timeout(REFRESH_EVERY);
    });

    let mut current_ip = String::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let _keep_alive = &tray;

        if let Event::UserEvent(ev) = event {
            match ev {
                UserEvent::Info(info) => {
                    ip_item.set_text(format!("IP pública: {}  (clic para copiar)", info.public_ip));
                    let _ = tray.set_tooltip(Some(info.tooltip()));
                    let _ = tray.set_icon(Some(make_icon(info.public_ip.contains('.'))));
                    current_ip = info.public_ip;
                }
                UserEvent::Menu(e) => {
                    if e.id == quit_item.id() {
                        *control_flow = ControlFlow::Exit;
                    } else if e.id == refresh_item.id() {
                        let _ = refresh_tx.send(());
                    } else if e.id == ip_item.id() && current_ip.contains('.') {
                        copy_to_clipboard(&current_ip);
                    }
                }
            }
        }
    });
}
