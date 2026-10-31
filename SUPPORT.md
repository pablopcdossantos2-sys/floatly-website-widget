# Suporte — Floatly

O Floatly é um navegador compacto para Windows. Ele mantém um endereço HTTP ou HTTPS escolhido pelo usuário em uma janela flutuante. O projeto é mantido de forma independente por seu desenvolvedor.

## Contato

Para dúvidas, solicitações de suporte ou comunicação de problemas:

**E-mail:** [pablopcsantos@outlook.com](mailto:pablopcsantos@outlook.com)

Ao relatar um problema, informe, sempre que possível, a versão do Floatly, a edição e a versão do Windows, os passos para reproduzir a falha e o resultado esperado.

## Primeira utilização

1. Abra o Floatly.
2. Na primeira execução, selecione **Escolher site**.
3. Em **Configurações → URL do site**, informe um endereço completo começando por `https://` ou `http://`.
4. Salve a configuração. O site deverá aparecer na janela compacta.

O Floatly não vem com um site de terceiros predefinido. A compatibilidade e os fluxos de autenticação dependem do site escolhido.

## Informações úteis para diagnóstico

Em uma instalação convencional pelo instalador NSIS, os registros locais ficam normalmente em:

`%APPDATA%\WidgetWebsite\diagnostics.log`

O arquivo pode conter horários, mensagens técnicas, informações sobre navegação e caminhos locais do Windows. As URLs são parcialmente reduzidas antes de serem registradas, mas isso não elimina todo risco de informação pessoal; revise o conteúdo e remova dados que não queira compartilhar. O aplicativo não envia automaticamente os registros ao desenvolvedor. Na instalação MSIX pela Microsoft Store, o Windows pode redirecionar o armazenamento de dados para a área privada do pacote, portanto o caminho NSIS acima pode não se aplicar.

Para preservar a privacidade, não envie senhas, códigos de autenticação, cookies, tokens ou arquivos de perfil do WebView2. Descreva o problema e compartilhe somente o trecho mínimo de log necessário.

## Escopo e limitações

- O Floatly não é um navegador completo com abas e histórico.
- O usuário escolhe o site principal; nenhum serviço específico é exigido ou endossado.
- Links externos são encaminhados ao navegador padrão do Windows.
- Alguns sites podem restringir WebView2, autenticação ou determinados recursos.
- O Floatly não oferece suporte técnico aos sites de terceiros acessados pelo usuário.

---

# Support — Floatly

Floatly is a compact browser for Windows. It keeps an HTTP or HTTPS address selected by the user in a floating window. The project is independently maintained by its developer.

## Contact

For questions, support requests, or bug reports:

**Email:** [pablopcsantos@outlook.com](mailto:pablopcsantos@outlook.com)

When reporting an issue, include the Floatly version, Windows edition and version, steps to reproduce the problem, and the expected result whenever possible.

## First use

1. Open Floatly.
2. On first launch, select **Choose website**.
3. Under **Settings → Website URL**, enter a full address beginning with `https://` or `http://`.
4. Save the setting. The website should appear in the compact window.

Floatly does not preconfigure a third-party website. Compatibility and sign-in flows depend on the website you choose.

## Useful diagnostic information

In a conventional NSIS installation, local logs are normally stored at:

`%APPDATA%\WidgetWebsite\diagnostics.log`

The file may contain timestamps, technical messages, navigation information, and local Windows paths. URLs are partially redacted before logging, but this does not remove every privacy risk; review the file and remove anything you do not want to share. The application does not automatically send logs to the developer. In a Microsoft Store MSIX installation, Windows may redirect application data to the package's private storage area, so the NSIS path above may not apply.

For privacy, do not send passwords, authentication codes, cookies, tokens, or WebView2 profile files. Describe the issue and share only the minimum log excerpt needed.

## Scope and limitations

- Floatly is not a full browser with tabs and history.
- Users choose the primary website; no specific service is required or endorsed.
- External links are sent to the Windows default browser.
- Some websites may restrict WebView2, sign-in, or specific features.
- Floatly does not provide technical support for third-party websites accessed by users.

---

# Soporte — Floatly

Floatly es un navegador compacto para Windows. Mantiene una dirección HTTP o HTTPS elegida por el usuario en una ventana flotante. El proyecto es mantenido de forma independiente por su desarrollador.

## Contacto

Para preguntas, solicitudes de soporte o informes de errores:

**Correo electrónico:** [pablopcsantos@outlook.com](mailto:pablopcsantos@outlook.com)

Al informar de un problema, incluye, siempre que sea posible, la versión de Floatly, la edición y versión de Windows, los pasos para reproducir el fallo y el resultado esperado.

## Primer uso

1. Abre Floatly.
2. En el primer inicio, selecciona **Elegir sitio**.
3. En **Configuración → URL del sitio**, introduce una dirección completa que empiece por `https://` o `http://`.
4. Guarda la configuración. El sitio debería aparecer en la ventana compacta.

Floatly no incluye un sitio web de terceros preconfigurado. La compatibilidad y los flujos de autenticación dependen del sitio elegido.

## Información útil para el diagnóstico

En una instalación convencional mediante el instalador NSIS, los registros locales suelen guardarse en:

`%APPDATA%\WidgetWebsite\diagnostics.log`

El archivo puede contener fechas, mensajes técnicos, información de navegación y rutas locales de Windows. Las URL se redactan parcialmente antes de registrarse, pero esto no elimina todos los riesgos de privacidad; revisa el archivo y elimina cualquier dato que no quieras compartir. La aplicación no envía automáticamente los registros al desarrollador. En una instalación MSIX desde Microsoft Store, Windows puede redirigir los datos de la aplicación al almacenamiento privado del paquete, por lo que la ruta NSIS anterior puede no ser válida.

Por privacidad, no envíes contraseñas, códigos de autenticación, cookies, tokens ni archivos del perfil de WebView2. Describe el problema y comparte solo el fragmento mínimo del registro que sea necesario.

## Alcance y limitaciones

- Floatly no es un navegador completo con pestañas e historial.
- El usuario elige el sitio principal; no se requiere ni se respalda ningún servicio específico.
- Los enlaces externos se envían al navegador predeterminado de Windows.
- Algunos sitios pueden restringir WebView2, el inicio de sesión o determinadas funciones.
- Floatly no ofrece soporte técnico para los sitios web de terceros que utilice el usuario.
