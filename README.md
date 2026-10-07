# ip-tray

Utilidad mínima para Windows que vive en la bandeja del sistema y muestra tu red de un vistazo.
Hecha en Rust: un solo `.exe` de ~1,5 MB, sin ventana ni instalador, ~14 MB de RAM.

Al pasar el mouse sobre el ícono:

```
IP pública: x.x.x.x
IP privada: 192.168.x.x
DNS: 8.8.8.8, 1.1.1.1
```

Un clic abre el menú: **copiar IP pública**, **Actualizar** y **Salir**.

## Características

- IP pública vía [api.ipify.org](https://api.ipify.org) (HTTPS), refrescada cada 5 minutos o a demanda.
- IP privada: la de la interfaz con salida a internet (no se envían paquetes para obtenerla).
- DNS: leídos de Windows con `GetNetworkParams`, sin lanzar procesos externos.
- Ícono de globo dibujado en código: **azul** con conexión, **rojo** sin conexión.
- Sin telemetría. La única conexión de red es la consulta a ipify.

## Compilar

Requisitos:

1. [Rust](https://rustup.rs) (`winget install Rustlang.Rustup`).
2. Build Tools de Visual Studio con la carga **Desktop development with C++**
   (incluye el linker MSVC y el Windows SDK, necesario para embeber el ícono del `.exe`).

```bash
cargo build --release
```

El binario queda en `target\release\ip-tray.exe`.

## Iniciar con Windows

Poné un acceso directo a `ip-tray.exe` en la carpeta de inicio (`Win+R` → `shell:startup`).

## Estructura

| Archivo | Rol |
| --- | --- |
| `src/main.rs` | Tray, menú, hilo de consulta de red y bucle de eventos |
| `src/icon.rs` | Dibujo del globo (compartido con `build.rs`) |
| `build.rs` | Genera el `.ico` y lo embebe en el `.exe` con `winresource` |

Dependencias: [`tao`](https://crates.io/crates/tao) y [`tray-icon`](https://crates.io/crates/tray-icon)
(ventana/tray), [`ureq`](https://crates.io/crates/ureq) (HTTP bloqueante),
[`windows-sys`](https://crates.io/crates/windows-sys) (DNS).

## Limitaciones

- El tooltip nativo de Windows se trunca a ~127 caracteres (puede cortarse con muchos DNS).
- Los DNS mostrados son los globales de Windows, no los de cada adaptador.
- Solo Windows.
