# Widget Website — Windows

**Idiomas:** [Português (Brasil)](README.md) · [English](README.en.md) · Español

Widget compacto para Windows que muestra el calendario económico de Investing.com en una ventana WebView2 sin bordes. Desarrollado con Tauri 2 para Windows 10/11 x64.

## Funciones

- Icono en la bandeja del sistema, instancia única y atajo global (predeterminado: CTRL+ALT+C). Cerrar la ventana la oculta en la bandeja; usa **Salir** para cerrar la aplicación.
- Sesión WebView2 persistente en %APPDATA%\WidgetWebsite\webview-data; configuración en %APPDATA%\WidgetWebsite\config.json.
- Interfaz disponible en portugués de Brasil, inglés y español. Cambia el idioma en **Configuración → Idioma**.
- Arrastra el widget desde el borde azul bien visible que rodea el contenido (parte superior, inferior o cualquiera de los lados). El tamaño y la posición se pueden configurar.
- URL, tamaño, posición, bloqueo del movimiento, mantener siempre visible, visibilidad en la barra de tareas, opacidad, zoom, esquinas redondeadas y actualización automática configurables.
- Los enlaces externos se abren en el navegador predeterminado. Los dominios de autenticación reconocidos se abren en una ventana secundaria con una barra de direcciones sencilla y la misma carpeta de datos de WebView2. La ventana intenta cerrarse cuando el inicio de sesión vuelve al sitio configurado.
- Flujo de compilación para Windows x64 e instalador NSIS.

## Requisitos

- Windows 10/11 x64, Node.js y npm.
- Rust estable 1.90 o posterior.
- Microsoft Edge WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/
- Para compilar localmente: Visual Studio Build Tools con Desarrollo para escritorio con C++ y Windows SDK.

## Desarrollo

Ejecuta estos comandos en PowerShell (es necesario generar los iconos antes de la primera compilación):

    npm install
    npm run icons
    npm run tauri dev

## Compilar / empaquetar

    npm install
    npm run icons
    npm run tauri build

El instalador se genera en src-tauri/target/release/bundle/nsis y el ejecutable en src-tauri/target/release. GitHub Actions compila el paquete para Windows y lo publica como artefacto widget-website-windows-x64.

## Idioma de la interfaz

Abre **Configuración → Idioma** y elige Portugués (Brasil), English o Español. La selección se guarda en %APPDATA%\WidgetWebsite\config.json y traduce la ventana de configuración, el menú de la bandeja del sistema, la barra de inicio de sesión y los mensajes de conexión propios del widget. El contenido de Investing.com seguirá usando el idioma que proporcione el propio sitio.

## Firma digital y avisos de seguridad de Windows

El flujo de GitHub Actions admite firma Authenticode opcional. Para activarla, consigue un **certificado de firma de código de Windows de confianza** y crea estos secretos del repositorio en **Settings → Secrets and variables → Actions**:

- WINDOWS_PFX_BASE64: archivo de certificado PFX codificado en Base64.
- WINDOWS_PFX_PASSWORD: contraseña del archivo PFX.

No guardes una clave privada en el repositorio ni en este README. Durante la compilación, el flujo importa el certificado, firma con SHA-256 y una marca de tiempo y elimina el archivo PFX temporal. Sin ambos secretos, CI avisa que el artefacto no está firmado y continúa la compilación.

**Importante:** una firma válida no garantiza que Microsoft Defender SmartScreen nunca muestre un aviso. Las aplicaciones o los archivos nuevos pueden mostrar advertencias hasta que se establezca la reputación del editor y del archivo. Publicar en Microsoft Store es la forma más directa de evitar avisos de descarga de SmartScreen. Consulta la [guía de reputación de SmartScreen de Microsoft](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation) y la [guía de firma de Tauri para Windows](https://tauri.app/distribute/sign/windows/).

## Primer inicio de sesión

1. Abre el widget y espera a que cargue el calendario.
2. El menú de la bandeja incluye **Iniciar sesión**. Usar correo y contraseña suele ser la opción más sencilla.
3. Reinicia la aplicación para comprobar que la sesión persiste. Algunos filtros pueden estar vinculados a la cuenta y quizá haya que configurarlos de nuevo.
4. Si la sesión caduca, vuelve a abrir el inicio de sesión desde el menú de la bandeja y autentícate otra vez.

La aplicación no lee, copia ni almacena contraseñas. Se conserva el agente de usuario predeterminado de WebView2.

## Diagnóstico de bloqueos

- Los registros se guardan en %APPDATA%\WidgetWebsite\diagnostics.log (normalmente C:\Users\<usuario>\AppData\Roaming\WidgetWebsite\diagnostics.log). Las marcas de tiempo están en UTC. El archivo anterior se rota a diagnostics.log.1 al alcanzar aproximadamente 1 MB.
- La página envía una señal de actividad de JavaScript/WebView2 cada cinco segundos. Si no llega durante 20 segundos, el monitor registra un aviso, comprueba si la ventana nativa responde y busca procesos msedgewebview2.exe descendientes. La ausencia de estos procesos es una pista de que WebView2 pudo cerrarse, no una prueba de la causa.
- Los registros incluyen el inicio, la navegación, los errores de inicialización, las acciones de movimiento y la recuperación de actividad. Las URL se registran sin parámetros de consulta ni fragmentos para reducir el riesgo de guardar tokens de sesión.
- En una compilación gráfica de Windows puede no haber terminal. El archivo de registro es la fuente de diagnóstico fiable; ábrelo con el Bloc de notas después de reproducir el bloqueo.
- Estos diagnósticos no pueden demostrar que una aplicación externa causó el bloqueo. Tampoco pueden registrar más información si el proceso o todos sus hilos se terminan abruptamente.

## Privacidad y limitaciones

- Sin telemetría. La navegación principal se limita al sitio configurado y a recursos de autenticación reconocidos; los demás enlaces HTTP/HTTPS se abren en el navegador predeterminado.
- Si el archivo de configuración está dañado, se reemplaza por valores predeterminados. **Restaurar tamaño predeterminado** solo cambia las dimensiones a 500 × 700 px; conserva las demás preferencias.
- CI ejecuta pruebas automáticas de valores predeterminados, normalización, validación de URL, compatibilidad con configuraciones antiguas sin idioma y recuperación de archivos dañados; también compila el instalador de Windows.
- La ventana secundaria de inicio de sesión comparte la carpeta de datos de WebView2 e incluye una barra de direcciones inyectada. Los dominios no autorizados se abren en el navegador predeterminado. El cierre automático depende de que el proveedor vuelva al sitio configurado; algunos inicios de sesión sociales pueden requerir cerrar la ventana manualmente.
- El zoom usa el control nativo de WebView2. La opacidad y las esquinas redondeadas usan CSS/recorte nativo; el aspecto de las esquinas puede variar en Windows 10.
- Las comprobaciones de red muestran un panel propio antes de la navegación principal y supervisan la conectividad mientras el sitio está abierto. El envoltorio predeterminado de Tauri no puede reemplazar todos los errores de navegación de Chromium en todos los escenarios.
- Es necesario probar el inicio de sesión real, los filtros guardados, los conflictos de atajos, escalas del 125 %/150 %, varios monitores, cambios de DPI y consumo de memoria en hardware Windows.

## Estructura

- src-tauri/src: configuración, ventana, comandos, diagnósticos y bandeja del sistema en Rust.
- src: inicialización TypeScript e interfaz de configuración.
- icons/widget.svg: fuente vectorial utilizada para generar los iconos de Windows.
