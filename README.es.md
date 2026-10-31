# Floatly — Windows

**Idiomas / Languages / Idiomas:** [Português (Brasil)](README.md) · [English](README-en.md) · Español

---

Navegador compacto para Windows que mantiene una dirección HTTP/HTTPS elegida por el usuario en una pequeña ventana WebView2 sin bordes. La dirección se configura en el primer inicio; no se incluye ningún sitio de terceros por defecto. Desarrollado con Tauri 2 para Windows 10/11 x64.

## Funciones

- Icono en la bandeja del sistema, instancia única y atajo global (predeterminado: CTRL+ALT+C). Cerrar la ventana la oculta en la bandeja; usa **Salir** para cerrar la aplicación.
- Sesión WebView2 persistente en %APPDATA%\WidgetWebsite\webview-data; configuración en %APPDATA%\WidgetWebsite\config.json. Se conserva el nombre interno anterior de la carpeta para mantener los datos de instalaciones previas de Floatly.
- Interfaz disponible en portugués de Brasil, inglés y español. Cambia el idioma en **Configuración → Idioma**.
- Una pequeña pestaña en la parte superior central permite arrastrar el widget. Las zonas de interacción de los bordes siguen activas, pero son transparentes y no dibujan un marco alrededor del contenido. El tamaño y la posición se pueden cambiar en la configuración.
- El tamaño predeterminado es 350 × 550 px y el zoom predeterminado del navegador integrado es del 80 %; ambos se pueden cambiar en la configuración.
- URL, tamaño, posición, bloqueo del movimiento, mantener siempre visible, visibilidad en la barra de tareas, zoom, esquinas redondeadas y actualización automática configurables.
- Los enlaces externos se abren en el navegador predeterminado. Los dominios de autenticación reconocidos se abren en una ventana secundaria con una barra de direcciones sencilla y la misma carpeta de datos de WebView2. La ventana intenta cerrarse cuando el inicio de sesión vuelve al sitio configurado.
- Flujo de compilación para Windows x64 e instalador NSIS.
- El paquete MSIX x64 para Microsoft Store se genera mediante un workflow específico de GitHub Actions; consulta [MICROSOFT_STORE_READINESS.md](MICROSOFT_STORE_READINESS.md) para ver la lista de publicación.

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

Floatly se prepara para la Store como paquete **MSIX x64**. Sigue la [guía paso a paso en portugués](GUIA_PUBLICACAO_MICROSOFT_STORE.md) y utiliza el [borrador multilingüe de la ficha](MICROSOFT_STORE_LISTING_DRAFT.md). Este paquete requiere Windows 10 versión 2004 (compilación 19041) o posterior; la versión convencional NSIS se distribuye por separado.

El workflow **Build MSIX for Microsoft Store** se ejecuta automáticamente cuando hay cambios en la rama `main` y utiliza la identidad del paquete ya confirmada en Partner Center. Genera un archivo `.msix` sin firma. Esto es intencional: Microsoft firma el MSIX después de la aprobación. Esta ruta no requiere certificado de firma de código, archivo PFX ni secretos de firma.

## Idioma de la interfaz

Abre **Configuración → Idioma** y elige Portugués (Brasil), English o Español. La selección se guarda en %APPDATA%\WidgetWebsite\config.json y traduce la ventana de configuración, el menú de la bandeja del sistema, la barra de inicio de sesión y los mensajes de conexión propios del widget. El contenido del sitio elegido seguirá usando el idioma que proporcione el propio sitio.

## Firma opcional del instalador independiente (no se usa en Store)

El workflow del instalador NSIS independiente admite firma Authenticode opcional. Esto es distinto del paquete MSIX de Microsoft Store, que no utiliza un certificado proporcionado por el desarrollador. Para firmar el instalador NSIS distribuido fuera de Store, consigue un **certificado de firma de código de Windows de confianza** y crea estos secretos del repositorio en **Settings → Secrets and variables → Actions**:

- WINDOWS_PFX_BASE64: archivo de certificado PFX codificado en Base64.
- WINDOWS_PFX_PASSWORD: contraseña del archivo PFX.

No guardes una clave privada en el repositorio ni en este README. La compilación se ejecuta sin acceso al certificado de firma. Un trabajo independiente usa `signtool` para firmar y verificar el instalador NSIS y el ejecutable publicado por separado con SHA-256 y una marca de tiempo; luego elimina el PFX temporal. El ejecutable publicado por separado está firmado, pero la copia que ya está incluida dentro del instalador se empaquetó sin firma. Sin ambos secretos, los artefactos se publican sin firma.

