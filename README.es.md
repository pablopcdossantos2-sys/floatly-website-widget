# Widget Website — Windows

**Idiomas:** [Português (Brasil)](README.md) · [English](README.en.md) · Español

Widget compacto para Windows que mantiene visible un sitio web elegido por el usuario en una ventana WebView2 sin bordes. El sitio se configura en el primer inicio; no se incluye ninguna URL de terceros por defecto. Desarrollado con Tauri 2 para Windows 10/11 x64.

## Funciones

- Icono en la bandeja del sistema, instancia única y atajo global (predeterminado: CTRL+ALT+C). Cerrar la ventana la oculta en la bandeja; usa **Salir** para cerrar la aplicación.
- Sesión WebView2 persistente en %APPDATA%\WidgetWebsite\webview-data; configuración en %APPDATA%\WidgetWebsite\config.json.
- Interfaz disponible en portugués de Brasil, inglés y español. Cambia el idioma en **Configuración → Idioma**.
- Una pequeña pestaña en la parte superior central permite arrastrar el widget. Las zonas de interacción de los bordes siguen activas, pero son transparentes y no dibujan un marco alrededor del contenido. El tamaño y la posición se pueden cambiar en la configuración.
- El tamaño predeterminado es 350 × 550 px y el zoom predeterminado del navegador integrado es del 80 %; ambos se pueden cambiar en la configuración.
- URL, tamaño, posición, bloqueo del movimiento, mantener siempre visible, visibilidad en la barra de tareas, zoom, esquinas redondeadas y actualización automática configurables.
- Los enlaces externos se abren en el navegador predeterminado. Los dominios de autenticación reconocidos se abren en una ventana secundaria con una barra de direcciones sencilla y la misma carpeta de datos de WebView2. La ventana intenta cerrarse cuando el inicio de sesión vuelve al sitio configurado.
- Flujo de compilación para Windows x64 e instalador NSIS.
- La preparación inicial para Microsoft Store se documenta en [MICROSOFT_STORE_READINESS.md](MICROSOFT_STORE_READINESS.md) (portugués); el envío aún depende de las tareas pendientes indicadas.

## Requisitos

- Windows 10/11 x64, Node.js 22 o posterior (LTS) y npm.
- Rust estable 1.90 o posterior.
- Microsoft Edge WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/
- Para compilar localmente: Visual Studio Build Tools con Desarrollo para escritorio con C++ y Windows SDK.

## Desarrollo

Ejecuta estos comandos en PowerShell (es necesario generar los iconos antes de la primera compilación):

    npm ci
    npm run icons
    npm run tauri dev

## Compilar / empaquetar

    npm ci
    npm run icons
    npm run tauri build

El instalador se genera en src-tauri/target/release/bundle/nsis y el ejecutable en src-tauri/target/release. GitHub Actions compila el paquete para Windows y lo publica como artefacto widget-website-windows-x64.

## Primer inicio

Al abrir el widget por primera vez, se te pedirá que elijas un sitio web. Haz clic en **Elegir sitio web** e introduce una URL en **Configuración → URL del sitio**. El campo empieza vacío: no hay ningún sitio de terceros preconfigurado. El sitio debe permitir la carga en WebView2; algunos sitios pueden restringir funciones o flujos de inicio de sesión.

Para ver una URL de ejemplo concreta, consulta la sección de ejemplo a continuación.

## Microsoft Store

La guía detallada paso a paso está disponible en [portugués](GUIA_PUBLICACAO_MICROSOFT_STORE.md), junto con un [borrador multilingüe de la ficha de la Store](MICROSOFT_STORE_LISTING_DRAFT.md). La compilación para Store es manual y requiere la identidad real del publisher y los secretos del certificado. El workflow no significa que la aplicación esté lista para enviarse ni que haya sido aprobada.

## Idioma de la interfaz

Abre **Configuración → Idioma** y elige Portugués (Brasil), English o Español. La selección se guarda en %APPDATA%\WidgetWebsite\config.json y traduce la ventana de configuración, el menú de la bandeja del sistema, la barra de inicio de sesión y los mensajes de conexión propios del widget. El contenido del sitio elegido seguirá usando el idioma que proporcione el propio sitio.

## Firma digital y avisos de seguridad de Windows

El flujo de GitHub Actions admite firma Authenticode opcional. Para activarla, consigue un **certificado de firma de código de Windows de confianza** y crea estos secretos del repositorio en **Settings → Secrets and variables → Actions**:

- WINDOWS_PFX_BASE64: archivo de certificado PFX codificado en Base64.
- WINDOWS_PFX_PASSWORD: contraseña del archivo PFX.

No guardes una clave privada en el repositorio ni en este README. La compilación se ejecuta sin acceso al certificado de firma. Un trabajo independiente usa `signtool` para firmar y verificar el instalador NSIS y el ejecutable publicado por separado con SHA-256 y una marca de tiempo; luego elimina el PFX temporal. El ejecutable publicado por separado está firmado, pero la copia que ya está incluida dentro del instalador se empaquetó sin firma. Sin ambos secretos, los artefactos se publican sin firma.

**Importante:** una firma válida no garantiza que Microsoft Defender SmartScreen nunca muestre un aviso. Las aplicaciones o los archivos nuevos pueden mostrar advertencias hasta que se establezca la reputación del editor y del archivo. Publicar en Microsoft Store es la forma más directa de evitar avisos de descarga de SmartScreen. Consulta la [guía de reputación de SmartScreen de Microsoft](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation) y la [guía de firma de Tauri para Windows](https://tauri.app/distribute/sign/windows/).

## Ejemplo de uso: Investing.com

Investing.com es solo un ejemplo de sitio web que puede mostrar el widget. Para usarlo, abre **Configuración → URL del sitio** e introduce `https://br.investing.com/economic-calendar`. El widget no es exclusivo de este servicio: puedes configurar cualquier sitio HTTP o HTTPS.

Para los sitios que requieren inicio de sesión, utiliza el flujo de autenticación proporcionado por el propio sitio. El menú de la bandeja permite iniciar sesión o configurar el sitio web.

La aplicación no lee, copia ni almacena contraseñas. Se conserva el agente de usuario predeterminado de WebView2.

## Diagnóstico de bloqueos

- Los registros se guardan en %APPDATA%\WidgetWebsite\diagnostics.log (normalmente C:\Users\<usuario>\AppData\Roaming\WidgetWebsite\diagnostics.log). Las marcas de tiempo están en UTC. El archivo anterior se rota a diagnostics.log.1 al alcanzar aproximadamente 1 MB.
- La página envía una señal de actividad de JavaScript/WebView2 cada cinco segundos. Si no llega durante 20 segundos, el monitor registra un aviso, comprueba si la ventana nativa responde y busca procesos msedgewebview2.exe descendientes. La ausencia de estos procesos es una pista de que WebView2 pudo cerrarse, no una prueba de la causa.
- Los registros incluyen el inicio, la navegación, los errores de inicialización, las acciones de movimiento y la recuperación de actividad. Las URL se registran sin parámetros de consulta ni fragmentos para reducir el riesgo de guardar tokens de sesión.
- En una compilación gráfica de Windows puede no haber terminal. El archivo de registro es la fuente de diagnóstico fiable; ábrelo con el Bloc de notas después de reproducir el bloqueo.
- Estos diagnósticos no pueden demostrar que una aplicación externa causó el bloqueo. Tampoco pueden registrar más información si el proceso o todos sus hilos se terminan abruptamente.

## Privacidad y limitaciones

- Sin telemetría. La navegación principal se limita al sitio configurado y a recursos de autenticación reconocidos; los enlaces a otros dominios se abren en el navegador predeterminado. El sitio remoto solo recibe permiso para enviar la señal autenticada de actividad.
- Si el archivo de configuración está dañado, se reemplaza por valores predeterminados. **Restaurar tamaño predeterminado** solo cambia las dimensiones a 350 × 550 px; conserva las demás preferencias.
- CI comprueba la alineación de versiones de Tauri, el manifiesto de comandos y las capabilities; ejecuta `npm ci`, la verificación de tipos, `npm audit`, `cargo fmt --check`, `cargo clippy`, `cargo audit` y `cargo test`, y después compila el instalador de Windows.
- La ventana secundaria de inicio de sesión comparte la carpeta de datos de WebView2 e incluye una barra de direcciones inyectada. Los dominios no autorizados se abren en el navegador predeterminado. El cierre automático depende de que el proveedor vuelva al sitio configurado; algunos inicios de sesión sociales pueden requerir cerrar la ventana manualmente.
- El zoom usa el control nativo de WebView2. El recorte de las esquinas puede variar en Windows 10.
- Las comprobaciones de red muestran un panel propio antes de la navegación principal y supervisan la conectividad mientras el sitio está abierto. El envoltorio predeterminado de Tauri no puede reemplazar todos los errores de navegación de Chromium en todos los escenarios.
- Es necesario probar el inicio de sesión real, los filtros guardados, los conflictos de atajos, escalas del 125 %/150 %, varios monitores, cambios de DPI y consumo de memoria en hardware Windows.

## Desinstalación y datos guardados

El desinstalador NSIS ofrece la casilla **Delete the application data**. Si se marca, elimina `%APPDATA%\\WidgetWebsite` (configuración, registros y sesión de WebView2) y `%LOCALAPPDATA%\\WidgetWebsite` si existe. Si se deja desmarcada, estos datos se conservan para una reinstalación posterior. La entrada de inicio automático de Windows siempre se elimina al desinstalar la aplicación.

## Estructura

- src-tauri/src: configuración, ventana, comandos, diagnósticos y bandeja del sistema en Rust.
- src: inicialización TypeScript e interfaz de configuración.
- icons/widget.svg: fuente vectorial utilizada para generar los iconos de Windows.