**Importante:** una firma válida no garantiza que Microsoft Defender SmartScreen nunca muestre un aviso. Las aplicaciones o los archivos nuevos pueden mostrar advertencias hasta que se establezca la reputación del editor y del archivo. Publicar en Microsoft Store es la forma más directa de evitar avisos de descarga de SmartScreen. Consulta la [guía de reputación de SmartScreen de Microsoft](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation) y la [guía de firma de Tauri para Windows](https://tauri.app/distribute/sign/windows/).

## Ejemplo de uso: Investing.com

Investing.com es solo un ejemplo de sitio web que puede mostrar el widget. Para usarlo, abre **Configuración → URL del sitio** e introduce `https://br.investing.com/economic-calendar`. El widget no es exclusivo de este servicio: puedes configurar cualquier sitio HTTP o HTTPS.

Para los sitios que requieren inicio de sesión, utiliza el flujo de autenticación proporcionado por el propio sitio. El menú de la bandeja permite iniciar sesión o configurar el sitio web.

La aplicación no lee, copia ni almacena contraseñas. Se conserva el agente de usuario predeterminado de WebView2.

## Diagnóstico de bloqueos

- Los registros se guardan normalmente en `%APPDATA%\WidgetWebsite\diagnostics.log`. Las marcas de tiempo están en UTC; los archivos rotan al alcanzar aproximadamente 1 MiB y se conservan hasta tres registros anteriores. Los registros no se envían automáticamente al desarrollador.
- La página envía una señal de actividad de JavaScript/WebView2 cada cinco segundos. Si no llega durante 20 segundos, el monitor registra un aviso; no reinicia ni cierra la aplicación y omite los periodos en que la ventana está minimizada u oculta. La ausencia de procesos msedgewebview2.exe descendientes es una pista, no una prueba de la causa.
- Los registros incluyen el inicio, la navegación, los errores de inicialización, las acciones de movimiento y la recuperación de actividad. Las URL registradas no incluyen credenciales, parámetros de consulta ni fragmentos; la ruta se limita a sus dos primeros segmentos y se enmascaran los segmentos largos o parecidos a identificadores. Revisa el archivo antes de compartirlo.
- En una compilación gráfica de Windows puede no haber terminal. El archivo de registro es la fuente de diagnóstico fiable; ábrelo con el Bloc de notas después de reproducir el bloqueo.
- Estos diagnósticos no pueden demostrar que una aplicación externa causó el bloqueo. Tampoco pueden registrar más información si el proceso o todos sus hilos se terminan abruptamente.

## Privacidad y soporte

Consulta la [Política de privacidad de Floatly](PRIVACY_POLICY.md), disponible en portugués, inglés y español, y la página de [Soporte](SUPPORT.md). La política identifica al responsable individual del proyecto y el canal de contacto para solicitudes de privacidad.

## Privacidad y limitaciones

- Sin telemetría. La navegación principal se limita al sitio configurado y a recursos de autenticación reconocidos; los enlaces a otros dominios se abren en el navegador predeterminado. El sitio remoto solo recibe permiso para enviar la señal autenticada de actividad.
- Si el JSON de configuración está completamente dañado, el archivo original se conserva como `config.json.bad-*`. Si hay campos individuales que reparar, Floatly crea copias `config.json.repaired-*` antes de corregir los valores y mantiene hasta tres copias recientes de reparación. **Restaurar tamaño predeterminado** solo cambia las dimensiones a 350 × 550 px; conserva las demás preferencias.
- CI comprueba la alineación de versiones de Tauri, el manifiesto de comandos y las capabilities; ejecuta `npm ci`, la verificación de tipos, `npm audit`, `cargo fmt --check`, `cargo clippy`, `cargo audit` y `cargo test`, y después compila el instalador de Windows.
- La ventana secundaria de inicio de sesión comparte la carpeta de datos de WebView2 e incluye una barra de direcciones inyectada. Los dominios no autorizados se abren en el navegador predeterminado. El cierre automático depende de que el proveedor vuelva al sitio configurado; algunos inicios de sesión sociales pueden requerir cerrar la ventana manualmente.
- El zoom usa el control nativo de WebView2. El recorte de las esquinas puede variar en Windows 10.
- Las comprobaciones de red muestran un panel propio antes de la navegación principal y supervisan la conectividad mientras el sitio está abierto. El envoltorio predeterminado de Tauri no puede reemplazar todos los errores de navegación de Chromium en todos los escenarios.


## Desinstalación y datos guardados

El desinstalador NSIS ofrece la casilla **Delete the application data**. Si se marca, elimina `%APPDATA%\WidgetWebsite` (configuración, registros y sesión de WebView2) y `%LOCALAPPDATA%\WidgetWebsite` si existe. Si se deja desmarcada, estos datos se conservan para una reinstalación posterior. La entrada de inicio automático de Windows siempre se elimina al desinstalar la aplicación.

## Estructura

- src-tauri/src: configuración, ventana, comandos, diagnósticos y bandeja del sistema en Rust.
- src: inicialización TypeScript e interfaz de configuración.
- icons/widget.svg: fuente vectorial utilizada para generar los iconos de Windows.
